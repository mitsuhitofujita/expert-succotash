use crate::error::{AppError, Result};
use crate::models::{CreateTodoRequest, Todo, UpdateTodoRequest};
use crate::store::TodoStore;
use axum::{
    Json,
    extract::{Path, State},
};

/// GET /api/todos - Get all todos
///
/// # Errors
/// Returns an error if the operation fails
pub async fn get_todos(State(store): State<TodoStore>) -> Result<Json<Vec<Todo>>> {
    tracing::debug!("Fetching all todos");
    let todos = store.get_all();
    Ok(Json(todos))
}

/// GET /api/todos/:id - Get a specific todo by ID
///
/// # Errors
/// Returns `NotFound` error if the todo with the specified ID does not exist
pub async fn get_todo(State(store): State<TodoStore>, Path(id): Path<u64>) -> Result<Json<Todo>> {
    tracing::debug!(todo_id = id, "Fetching todo by id");

    store
        .get_by_id(id)
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {id} not found")))
}

/// POST /api/todos - Create a new todo
///
/// # Errors
/// Returns `ValidationError` if the payload validation fails
pub async fn create_todo(
    State(store): State<TodoStore>,
    Json(payload): Json<CreateTodoRequest>,
) -> Result<Json<Todo>> {
    tracing::debug!(title = %payload.title, "Creating new todo");

    // Validation
    payload.validate().map_err(AppError::ValidationError)?;

    let todo = store.create(payload.title, payload.description);
    Ok(Json(todo))
}

/// PUT /api/todos/:id - Update an existing todo
///
/// # Errors
/// Returns `ValidationError` if the payload validation fails,
/// or `NotFound` if the todo with the specified ID does not exist
pub async fn update_todo(
    State(store): State<TodoStore>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateTodoRequest>,
) -> Result<Json<Todo>> {
    tracing::debug!(todo_id = id, "Updating todo");

    // Validation
    payload.validate().map_err(AppError::ValidationError)?;

    store
        .update(id, payload.title, payload.description, payload.completed)
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {id} not found")))
}

/// DELETE /api/todos/:id - Delete a todo by ID
///
/// # Errors
/// Returns `NotFound` error if the todo with the specified ID does not exist
pub async fn delete_todo(
    State(store): State<TodoStore>,
    Path(id): Path<u64>,
) -> Result<Json<serde_json::Value>> {
    tracing::debug!(todo_id = id, "Deleting todo");

    if store.delete(id) {
        Ok(Json(serde_json::json!({
            "message": format!("Todo with id {id} deleted successfully")
        })))
    } else {
        Err(AppError::NotFound(format!("Todo with id {id} not found")))
    }
}
