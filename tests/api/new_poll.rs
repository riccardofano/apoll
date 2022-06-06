use crate::helpers::TestApp;

#[tokio::test]
async fn create_poll_returns_status_200() {
    let app = TestApp::new().await;
    let client = reqwest::Client::new();

    let response = client
        .post(app.endpoint("/new"))
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(response.status().as_u16(), 200);
}
