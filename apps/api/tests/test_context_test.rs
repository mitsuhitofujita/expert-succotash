mod helpers;

use helpers::TestContext;

/// Test that `TestContext` can be initialized successfully
#[tokio::test]
async fn test_context_initialization() {
    let ctx = TestContext::new().await;

    // Verify that the pool is connected
    let result: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(ctx.pool())
        .await
        .expect("Failed to execute query");

    assert_eq!(result.0, 1);
}

/// Test that transactions can be started and rolled back
#[tokio::test]
async fn test_transaction_rollback() {
    let mut ctx = TestContext::new().await;
    let tx = ctx.begin_transaction().await;

    // Insert a test user
    sqlx::query(
        "INSERT INTO users (id, name, email, created_at, updated_at)
         VALUES ('00000000-0000-0000-0000-000000000001', 'Test User', 'test@example.com', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .execute(&mut **tx)
    .await
    .expect("Failed to insert user");

    // Verify the user exists within the transaction
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE id = '00000000-0000-0000-0000-000000000001'",
    )
    .fetch_one(&mut **tx)
    .await
    .expect("Failed to count users");

    assert_eq!(count.0, 1, "User should exist within transaction");

    // Drop the transaction (implicit rollback)
    drop(ctx);

    // Create a new context to verify rollback
    let ctx2 = TestContext::new().await;
    let count2: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE id = '00000000-0000-0000-0000-000000000001'",
    )
    .fetch_one(ctx2.pool())
    .await
    .expect("Failed to count users after rollback");

    assert_eq!(
        count2.0, 0,
        "User should not exist after transaction rollback"
    );
}

/// Test that multiple test records can be inserted in a transaction
#[tokio::test]
async fn test_multiple_inserts_in_transaction() {
    let mut ctx = TestContext::new().await;
    let tx = ctx.begin_transaction().await;

    // Insert multiple test users
    sqlx::query(
        "INSERT INTO users (id, name, email, created_at, updated_at)
         VALUES
           ('00000000-0000-0000-0000-000000000001', 'Test User 1', 'test1@example.com', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
           ('00000000-0000-0000-0000-000000000002', 'Test User 2', 'test2@example.com', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
           ('00000000-0000-0000-0000-000000000003', 'Test User 3', 'test3@example.com', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .execute(&mut **tx)
    .await
    .expect("Failed to insert users");

    // Verify all users were inserted
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&mut **tx)
        .await
        .expect("Failed to count users");

    assert_eq!(count.0, 3, "Should have 3 users in transaction");

    // Transaction will be rolled back automatically
}

/// Test explicit rollback
#[tokio::test]
async fn test_explicit_rollback() {
    let mut ctx = TestContext::new().await;
    let tx = ctx.begin_transaction().await;

    // Insert a test user
    sqlx::query(
        "INSERT INTO users (id, name, email, created_at, updated_at)
         VALUES ('00000000-0000-0000-0000-000000000099', 'Rollback User', 'rollback@example.com', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .execute(&mut **tx)
    .await
    .expect("Failed to insert user");

    // Explicitly rollback
    ctx.rollback().await;

    // Verify rollback
    let ctx2 = TestContext::new().await;
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE id = '00000000-0000-0000-0000-000000000099'",
    )
    .fetch_one(ctx2.pool())
    .await
    .expect("Failed to count users");

    assert_eq!(count.0, 0, "User should not exist after explicit rollback");
}
