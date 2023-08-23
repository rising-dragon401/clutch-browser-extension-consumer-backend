use crate::repos::db::AppState;
use crate::routes::verification_api;
use axum::{response::Html, routing::get, Router};
use hyper::{StatusCode, Uri};
use utoipa_swagger_ui::SwaggerUi;
use utoipa::OpenApi;
use super::{account_api, guardian_api, transaction_api};

#[derive(OpenApi)]
#[openapi(
    info(description = "Clutch Api description"),
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
