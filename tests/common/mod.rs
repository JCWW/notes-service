use std::sync::LazyLock;

use notes_service::{AppState, build_router};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Connection, PgConnection, PgPool};
use uuid::Uuid;

static TRACING: LazyLock<()> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    if std::env::var("TEST_LOG").is_ok() {
        notes_service::telemetry::init("notes_service=debug,tower_http=debug");
    }
});

pub struct TestApp {
    /// e.g. "http://127.0.0.1:54312"
    pub address: String,
    /// Direct pool, for seeding fixtures and asserting on stored state.
    pub db: PgPool,
    pub client: reqwest::Client,
    db_name: String,
    maintenance: PgConnectOptions,
}

pub struct AuthedClient<'a> {
    app: &'a TestApp,
    user_id: Uuid,
}

impl AuthedClient<'_> {
    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.app
            .client
            .get(self.app.url(path))
            .bearer_auth(self.user_id)
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.app
            .client
            .post(self.app.url(path))
            .bearer_auth(self.user_id)
    }

    pub fn patch(&self, path: &str) -> reqwest::RequestBuilder {
        self.app
            .client
            .patch(self.app.url(path))
            .bearer_auth(self.user_id)
    }

    pub fn delete(&self, path: &str) -> reqwest::RequestBuilder {
        self.app
            .client
            .delete(self.app.url(path))
            .bearer_auth(self.user_id)
    }
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.address, path)
    }

    pub fn as_user(&self, user_id: Uuid) -> AuthedClient<'_> {
        AuthedClient { app: self, user_id }
    }

    pub async fn create_note(&self, author: Uuid, title: &str) -> Uuid {
        let body: serde_json::Value = self
            .as_user(author)
            .post("/notes")
            .json(&serde_json::json!({ "title": title, "body": "" }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        body["id"].as_str().unwrap().parse().unwrap()
    }

    pub async fn seed_team_via_api(&self, admin: Uuid, name: &str) -> Uuid {
        let body: serde_json::Value = self
            .as_user(admin)
            .post("/teams")
            .json(&serde_json::json!({ "name": name }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        body["id"].as_str().unwrap().parse().unwrap()
    }

    pub async fn add_member(&self, admin: Uuid, team: Uuid, member: Uuid) {
        let res = self
            .as_user(admin)
            .post(&format!("/teams/{team}/members"))
            .json(&serde_json::json!({ "user_id": member }))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 204);
    }

    pub async fn seed_user(&self, name: &str) -> Uuid {
        sqlx::query_scalar!(
            "INSERT INTO users (email, display_name) VALUES ($1, $2) RETURNING id",
            format!("{name}@example.test"),
            name
        )
        .fetch_one(&self.db)
        .await
        .expect("seed user")
    }
}

/// Spins up the app on a random port against a freshly created, migrated
/// database, so tests can run concurrently without stepping on each other.
pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);
    dotenvy::dotenv().ok();

    let base: PgConnectOptions = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set")
        .parse()
        .expect("DATABASE_URL is not a valid Postgres URL");

    // Uniquely named so parallel tests never collide.
    let db_name = format!("notes_test_{}", Uuid::new_v4().simple());
    let maintenance = base.clone().database("postgres");

    let mut conn = PgConnection::connect_with(&maintenance)
        .await
        .expect("connect to maintenance database");
    
    sqlx::query(sqlx::AssertSqlSafe(format!(
        r#"CREATE DATABASE "{db_name}""#
    )))
    .execute(&mut conn)
    .await
    .expect("create test database");

    // Keep this small. Postgres allows ~100 connections by default and
    // cargo runs tests in parallel, so a large pool per test exhausts it.
    let db = PgPoolOptions::new()
        .max_connections(3)
        .connect_with(base.database(&db_name))
        .await
        .expect("connect to test database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("run migrations");

    // Port 0 lets the OS pick a free port.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    let port = listener.local_addr().unwrap().port();

    let app = build_router(AppState { db: db.clone() });
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server crashed");
    });

    TestApp {
        address: format!("http://127.0.0.1:{port}"),
        db,
        client: reqwest::Client::new(),
        db_name,
        maintenance,
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        let db_name = self.db_name.clone();
        let maintenance = self.maintenance.clone();

        // Close our own connections first, or DROP DATABASE blocks.
        let pool = self.db.clone();

        let _ = std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async move {
                    pool.close().await;
                    if let Ok(mut conn) =
                        PgConnection::connect_with(&maintenance).await
                    {
                        // FORCE terminates any stragglers. Postgres 13+.
                        let _ = conn
                            .execute(
                                format!(
                                    r#"DROP DATABASE IF EXISTS "{db_name}" WITH (FORCE)"#
                                )
                                .as_str(),
                            )
                            .await;
                    }
                });
        })
        .join();
    }
}
