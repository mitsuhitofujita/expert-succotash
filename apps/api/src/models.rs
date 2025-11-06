use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Todo リソースのデータモデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
}

/// User entity from database
/// Matches the schema in `20251104145951_create_users_table.sql`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub picture: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Note: deleted_at is used internally for soft delete but not exposed in public API
}

/// User creation request
#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub picture: Option<String>,
}

/// User update request
#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
    pub picture: Option<String>,
}

/// Attendance event entity from database
/// Matches the schema in `20251105142320_create_attendance_events.sql`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceEvent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_type: String,
    pub event_time: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Attendance event creation request
#[derive(Debug, Deserialize)]
pub struct CreateAttendanceEvent {
    pub user_id: Uuid,
    pub event_type: String,
    pub event_time: DateTime<Utc>,
    // Note: recorded_at and created_at are set by the server
}

/// Todo作成時のリクエストボディ
#[derive(Debug, Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
}

/// Todo更新時のリクエストボディ
#[derive(Debug, Deserialize)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
}

impl CreateTodoRequest {
    /// Validate the create todo request
    ///
    /// # Errors
    /// Returns an error string if validation fails:
    /// - Title is empty or only whitespace
    /// - Title exceeds 200 characters
    /// - Description exceeds 1000 characters
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        if self.title.len() > 200 {
            return Err("Title must be 200 characters or less".to_string());
        }
        if let Some(desc) = &self.description
            && desc.len() > 1000
        {
            return Err("Description must be 1000 characters or less".to_string());
        }
        Ok(())
    }
}

impl UpdateTodoRequest {
    /// Validate the update todo request
    ///
    /// # Errors
    /// Returns an error string if validation fails:
    /// - Title is empty or only whitespace
    /// - Title exceeds 200 characters
    /// - Description exceeds 1000 characters
    pub fn validate(&self) -> Result<(), String> {
        if let Some(title) = &self.title {
            if title.trim().is_empty() {
                return Err("Title cannot be empty".to_string());
            }
            if title.len() > 200 {
                return Err("Title must be 200 characters or less".to_string());
            }
        }
        if let Some(desc) = &self.description
            && desc.len() > 1000
        {
            return Err("Description must be 1000 characters or less".to_string());
        }
        Ok(())
    }
}
