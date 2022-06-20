use actix_web::{HttpResponse, ResponseError};
use reqwest::StatusCode;

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

pub async fn suggest_answer() -> Result<HttpResponse, SuggestionError> {
    Ok(HttpResponse::Ok().finish())
}
