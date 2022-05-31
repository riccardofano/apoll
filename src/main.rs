use actix_web::{web, App, HttpResponse, HttpServer, Responder};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::get().to(hello)))
        .bind(("127.0.0.1", 8000))?
        .run()
        .await
}

async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello")
}
