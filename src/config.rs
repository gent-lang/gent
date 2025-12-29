//! Configuration loading for GENT

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

/// GENT configuration
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub default_model: Option<String>,
    /// Whether to use mock LLM instead of real API calls
    pub mock_mode: bool,
    /// Custom response to return in mock mode
    pub mock_response: Option<String>,
}

impl Config {
    /// Load configuration from environment and .gent.env file
    pub fn load() -> Self {
        Self::load_from_dir(Path::new("."))
    }

    /// Load configuration from a specific directory
    pub fn load_from_dir(dir: &Path) -> Self {
        let mut config = Self::default();

        // First, load from .gent.env file (lower priority)
        let env_file = dir.join(".gent.env");
        if env_file.exists() {
            if let Ok(contents) = fs::read_to_string(&env_file) {
                let file_vars = Self::parse_env_file(&contents);
                config.apply_vars(&file_vars);
            }
        }

        // Then, load from environment variables (higher priority)
        config.apply_env_vars();

        config
    }

    fn parse_env_file(contents: &str) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                vars.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        vars
    }

    fn apply_vars(&mut self, vars: &HashMap<String, String>) {
        if let Some(key) = vars.get("OPENAI_API_KEY") {
            self.openai_api_key = Some(key.clone());
        }
        if let Some(key) = vars.get("ANTHROPIC_API_KEY") {
            self.anthropic_api_key = Some(key.clone());
        }
        if let Some(model) = vars.get("GENT_DEFAULT_MODEL") {
            self.default_model = Some(model.clone());
        }
    }

    fn apply_env_vars(&mut self) {
        if let Ok(key) = env::var("OPENAI_API_KEY") {
            self.openai_api_key = Some(key);
        }
        if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
            self.anthropic_api_key = Some(key);
        }
        if let Ok(model) = env::var("GENT_DEFAULT_MODEL") {
            self.default_model = Some(model);
        }
    }

    /// Get OpenAI API key or return error
    pub fn require_openai_key(&self) -> Result<&str, crate::errors::GentError> {
        self.openai_api_key
            .as_deref()
            .ok_or_else(|| crate::errors::GentError::MissingApiKey {
                provider: "OPENAI".to_string(),
            })
    }

    /// Get Anthropic API key or return error
    pub fn require_anthropic_key(&self) -> Result<&str, crate::errors::GentError> {
        self.anthropic_api_key
            .as_deref()
            .ok_or_else(|| crate::errors::GentError::MissingApiKey {
                provider: "ANTHROPIC".to_string(),
            })
    }

    /// Create a config with mock mode enabled (for testing)
    pub fn mock() -> Self {
        Self {
            mock_mode: true,
            ..Default::default()
        }
    }

    /// Create a config with mock mode and a custom response (for testing)
    pub fn mock_with_response(response: impl Into<String>) -> Self {
        Self {
            mock_mode: true,
            mock_response: Some(response.into()),
            ..Default::default()
        }
    }
}
