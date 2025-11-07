use sqlx::{PgPool, Postgres, Transaction};

/// Test context for managing database transactions in tests
///
/// This struct provides transaction-based test isolation:
/// - Each test gets a clean database state
/// - All changes are rolled back after the test
/// - Tests can explicitly create only the data they need
pub struct TestContext {
    pool: PgPool,
    tx: Option<Transaction<'static, Postgres>>,
}

impl TestContext {
    /// Create a new test context
    ///
    /// This initializes the database connection pool and runs migrations.
    /// Uses the `TEST_DATABASE_URL` environment variable for the connection string.
    ///
    /// # Panics
    /// Panics if:
    /// - `TEST_DATABASE_URL` environment variable is not set
    /// - Database connection fails
    /// - Migration execution fails
    ///
    /// # Example
    /// ```no_run
    /// use helpers::TestContext;
    ///
    /// #[tokio::test]
    /// async fn test_example() {
    ///     let mut ctx = TestContext::new().await;
    ///     // Use ctx for testing
    /// }
    /// ```
    pub async fn new() -> Self {
        let database_url =
            std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set for tests");

        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        // Run migrations
        sqlx::migrate!("./db/migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        Self { pool, tx: None }
    }

    /// Begin a new transaction
    ///
    /// This starts a transaction that will be automatically rolled back
    /// when the `TestContext` is dropped or when `rollback()` is called explicitly.
    ///
    /// # Returns
    /// A mutable reference to the transaction that can be used for database operations
    ///
    /// # Panics
    /// Panics if:
    /// - A transaction has already been started
    /// - Failed to begin transaction
    ///
    /// # Example
    /// ```no_run
    /// use helpers::TestContext;
    ///
    /// #[tokio::test]
    /// async fn test_with_transaction() {
    ///     let mut ctx = TestContext::new().await;
    ///     let tx = ctx.begin_transaction().await;
    ///
    ///     // Insert test data
    ///     sqlx::query(
    ///         "INSERT INTO users (id, name, email, created_at, updated_at)
    ///          VALUES ('00000000-0000-0000-0000-000000000001', 'Test User', 'test@example.com', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    ///     )
    ///     .execute(&mut **tx)
    ///     .await
    ///     .unwrap();
    ///
    ///     // Transaction will be rolled back when ctx is dropped
    /// }
    /// ```
    pub async fn begin_transaction(&mut self) -> &mut Transaction<'static, Postgres> {
        assert!(self.tx.is_none(), "Transaction already started");

        let tx = self
            .pool
            .begin()
            .await
            .expect("Failed to begin transaction");

        self.tx = Some(tx);
        self.tx.as_mut().unwrap()
    }

    /// Explicitly rollback the transaction
    ///
    /// Note: The transaction will be automatically rolled back when `TestContext`
    /// is dropped, so calling this method is optional.
    ///
    /// # Panics
    /// Panics if no transaction has been started
    pub async fn rollback(mut self) {
        if let Some(tx) = self.tx.take() {
            tx.rollback().await.expect("Failed to rollback transaction");
        } else {
            panic!("No transaction to rollback");
        }
    }

    /// Get a reference to the database pool
    ///
    /// This can be used for operations that don't need transaction isolation.
    #[must_use]
    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }
}

// Example usage in tests:
//
// #[tokio::test]
// async fn test_create_user() {
//     let mut ctx = TestContext::new().await;
//     let tx = ctx.begin_transaction().await;
//
//     // Insert test data using complete SQL (no bind parameters for clarity)
//     sqlx::query(
//         "INSERT INTO users (id, name, email, created_at, updated_at)
//          VALUES
//            ('00000000-0000-0000-0000-000000000001', 'Test User 1', 'test1@example.com', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
//            ('00000000-0000-0000-0000-000000000002', 'Test User 2', 'test2@example.com', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
//     )
//     .execute(&mut **tx)
//     .await
//     .unwrap();
//
//     // Verify data was inserted
//     let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
//         .fetch_one(&mut **tx)
//         .await
//         .unwrap();
//
//     assert_eq!(count.0, 2);
//
//     // Transaction will be rolled back when ctx is dropped
// }
//
// Note on Router integration:
// The current implementation uses repositories with stored PgPool,
// which makes it challenging to use transactions with the router pattern.
// For now, tests should use direct SQL queries with the transaction.
// Future improvements could include:
// 1. Modifying repositories to accept generic Executor trait
// 2. Creating a test-specific router factory that uses transactions
// 3. Using repository methods directly with transaction pool
