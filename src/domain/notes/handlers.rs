use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use super::model::{validate_title, CreateNote, Note, NoteView, UpdateNote};
use super::repo;
use crate::{auth::CurrentUser, error::AppError, AppState};

const DEFAULT_LIMIT: i64 = 50;

/// Personal notes only, for now. When teams arrive this becomes a check
/// against team membership as well.
fn authorize(note: &Note, caller: Uuid) -> Result<(), AppError> {
    if note.author_id == caller {
        Ok(())
    } else {
        // 404, not 403: a stranger should not learn that this id exists.
        Err(AppError::NotFound)
    }
}

pub async fn create(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Json(payload): Json<CreateNote>,
) -> Result<(StatusCode, Json<NoteView>), AppError> {
    validate_title(&payload.title)?;

    let note = repo::create(&state.db, user_id, payload.title.trim(), &payload.body).await?;

    Ok((StatusCode::CREATED, Json(note.into())))
}

pub async fn list(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<NoteView>>, AppError> {
    let notes = repo::list_for_author(&state.db, user_id, DEFAULT_LIMIT).await?;

    Ok(Json(notes.into_iter().map(NoteView::from).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<NoteView>, AppError> {
    let note = repo::find(&state.db, id).await?.ok_or(AppError::NotFound)?;
    authorize(&note, user_id)?;

    Ok(Json(note.into()))
}

pub async fn update(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateNote>,
) -> Result<Json<NoteView>, AppError> {
    if let Some(title) = &payload.title {
        validate_title(title)?;
    }

    // Read then write. This is a race until If-Match lands two commits from
    // now, at which point the UPDATE carries its own version predicate.
    let existing = repo::find(&state.db, id).await?.ok_or(AppError::NotFound)?;
    authorize(&existing, user_id)?;

    let updated = repo::update(
        &state.db,
        id,
        payload.title.as_deref().map(str::trim),
        payload.body.as_deref(),
    )
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(updated.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let note = repo::find(&state.db, id).await?.ok_or(AppError::NotFound)?;
    authorize(&note, user_id)?;

    repo::soft_delete(&state.db, id).await?;

    Ok(StatusCode::NO_CONTENT)
}