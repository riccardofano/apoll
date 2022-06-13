use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use reqwest::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

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
    path: web::Path<Uuid>,
    form: web::Form<JoinForm>,
    db_poll: web::Data<PgPool>,
) -> Result<HttpResponse, JoinError> {
    let poll_id = path.into_inner();
    tracing::Span::current().record("user_id", &tracing::field::display(&poll_id));
    tracing::Span::current().record("user_name", &tracing::field::display(&form.0.username));

    let prompt = validate_poll_id(&db_poll, poll_id)
        .await
        .context("failed to retrieve poll_id")?;

    if prompt.is_none() {
        return Err(JoinError::NotFoundError);
    }

    Ok(HttpResponse::Ok().finish())
}

// TODO: replace this with a middleware
#[tracing::instrument(
    name = "validate prompt existance"
    skip(db_poll),
)]
async fn validate_poll_id(db_poll: &PgPool, poll_id: Uuid) -> Result<Option<String>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT prompt
        FROM polls
        WHERE poll_id = $1
        "#,
        poll_id
    )
    .fetch_optional(db_poll)
    .await?;

    Ok(result.map(|r| r.prompt))
}
