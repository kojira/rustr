/// 多言語対応リソース管理
use serde::{Deserialize, Serialize};

/// サポート言語
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Japanese,
    English,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::Japanese => "ja",
            Language::English => "en",
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Language::Japanese => "日本語",
            Language::English => "English",
        }
    }
    
    pub fn all() -> &'static [Language] {
        &[Language::Japanese, Language::English]
    }
}

/// 文字列リソース
pub struct I18n {
    language: Language,
}

impl I18n {
    pub fn new(language: Language) -> Self {
        Self { language }
    }
    
    pub fn language(&self) -> &Language {
        &self.language
    }
    
    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }
    
    // アプリケーション名
    pub fn app_name(&self) -> &'static str {
        "Rustr"
    }
    
    // トップバー
    pub fn button_public(&self) -> &'static str {
        match self.language {
            Language::Japanese => "📢 Public",
            Language::English => "📢 Public",
        }
    }
    
    pub fn button_dms(&self) -> &'static str {
        match self.language {
            Language::Japanese => "💬 DMs",
            Language::English => "💬 DMs",
        }
    }
    
    pub fn button_settings(&self) -> &'static str {
        match self.language {
            Language::Japanese => "⚙",
            Language::English => "⚙",
        }
    }
    
    // オンボーディング
    pub fn onboarding_welcome_title(&self) -> &'static str {
        match self.language {
            Language::Japanese => "🦀 Rustrへようこそ",
            Language::English => "🦀 Welcome to Rustr",
        }
    }
    
    pub fn onboarding_welcome_description(&self) -> &'static str {
        match self.language {
            Language::Japanese => "RustとeGuiで構築された分散型Nostrクライアント",
            Language::English => "A decentralized Nostr client built with Rust and egui",
        }
    }
    
    pub fn onboarding_get_started(&self) -> &'static str {
        match self.language {
            Language::Japanese => "はじめる →",
            Language::English => "Get Started →",
        }
    }
    
    pub fn onboarding_choose_signer_title(&self) -> &'static str {
        match self.language {
            Language::Japanese => "署名方法を選択",
            Language::English => "Choose Your Signer",
        }
    }
    
    pub fn onboarding_use_extension(&self) -> &'static str {
        match self.language {
            Language::Japanese => "🔌 ブラウザ拡張機能を使用 (NIP-07)",
            Language::English => "🔌 Use Browser Extension (NIP-07)",
        }
    }
    
    pub fn onboarding_import_key(&self) -> &'static str {
        match self.language {
            Language::Japanese => "📥 既存の鍵をインポート",
            Language::English => "📥 Import Existing Key",
        }
    }
    
    pub fn onboarding_create_key(&self) -> &'static str {
        match self.language {
            Language::Japanese => "✨ 新しい鍵を作成",
            Language::English => "✨ Create New Key",
        }
    }
    
    pub fn onboarding_import_key_title(&self) -> &'static str {
        match self.language {
            Language::Japanese => "鍵をインポート",
            Language::English => "Import Your Key",
        }
    }
    
    pub fn onboarding_enter_nsec(&self) -> &'static str {
        match self.language {
            Language::Japanese => "nsec（秘密鍵）を入力:",
            Language::English => "Enter your nsec (private key):",
        }
    }
    
    pub fn onboarding_passphrase(&self) -> &'static str {
        match self.language {
            Language::Japanese => "パスフレーズ（暗号化用）:",
            Language::English => "Passphrase (for encryption):",
        }
    }
    
    pub fn onboarding_back(&self) -> &'static str {
        match self.language {
            Language::Japanese => "← 戻る",
            Language::English => "← Back",
        }
    }
    
    pub fn onboarding_import(&self) -> &'static str {
        match self.language {
            Language::Japanese => "インポート →",
            Language::English => "Import →",
        }
    }
    
    pub fn onboarding_create_key_title(&self) -> &'static str {
        match self.language {
            Language::Japanese => "新しい鍵を作成",
            Language::English => "Create New Key",
        }
    }
    
    pub fn onboarding_set_passphrase(&self) -> &'static str {
        match self.language {
            Language::Japanese => "鍵を暗号化するパスフレーズを設定:",
            Language::English => "Set a passphrase to encrypt your key:",
        }
    }
    
    pub fn onboarding_important_warning(&self) -> &'static str {
        match self.language {
            Language::Japanese => "⚠ 重要: パスフレーズを安全に保管してください！",
            Language::English => "⚠ Important: Save your passphrase securely!",
        }
    }
    
    pub fn onboarding_need_passphrase(&self) -> &'static str {
        match self.language {
            Language::Japanese => "アカウントにアクセスするために必要です。",
            Language::English => "You'll need it to access your account.",
        }
    }
    
    pub fn onboarding_create(&self) -> &'static str {
        match self.language {
            Language::Japanese => "鍵を作成 →",
            Language::English => "Create Key →",
        }
    }
    
    pub fn onboarding_error_fill_fields(&self) -> &'static str {
        match self.language {
            Language::Japanese => "すべてのフィールドを入力してください",
            Language::English => "Please fill in all fields",
        }
    }
    
    pub fn onboarding_error_enter_passphrase(&self) -> &'static str {
        match self.language {
            Language::Japanese => "パスフレーズを入力してください",
            Language::English => "Please enter a passphrase",
        }
    }
    
    // チャンネル作成
    pub fn channel_create_title(&self) -> &'static str {
        match self.language {
            Language::Japanese => "📢 新しいチャンネルを作成",
            Language::English => "📢 Create New Channel",
        }
    }
    
    pub fn channel_name_label(&self) -> &'static str {
        match self.language {
            Language::Japanese => "チャンネル名:",
            Language::English => "Channel name:",
        }
    }
    
    pub fn channel_about_label(&self) -> &'static str {
        match self.language {
            Language::Japanese => "説明:",
            Language::English => "Description:",
        }
    }
    
    pub fn button_cancel(&self) -> &'static str {
        match self.language {
            Language::Japanese => "✖ キャンセル",
            Language::English => "✖ Cancel",
        }
    }
    
    pub fn button_create(&self) -> &'static str {
        match self.language {
            Language::Japanese => "✅ 作成",
            Language::English => "✅ Create",
        }
    }
    
    // コンポーザー
    pub fn composer_title(&self) -> &'static str {
        match self.language {
            Language::Japanese => "✏ メッセージを作成",
            Language::English => "✏ Compose Message",
        }
    }
    
    pub fn composer_placeholder(&self) -> &'static str {
        match self.language {
            Language::Japanese => "メッセージを入力...",
            Language::English => "Type your message here...",
        }
    }
    
    pub fn composer_send(&self) -> &'static str {
        match self.language {
            Language::Japanese => "📤 送信",
            Language::English => "📤 Send",
        }
    }
    
    pub fn composer_char_count(&self, count: usize) -> String {
        match self.language {
            Language::Japanese => format!("{} 文字", count),
            Language::English => format!("{} chars", count),
        }
    }
    
    pub fn button_close(&self) -> &'static str {
        match self.language {
            Language::Japanese => "✖ 閉じる",
            Language::English => "✖ Close",
        }
    }
    
    pub fn button_new_post(&self) -> &'static str {
        match self.language {
            Language::Japanese => "✏ 新規投稿",
            Language::English => "✏ New Post",
        }
    }
    
    // タイムライン
    pub fn timeline_empty(&self) -> &'static str {
        match self.language {
            Language::Japanese => "まだイベントがありません。会話を始めましょう！",
            Language::English => "No events yet. Start a conversation!",
        }
    }
    
    pub fn timeline_reply(&self) -> &'static str {
        match self.language {
            Language::Japanese => "💬 返信",
            Language::English => "💬 Reply",
        }
    }
    
    pub fn timeline_like(&self) -> &'static str {
        match self.language {
            Language::Japanese => "❤ いいね",
            Language::English => "❤ Like",
        }
    }
    
    // 設定
    pub fn settings_title(&self) -> &'static str {
        match self.language {
            Language::Japanese => "⚙️ 設定",
            Language::English => "⚙️ Settings",
        }
    }
    
    pub fn settings_language(&self) -> &'static str {
        match self.language {
            Language::Japanese => "🌐 言語",
            Language::English => "🌐 Language",
        }
    }
    
    pub fn settings_font(&self) -> &'static str {
        match self.language {
            Language::Japanese => "🔤 フォント設定",
            Language::English => "🔤 Font Settings",
        }
    }
    
    pub fn settings_font_family(&self) -> &'static str {
        match self.language {
            Language::Japanese => "フォントファミリー",
            Language::English => "Font Family",
        }
    }
    
    pub fn settings_restart_required(&self) -> &'static str {
        match self.language {
            Language::Japanese => "⚠️ フォント変更を適用するには再起動が必要です",
            Language::English => "⚠️ Restart required to apply font changes",
        }
    }
    
    pub fn settings_save_and_restart(&self) -> &'static str {
        match self.language {
            Language::Japanese => "💾 保存して再起動",
            Language::English => "💾 Save and Restart",
        }
    }
    
    pub fn settings_font_info(&self) -> &'static str {
        match self.language {
            Language::Japanese => "ℹ️ フォント情報",
            Language::English => "ℹ️ Font Information",
        }
    }
    
    pub fn settings_font_preview(&self) -> &'static str {
        match self.language {
            Language::Japanese => "📝 プレビュー",
            Language::English => "📝 Preview",
        }
    }
    
    pub fn settings_preview_japanese(&self) -> &'static str {
        match self.language {
            Language::Japanese => "日本語: こんにちは、世界！",
            Language::English => "Japanese: こんにちは、世界！",
        }
    }
    
    pub fn settings_preview_english(&self) -> &'static str {
        "English: Hello, World!"
    }
    
    pub fn settings_preview_emoji(&self) -> &'static str {
        match self.language {
            Language::Japanese => "絵文字: 🎉 🚀 ✨ 💡 🔥",
            Language::English => "Emoji: 🎉 🚀 ✨ 💡 🔥",
        }
    }
    
    pub fn settings_preview_numbers(&self) -> &'static str {
        match self.language {
            Language::Japanese => "数字: 0123456789",
            Language::English => "Numbers: 0123456789",
        }
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new(Language::Japanese)
    }
}

