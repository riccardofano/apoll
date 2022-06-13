use actix_web::{http::header::ContentType, web, HttpResponse, ResponseError};
use anyhow::Context;
use reqwest::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum ShowPollError {
    #[error("could not find poll")]
    InvalidPollError,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for ShowPollError {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            ShowPollError::InvalidPollError => StatusCode::NOT_FOUND,
            ShowPollError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(
    name = "Show poll page"
    skip_all,
    fields(poll_id=tracing::field::Empty)
)]
pub async fn show_poll(
    path: web::Path<Uuid>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, ShowPollError> {
    let poll_id = path.into_inner();
    tracing::Span::current().record("poll_id", &tracing::field::display(&poll_id));

    let prompt = validate_poll_id(&db_pool, &poll_id)
        .await
        .context("failed to query database for poll_id")?
        .ok_or(ShowPollError::InvalidPollError)?;

    let poll_users = get_poll_users(&db_pool, &poll_id)
        .await
        .context("failed to retrieve poll users")?;

    let users_li = poll_users
        .iter()
        .map(|u| format!("<li>{}</li>", u.username))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8">
        <title>Login</title>
    </head>
    <body>
        <h1>{prompt}</h1>
        <h2>Users</h2>
        <ul>
            {users_li}
        </ul>
    </body>
</html>"#
        )))
}

#[tracing::instrument(
    name = "retrieve pool details from database"
    skip(db_pool)
)]
async fn validate_poll_id(db_pool: &PgPool, poll_id: &Uuid) -> Result<Option<String>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT prompt
        FROM polls
        WHERE poll_id = $1
        "#,
        poll_id
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(result.map(|r| r.prompt))
}

struct User {
    user_id: Uuid,
    username: String,
}

async fn get_poll_users(db_pool: &PgPool, poll_id: &Uuid) -> Result<Vec<User>, sqlx::Error> {
    let rows = sqlx::query_as!(
        User,
        r#"
        SELECT user_id, username
        FROM poll_users
        WHERE poll_id = $1
        "#,
        poll_id
    )
    .fetch_all(db_pool)
    .await?;

    Ok(rows)
}
