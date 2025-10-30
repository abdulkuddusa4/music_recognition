use std::path::Path;
use cot::http::method::Method;
use cot::request::{Request, RequestExt};
use cot::request::extractors::RequestDb;
use cot::json::Json;
use serde_json::{json, Value};
use http_body_util::BodyExt;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn test_view(
    request: Request,
    RequestDb(db): RequestDb
) -> Json<Value> {
    
    println!("REQUEST PROCESSING");
    
    if request.method() == Method::POST {
        // Get the Content-Type header and clone the boundary
        let boundary = request
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .and_then(|content_type| {
                content_type
                    .split("boundary=")
                    .nth(1)
                    .map(|s| s.trim().to_string())  // Convert to owned String
            });
        
        let boundary = match boundary {
            Some(b) => b,
            None => {
                return Json(json!({
                    "status": "error",
                    "message": "No boundary found in Content-Type header"
                }));
            }
        };
        
        println!("Boundary: {}", boundary);
        
        // Now we can safely move request
        let (parts, body) = request.into_parts();
        let body_bytes = body.collect().await.unwrap().to_bytes();
        
        println!("Body size: {} bytes", body_bytes.len());
        
        // Create a stream for multer
        let stream = futures_util::stream::once(async move {
            Ok::<_, std::io::Error>(body_bytes)
        });
        
        // Parse with multer (boundary is now owned, so no borrow issue)
        let mut multipart = multer::Multipart::new(stream, boundary);
        
        let mut uploaded_files = Vec::new();
        
        // Process each field
        while let Some(mut field) = multipart.next_field().await.unwrap() {
            let field_name = field.name().unwrap_or("unknown").to_string();
            let file_name = field.file_name().map(|s| s.to_string());

            if field_name != "audio_file"{
                continue;
            }
            if let Some(original_filename) = file_name {                
                let data = field.bytes().await.unwrap();
                
                let safe_filename = sanitize_filename(&original_filename);
                
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let file_path = format!("uploads/{}_{}", timestamp, safe_filename);
                tokio::fs::create_dir_all("uploads").await.unwrap();
                
                match save_file(&file_path, &data).await {
                    Ok(_) => {
                        uploaded_files.push(file_path);
                    }
                    Err(e) => {
                    }
                }
            } else {
                let value = field.text().await.unwrap();
            }
        }
        
        return Json(json!({
            "status": "success",
            "message": format!("Uploaded {} file(s)", uploaded_files.len()),
            "files": uploaded_files,
            "count": uploaded_files.len()
        }));
    }
    
    Json(json!({
        "status": "error",
        "message": "Not a POST request"
    }))
}

// Helper function to save file
async fn save_file(path: &str, data: &[u8]) -> Result<(), std::io::Error> {
    let mut file = File::create(path).await?;
    file.write_all(data).await?;
    file.flush().await?;
    Ok(())
}

// Sanitize filename to prevent path traversal attacks
fn sanitize_filename(filename: &str) -> String {
    let name = Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unnamed");
    
    // Remove dangerous characters but keep dots for extensions
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}