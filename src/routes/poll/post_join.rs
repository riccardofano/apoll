use actix_web::{web, HttpResponse, ResponseError};
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum JoinError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for JoinError {
    fn status_code(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub async fn join_poll(_path: web::Path<Uuid>) -> Result<HttpResponse, JoinError> {
    Ok(HttpResponse::Ok().finish())
}
