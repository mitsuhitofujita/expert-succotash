use crate::models::Todo;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// インメモリのTodoデータストア
#[derive(Debug, Clone)]
pub struct TodoStore {
    todos: Arc<Mutex<HashMap<u64, Todo>>>,
    next_id: Arc<Mutex<u64>>,
}

impl TodoStore {
    /// 新しい `TodoStore` を作成
    pub fn new() -> Self {
        Self {
            todos: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    /// 全Todoを取得
    pub fn get_all(&self) -> Vec<Todo> {
        let todos = self.todos.lock().unwrap();
        todos.values().cloned().collect()
    }

    /// ID で `Todo` を取得
    pub fn get_by_id(&self, id: u64) -> Option<Todo> {
        let todos = self.todos.lock().unwrap();
        todos.get(&id).cloned()
    }

    /// 新しい `Todo` を作成
    pub fn create(&self, title: String, description: Option<String>) -> Todo {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        drop(next_id);

        let todo = Todo {
            id,
            title,
            description,
            completed: false,
        };

        self.todos.lock().unwrap().insert(id, todo.clone());

        tracing::info!(todo_id = id, "Created new todo");
        todo
    }

    /// `Todo` を更新
    pub fn update(
        &self,
        id: u64,
        title: Option<String>,
        description: Option<String>,
        completed: Option<bool>,
    ) -> Option<Todo> {
        let mut todos = self.todos.lock().unwrap();

        if let Some(todo) = todos.get_mut(&id) {
            if let Some(t) = title {
                todo.title = t;
            }
            if let Some(d) = description {
                todo.description = Some(d);
            }
            if let Some(c) = completed {
                todo.completed = c;
            }

            tracing::info!(todo_id = id, "Updated todo");
            Some(todo.clone())
        } else {
            None
        }
    }

    /// `Todo` を削除
    pub fn delete(&self, id: u64) -> bool {
        let mut todos = self.todos.lock().unwrap();
        if todos.remove(&id).is_some() {
            tracing::info!(todo_id = id, "Deleted todo");
            true
        } else {
            false
        }
    }
}

impl Default for TodoStore {
    fn default() -> Self {
        Self::new()
    }
}
