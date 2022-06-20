use actix_web::{http::header::ContentType, web, HttpResponse, ResponseError};
use anyhow::Context;
use reqwest::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{middleware::PollInfo, user_session::TypedSession};

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
    db_pool: web::Data<PgPool>,
    poll_info: PollInfo,
    session: TypedSession,
) -> Result<HttpResponse, ShowPollError> {
    let PollInfo { poll_id, prompt } = poll_info;
    tracing::Span::current().record("poll_id", &tracing::field::display(&poll_id));

    let mut user_greeting = String::new();
    let mut suggest_form = String::new();
    let mut join_form = String::new();
    if let Some(user) = get_session_user(session, &db_pool, &poll_id).await? {
        user_greeting = format!("<p>Logged in as {}</p>", user.username);
        suggest_form = format!(
            r#"<form action="/poll/{poll_id}/suggest" method="post">
                <input type="text" placeholder="Add suggestion" name="suggestion" />
                <button type="submit">Add Suggestion</button>
            </form>"#
        );
    } else {
        join_form = format!(
            r#"<form action="/poll/{poll_id}/join" method="post">
                <input type="text" placeholder="Username" name="username" />
                <button type="submit">Join poll</button>
            </form>"#
        )
    }

    // Retrieve users
    let poll_users = get_poll_users(&db_pool, &poll_id)
        .await
        .context("failed to retrieve poll users")?;
    let users_li = poll_users
        .iter()
        .map(|u| format!("<li>{}</li>", u.username))
        .collect::<Vec<_>>()
        .join("\n");

    // Retrieve suggestions
    let suggestions = get_suggestions(&db_pool, &poll_id)
        .await
        .context("failed to retrieve suggestions")?;
    let suggestions_li = suggestions
        .iter()
        .map(|s| format!("<li>{s}</li>"))
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
        {user_greeting}
        <h1>{prompt}</h1>
        {join_form}
        {suggest_form}
        <h2>Users</h2>
        <ul>
            {users_li}
        </ul>
        <h2>Suggestions</h2>
        <ul>
            {suggestions_li}
        </ul>
    </body>
</html>"#
        )))
}

#[derive(Debug)]
struct User {
    // TODO: remove this when user_id gets used
    #[allow(dead_code)]
    user_id: Uuid,
    username: String,
}

#[tracing::instrument(name = "get user from session", skip(session, db_pool))]
async fn get_session_user(
    session: TypedSession,
    db_pool: &PgPool,
    poll_id: &Uuid,
) -> Result<Option<User>, anyhow::Error> {
    let user_id = session
        .get_user_id()
        .context("failed to retrieve user_id from session store")?;

    let user = match user_id {
        Some(user_id) => get_user_from_id(db_pool, &user_id, poll_id)
            .await
            .context("failed to retrieve user from database")?,
        None => return Ok(None),
    };

    Ok(user)
}

#[tracing::instrument(name = "retrieve logged in user info", skip(db_pool))]
async fn get_user_from_id(
    db_pool: &PgPool,
    user_id: &Uuid,
    poll_id: &Uuid,
) -> Result<Option<User>, sqlx::Error> {
    let result = sqlx::query_as!(
        User,
        r#"
        SELECT user_id, username
        FROM poll_users
        WHERE poll_id = $1 AND user_id = $2
        LIMIT 1
        "#,
        poll_id,
        user_id
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(result)
}

#[tracing::instrument(name = "retrieve poll users", skip(db_pool))]
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

#[tracing::instrument(name = "retrieve poll suggestions", skip(db_pool))]
async fn get_suggestions(db_pool: &PgPool, poll_id: &Uuid) -> Result<Vec<String>, sqlx::Error> {
    struct Row {
        suggestion: String,
    }

    let rows = sqlx::query_as!(
        Row,
        r#"
        SELECT (suggestion)
        FROM suggestions
        WHERE poll_id = $1
        "#,
        poll_id
    )
    .fetch_all(db_pool)
    .await?;

    let suggestions = rows.into_iter().map(|r| r.suggestion).collect();
    Ok(suggestions)
}
