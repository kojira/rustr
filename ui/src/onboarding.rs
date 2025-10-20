use eframe::egui;
use crate::i18n::I18n;

/// オンボーディング画面
pub struct Onboarding {
    step: OnboardingStep,
    nsec_input: String,
    passphrase_input: String,
    error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OnboardingStep {
    Welcome,
    ChooseSigner,
    ImportKey,
    CreateKey,
    Completed,
}

/// オンボーディング結果
#[derive(Debug, Clone)]
pub enum OnboardingResult {
    Nip07,
    ImportKey { nsec: String, passphrase: String },
    CreateKey { passphrase: String },
}

impl Onboarding {
    pub fn new() -> Self {
        Self {
            step: OnboardingStep::Welcome,
            nsec_input: String::new(),
            passphrase_input: String::new(),
            error_message: None,
        }
    }
    
    /// オンボーディング画面を表示
    /// 完了したら Some(OnboardingResult) を返す
    pub fn show(&mut self, ui: &mut egui::Ui, i18n: &I18n) -> Option<OnboardingResult> {
        match self.step {
            OnboardingStep::Welcome => {
                self.show_welcome(ui, i18n);
                None
            }
            OnboardingStep::ChooseSigner => {
                self.show_choose_signer(ui, i18n)
            }
            OnboardingStep::ImportKey => {
                self.show_import_key(ui, i18n)
            }
            OnboardingStep::CreateKey => {
                self.show_create_key(ui, i18n)
            }
            OnboardingStep::Completed => None,
        }
    }
    
    fn show_welcome(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            crate::emoji_label::emoji_heading(ui, i18n.onboarding_welcome_title());
            ui.add_space(20.0);
            crate::emoji_label::emoji_label(ui, i18n.onboarding_welcome_description());
            ui.add_space(40.0);
            
            if ui.button(i18n.onboarding_get_started()).clicked() {
                self.step = OnboardingStep::ChooseSigner;
            }
        });
    }
    
    fn show_choose_signer(&mut self, ui: &mut egui::Ui, i18n: &I18n) -> Option<OnboardingResult> {
        let mut result = None;
        
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            crate::emoji_label::emoji_heading(ui, i18n.onboarding_choose_signer_title());
            ui.add_space(20.0);
            
            ui.group(|ui| {
                ui.set_min_width(400.0);
                
                if ui.button(i18n.onboarding_use_extension()).clicked() {
                    log::info!("Selected NIP-07");
                    result = Some(OnboardingResult::Nip07);
                }
                
                ui.add_space(10.0);
                
                if ui.button(i18n.onboarding_import_key()).clicked() {
                    self.step = OnboardingStep::ImportKey;
                }
                
                ui.add_space(10.0);
                
                if ui.button(i18n.onboarding_create_key()).clicked() {
                    self.step = OnboardingStep::CreateKey;
                }
            });
            
            if let Some(error) = &self.error_message {
                ui.add_space(20.0);
                let error_text = egui::RichText::new(error).color(egui::Color32::RED);
                crate::emoji_label::emoji_label(ui, error_text);
            }
        });
        
        result
    }
    
    fn show_import_key(&mut self, ui: &mut egui::Ui, i18n: &I18n) -> Option<OnboardingResult> {
        let mut result = None;
        
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            crate::emoji_label::emoji_heading(ui, i18n.onboarding_import_key_title());
            ui.add_space(20.0);
            
            ui.group(|ui| {
                ui.set_min_width(400.0);
                
                crate::emoji_label::emoji_label(ui, i18n.onboarding_enter_nsec());
                ui.text_edit_singleline(&mut self.nsec_input);
                
                ui.add_space(10.0);
                
                crate::emoji_label::emoji_label(ui, i18n.onboarding_passphrase());
                ui.add(egui::TextEdit::singleline(&mut self.passphrase_input).password(true));
                
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    if ui.button(i18n.onboarding_back()).clicked() {
                        self.step = OnboardingStep::ChooseSigner;
                        self.error_message = None;
                    }
                    
                    if ui.button(i18n.onboarding_import()).clicked() {
                        if self.nsec_input.is_empty() || self.passphrase_input.is_empty() {
                            self.error_message = Some(i18n.onboarding_error_fill_fields().to_string());
                        } else {
                            log::info!("Importing key");
                            result = Some(OnboardingResult::ImportKey {
                                nsec: self.nsec_input.clone(),
                                passphrase: self.passphrase_input.clone(),
                            });
                        }
                    }
                });
            });
            
            if let Some(error) = &self.error_message {
                ui.add_space(20.0);
                let error_text = egui::RichText::new(error).color(egui::Color32::RED);
                crate::emoji_label::emoji_label(ui, error_text);
            }
        });
        
        result
    }
    
    fn show_create_key(&mut self, ui: &mut egui::Ui, i18n: &I18n) -> Option<OnboardingResult> {
        let mut result = None;
        
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            crate::emoji_label::emoji_heading(ui, i18n.onboarding_create_key_title());
            ui.add_space(20.0);
            
            ui.group(|ui| {
                ui.set_min_width(400.0);
                
                crate::emoji_label::emoji_label(ui, i18n.onboarding_set_passphrase());
                ui.add(egui::TextEdit::singleline(&mut self.passphrase_input).password(true));
                
                ui.add_space(10.0);
                
                crate::emoji_label::emoji_label(ui, i18n.onboarding_important_warning());
                crate::emoji_label::emoji_label(ui, i18n.onboarding_need_passphrase());
                
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    if ui.button(i18n.onboarding_back()).clicked() {
                        self.step = OnboardingStep::ChooseSigner;
                        self.error_message = None;
                    }
                    
                    if ui.button(i18n.onboarding_create()).clicked() {
                        if self.passphrase_input.is_empty() {
                            self.error_message = Some(i18n.onboarding_error_enter_passphrase().to_string());
                        } else {
                            log::info!("Creating new key");
                            result = Some(OnboardingResult::CreateKey {
                                passphrase: self.passphrase_input.clone(),
                            });
                        }
                    }
                });
            });
            
            if let Some(error) = &self.error_message {
                ui.add_space(20.0);
                let error_text = egui::RichText::new(error).color(egui::Color32::RED);
                crate::emoji_label::emoji_label(ui, error_text);
            }
        });
        
        result
    }
}
