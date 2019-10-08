use std::process::Stdio;
use std::sync::Arc;

use tempfile::{Builder, TempPath};
use tokio::net::process::Command;

/// Extract frames from the given video file.
///
/// Currently only the first frame is extracted.
/// The temporary file the frame is written to is returned.
///
/// This operation is expensive.
// TODO: run ffmpeg command asynchronously through tokio
// TODO: make sure user has ffmpeg installed
#[cfg(feature = "ffmpeg")]
pub async fn extract_frames(path: Arc<TempPath>) -> Result<Arc<TempPath>, ()> {
    let input = path.to_str().expect("failed to get path string");

    // Create new temporary file
    let name = input.split('_').last().unwrap_or("");
    let (_, frame_path) = Builder::new()
        .suffix(&format!("{}_frame.jpg", name))
        .tempfile()
        .expect("failed to create file for video frame")
        .into_parts();

    let output = frame_path.to_str().expect("failed to get thumb path string");

    // Run command to extract video frame
    println!("Extracting video frame to '{}'...", output);
    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(input)
        .arg("-vf")
        .arg("select=eq(n\\,0)")
        .arg("-q:v")
        .arg("3")
        .arg("-y")
        .arg(output)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;

    // Make sure the command succeeded
    match status {
        Err(err) => {
            println!("Failed to extract video frame, command failed, ignoring: {}", err);
            return Err(());
        },
        Ok(status) if !status.success() => {
            let code = status.code().map(|c| c.to_string()).unwrap_or_else(|| "?".into());
            println!("Failed to extract video frame, command had non-zero exit code, ignoring: {}", code);
            return Err(());
        },
        Ok(_) => {},
    }

    Ok(Arc::new(frame_path))
}
