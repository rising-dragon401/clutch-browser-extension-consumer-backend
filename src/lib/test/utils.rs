use crate::{
    config::settings::{Env, Settings},
    models::api::{AccountCreateResponse, ApiErrorResponse, ApiPayload, ApiResponse},
    operations::time::get_unix_timestamp_ms,
    repos::{
        db::{db_connect, AppState},
        migration::migrate,
        verification_repo,
    },
    routes::api::router,
};
use axum_test_helper::{TestClient, TestResponse};
use hyper::StatusCode;
use sea_orm::DatabaseConnection;
use std::fs;
use uuid::Uuid;

pub async fn setup() -> (TestClient, AppState, String) {
    // let settings = &Settings::new(Env::Test).unwrap();

    // let random_db = format!("tests/db/db_{}", Uuid::new_v4());
    // let random_db_connection_url = format!("sqlite://{}", random_db);

    // migrate(&random_db);

    // let app_state = AppState {
    //     settings: settings.to_owned(),
    //     database: db_connect(random_db_connection_url).await,
    // };

    // let router = router(app_state.clone());
    // let client = TestClient::new(router);

    // (client, app_state, random_db)
    unimplemented!()
}

pub async fn tear_down(db: String) {
    fs::remove_file(db).expect("Unable to remove database file");
}

// Account Utils

pub async fn create_account(client: &TestClient, email: String, code: String) -> TestResponse {
    client
        .post("/accounts")
        .body(format!("{{\"email\":\"{}\",\"code\":\"{}\"}}", email, code))
        .header("Content-Type", "application/json")
        .send()
        .await
}

pub async fn create_verified_account_jwt(
    db: &DatabaseConnection,
    client: &TestClient,
    email: String,
) -> String {
    let res = create_verify(client, email.clone()).await;
    assert_eq!(res.status(), StatusCode::OK);

    let email = email;
    let verify_id = Uuid::new_v4();
    let one_minute = 60 * 1000;
    let verify_expires_at = get_unix_timestamp_ms() + one_minute;
    let code = "123456";

    verification_repo::create(db, verify_id, &email, code, verify_expires_at)
        .await
        .expect("error creating verification");

    let res = create_account(client, email, code.to_string()).await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<AccountCreateResponse, ApiErrorResponse>>()
        .await;

    match json_response.payload {
        ApiPayload::Success(AccountCreateResponse {
            jwt,
            contract_wallet_addr,
        }) => jwt,
        ApiPayload::Error(ApiErrorResponse { error_message }) => {
            panic!("error: {}", error_message)
        }
    }
}

pub async fn create_verify(client: &TestClient, email: String) -> TestResponse {
    client
        .post("/email/verify")
        .body(format!("{{\"email\":\"{}\"}}", email))
        .header("Content-Type", "application/json")
        .send()
        .await
}
