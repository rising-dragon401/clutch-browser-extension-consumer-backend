use crate::repos::db::AppState;
use axum::{response::Html, routing::get, Router};
use hyper::{StatusCode, Uri};
use utoipa_swagger_ui::SwaggerUi;
use utoipa::OpenApi;
use super::{account_api, guardian_api, transaction_api, verification_api};
use crate::models::api;
use clutch_wallet_lib::utils::bundler;

#[derive(OpenApi)]
#[openapi(
    info(description = "Clutch Api description"),
    components(schemas(api::Account, api::VerificationRequest, api::VerificationResponse, api::AccountCreateRequest, api::AccountCreateResponse,
    api::SendTransactionRequest, api::SendTransactionResponse, bundler::UserOperationTransport)),
    paths(verification_api::create_verification, account_api::create_account, transaction_api::send_transaction)
)]
struct ApiDoc;


pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(handler))
        .nest("/email", verification_api::routes(&app_state))
        .nest("/accounts", account_api::routes(&app_state))
        .nest("/guardian", guardian_api::routes(&app_state))
        .nest("/transaction", transaction_api::routes(&app_state))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .fallback(fallback)
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn fallback(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("No route for {}", uri))
}
