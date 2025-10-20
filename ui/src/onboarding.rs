use eframe::egui;

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
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<OnboardingResult> {
        match self.step {
            OnboardingStep::Welcome => {
                self.show_welcome(ui);
                None
            }
            OnboardingStep::ChooseSigner => {
                self.show_choose_signer(ui)
            }
            OnboardingStep::ImportKey => {
                self.show_import_key(ui)
            }
            OnboardingStep::CreateKey => {
                self.show_create_key(ui)
            }
            OnboardingStep::Completed => None,
        }
    }
    
    fn show_welcome(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading("🦀 Welcome to Rustr");
            ui.add_space(20.0);
            ui.label("A decentralized Nostr client built with Rust and egui");
            ui.add_space(40.0);
            
            if ui.button("Get Started →").clicked() {
                self.step = OnboardingStep::ChooseSigner;
            }
        });
    }
    
    fn show_choose_signer(&mut self, ui: &mut egui::Ui) -> Option<OnboardingResult> {
        let mut result = None;
        
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("Choose Your Signer");
            ui.add_space(20.0);
            
            ui.group(|ui| {
                ui.set_min_width(400.0);
                
                if ui.button("🔌 Use Browser Extension (NIP-07)").clicked() {
                    log::info!("Selected NIP-07");
                    result = Some(OnboardingResult::Nip07);
                }
                
                ui.add_space(10.0);
                
                if ui.button("📥 Import Existing Key").clicked() {
                    self.step = OnboardingStep::ImportKey;
                }
                
                ui.add_space(10.0);
                
                if ui.button("✨ Create New Key").clicked() {
                    self.step = OnboardingStep::CreateKey;
                }
            });
            
            if let Some(error) = &self.error_message {
                ui.add_space(20.0);
                ui.colored_label(egui::Color32::RED, error);
            }
        });
        
        result
    }
    
    fn show_import_key(&mut self, ui: &mut egui::Ui) -> Option<OnboardingResult> {
        let mut result = None;
        
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("Import Your Key");
            ui.add_space(20.0);
            
            ui.group(|ui| {
                ui.set_min_width(400.0);
                
                ui.label("Enter your nsec (private key):");
                ui.text_edit_singleline(&mut self.nsec_input);
                
                ui.add_space(10.0);
                
                ui.label("Passphrase (for encryption):");
                ui.add(egui::TextEdit::singleline(&mut self.passphrase_input).password(true));
                
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    if ui.button("← Back").clicked() {
                        self.step = OnboardingStep::ChooseSigner;
                        self.error_message = None;
                    }
                    
                    if ui.button("Import →").clicked() {
                        if self.nsec_input.is_empty() || self.passphrase_input.is_empty() {
                            self.error_message = Some("Please fill in all fields".to_string());
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
                ui.colored_label(egui::Color32::RED, error);
            }
        });
        
        result
    }
    
    fn show_create_key(&mut self, ui: &mut egui::Ui) -> Option<OnboardingResult> {
        let mut result = None;
        
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("Create New Key");
            ui.add_space(20.0);
            
            ui.group(|ui| {
                ui.set_min_width(400.0);
                
                ui.label("Set a passphrase to encrypt your key:");
                ui.add(egui::TextEdit::singleline(&mut self.passphrase_input).password(true));
                
                ui.add_space(10.0);
                
                ui.label("⚠ Important: Save your passphrase securely!");
                ui.label("You'll need it to access your account.");
                
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    if ui.button("← Back").clicked() {
                        self.step = OnboardingStep::ChooseSigner;
                        self.error_message = None;
                    }
                    
                    if ui.button("Create Key →").clicked() {
                        if self.passphrase_input.is_empty() {
                            self.error_message = Some("Please enter a passphrase".to_string());
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
                ui.colored_label(egui::Color32::RED, error);
            }
        });
        
        result
    }
}
