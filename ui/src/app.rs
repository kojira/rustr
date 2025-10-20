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
use crate::settings::SettingsView;
use crate::i18n::I18n;

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

/// サイドバーのタブ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SidebarTab {
    Public,
    DMs,
}

/// メインアプリケーション
pub struct NostrApp {
    state: AppState,
    onboarding: Onboarding,
    timeline: Timeline,
    composer: Composer,
    settings: SettingsView,
    i18n: I18n,
    
    // Core (Rc<RefCell<>>でUIから変更可能にする)
    core: Rc<RefCell<Option<CoreHandle>>>,
    storage: Rc<RefCell<Option<Arc<IndexedDbStorage>>>>,
    
    // UI状態
    show_composer: bool,
    show_settings: bool,
    show_channel_create: bool,
    channel_name_input: String,
    channel_about_input: String,
    sidebar_tab: SidebarTab,  // Public / DMs タブ
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
            settings: SettingsView::new(),
            i18n: I18n::default(),
            core: Rc::new(RefCell::new(None)),
            storage: Rc::new(RefCell::new(None)),
            show_composer: false,
            show_settings: false,
            show_channel_create: false,
            channel_name_input: String::new(),
            channel_about_input: String::new(),
            sidebar_tab: SidebarTab::Public,
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
            "wss://x.kojira.io".to_string(),
            "wss://yabu.me".to_string(),
            "wss://r.kojira.io".to_string(),
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
        // try_borrow_mut()を使って、借用できない場合はスキップ
        if let Ok(mut core_borrow) = self.core.try_borrow_mut() {
            if let Some(core) = core_borrow.as_mut() {
                // poll_eventsを実行
                let events = core.poll_events(50);
                for event in events {
                    self.timeline.add_event(event);
                }
            }
        }
        
        // 非同期でtick()を実行（借用できない場合はスキップ）
        let core_ref = self.core.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(mut core_borrow) = core_ref.try_borrow_mut() {
                if let Some(core) = core_borrow.as_mut() {
                    if let Err(e) = core.tick().await {
                        log::error!("Tick error: {:?}", e);
                    }
                }
            }
        });
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
    
    #[cfg(feature = "debug-test")]
    pub fn debug_create_channel(&mut self, name: String, about: String) {
        let core_ref = self.core.clone();
        
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(mut core_borrow) = core_ref.try_borrow_mut() {
                if let Some(core) = core_borrow.as_mut() {
                    match core.create_channel(&name, &about, "").await {
                        Ok(id) => {
                            log::info!("✅ Channel created: {}", id);
                            // チャンネルIDをローカルストレージに保存
                            if let Some(window) = web_sys::window() {
                                if let Ok(Some(storage)) = window.local_storage() {
                                    let _ = storage.set_item("debug_channel_id", &id);
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to create channel: {:?}", e);
                        }
                    }
                }
            }
        });
    }
    
    #[cfg(feature = "debug-test")]
    pub fn debug_get_channel_id(&self) -> Option<String> {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(id)) = storage.get_item("debug_channel_id") {
                    return Some(id);
                }
            }
        }
        None
    }
    
    #[cfg(feature = "debug-test")]
    fn execute_debug_step(&mut self, step: &crate::debug_test::TestStep) {
        use crate::debug_test::TestStep;
        
        match step {
            TestStep::Idle => {
                log::info!("🧪 Starting debug test scenario...");
                self.debug_test.advance_step();
            }
            
            TestStep::OnboardingCreateKey => {
                log::info!("🧪 Simulating: Create new key");
                self.debug_skip_onboarding();
                self.debug_test.advance_step();
            }
            
            TestStep::TransitionToMain => {
                log::info!("🧪 Verifying: Main screen loaded");
                if self.is_main_screen() {
                    log::info!("✅ Main screen is active");
                    self.debug_test.advance_step();
                } else {
                    log::warn!("⏳ Waiting for main screen...");
                }
            }
            
            TestStep::CreateChannel { name, about } => {
                // 既存のチャンネルIDがあれば再利用
                if let Some(existing_id) = self.debug_get_channel_id() {
                    log::info!("♻️  Reusing existing channel: {}", existing_id);
                    self.debug_test.advance_step();
                } else {
                    log::info!("🧪 Creating new channel: {}", name);
                    self.debug_create_channel(name.clone(), about.clone());
                    self.debug_test.wait_frames = 120; // 2秒待機
                    self.debug_test.advance_step();
                }
            }
            
            TestStep::WaitForChannelCreation => {
                // OKレスポンスを待つ（実際にはevent_bufferから取得すべき）
                log::info!("⏳ Waiting for channel creation...");
                // 簡易実装: 一定時間待機後に次へ
                self.debug_test.advance_step();
            }
            
            TestStep::OpenChannel { channel_id } => {
                // ローカルストレージからチャンネルIDを取得
                let actual_channel_id = if channel_id.is_empty() {
                    self.debug_get_channel_id().unwrap_or_default()
                } else {
                    channel_id.clone()
                };
                
                if !actual_channel_id.is_empty() {
                    log::info!("🧪 Opening channel: {}", actual_channel_id);
                    self.debug_open_channel(actual_channel_id);
                    self.debug_test.advance_step();
                } else {
                    log::warn!("⏳ Waiting for channel ID...");
                }
            }
            
            TestStep::SendMessage { content } => {
                log::info!("🧪 Sending message: {}", content);
                self.debug_send_message(content.clone());
                self.debug_test.advance_step();
            }
            
            TestStep::VerifyTimeline => {
                log::info!("🧪 Verifying timeline...");
                let event_count = self.debug_get_timeline_count();
                log::info!("📊 Timeline has {} events", event_count);
                self.debug_test.wait_frames = 180;
                self.debug_test.advance_step();
            }
            
            TestStep::OpenDm { peer } => {
                log::info!("🧪 Opening DM with: {}", peer);
                self.debug_open_dm(peer.clone());
                self.debug_test.advance_step();
            }
            
            TestStep::SendDm { content } => {
                log::info!("🧪 Sending DM: {}", content);
                self.debug_send_message(content.clone());
                self.debug_test.advance_step();
            }
            
            TestStep::Completed => {
                // 何もしない
            }
        }
    }
}

impl eframe::App for NostrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // デバッグテストの実行
        #[cfg(feature = "debug-test")]
        {
            let should_run = self.debug_test.is_enabled();
            if should_run {
                // debug_testの状態を取得してからappを操作
                let current_step = self.debug_test.current_step().clone();
                let frame_counter = self.debug_test.frame_counter;
                let wait_frames = self.debug_test.wait_frames;
                
                // フレームカウンターを更新
                self.debug_test.frame_counter += 1;
                
                // 待機フレーム数に達したらステップを実行
                if frame_counter >= wait_frames {
                    self.execute_debug_step(&current_step);
                }
                
                // デバッグ情報を画面上部に表示
                let status_text = self.debug_test.get_status_text();
                egui::TopBottomPanel::top("debug_test_status").show(ctx, |ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 200, 0),
                        status_text
                    );
                });
            }
        }
        
        match self.state {
            AppState::Onboarding => {
                // オンボーディング画面
                egui::CentralPanel::default().show(ctx, |ui| {
                    if let Some(result) = self.onboarding.show(ui, &self.i18n) {
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
                crate::emoji_label::emoji_heading(ui, "🦀 Rustr");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(self.i18n.button_settings()).clicked() {
                        self.show_settings = !self.show_settings;
                    }
                });
            });
        });
        
        // 左サイドバー（チャンネル/DM一覧）
        egui::SidePanel::left("sidebar")
            .default_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                // タブ切り替え
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.sidebar_tab, SidebarTab::Public, self.i18n.button_public());
                    ui.selectable_value(&mut self.sidebar_tab, SidebarTab::DMs, self.i18n.button_dms());
                });
                
                ui.separator();
                
                // タブに応じた一覧を表示
                match self.sidebar_tab {
                    SidebarTab::Public => {
                        self.show_channel_list(ui);
                    }
                    SidebarTab::DMs => {
                        self.show_dm_list(ui);
                    }
                }
            });
        
        // 設定モーダル
        if self.show_settings {
            egui::Window::new("⚙️ 設定")
                .collapsible(false)
                .resizable(true)
                .default_width(500.0)
                .show(ctx, |ui| {
                    self.settings.show(ctx, ui, &mut self.i18n);
                    
                    ui.add_space(10.0);
                    if ui.button(self.i18n.button_close()).clicked() {
                        self.show_settings = false;
                    }
                });
        }
        
        // チャンネル作成モーダル
        if self.show_channel_create {
            self.show_channel_create_dialog(ctx);
        }
        
        // コンポーザー（下部）
        if self.show_composer {
            egui::TopBottomPanel::bottom("composer").show(ctx, |ui| {
                if let Some(content) = self.composer.show(ui, &self.i18n) {
                    self.send_message(content);
                }
                
                if ui.button(self.i18n.button_close()).clicked() {
                    self.show_composer = false;
                }
            });
        } else {
            // コンポーザーを開くボタン
            egui::TopBottomPanel::bottom("composer_button").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(self.i18n.button_new_post()).clicked() {
                        self.show_composer = true;
                    }
                    
                    // 現在のチャンネル/DM表示
                    if let Some(channel) = &self.current_channel {
                        crate::emoji_label::emoji_label(ui, format!("📢 {}", channel));
                    } else if let Some(peer) = &self.current_dm_peer {
                        crate::emoji_label::emoji_label(ui, format!("💬 {}", peer));
                    }
                });
            });
        }
        
        // タイムライン（中央）
        egui::CentralPanel::default().show(ctx, |ui| {
            self.timeline.show(ui, &self.i18n);
        });
    }
    
    /// チャンネル作成ダイアログ
    fn show_channel_create_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new(self.i18n.channel_create_title())
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    crate::emoji_label::emoji_label(ui, self.i18n.channel_name_label());
                    ui.text_edit_singleline(&mut self.channel_name_input);
                    
                    ui.add_space(10.0);
                    
                    crate::emoji_label::emoji_label(ui, self.i18n.channel_about_label());
                    ui.text_edit_multiline(&mut self.channel_about_input);
                    
                    ui.add_space(20.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button(self.i18n.button_cancel()).clicked() {
                            self.show_channel_create = false;
                            self.channel_name_input.clear();
                            self.channel_about_input.clear();
                        }
                        
                        if ui.button(self.i18n.button_create()).clicked() {
                            if !self.channel_name_input.is_empty() {
                                self.create_new_channel();
                            }
                        }
                    });
                });
            });
    }
    
    /// 新しいチャンネルを作成
    fn create_new_channel(&mut self) {
        let name = self.channel_name_input.clone();
        let about = self.channel_about_input.clone();
        let core_ref = self.core.clone();
        
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(core) = core_ref.borrow_mut().as_mut() {
                match core.create_channel(&name, &about, "").await {
                    Ok(channel_id) => {
                        log::info!("✅ Channel created: {}", channel_id);
                        // チャンネルを開く
                        if let Err(e) = core.open_channel(&channel_id).await {
                            log::error!("Failed to open channel: {:?}", e);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to create channel: {:?}", e);
                    }
                }
            }
        });
        
        self.show_channel_create = false;
        self.channel_name_input.clear();
        self.channel_about_input.clear();
    }
    
    /// チャンネル一覧を表示
    fn show_channel_list(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 新規作成ボタン
            if ui.button(self.i18n.channel_create_button()).clicked() {
                self.show_channel_create = true;
            }
            
            ui.separator();
            
            // TODO: ストレージからチャンネル一覧を取得して表示
            // 現在は仮実装
            ui.label(self.i18n.channel_list_empty());
        });
    }
    
    /// DM一覧を表示
    fn show_dm_list(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // TODO: ストレージからDM一覧を取得して表示
            // 現在は仮実装
            ui.label(self.i18n.dm_list_empty());
        });
    }
}
