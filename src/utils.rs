use actix_web::{error::InternalError, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use reqwest::header::LOCATION;

pub fn redirect(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}

pub fn flash_message_redirect<E: std::error::Error>(e: E, location: &str) -> InternalError<E> {
    FlashMessage::error(e.to_string()).send();
    InternalError::from_response(e, redirect(location))
}
