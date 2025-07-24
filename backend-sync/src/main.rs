use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::fs;
use std::env;
use std::panic;
use serde_json;
use uuid::Uuid;

#[derive(Debug)]
struct AudioData {
    data: Vec<u8>,
    mime_type: String,
}

fn main() {
    println!("Starting Care Voice backend (sync version)...");
    eprintln!("Starting Care Voice backend (sync version)...");
    
    let listener = match TcpListener::bind("0.0.0.0:8000") {
        Ok(l) => {
            println!("Successfully bound to 0.0.0.0:8000");
            eprintln!("Successfully bound to 0.0.0.0:8000");
            l
        },
        Err(e) => {
            println!("Failed to bind: {}", e);
            eprintln!("Failed to bind: {}", e);
            std::process::exit(1);
        }
    };

    println!("Backend running on http://0.0.0.0:8000");
    eprintln!("Backend running on http://0.0.0.0:8000");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                eprintln!("Error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 8192];
    let bytes_read = stream.read(&mut buffer).unwrap_or(0);
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    
    println!("Received request: {}", &request[..request.find('\n').unwrap_or(50).min(50)]);
    
    if request.starts_with("GET /health") {
        handle_health(stream);
    } else if request.starts_with("GET /test-api") {
        handle_test_api(stream);
    } else if request.starts_with("POST /upload") {
        handle_upload(stream, &request);
    } else if request.starts_with("OPTIONS") {
        handle_cors_preflight(stream);
    } else {
        handle_not_found(stream);
    }
}

fn handle_health(mut stream: TcpStream) {
    println!("Handling health check");
    let response_body = r#"{"status":"ok","message":"Care Voice backend is running"}"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        response_body.len(),
        response_body
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_test_api(mut stream: TcpStream) {
    println!("Testing API key and connection");
    
    let api_key = match env::var("GEMINI_API_KEY") {
        Ok(key) => {
            if key.is_empty() {
                send_test_error(&mut stream, "GEMINI_API_KEY 環境變數為空");
                return;
            }
            key
        }
        Err(_) => {
            send_test_error(&mut stream, "GEMINI_API_KEY 環境變數未設定");
            return;
        }
    };
    
    println!("API Key found, length: {} characters", api_key.len());
    
    // Test API with a simple request
    match test_gemini_api(&api_key) {
        Ok(result) => {
            let response_json = serde_json::json!({
                "status": "success",
                "api_key_length": api_key.len(),
                "test_result": result
            });
            
            let response_body = response_json.to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        Err(e) => {
            send_test_error(&mut stream, &format!("API 測試失敗: {}", e));
        }
    }
}

fn send_test_error(stream: &mut TcpStream, message: &str) {
    let error_json = serde_json::json!({
        "status": "error",
        "message": message
    });
    let response_body = error_json.to_string();
    let response = format!(
        "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        response_body.len(),
        response_body
    );
    
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn test_gemini_api(api_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    
    let test_payload = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": "Hello, please respond with 'API test successful' in Traditional Chinese."
            }]
        }]
    });
    
    println!("Sending test request to Gemini API");
    
    let response = client
        .post(&format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}", api_key))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&test_payload)
        .send()?;
    
    let status = response.status();
    println!("Test API response status: {}", status);
    
    // Check for rate limit headers
    if let Some(remaining) = response.headers().get("x-ratelimit-remaining") {
        println!("Rate limit remaining: {:?}", remaining);
    }
    
    if !status.is_success() {
        let error_text = response.text()?;
        println!("Test API error response: {}", error_text);
        return Err(format!("HTTP {}: {}", status, error_text).into());
    }
    
    let result: serde_json::Value = response.json()?;
    let test_response = result["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("無回應")
        .to_string();
    
    println!("Test API successful, response: {}", test_response);
    Ok(test_response)
}

fn handle_cors_preflight(mut stream: TcpStream) {
    println!("Handling CORS preflight");
    let response = r#"HTTP/1.1 200 OK
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, OPTIONS
Access-Control-Allow-Headers: Content-Type
Content-Length: 0

"#;

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_upload(mut stream: TcpStream, request: &str) {
    println!("Handling upload request");
    
    // Extract content length and content type
    let content_length = extract_content_length(request).unwrap_or(0);
    let content_type = extract_content_type(request);
    
    println!("Content-Length: {}", content_length);
    println!("Content-Type: {:?}", content_type);
    
    if content_length == 0 {
        send_error(&mut stream, "No content length specified");
        return;
    }

    // Extract boundary from Content-Type header if present
    let boundary_from_header = content_type
        .as_ref()
        .and_then(|ct| extract_boundary_from_content_type(ct));
    
    if let Some(ref boundary) = boundary_from_header {
        println!("Boundary from header: '{}'", boundary);
    }

    // Read the full request including headers and body
    let mut full_request = request.as_bytes().to_vec();
    let initial_read = full_request.len();
    
    // Calculate how much more we need to read
    let headers_end = request.find("\r\n\r\n").unwrap_or(0) + 4;
    let body_already_read = initial_read.saturating_sub(headers_end);
    let remaining_to_read = content_length.saturating_sub(body_already_read);
    
    println!("Headers end at: {}, body already read: {}, remaining: {}", 
             headers_end, body_already_read, remaining_to_read);

    // Read remaining body data
    if remaining_to_read > 0 {
        let mut remaining_buffer = vec![0u8; remaining_to_read];
        let mut total_read = 0;
        
        while total_read < remaining_to_read {
            match stream.read(&mut remaining_buffer[total_read..]) {
                Ok(0) => break,
                Ok(n) => total_read += n,
                Err(e) => {
                    println!("Error reading request body: {}", e);
                    send_error(&mut stream, "Failed to read request body");
                    return;
                }
            }
        }
        
        full_request.extend_from_slice(&remaining_buffer[..total_read]);
        println!("Total request size: {} bytes", full_request.len());
    }

    // Extract just the body part for multipart parsing
    let body_start = headers_end;
    let body_data = &full_request[body_start..];
    
    println!("Processing body data: {} bytes", body_data.len());

    // Process multipart data with error handling
    match std::panic::catch_unwind(|| {
        extract_audio_with_mime_type(body_data, boundary_from_header)
    }) {
        Ok(Some(audio_info)) => {
            println!("Successfully extracted audio data: {} bytes, MIME: {}", 
                     audio_info.data.len(), audio_info.mime_type);
            match process_audio_with_mime(&audio_info.data, &audio_info.mime_type) {
                Ok((transcript, summary)) => {
                    let response_json = serde_json::json!({
                        "full_transcript": transcript,
                        "summary": summary
                    });
                    
                    let response_body = response_json.to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                        response_body.len(),
                        response_body
                    );
                    
                    stream.write(response.as_bytes()).unwrap();
                    stream.flush().unwrap();
                }
                Err(e) => {
                    println!("Audio processing error: {}", e);
                    send_error(&mut stream, "Failed to process audio");
                }
            }
        }
        Ok(None) => {
            println!("No audio data found in multipart");
            send_error(&mut stream, "No audio data found");
        }
        Err(panic_info) => {
            println!("Panic occurred while parsing multipart data: {:?}", panic_info);
            send_error(&mut stream, "Internal error processing upload");
        }
    }
}

fn handle_not_found(mut stream: TcpStream) {
    let response = r#"HTTP/1.1 404 Not Found
Content-Type: application/json
Access-Control-Allow-Origin: *
Content-Length: 20

{"error":"Not found"}"#;

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn send_error(stream: &mut TcpStream, message: &str) {
    let error_json = serde_json::json!({"error": message});
    let response_body = error_json.to_string();
    let response = format!(
        "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        response_body.len(),
        response_body
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

fn extract_content_type(request: &str) -> Option<String> {
    for line in request.lines() {
        if line.to_lowercase().starts_with("content-type:") {
            if let Some(type_str) = line.split(':').nth(1) {
                return Some(type_str.trim().to_string());
            }
        }
    }
    None
}

fn extract_boundary_from_content_type(content_type: &str) -> Option<String> {
    // Look for boundary= in Content-Type header
    if let Some(boundary_start) = content_type.find("boundary=") {
        let boundary_part = &content_type[boundary_start + 9..];
        
        // Handle quoted and unquoted boundaries
        let boundary = if boundary_part.starts_with('"') {
            // Quoted boundary
            boundary_part.split('"').nth(1)?
        } else {
            // Unquoted boundary - take until semicolon or end
            boundary_part.split(';').next()?.trim()
        };
        
        if !boundary.is_empty() {
            return Some(boundary.to_string());
        }
    }
    None
}

fn extract_audio_from_multipart(data: &[u8]) -> Option<Vec<u8>> {
    println!("Parsing multipart data, total size: {} bytes", data.len());
    let data_str = String::from_utf8_lossy(data);
    
    // Debug: print first 500 chars of the request
    let preview = &data_str[..data_str.len().min(500)];
    println!("Request preview: {}", preview);
    
    // Try multiple boundary detection methods
    let boundary = if let Some(boundary) = extract_boundary_from_data(&data_str) {
        println!("Found boundary: '{}'", boundary);
        boundary
    } else {
        println!("No boundary found in multipart data");
        return None;
    };
    
    // Look for audio content with various MIME types
    let audio_patterns = [
        "Content-Type: audio/webm",
        "Content-Type: audio/wav", 
        "Content-Type: audio/mpeg",
        "Content-Type: audio/mp4",
        "content-type: audio/webm",  // lowercase variant
    ];
    
    for pattern in &audio_patterns {
        println!("Searching for pattern: {}", pattern);
        if let Some(audio_start) = data_str.find(pattern) {
            println!("Found audio content type at position: {}", audio_start);
            
            // Find the start of actual data (after headers)
            if let Some(data_start) = data_str[audio_start..].find("\r\n\r\n") {
                let actual_start = audio_start + data_start + 4;
                println!("Audio data starts at position: {}", actual_start);
                
                // Find end boundary - try both with and without leading --
                let end_patterns = [
                    format!("\r\n--{}", boundary),
                    format!("--{}", boundary),
                    format!("\n--{}", boundary),
                ];
                
                for end_pattern in &end_patterns {
                    if let Some(end_pos) = data_str[actual_start..].find(end_pattern) {
                        let audio_end = actual_start + end_pos;
                        let audio_size = audio_end - actual_start;
                        println!("Found end boundary, audio size: {} bytes", audio_size);
                        
                        if audio_size > 0 {
                            return Some(data[actual_start..audio_end].to_vec());
                        }
                    }
                }
                
                // If no end boundary found, try to use remaining data
                println!("No end boundary found, using remaining data");
                let remaining_size = data.len() - actual_start;
                if remaining_size > 100 { // Minimum reasonable audio size
                    println!("Using remaining {} bytes as audio data", remaining_size);
                    return Some(data[actual_start..].to_vec());
                }
            }
        }
    }
    
    println!("No audio data found in multipart");
    None
}

fn extract_boundary_from_data(data_str: &str) -> Option<String> {
    // Try multiple boundary extraction methods
    
    // Method 1: Look for boundary= in the data
    if let Some(boundary_start) = data_str.find("boundary=") {
        let boundary_line = &data_str[boundary_start + 9..];
        
        // Try different terminators
        for terminator in &["\r\n", "\n", "\r", " ", ";"] {
            if let Some(boundary_end) = boundary_line.find(terminator) {
                let boundary = boundary_line[..boundary_end].trim_matches('"');
                if !boundary.is_empty() {
                    return Some(boundary.to_string());
                }
            }
        }
        
        // If no terminator found, take the rest of the line
        let boundary = boundary_line.split_whitespace().next()?.trim_matches('"');
        if !boundary.is_empty() {
            return Some(boundary.to_string());
        }
    }
    
    // Method 2: Look for --boundary pattern at the start
    for line in data_str.lines().take(10) { // Check first 10 lines
        if line.starts_with("--") && line.len() > 10 {
            let potential_boundary = &line[2..];
            if potential_boundary.chars().all(|c| c.is_alphanumeric() || "-_".contains(c)) {
                return Some(potential_boundary.to_string());
            }
        }
    }
    
    None
}

fn find_safe_preview_length(data_str: &str, max_len: usize) -> usize {
    // Find a safe char boundary before max_len
    let mut len = data_str.len().min(max_len);
    
    // Look for the start of binary data (after \r\n\r\n)
    if let Some(binary_start) = data_str.find("\r\n\r\n") {
        // Only show headers, not binary data
        len = len.min(binary_start + 4);
    }
    
    // Make sure we don't cut in the middle of a UTF-8 character
    while len > 0 && !data_str.is_char_boundary(len) {
        len -= 1;
    }
    
    len
}

fn extract_audio_with_mime_type(data: &[u8], boundary_hint: Option<String>) -> Option<AudioData> {
    println!("Parsing multipart data with boundary hint: {:?}, total size: {} bytes", boundary_hint, data.len());
    let data_str = String::from_utf8_lossy(data);
    
    // Debug: safely print first part of the request (headers only)
    let safe_preview_len = find_safe_preview_length(&data_str, 500);
    let preview = &data_str[..safe_preview_len];
    println!("Request body preview (first {} chars):\n{}", safe_preview_len, preview);
    
    // Try to use boundary hint first, then fallback to detection
    let boundary = boundary_hint
        .or_else(|| extract_boundary_from_data(&data_str))
        .or_else(|| {
            // Last resort: look for common boundary patterns
            for line in data_str.lines().take(5) {
                if line.starts_with("--") && line.len() > 10 {
                    return Some(line[2..].to_string());
                }
            }
            None
        });
    
    let boundary = match boundary {
        Some(b) => {
            println!("Using boundary: '{}'", b);
            b
        }
        None => {
            println!("No boundary found, cannot parse multipart data");
            return None;
        }
    };
    
    // Look for audio content with various patterns
    let audio_patterns = [
        "Content-Type: audio/wav",     // WAV - most reliable
        "Content-Type: audio/mpeg",    // MP3/MPEG
        "Content-Type: audio/mp4",     // MP4 audio
        "Content-Type: audio/mp3",     // MP3
        "Content-Type: audio/aac",     // AAC  
        "Content-Type: audio/ogg",     // OGG
        "Content-Type: audio/flac",    // FLAC
        "Content-Type: audio/webm",    // WebM
        "content-type: audio/",        // Lowercase variants
        "name=\"audio\"",              // Form field name
    ];
    
    // Find the audio part
    for pattern in &audio_patterns {
        println!("Searching for pattern: '{}'", pattern);
        if let Some(pattern_pos) = data_str.find(pattern) {
            println!("Found pattern '{}' at position: {}", pattern, pattern_pos);
            
            // Look backward to find the start of this part (boundary)
            let part_start = data_str[..pattern_pos].rfind(&format!("--{}", boundary))
                .unwrap_or(0);
            println!("Part starts at position: {}", part_start);
            
            // Find the start of actual data (after all headers in this part)
            if let Some(data_start_offset) = data_str[pattern_pos..].find("\r\n\r\n") {
                let data_start = pattern_pos + data_start_offset + 4;
                println!("Audio data starts at position: {}", data_start);
                
                // Find the end of this part (next boundary)
                let search_start = data_start;
                let end_boundary_patterns = [
                    format!("\r\n--{}", boundary),
                    format!("\n--{}", boundary),  
                    format!("--{}", boundary),
                ];
                
                let mut data_end = data.len();
                for end_pattern in &end_boundary_patterns {
                    if let Some(end_pos) = data_str[search_start..].find(end_pattern) {
                        data_end = search_start + end_pos;
                        println!("Found end boundary at position: {}", data_end);
                        break;
                    }
                }
                
                let audio_size = data_end - data_start;
                println!("Audio data size: {} bytes", audio_size);
                
                if audio_size > 44 { // Minimum for a small audio file (44 bytes for WAV header)
                    println!("Extracting audio data: {} bytes", audio_size);
                    
                    // Extract MIME type from the pattern match
                    let mime_type = if pattern.contains("Content-Type:") {
                        // Extract MIME type from Content-Type header
                        let detected_type = pattern.split(":").nth(1).unwrap_or("audio/webm").trim();
                        // Map to Gemini-compatible formats  
                        if detected_type.contains("audio/webm") {
                            "audio/webm".to_string()
                        } else if detected_type == "audio/ogg" {
                            "audio/ogg".to_string()
                        } else {
                            detected_type.to_string()
                        }
                    } else {
                        // Default fallback if we found by field name
                        "audio/webm".to_string()
                    };
                    
                    println!("Detected MIME type: {}", mime_type);
                    
                    return Some(AudioData {
                        data: data[data_start..data_end].to_vec(),
                        mime_type,
                    });
                } else {
                    println!("Audio data too small: {} bytes", audio_size);
                }
            }
        }
    }
    
    println!("No valid audio data found in multipart");
    None
}

fn process_audio_with_mime(audio_data: &[u8], mime_type: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let api_key = env::var("GEMINI_API_KEY")
        .map_err(|_| "GEMINI_API_KEY environment variable not set")?;
    
    println!("Processing audio with Gemini API, MIME type: {}", mime_type);
    
    // Validate audio file size
    if audio_data.len() < 1000 {
        return Err("音頻檔案太小（少於1KB），請錄製較長的音頻（建議至少3-5秒）。".into());
    }
    
    // Try Files API first for larger files (>100KB) for better reliability
    if audio_data.len() > 100 * 1024 {  // > 100KB, use Files API
        println!("File size {} bytes > 100KB, trying Files API first...", audio_data.len());
    } else {
        println!("File size {} bytes <= 100KB, using inline data approach...", audio_data.len());
    }
    
    if audio_data.len() > 20 * 1024 * 1024 {  // 20MB limit
        return Err("音頻檔案過大（超過20MB），請使用較小的音頻檔案。".into());
    }
    
    // Save audio to temp file with appropriate extension
    let file_id = Uuid::new_v4().to_string();
    let file_extension = match mime_type {
        "audio/wav" => "wav",
        "audio/mp3" | "audio/mpeg" => "mp3",
        "audio/mp4" => "mp4",
        "audio/aac" => "aac",
        "audio/ogg" => "ogg",
        "audio/flac" => "flac",
        "audio/webm" => "webm",
        _ => "webm", // fallback
    };
    let temp_file_path = format!("/tmp/{}.{}", file_id, file_extension);
    fs::write(&temp_file_path, audio_data)?;
    
    // Call Gemini API (synchronous version)
    // Try Files API first for larger files, fallback to inline data
    if audio_data.len() > 100 * 1024 {  // > 100KB, try Files API first
        match call_gemini_files_api(&api_key, &temp_file_path, mime_type) {
            Ok(result) => {
                let _ = fs::remove_file(&temp_file_path);
                return Ok(result);
            }
            Err(e) => {
                println!("Files API failed ({}), falling back to inline data", e);
            }
        }
    }

    let result = call_gemini_sync_with_mime(&api_key, &temp_file_path, mime_type);
    
    // Clean up temp file
    let _ = fs::remove_file(&temp_file_path);
    
    result
}

fn process_audio(audio_data: &[u8]) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Fallback function for backward compatibility
    process_audio_with_mime(audio_data, "audio/webm")
}

fn call_gemini_sync_with_mime(api_key: &str, audio_path: &str, mime_type: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    use base64::Engine;
    
    let audio_data = fs::read(audio_path)?;
    let base64_audio = base64::engine::general_purpose::STANDARD.encode(&audio_data);
    
    println!("Calling Gemini API for transcription, audio size: {} bytes, MIME: {}", audio_data.len(), mime_type);
    
    // Use blocking reqwest client
    let client = reqwest::blocking::Client::new();
    
    let transcript_payload = serde_json::json!({
        "contents": [{
            "parts": [
                {
                    "text": "Generate a complete, detailed transcript of this audio."
                },
                {
                    "inlineData": {
                        "mimeType": mime_type,
                        "data": base64_audio
                    }
                }
            ]
        }]
    });
    
    // Log the payload size for debugging
    let payload_str = serde_json::to_string(&transcript_payload)?;
    println!("Payload size: {} bytes", payload_str.len());
    
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}", api_key);
    println!("Request URL: {}", url);
    println!("Request payload: {}", serde_json::to_string_pretty(&transcript_payload)?);
    
    let transcript_response = client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::USER_AGENT, "care-voice-backend/1.0")
        .json(&transcript_payload)
        .send()?;
    
    // Log response status and headers for debugging
    let status = transcript_response.status();
    println!("Gemini API response status: {}", status);
    
    // Check for rate limit headers
    if let Some(remaining) = transcript_response.headers().get("x-ratelimit-remaining") {
        println!("Rate limit remaining: {:?}", remaining);
    }
    if let Some(reset) = transcript_response.headers().get("x-ratelimit-reset") {
        println!("Rate limit reset: {:?}", reset);
    }
    
    // Check if response is successful
    if !status.is_success() {
        let error_text = transcript_response.text()?;
        println!("Gemini API error response: {}", error_text);
        
        // Try to parse error JSON for more details
        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
            if let Some(error_msg) = error_json["error"]["message"].as_str() {
                return Err(format!("Gemini API 錯誤: {}", error_msg).into());
            }
            if let Some(error_code) = error_json["error"]["code"].as_i64() {
                return Err(format!("Gemini API 錯誤代碼: {}, 狀態: {}", error_code, status).into());
            }
        }
        
        return Err(format!("Gemini API 請求失敗: HTTP {}", status).into());
    }
    
    let transcript_result: serde_json::Value = transcript_response.json()?;
    let full_transcript = transcript_result["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("轉錄失敗")
        .to_string();
    
    println!("Got transcript: {}", &full_transcript[..full_transcript.len().min(50)]);
    
    let summary_payload = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": format!("請根據以下逐字稿內容，提供一個關懷重點摘要。請特別關注對話中的情緒表達、需求、困難或重要訊息：\n\n{}", full_transcript)
            }]
        }]
    });
    
    let summary_response = client
        .post(&format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}", api_key))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&summary_payload)
        .send()?;
    
    // Log response status for summary request
    let summary_status = summary_response.status();
    println!("Gemini API summary response status: {}", summary_status);
    
    // Check for rate limit headers on summary request  
    if let Some(remaining) = summary_response.headers().get("x-ratelimit-remaining") {
        println!("Summary rate limit remaining: {:?}", remaining);
    }
    
    // Check if summary response is successful
    if !summary_status.is_success() {
        let error_text = summary_response.text()?;
        println!("Gemini API summary error response: {}", error_text);
        
        // Try to parse error JSON for more details
        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
            if let Some(error_msg) = error_json["error"]["message"].as_str() {
                return Err(format!("Gemini API 摘要錯誤: {}", error_msg).into());
            }
        }
        
        return Err(format!("Gemini API 摘要請求失敗: HTTP {}", summary_status).into());
    }
    
    let summary_result: serde_json::Value = summary_response.json()?;
    let summary = summary_result["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("摘要生成失敗")
        .to_string();
    
    println!("Generated summary");
    
    Ok((full_transcript, summary))
}

fn call_gemini_files_api(api_key: &str, audio_path: &str, mime_type: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    use base64::Engine;
    
    println!("Using Gemini Files API for audio processing");
    let client = reqwest::blocking::Client::new();
    
    // Read audio data from file
    let audio_data = fs::read(audio_path)?;
    
    // Step 1: Upload file to Gemini Files API using simple upload
    let upload_url = format!("https://generativelanguage.googleapis.com/upload/v1beta/files?uploadType=media&key={}", api_key);
    
    println!("Uploading {} bytes to Files API (simple upload)", audio_data.len());
    
    let upload_response = client
        .post(&upload_url)
        .header("Content-Type", mime_type)
        .header("X-Goog-Upload-Protocol", "raw")
        .body(audio_data)
        .send()?;
    
    let upload_status = upload_response.status();
    println!("Files API upload response status: {}", upload_status);
    
    if !upload_status.is_success() {
        let error_text = upload_response.text()?;
        println!("Files API upload error: {}", error_text);
        return Err(format!("Files API 上傳失敗: HTTP {}", upload_status).into());
    }
    
    let upload_result: serde_json::Value = upload_response.json()?;
    let file_uri = upload_result["file"]["uri"]
        .as_str()
        .ok_or("無法獲取上傳檔案的 URI")?;
    
    println!("File uploaded successfully, URI: {}", file_uri);
    
    // Step 2: Use uploaded file for transcription
    let transcript_payload = serde_json::json!({
        "contents": [{
            "parts": [
                {
                    "text": "Generate a complete, detailed transcript of this audio."
                },
                {
                    "fileData": {
                        "mimeType": mime_type,
                        "fileUri": file_uri
                    }
                }
            ]
        }]
    });
    
    let transcript_response = client
        .post(&format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}", api_key))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&transcript_payload)
        .send()?;
    
    let transcript_status = transcript_response.status();
    println!("Files API transcription response status: {}", transcript_status);
    
    if !transcript_status.is_success() {
        let error_text = transcript_response.text()?;
        println!("Files API transcription error: {}", error_text);
        return Err(format!("Files API 轉錄失敗: HTTP {}", transcript_status).into());
    }
    
    let transcript_result: serde_json::Value = transcript_response.json()?;
    let full_transcript = transcript_result["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("轉錄失敗")
        .to_string();
    
    println!("Files API transcript: {}", &full_transcript[..full_transcript.len().min(50)]);
    
    // Step 3: Generate summary
    let summary_payload = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": format!("請根據以下逐字稿內容，提供一個關懷重點摘要。請特別關注對話中的情緒表達、需求、困難或重要訊息：\n\n{}", full_transcript)
            }]
        }]
    });
    
    let summary_response = client
        .post(&format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}", api_key))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&summary_payload)
        .send()?;
    
    let summary_status = summary_response.status();
    println!("Files API summary response status: {}", summary_status);
    
    if !summary_status.is_success() {
        let error_text = summary_response.text()?;
        println!("Files API summary error: {}", error_text);
        return Err(format!("Files API 摘要失敗: HTTP {}", summary_status).into());
    }
    
    let summary_result: serde_json::Value = summary_response.json()?;
    let summary = summary_result["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("摘要生成失敗")
        .to_string();
    
    println!("Files API processing completed successfully");
    
    Ok((full_transcript, summary))
}

fn call_gemini_sync(api_key: &str, audio_path: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Fallback function for backward compatibility
    call_gemini_sync_with_mime(api_key, audio_path, "audio/webm")
}