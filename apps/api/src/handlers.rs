use crate::error::{AppError, Result};
use crate::models::{CreateTodoRequest, Todo, UpdateTodoRequest};
use crate::store::TodoStore;
use axum::{
    Json,
    extract::{Path, State},
};

/// GET /todos - すべてのTodoを取得
pub async fn get_todos(State(store): State<TodoStore>) -> Result<Json<Vec<Todo>>> {
    tracing::debug!("Fetching all todos");
    let todos = store.get_all();
    Ok(Json(todos))
}

/// GET /todos/:id - 特定のTodoを取得
pub async fn get_todo(State(store): State<TodoStore>, Path(id): Path<u64>) -> Result<Json<Todo>> {
    tracing::debug!(todo_id = id, "Fetching todo by id");

    store
        .get_by_id(id)
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {id} not found")))
}

/// POST /todos - 新しいTodoを作成
pub async fn create_todo(
    State(store): State<TodoStore>,
    Json(payload): Json<CreateTodoRequest>,
) -> Result<Json<Todo>> {
    tracing::debug!(title = %payload.title, "Creating new todo");

    // バリデーション
    payload.validate().map_err(AppError::ValidationError)?;

    let todo = store.create(payload.title, payload.description);
    Ok(Json(todo))
}

/// PUT /todos/:id - Todoを更新
pub async fn update_todo(
    State(store): State<TodoStore>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateTodoRequest>,
) -> Result<Json<Todo>> {
    tracing::debug!(todo_id = id, "Updating todo");

    // バリデーション
    payload.validate().map_err(AppError::ValidationError)?;

    store
        .update(id, payload.title, payload.description, payload.completed)
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {id} not found")))
}

/// DELETE /todos/:id - Todoを削除
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
