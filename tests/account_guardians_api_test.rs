use hyper::{header::AUTHORIZATION, StatusCode};
use lib::{
    models::api::{ApiErrorResponse, ApiResponse, ListAccountGuardiansResponse},
    operations::jwt::decode_jwt,
    repos::{guardian_account_repo, guardian_repo, nomination_repo},
    test::utils::{create_verified_account_jwt, setup, tear_down},
};
use uuid::Uuid;

#[tokio::test]
async fn test_can_retrieve_all_guardians_for_an_account() {
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

    let nomination_id = Uuid::new_v4();

    nomination_repo::create(
        &app_state.database,
        nomination_id,
        guardian_email.clone(),
        account_id.clone().to_string(),
        guardian_id.to_string(),
        "ACCEPTED".to_string(),
    )
    .await
    .unwrap();

    guardian_account_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_id.to_string(),
        account_id.to_string(),
        "AVAILABLE".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .get("/accounts/guardians")
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<ListAccountGuardiansResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
    ".**.id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_can_retrieve_all_guardians_for_an_account_filter_by_status() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;

    let guardian_id_1 = Uuid::new_v4();
    let guardian_id_2 = Uuid::new_v4();

    guardian_repo::create(
        &app_state.database,
        guardian_id_1.clone(),
        "guardian1@example.com".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    guardian_repo::create(
        &app_state.database,
        guardian_id_2.clone(),
        "guardian2@example.com".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    guardian_account_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_id_1.to_string(),
        account_id.to_string(),
        "AVAILABLE".to_string(),
    )
    .await
    .unwrap();

    guardian_account_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_id_2.to_string(),
        account_id.to_string(),
        "ACTIVE".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .get("/accounts/guardians?status=active")
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<ListAccountGuardiansResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
    ".**.id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_can_retrieve_all_guardians_for_an_account_filter_by_guardian_id() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;

    let guardian_id_1 = Uuid::new_v4();
    let guardian_id_2 = Uuid::new_v4();

    guardian_repo::create(
        &app_state.database,
        guardian_id_1.clone(),
        "guardian1@example.com".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    guardian_repo::create(
        &app_state.database,
        guardian_id_2.clone(),
        "guardian2@example.com".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    guardian_account_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_id_1.to_string(),
        account_id.to_string(),
        "AVAILABLE".to_string(),
    )
    .await
    .unwrap();

    guardian_account_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_id_2.to_string(),
        account_id.to_string(),
        "ACTIVE".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .get(format!("/accounts/guardians?guardian_id={}", guardian_id_1).as_str())
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<ListAccountGuardiansResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
    ".**.id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_can_remove_an_available_guardian_from_an_account() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;

    let guardian_id_1 = Uuid::new_v4();
    let guardian_id_2 = Uuid::new_v4();

    guardian_repo::create(
        &app_state.database,
        guardian_id_1.clone(),
        "guardian1@example.com".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    guardian_repo::create(
        &app_state.database,
        guardian_id_2.clone(),
        "guardian2@example.com".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    guardian_account_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_id_1.to_string(),
        account_id.to_string(),
        "AVAILABLE".to_string(),
    )
    .await
    .unwrap();

    guardian_account_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_id_2.to_string(),
        account_id.to_string(),
        "ACTIVE".to_string(),
    )
    .await
    .unwrap();

    let delete_res = client
        .delete(format!("/accounts/guardians/{}", guardian_id_1).as_str())
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(delete_res.status(), StatusCode::OK);

    let res = client
        .get("/accounts/guardians")
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<ListAccountGuardiansResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
    ".**.id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_error_when_remove_an_active_guardian_from_an_account() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;

    let guardian_id_1 = Uuid::new_v4();
    let guardian_id_2 = Uuid::new_v4();

    guardian_repo::create(
        &app_state.database,
        guardian_id_1.clone(),
        "guardian1@example.com".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    guardian_repo::create(
        &app_state.database,
        guardian_id_2.clone(),
        "guardian2@example.com".to_string(),
        None,
        None,
    )
    .await
    .unwrap();

    guardian_account_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_id_1.to_string(),
        account_id.to_string(),
        "AVAILABLE".to_string(),
    )
    .await
    .unwrap();

    guardian_account_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_id_2.to_string(),
        account_id.to_string(),
        "ACTIVE".to_string(),
    )
    .await
    .unwrap();

    let delete_res = client
        .delete(format!("/accounts/guardians/{}", guardian_id_2).as_str())
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(delete_res.status(), StatusCode::OK);
    insta::assert_yaml_snapshot!(delete_res.text().await);

    let res = client
        .get("/accounts/guardians")
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<ListAccountGuardiansResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
    ".**.id" => "[uuid]",
    });

    tear_down(db_url).await;
}
