use actix_web::{web, HttpResponse};
use uuid::Uuid;

pub async fn show_poll(path: web::Path<Uuid>) -> HttpResponse {
    HttpResponse::Ok().body(path.into_inner().to_string())
}
