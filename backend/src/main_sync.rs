use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::fs;
use std::env;
use serde_json;
use uuid::Uuid;

fn main() {
    eprintln!("Starting Care Voice backend (sync version)...");
    
    let listener = match TcpListener::bind("0.0.0.0:8000") {
        Ok(l) => {
            eprintln!("Successfully bound to 0.0.0.0:8000");
            l
        },
        Err(e) => {
            eprintln!("Failed to bind: {}", e);
            std::process::exit(1);
        }
    };

    eprintln!("Backend running on http://0.0.0.0:8000");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 8192];
    let bytes_read = stream.read(&mut buffer).unwrap_or(0);
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    
    if request.starts_with("GET /health") {
        handle_health(stream);
    } else if request.starts_with("POST /upload") {
        handle_upload(stream, &request);
    } else if request.starts_with("OPTIONS") {
        handle_cors_preflight(stream);
    } else {
        handle_not_found(stream);
    }
}

fn handle_health(mut stream: TcpStream) {
    let response = r#"HTTP/1.1 200 OK
Content-Type: application/json
Access-Control-Allow-Origin: *

{"status":"ok","message":"Care Voice backend is running"}"#;

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_cors_preflight(mut stream: TcpStream) {
    let response = r#"HTTP/1.1 200 OK
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, OPTIONS
Access-Control-Allow-Headers: Content-Type

"#;

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_upload(mut stream: TcpStream, request: &str) {
    // Extract content length
    let content_length = extract_content_length(request).unwrap_or(0);
    
    if content_length == 0 {
        send_error(&mut stream, "No content length specified");
        return;
    }

    // Read the body data
    let mut body_buffer = vec![0u8; content_length];
    let mut total_read = 0;
    
    // Find where headers end
    if let Some(body_start) = request.find("\r\n\r\n") {
        let header_end = body_start + 4;
        let header_bytes = header_end.min(request.len());
        
        // Copy any body data that was already read
        let already_read = request.len() - header_bytes;
        if already_read > 0 {
            let start_idx = header_bytes;
            let copy_len = already_read.min(content_length);
            body_buffer[..copy_len].copy_from_slice(&request.as_bytes()[start_idx..start_idx + copy_len]);
            total_read = copy_len;
        }
    }

    // Read remaining body data
    while total_read < content_length {
        match stream.read(&mut body_buffer[total_read..]) {
            Ok(0) => break,
            Ok(n) => total_read += n,
            Err(_) => {
                send_error(&mut stream, "Failed to read request body");
                return;
            }
        }
    }

    // Process multipart data (simplified)
    if let Some(audio_data) = extract_audio_from_multipart(&body_buffer) {
        match process_audio(&audio_data) {
            Ok((transcript, summary)) => {
                let response_json = serde_json::json!({
                    "full_transcript": transcript,
                    "summary": summary
                });
                
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                    response_json
                );
                
                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            Err(e) => {
                eprintln!("Audio processing error: {}", e);
                send_error(&mut stream, "Failed to process audio");
            }
        }
    } else {
        send_error(&mut stream, "No audio data found");
    }
}

fn handle_not_found(mut stream: TcpStream) {
    let response = r#"HTTP/1.1 404 Not Found
Content-Type: application/json
Access-Control-Allow-Origin: *

{"error":"Not found"}"#;

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn send_error(stream: &mut TcpStream, message: &str) {
    let error_json = serde_json::json!({"error": message});
    let response = format!(
        "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
        error_json
    );
    
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn extract_content_length(request: &str) -> Option<usize> {
    for line in request.lines() {
        if line.to_lowercase().starts_with("content-length:") {
            if let Some(length_str) = line.split(':').nth(1) {
                return length_str.trim().parse().ok();
            }
        }
    }
    None
}

fn extract_audio_from_multipart(data: &[u8]) -> Option<Vec<u8>> {
    // Simple multipart parser - look for audio/webm content
    let data_str = String::from_utf8_lossy(data);
    
    // Find boundary
    if let Some(boundary_start) = data_str.find("boundary=") {
        let boundary_line = &data_str[boundary_start + 9..];
        if let Some(boundary_end) = boundary_line.find('\r') {
            let boundary = &boundary_line[..boundary_end];
            
            // Look for audio data after Content-Type: audio/webm
            if let Some(audio_start) = data_str.find("Content-Type: audio/webm") {
                if let Some(data_start) = data_str[audio_start..].find("\r\n\r\n") {
                    let actual_start = audio_start + data_start + 4;
                    
                    // Find end boundary
                    if let Some(end_pos) = data_str[actual_start..].find(&format!("--{}", boundary)) {
                        let audio_end = actual_start + end_pos;
                        return Some(data[actual_start..audio_end].to_vec());
                    }
                }
            }
        }
    }
    
    None
}

fn process_audio(audio_data: &[u8]) -> Result<(String, String), Box<dyn std::error::Error>> {
    let api_key = env::var("GEMINI_API_KEY")
        .map_err(|_| "GEMINI_API_KEY environment variable not set")?;
    
    // Save audio to temp file
    let file_id = Uuid::new_v4().to_string();
    let temp_file_path = format!("/tmp/{}.webm", file_id);
    fs::write(&temp_file_path, audio_data)?;
    
    // Call Gemini API (synchronous version)
    let result = call_gemini_sync(&api_key, &temp_file_path);
    
    // Clean up temp file
    let _ = fs::remove_file(&temp_file_path);
    
    result
}

fn call_gemini_sync(api_key: &str, audio_path: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    use base64::Engine;
    
    let audio_data = fs::read(audio_path)?;
    let base64_audio = base64::engine::general_purpose::STANDARD.encode(audio_data);
    
    // Use blocking reqwest client
    let client = reqwest::blocking::Client::new();
    
    let transcript_payload = serde_json::json!({
        "contents": [{
            "parts": [
                {
                    "text": "請將這個音頻檔案轉錄為中文文字。請提供完整的逐字稿，包含所有對話內容。"
                },
                {
                    "inline_data": {
                        "mime_type": "audio/webm",
                        "data": base64_audio
                    }
                }
            ]
        }]
    });
    
    let transcript_response = client
        .post(&format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}", api_key))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&transcript_payload)
        .send()?;
    
    let transcript_result: serde_json::Value = transcript_response.json()?;
    let full_transcript = transcript_result["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("轉錄失敗")
        .to_string();
    
    let summary_payload = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": format!("請根據以下逐字稿內容，提供一個關懷重點摘要。請特別關注對話中的情緒表達、需求、困難或重要訊息：\n\n{}", full_transcript)
            }]
        }]
    });
    
    let summary_response = client
        .post(&format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}", api_key))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&summary_payload)
        .send()?;
    
    let summary_result: serde_json::Value = summary_response.json()?;
    let summary = summary_result["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("摘要生成失敗")
        .to_string();
    
    Ok((full_transcript, summary))
}