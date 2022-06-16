use apoll::{
    configuration::Settings,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set up tracing
    let subscriber = get_subscriber("apoll".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = Settings::new().expect("failed to read configuration");
    let application = Application::build(configuration.clone()).await?;

    application.run_until_stopped().await?;

    Ok(())
}
