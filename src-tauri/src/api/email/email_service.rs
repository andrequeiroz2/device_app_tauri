use serde::Deserialize;
use tauri::AppHandle;
use tracing::{instrument, error, info};
use crate::api::email::email_config::EmailConfig;
use crate::api::email::email_templates::reset_password_template;

#[derive(Deserialize)]
struct ResendEmailResponse {
    id: String,
}

#[derive(Deserialize)]
struct ResendErrorResponse {
    message: String,
}

#[instrument(skip(app_handle), fields(to = %to, token = %token))]
pub async fn send_reset_password_email(
    app_handle: &AppHandle,
    to: &str,
    token: &str,
) -> Result<(), String> {
    // 1) Carregar configuração de email
    let config = EmailConfig::init(app_handle)
        .map_err(|e| {
            error!(error = %e, "send_reset_password_email: failed to load email config");
            format!("Failed to load email configuration: {}", e)
        })?;

    // 2) Gerar template HTML (apenas o token)
    let html_content = reset_password_template(token);

    // 4) Preparar requisição para API do Resend
    let client = reqwest::Client::new();
    let api_url = "https://api.resend.com/emails";
    
    let payload = serde_json::json!({
        "from": config.from_email(),
        "to": [to],
        "subject": "Redefinir Senha",
        "html": html_content,
    });

    info!(
        from = %config.from_email(),
        to = %to,
        api_url = %api_url,
        "send_reset_password_email: sending email via Resend"
    );

    // 5) Enviar email via Resend API
    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", config.api_key()))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "send_reset_password_email: reqwest error");
            format!("Failed to send email request: {}", e)
        })?;

    // 6) Verificar status da resposta
    let status = response.status();
    
    if status.is_success() {
        match response.json::<ResendEmailResponse>().await {
            Ok(resend_response) => {
                info!(
                    email_id = %resend_response.id,
                    to = %to,
                    "send_reset_password_email: email sent successfully"
                );
                Ok(())
            }
            Err(e) => {
                error!(error = %e, "send_reset_password_email: failed to parse success response");
                Err(format!("Failed to parse email response: {}", e))
            }
        }
    } else {
        // Tentar ler mensagem de erro da Resend
        let error_msg = match response.json::<ResendErrorResponse>().await {
            Ok(err_resp) => err_resp.message,
            Err(_) => format!("HTTP {}: {}", status, status.canonical_reason().unwrap_or("Unknown error")),
        };
        
        error!(
            status = %status,
            error = %error_msg,
            "send_reset_password_email: Resend API error"
        );
        
        Err(format!("Failed to send email: {}", error_msg))
    }
}

