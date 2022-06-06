use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};

pub async fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || App::new().route("/", web::get().to(hello)))
        .listen(listener)?
        .run();

    Ok(server)
}

async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello")
}
