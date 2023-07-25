use crate::models::api::{
    api_error, api_success, ApiErrorResponse, ApiResponse, VerificationRequest,
    VerificationResponse,
};
use crate::operations::code::generate_code;
use crate::operations::email::send_verification_code_email;
use crate::operations::time::get_unix_timestamp_ms;
use crate::repos::db::AppState;
use crate::repos::verification_repo;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::{routing::post, Router};
use email_address::EmailAddress;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

pub fn routes<S>(app_state: &AppState) -> Router<S> {
    Router::new()
        .route("/verify", post(create_verification))
        .with_state(app_state.to_owned())
}

async fn create_verification(
    app_state: State<AppState>,
    Json(req): Json<VerificationRequest>,
) -> Result<Json<ApiResponse<VerificationResponse, ApiErrorResponse>>, StatusCode> {
    match try_create_verification(&app_state, &req).await {
        Ok(payload) => Ok(Json(api_success(payload))),
        Err(error_payload) => Ok(Json(api_error(format!("{}", error_payload)))),
    }
}

async fn try_create_verification(
    app_state: &State<AppState>,
    req: &VerificationRequest,
) -> anyhow::Result<VerificationResponse> {
    if EmailAddress::is_valid(&req.email) {
        let code = generate_code();
        verify_email_send_limit(app_state, req).await?;
        store_verification(app_state.database.clone(), req.email.clone(), code.clone()).await?;
        send_email_verification_code(app_state, req.email.clone(), code).await?;
        Ok(VerificationResponse { success: true })
    } else {
        Err(anyhow::anyhow!("Invalid email format {}", req.email))
    }
}

async fn verify_email_send_limit(
    app_state: &State<AppState>,
    req: &VerificationRequest,
) -> Result<(), anyhow::Error> {
    let email_count = verification_repo::count_by_email(&app_state.database, &req.email).await?;
    let email_send_limit = app_state.settings.email.send_limit.try_into().unwrap_or(0);
    if email_count > email_send_limit {
        return Err(anyhow::anyhow!(
            "Email verification limit exceeded (attempts: {}, limit: {}) for email: {}",
            email_count,
            email_send_limit,
            req.email
        ));
    };
    Ok(())
}

async fn send_email_verification_code(
    app_state: &State<AppState>,
    email: String,
    code: String,
) -> anyhow::Result<()> {
    send_verification_code_email(
        app_state.settings.email.key().clone(),
        app_state.settings.email.template_id,
        app_state.settings.email.base_url.clone(),
        email,
        code,
    )
    .await
}

pub async fn store_verification(
    db: DatabaseConnection,
    email: String,
    code: String,
) -> anyhow::Result<()> {
    let id = Uuid::new_v4();
    let one_minute = 60 * 1000;
    let expires_at = get_unix_timestamp_ms() + one_minute;

    verification_repo::create(&db, id, &email, &code, expires_at)
        .await
        .map_err(|e| anyhow::anyhow!("Error storing verification: {}", e))
}
