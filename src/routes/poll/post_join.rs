use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use reqwest::{header::LOCATION, StatusCode};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{middleware::PollInfo, user_session::TypedSession};

#[derive(thiserror::Error, Debug)]
pub enum JoinError {
    #[error("pool does not exist")]
    NotFoundError,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for JoinError {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            JoinError::NotFoundError => StatusCode::NOT_FOUND,
            JoinError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(serde::Deserialize)]
pub struct JoinForm {
    username: String,
}

#[tracing::instrument(
    name = "let new user join a poll"
    skip_all,
    fields(
        user_name = tracing::field::Empty,
        user_id = tracing::field::Empty
    )
)]
pub async fn join_poll(
    form: web::Form<JoinForm>,
    db_pool: web::Data<PgPool>,
    poll_info: PollInfo,
    session: TypedSession,
) -> Result<HttpResponse, JoinError> {
    tracing::Span::current().record("poll_id", &tracing::field::display(&poll_info.poll_id));
    tracing::Span::current().record("user_name", &tracing::field::display(&form.0.username));

    // Reject user if they're already logged in
    if session
        .get_user_id()
        .map_err(|e| JoinError::UnexpectedError(e.into()))?
        .is_some()
    {
        // TODO: add message saying you're already logged in
        return Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, format!("/poll/{}", poll_info.poll_id)))
            .finish());
    };

    let user_id = create_and_insert_user(&db_pool, poll_info.poll_id, form.0.username)
        .await
        .context("failed to create and insert user into the poll")?;

    session.renew();
    session
        .insert_user_id(user_id)
        .context("failed to insert user_id into session store")?;

    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, format!("/poll/{}", poll_info.poll_id)))
        .finish())
}

#[tracing::instrument(
    name = "create and insert user into poll users"
    skip(db_pool)
)]
async fn create_and_insert_user(
    db_pool: &PgPool,
    poll_id: Uuid,
    username: String,
) -> Result<Uuid, sqlx::Error> {
    let mut transaction = db_pool.begin().await?;

    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO users (user_id, created_at)
        VALUES ($1, now())
        "#,
        &user_id
    )
    .execute(&mut transaction)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO poll_users (poll_id, user_id, username)
        VALUES ($1, $2, $3)
        "#,
        poll_id,
        &user_id,
        username
    )
    .execute(&mut transaction)
    .await?;

    transaction.commit().await?;

    Ok(user_id)
}
