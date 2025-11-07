use crate::error::{AppError, Result};
use crate::models::{CreateUser, UpdateUser, User};
use crate::repository::UserRepository;
use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request payload for creating a new user
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub picture: Option<String>,
}

/// Request payload for updating an existing user
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub picture: Option<String>,
}

/// Response payload for user data
/// Note: Excludes sensitive fields like `password_hash`
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub picture: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            picture: user.picture,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

impl CreateUserRequest {
    /// Validate the create user request
    ///
    /// # Errors
    /// Returns validation error if:
    /// - Name is empty or only whitespace
    /// - Name exceeds 100 characters
    /// - Email is empty or only whitespace
    /// - Email is not a valid email format
    /// - Email exceeds 255 characters
    fn validate(&self) -> Result<()> {
        // Validate name
        if self.name.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Name cannot be empty".to_string(),
            ));
        }
        if self.name.len() > 100 {
            return Err(AppError::ValidationError(
                "Name must be 100 characters or less".to_string(),
            ));
        }

        // Validate email
        if self.email.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Email cannot be empty".to_string(),
            ));
        }
        if self.email.len() > 255 {
            return Err(AppError::ValidationError(
                "Email must be 255 characters or less".to_string(),
            ));
        }
        // Basic email format validation
        if !self.email.contains('@') || !self.email.contains('.') {
            return Err(AppError::ValidationError(
                "Email must be a valid email address".to_string(),
            ));
        }

        Ok(())
    }
}

impl UpdateUserRequest {
    /// Validate the update user request
    ///
    /// # Errors
    /// Returns validation error if:
    /// - Name is empty or only whitespace
    /// - Name exceeds 100 characters
    /// - Email is empty or only whitespace
    /// - Email is not a valid email format
    /// - Email exceeds 255 characters
    fn validate(&self) -> Result<()> {
        // Validate name if provided
        if let Some(name) = &self.name {
            if name.trim().is_empty() {
                return Err(AppError::ValidationError(
                    "Name cannot be empty".to_string(),
                ));
            }
            if name.len() > 100 {
                return Err(AppError::ValidationError(
                    "Name must be 100 characters or less".to_string(),
                ));
            }
        }

        // Validate email if provided
        if let Some(email) = &self.email {
            if email.trim().is_empty() {
                return Err(AppError::ValidationError(
                    "Email cannot be empty".to_string(),
                ));
            }
            if email.len() > 255 {
                return Err(AppError::ValidationError(
                    "Email must be 255 characters or less".to_string(),
                ));
            }
            // Basic email format validation
            if !email.contains('@') || !email.contains('.') {
                return Err(AppError::ValidationError(
                    "Email must be a valid email address".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// GET /api/users - Get all users
///
/// # Errors
/// Returns an error if the database query fails
pub async fn get_users(State(_repo): State<UserRepository>) -> Result<Json<Vec<UserResponse>>> {
    tracing::debug!("Fetching all users");

    // Note: We need to add a list_all method to UserRepository
    // For now, we'll return an error indicating this needs to be implemented
    Err(AppError::InternalServerError(
        "List all users not yet implemented".to_string(),
    ))
}

/// GET /api/users/:id - Get a specific user by ID
///
/// # Errors
/// Returns `NotFound` error if the user with the specified ID does not exist
pub async fn get_user(
    State(repo): State<UserRepository>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>> {
    tracing::debug!("Fetching user with id: {id}");

    let user = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User with id {id} not found")))?;

    Ok(Json(user.into()))
}

/// POST /api/users - Create a new user
///
/// # Errors
/// Returns `ValidationError` if the payload validation fails
/// Returns error if database operation fails
pub async fn create_user(
    State(repo): State<UserRepository>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>> {
    tracing::debug!(name = %payload.name, email = %payload.email, "Creating new user");

    // Validation
    payload.validate()?;

    // Create user in database
    let create_user = CreateUser {
        name: payload.name,
        email: payload.email,
        picture: payload.picture,
    };

    let user = repo.create(create_user).await?;

    Ok(Json(user.into()))
}

/// PUT /api/users/:id - Update an existing user
///
/// # Errors
/// Returns `ValidationError` if the payload validation fails
/// Returns `NotFound` if the user with the specified ID does not exist
/// Returns error if database operation fails
pub async fn update_user(
    State(repo): State<UserRepository>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    tracing::debug!(user_id = %id, "Updating user");

    // Validation
    payload.validate()?;

    // Update user in database
    let update_user = UpdateUser {
        name: payload.name,
        email: payload.email,
        picture: payload.picture,
    };

    let user = repo.update(id, update_user).await?;

    Ok(Json(user.into()))
}

/// DELETE /api/users/:id - Delete a user by ID (soft delete)
///
/// # Errors
/// Returns `NotFound` error if the user with the specified ID does not exist
/// Returns error if database operation fails
pub async fn delete_user(
    State(repo): State<UserRepository>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    tracing::debug!(user_id = %id, "Deleting user");

    repo.delete(id).await?;

    Ok(Json(serde_json::json!({
        "message": format!("User with id {id} deleted successfully")
    })))
}
