use crate::helpers::TestApp;
use claim::assert_ok;
use uuid::Uuid;

#[tokio::test]
async fn create_poll_returns_200_with_valid_form_data() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "username": "username",
        "prompt": "Is this a good prompt?",
    });

    let response = app
        .api_client
        .post(app.endpoint("/new"))
        .form(&body)
        .send()
        .await
        .expect("failed to execute request");

    // Response was redirected
    assert_eq!(response.status().as_u16(), 303);

    let location = response
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    let poll_id = &location.replace("/poll/", "");
    // To a valid poll_id
    assert_ok!(Uuid::parse_str(poll_id));
}

#[tokio::test]
async fn create_poll_should_display_message_on_invalid_data() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "username": "username",
        "prompt": ""
    });

    let response = app
        .api_client
        .post(app.endpoint("/new"))
        .form(&body)
        .send()
        .await
        .expect("failed to execute request");

    // Should be redirected to the create poll page
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("location").unwrap(), "/");

    let response = app
        .api_client
        .get(app.endpoint("/"))
        .send()
        .await
        .expect("failed to execute reqeust");

    // Page should display the message
    assert_eq!(response.status().as_u16(), 200);
    let text = response.text().await.unwrap();
    assert!(text.contains("<p><i>prompt: length is invalid.</i></p>"));
}
