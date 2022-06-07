use actix_web::{web, HttpResponse, ResponseError};
use reqwest::StatusCode;
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

pub async fn create_poll(form: web::Form<PollFormData>) -> Result<HttpResponse, CreatePollError> {
    let _form = form.validate().map_err(CreatePollError::ValidationError)?;

    Ok(HttpResponse::Ok().finish())
}
