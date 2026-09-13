use sqlx::PgPool;
use uuid::Uuid;

use super::model::Note;

pub async fn create(
    db: &PgPool,
    author_id: Uuid,
    title: &str,
    body: &str,
) -> Result<Note, sqlx::Error> {
    sqlx::query_as!(
        Note,
        r#"
        INSERT INTO notes (author_id, title, body)
        VALUES ($1, $2, $3)
        RETURNING id, author_id, team_id, title, body, version, created_at, updated_at
        "#,
        author_id,
        title,
        body
    )
    .fetch_one(db)
    .await
}

pub async fn find(db: &PgPool, id: Uuid) -> Result<Option<Note>, sqlx::Error> {
    sqlx::query_as!(
        Note,
        r#"
        SELECT id, author_id, team_id, title, body, version, created_at, updated_at
        FROM notes
        WHERE id = $1 AND deleted_at IS NULL
        "#,
        id
    )
    .fetch_optional(db)
    .await
}

pub async fn list_for_author(
    db: &PgPool,
    author_id: Uuid,
    limit: i64,
) -> Result<Vec<Note>, sqlx::Error> {
    sqlx::query_as!(
        Note,
        r#"
        SELECT id, author_id, team_id, title, body, version, created_at, updated_at
        FROM notes
        WHERE author_id = $1 AND deleted_at IS NULL
        ORDER BY updated_at DESC, id DESC
        LIMIT $2
        "#,
        author_id,
        limit
    )
    .fetch_all(db)
    .await
}

/// COALESCE gives PATCH semantics in one statement. The `::text` casts are
/// required — Postgres cannot infer a parameter's type inside COALESCE.
pub async fn update(
    db: &PgPool,
    id: Uuid,
    title: Option<&str>,
    body: Option<&str>,
) -> Result<Option<Note>, sqlx::Error> {
    sqlx::query_as!(
        Note,
        r#"
        UPDATE notes
        SET title      = COALESCE($2::text, title),
            body       = COALESCE($3::text, body),
            version    = version + 1,
            updated_at = now()
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING id, author_id, team_id, title, body, version, created_at, updated_at
        "#,
        id,
        title,
        body
    )
    .fetch_optional(db)
    .await
}

pub async fn soft_delete(db: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let rows = sqlx::query!(
        "UPDATE notes SET deleted_at = now() WHERE id = $1 AND deleted_at IS NULL",
        id
    )
    .execute(db)
    .await?
    .rows_affected();

    Ok(rows > 0)
}