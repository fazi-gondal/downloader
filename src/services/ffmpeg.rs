//! FFmpeg service for merge, remux, convert, and probe.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::core::{AppError, Result};

#[derive(Debug, Clone)]
pub struct FfmpegService {
    binary: PathBuf,
}

impl Default for FfmpegService {
    fn default() -> Self {
        Self {
            binary: PathBuf::from("ffmpeg"),
        }
    }
}

impl FfmpegService {
    pub fn new(binary: impl Into<PathBuf>) -> Self {
        Self {
            binary: binary.into(),
        }
    }

    pub fn binary(&self) -> &Path {
        &self.binary
    }

    /// Return ffmpeg version string or error if missing.
    pub fn check_available(&self) -> Result<String> {
        let output = Command::new(&self.binary)
            .arg("-version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| AppError::Ffmpeg(format!("ffmpeg not found: {e}")))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let first = stdout.lines().next().unwrap_or("ffmpeg").to_string();
            Ok(first)
        } else {
            Err(AppError::Ffmpeg("ffmpeg -version failed".into()))
        }
    }

    /// Lossless remux / container change when codecs are compatible.
    pub fn remux(&self, input: &Path, output: &Path) -> Result<()> {
        let status = Command::new(&self.binary)
            .args([
                "-y",
                "-i",
                input.to_str().unwrap_or(""),
                "-c",
                "copy",
                output.to_str().unwrap_or(""),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .status()
            .map_err(|e| AppError::Ffmpeg(e.to_string()))?;

        if status.success() {
            Ok(())
        } else {
            Err(AppError::Ffmpeg("remux failed".into()))
        }
    }

    /// Merge video file + one or more audio files into an MKV with dispositions.
    pub fn merge_multi_audio(
        &self,
        video: &Path,
        audio_paths: &[(PathBuf, String, String)],
        output: &Path,
    ) -> Result<()> {
        let mut cmd = Command::new(&self.binary);
        cmd.arg("-y").arg("-i").arg(video);

        for (path, _, _) in audio_paths {
            cmd.arg("-i").arg(path);
        }

        cmd.args(["-map", "0:v:0"]);
        for i in 0..audio_paths.len() {
            cmd.arg("-map").arg(format!("{}:a:0", i + 1));
        }

        cmd.args(["-c:v", "copy", "-c:a", "aac", "-b:a", "192k"]);

        for (i, (_, lang, title)) in audio_paths.iter().enumerate() {
            if i == 0 {
                cmd.arg(format!("-disposition:a:{i}")).arg("default");
            } else {
                cmd.arg(format!("-disposition:a:{i}")).arg("0");
            }
            cmd.arg(format!("-metadata:s:a:{i}"))
                .arg(format!("language={lang}"));
            cmd.arg(format!("-metadata:s:a:{i}"))
                .arg(format!("title={title}"));
        }

        cmd.arg(output);

        let status = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .status()
            .map_err(|e| AppError::Ffmpeg(e.to_string()))?;

        if status.success() {
            Ok(())
        } else {
            Err(AppError::Ffmpeg("multi-audio merge failed".into()))
        }
    }

    /// Run arbitrary ffmpeg args (used by ConversionService).
    pub fn run_args(&self, args: &[&str]) -> Result<()> {
        let status = Command::new(&self.binary)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .status()
            .map_err(|e| AppError::Ffmpeg(e.to_string()))?;

        if status.success() {
            Ok(())
        } else {
            Err(AppError::Ffmpeg("ffmpeg command failed".into()))
        }
    }
}
