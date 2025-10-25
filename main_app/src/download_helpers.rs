use std::path::Path;
use tokio::process::Command;

pub async fn download_youtube_audio(
    youtube_url: &str,
    output_path: &str,
) -> Result<(), String> {
    
    
    // Ensure output directory exists
    if let Some(parent) = Path::new(output_path).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }
    
    println!("🎵 Downloading audio from: {}", youtube_url);
    println!("📁 Output path: {}", output_path);
    let output = Command::new("yt-dlp")
        .arg("--extract-audio")           // Extract audio only
        .arg("--audio-format")
        .arg("mp3")                       // Convert to mp3
        .arg("--audio-quality")
        .arg("0")                         // Best quality (0-9, 0 is best)
        .arg("--output")
        .arg(output_path)                 // Output file path
        .arg("--no-playlist")             // Don't download playlists
        .arg("--no-warnings")             // Suppress warnings
        .arg("--progress")                // Show progress
        .arg(youtube_url)                 // YouTube URL
        .output()
        .await
        .map_err(|e| format!("Failed to execute yt-dlp: {}. Is yt-dlp installed?", e))?;
    
    if output.status.success() {
        println!("✅ Download complete: {}", output_path);
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        println!("ERROR DOWNLOADING {}", error);
        Err(format!("yt-dlp failed: {}", error))
    }
}