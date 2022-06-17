use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

#[tracing::instrument(
    name = "Show new poll page"
    skip_all,
    fields(poll_id=tracing::field::Empty)
)]
pub async fn new_poll(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let error_html = flash_messages
        .iter()
        .map(|m| format!("<p><i>{}</i></p>", m.content()))
        .collect::<Vec<_>>()
        .join("\n");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8">
        <title>Apoll | Create poll</title>
    </head>
    <body>
        {error_html}
        <h1>Create a new poll</h1>
        <form action="/new" method="post">
        <label for="username">Username
            <input type="text" name="username" />
        </label><br>
        <label for="prompt">Poll prompt
            <input type="text" name="prompt" />
        </label><br>
        <button type="submit">Create poll</button>
        </form>
    </body>
</html>"#
        ))
}
