use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::routes::poll::{create_poll, join_poll, show_poll};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/", web::get().to(hello))
            .route("/new", web::post().to(create_poll))
            .service(
                web::scope("/poll")
                    .route("/{poll_id}", web::get().to(show_poll))
                    .route("/{poll_id}/join", web::post().to(join_poll)),
            )
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello")
}
