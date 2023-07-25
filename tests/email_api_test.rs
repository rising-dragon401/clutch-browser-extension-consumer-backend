use axum::http::StatusCode;
use axum_test_helper::{TestClient, TestResponse};
use lib::test::utils::setup;
use lib::test::utils::tear_down;

#[tokio::test]
async fn test_verify_email() {
    let (client, _app_state, db_url) = setup().await;

    let res = client
        .post("/email/verify")
        .body("{\"email\":\"a@me.com\"}")
        .header("Content-Type", "application/json")
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);
    insta::assert_yaml_snapshot!(res.text().await);

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_invalid_email_format() {
    let (client, _app_state, db_url) = setup().await;

    let res = client
        .post("/email/verify")
        .body("{\"email\":\"a\"}")
        .header("Content-Type", "application/json")
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);
    insta::assert_yaml_snapshot!(res.text().await);

    tear_down(db_url).await;
}

#[tokio::test]
async fn test_error_when_exceeds_email_send_limit() {
    let (client, _app_state, db_url) = setup().await;

    async fn verify_email(client: &TestClient) -> TestResponse {
        client
            .post("/email/verify")
            .body("{\"email\":\"exceeds@example.com\"}")
            .header("Content-Type", "application/json")
            .send()
            .await
    }

    for _ in 0..5 {
        verify_email(&client).await;
    }

    let res = verify_email(&client).await;
    assert_eq!(res.status(), StatusCode::OK);
    insta::assert_yaml_snapshot!(res.text().await);

    tear_down(db_url).await;
}
