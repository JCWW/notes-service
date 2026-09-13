use std::sync::LazyLock;

static TRACING: LazyLock<()> = LazyLock::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        notes_service::telemetry::init("notes_service=debug,tower_http=debug");
    }
});

impl TestApp {
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