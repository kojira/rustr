//! デバッグモード用の自動テストシナリオ
//! 
//! ビルド時に `--features debug-test` を指定すると、
//! アプリ起動時に自動でテストシナリオが実行されます。

use crate::app::NostrApp;
use log;

/// テストシナリオの実行状態
#[derive(Debug, Clone, PartialEq)]
pub enum TestStep {
    /// 待機中
    Idle,
    /// オンボーディング: 新規キー生成
    OnboardingCreateKey,
    /// メイン画面に遷移
    TransitionToMain,
    /// チャンネル作成
    CreateChannel { name: String, about: String },
    /// チャンネル作成完了待ち
    WaitForChannelCreation,
    /// チャンネルを開く
    OpenChannel { channel_id: String },
    /// メッセージ送信
    SendMessage { content: String },
    /// タイムライン確認
    VerifyTimeline,
    /// DM画面を開く
    OpenDm { peer: String },
    /// DM送信
    SendDm { content: String },
    /// 完了
    Completed,
}

/// デバッグテストランナー
pub struct DebugTestRunner {
    enabled: bool,
    current_step: TestStep,
    step_index: usize,
    pub frame_counter: u32,
    pub wait_frames: u32,
    scenario: Vec<TestStep>,
    pub created_channel_id: Option<String>,
}

impl DebugTestRunner {
    /// 新規作成
    pub fn new(enabled: bool) -> Self {
        let scenario = vec![
            TestStep::Idle,
            TestStep::OnboardingCreateKey,
            TestStep::TransitionToMain,
            TestStep::CreateChannel {
                name: "🧪 Rustr Debug Test".to_string(),
                about: "Automated test channel - reused across test runs".to_string(),
            },
            TestStep::WaitForChannelCreation,
            TestStep::OpenChannel { 
                channel_id: String::new() // LocalStorageから取得される
            },
            TestStep::SendMessage { 
                content: "🤖 自動テスト: Hello from debug mode!".to_string() 
            },
            TestStep::VerifyTimeline,
            TestStep::Completed,
        ];

        Self {
            enabled,
            current_step: TestStep::Idle,
            step_index: 0,
            frame_counter: 0,
            wait_frames: 60, // 1秒待機（60fps想定）
            scenario,
            created_channel_id: None,
        }
    }

    /// テストが有効か
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 現在のステップ
    pub fn current_step(&self) -> &TestStep {
        &self.current_step
    }

    /// 次のステップへ進む
    pub fn advance_step(&mut self) {
        self.step_index += 1;
        if self.step_index < self.scenario.len() {
            self.current_step = self.scenario[self.step_index].clone();
            self.frame_counter = 0;
            log::info!("🧪 Test step {}/{}: {:?}", 
                self.step_index, 
                self.scenario.len(), 
                self.current_step
            );
        } else {
            self.current_step = TestStep::Completed;
            log::info!("✅ All test steps completed!");
        }
    }

    /// ステータス文字列を取得（UI表示用）
    pub fn get_status_text(&self) -> String {
        if !self.enabled {
            return String::new();
        }

        match &self.current_step {
            TestStep::Completed => {
                "✅ Debug Test: COMPLETED".to_string()
            }
            _ => {
                format!(
                    "🧪 Debug Test: Step {}/{} - {:?} (frame: {})",
                    self.step_index,
                    self.scenario.len(),
                    self.current_step,
                    self.frame_counter
                )
            }
        }
    }
}

/// デバッグモードが有効かチェック
pub fn is_debug_test_enabled() -> bool {
    // URLパラメータで制御
    if let Some(window) = web_sys::window() {
        if let Some(location) = window.location().href().ok() {
            return location.contains("debug_test=1") || location.contains("debug_test=true");
        }
    }
    
    // または環境変数（ビルド時）
    #[cfg(feature = "debug-test")]
    return true;
    
    #[cfg(not(feature = "debug-test"))]
    false
}

