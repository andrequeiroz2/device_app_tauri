use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{AppHandle, Manager};
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct EmailConfigFile {
    resend_api_key: Option<String>,
    from_email: Option<String>,
    reset_password_url: Option<String>,
}

pub struct EmailConfig {
    api_key: String,
    from_email: String,
    reset_password_url: String,
}

impl EmailConfig {
    const DEFAULT_API_KEY: &'static str = "re_NhZ9awxr_4w8fxPdddCcvgzQcHaHB1FpT";
    const DEFAULT_FROM_EMAIL: &'static str = "onboarding@resend.dev";
    // Protocol handler para desktop: tauri://reset-password?token=...
    const DEFAULT_RESET_PASSWORD_URL: &'static str = "tauri://reset-password";

    pub fn init(app_handle: &AppHandle) -> Result<EmailConfig, String> {
        let mut api_key = Self::DEFAULT_API_KEY.to_string();
        let mut from_email = Self::DEFAULT_FROM_EMAIL.to_string();
        let mut reset_password_url = Self::DEFAULT_RESET_PASSWORD_URL.to_string();

        // Tentar ler arquivo de config (opcional - permite override)
        let config_dir = app_handle
            .path()
            .app_config_dir()
            .map_err(|e| format!("app_config_dir error: {}", e))?;
        
        if !config_dir.exists() {
            if let Err(e) = fs::create_dir_all(&config_dir) {
                warn!("Failed to create config dir: {}, using defaults", e);
            }
        }
        
        let config_path = config_dir.join("email_config.json");
        
        // Se arquivo existe, ler e usar para override dos valores padrão
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_json::from_str::<EmailConfigFile>(&content) {
                        Ok(config_file) => {
                            if let Some(key) = config_file.resend_api_key {
                                if !key.is_empty() {
                                    api_key = key;
                                    info!("Using custom RESEND_API_KEY from config file");
                                }
                            }
                            if let Some(email) = config_file.from_email {
                                if !email.is_empty() {
                                    from_email = email;
                                    info!("Using custom from_email from config file");
                                }
                            }
                            if let Some(url) = config_file.reset_password_url {
                                if !url.is_empty() {
                                    reset_password_url = url;
                                    info!("Using custom reset_password_url from config file");
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse email_config.json: {}, using defaults", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read email_config.json: {}, using defaults", e);
                }
            }
        }
        
        Ok(EmailConfig {
            api_key,
            from_email,
            reset_password_url,
        })
    }
    
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
    
    pub fn from_email(&self) -> &str {
        &self.from_email
    }
    
    pub fn reset_password_url(&self) -> &str {
        &self.reset_password_url
    }
}

