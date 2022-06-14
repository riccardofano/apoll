use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};
use actix_web_lab::middleware::from_fn;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::{
    middleware::validate_poll_id,
    routes::poll::{create_poll, join_poll, show_poll},
};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/", web::get().to(hello))
            .route("/new", web::post().to(create_poll))
            .service(
                web::scope("/poll/{poll_id}")
                    .wrap(from_fn(validate_poll_id))
                    .route("", web::get().to(show_poll))
                    .route("/join", web::post().to(join_poll)),
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
