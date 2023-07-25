use hyper::{header::AUTHORIZATION, StatusCode};
use lib::{
    models::api::{
        ApiErrorResponse, ApiResponse, ListGuardianAccountsResponse, ListNominationsResponse,
        NominationUpdateResponse,
    },
    operations::{jwt::decode_jwt, time::get_unix_timestamp_ms},
    repos::{account_repo, guardian_account_repo, guardian_repo, nomination_repo},
    test::utils::{create_verified_account_jwt, setup, tear_down},
};
use uuid::Uuid;

#[tokio::test]
async fn test_list_all_nominations_for_user_when_user_is_also_a_guardian() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;

    let guardian_id = Uuid::new_v4();
    let guardian_email = "guardian@example.com".to_string();
    guardian_repo::create(
        &app_state.database,
        guardian_id,
        guardian_email.clone(),
        Some(account_id.clone()),
        None,
    )
    .await
    .unwrap();

    nomination_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_email.clone(),
        Uuid::new_v4().to_string(),
        guardian_id.to_string(),
        "PENDING".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .get("/guardian/nominations")
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<ListNominationsResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.id" => "[uuid]",
        ".**.guardian_id" => "[uuid]",
        ".**.account_id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_list_filter_nominations_by_status_for_user_when_user_is_also_a_guardian() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;

    let guardian_id = Uuid::new_v4();
    let guardian_email = "guardian@example.com".to_string();
    guardian_repo::create(
        &app_state.database,
        guardian_id,
        guardian_email.clone(),
        Some(account_id.clone()),
        None,
    )
    .await
    .unwrap();

    nomination_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_email.clone(),
        Uuid::new_v4().to_string(),
        guardian_id.to_string(),
        "PENDING".to_string(),
    )
    .await
    .unwrap();

    nomination_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_email.clone(),
        Uuid::new_v4().to_string(),
        guardian_id.to_string(),
        "ACCEPTED".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .get("/guardian/nominations?status=accepted")
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<ListNominationsResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.id" => "[uuid]",
        ".**.guardian_id" => "[uuid]",
        ".**.account_id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_list_filter_nominations_by_id_for_user_when_user_is_also_a_guardian() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;

    let guardian_id = Uuid::new_v4();
    let guardian_email = "guardian@example.com".to_string();
    guardian_repo::create(
        &app_state.database,
        guardian_id,
        guardian_email.clone(),
        Some(account_id.clone()),
        None,
    )
    .await
    .unwrap();

    let nomination_id = Uuid::new_v4();

    nomination_repo::create(
        &app_state.database,
        nomination_id,
        guardian_email.clone(),
        Uuid::new_v4().to_string(),
        guardian_id.to_string(),
        "PENDING".to_string(),
    )
    .await
    .unwrap();

    nomination_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_email.clone(),
        Uuid::new_v4().to_string(),
        guardian_id.to_string(),
        "ACCEPTED".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .get(format!("/guardian/nominations?nomination_id={}", nomination_id).as_str())
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<ListNominationsResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.id" => "[uuid]",
        ".**.guardian_id" => "[uuid]",
        ".**.account_id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_successfully_accept_nomination_as_a_guardian() {
    let (client, app_state, db_url) = setup().await;

    let guardian_email = "guardian@example.com".to_string();
    let jwt =
        create_verified_account_jwt(&app_state.database, &client, guardian_email.clone()).await;

    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;
    let guardian_id = Uuid::new_v4();

    guardian_repo::create(
        &app_state.database,
        guardian_id,
        guardian_email.clone(),
        Some(account_id.clone()),
        None,
    )
    .await
    .unwrap();

    let nomination_id = Uuid::new_v4();
    let nominators_account_id = Uuid::new_v4();

    nomination_repo::create(
        &app_state.database,
        nomination_id,
        guardian_email.clone(),
        nominators_account_id.to_string(),
        guardian_id.to_string(),
        "PENDING".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .put(format!("/guardian/nomination/{}", nomination_id).as_str())
        .body("{\"status\":\"accepted\"}")
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<NominationUpdateResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.nomination_id" => "[uuid]",
    });

    let guardian_accounts = guardian_account_repo::find_all_guardians_by_account_id(
        &app_state.database,
        nominators_account_id.to_string(),
    )
    .await
    .unwrap();
    assert_eq!(guardian_accounts.len(), 1);
    let first_guardian = guardian_accounts.first().unwrap();
    assert_eq!(first_guardian.guardian_id, guardian_id.to_string());
    assert_eq!(first_guardian.status, "AVAILABLE".to_string());

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_successfully_reject_nomination_as_a_guardian() {
    let (client, app_state, db_url) = setup().await;

    let guardian_email = "guardian@example.com".to_string();
    let jwt =
        create_verified_account_jwt(&app_state.database, &client, guardian_email.clone()).await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;

    let guardian_id = Uuid::new_v4();
    guardian_repo::create(
        &app_state.database,
        guardian_id,
        guardian_email.clone(),
        Some(account_id.clone()),
        None,
    )
    .await
    .unwrap();

    let nomination_id = Uuid::new_v4();

    nomination_repo::create(
        &app_state.database,
        nomination_id,
        guardian_email.clone(),
        Uuid::new_v4().to_string(),
        guardian_id.to_string(),
        "PENDING".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .put(format!("/guardian/nomination/{}", nomination_id).as_str())
        .body("{\"status\":\"rejected\"}")
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<NominationUpdateResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.nomination_id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_error_when_try_to_accept_a_rejected_nomination_as_a_guardian() {
    let (client, app_state, db_url) = setup().await;

    let guardian_email = "guardian@example.com".to_string();
    let jwt =
        create_verified_account_jwt(&app_state.database, &client, guardian_email.clone()).await;

    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;
    let guardian_id = Uuid::new_v4();

    guardian_repo::create(
        &app_state.database,
        guardian_id,
        guardian_email.clone(),
        Some(account_id.clone()),
        None,
    )
    .await
    .unwrap();

    let nomination_id = Uuid::new_v4();

    nomination_repo::create(
        &app_state.database,
        nomination_id,
        guardian_email.clone(),
        Uuid::new_v4().to_string(),
        guardian_id.to_string(),
        "REJECTED".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .put(format!("/guardian/nomination/{}", nomination_id).as_str())
        .body("{\"status\":\"accepted\"}")
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<NominationUpdateResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.nomination_id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_error_when_try_to_reject_an_accepted_nomination_as_a_guardian() {
    let (client, app_state, db_url) = setup().await;

    let guardian_email = "guardian@example.com".to_string();
    let jwt =
        create_verified_account_jwt(&app_state.database, &client, guardian_email.clone()).await;

    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;
    let guardian_id = Uuid::new_v4();

    guardian_repo::create(
        &app_state.database,
        guardian_id,
        guardian_email.clone(),
        Some(account_id.clone()),
        None,
    )
    .await
    .unwrap();

    let nomination_id = Uuid::new_v4();

    nomination_repo::create(
        &app_state.database,
        nomination_id,
        guardian_email.clone(),
        Uuid::new_v4().to_string(),
        guardian_id.to_string(),
        "ACCEPTED".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .put(format!("/guardian/nomination/{}", nomination_id).as_str())
        .body("{\"status\":\"rejected\"}")
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<NominationUpdateResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.nomination_id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_error_when_invalid_status_on_update_nomination_as_a_guardian() {
    let (client, app_state, db_url) = setup().await;

    let guardian_email = "guardian@example.com".to_string();
    let jwt =
        create_verified_account_jwt(&app_state.database, &client, guardian_email.clone()).await;

    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;
    let guardian_id = Uuid::new_v4();

    guardian_repo::create(
        &app_state.database,
        guardian_id,
        guardian_email.clone(),
        Some(account_id.clone()),
        None,
    )
    .await
    .unwrap();

    let nomination_id = Uuid::new_v4();

    nomination_repo::create(
        &app_state.database,
        nomination_id,
        guardian_email.clone(),
        Uuid::new_v4().to_string(),
        guardian_id.to_string(),
        "ACCEPTED".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .put(format!("/guardian/nomination/{}", nomination_id).as_str())
        .body("{\"status\":\"invalid\"}")
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<NominationUpdateResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.nomination_id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_as_a_guardian_list_accounts_im_guardian_for() {
    let (client, app_state, db_url) = setup().await;

    let guardian_email = "guardian@example.com".to_string();
    let jwt =
        create_verified_account_jwt(&app_state.database, &client, guardian_email.clone()).await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;

    let guardian_id = Uuid::new_v4();
    guardian_repo::create(
        &app_state.database,
        guardian_id,
        guardian_email.clone(),
        Some(account_id.clone()),
        None,
    )
    .await
    .unwrap();

    let some_user_account_id = Uuid::new_v4();
    let some_user_email = "someone@example.com".to_string();
    let some_user_wallet_address = "w123".to_string();
    let some_user_eoa_address = "eoa123".to_string();
    account_repo::create(
        &app_state.database,
        some_user_account_id.clone(),
        some_user_email.clone(),
        some_user_wallet_address.clone(),
        some_user_eoa_address.clone(),
        get_unix_timestamp_ms(),
    )
    .await
    .unwrap();

    let nomination_id = Uuid::new_v4();

    nomination_repo::create(
        &app_state.database,
        nomination_id,
        guardian_email.clone(),
        some_user_account_id.clone().to_string(),
        guardian_id.to_string(),
        "ACCEPTED".to_string(),
    )
    .await
    .unwrap();

    guardian_account_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_id.to_string(),
        some_user_account_id.to_string(),
        "AVAILABLE".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .get("/guardian/accounts")
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<ListGuardianAccountsResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
    ".**.id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_as_a_guardian_list_accounts_im_guardian_for_filtered_by_account_id() {
    let (client, app_state, db_url) = setup().await;

    let guardian_email = "guardian@example.com".to_string();
    let jwt =
        create_verified_account_jwt(&app_state.database, &client, guardian_email.clone()).await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;

    let guardian_id = Uuid::new_v4();
    guardian_repo::create(
        &app_state.database,
        guardian_id,
        guardian_email.clone(),
        Some(account_id.clone()),
        None,
    )
    .await
    .unwrap();

    let some_user_account_id = Uuid::new_v4();
    let some_user_email = "someone@example.com".to_string();
    let some_user_wallet_address = "w123".to_string();
    let some_user_eoa_address = "eoa123".to_string();
    account_repo::create(
        &app_state.database,
        some_user_account_id.clone(),
        some_user_email.clone(),
        some_user_wallet_address.clone(),
        some_user_eoa_address.clone(),
        get_unix_timestamp_ms(),
    )
    .await
    .unwrap();

    let nomination_id = Uuid::new_v4();

    nomination_repo::create(
        &app_state.database,
        nomination_id,
        guardian_email.clone(),
        some_user_account_id.clone().to_string(),
        guardian_id.to_string(),
        "ACCEPTED".to_string(),
    )
    .await
    .unwrap();

    guardian_account_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        guardian_id.to_string(),
        some_user_account_id.to_string(),
        "AVAILABLE".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .get(format!("/guardian/accounts?account_id={}", some_user_account_id).as_str())
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<ListGuardianAccountsResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
    ".**.id" => "[uuid]",
    });

    tear_down(db_url).await;
}
