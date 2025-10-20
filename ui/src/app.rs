use eframe::egui;
use std::sync::Arc;

use core::CoreHandle;
use core::storage::indexeddb::IndexedDbStorage;
use core::signer::internal::InternalSigner;
use core::signer::Signer;

use crate::timeline::Timeline;
use crate::composer::Composer;
use crate::onboarding::{Onboarding, OnboardingResult};

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
    
    // Core
    core: Option<CoreHandle>,
    storage: Option<Arc<IndexedDbStorage>>,
    
    // UI状態
    show_composer: bool,
    current_channel: Option<String>,
    current_dm_peer: Option<String>,
}

impl NostrApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // TODO: Storageから鍵の有無を確認して初期状態を決定
        let state = AppState::Onboarding;
        
        Self {
            state,
            onboarding: Onboarding::new(),
            timeline: Timeline::new(),
            composer: Composer::new(),
            core: None,
            storage: None,
            show_composer: false,
            current_channel: None,
            current_dm_peer: None,
        }
    }
    
    /// オンボーディング完了時の処理
    fn complete_onboarding(&mut self, result: OnboardingResult) {
        self.state = AppState::Main;
        
        // CoreHandleを初期化（非同期）
        wasm_bindgen_futures::spawn_local(async move {
            match Self::init_core_from_onboarding(result).await {
                Ok((_core, _storage)) => {
                    log::info!("Core initialized successfully");
                    // TODO: CoreとStorageをNostrAppに設定する方法が必要
                    // 現状ではクロージャ内でselfにアクセスできない
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
        
        // Relay接続開始
        // TODO: connect処理
        
        Ok((core, storage))
    }
    
    /// チャンネルを開く
    fn open_channel(&mut self, channel_id: String) {
        self.current_channel = Some(channel_id.clone());
        self.current_dm_peer = None;
        self.timeline.load_channel(&channel_id);
        log::info!("Opened channel: {}", channel_id);
    }
    
    /// DMを開く
    fn open_dm(&mut self, peer: String) {
        self.current_dm_peer = Some(peer.clone());
        self.current_channel = None;
        self.timeline.load_dm(&peer);
        log::info!("Opened DM with: {}", peer);
    }
    
    /// メッセージ送信
    fn send_message(&mut self, content: String) {
        if let Some(channel_id) = &self.current_channel {
            log::info!("Sending to channel {}: {}", channel_id, content);
            // TODO: CoreHandle経由で送信
        } else if let Some(peer) = &self.current_dm_peer {
            log::info!("Sending DM to {}: {}", peer, content);
            // TODO: CoreHandle経由で送信
        }
        
        self.show_composer = false;
        self.composer.clear();
    }
}

impl eframe::App for NostrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                    // TODO: DM一覧を表示
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙").clicked() {
                        // TODO: 設定画面
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
