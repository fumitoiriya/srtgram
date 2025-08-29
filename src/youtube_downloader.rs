use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

async fn get_available_subtitles(url: &str) -> io::Result<Vec<String>> {
    let output = Command::new("yt-dlp").arg("--list-subs").arg(url).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Failed to list subtitles. Stderr: {}", stderr);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to list subtitles: {}", stderr),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut subs = Vec::new();
    let mut in_manual_subs_section = false;

    for line in stdout.lines() {
        if line.contains("Available subtitles for") {
            in_manual_subs_section = true;
            continue;
        } else if line.contains("Available automatic captions for") {
            in_manual_subs_section = false;
            continue;
        }

        if in_manual_subs_section {
            if line.starts_with("Language") || line.trim().is_empty() {
                continue;
            }
            if let Some(lang) = line.split_whitespace().next() {
                subs.push(lang.to_string());
            }
        }
    }
    Ok(subs)
}

pub async fn download_youtube_subtitles(url: &str, output_dir: &Path) -> io::Result<PathBuf> {
    println!("Attempting to download subtitles from YouTube: {}", url);

    let output_template = "subtitle";
    let base_lang = "en";

    // 1. Check for available manual English subtitles
    println!("Checking for available manual English subtitles...");
    let mut lang_to_try = base_lang.to_string();
    let mut is_manual_sub_found = false;

    match get_available_subtitles(url).await {
        Ok(langs) => {
            let best_lang = langs.iter().find(|l| l.as_str() == base_lang)
                               .or_else(|| langs.iter().find(|l| l.starts_with("en-")));

            if let Some(lang) = best_lang {
                println!("Found manual English subtitle: {}", lang);
                lang_to_try = lang.clone();
                is_manual_sub_found = true;
            } else {
                println!("No manual English subtitles found. Will try for automatic captions.");
            }
        }
        Err(e) => {
            eprintln!("Could not get subtitle list: {}. Falling back to default.", e);
        }
    }

    // 2. Try to download the determined subtitle type
    if is_manual_sub_found {
        println!("Trying to download manual '{}' subtitles...", &lang_to_try);
        let manual_srt_path = output_dir.join(format!("{}.{}.srt", output_template, &lang_to_try));

        let output = Command::new("yt-dlp")
            .arg("--write-sub")
            .arg("--sub-lang")
            .arg(&lang_to_try)
            .arg("--sub-format")
            .arg("srt")
            .arg("--skip-download")
            .arg("-P")
            .arg(output_dir.to_str().unwrap())
            .arg("-o")
            .arg(output_template)
            .arg(url)
            .output()?;

        if output.status.success() && manual_srt_path.exists() {
            println!("Successfully downloaded manual subtitles to: {}", manual_srt_path.display());
            return Ok(manual_srt_path);
        }
        println!("Manual subtitles download failed. Falling back to automatic.");
    }
    
    // 3. If manual subtitles fail or don't exist, try auto-generated ones
    println!("Trying automatic subtitles...");
    let auto_srt_path = output_dir.join(format!("{}.{}.srt", output_template, base_lang));
    let output_auto = Command::new("yt-dlp")
        .arg("--write-auto-subs")
        .arg("--sub-lang")
        .arg(base_lang)
        .arg("--sub-format")
        .arg("srt")
        .arg("--skip-download")
        .arg("-P")
        .arg(output_dir.to_str().unwrap())
        .arg("-o")
        .arg(output_template)
        .arg(url)
        .output()?;

    if !output_auto.status.success() || !auto_srt_path.exists() {
        let stderr = String::from_utf8_lossy(&output_auto.stderr);
        eprintln!("yt-dlp failed to download any subtitles.\nStderr: {}", stderr);
        return Err(io::Error::new(io::ErrorKind::Other, format!("yt-dlp failed to download any subtitles. Stderr: {}", stderr)));
    }

    println!("Successfully downloaded automatic subtitles to: {}", auto_srt_path.display());

    Ok(auto_srt_path)
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