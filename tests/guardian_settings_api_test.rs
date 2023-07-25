use hyper::{header::AUTHORIZATION, StatusCode};
use lib::{
    models::api::{
        AccountGuardianSettingsResponse, ApiErrorResponse, ApiResponse, SigningStrategy,
    },
    operations::jwt::decode_jwt,
    repos::{guardian_account_repo, guardian_repo, guardian_settings_repo},
    test::utils::{create_verified_account_jwt, setup, tear_down},
};
use uuid::Uuid;

#[tokio::test]
async fn test_can_retrieve_guardian_settings_for_an_account() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;
    let guardian_email = "guardian@example.com".to_string();

    let guardian_id = Uuid::new_v4();
    guardian_repo::create(
        &app_state.database,
        guardian_id,
        guardian_email.clone(),
        None,
        None,
    )
    .await
    .unwrap();

    guardian_account_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_id.to_string(),
        account_id.to_string(),
        "ACTIVE".to_string(),
    )
    .await
    .unwrap();

    guardian_settings_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        SigningStrategy::OneOfOne,
        account_id.to_string(),
    )
    .await
    .unwrap();

    let res = client
        .get("/accounts/guardians/settings")
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<AccountGuardianSettingsResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
    ".**.id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_can_successfully_update_guardian_settings() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;

    let guardian_email = "guardian@example.com".to_string();

    let guardian_id = Uuid::new_v4();
    guardian_repo::create(
        &app_state.database,
        guardian_id,
        guardian_email.clone(),
        None,
        None,
    )
    .await
    .unwrap();

    let guardian_account_id = Uuid::new_v4();
    guardian_account_repo::create(
        &app_state.database,
        guardian_account_id,
        guardian_id.to_string(),
        account_id.to_string(),
        "ACTIVE".to_string(),
    )
    .await
    .unwrap();

    guardian_settings_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        SigningStrategy::OneOfOne,
        account_id.to_string(),
    )
    .await
    .unwrap();

    let res = client
        .put("/accounts/guardians/settings")
        .body(format!(
            "{{\"signers\":\"{}\",\"guardians\":[\"{}\"]}}",
            "OneOfOne".to_string(),
            guardian_account_id.to_string()
        ))
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<AccountGuardianSettingsResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
    ".**.id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_error_when_invalid_amount_of_guardians_for_proposed_signing_strategy_on_update_guardian_settings(
) {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;

    let guardian_email = "guardian@example.com".to_string();

    let guardian_id = Uuid::new_v4();
    guardian_repo::create(
        &app_state.database,
        guardian_id,
        guardian_email.clone(),
        None,
        None,
    )
    .await
    .unwrap();

    let guardian_account_id = Uuid::new_v4();
    guardian_account_repo::create(
        &app_state.database,
        guardian_account_id,
        guardian_id.to_string(),
        account_id.to_string(),
        "ACTIVE".to_string(),
    )
    .await
    .unwrap();

    guardian_settings_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        SigningStrategy::OneOfOne,
        account_id.to_string(),
    )
    .await
    .unwrap();

    let res = client
        .put("/accounts/guardians/settings")
        .body(format!(
            "{{\"signers\":\"{}\",\"guardians\":[\"{}\"]}}",
            "OneOfTwo".to_string(),
            guardian_account_id.to_string()
        ))
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<AccountGuardianSettingsResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
    ".**.id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_error_when_invalid_guardians_for_on_update_guardian_settings() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;

    let guardian_email = "guardian@example.com".to_string();

    let guardian_id = Uuid::new_v4();
    guardian_repo::create(
        &app_state.database,
        guardian_id,
        guardian_email.clone(),
        None,
        None,
    )
    .await
    .unwrap();

    let guardian_account_id = "d69e35f5-3ce3-402b-b70b-11745651d88f".to_string();
    guardian_account_repo::create(
        &app_state.database,
        guardian_account_id.parse().unwrap(),
        guardian_id.to_string(),
        account_id.to_string(),
        "ACTIVE".to_string(),
    )
    .await
    .unwrap();

    guardian_settings_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        SigningStrategy::OneOfOne,
        account_id.to_string(),
    )
    .await
    .unwrap();

    let res = client
        .put("/accounts/guardians/settings")
        .body(format!(
            "{{\"signers\":\"{}\",\"guardians\":[\"{}\"]}}",
            "OneOfOne".to_string(),
            "6ac5790f-148d-46be-a657-0b06ad41d135".to_string()
        ))
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<AccountGuardianSettingsResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
    ".**.id" => "[uuid]",
    });

    tear_down(db_url).await;
}
