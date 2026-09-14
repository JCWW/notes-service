use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "team_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TeamRole {
    Member,
    Admin,
}

#[derive(Debug, Serialize)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MemberView {
    pub user_id: Uuid,
    pub display_name: String,
    pub role: TeamRole,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTeam {
    pub name: String,
    /// Derived from the name when omitted.
    #[serde(default)]
    pub slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMember {
    pub user_id: Uuid,
    #[serde(default = "default_role")]
    pub role: TeamRole,
}

fn default_role() -> TeamRole {
    TeamRole::Member
}

/// Lowercase, alphanumerics kept, everything else collapsed to a hyphen.
pub fn slugify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_dash = true; // suppresses a leading hyphen

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }

    out.trim_end_matches('-').to_owned()
}

pub fn validate_team(name: &str, slug: &str) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("name must not be empty".into()));
    }
    if slug.is_empty() {
        return Err(AppError::Validation(
            "slug must contain at least one alphanumeric character".into(),
        ));
    }
    Ok(())
}