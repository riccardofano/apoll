use actix_web::HttpResponse;

pub async fn create_poll() -> HttpResponse {
    HttpResponse::Ok().finish()
}
