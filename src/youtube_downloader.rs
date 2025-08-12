use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

pub async fn download_youtube_subtitles(url: &str, output_dir: &Path) -> io::Result<PathBuf> {
    println!("Attempting to download subtitles from YouTube: {}", url);

    // yt-dlpにファイル名を完全に任せるため、-oオプションはテンプレートを使う
    // -Pでディレクトリを指定し、-oでシンプルなファイル名テンプレートを使う
    let output_template = "subtitle"; // yt-dlpが拡張子を付与する

    let output = Command::new("yt-dlp")
        .arg("--write-auto-subs") // User's fix
        .arg("--sub-lang")
        .arg("en")
        .arg("--sub-format")
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

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let mut final_srt_path = PathBuf::new(); // Initialize as empty

    // stdoutから "Destination: " の行を探し、ファイル名を抽出する
    for line in stdout_str.lines() {
        if line.contains("Destination:") && line.contains(".srt") {
            if let Some(start_index) = line.find("Destination: ") {
                let file_name_str = line[start_index + "Destination: ".len()..].trim();

                break;
            }
        }
    }

    // もしstdoutからファイル名が抽出できなかった場合、デフォルトのパスを試す
    if final_srt_path.as_os_str().is_empty() {
        final_srt_path = output_dir.join("subtitle.srt"); // Fallback to expected name
        // もしそれでも見つからなければ、subtitle.en.srtも試す
        if !final_srt_path.exists() {
            final_srt_path = output_dir.join("subtitle.en.srt");
        }
    }


    println!("Subtitles downloaded to: {}", final_srt_path.display());

    Ok(final_srt_path)
}