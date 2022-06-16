use fake::{faker::name::en::FirstName, Fake};
use reqwest::Response;
use uuid::Uuid;

use crate::helpers::TestApp;

fn location_string(res: Response) -> String {
    res.headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn post_join_should_return_200_ok() {
    let app = TestApp::new().await;

    let poll_id = app.post_create_poll("Test question?", "TestUser").await;

    let username: String = FirstName().fake();
    let body = serde_json::json!({ "username": &username });

    let response = app.join_poll(&poll_id, &body).await;

    assert_eq!(response.status().as_u16(), 303);
    assert!(location_string(response).contains(&poll_id.to_string()));
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
    assert_eq!(response.status().as_u16(), 303);
    assert!(location_string(response).contains(&poll_id.to_string()));

    // Visit poll page
    let response = app.get_poll_page(&poll_id.to_string()).await;
    assert_eq!(response.status().as_u16(), 200);

    // Assert username is in the poll page
    let text = response.text().await.unwrap();
    assert!(text.contains(&username));

    // Assert greeting is displayed
    assert!(text.contains(&format!("<p>Logged in as {username}</p>")))
}

#[tokio::test]
async fn joined_user_should_be_rejected_if_they_try_to_join_again() {
    let app = TestApp::new().await;

    let poll_id = app.post_create_poll("Test Question", "testuser").await;
    let username: String = FirstName().fake();
    let body = serde_json::json!({ "username": &username });

    // Join poll
    let response = app.join_poll(&poll_id, &body).await;
    assert_eq!(response.status().as_u16(), 303);
    assert!(location_string(response).contains(&poll_id.to_string()));

    // Assert username is in the poll page
    let response = app.get_poll_page(&poll_id.to_string()).await;
    let text = response.text().await.unwrap();
    assert!(text.contains(&username));

    // Join poll again
    let response = app.join_poll(&poll_id, &body).await;
    assert_eq!(response.status().as_u16(), 303);
    assert!(location_string(response).contains(&poll_id.to_string()));

    // Assert username is in the poll page
    let response = app.get_poll_page(&poll_id.to_string()).await;
    let text = response.text().await.unwrap();
    // Count should be 2 (user list and "logged in as user `username`")
    // There shouldn't be 2 instances of the name in the user list
    assert_eq!(text.matches(&username).count(), 2);

    // Assert greeting is displayed
    assert!(text.contains(&format!("<p>Logged in as {username}</p>")))
}
