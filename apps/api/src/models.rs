use serde::{Deserialize, Serialize};

/// Todo リソースのデータモデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
}

/// Todo作成時のリクエストボディ
#[derive(Debug, Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
}

/// Todo更新時のリクエストボディ
#[derive(Debug, Deserialize)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
}

impl CreateTodoRequest {
    /// バリデーション
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        if self.title.len() > 200 {
            return Err("Title must be 200 characters or less".to_string());
        }
        if let Some(desc) = &self.description
            && desc.len() > 1000
        {
            return Err("Description must be 1000 characters or less".to_string());
        }
        Ok(())
    }
}

impl UpdateTodoRequest {
    /// バリデーション
    pub fn validate(&self) -> Result<(), String> {
        if let Some(title) = &self.title {
            if title.trim().is_empty() {
                return Err("Title cannot be empty".to_string());
            }
            if title.len() > 200 {
                return Err("Title must be 200 characters or less".to_string());
            }
        }
        if let Some(desc) = &self.description
            && desc.len() > 1000
        {
            return Err("Description must be 1000 characters or less".to_string());
        }
        Ok(())
    }
}
