use apoll::{
    configuration::Settings,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::postgres::PgPoolOptions;

use std::{net::TcpListener, time::Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set up tracing
    let subscriber = get_subscriber("apoll".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = Settings::new().expect("failed to read configuration");
    let connection_pool = PgPoolOptions::new()
        .connect_timeout(Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let address = configuration.address();
    let listener = TcpListener::bind(address)?;

    let server = run(
        listener,
        connection_pool,
        configuration.application.hmac_secret,
        configuration.redis_uri,
    )
    .await?;

    server.await?;

    Ok(())
}
