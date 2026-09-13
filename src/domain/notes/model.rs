use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

/// Database row. Field order must match the SELECT column order used by
/// `query_as!`, which maps positionally.
#[derive(Debug, Clone)]
pub struct Note {
    pub id: Uuid,
    pub author_id: Uuid,
    pub team_id: Option<Uuid>,
    pub title: String,
    pub body: String,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct NoteView {
    pub id: Uuid,
    pub author_id: Uuid,
    pub team_id: Option<Uuid>,
    pub title: String,
    pub body: String,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Note> for NoteView {
    fn from(n: Note) -> Self {
        Self {
            id: n.id,
            author_id: n.author_id,
            team_id: n.team_id,
            title: n.title,
            body: n.body,
            version: n.version,
            created_at: n.created_at,
            updated_at: n.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateNote {
    pub title: String,
    #[serde(default)]
    pub body: String,
}

/// PATCH semantics: an absent field means "leave it alone".
#[derive(Debug, Deserialize)]
pub struct UpdateNote {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
}

pub const MAX_TITLE: usize = 200;

/// Validated in Rust so the caller gets 422 with a useful message. The
/// CHECK constraint in the schema stays as a backstop, not as the interface.
pub fn validate_title(title: &str) -> Result<(), AppError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("title must not be empty".into()));
    }
    if trimmed.chars().count() > MAX_TITLE {
        return Err(AppError::Validation(format!(
            "title must be at most {MAX_TITLE} characters"
        )));
    }
    Ok(())
}