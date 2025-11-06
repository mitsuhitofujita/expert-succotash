use crate::error::Result;
use crate::models::{CreateUser, UpdateUser, User};
use sqlx::PgPool;
use uuid::Uuid;

/// User repository for database operations
/// Handles CRUD operations for the users table with soft delete support
#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    /// Create a new `UserRepository` instance
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find a user by ID (only active users, `deleted_at` IS NULL)
    ///
    /// # Arguments
    /// * `id` - The UUID of the user
    ///
    /// # Returns
    /// * `Ok(Some(User))` - User found
    /// * `Ok(None)` - User not found or deleted
    ///
    /// # Errors
    /// Returns `AppError` if database query fails
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, name, email, picture, created_at, updated_at
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find a user by email address (only active users, `deleted_at` IS NULL)
    ///
    /// # Arguments
    /// * `email` - The email address to search for
    ///
    /// # Returns
    /// * `Ok(Some(User))` - User found
    /// * `Ok(None)` - User not found or deleted
    ///
    /// # Errors
    /// Returns `AppError` if database query fails
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, name, email, picture, created_at, updated_at
            FROM users
            WHERE email = $1 AND deleted_at IS NULL
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Create a new user
    ///
    /// # Arguments
    /// * `user` - The user creation request data
    ///
    /// # Returns
    /// * `Ok(User)` - The created user with generated ID and timestamps
    ///
    /// # Errors
    /// Returns `AppError` if database query fails (e.g., unique constraint violation)
    pub async fn create(&self, user: CreateUser) -> Result<User> {
        let created_user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (name, email, picture)
            VALUES ($1, $2, $3)
            RETURNING id, name, email, picture, created_at, updated_at
            "#,
            user.name,
            user.email,
            user.picture
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(created_user)
    }

    /// Update an existing user
    /// Only updates fields that are provided (Some) in the `UpdateUser` struct
    /// Automatically updates the `updated_at` timestamp
    ///
    /// # Arguments
    /// * `id` - The UUID of the user to update
    /// * `user` - The user update request data with optional fields
    ///
    /// # Returns
    /// * `Ok(User)` - The updated user
    ///
    /// # Errors
    /// Returns `AppError` if database query fails or user not found
    pub async fn update(&self, id: Uuid, user: UpdateUser) -> Result<User> {
        let updated_user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET
                name = COALESCE($2, name),
                email = COALESCE($3, email),
                picture = COALESCE($4, picture),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, name, email, picture, created_at, updated_at
            "#,
            id,
            user.name,
            user.email,
            user.picture
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_user)
    }

    /// Delete a user (soft delete by setting `deleted_at` timestamp)
    /// The user will no longer appear in queries but the record is preserved
    ///
    /// # Arguments
    /// * `id` - The UUID of the user to delete
    ///
    /// # Returns
    /// * `Ok(())` - User successfully deleted
    ///
    /// # Errors
    /// Returns `AppError` if database query fails or user not found
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET deleted_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(crate::error::AppError::NotFound(format!(
                "User with id {id} not found"
            )));
        }

        Ok(())
    }
}
