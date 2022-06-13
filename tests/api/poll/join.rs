use fake::{faker::name::en::FirstName, Fake};
use uuid::Uuid;

use crate::helpers::TestApp;

#[tokio::test]
async fn post_join_should_return_200_ok() {
    let app = TestApp::new().await;

    let poll_id = app.post_create_poll("Test question?", "TestUser").await;

    let username: String = FirstName().fake();
    let body = serde_json::json!({ "username": &username });

    let response = app.join_poll(&poll_id, &body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn join_should_return_404_if_poll_id_does_not_exist() {
    let app = TestApp::new().await;

    let username: String = FirstName().fake();
    let body = serde_json::json!({ "username": &username });

    let poll_id = Uuid::new_v4();
    let response = app.join_poll(&poll_id, &body).await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn joined_user_should_appear_in_the_poll_page() {
    let app = TestApp::new().await;

    let poll_id = app.post_create_poll("Test Question", "testuser").await;
    let username: String = FirstName().fake();
    let body = serde_json::json!({ "username": &username });

    // Join poll
    let response = app.join_poll(&poll_id, &body).await;
    assert_eq!(response.status().as_u16(), 200);

    // Visit poll page
    let response = app
        .api_client
        .get(app.endpoint(&format!("/poll/{poll_id}")))
        .send()
        .await
        .expect("failed to send request");
    assert_eq!(response.status().as_u16(), 200);

    // Assert username is in the poll page
    let text = response.text().await.unwrap();
    assert!(text.contains(&username));
}
