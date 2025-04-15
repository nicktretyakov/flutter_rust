use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;
use std::ffi::{c_char, CStr, CString};
use std::ptr;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

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

        let response = self
            .client
            .post("https://api.openai.com/v1/audio/transcriptions")
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
        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
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

    async fn process_audio(
        &self,
        audio_path: &str,
        output_path: &str,
    ) -> Result<(), Box<dyn Error>> {
        // Step 1: Transcribe audio to text
        println!("Transcribing audio...");
        let transcription = self.openai_service.transcribe_audio(audio_path).await?;
        println!("Transcription: {}", transcription);

        // Step 2: Generate response using GPT
        println!("Generating response...");
        let response = self
            .openai_service
            .generate_response(&transcription)
            .await?;
        println!("Response: {}", response);

        // Step 3: Save response to file
        let mut file = File::create(output_path).await?;
        file.write_all(response.as_bytes()).await?;
        println!("Response saved to {}", output_path);

        Ok(())
    }
}

// FFI functions for Flutter integration

#[no_mangle]
pub extern "C" fn process_audio(
    api_key_ptr: *const c_char,
    audio_path_ptr: *const c_char,
    output_path_ptr: *const c_char,
) -> *mut c_char {
    // Safety checks
    if api_key_ptr.is_null() || audio_path_ptr.is_null() || output_path_ptr.is_null() {
        return CString::new("Error: Null pointer provided")
            .unwrap()
            .into_raw();
    }

    // Convert C strings to Rust strings
    let api_key = unsafe {
        CStr::from_ptr(api_key_ptr)
            .to_str()
            .unwrap_or("")
            .to_owned()
    };
    let audio_path = unsafe {
        CStr::from_ptr(audio_path_ptr)
            .to_str()
            .unwrap_or("")
            .to_owned()
    };
    let output_path = unsafe {
        CStr::from_ptr(output_path_ptr)
            .to_str()
            .unwrap_or("")
            .to_owned()
    };

    if api_key.is_empty() || audio_path.is_empty() || output_path.is_empty() {
        return CString::new("Error: Empty string provided")
            .unwrap()
            .into_raw();
    }

    // Create a new runtime for async operations
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            return CString::new(format!("Error creating runtime: {}", e))
                .unwrap()
                .into_raw()
        }
    };

    // Process the audio
    let assistant = VoiceAssistantService::new(api_key);
    let result = runtime.block_on(async {
        match assistant.process_audio(&audio_path, &output_path).await {
            Ok(()) => "Success".to_string(), // Convert &str to String
            Err(e) => format!("Error: {}", e),
        }
    });

    // Return the result
    CString::new(result).unwrap().into_raw()
}

// Free memory allocated by Rust
#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

// Simple test function to verify FFI is working
#[no_mangle]
pub extern "C" fn hello_rust() -> *mut c_char {
    CString::new("Hello from Rust!").unwrap().into_raw()
}
