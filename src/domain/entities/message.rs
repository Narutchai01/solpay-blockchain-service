use serde::{Deserialize, Serialize};

/// Represents a message received from the message queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMessage {
    pub id: String,
    pub payload: serde_json::Value,
    pub message_type: String,
    #[serde(default)]
    pub created_at: Option<String>,
}

impl QueueMessage {
    pub fn new(id: String, payload: serde_json::Value, message_type: String) -> Self {
        Self {
            id,
            payload,
            message_type,
            created_at: None,
        }
    }
}
