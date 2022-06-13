use crate::helpers::TestApp;

#[tokio::test]
async fn post_join_should_return_200_ok() {
    let app = TestApp::new().await;
    let client = reqwest::Client::new();

    let poll_id = app.post_create_poll("Test question?", "TestUser").await;

    let response = client
        .post(&app.endpoint(&format!("/poll/{poll_id}/join")))
        .body("")
        .send()
        .await
        .expect("could not send join request");

    assert_eq!(response.status().as_u16(), 200);
}
