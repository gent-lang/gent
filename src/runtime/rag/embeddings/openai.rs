//! OpenAI embeddings provider

use super::EmbeddingProvider;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// OpenAI embeddings provider using the embeddings API
pub struct OpenAIEmbeddings {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAIEmbeddings {
    /// Create a new OpenAI embeddings provider with the default model (text-embedding-3-small)
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: "text-embedding-3-small".to_string(),
        }
    }

    /// Set the model to use for embeddings
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }
}

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddings {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let results = self.embed_batch(&[text]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| "No embedding returned".to_string())
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String> {
        let request = EmbeddingRequest {
            model: self.model.clone(),
            input: texts.iter().map(|s| s.to_string()).collect(),
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("OpenAI API error: {}", error_text));
        }

        let data: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(data.data.into_iter().map(|d| d.embedding).collect())
    }

    fn dimensions(&self) -> usize {
        match self.model.as_str() {
            "text-embedding-3-large" => 3072,
            _ => 1536,
        }
    }
}
