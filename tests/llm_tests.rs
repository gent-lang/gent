use gent::runtime::{LLMClient, LLMResponse, Message, MockLLMClient, Role};

// ============================================
// Role Tests
// ============================================

#[test]
fn test_role_system() {
    let role = Role::System;
    assert!(matches!(role, Role::System));
}

#[test]
fn test_role_user() {
    let role = Role::User;
    assert!(matches!(role, Role::User));
}

#[test]
fn test_role_assistant() {
    let role = Role::Assistant;
    assert!(matches!(role, Role::Assistant));
}

#[test]
fn test_role_equality() {
    assert_eq!(Role::System, Role::System);
    assert_eq!(Role::User, Role::User);
    assert_ne!(Role::System, Role::User);
}

#[test]
fn test_role_debug() {
    let debug = format!("{:?}", Role::System);
    assert!(debug.contains("System"));
}

#[test]
fn test_role_clone() {
    let role = Role::User;
    let cloned = role.clone();
    assert_eq!(role, cloned);
}

// ============================================
// Message Tests
// ============================================

#[test]
fn test_message_new() {
    let msg = Message::new(Role::User, "Hello");
    assert_eq!(msg.role, Role::User);
    assert_eq!(msg.content, "Hello");
}

#[test]
fn test_message_system() {
    let msg = Message::system("You are helpful.");
    assert_eq!(msg.role, Role::System);
    assert_eq!(msg.content, "You are helpful.");
}

#[test]
fn test_message_user() {
    let msg = Message::user("Hi there!");
    assert_eq!(msg.role, Role::User);
    assert_eq!(msg.content, "Hi there!");
}

#[test]
fn test_message_assistant() {
    let msg = Message::assistant("Hello!");
    assert_eq!(msg.role, Role::Assistant);
    assert_eq!(msg.content, "Hello!");
}

#[test]
fn test_message_string_conversion() {
    let msg = Message::user(String::from("test"));
    assert_eq!(msg.content, "test");
}

#[test]
fn test_message_equality() {
    let m1 = Message::user("test");
    let m2 = Message::user("test");
    let m3 = Message::user("other");
    assert_eq!(m1, m2);
    assert_ne!(m1, m3);
}

#[test]
fn test_message_clone() {
    let m1 = Message::user("test");
    let m2 = m1.clone();
    assert_eq!(m1, m2);
}

#[test]
fn test_message_debug() {
    let msg = Message::user("test");
    let debug = format!("{:?}", msg);
    assert!(debug.contains("Message"));
    assert!(debug.contains("User"));
    assert!(debug.contains("test"));
}

// ============================================
// LLMResponse Tests
// ============================================

#[test]
fn test_llm_response_new() {
    let response = LLMResponse::new("Hello!");
    assert_eq!(response.content, "Hello!");
}

#[test]
fn test_llm_response_string_conversion() {
    let response = LLMResponse::new(String::from("test"));
    assert_eq!(response.content, "test");
}

#[test]
fn test_llm_response_equality() {
    let r1 = LLMResponse::new("test");
    let r2 = LLMResponse::new("test");
    let r3 = LLMResponse::new("other");
    assert_eq!(r1, r2);
    assert_ne!(r1, r3);
}

#[test]
fn test_llm_response_clone() {
    let r1 = LLMResponse::new("test");
    let r2 = r1.clone();
    assert_eq!(r1, r2);
}

#[test]
fn test_llm_response_debug() {
    let response = LLMResponse::new("test");
    let debug = format!("{:?}", response);
    assert!(debug.contains("LLMResponse"));
    assert!(debug.contains("test"));
}

// ============================================
// MockLLMClient Tests
// ============================================

#[test]
fn test_mock_client_new() {
    let client = MockLLMClient::new();
    assert!(client.response().contains("Hello"));
}

#[test]
fn test_mock_client_default() {
    let client = MockLLMClient::default();
    assert!(client.response().contains("friendly"));
}

#[test]
fn test_mock_client_with_response() {
    let client = MockLLMClient::with_response("Custom response");
    assert_eq!(client.response(), "Custom response");
}

#[test]
fn test_mock_client_with_response_string() {
    let client = MockLLMClient::with_response(String::from("Custom"));
    assert_eq!(client.response(), "Custom");
}

#[test]
fn test_mock_client_chat_returns_response() {
    let client = MockLLMClient::with_response("Test response");
    let messages = vec![Message::user("Hello")];
    let result = client.chat(messages);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().content, "Test response");
}

#[test]
fn test_mock_client_ignores_messages() {
    let client = MockLLMClient::with_response("Fixed response");

    let messages1 = vec![Message::user("Hello")];
    let messages2 = vec![
        Message::system("You are helpful."),
        Message::user("How are you?"),
    ];

    let r1 = client.chat(messages1).unwrap();
    let r2 = client.chat(messages2).unwrap();

    assert_eq!(r1.content, r2.content);
}

#[test]
fn test_mock_client_empty_messages() {
    let client = MockLLMClient::with_response("Response");
    let result = client.chat(vec![]);
    assert!(result.is_ok());
}

#[test]
fn test_mock_client_clone() {
    let c1 = MockLLMClient::with_response("test");
    let c2 = c1.clone();
    assert_eq!(c1.response(), c2.response());
}

#[test]
fn test_mock_client_debug() {
    let client = MockLLMClient::with_response("test");
    let debug = format!("{:?}", client);
    assert!(debug.contains("MockLLMClient"));
    assert!(debug.contains("test"));
}

// ============================================
// LLMClient Trait Tests
// ============================================

#[test]
fn test_llm_client_trait_object() {
    let client: Box<dyn LLMClient> = Box::new(MockLLMClient::with_response("Boxed"));
    let messages = vec![Message::user("test")];
    let result = client.chat(messages);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().content, "Boxed");
}

#[test]
fn test_llm_client_as_ref() {
    let client = MockLLMClient::with_response("Ref test");
    let client_ref: &dyn LLMClient = &client;
    let result = client_ref.chat(vec![Message::user("hi")]);
    assert!(result.is_ok());
}

// ============================================
// Thread Safety Tests
// ============================================

#[test]
fn test_mock_client_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<MockLLMClient>();
}

#[test]
fn test_mock_client_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<MockLLMClient>();
}
