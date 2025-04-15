use std::env;
use std::error::Error;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// OpenAI API response structures
#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionMessage,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

// API service for handling OpenAI interactions
struct OpenAIService {
    client: Client,
    api_key: String,
}

impl OpenAIService {
    fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    async fn transcribe_audio(&self, audio_path: &str) -> Result<String, Box<dyn Error>> {
        let audio_bytes = fs::read(audio_path).await?;

        let file_part = Part::bytes(audio_bytes)
            .file_name("audio.wav")
            .mime_str("audio/wav")?;

        let form = Form::new()
            .part("file", file_part)
            .text("model", "whisper-1");

        let response = self.client.post("https://api.openai.com/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API error: {}", error_text).into());
        }

        let whisper_response: WhisperResponse = response.json().await?;
        Ok(whisper_response.text)
    }

    async fn generate_response(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        let response = self.client.post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": "gpt-4o",
                "messages": [{"role": "user", "content": prompt}]
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API error: {}", error_text).into());
        }

        let chat_response: ChatCompletionResponse = response.json().await?;
        Ok(chat_response.choices[0].message.content.clone())
    }
}

// Voice assistant service to orchestrate the entire process
struct VoiceAssistantService {
    openai_service: OpenAIService,
}

impl VoiceAssistantService {
    fn new(api_key: String) -> Self {
        Self {
            openai_service: OpenAIService::new(api_key),
        }
    }

    async fn process_audio(&self, audio_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        // Step 1: Transcribe audio to text
        println!("Transcribing audio...");
        let transcription = self.openai_service.transcribe_audio(audio_path).await?;
        println!("Transcription: {}", transcription);

        // Step 2: Generate response using GPT
        println!("Generating response...");
        let response = self.openai_service.generate_response(&transcription).await?;
        println!("Response: {}", response);

        // Step 3: Save response to file
        let mut file = File::create(output_path).await?;
        file.write_all(response.as_bytes()).await?;
        println!("Response saved to {}", output_path);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Get API key from environment variable for security
    let api_key = env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable must be set");

    let assistant = VoiceAssistantService::new(api_key);

    // Example usage - would normally come from Flutter
    assistant.process_audio("audio.wav", "response.txt").await?;

    Ok(())
}
