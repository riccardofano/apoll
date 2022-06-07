use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};
use sqlx::PgPool;

use crate::routes::create_poll;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(hello))
            .route("/new", web::post().to(create_poll))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello")
}
