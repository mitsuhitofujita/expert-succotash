use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Database Connection Checker ===\n");

    // Get DATABASE_URL from environment
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL environment variable is not set")?;

    println!("Connecting to database...");
    println!("URL: {}\n", mask_password(&database_url));

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    println!("✓ Connection successful!\n");

    // Execute a simple query to verify the connection
    println!("Executing test query...");
    let result: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await
        .context("Failed to execute test query")?;

    println!("✓ Test query successful! Result: {}\n", result.0);

    // Get database version
    println!("Fetching database version...");
    let version: (String,) = sqlx::query_as("SELECT version()")
        .fetch_one(&pool)
        .await
        .context("Failed to fetch database version")?;

    println!("✓ Database version:");
    println!("{}\n", version.0);

    // Close the connection
    pool.close().await;
    println!("✓ Connection closed successfully!");

    Ok(())
}

/// Mask password in database URL for security
fn mask_password(url: &str) -> String {
    if let Some(at_pos) = url.rfind('@')
        && let Some(colon_pos) = url[..at_pos].rfind(':')
        && let Some(protocol_end) = url.find("://")
    {
        let protocol_part = &url[..protocol_end + 3];
        let user_part = &url[(protocol_end + 3)..=colon_pos];
        let host_part = &url[at_pos..];
        return format!("{protocol_part}{user_part}****{host_part}");
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_password() {
        let url = "postgresql://user:password@localhost:5432/database";
        let masked = mask_password(url);
        assert_eq!(masked, "postgresql://user:****@localhost:5432/database");
    }

    #[test]
    fn test_mask_password_no_password() {
        let url = "postgresql://localhost:5432/database";
        let masked = mask_password(url);
        assert_eq!(masked, "postgresql://localhost:5432/database");
    }
}
