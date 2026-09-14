use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use super::model::{slugify, validate_team, AddMember, CreateTeam, MemberView, Team, TeamRole};
use super::repo;
use crate::{auth::CurrentUser, error::is_unique_violation, error::AppError, AppState};

/// Membership is required to see anything about a team. Non-members get 404
/// rather than 403 — a team's existence is not public.
async fn require_role(
    state: &AppState,
    team_id: Uuid,
    user_id: Uuid,
    needs_admin: bool,
) -> Result<TeamRole, AppError> {
    let role = repo::role_of(&state.db, team_id, user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if needs_admin && role != TeamRole::Admin {
        // Here 403 is right: the caller already knows the team exists.
        return Err(AppError::Forbidden);
    }

    Ok(role)
}

pub async fn create(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(payload): Json<CreateTeam>,
) -> Result<(StatusCode, Json<Team>), AppError> {
    let name = payload.name.trim().to_owned();
    let slug = payload
        .slug
        .map(|s| slugify(&s))
        .unwrap_or_else(|| slugify(&name));
    validate_team(&name, &slug)?;

    let team = repo::create(&state.db, &name, &slug, user_id)
        .await
        .map_err(|err| {
            if is_unique_violation(&err) {
                AppError::Conflict(format!("a team with slug '{slug}' already exists"))
            } else {
                AppError::Db(err)
            }
        })?;

    Ok((StatusCode::CREATED, Json(team)))
}

pub async fn list(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<Team>>, AppError> {
    Ok(Json(repo::list_for_user(&state.db, user_id).await?))
}

pub async fn list_members(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(team_id): Path<Uuid>,
) -> Result<Json<Vec<MemberView>>, AppError> {
    require_role(&state, team_id, user_id, false).await?;

    Ok(Json(repo::list_members(&state.db, team_id).await?))
}

pub async fn add_member(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(team_id): Path<Uuid>,
    Json(payload): Json<AddMember>,
) -> Result<StatusCode, AppError> {
    require_role(&state, team_id, user_id, true).await?;

    repo::add_member(&state.db, team_id, payload.user_id, payload.role)
        .await
        .map_err(|err| {
            // Foreign key to users. The composite PK case is absorbed by the
            // ON CONFLICT clause above.
            if matches!(&err, sqlx::Error::Database(db) if db.code().as_deref() == Some("23503")) {
                AppError::Validation("no such user".into())
            } else {
                AppError::Db(err)
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_member(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path((team_id, member_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    require_role(&state, team_id, user_id, true).await?;

    if !repo::remove_member(&state.db, team_id, member_id).await? {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}