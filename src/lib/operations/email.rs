use reqwest::Client;
use sendinblue_v3::apis::configuration::{ApiKey, Configuration};
use sendinblue_v3::apis::smtp_api::send_transac_email;
use sendinblue_v3::models::{SendSmtpEmail, SendSmtpEmailToInner};

pub async fn send_verification_code_email(
    api_key: String,
    template_id: i64,
    base_path: String,
    to: String,
    code: String,
) -> anyhow::Result<()> {
    let email = SendSmtpEmail {
        to: vec![SendSmtpEmailToInner {
            email: to,
            name: None,
        }],
        template_id: Some(template_id),
        params: Some(serde_json::json!({
            "code": code,
        })),
        ..Default::default()
    };

    let configuration = Configuration {
        base_path,
        client: Client::new(),
        api_key: Some(ApiKey {
            prefix: None,
            key: api_key.clone(),
        }),
        ..Default::default()
    };

    if api_key.clone() == "skip" {
        Ok(())
    } else {
        send_transac_email(&configuration, email)
            .await
            .map_err(|e| anyhow::anyhow!("Error sending email: {}", e))
            .map(|_| ())
    }
}
