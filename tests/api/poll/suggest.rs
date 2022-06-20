use crate::helpers::{location_string, TestApp};

#[tokio::test]
async fn suggest_returns_200_upon_successful_request() {
    let app = TestApp::new().await;

    let poll_id = app.post_create_poll("prompt", "username").await;

    app.join_poll(&poll_id, &serde_json::json!({"username": "newuser"}))
        .await;

    let response = app
        .api_client
        .post(app.endpoint(&format!("/poll/{poll_id}/suggest")))
        .form(&serde_json::json!({ "suggestion": "suggestion" }))
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(&location_string(response), &format!("/poll/{poll_id}"))
}
