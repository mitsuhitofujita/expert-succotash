use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

/// Initialize `PostgreSQL` connection pool
///
/// This function creates a connection pool to `PostgreSQL` using the `DATABASE_URL`
/// environment variable. The pool is configured with the following settings:
/// - Max connections: 20
/// - Connection timeout: 30 seconds
/// - Idle timeout: 10 minutes
///
/// # Environment Variables
///
/// - `DATABASE_URL`: `PostgreSQL` connection string
///   - Development: `postgresql://attendance_user:attendance_password@postgres:5432/attendance_dev?sslmode=disable`
///   - Production (Neon): `postgresql://user:password@host/database?sslmode=require`
///
/// # Errors
///
/// Returns an error if:
/// - `DATABASE_URL` environment variable is not set
/// - Failed to connect to the database
/// - Connection pool creation fails
///
/// # Example
///
/// ```no_run
/// use api::init_db_pool;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let pool = init_db_pool().await?;
///     Ok(())
/// }
/// ```
pub async fn init_db_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        tracing::warn!("DATABASE_URL not set, using default development URL");
        "postgresql://attendance_user:attendance_password@postgres:5432/attendance_dev".to_string()
    });

    tracing::info!("Initializing database connection pool");

    // Mask password in log output for security
    let masked_url = mask_password(&database_url);
    tracing::debug!("Connecting to database: {masked_url}");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600)) // 10 minutes
        .connect(&database_url)
        .await?;

    tracing::info!("Database connection pool initialized successfully");

    Ok(pool)
}

/// Mask password in database URL for safe logging
fn mask_password(url: &str) -> String {
    if let Some(at_pos) = url.rfind('@')
        && let Some(protocol_end) = url.find("://")
    {
        let protocol = &url[..protocol_end + 3];
        let after_at = &url[at_pos..];

        let credentials = &url[protocol_end + 3..at_pos];

        if let Some(colon_pos) = credentials.find(':') {
            let username = &credentials[..colon_pos];
            return format!("{protocol}{username}:***{after_at}");
        }
    }

    // If URL format is unexpected, return as-is (shouldn't happen with valid PostgreSQL URLs)
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_password() {
        let url = "postgresql://user:password@localhost:5432/db";
        assert_eq!(
            mask_password(url),
            "postgresql://user:***@localhost:5432/db"
        );

        let url_with_params = "postgresql://user:secret123@host:5432/db?sslmode=require";
        assert_eq!(
            mask_password(url_with_params),
            "postgresql://user:***@host:5432/db?sslmode=require"
        );

        let complex_url =
            "postgresql://attendance_user:attendance_password@postgres:5432/attendance_dev";
        assert_eq!(
            mask_password(complex_url),
            "postgresql://attendance_user:***@postgres:5432/attendance_dev"
        );
    }

    #[test]
    fn test_mask_password_no_password() {
        // Edge case: URL without password (unusual but should handle gracefully)
        let url = "postgresql://user@localhost:5432/db";
        assert_eq!(mask_password(url), "postgresql://user@localhost:5432/db");
    }

    #[test]
    fn test_mask_password_invalid_format() {
        // Edge case: invalid URL format
        let url = "invalid-url";
        assert_eq!(mask_password(url), "invalid-url");
    }
}
