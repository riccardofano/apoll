use actix_web::HttpResponse;

pub struct FormData {
    pub username: String,
    pub prompt: String,
}

pub async fn create_poll() -> HttpResponse {
    HttpResponse::Ok().finish()
}
