use crate::helpers::TestApp;

#[tokio::test]
async fn suggest_returns_200_upon_successful_request() {
    let app = TestApp::new().await;

    let poll_id = app.post_create_poll("prompt", "username").await;

    let response = app
        .api_client
        .post(app.endpoint(&format!("/poll/{poll_id}/suggest")))
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(response.status().as_u16(), 200);
}
