use axum::http::StatusCode;
use axum_test_helper::TestClient;
use hyper::header::AUTHORIZATION;
use lib::models::api::AccountCreateResponse;
use lib::models::api::ApiErrorResponse;
use lib::models::api::ApiResponse;
use lib::models::api::ListAccountsResponse;
use lib::operations::time::get_unix_timestamp_ms;
use lib::repos::account_repo;
use lib::repos::verification_repo;
use lib::test::utils::create_account;
use lib::test::utils::create_verified_account_jwt;
use lib::test::utils::create_verify;
use lib::test::utils::setup;
use lib::test::utils::tear_down;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

#[tokio::test]
async fn test_successfully_create_account_with_jwt() {
    let (client, app_state, db_url) = setup().await;

    let email = "someone@example.com".to_string();
    let id = Uuid::new_v4();
    let one_minute = 60 * 1000;
    let expires_at = get_unix_timestamp_ms() + one_minute;
    let code = "123456";

    verification_repo::create(&app_state.database, id, &email, code, expires_at)
        .await
        .expect("error creating verification");

    let res = create_account(&client, email.clone(), code.to_string()).await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<AccountCreateResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.jwt" => "[jwt]"
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_invalid_email_format() {
    let (client, _app_state, db_url) = setup().await;

    let res = client
        .post("/accounts")
        .body("{\"email\":\"a\"}")
        .header("Content-Type", "application/json")
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);
    insta::assert_yaml_snapshot!(res.text().await);

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_error_when_account_already_exists() {
    let (client, app_state, db_url) = setup().await;

    let email = "someone@example.com".to_string();
    create_verify(&client, email.clone()).await;

    let code = verification_repo::find_all_by_email(&app_state.database, email.clone())
        .await
        .expect("error getting code")
        .first()
        .expect("no code found")
        .code
        .clone();

    let created = create_account(&client, email.clone(), code.clone()).await;
    assert_eq!(created.status(), StatusCode::OK);

    let res = create_account(&client, email.clone(), code.clone()).await;
    assert_eq!(res.status(), StatusCode::OK);

    insta::assert_yaml_snapshot!(res.text().await);

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_error_when_verification_not_found_for_account() {
    let (client, _app_state, db_url) = setup().await;

    let email = "someone@example.com".to_string();

    let res = create_account(&client, email.clone(), "123456".to_string()).await;
    assert_eq!(res.status(), StatusCode::OK);

    insta::assert_yaml_snapshot!(res.text().await);

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_error_when_verification_code_expired() {
    let (client, app_state, db_url) = setup().await;

    let email = "someone@example.com".to_string();
    let id = Uuid::new_v4();
    let one_minute = 60 * 1000;
    let expires_at = get_unix_timestamp_ms() - one_minute;
    let code = "123456";

    verification_repo::create(&app_state.database, id, &email, code, expires_at)
        .await
        .expect("error creating verification");

    let res = create_account(&client, email.clone(), code.to_string()).await;
    assert_eq!(res.status(), StatusCode::OK);

    insta::assert_yaml_snapshot!(res.text().await);

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_retrieve_all_accounts() {
    let (client, app_state, db_url) = setup().await;

    let email = "first@example.com".to_string();
    create_verified_account(
        &app_state.database,
        &client,
        email,
        "".to_string(),
        "".to_string(),
    )
    .await;
    let res = client.get("/accounts").send().await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<ListAccountsResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.accounts[0].id" => "[uuid]",
        ".**.accounts[0].updated_at" => "[timestamp]"
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_find_all_by_wallet_address() {
    let (client, app_state, db_url) = setup().await;

    let email = "first@example.com".to_string();
    let wallet_address = "123".to_string();
    create_verified_account(
        &app_state.database,
        &client,
        email,
        wallet_address.clone(),
        "".to_string(),
    )
    .await;
    create_verified_account(
        &app_state.database,
        &client,
        "another@me.com".to_string(),
        "another_wallet".to_string(),
        "".to_string(),
    )
    .await;
    let res = client
        .get(format!("/accounts?wallet_address={}", wallet_address).as_str())
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<ListAccountsResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.accounts[0].id" => "[uuid]",
        ".**.accounts[0].updated_at" => "[timestamp]"
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_find_all_by_eoa_address() {
    let (client, app_state, db_url) = setup().await;

    let email = "first@example.com".to_string();
    let eao_address = "123".to_string();
    create_verified_account(
        &app_state.database,
        &client,
        email,
        "".to_string(),
        eao_address.clone(),
    )
    .await;
    create_verified_account(
        &app_state.database,
        &client,
        "another@me.com".to_string(),
        "".to_string(),
        "another_eoa".to_string(),
    )
    .await;
    let res = client
        .get(format!("/accounts?eoa_address={}", eao_address).as_str())
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<ListAccountsResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.accounts[0].id" => "[uuid]",
        ".**.accounts[0].updated_at" => "[timestamp]"
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_find_all_by_email_address() {
    let (client, app_state, db_url) = setup().await;

    let email = "first@example.com".to_string();

    create_verified_account(
        &app_state.database,
        &client,
        email.clone(),
        "".to_string(),
        "".to_string(),
    )
    .await;
    create_verified_account(
        &app_state.database,
        &client,
        "another@me.com".to_string(),
        "".to_string(),
        "".to_string(),
    )
    .await;
    let res = client
        .get(format!("/accounts?email={}", email.clone()).as_str())
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<ListAccountsResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.accounts[0].id" => "[uuid]",
        ".**.accounts[0].updated_at" => "[timestamp]"
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_successfully_update_account() {
    let (client, app_state, db_url) = setup().await;

    let email = "first@example.com".to_string();
    let wallet_address = "123".to_string();
    let jwt = create_verified_account_jwt(&app_state.database, &client, email.clone()).await;

    let res = client
        .put("/accounts")
        .body(format!(
            "{{\"email\":\"{}\",\"wallet_address\":\"{}\"}}",
            email.clone(),
            wallet_address
        ))
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);
    insta::assert_yaml_snapshot!(res.text().await);

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_error_when_no_bearer_token_on_update_account() {
    let (client, app_state, db_url) = setup().await;

    let email = "first@example.com".to_string();
    let wallet_address = "123".to_string();
    create_verified_account(
        &app_state.database,
        &client,
        email.clone(),
        wallet_address.clone(),
        "".to_string(),
    )
    .await;

    let res = client
        .put("/accounts")
        .body(format!(
            "{{\"email\":\"{}\",\"wallet_address\":\"{}\"}}",
            email.clone(),
            wallet_address
        ))
        .header("Content-Type", "application/json")
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    insta::assert_yaml_snapshot!(res.text().await);

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_error_when_cannot_decode_bearer_token_on_update_account() {
    let (client, app_state, db_url) = setup().await;

    let email = "first@example.com".to_string();
    let wallet_address = "123".to_string();
    create_verified_account(
        &app_state.database,
        &client,
        email.clone(),
        wallet_address.clone(),
        "".to_string(),
    )
    .await;

    let res = client
        .put("/accounts")
        .body(format!(
            "{{\"email\":\"{}\",\"wallet_address\":\"{}\"}}",
            email.clone(),
            wallet_address
        ))
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, "Bearer 123")
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);
    insta::assert_yaml_snapshot!(res.text().await);

    tear_down(db_url).await;
}

// Helper functions

async fn create_verified_account(
    db: &DatabaseConnection,
    client: &TestClient,
    email: String,
    wallet_address: String,
    eoa_addres: String,
) -> (Uuid, String, i64) {
    let res = create_verify(client, email.clone()).await;
    assert_eq!(res.status(), StatusCode::OK);

    let email = email;
    let verify_id = Uuid::new_v4();
    let one_minute = 60 * 1000;
    let verify_expires_at = get_unix_timestamp_ms() + one_minute;
    let code = "123456";

    let account_id = Uuid::new_v4();
    let account_created_at = get_unix_timestamp_ms();

    verification_repo::create(&db, verify_id, &email, code, verify_expires_at)
        .await
        .expect("error creating verification");

    account_repo::create(
        &db,
        account_id,
        email.clone(),
        wallet_address,
        eoa_addres,
        account_created_at,
    )
    .await
    .expect("error creating account");

    (verify_id, code.to_string(), verify_expires_at)
}
