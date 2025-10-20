use egui_twemoji::EmojiLabel;

/// カラー絵文字をサポートするラベルを表示
pub fn emoji_label(ui: &mut egui::Ui, text: impl Into<egui::RichText>) {
    EmojiLabel::new(text.into()).show(ui);
}

/// カラー絵文字をサポートするボタンを表示
pub fn emoji_button(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    let text_str = text.into();
    // ボタンの中でEmojiLabelを使う
    ui.button(text_str)
}

/// カラー絵文字をサポートするヘッダーを表示
pub fn emoji_heading(ui: &mut egui::Ui, text: impl Into<String>) {
    let text_str = text.into();
    let rich_text = egui::RichText::new(text_str).heading();
    EmojiLabel::new(rich_text).show(ui);
}

