use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// Import shared types from chat module
use crate::chat::{Attachment, ChatMessage, ChatRole, Usage};

/// Database manager for conversation storage
#[derive(Debug, Clone)]
pub struct ChatDatabase {
    // For now, we'll use in-memory storage until libsql API is properly figured out
    conversations: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Conversation>>>,
    messages: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Vec<EnhancedMessage>>>>,
    // Separate storage for ChatMessage compatibility (legacy support)
    chat_messages: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Vec<ChatMessage>>>>,
}

impl ChatDatabase {
    /// Create a new database connection
    pub async fn new(_database_url: &str) -> Result<Self> {
        let chat_db = Self {
            conversations: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            messages: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            chat_messages: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
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

        // Delete associated enhanced messages
        {
            let mut messages = self.messages.write().await;
            messages.remove(conversation_id);
        }

        // Delete associated chat messages (legacy support)
        {
            let mut chat_messages = self.chat_messages.write().await;
            chat_messages.remove(conversation_id);
        }

        Ok(())
    }

    /// Save an enhanced message (AI SDK compatible)
    pub async fn save_enhanced_message(&self, message: &EnhancedMessage) -> Result<()> {
        let mut messages = self.messages.write().await;
        let conversation_messages = messages
            .entry(message.conversation_id.clone())
            .or_insert_with(Vec::new);

        // Remove existing message with same ID if it exists
        conversation_messages.retain(|m| m.id != message.id);

        // Add the new message
        conversation_messages.push(message.clone());

        // Sort messages by created_at
        if let Some(created_at) = message.created_at {
            conversation_messages.sort_by(|a, b| {
                a.created_at
                    .unwrap_or_default()
                    .cmp(&b.created_at.unwrap_or_default())
            });
        }

        // Update conversation's updated_at timestamp
        if let Some(conversation) = self
            .conversations
            .read()
            .await
            .get(&message.conversation_id)
        {
            let mut conversations = self.conversations.write().await;
            if let Some(conv) = conversations.get_mut(&message.conversation_id) {
                conv.updated_at = message.created_at.unwrap_or_else(|| chrono::Utc::now());
            }
        }

        Ok(())
    }

    /// Save a legacy ChatMessage (backward compatibility)
    pub async fn save_chat_message(&self, message: &ChatMessage) -> Result<()> {
        let mut chat_messages = self.chat_messages.write().await;
        let conversation_messages = chat_messages
            .entry(extract_conversation_id(&message.id))
            .or_insert_with(Vec::new);

        // Remove existing message with same ID if it exists
        conversation_messages.retain(|m| m.id != message.id);

        // Add the new message
        conversation_messages.push(message.clone());

        // Sort messages by created_at
        conversation_messages.sort_by(|a, b| {
            a.created_at
                .unwrap_or_default()
                .cmp(&b.created_at.unwrap_or_default())
        });

        // Also convert to enhanced message and save
        let enhanced = convert_chat_to_enhanced(message);
        self.save_enhanced_message(&enhanced).await?;

        Ok(())
    }

    /// Legacy message save method (renamed from save_message)
    pub async fn save_legacy_message(&self, message: &Message) -> Result<()> {
        let mut messages = self.messages.write().await;
        let conversation_messages = messages
            .entry(message.conversation_id.clone())
            .or_insert_with(Vec::new);

        // Convert legacy Message to EnhancedMessage
        let enhanced = convert_legacy_to_enhanced(message);

        // Save as enhanced message
        self.save_enhanced_message(&enhanced).await
    }

    /// Get all enhanced messages for a conversation
    pub async fn get_enhanced_messages(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<EnhancedMessage>> {
        let messages = self.messages.read().await;
        Ok(messages.get(conversation_id).cloned().unwrap_or_default())
    }

    /// Get all legacy ChatMessages for a conversation
    pub async fn get_chat_messages(&self, conversation_id: &str) -> Result<Vec<ChatMessage>> {
        let chat_messages = self.chat_messages.read().await;
        Ok(chat_messages
            .get(conversation_id)
            .cloned()
            .unwrap_or_default())
    }

    /// Get ChatMessages (for backward compatibility)
    pub async fn get_conversation_messages(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<EnhancedMessage>> {
        self.get_enhanced_messages(conversation_id).await
    }

    /// Delete an enhanced message
    pub async fn delete_enhanced_message(&self, message_id: &str) -> Result<()> {
        let mut messages = self.messages.write().await;
        for (_, conversation_messages) in messages.iter_mut() {
            conversation_messages.retain(|m| m.id != message_id);
        }

        // Also delete from chat_messages if exists
        let mut chat_messages = self.chat_messages.write().await;
        for (_, conversation_messages) in chat_messages.iter_mut() {
            conversation_messages.retain(|m| m.id != message_id);
        }

        Ok(())
    }

    /// Delete a legacy message
    pub async fn delete_message(&self, message_id: &str) -> Result<()> {
        self.delete_enhanced_message(message_id).await
    }

    /// Search conversations by title or content (searches both enhanced and legacy messages)
    pub async fn search_conversations(&self, query: &str) -> Result<Vec<Conversation>> {
        let conversations = self.conversations.read().await;
        let enhanced_messages = self.messages.read().await;
        let chat_messages = self.chat_messages.read().await;

        let mut matching_conversations = Vec::new();
        let query_lower = query.to_lowercase();

        for conversation in conversations.values() {
            // Check if query matches conversation title
            if conversation.title.to_lowercase().contains(&query_lower) {
                matching_conversations.push(conversation.clone());
                continue;
            }

            // Check if query matches any enhanced message content in this conversation
            if let Some(conversation_messages) = enhanced_messages.get(&conversation.id) {
                for message in conversation_messages {
                    if message.content.to_lowercase().contains(&query_lower) {
                        matching_conversations.push(conversation.clone());
                        break;
                    }
                }
            }

            // Also check legacy messages if no match found in enhanced messages
            if matching_conversations.is_empty()
                || matching_conversations.last().unwrap().id != conversation.id
            {
                if let Some(conversation_messages) = chat_messages.get(&conversation.id) {
                    for message in conversation_messages {
                        if message.content.to_lowercase().contains(&query_lower) {
                            matching_conversations.push(conversation.clone());
                            break;
                        }
                    }
                }
            }
        }

        // Sort by updated_at descending
        matching_conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(matching_conversations)
    }

    /// Get conversation statistics
    pub async fn get_conversation_stats(&self, conversation_id: &str) -> Result<ConversationStats> {
        let enhanced_messages = self.messages.read().await;
        let chat_messages = self.chat_messages.read().await;

        let enhanced_count = enhanced_messages
            .get(conversation_id)
            .map(|msgs| msgs.len())
            .unwrap_or(0);

        let chat_count = chat_messages
            .get(conversation_id)
            .map(|msgs| msgs.len())
            .unwrap_or(0);

        let total_tokens = enhanced_messages
            .get(conversation_id)
            .map(|msgs| {
                msgs.iter()
                    .filter_map(|msg| msg.usage.as_ref())
                    .map(|usage| usage.total_tokens)
                    .sum::<u32>()
            })
            .unwrap_or(0);

        let has_tool_calls = enhanced_messages
            .get(conversation_id)
            .map(|msgs| msgs.iter().any(|msg| msg.tool_calls.is_some()))
            .unwrap_or(false);

        Ok(ConversationStats {
            message_count: enhanced_count.max(chat_count),
            total_tokens,
            has_tool_calls,
            last_activity: self
                .conversations
                .read()
                .await
                .get(conversation_id)
                .map(|conv| conv.updated_at)
                .unwrap_or_else(|| chrono::Utc::now()),
        })
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

// Use ChatMessage and Attachment from chat module instead of defining duplicates

/// Enhanced message structure with AI SDK specific fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMessage {
    // Base message fields (compatible with ChatMessage)
    pub id: String,
    pub conversation_id: String,
    pub role: ChatRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    // AI SDK specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_results: Option<Vec<ToolResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_message_id: Option<String>, // For message branching
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>, // Whether this was a streaming response
}

/// Tool call structure for AI SDK compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,
}

/// Tool function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}

/// Tool result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub result: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
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

/// Create a new enhanced message with default settings
pub fn create_enhanced_message(
    conversation_id: &str,
    role: ChatRole,
    content: &str,
    model: Option<String>,
) -> EnhancedMessage {
    EnhancedMessage {
        id: format!("msg_{}", fastrand::u64(1000..9999)),
        conversation_id: conversation_id.to_string(),
        role,
        content: content.to_string(),
        created_at: Some(Utc::now()),
        attachments: None,
        metadata: None,
        model,
        usage: None,
        tool_calls: None,
        tool_results: None,
        finish_reason: None,
        parent_message_id: None,
        streaming: Some(false),
    }
}

/// Create a new legacy message with default settings
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

/// Conversation statistics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationStats {
    pub message_count: usize,
    pub total_tokens: u32,
    pub has_tool_calls: bool,
    pub last_activity: DateTime<Utc>,
}

// Helper functions for converting between message types

/// Extract conversation ID from message ID (for legacy chat messages)
fn extract_conversation_id(message_id: &str) -> String {
    // Extract conversation ID from message ID in format "conv_{id}_msg_{msg_id}"
    if let Some(conv_part) = message_id.strip_prefix("conv_") {
        if let Some(end_idx) = conv_part.find("_msg_") {
            return conv_part[..end_idx].to_string();
        }
    }

    // Fallback to the whole ID if pattern doesn't match
    message_id.to_string()
}

/// Convert ChatMessage to EnhancedMessage
fn convert_chat_to_enhanced(chat_message: &ChatMessage) -> EnhancedMessage {
    // Convert metadata from HashMap to serde_json::Value
    let metadata = chat_message.metadata.as_ref().map(|hashmap| {
        serde_json::Value::Object(serde_json::Map::from_iter(
            hashmap.iter().map(|(k, v)| (k.clone(), v.clone())),
        ))
    });

    EnhancedMessage {
        id: chat_message.id.clone(),
        conversation_id: extract_conversation_id(&chat_message.id),
        role: chat_message.role.clone(),
        content: chat_message.content.clone(),
        created_at: chat_message.created_at,
        attachments: chat_message.attachments.clone(),
        metadata,
        model: None,
        usage: None,
        tool_calls: None,
        tool_results: None,
        finish_reason: None,
        parent_message_id: None,
        streaming: None,
    }
}

/// Convert legacy Message to EnhancedMessage
fn convert_legacy_to_enhanced(legacy_message: &Message) -> EnhancedMessage {
    // Convert role string to ChatRole enum
    let role = match legacy_message.role.to_lowercase().as_str() {
        "user" => ChatRole::User,
        "assistant" => ChatRole::Assistant,
        "system" => ChatRole::System,
        _ => ChatRole::User, // Default fallback
    };

    // Convert metadata from HashMap to serde_json::Value
    let metadata = legacy_message.metadata.as_ref().map(|hashmap| {
        serde_json::Value::Object(serde_json::Map::from_iter(
            hashmap.iter().map(|(k, v)| (k.clone(), v.clone())),
        ))
    });

    EnhancedMessage {
        id: legacy_message.id.clone(),
        conversation_id: legacy_message.conversation_id.clone(),
        role,
        content: legacy_message.content.clone(),
        created_at: Some(legacy_message.created_at),
        attachments: legacy_message.attachments.clone(),
        metadata,
        model: None,
        usage: None,
        tool_calls: None,
        tool_results: None,
        finish_reason: None,
        parent_message_id: None,
        streaming: None,
    }
}

// Legacy Message type (keep for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub attachments: Option<Vec<Attachment>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
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
    async fn test_enhanced_message_crud() {
        let db = ChatDatabase::new("file::memory:").await.unwrap();

        // Create conversation first
        let conversation = create_conversation("Test Conversation", "gpt-3.5-turbo");
        db.save_conversation(&conversation).await.unwrap();

        // Create enhanced message
        let message = create_enhanced_message(
            &conversation.id,
            ChatRole::User,
            "Hello, world!",
            Some("gpt-3.5-turbo".to_string()),
        );
        db.save_enhanced_message(&message).await.unwrap();

        // Get messages
        let messages = db.get_enhanced_messages(&conversation.id).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello, world!");
        assert_eq!(messages[0].role, ChatRole::User);
        assert_eq!(messages[0].model, Some("gpt-3.5-turbo".to_string()));

        // Delete message
        db.delete_enhanced_message(&message.id).await.unwrap();
        let messages = db.get_enhanced_messages(&conversation.id).await.unwrap();
        assert_eq!(messages.len(), 0);
    }

    #[tokio::test]
    async fn test_chat_message_compatibility() {
        let db = ChatDatabase::new("file::memory:").await.unwrap();

        // Create conversation first
        let conversation = create_conversation("Test Conversation", "gpt-3.5-turbo");
        db.save_conversation(&conversation).await.unwrap();

        // Create ChatMessage
        let chat_message = ChatMessage {
            id: format!("conv_{}_msg_123", conversation.id),
            role: ChatRole::User,
            content: "Hello from ChatMessage!".to_string(),
            created_at: Some(chrono::Utc::now()),
            attachments: None,
            metadata: None,
        };

        // Save as ChatMessage
        db.save_chat_message(&chat_message).await.unwrap();

        // Get messages as ChatMessage
        let chat_messages = db.get_chat_messages(&conversation.id).await.unwrap();
        assert_eq!(chat_messages.len(), 1);
        assert_eq!(chat_messages[0].content, "Hello from ChatMessage!");

        // Get as EnhancedMessage
        let enhanced_messages = db.get_enhanced_messages(&conversation.id).await.unwrap();
        assert_eq!(enhanced_messages.len(), 1);
        assert_eq!(enhanced_messages[0].content, "Hello from ChatMessage!");
        assert_eq!(enhanced_messages[0].role, ChatRole::User);
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

        db.save_legacy_message(&msg1).await.unwrap();
        db.save_legacy_message(&msg2).await.unwrap();

        // Search by title
        let results = db.search_conversations("Rust").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Chat about Rust");

        // Search by content
        let results = db.search_conversations("Python").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Chat about Python");
    }

    #[tokio::test]
    async fn test_ai_sdk_features() {
        let db = ChatDatabase::new("file::memory:").await.unwrap();

        // Create conversation
        let conversation = create_conversation("AI SDK Test", "gpt-4");
        db.save_conversation(&conversation).await.unwrap();

        // Create message with AI SDK features
        let tool_call = ToolCall {
            id: "tool_123".to_string(),
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: "get_weather".to_string(),
                description: Some("Get current weather".to_string()),
                parameters: serde_json::json!({"location": "San Francisco"}),
            },
            args: Some(serde_json::json!({"location": "San Francisco"})),
        };

        let usage = Usage {
            prompt_tokens: 50,
            completion_tokens: 30,
            total_tokens: 80,
        };

        let message = EnhancedMessage {
            id: "msg_ai_sdk".to_string(),
            conversation_id: conversation.id.clone(),
            role: ChatRole::Assistant,
            content: "I'll help you get the weather information.".to_string(),
            created_at: Some(chrono::Utc::now()),
            attachments: None,
            metadata: None,
            model: Some("gpt-4".to_string()),
            usage: Some(usage),
            tool_calls: Some(vec![tool_call]),
            tool_results: None,
            finish_reason: Some("tool_calls".to_string()),
            parent_message_id: None,
            streaming: Some(true),
        };

        db.save_enhanced_message(&message).await.unwrap();

        // Get conversation stats
        let stats = db.get_conversation_stats(&conversation.id).await.unwrap();
        assert_eq!(stats.message_count, 1);
        assert_eq!(stats.total_tokens, 80);
        assert!(stats.has_tool_calls);

        // Verify AI SDK features are preserved
        let messages = db.get_enhanced_messages(&conversation.id).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].model, Some("gpt-4".to_string()));
        assert!(messages[0].tool_calls.is_some());
        assert_eq!(messages[0].usage.as_ref().unwrap().total_tokens, 80);
        assert_eq!(messages[0].finish_reason, Some("tool_calls".to_string()));
        assert_eq!(messages[0].streaming, Some(true));
    }

    #[tokio::test]
    async fn test_legacy_message_crud() {
        let db = ChatDatabase::new("file::memory:").await.unwrap();

        // Create conversation first
        let conversation = create_conversation("Test Conversation", "gpt-3.5-turbo");
        db.save_conversation(&conversation).await.unwrap();

        // Create legacy message
        let message = create_message(&conversation.id, "user", "Hello, legacy!", None);
        db.save_legacy_message(&message).await.unwrap();

        // Get messages
        let messages = db
            .get_conversation_messages(&conversation.id)
            .await
            .unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello, legacy!");
        assert_eq!(messages[0].role, ChatRole::User); // Should be converted to ChatRole

        // Delete message
        db.delete_message(&message.id).await.unwrap();
        let messages = db
            .get_conversation_messages(&conversation.id)
            .await
            .unwrap();
        assert_eq!(messages.len(), 0);
    }
}

