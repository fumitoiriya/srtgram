use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

pub async fn download_youtube_subtitles(url: &str, output_dir: &Path) -> io::Result<PathBuf> {
    println!("Attempting to download subtitles from YouTube: {}", url);

    let output_template = "subtitle";
    let lang = "en";
    let final_srt_path = output_dir.join(format!("{}.{}.srt", output_template, lang));

    // 1. Manually created subtitles first
    println!("Trying to download manual subtitles...");
    let output = Command::new("yt-dlp")
        .arg("--write-sub")
        .arg("--sub-lang")
        .arg(lang)
        .arg("--sub-format")
        .arg("srt")
        .arg("--skip-download")
        .arg("-P")
        .arg(output_dir.to_str().unwrap())
        .arg("-o")
        .arg(output_template)
        .arg(url)
        .output()?;

    // Check if manual subtitles were downloaded successfully
    if output.status.success() && final_srt_path.exists() {
        println!("Successfully downloaded manual subtitles to: {}", final_srt_path.display());
        return Ok(final_srt_path);
    }

    // 2. If manual subtitles fail or don't exist, try auto-generated ones
    println!("Manual subtitles not found. Trying to download automatic subtitles...");
    let output_auto = Command::new("yt-dlp")
        .arg("--write-auto-subs") // Changed argument
        .arg("--sub-lang")
        .arg(lang)
        .arg("--sub-format")
        .arg("srt")
        .arg("--skip-download")
        .arg("-P")
        .arg(output_dir.to_str().unwrap())
        .arg("-o")
        .arg(output_template)
        .arg(url)
        .output()?;

    if !output_auto.status.success() || !final_srt_path.exists() {
        eprintln!("yt-dlp failed to download any subtitles.\nStdout: {}\nStderr: {}", String::from_utf8_lossy(&output_auto.stdout), String::from_utf8_lossy(&output_auto.stderr));
        return Err(io::Error::new(io::ErrorKind::Other, "yt-dlp failed to download any subtitles."));
    }

    println!("Successfully downloaded automatic subtitles to: {}", final_srt_path.display());

    Ok(final_srt_path)
}

pub async fn get_youtube_video_title(url: &str) -> io::Result<String> {
    let output = Command::new("yt-dlp")
        .arg("--get-title")
        .arg(url)
        .output()?;

    if !output.status.success() {
        eprintln!("yt-dlp failed to get title:\nStdout: {}\nStderr: {}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        return Err(io::Error::new(io::ErrorKind::Other, "yt-dlp failed to get video title."));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn get_youtube_video_duration(url: &str) -> io::Result<String> {
    let output = Command::new("yt-dlp")
        .arg("--get-duration")
        .arg(url)
        .output()?;

    if !output.status.success() {
        eprintln!("yt-dlp failed to get duration:\nStdout: {}\nStderr: {}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        return Err(io::Error::new(io::ErrorKind::Other, "yt-dlp failed to get video duration."));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn download_youtube_thumbnail(url: &str, output_dir: &Path) -> io::Result<PathBuf> {
    println!("Downloading thumbnail...");
    let output = Command::new("yt-dlp")
        .arg("--write-thumbnail")
        .arg("--skip-download")
        .arg("-o")
        .arg(output_dir.join("thumbnail.%(ext)s").to_str().unwrap())
        .arg(url)
        .output()?;

    if !output.status.success() {
        eprintln!("yt-dlp failed to download thumbnail:\nStdout: {}\nStderr: {}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        return Err(io::Error::new(io::ErrorKind::Other, "yt-dlp failed to download thumbnail."));
    }

    let thumbnail_path = find_thumbnail_file(output_dir)?;
    let final_thumbnail_path = output_dir.join("thumbnail.png");

    if thumbnail_path != final_thumbnail_path {
        if final_thumbnail_path.exists() {
            std::fs::remove_file(&final_thumbnail_path)?;
        }
        std::fs::rename(&thumbnail_path, &final_thumbnail_path)?;
    }
    
    println!("Thumbnail downloaded to: {}", final_thumbnail_path.display());
    Ok(final_thumbnail_path)
}

fn find_thumbnail_file(dir: &Path) -> io::Result<PathBuf> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(stem) = path.file_stem() {
            if stem == "thumbnail" {
                if let Some(ext) = path.extension() {
                    if ext == "webp" || ext == "jpg" || ext == "png" || ext == "jpeg" {
                         return Ok(path);
                    }
                }
            }
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "Thumbnail file not found after download."))
}
