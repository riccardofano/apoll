use actix_web::{error::InternalError, web, HttpResponse, ResponseError};
use reqwest::{header::LOCATION, StatusCode};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::{domain::PollFormData, user_session::TypedSession, utils::flash_message_redirect};

#[derive(thiserror::Error, Debug)]
pub enum CreatePollError {
    #[error("{0}")]
    Validation(ValidationErrors),
    #[error("failed to read user session")]
    Session(#[from] serde_json::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl ResponseError for CreatePollError {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            CreatePollError::Validation(_) => StatusCode::BAD_REQUEST,
            CreatePollError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CreatePollError::Session(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(
    name = "Creating a new poll",
    skip_all,
    fields(
        user_name = %form.username,
        poll_prompt = %form.prompt
    )
)]
pub async fn create_poll(
    form: web::Form<PollFormData>,
    db_pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, InternalError<CreatePollError>> {
    if let Err(e) = form.validate() {
        return Err(flash_message_redirect(CreatePollError::Validation(e), "/"));
    }

    let mut transaction = db_pool.begin().await.map_err(unexpected)?;
    // Create new user
    let user_id = insert_new_user(&mut transaction)
        .await
        .map_err(unexpected)?;
    // Create new poll
    let poll_id = insert_new_poll(&mut transaction, &user_id, form.0.prompt)
        .await
        .map_err(unexpected)?;
    // Create poll_user instance with new user and poll
    link_poll_user(&mut transaction, &poll_id, &user_id, form.0.username)
        .await
        .map_err(unexpected)?;
    transaction.commit().await.map_err(unexpected)?;

    // Log user in
    session.renew();
    session
        .insert_user_id(user_id)
        .map_err(|e| flash_message_redirect(CreatePollError::Session(e), "/"))?;

    let response = HttpResponse::SeeOther()
        .insert_header((LOCATION, format!("/poll/{poll_id}")))
        .finish();
    Ok(response)
}

fn unexpected(e: sqlx::Error) -> InternalError<CreatePollError> {
    flash_message_redirect(CreatePollError::Unexpected(e.into()), "/")
}

#[tracing::instrument(name = "Inserting new poll creator in the database", skip_all)]
async fn insert_new_user(transaction: &mut Transaction<'_, Postgres>) -> Result<Uuid, sqlx::Error> {
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO users (user_id, created_at)
        VALUES ($1, now())
        "#,
        user_id
    )
    .execute(transaction)
    .await?;

    Ok(user_id)
}

#[tracing::instrument(
    name = "Inserting poll details in the database",
    skip_all,
    fields(creator_id = %creator_id, poll_prompt = %prompt)
)]
async fn insert_new_poll(
    transaction: &mut Transaction<'_, Postgres>,
    creator_id: &Uuid,
    prompt: String,
) -> Result<Uuid, sqlx::Error> {
    let poll_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO polls (poll_id, creator_id, prompt, created_at)
        VALUES ($1, $2, $3, now())
        "#,
        poll_id,
        creator_id,
        prompt
    )
    .execute(transaction)
    .await?;

    Ok(poll_id)
}

#[tracing::instrument(
    name = "Linking poll and its creator in the database",
    skip_all,
    fields(poll_id = %poll_id, creator_id = %user_id, user_name = %username)
)]
async fn link_poll_user(
    transaction: &mut Transaction<'_, Postgres>,
    poll_id: &Uuid,
    user_id: &Uuid,
    username: String,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO poll_users (poll_id, user_id, username)
        VALUES ($1, $2, $3)
        "#,
        poll_id,
        user_id,
        username
    )
    .execute(transaction)
    .await?;

    Ok(())
}
