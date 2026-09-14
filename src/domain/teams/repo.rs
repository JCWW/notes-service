use sqlx::PgPool;
use uuid::Uuid;

use super::model::{MemberView, Team, TeamRole};

/// Creating a team and making the creator its admin must both happen or
/// neither: a team with no admin can never be administered.
pub async fn create(
    db: &PgPool,
    name: &str,
    slug: &str,
    creator: Uuid,
) -> Result<Team, sqlx::Error> {
    let mut tx = db.begin().await?;

    let team = sqlx::query_as!(
        Team,
        "INSERT INTO teams (name, slug) VALUES ($1, $2) RETURNING id, name, slug, created_at",
        name,
        slug
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO team_members (team_id, user_id, role) VALUES ($1, $2, 'admin')",
        team.id,
        creator
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(team)
}

pub async fn list_for_user(db: &PgPool, user_id: Uuid) -> Result<Vec<Team>, sqlx::Error> {
    sqlx::query_as!(
        Team,
        r#"
        SELECT t.id AS "id!", t.name AS "name!", t.slug AS "slug!",
               t.created_at AS "created_at!"
        FROM teams t
        JOIN team_members m ON m.team_id = t.id
        WHERE m.user_id = $1
        ORDER BY t.name
        "#,
        user_id
    )
    .fetch_all(db)
    .await
}

pub async fn role_of(
    db: &PgPool,
    team_id: Uuid,
    user_id: Uuid,
) -> Result<Option<TeamRole>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT role AS "role: TeamRole" FROM team_members WHERE team_id = $1 AND user_id = $2"#,
        team_id,
        user_id
    )
    .fetch_optional(db)
    .await
}

pub async fn list_members(db: &PgPool, team_id: Uuid) -> Result<Vec<MemberView>, sqlx::Error> {
    sqlx::query_as!(
        MemberView,
        r#"
        SELECT m.user_id AS "user_id!",
               u.display_name AS "display_name!",
               m.role AS "role!: TeamRole",
               m.joined_at AS "joined_at!"
        FROM team_members m
        JOIN users u ON u.id = m.user_id
        WHERE m.team_id = $1
        ORDER BY u.display_name
        "#,
        team_id
    )
    .fetch_all(db)
    .await
}

pub async fn add_member(
    db: &PgPool,
    team_id: Uuid,
    user_id: Uuid,
    role: TeamRole,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO team_members (team_id, user_id, role)
        VALUES ($1, $2, $3)
        ON CONFLICT (team_id, user_id) DO UPDATE SET role = EXCLUDED.role
        "#,
        team_id,
        user_id,
        role as TeamRole
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn remove_member(
    db: &PgPool,
    team_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let rows = sqlx::query!(
        "DELETE FROM team_members WHERE team_id = $1 AND user_id = $2",
        team_id,
        user_id
    )
    .execute(db)
    .await?
    .rows_affected();

    Ok(rows > 0)
}