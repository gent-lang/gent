use gent::runtime::{LLMClient, LLMResponse, Message, MockLLMClient, Role};

// ============================================
// Role Tests
// ============================================

#[tokio::test]
async fn test_role_system() {
    let role = Role::System;
    assert!(matches!(role, Role::System));
}

#[tokio::test]
async fn test_role_user() {
    let role = Role::User;
    assert!(matches!(role, Role::User));
}

#[tokio::test]
async fn test_role_assistant() {
    let role = Role::Assistant;
    assert!(matches!(role, Role::Assistant));
}

#[tokio::test]
async fn test_role_equality() {
    assert_eq!(Role::System, Role::System);
    assert_eq!(Role::User, Role::User);
    assert_ne!(Role::System, Role::User);
}

#[tokio::test]
async fn test_role_debug() {
    let debug = format!("{:?}", Role::System);
    assert!(debug.contains("System"));
}

#[tokio::test]
async fn test_role_clone() {
    let role = Role::User;
    let cloned = role.clone();
    assert_eq!(role, cloned);
}

// ============================================
// Message Tests
// ============================================

#[tokio::test]
async fn test_message_new() {
    let msg = Message::new(Role::User, "Hello");
    assert_eq!(msg.role, Role::User);
    assert_eq!(msg.content, "Hello");
}

#[tokio::test]
async fn test_message_system() {
    let msg = Message::system("You are helpful.");
    assert_eq!(msg.role, Role::System);
    assert_eq!(msg.content, "You are helpful.");
}

#[tokio::test]
async fn test_message_user() {
    let msg = Message::user("Hi there!");
    assert_eq!(msg.role, Role::User);
    assert_eq!(msg.content, "Hi there!");
}

#[tokio::test]
async fn test_message_assistant() {
    let msg = Message::assistant("Hello!");
    assert_eq!(msg.role, Role::Assistant);
    assert_eq!(msg.content, "Hello!");
}

#[tokio::test]
async fn test_message_string_conversion() {
    let msg = Message::user(String::from("test"));
    assert_eq!(msg.content, "test");
}

#[tokio::test]
async fn test_message_equality() {
    let m1 = Message::user("test");
    let m2 = Message::user("test");
    let m3 = Message::user("other");
    assert_eq!(m1, m2);
    assert_ne!(m1, m3);
}

#[tokio::test]
async fn test_message_clone() {
    let m1 = Message::user("test");
    let m2 = m1.clone();
    assert_eq!(m1, m2);
}

#[tokio::test]
async fn test_message_debug() {
    let msg = Message::user("test");
    let debug = format!("{:?}", msg);
    assert!(debug.contains("Message"));
    assert!(debug.contains("User"));
    assert!(debug.contains("test"));
}

// ============================================
// LLMResponse Tests
// ============================================

#[tokio::test]
async fn test_llm_response_new() {
    let response = LLMResponse::new("Hello!");
    assert_eq!(response.content, Some("Hello!".to_string()));
}

#[tokio::test]
async fn test_llm_response_string_conversion() {
    let response = LLMResponse::new(String::from("test"));
    assert_eq!(response.content, Some("test".to_string()));
}

#[tokio::test]
async fn test_llm_response_equality() {
    let r1 = LLMResponse::new("test");
    let r2 = LLMResponse::new("test");
    let r3 = LLMResponse::new("other");
    assert_eq!(r1, r2);
    assert_ne!(r1, r3);
}

#[tokio::test]
async fn test_llm_response_clone() {
    let r1 = LLMResponse::new("test");
    let r2 = r1.clone();
    assert_eq!(r1, r2);
}

#[tokio::test]
async fn test_llm_response_debug() {
    let response = LLMResponse::new("test");
    let debug = format!("{:?}", response);
    assert!(debug.contains("LLMResponse"));
    assert!(debug.contains("test"));
}

// ============================================
// MockLLMClient Tests
// ============================================

#[tokio::test]
async fn test_mock_client_new() {
    let client = MockLLMClient::new();
    assert!(client.response().contains("Hello"));
}

#[tokio::test]
async fn test_mock_client_default() {
    let client = MockLLMClient::default();
    assert!(client.response().contains("friendly"));
}

#[tokio::test]
async fn test_mock_client_with_response() {
    let client = MockLLMClient::with_response("Custom response");
    assert_eq!(client.response(), "Custom response");
}

#[tokio::test]
async fn test_mock_client_with_response_string() {
    let client = MockLLMClient::with_response(String::from("Custom"));
    assert_eq!(client.response(), "Custom");
}

#[tokio::test]
async fn test_mock_client_chat_returns_response() {
    let client = MockLLMClient::with_response("Test response");
    let messages = vec![Message::user("Hello")];
    let result = client.chat(messages, vec![]).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().content, Some("Test response".to_string()));
}

#[tokio::test]
async fn test_mock_client_ignores_messages() {
    let client = MockLLMClient::with_response("Fixed response");

    let messages1 = vec![Message::user("Hello")];
    let messages2 = vec![
        Message::system("You are helpful."),
        Message::user("How are you?"),
    ];

    let r1 = client.chat(messages1, vec![]).await.unwrap();
    let r2 = client.chat(messages2, vec![]).await.unwrap();

    assert_eq!(r1.content, r2.content);
}

#[tokio::test]
async fn test_mock_client_empty_messages() {
    let client = MockLLMClient::with_response("Response");
    let result = client.chat(vec![], vec![]).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mock_client_clone() {
    let c1 = MockLLMClient::with_response("test");
    let c2 = c1.clone();
    assert_eq!(c1.response(), c2.response());
}

#[tokio::test]
async fn test_mock_client_debug() {
    let client = MockLLMClient::with_response("test");
    let debug = format!("{:?}", client);
    assert!(debug.contains("MockLLMClient"));
    assert!(debug.contains("test"));
}

// ============================================
// LLMClient Trait Tests
// ============================================

#[tokio::test]
async fn test_llm_client_trait_object() {
    let client: Box<dyn LLMClient> = Box::new(MockLLMClient::with_response("Boxed"));
    let messages = vec![Message::user("test")];
    let result = client.chat(messages, vec![]).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().content, Some("Boxed".to_string()));
}

#[tokio::test]
async fn test_llm_client_as_ref() {
    let client = MockLLMClient::with_response("Ref test");
    let client_ref: &dyn LLMClient = &client;
    let result = client_ref.chat(vec![Message::user("hi")], vec![]).await;
    assert!(result.is_ok());
}

// ============================================
// Thread Safety Tests
// ============================================

#[tokio::test]
async fn test_mock_client_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<MockLLMClient>();
}

#[tokio::test]
async fn test_mock_client_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<MockLLMClient>();
}
