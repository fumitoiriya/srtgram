use std::io::{self, ErrorKind};
use std::path::PathBuf;
use std::process::Command;

/// Downloads English SRT subtitles from a given YouTube URL.
///
/// This function uses the external `yt-dlp` command-line tool.
/// It saves the downloaded subtitle to a file named `subtitle.srt.en.srt`
/// in the current working directory.
///
/// # Arguments
/// * `youtube_url` - The URL of the YouTube video.
///
/// # Returns
/// A `Result` containing the `PathBuf` to the downloaded subtitle file,
/// or an `io::Error` if `yt-dlp` is not found or the download fails.
pub async fn download_youtube_subtitles(youtube_url: &str) -> io::Result<PathBuf> {
    println!("Attempting to download subtitles from YouTube: {}", youtube_url);

    // yt-dlpの存在チェック
    if let Err(_) = Command::new("yt-dlp").arg("--version").output() {
        eprintln!("Error: 'yt-dlp' not found. Please install yt-dlp to use YouTube subtitle download feature.");
        eprintln!("  (e.g., pip install yt-dlp or brew install yt-dlp)");
        return Err(io::Error::new(ErrorKind::NotFound, "yt-dlp not found."));
    }

    // ダウンロードした字幕の保存先ファイル名 (yt-dlpが自動で言語コードと拡張子を付与するため、それに合わせる)
    let base_filename = "subtitle";
    let downloaded_subtitle_filename = format!("{}.en.srt", base_filename);
    let downloaded_subtitle_path = PathBuf::from(downloaded_subtitle_filename);

    // yt-dlpで字幕をダウンロード
    let output = Command::new("yt-dlp")
        .arg("--write-auto-subs") // 自動生成字幕も対象に含める
        .arg("--sub-lang")
        .arg("en") // 英語字幕を指定
        .arg("--sub-format")
        .arg("srt")
        .arg("--skip-download")
        .arg("--output")
        .arg(base_filename) // ここはベースファイル名のみ指定
        .arg(youtube_url)
        .output()?;

    // yt-dlpの標準出力と標準エラー出力を表示
    println!("yt-dlp Stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("yt-dlp Stderr:\n{}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        eprintln!("Error downloading subtitles:");
        return Err(io::Error::new(ErrorKind::Other, "Failed to download subtitles."));
    }

    println!("Subtitles downloaded to: {}", downloaded_subtitle_path.display());
    Ok(downloaded_subtitle_path)
}