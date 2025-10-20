use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;
use web_sys::{WebSocket, MessageEvent, ErrorEvent, CloseEvent};
use wasm_bindgen::{JsCast, closure::Closure};
use serde_json;

use crate::error::{Result, CoreError};

/// 接続状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

/// 指数バックオフ管理
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    current_delay: u32,
    max_delay: u32,
    min_delay: u32,
}

impl ExponentialBackoff {
    pub fn new() -> Self {
        Self {
            current_delay: 1,
            max_delay: 60,
            min_delay: 1,
        }
    }

    pub fn next_delay(&mut self) -> u32 {
        let delay = self.current_delay;
        self.current_delay = (self.current_delay * 2).min(self.max_delay);
        delay
    }

    pub fn reset(&mut self) {
        self.current_delay = self.min_delay;
    }
}

/// Relay接続
pub struct RelayConnection {
    pub url: String,
    ws: Option<WebSocket>,
    state: Rc<RefCell<ConnectionState>>,
    backoff: ExponentialBackoff,
    subscriptions: HashMap<String, String>, // sub_id -> filter_json
    eose_received: HashSet<String>,
    last_connect_attempt: f64,
    message_queue: Rc<RefCell<Vec<RelayMessage>>>,
    // クロージャを保持してドロップされないようにする
    _on_open: Option<Closure<dyn FnMut()>>,
    _on_message: Option<Closure<dyn FnMut(MessageEvent)>>,
    _on_error: Option<Closure<dyn FnMut(ErrorEvent)>>,
    _on_close: Option<Closure<dyn FnMut(CloseEvent)>>,
}

impl RelayConnection {
    pub fn new(url: String) -> Self {
        Self {
            url,
            ws: None,
            state: Rc::new(RefCell::new(ConnectionState::Disconnected)),
            backoff: ExponentialBackoff::new(),
            subscriptions: HashMap::new(),
            eose_received: HashSet::new(),
            last_connect_attempt: 0.0,
            message_queue: Rc::new(RefCell::new(Vec::new())),
            _on_open: None,
            _on_message: None,
            _on_error: None,
            _on_close: None,
        }
    }

    pub fn state(&self) -> ConnectionState {
        *self.state.borrow()
    }

    pub fn is_connected(&self) -> bool {
        *self.state.borrow() == ConnectionState::Connected
    }

    /// 接続試行
    pub async fn connect(&mut self) -> Result<()> {
        let current_state = *self.state.borrow();
        if current_state == ConnectionState::Connecting || current_state == ConnectionState::Connected {
            return Ok(());
        }

        *self.state.borrow_mut() = ConnectionState::Connecting;
        self.last_connect_attempt = now();

        let ws = WebSocket::new(&self.url)?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        // 状態を共有
        let state = self.state.clone();
        let message_queue = self.message_queue.clone();
        let url = self.url.clone();

        // onopen ハンドラー
        {
            let state = state.clone();
            let url = url.clone();
            let on_open = Closure::wrap(Box::new(move || {
                log::info!("WebSocket connected to {}", url);
                *state.borrow_mut() = ConnectionState::Connected;
            }) as Box<dyn FnMut()>);
            ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
            self._on_open = Some(on_open);
        }

        // onmessage ハンドラー
        {
            let message_queue = message_queue.clone();
            let on_message = Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Some(text) = event.data().as_string() {
                    match RelayMessage::parse(&text) {
                        Ok(msg) => {
                            message_queue.borrow_mut().push(msg);
                        }
                        Err(e) => {
                            log::warn!("Failed to parse relay message: {:?}", e);
                        }
                    }
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            self._on_message = Some(on_message);
        }

        // onerror ハンドラー
        {
            let url = url.clone();
            let on_error = Closure::wrap(Box::new(move |_event: ErrorEvent| {
                log::error!("WebSocket error on {}", url);
            }) as Box<dyn FnMut(ErrorEvent)>);
            ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
            self._on_error = Some(on_error);
        }

        // onclose ハンドラー
        {
            let state = state.clone();
            let url = url.clone();
            let on_close = Closure::wrap(Box::new(move |_event: CloseEvent| {
                log::info!("WebSocket closed for {}", url);
                *state.borrow_mut() = ConnectionState::Disconnected;
            }) as Box<dyn FnMut(CloseEvent)>);
            ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
            self._on_close = Some(on_close);
        }

        self.ws = Some(ws);
        Ok(())
    }

    /// メッセージ送信
    pub async fn send(&self, msg: &str) -> Result<()> {
        if let Some(ws) = &self.ws {
            if *self.state.borrow() == ConnectionState::Connected {
                ws.send_with_str(msg)?;
            }
        }
        Ok(())
    }

    /// 購読追加
    pub fn add_subscription(&mut self, sub_id: String, filter_json: String) {
        self.subscriptions.insert(sub_id, filter_json);
    }

    /// EOSE受信記録
    pub fn mark_eose(&mut self, sub_id: &str) {
        self.eose_received.insert(sub_id.to_string());
    }

    /// EOSE受信済みか
    pub fn has_eose(&self, sub_id: &str) -> bool {
        self.eose_received.contains(sub_id)
    }

    /// 受信メッセージを取得（キューをクリア）
    pub fn drain_messages(&mut self) -> Vec<RelayMessage> {
        self.message_queue.borrow_mut().drain(..).collect()
    }

    /// 受信メッセージ数
    pub fn message_count(&self) -> usize {
        self.message_queue.borrow().len()
    }

    /// 再接続が必要か
    pub fn needs_reconnect(&self) -> bool {
        if *self.state.borrow() == ConnectionState::Connected {
            return false;
        }

        let elapsed = now() - self.last_connect_attempt;
        let delay = self.backoff.current_delay as f64;
        elapsed >= delay
    }

    /// 再接続試行
    pub async fn reconnect_if_needed(&mut self) -> Result<()> {
        if self.needs_reconnect() {
            log::info!("Reconnecting to {}", self.url);
            self.connect().await?;
        }
        Ok(())
    }

    /// 接続成功時の処理
    pub fn on_open(&mut self) {
        log::info!("Connected to {}", self.url);
        *self.state.borrow_mut() = ConnectionState::Connected;
        self.backoff.reset();
    }

    /// 切断時の処理
    pub fn on_close(&mut self) {
        log::info!("Disconnected from {}", self.url);
        *self.state.borrow_mut() = ConnectionState::Disconnected;
        self.ws = None;
    }

    /// エラー時の処理
    pub fn on_error(&mut self, error: &str) {
        log::error!("WebSocket error on {}: {}", self.url, error);
        *self.state.borrow_mut() = ConnectionState::Disconnected;
    }
}

/// 現在時刻（秒）
fn now() -> f64 {
    js_sys::Date::now() / 1000.0
}

/// Relayメッセージ型
#[derive(Debug, Clone)]
pub enum RelayMessage {
    Event { sub_id: String, event_json: String },
    Eose { sub_id: String },
    Ok { event_id: String, accepted: bool, message: String },
    Notice { message: String },
}

impl RelayMessage {
    /// JSONからパース
    pub fn parse(json: &str) -> Result<Self> {
        let arr: Vec<serde_json::Value> = serde_json::from_str(json)?;
        
        if arr.is_empty() {
            return Err(CoreError::ParseError("Empty message array".to_string()));
        }

        let msg_type = arr[0].as_str().ok_or_else(|| CoreError::ParseError("Message type not a string".to_string()))?;

        match msg_type {
            "EVENT" => {
                if arr.len() < 3 {
                    return Err(CoreError::ParseError("Invalid EVENT message".to_string()));
                }
                let sub_id = arr[1].as_str().ok_or_else(|| CoreError::ParseError("sub_id not a string".to_string()))?.to_string();
                let event_json = arr[2].to_string();
                Ok(RelayMessage::Event { sub_id, event_json })
            }
            "EOSE" => {
                if arr.len() < 2 {
                    return Err(CoreError::ParseError("Invalid EOSE message".to_string()));
                }
                let sub_id = arr[1].as_str().ok_or_else(|| CoreError::ParseError("sub_id not a string".to_string()))?.to_string();
                Ok(RelayMessage::Eose { sub_id })
            }
            "OK" => {
                if arr.len() < 4 {
                    return Err(CoreError::ParseError("Invalid OK message".to_string()));
                }
                let event_id = arr[1].as_str().ok_or_else(|| CoreError::ParseError("event_id not a string".to_string()))?.to_string();
                let accepted = arr[2].as_bool().ok_or_else(|| CoreError::ParseError("accepted not a bool".to_string()))?;
                let message = arr[3].as_str().unwrap_or("").to_string();
                Ok(RelayMessage::Ok { event_id, accepted, message })
            }
            "NOTICE" => {
                if arr.len() < 2 {
                    return Err(CoreError::ParseError("Invalid NOTICE message".to_string()));
                }
                let message = arr[1].as_str().ok_or_else(|| CoreError::ParseError("message not a string".to_string()))?.to_string();
                Ok(RelayMessage::Notice { message })
            }
            _ => Err(CoreError::ParseError(format!("Unknown message type: {}", msg_type))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let mut backoff = ExponentialBackoff::new();
        assert_eq!(backoff.next_delay(), 1);
        assert_eq!(backoff.next_delay(), 2);
        assert_eq!(backoff.next_delay(), 4);
        assert_eq!(backoff.next_delay(), 8);
        
        backoff.reset();
        assert_eq!(backoff.next_delay(), 1);
    }

    #[test]
    fn test_relay_message_parse() {
        let json = r#"["EVENT","sub1",{"id":"abc","kind":1}]"#;
        let msg = RelayMessage::parse(json).unwrap();
        match msg {
            RelayMessage::Event { sub_id, .. } => assert_eq!(sub_id, "sub1"),
            _ => panic!("Expected EVENT message"),
        }

        let json = r#"["EOSE","sub1"]"#;
        let msg = RelayMessage::parse(json).unwrap();
        match msg {
            RelayMessage::Eose { sub_id } => assert_eq!(sub_id, "sub1"),
            _ => panic!("Expected EOSE message"),
        }

        let json = r#"["OK","event123",true,""]"#;
        let msg = RelayMessage::parse(json).unwrap();
        match msg {
            RelayMessage::Ok { event_id, accepted, .. } => {
                assert_eq!(event_id, "event123");
                assert!(accepted);
            }
            _ => panic!("Expected OK message"),
        }
    }
}

