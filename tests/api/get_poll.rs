use fake::{
    faker::{lorem::en::Sentence, name::en::FirstName},
    Fake,
};
use uuid::Uuid;

use crate::helpers::TestApp;

#[tokio::test]
async fn page_should_return_404_if_path_is_invalid_uuid() {
    let app = TestApp::new().await;
    let client = reqwest::Client::new();

    let poll_id = "1";

    let response = client
        .get(app.endpoint(&format!("/poll/{poll_id}")))
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(response.status().as_u16(), 404)
}

#[tokio::test]
async fn page_should_return_404_if_poll_doesnt_exist() {
    let app = TestApp::new().await;
    let client = reqwest::Client::new();

    let poll_id = Uuid::new_v4();

    let response = client
        .get(app.endpoint(&format!("/poll/{poll_id}")))
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(response.status().as_u16(), 404)
}

#[tokio::test]
async fn page_should_return_200_if_path_is_a_valid_poll() {
    let app = TestApp::new().await;
    let client = reqwest::Client::new();

    let username: String = FirstName().fake();
    let poll_id = app.post_create_poll("Poll question?", &username).await;

    let response = client
        .get(app.endpoint(&format!("/poll/{poll_id}")))
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn page_should_display_creator_and_prompt() {
    let app = TestApp::new().await;
    let client = reqwest::Client::new();

    let prompt: String = Sentence(1..10).fake();
    let username: String = FirstName().fake();
    let poll_id = app.post_create_poll(&prompt, &username).await;

    let response_text = client
        .get(app.endpoint(&format!("/poll/{poll_id}")))
        .send()
        .await
        .expect("failed to send request")
        .text()
        .await
        .unwrap();

    assert!(response_text.contains(&username));
    assert!(response_text.contains(&prompt));
}
