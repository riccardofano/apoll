use apoll::startup::run;

use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let address = format!("127.0.0.1:{}", 3000);
    let listener = TcpListener::bind(address)?;

    let server = run(listener).await?;
    server.await?;

    Ok(())
}
