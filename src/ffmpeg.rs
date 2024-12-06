use crate::KRec;
use color_eyre::{eyre::eyre, Result};
use std::path::Path;
use tempfile::NamedTempFile;
use thiserror::Error;
use tracing::{debug, info, instrument, warn};

#[derive(Error, Debug)]
pub enum FFmpegError {
    #[error("FFmpeg error: {0}")]
    FFmpeg(String),
    #[error("Input file not found: {0}")]
    InputNotFound(String),
}

#[instrument(skip(video_path, krec_path, output_path))]
pub fn combine_with_video(
    video_path: impl AsRef<Path>,
    krec_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<()> {
    info!("Combining video with KRec data");
    debug!(
        "Video: {}, KRec: {}, Output: {}",
        video_path.as_ref().display(),
        krec_path.as_ref().display(),
        output_path.as_ref().display()
    );

    // Read the KRec file to get UUID and task
    let krec = KRec::load(
        krec_path
            .as_ref()
            .to_str()
            .ok_or_else(|| eyre!("Invalid KRec path: {}", krec_path.as_ref().display()))?,
    )?;

    if krec.header.uuid.is_empty() {
        return Err(eyre!("KRec file missing UUID"));
    }

    if krec.header.task.is_empty() {
        return Err(eyre!("KRec file missing task"));
    }

    if krec.header.robot_platform.is_empty() {
        return Err(eyre!("KRec file missing robot platform"));
    }

    if krec.header.robot_serial.is_empty() {
        return Err(eyre!("KRec file missing robot serial"));
    }

    let status = std::process::Command::new("ffmpeg")
        .args([
            "-i",
            &video_path.as_ref().to_string_lossy(),
            "-attach",
            &krec_path.as_ref().to_string_lossy(),
            "-metadata:s:t",
            "mimetype=application/octet-stream",
            "-metadata:s:t",
            &format!("uuid={}", krec.header.uuid),
            "-metadata:s:t",
            &format!("task={}", krec.header.task),
            "-metadata:s:t",
            &format!("robot_platform={}", krec.header.robot_platform),
            "-metadata:s:t",
            &format!("robot_serial={}", krec.header.robot_serial),
            "-c",
            "copy",
            &output_path.as_ref().to_string_lossy(),
        ])
        .status()
        .map_err(|e| eyre!("Failed to execute ffmpeg: {}", e))?;

    if status.success() {
        info!("Successfully combined video with KRec data");
        Ok(())
    } else {
        let err = eyre!("FFmpeg command failed with status: {}", status);
        warn!("{}", err);
        Err(err)
    }
}

pub fn extract_from_video(video_path: &str, verbose: Option<bool>) -> Result<KRec, FFmpegError> {
    info!("Starting extract_from_video");

    // Check if input file exists
    if !Path::new(video_path).exists() {
        return Err(FFmpegError::InputNotFound(video_path.to_string()));
    }

    // Create a temporary file for FFmpeg output
    let temp_file = NamedTempFile::new()
        .map_err(|e| FFmpegError::FFmpeg(format!("Failed to create temporary file: {}", e)))?;
    let temp_path = temp_file.path().to_string_lossy().to_string();

    // Construct ffmpeg command
    let mut command = std::process::Command::new("ffmpeg");
    command.args([
        "-y",
        "-dump_attachment:t:0",
        &temp_path,
        "-i",
        video_path,
        "-f",
        "null",
        "/dev/null",
    ]);

    // Control ffmpeg output based on verbose flag
    if !verbose.unwrap_or(false) {
        command
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
    }

    debug!("Constructed ffmpeg command: {:?}", command);

    let status = command.status().map_err(|e| {
        warn!("Failed to execute ffmpeg: {}", e);
        FFmpegError::FFmpeg(e.to_string())
    })?;

    if !status.success() {
        let error_msg = format!("FFmpeg command failed with status: {}", status);
        warn!("{}", error_msg);
        return Err(FFmpegError::FFmpeg(error_msg));
    }

    // Load the KRec from the temporary file
    let krec = KRec::load(&temp_path).map_err(|e| {
        FFmpegError::FFmpeg(format!("Failed to load KRec from temporary file: {}", e))
    })?;

    // The temporary file will be automatically deleted when temp_file goes out of scope
    info!("Successfully extracted KRec from video");
    Ok(krec)
}
