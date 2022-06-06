use apoll::{configuration::Settings, startup::run};
use sqlx::postgres::PgPoolOptions;

use std::{net::TcpListener, time::Duration};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = Settings::new().expect("failed to read configuration");
    let connection_pool = PgPoolOptions::new()
        .connect_timeout(Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let address = configuration.address();
    let listener = TcpListener::bind(address)?;

    let server = run(listener, connection_pool).await?;
    server.await?;

    Ok(())
}
