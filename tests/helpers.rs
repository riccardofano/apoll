use std::net::TcpListener;

use apoll::{
    configuration::{DatabaseSettings, Settings},
    startup::run,
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tokio::runtime::Runtime;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_name: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn new() -> Self {
        // Assign a random name and port to the application
        let configuration = {
            let mut c = Settings::new().expect("failed to read configuration");
            c.database.database_name = Uuid::new_v4().to_string();
            c.application.port = 0;
            c
        };

        // Create and migrate the database
        let db_pool = TestApp::configure_database(&configuration.database).await;

        // Run the server
        let listener =
            TcpListener::bind(configuration.address()).expect("failed to bind random port");
        let server = run(listener, db_pool.clone()).expect("failed to bind address");
        let _ = tokio::spawn(server);

        TestApp {
            address: configuration.address(),
            db_name: configuration.database.database_name,
            db_pool,
        }
    }

    async fn configure_database(config: &DatabaseSettings) -> PgPool {
        // Create the database
        let mut connection = PgConnection::connect_with(&config.without_db())
            .await
            .expect("failed to connect to Postgres");
        connection
            .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
            .await
            .expect("failed to create database");

        // Migrate the database
        let db_pool = PgPool::connect_with(config.with_db())
            .await
            .expect("failed to connect to Postgres");
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await
            .expect("failed to migrate the database");

        db_pool
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        let db_name = self.db_name.clone();

        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let config = Settings::new().expect("Failed to read configuration");
                let mut conn = PgConnection::connect_with(&config.database.without_db())
                    .await
                    .expect("Failed to connect to Postgres");

                conn.execute(&*format!(
                    "SELECT pg_terminate_backend(pg_stat_activity.pid)
                    FROM pg_stat_activity
                    WHERE datname = '{}'
                      AND pid <> pg_backend_pid();",
                    db_name
                ))
                .await
                .expect("Failed to disconnect other sessions");

                conn.execute(&*format!("DROP DATABASE \"{}\";", db_name))
                    .await
                    .unwrap_or_else(|_| panic!("Failed to drop temporary database: {}", db_name));
                // TODO: replace this with tracer
                println!("Dropped database: {db_name}");
                let _ = tx.send(());
            })
        });

        let _ = rx.recv();
        // TODO: replace this with tracer
        println!("ran test teardown");
    }
}
