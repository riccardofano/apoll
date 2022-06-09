use actix_web::{web, HttpResponse, ResponseError};
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

    let prompt = validate_poll_id(&db_pool, poll_id)
        .await
        .context("failed to query database for poll_id")?
        .ok_or(ShowPollError::InvalidPollError)?;

    Ok(HttpResponse::Ok().body(prompt))
}

#[tracing::instrument(
    name = "retrieve pool details from database"
    skip(db_pool)
)]
pub async fn validate_poll_id(
    db_pool: &PgPool,
    poll_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
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
