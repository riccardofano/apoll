use actix_web::{error::InternalError, web, HttpResponse, ResponseError};
use actix_web_lab::__reexports::futures_util::TryFutureExt;
use reqwest::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{middleware::PollInfo, user_session::TypedSession, utils::flash_message_redirect};

#[derive(thiserror::Error, Debug)]
pub enum SuggestionError {
    #[error("You must be logged in to suggest an answer")]
    Unauthorized,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl ResponseError for SuggestionError {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            SuggestionError::Unauthorized => StatusCode::UNAUTHORIZED,
            SuggestionError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(serde::Deserialize)]
pub struct SuggestionForm {
    pub suggestion: String,
}

#[tracing::instrument(
    name = "suggest new answer"
    skip_all
    fields(poll_id = tracing::field::Empty, user_id = tracing::field::Empty)
)]
pub async fn suggest_answer(
    poll_info: PollInfo,
    session: TypedSession,
    db_pool: web::Data<PgPool>,
    form: web::Form<SuggestionForm>,
) -> Result<HttpResponse, InternalError<SuggestionError>> {
    let poll_id = poll_info.poll_id;
    tracing::Span::current().record("poll_id", &tracing::field::display(&poll_id));

    let poll_uri = &format!("/poll/{poll_id}");
    let user_id = session
        .get_user_id()
        .map_err(|e| flash_message_redirect(SuggestionError::Unexpected(e.into()), poll_uri))?
        .ok_or_else(|| flash_message_redirect(SuggestionError::Unauthorized, poll_uri))?;

    insert_suggestion(&db_pool, &poll_id, &user_id, form.0.suggestion)
        .await
        .map_err(|e| flash_message_redirect(SuggestionError::Unexpected(e.into()), poll_uri))?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "insert new suggestion"
    skip(_db_pool)
)]
async fn insert_suggestion(
    _db_pool: &PgPool,
    _poll_id: &Uuid,
    _user_id: &Uuid,
    _suggestion: String,
) -> Result<(), sqlx::Error> {
    todo!()
}
