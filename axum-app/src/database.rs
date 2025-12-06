use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Database manager for conversation storage
#[derive(Debug, Clone)]
pub struct ChatDatabase {
    // For now, we'll use in-memory storage until libsql API is properly figured out
    conversations: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Conversation>>>,
    messages: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Vec<Message>>>>,
}

impl ChatDatabase {
    /// Create a new database connection
    pub async fn new(_database_url: &str) -> Result<Self> {
        let chat_db = Self {
            conversations: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            messages: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        };

        // Initialize database schema (for in-memory, this is just setup)
        chat_db.init_schema().await?;

        Ok(chat_db)
    }

    /// Initialize database schema
    async fn init_schema(&self) -> Result<()> {
        // For in-memory storage, no schema initialization needed
        // In the future, this would create tables if using a real database
        Ok(())
    }

    /// Save a new conversation
    pub async fn save_conversation(&self, conversation: &Conversation) -> Result<()> {
        let mut conversations = self.conversations.write().await;
        conversations.insert(conversation.id.clone(), conversation.clone());
        Ok(())
    }

    /// Get a conversation by ID
    pub async fn get_conversation(&self, conversation_id: &str) -> Result<Option<Conversation>> {
        let conversations = self.conversations.read().await;
        Ok(conversations.get(conversation_id).cloned())
    }

    /// Get all conversations
    pub async fn get_all_conversations(&self) -> Result<Vec<Conversation>> {
        let conversations = self.conversations.read().await;
        let mut sorted_conversations: Vec<_> = conversations.values().cloned().collect();
        sorted_conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(sorted_conversations)
    }

    /// Delete a conversation and all its messages
    pub async fn delete_conversation(&self, conversation_id: &str) -> Result<()> {
        // Delete conversation
        {
            let mut conversations = self.conversations.write().await;
            conversations.remove(conversation_id);
        }

        // Delete associated messages
        {
            let mut messages = self.messages.write().await;
            messages.remove(conversation_id);
        }

        Ok(())
    }

    /// Save a message
    pub async fn save_message(&self, message: &Message) -> Result<()> {
        let mut messages = self.messages.write().await;
        let conversation_messages = messages.entry(message.conversation_id.clone()).or_insert_with(Vec::new);

        // Remove existing message with same ID if it exists
        conversation_messages.retain(|m| m.id != message.id);

        // Add the new message
        conversation_messages.push(message.clone());

        // Sort messages by created_at
        conversation_messages.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        // Update conversation's updated_at timestamp
        if let Some(conversation) = self.conversations.read().await.get(&message.conversation_id) {
            let mut conversations = self.conversations.write().await;
            if let Some(conv) = conversations.get_mut(&message.conversation_id) {
                conv.updated_at = message.created_at;
            }
        }

        Ok(())
    }

    /// Get all messages for a conversation
    pub async fn get_conversation_messages(&self, conversation_id: &str) -> Result<Vec<Message>> {
        let messages = self.messages.read().await;
        Ok(messages.get(conversation_id).cloned().unwrap_or_default())
    }

    /// Delete a message
    pub async fn delete_message(&self, message_id: &str) -> Result<()> {
        let mut messages = self.messages.write().await;
        for (_, conversation_messages) in messages.iter_mut() {
            conversation_messages.retain(|m| m.id != message_id);
        }
        Ok(())
    }

    /// Search conversations by title or content
    pub async fn search_conversations(&self, query: &str) -> Result<Vec<Conversation>> {
        let conversations = self.conversations.read().await;
        let messages = self.messages.read().await;

        let mut matching_conversations = Vec::new();
        let query_lower = query.to_lowercase();

        for conversation in conversations.values() {
            // Check if query matches conversation title
            if conversation.title.to_lowercase().contains(&query_lower) {
                matching_conversations.push(conversation.clone());
                continue;
            }

            // Check if query matches any message content in this conversation
            if let Some(conversation_messages) = messages.get(&conversation.id) {
                for message in conversation_messages {
                    if message.content.to_lowercase().contains(&query_lower) {
                        matching_conversations.push(conversation.clone());
                        break;
                    }
                }
            }
        }

        // Sort by updated_at descending
        matching_conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(matching_conversations)
    }
}

/// Conversation data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

/// Message data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub attachments: Option<Vec<Attachment>>,
    pub metadata: Option<serde_json::Value>,
}

/// Attachment data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

/// Create a new conversation with default settings
pub fn create_conversation(title: &str, model: &str) -> Conversation {
    let now = Utc::now();
    Conversation {
        id: format!("conv_{}", fastrand::u64(1000..9999)),
        title: title.to_string(),
        model: model.to_string(),
        created_at: now,
        updated_at: now,
        metadata: None,
    }
}

/// Create a new message with default settings
pub fn create_message(
    conversation_id: &str,
    role: &str,
    content: &str,
    attachments: Option<Vec<Attachment>>,
) -> Message {
    Message {
        id: format!("msg_{}", fastrand::u64(1000..9999)),
        conversation_id: conversation_id.to_string(),
        role: role.to_string(),
        content: content.to_string(),
        created_at: Utc::now(),
        attachments,
        metadata: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_initialization() {
        let db = ChatDatabase::new("file::memory:").await;
        assert!(db.is_ok());
    }

    #[tokio::test]
    async fn test_conversation_crud() {
        let db = ChatDatabase::new("file::memory:").await.unwrap();

        // Create conversation
        let conversation = create_conversation("Test Conversation", "gpt-3.5-turbo");
        db.save_conversation(&conversation).await.unwrap();

        // Get conversation
        let retrieved = db.get_conversation(&conversation.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test Conversation");

        // Get all conversations
        let conversations = db.get_all_conversations().await.unwrap();
        assert_eq!(conversations.len(), 1);

        // Delete conversation
        db.delete_conversation(&conversation.id).await.unwrap();
        let deleted = db.get_conversation(&conversation.id).await.unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_message_crud() {
        let db = ChatDatabase::new("file::memory:").await.unwrap();

        // Create conversation first
        let conversation = create_conversation("Test Conversation", "gpt-3.5-turbo");
        db.save_conversation(&conversation).await.unwrap();

        // Create message
        let message = create_message(&conversation.id, "user", "Hello, world!", None);
        db.save_message(&message).await.unwrap();

        // Get messages
        let messages = db.get_conversation_messages(&conversation.id).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello, world!");

        // Delete message
        db.delete_message(&message.id).await.unwrap();
        let messages = db.get_conversation_messages(&conversation.id).await.unwrap();
        assert_eq!(messages.len(), 0);
    }

    #[tokio::test]
    async fn test_search_conversations() {
        let db = ChatDatabase::new("file::memory:").await.unwrap();

        // Create conversations
        let conv1 = create_conversation("Chat about Rust", "gpt-3.5-turbo");
        let conv2 = create_conversation("Chat about Python", "gpt-3.5-turbo");

        db.save_conversation(&conv1).await.unwrap();
        db.save_conversation(&conv2).await.unwrap();

        // Add messages
        let msg1 = create_message(&conv1.id, "user", "Tell me about Rust programming", None);
        let msg2 = create_message(&conv2.id, "user", "Explain Python decorators", None);

        db.save_message(&msg1).await.unwrap();
        db.save_message(&msg2).await.unwrap();

        // Search by title
        let results = db.search_conversations("Rust").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Chat about Rust");

        // Search by content
        let results = db.search_conversations("Python").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Chat about Python");
    }
}