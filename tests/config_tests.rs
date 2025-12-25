use gent::config::Config;
use std::env;
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

// Mutex to serialize tests that modify environment variables
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_config_from_env() {
    let _lock = ENV_LOCK.lock().unwrap();
    env::set_var("OPENAI_API_KEY", "test-key-123");
    let config = Config::load();
    assert_eq!(config.openai_api_key, Some("test-key-123".to_string()));
    env::remove_var("OPENAI_API_KEY");
}

#[test]
fn test_config_missing_key() {
    let _lock = ENV_LOCK.lock().unwrap();
    env::remove_var("OPENAI_API_KEY");
    env::remove_var("ANTHROPIC_API_KEY");
    let config = Config::load();
    assert!(config.openai_api_key.is_none());
}

#[test]
fn test_config_from_file() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let env_file = dir.path().join(".gent.env");
    fs::write(&env_file, "OPENAI_API_KEY=file-key-456\n").unwrap();

    // Clear env var so file takes effect
    env::remove_var("OPENAI_API_KEY");

    let config = Config::load_from_dir(dir.path());
    assert_eq!(config.openai_api_key, Some("file-key-456".to_string()));
}

#[test]
fn test_config_env_overrides_file() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let env_file = dir.path().join(".gent.env");
    fs::write(&env_file, "OPENAI_API_KEY=file-key\n").unwrap();

    env::set_var("OPENAI_API_KEY", "env-key");

    let config = Config::load_from_dir(dir.path());
    assert_eq!(config.openai_api_key, Some("env-key".to_string()));

    env::remove_var("OPENAI_API_KEY");
}

#[test]
fn test_config_default_model() {
    let _lock = ENV_LOCK.lock().unwrap();
    env::set_var("GENT_DEFAULT_MODEL", "gpt-4o");
    let config = Config::load();
    assert_eq!(config.default_model, Some("gpt-4o".to_string()));
    env::remove_var("GENT_DEFAULT_MODEL");
}
