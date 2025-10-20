use eframe::egui;
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;

use core::CoreHandle;
use core::storage::indexeddb::IndexedDbStorage;
use core::signer::internal::InternalSigner;
use core::signer::Signer;

use crate::timeline::Timeline;
use crate::composer::Composer;
use crate::onboarding::{Onboarding, OnboardingResult};

#[cfg(feature = "debug-test")]
use crate::debug_test::{DebugTestRunner, is_debug_test_enabled};

/// アプリケーションの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppState {
    /// 初回起動（オンボーディング）
    Onboarding,
    /// メインビュー（タイムライン + コンポーザー）
    Main,
}

/// メインアプリケーション
pub struct NostrApp {
    state: AppState,
    onboarding: Onboarding,
    timeline: Timeline,
    composer: Composer,
    
    // Core (Rc<RefCell<>>でUIから変更可能にする)
    core: Rc<RefCell<Option<CoreHandle>>>,
    storage: Rc<RefCell<Option<Arc<IndexedDbStorage>>>>,
    
    // UI状態
    show_composer: bool,
    current_channel: Option<String>,
    current_dm_peer: Option<String>,
    error_message: Option<String>,
    
    // デバッグテスト
    #[cfg(feature = "debug-test")]
    debug_test: DebugTestRunner,
}

impl NostrApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let state = AppState::Onboarding;
        
        #[cfg(feature = "debug-test")]
        let debug_test = DebugTestRunner::new(is_debug_test_enabled());
        
        #[cfg(feature = "debug-test")]
        if debug_test.is_enabled() {
            log::info!("🧪 Debug test mode enabled!");
        }
        
        Self {
            state,
            onboarding: Onboarding::new(),
            timeline: Timeline::new(),
            composer: Composer::new(),
            core: Rc::new(RefCell::new(None)),
            storage: Rc::new(RefCell::new(None)),
            show_composer: false,
            current_channel: None,
            current_dm_peer: None,
            error_message: None,
            #[cfg(feature = "debug-test")]
            debug_test,
        }
    }
    
    /// オンボーディング完了時の処理
    fn complete_onboarding(&mut self, result: OnboardingResult) {
        self.state = AppState::Main;
        
        let core_ref = self.core.clone();
        let storage_ref = self.storage.clone();
        
        // CoreHandleを初期化（非同期）
        wasm_bindgen_futures::spawn_local(async move {
            match Self::init_core_from_onboarding(result).await {
                Ok((mut core, storage)) => {
                    log::info!("Core initialized successfully");
                    
                    // Relay接続を開始
                    if let Err(e) = core.connect_all().await {
                        log::error!("Failed to connect to relays: {:?}", e);
                    }
                    
                    *core_ref.borrow_mut() = Some(core);
                    *storage_ref.borrow_mut() = Some(storage);
                }
                Err(e) => {
                    log::error!("Failed to initialize core: {:?}", e);
                }
            }
        });
        
        log::info!("Onboarding completed, transitioning to main view");
    }
    
    /// Core初期化（オンボーディング結果から）
    async fn init_core_from_onboarding(result: OnboardingResult) -> core::Result<(CoreHandle, Arc<IndexedDbStorage>)> {
        use core::storage::Storage;
        
        // Storage初期化
        let storage = Arc::new(IndexedDbStorage::init().await?);
        
        // Signerを作成
        let signer: Arc<dyn Signer> = match result {
            OnboardingResult::Nip07 => {
                use core::signer::nip07::Nip07Signer;
                Arc::new(Nip07Signer)
            }
            OnboardingResult::ImportKey { nsec, passphrase: _ } => {
                // nsecをデコードして秘密鍵バイト列に変換
                // 簡易実装: hexとして扱う
                let secret_bytes = hex::decode(&nsec)
                    .map_err(|e| core::CoreError::ParseError(format!("Invalid nsec: {}", e)))?;
                Arc::new(InternalSigner::from_secret_key(&secret_bytes)?)
            }
            OnboardingResult::CreateKey { passphrase: _ } => {
                Arc::new(InternalSigner::generate("").await?)
            }
        };
        
        // デフォルトRelay一覧
        let relay_urls = vec![
            "wss://relay.damus.io".to_string(),
            "wss://nos.lol".to_string(),
            "wss://relay.nostr.band".to_string(),
        ];
        
        // CoreHandle初期化
        let mut core = CoreHandle::init(relay_urls, storage.clone()).await?;
        core.set_signer(signer);
        
        Ok((core, storage))
    }
    
    /// チャンネルを開く
    fn open_channel(&mut self, channel_id: String) {
        self.current_channel = Some(channel_id.clone());
        self.current_dm_peer = None;
        self.timeline.load_channel(&channel_id);
        
        // CoreHandleでチャンネルを購読
        let core_ref = self.core.clone();
        let channel_id_clone = channel_id.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(core) = core_ref.borrow_mut().as_mut() {
                if let Err(e) = core.open_channel(&channel_id_clone).await {
                    log::error!("Failed to open channel: {:?}", e);
                }
            }
        });
        
        log::info!("Opened channel: {}", channel_id);
    }
    
    /// DMを開く
    fn open_dm(&mut self, peer: String) {
        self.current_dm_peer = Some(peer.clone());
        self.current_channel = None;
        self.timeline.load_dm(&peer);
        
        // CoreHandleでDMを購読
        let core_ref = self.core.clone();
        let peer_clone = peer.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(core) = core_ref.borrow_mut().as_mut() {
                if let Err(e) = core.open_dm(&peer_clone).await {
                    log::error!("Failed to open DM: {:?}", e);
                }
            }
        });
        
        log::info!("Opened DM with: {}", peer);
    }
    
    /// メッセージ送信
    fn send_message(&mut self, content: String) {
        let core_ref = self.core.clone();
        
        if let Some(channel_id) = &self.current_channel {
            let channel_id = channel_id.clone();
            log::info!("Sending to channel {}: {}", channel_id, content);
            
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(core) = core_ref.borrow_mut().as_mut() {
                    match core.send_public(&channel_id, &content).await {
                        Ok(event_id) => {
                            log::info!("Message sent: {}", event_id);
                        }
                        Err(e) => {
                            log::error!("Failed to send message: {:?}", e);
                        }
                    }
                }
            });
        } else if let Some(peer) = &self.current_dm_peer {
            let peer = peer.clone();
            log::info!("Sending DM to {}: {}", peer, content);
            
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(core) = core_ref.borrow_mut().as_mut() {
                    match core.send_dm(&peer, &content).await {
                        Ok(event_id) => {
                            log::info!("DM sent: {}", event_id);
                        }
                        Err(e) => {
                            log::error!("Failed to send DM: {:?}", e);
                        }
                    }
                }
            });
        }
        
        self.show_composer = false;
        self.composer.clear();
    }

    /// 定期処理（tick）
    fn tick(&mut self) {
        let core_ref = self.core.clone();
        
        // CoreHandleのtick()を非同期で呼び出し
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(core) = core_ref.borrow_mut().as_mut() {
                if let Err(e) = core.tick().await {
                    log::error!("Tick error: {:?}", e);
                }
            }
        });
        
        // poll_events()でUIイベントを取得してタイムラインに渡す
        if let Some(core) = self.core.borrow_mut().as_mut() {
            let events = core.poll_events(50);
            for event in events {
                self.timeline.add_event(event);
            }
        }
    }
    
    // === デバッグAPI ===
    
    #[cfg(feature = "debug-test")]
    pub fn debug_skip_onboarding(&mut self) {
        // 新規キー生成でオンボーディングをスキップ
        self.complete_onboarding(OnboardingResult::CreateKey { 
            passphrase: String::new() 
        });
    }
    
    #[cfg(feature = "debug-test")]
    pub fn is_main_screen(&self) -> bool {
        self.state == AppState::Main
    }
    
    #[cfg(feature = "debug-test")]
    pub fn debug_open_channel(&mut self, channel_id: String) {
        self.open_channel(channel_id);
    }
    
    #[cfg(feature = "debug-test")]
    pub fn debug_open_dm(&mut self, peer: String) {
        self.open_dm(peer);
    }
    
    #[cfg(feature = "debug-test")]
    pub fn debug_send_message(&mut self, content: String) {
        self.send_message(content);
    }
    
    #[cfg(feature = "debug-test")]
    pub fn debug_get_timeline_count(&self) -> usize {
        self.timeline.event_count()
    }
}

impl eframe::App for NostrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // デバッグテストの実行
        #[cfg(feature = "debug-test")]
        if self.debug_test.is_enabled() {
            self.debug_test.tick(self);
            
            // デバッグ情報を画面上部に表示
            egui::TopBottomPanel::top("debug_test_status").show(ctx, |ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 200, 0),
                    self.debug_test.get_status_text()
                );
            });
        }
        
        match self.state {
            AppState::Onboarding => {
                // オンボーディング画面
                egui::CentralPanel::default().show(ctx, |ui| {
                    if let Some(result) = self.onboarding.show(ui) {
                        self.complete_onboarding(result);
                    }
                });
            }
            AppState::Main => {
                // tick()を呼び出してメッセージ処理
                self.tick();
                
                // メインビュー
                self.show_main_view(ctx);
            }
        }
        
        // 定期的な再描画をリクエスト（アニメーション用）
        ctx.request_repaint();
    }
}

impl NostrApp {
    fn show_main_view(&mut self, ctx: &egui::Context) {
        // トップバー
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🦀 Rustr");
                
                ui.separator();
                
                if ui.button("📢 Public").clicked() {
                    self.open_channel("general".to_string());
                }
                
                if ui.button("💬 DMs").clicked() {
                    // デモ用：特定のpubkeyとのDMを開く
                    // 実装: StorageからDMスレッド一覧を取得して選択UI表示
                    log::info!("DM list feature - not yet implemented");
                    ui.label("DM一覧機能は今後実装予定");
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙").clicked() {
                        log::info!("Settings feature - not yet implemented");
                        // 設定画面: Relay管理、鍵のエクスポート等
                    }
                });
            });
        });
        
        // コンポーザー（下部）
        if self.show_composer {
            egui::TopBottomPanel::bottom("composer").show(ctx, |ui| {
                if let Some(content) = self.composer.show(ui) {
                    self.send_message(content);
                }
                
                if ui.button("✖ Close").clicked() {
                    self.show_composer = false;
                }
            });
        } else {
            // コンポーザーを開くボタン
            egui::TopBottomPanel::bottom("composer_button").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("✏ New Post").clicked() {
                        self.show_composer = true;
                    }
                    
                    // 現在のチャンネル/DM表示
                    if let Some(channel) = &self.current_channel {
                        ui.label(format!("📢 {}", channel));
                    } else if let Some(peer) = &self.current_dm_peer {
                        ui.label(format!("💬 {}", peer));
                    }
                });
            });
        }
        
        // タイムライン（中央）
        egui::CentralPanel::default().show(ctx, |ui| {
            self.timeline.show(ui);
        });
    }
}
