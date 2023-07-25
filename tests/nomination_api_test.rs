use hyper::{header::AUTHORIZATION, StatusCode};
use lib::{
    models::api::{
        ApiErrorResponse, ApiPayload, ApiResponse, ListNominationsResponse,
        NominationCreateResponse, NominationDeleteResponse,
    },
    operations::jwt::{decode_jwt, generate_jwt},
    repos::{guardian_repo, nomination_repo},
    test::utils::{create_verified_account_jwt, setup, tear_down},
};
use uuid::Uuid;

#[tokio::test]
async fn test_invalid_email_format() {
    let (client, app_state, db_url) = setup().await;

    let jwt = create_verified_account_jwt(
        &app_state.database,
        &client,
        "someone@example.com".to_string(),
    )
    .await;

    let res = client
        .post("/accounts/nominations")
        .body("{\"email\":\"a\"}")
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);
    insta::assert_yaml_snapshot!(res.text().await);

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_account_not_found() {
    let (client, app_state, db_url) = setup().await;

    create_verified_account_jwt(
        &app_state.database,
        &client,
        "someone@example.com".to_string(),
    )
    .await;

    let jwt = generate_jwt("does_not_exist".to_string()).await.unwrap();

    let res = client
        .post("/accounts/nominations")
        .body("{\"email\":\"guardian@example.com\"}")
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);
    insta::assert_yaml_snapshot!(res.text().await);

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_create_nomination_when_guardian_exists() {
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
        None,
        None,
    )
    .await
    .unwrap();

    let res = client
        .post("/accounts/nominations")
        .body(format!("{{\"email\":\"{}\"}}", guardian_email.clone()))
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<NominationCreateResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.nomination_id" => "[uuid]"
    });

    let nomination_id = match json_response.payload {
        ApiPayload::Success(res) => res.nomination_id,
        ApiPayload::Error(e) => panic!("Error: {:?}", e),
    };
    let nomination = nomination_repo::find_by_account_and_id(
        &app_state.database,
        account_id.clone(),
        nomination_id,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(nomination.guardian_id, guardian_id.to_string());
    assert_eq!(nomination.email, guardian_email.clone());
    assert_eq!(nomination.account_id, account_id.clone());

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_create_nomination_when_guardian_does_not_exists_but_a_clutch_user_exists() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;

    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;

    let guardian_email = "guardian@example.com".to_string();
    let guardian_jwt =
        create_verified_account_jwt(&app_state.database, &client, guardian_email.clone()).await;
    let guardian_account_id = decode_jwt(guardian_jwt.clone()).await.unwrap().sub;

    let res = client
        .post("/accounts/nominations")
        .body(format!("{{\"email\":\"{}\"}}", guardian_email.clone()))
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt.clone()))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<NominationCreateResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.nomination_id" => "[uuid]"
    });

    let nomination_id = match json_response.payload {
        ApiPayload::Success(res) => res.nomination_id,
        ApiPayload::Error(e) => panic!("Error: {:?}", e),
    };

    let guardian = guardian_repo::find_by_email(&app_state.database, guardian_email.clone())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(guardian.account_id.unwrap(), guardian_account_id);

    let nomination = nomination_repo::find_by_account_and_id(
        &app_state.database,
        account_id.clone(),
        nomination_id,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(nomination.guardian_id, guardian.id.to_string());
    assert_eq!(nomination.email, guardian_email.clone());
    assert_eq!(nomination.account_id, account_id.clone());

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_create_nomination_when_guardian_does_not_exists_and_clutch_user_does_not_exist() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;

    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;

    let guardian_email = "guardian@example.com".to_string();

    let res = client
        .post("/accounts/nominations")
        .body(format!("{{\"email\":\"{}\"}}", guardian_email.clone()))
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt.clone()))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<NominationCreateResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.nomination_id" => "[uuid]"
    });

    let nomination_id = match json_response.payload {
        ApiPayload::Success(res) => res.nomination_id,
        ApiPayload::Error(e) => panic!("Error: {:?}", e),
    };

    let guardian = guardian_repo::find_by_email(&app_state.database, guardian_email.clone())
        .await
        .unwrap();
    assert!(guardian.is_some());

    let nomination = nomination_repo::find_by_account_and_id(
        &app_state.database,
        account_id.clone(),
        nomination_id,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(nomination.guardian_id, guardian.unwrap().id.to_string());
    assert_eq!(nomination.email, guardian_email.clone());
    assert_eq!(nomination.account_id, account_id.clone());

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_find_all_nominations() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;
    let guardian_email = "guardian@example.com".to_string();

    let nomination_id = Uuid::new_v4();
    nomination_repo::create(
        &app_state.database,
        nomination_id,
        guardian_email,
        account_id.clone(),
        "guardian_id".to_string(),
        "PENDING".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .get("/accounts/nominations")
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
        ".**.account_id" => "[uuid]"
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_find_all_by_id() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;
    let nomination_id_1 = Uuid::new_v4();

    nomination_repo::create(
        &app_state.database,
        nomination_id_1,
        "guardian1@example.com".to_string(),
        account_id.clone(),
        "guardian_id1".to_string(),
        "PENDING".to_string(),
    )
    .await
    .unwrap();
    nomination_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        "guardian2@example.com".to_string(),
        account_id.clone(),
        "guardian_id2".to_string(),
        "PENDING".to_string(),
    )
    .await
    .unwrap();
    nomination_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        "guardian3@example.com".to_string(),
        account_id.clone(),
        "guardian_id3".to_string(),
        "PENDING".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .get(format!("/accounts/nominations?nomination_id={}", nomination_id_1).as_str())
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
        ".**.account_id" => "[uuid]"
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_find_all_by_status() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;
    let nomination_id_1 = Uuid::new_v4();

    nomination_repo::create(
        &app_state.database,
        nomination_id_1,
        "guardian1@example.com".to_string(),
        account_id.clone(),
        "guardian_id1".to_string(),
        "PENDING".to_string(),
    )
    .await
    .unwrap();
    nomination_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        "guardian2@example.com".to_string(),
        account_id.clone(),
        "guardian_id2".to_string(),
        "ACCEPTED".to_string(),
    )
    .await
    .unwrap();
    nomination_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        "guardian3@example.com".to_string(),
        account_id.clone(),
        "guardian_id3".to_string(),
        "REJECTED".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .get("/accounts/nominations?status=pending")
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
        ".**.account_id" => "[uuid]"
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_find_all_by_email() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;
    let nomination_id_1 = Uuid::new_v4();

    nomination_repo::create(
        &app_state.database,
        nomination_id_1,
        "guardian1@example.com".to_string(),
        account_id.clone(),
        "guardian_id1".to_string(),
        "PENDING".to_string(),
    )
    .await
    .unwrap();
    nomination_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        "guardian2@example.com".to_string(),
        account_id.clone(),
        "guardian_id2".to_string(),
        "ACCEPTED".to_string(),
    )
    .await
    .unwrap();
    nomination_repo::create(
        &app_state.database,
        Uuid::new_v4(),
        "guardian3@example.com".to_string(),
        account_id.clone(),
        "guardian_id3".to_string(),
        "REJECTED".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .get("/accounts/nominations?email=guardian3@example.com")
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
        ".**.account_id" => "[uuid]"
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_successfully_delete_nomination() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;
    let nomination_id_1 = Uuid::new_v4();

    nomination_repo::create(
        &app_state.database,
        nomination_id_1,
        "guardian1@example.com".to_string(),
        account_id.clone(),
        "guardian_id1".to_string(),
        "PENDING".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .delete(format!("/accounts/nominations/{}", nomination_id_1).as_str())
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<NominationDeleteResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.nomination_id" => "[uuid]",
    });

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_error_on_delete_nomination_if_status_is_accepted_or_rejected() {
    let (client, app_state, db_url) = setup().await;

    let jwt =
        create_verified_account_jwt(&app_state.database, &client, "user@example.com".to_string())
            .await;
    let account_id = decode_jwt(jwt.clone()).await.unwrap().sub;
    let nomination_id_1 = Uuid::new_v4();
    let nomination_id_2 = Uuid::new_v4();

    nomination_repo::create(
        &app_state.database,
        nomination_id_1,
        "guardian1@example.com".to_string(),
        account_id.clone(),
        "guardian_id1".to_string(),
        "ACCEPTED".to_string(),
    )
    .await
    .unwrap();

    nomination_repo::create(
        &app_state.database,
        nomination_id_2,
        "guardian2@example.com".to_string(),
        account_id.clone(),
        "guardian_id2".to_string(),
        "REJECTED".to_string(),
    )
    .await
    .unwrap();

    let res = client
        .delete(format!("/accounts/nominations/{}", nomination_id_1).as_str())
        .header("Content-Type", "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", jwt))
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let json_response = res
        .json::<ApiResponse<NominationDeleteResponse, ApiErrorResponse>>()
        .await;

    insta::assert_yaml_snapshot!(json_response, {
        ".**.nomination_id" => "[uuid]",
    });

    tear_down(db_url).await;
}
