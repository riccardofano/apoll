use actix_web::{web, HttpResponse};
use validator::Validate;

use crate::domain::PollFormData;

pub async fn create_poll(form: web::Form<PollFormData>) -> HttpResponse {
    let _form = match form.validate() {
        Ok(form) => form,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    HttpResponse::Ok().finish()
}
