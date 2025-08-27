use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

pub async fn download_youtube_subtitles(url: &str, output_dir: &Path) -> io::Result<PathBuf> {
    println!("Attempting to download subtitles from YouTube: {}", url);

    // yt-dlpにファイル名を完全に任せるため、-oオプションはテンプレートを使う
    // -Pでディレクトリを指定し、-oでシンプルなファイル名テンプレートを使う
    let output_template = "subtitle"; // yt-dlpが拡張子を付与する
    let lang = "en";

    let output = Command::new("yt-dlp")
        .arg("--write-auto-subs") // User's fix
        .arg("--sub-lang")
        .arg(lang)
        .arg("--convert-subtitles")
        .arg("srt")
        .arg("--skip-download") // Keep this
        .arg("-P") // Specify output directory
        .arg(output_dir.to_str().unwrap())
        .arg("-o") // Specify output filename template
        .arg(output_template)
        .arg(url)
        .output()?;

    if !output.status.success() {
        eprintln!("yt-dlp failed:\nStdout: {}\nStderr: {}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        return Err(io::Error::new(io::ErrorKind::Other, "yt-dlp failed to download subtitles."));
    }

    let final_srt_path =  output_dir.join(format!("{}.{}.srt",output_template, lang));

    println!("Subtitles downloaded to: {}", final_srt_path.display());

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
