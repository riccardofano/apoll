use crate::helpers::TestApp;

#[tokio::test]
async fn create_poll_returns_200_with_valid_form_data() {
    let app = TestApp::new().await;
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "username": "username",
        "prompt": "Is this a good prompt?",
    });

    let response = client
        .post(app.endpoint("/new"))
        .form(&body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(response.status().as_u16(), 200);
}
