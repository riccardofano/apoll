use crate::helpers::{location_string, TestApp};

#[tokio::test]
async fn suggest_redirects_to_poll_page_upon_successful_request() {
    let app = TestApp::new().await;

    let poll_id = app.post_create_poll("prompt", "username").await;

    app.join_poll(&poll_id, &serde_json::json!({"username": "newuser"}))
        .await;

    let response = app
        .post_suggestion(
            &poll_id,
            &serde_json::json!({"suggestion": "my suggestion"}),
        )
        .await;

    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(&location_string(response), &format!("/poll/{poll_id}"))
}

#[tokio::test]
async fn suggestions_gets_displayed_on_page_after_insertion() {
    let app = TestApp::new().await;

    let poll_id = app.post_create_poll("prompt", "username").await;

    app.join_poll(&poll_id, &serde_json::json!({"username": "newuser"}))
        .await;

    let suggestion = "my brilliant suggestion";
    let body = serde_json::json!({ "suggestion": suggestion });
    let response = app.post_suggestion(&poll_id, &body).await;

    // Redirected to poll page
    assert_eq!(response.status().as_u16(), 303);

    // Get poll page
    let response = app.get_poll_page(&poll_id.to_string()).await;
    let text = response.text().await.unwrap();

    // New suggestion should appear in the page now
    assert!(text.contains(suggestion));
}
