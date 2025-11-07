use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

/// Helper function to create the test app
async fn create_app() -> Router {
    let store = api::TodoStore::new();

    // Initialize test database pool
    // Note: Tests require a running PostgreSQL instance with TEST_DATABASE_URL set
    let pool = api::init_db_pool()
        .await
        .expect("Failed to initialize test database pool");

    api::create_router(store, pool)
}

/// Helper function to parse JSON response body
async fn parse_json_body(body: Body) -> Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_health_check() {
    let app = create_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_body(response.into_body()).await;
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_create_todo() {
    let app = create_app().await;

    let payload = json!({
        "title": "Test Todo",
        "description": "This is a test todo"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_body(response.into_body()).await;
    assert_eq!(body["title"], "Test Todo");
    assert_eq!(body["description"], "This is a test todo");
    assert_eq!(body["completed"], false);
    assert!(body["id"].is_number());
}

#[tokio::test]
async fn test_create_todo_validation_empty_title() {
    let app = create_app().await;

    let payload = json!({
        "title": "   ",
        "description": "This should fail"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_todo_validation_title_too_long() {
    let app = create_app().await;

    let long_title = "a".repeat(201);
    let payload = json!({
        "title": long_title,
        "description": "This should fail"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_all_todos_empty() {
    let app = create_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/todos")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_body(response.into_body()).await;
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_all_todos_with_items() {
    let app = create_app().await;

    // Create first todo
    let payload1 = json!({
        "title": "First Todo",
        "description": "First description"
    });

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(payload1.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Create second todo
    let payload2 = json!({
        "title": "Second Todo",
        "description": "Second description"
    });

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(payload2.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Get all todos
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/todos")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_body(response.into_body()).await;
    assert!(body.is_array());
    let todos = body.as_array().unwrap();
    assert_eq!(todos.len(), 2);
}

#[tokio::test]
async fn test_get_todo_by_id() {
    let app = create_app().await;

    // Create a todo
    let payload = json!({
        "title": "Test Todo",
        "description": "Test description"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let created_todo = parse_json_body(create_response.into_body()).await;
    let todo_id = created_todo["id"].as_u64().unwrap();

    // Get the todo by ID
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/todos/{todo_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_body(response.into_body()).await;
    assert_eq!(body["id"], todo_id);
    assert_eq!(body["title"], "Test Todo");
    assert_eq!(body["description"], "Test description");
}

#[tokio::test]
async fn test_get_todo_not_found() {
    let app = create_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/todos/999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_todo() {
    let app = create_app().await;

    // Create a todo
    let payload = json!({
        "title": "Original Title",
        "description": "Original description"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let created_todo = parse_json_body(create_response.into_body()).await;
    let todo_id = created_todo["id"].as_u64().unwrap();

    // Update the todo
    let update_payload = json!({
        "title": "Updated Title",
        "completed": true
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/todos/{todo_id}"))
                .header("content-type", "application/json")
                .body(Body::from(update_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_body(response.into_body()).await;
    assert_eq!(body["id"], todo_id);
    assert_eq!(body["title"], "Updated Title");
    assert_eq!(body["description"], "Original description"); // Description should remain unchanged
    assert_eq!(body["completed"], true);
}

#[tokio::test]
async fn test_update_todo_not_found() {
    let app = create_app().await;

    let update_payload = json!({
        "title": "Updated Title"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/todos/999")
                .header("content-type", "application/json")
                .body(Body::from(update_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_todo_validation() {
    let app = create_app().await;

    // Create a todo
    let payload = json!({
        "title": "Original Title"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let created_todo = parse_json_body(create_response.into_body()).await;
    let todo_id = created_todo["id"].as_u64().unwrap();

    // Try to update with invalid title (empty)
    let update_payload = json!({
        "title": "   "
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/todos/{todo_id}"))
                .header("content-type", "application/json")
                .body(Body::from(update_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_todo() {
    let app = create_app().await;

    // Create a todo
    let payload = json!({
        "title": "Todo to Delete"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let created_todo = parse_json_body(create_response.into_body()).await;
    let todo_id = created_todo["id"].as_u64().unwrap();

    // Delete the todo
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/todos/{todo_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = parse_json_body(response.into_body()).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("deleted successfully")
    );

    // Verify the todo is deleted
    let get_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/todos/{todo_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_todo_not_found() {
    let app = create_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/todos/999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_full_crud_workflow() {
    let app = create_app().await;

    // 1. Create a todo
    let create_payload = json!({
        "title": "Workflow Test",
        "description": "Testing full CRUD workflow"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header("content-type", "application/json")
                .body(Body::from(create_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);
    let created_todo = parse_json_body(create_response.into_body()).await;
    let todo_id = created_todo["id"].as_u64().unwrap();

    // 2. Read the todo
    let read_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/todos/{todo_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(read_response.status(), StatusCode::OK);

    // 3. Update the todo
    let update_payload = json!({
        "completed": true
    });

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/todos/{todo_id}"))
                .header("content-type", "application/json")
                .body(Body::from(update_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_response.status(), StatusCode::OK);
    let updated_todo = parse_json_body(update_response.into_body()).await;
    assert_eq!(updated_todo["completed"], true);

    // 4. Delete the todo
    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/todos/{todo_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::OK);

    // 5. Verify deletion
    let verify_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/todos/{todo_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(verify_response.status(), StatusCode::NOT_FOUND);
}
