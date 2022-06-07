use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use reqwest::StatusCode;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::domain::PollFormData;

#[derive(thiserror::Error, Debug)]
pub enum CreatePollError {
    #[error("{0}")]
    ValidationError(ValidationErrors),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for CreatePollError {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            CreatePollError::ValidationError(_) => StatusCode::BAD_REQUEST,
            CreatePollError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn create_poll(
    form: web::Form<PollFormData>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, CreatePollError> {
    let _form = form.validate().map_err(CreatePollError::ValidationError)?;

    let mut transaction = db_pool
        .begin()
        .await
        .context("failed to begin Postgres transaction from pool")?;

    let user_id = insert_new_user(&mut transaction)
        .await
        .context("could not insert new user")?;

    transaction
        .commit()
        .await
        .context("could not commit Postgres transaction")?;

    Ok(HttpResponse::Ok().finish())
}

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
