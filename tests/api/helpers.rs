use std::net::TcpListener;

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tokio::runtime::Runtime;
use tracing::info;
use uuid::Uuid;

use apoll::configuration::{DatabaseSettings, Settings};
use apoll::startup::run;
use apoll::telemetry::{get_subscriber, init_subscriber};

// Only initialize tracing once
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_name: String,
    pub db_pool: PgPool,
    pub api_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        // Start tracing
        Lazy::force(&TRACING);

        // Assign a random name and port to the application
        let configuration = {
            let mut c = Settings::new().expect("failed to read configuration");
            c.database.database_name = Uuid::new_v4().to_string();
            c.application.port = 0;
            c
        };

        // Create and migrate the database
        let db_pool = TestApp::configure_database(&configuration.database).await;

        // Create API client
        let api_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .cookie_store(true)
            .build()
            .unwrap();

        // Run the server
        let listener =
            TcpListener::bind(configuration.address()).expect("failed to bind random port");
        let application_port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            db_pool.clone(),
            configuration.application.hmac_secret,
            configuration.redis_uri,
        )
        .await
        .expect("failed to bind address");
        let _ = tokio::spawn(server);

        TestApp {
            address: format!("http://localhost:{}", application_port),
            db_name: configuration.database.database_name,
            db_pool,
            api_client,
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

    pub fn endpoint(&self, s: &str) -> String {
        format!("{}{}", &self.address, s)
    }

    pub async fn post_create_user(&self) -> Uuid {
        let user_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO users (user_id, created_at)
            VALUES ($1, now())
            "#,
            user_id
        )
        .execute(&self.db_pool)
        .await
        .expect("failed to create new user");

        user_id
    }

    pub async fn post_create_poll(&self, prompt: &str, username: &str) -> Uuid {
        let user_id = self.post_create_user().await;
        let poll_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO polls (poll_id, creator_id, prompt, created_at)
            VALUES ($1, $2, $3, now())
            "#,
            poll_id,
            user_id,
            prompt
        )
        .execute(&self.db_pool)
        .await
        .expect("failed to create poll");

        sqlx::query!(
            r#"
            INSERT INTO poll_users (poll_id, user_id, username)
            VALUES ($1, $2, $3)
            "#,
            poll_id,
            user_id,
            username
        )
        .execute(&self.db_pool)
        .await
        .expect("failed to insert poll_users junction table entry");

        poll_id
    }

    pub async fn get_poll_page(&self, poll_id: &str) -> reqwest::Response {
        self.api_client
            .get(self.endpoint(&format!("/poll/{poll_id}")))
            .send()
            .await
            .expect("failed to send get request")
    }

    pub async fn join_poll<Body: serde::Serialize>(
        &self,
        poll_id: &Uuid,
        body: &Body,
    ) -> reqwest::Response {
        self.api_client
            .post(&self.endpoint(&format!("/poll/{poll_id}/join")))
            .form(body)
            .send()
            .await
            .expect("could not send join request")
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
                info!("Dropped database: {db_name}");
                let _ = tx.send(());
            })
        });

        let _ = rx.recv();
        info!("ran test teardown");
    }
}
