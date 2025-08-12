use clap::Parser;
use regex::Regex;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

mod analyzer;
mod html_generator;
pub mod parser;
mod youtube_downloader;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'l', long, value_name = "FILE")]
    local_file: Option<String>,

    #[arg(short = 'y', long, value_name = "URL")]
    youtube_url: Option<String>,

    #[arg(short = 'm', long, value_name = "MODEL")]
    model: Option<String>,

    #[arg(long, value_name = "LIMIT")]
    limit: Option<usize>,
}

fn get_youtube_id(url: &str) -> Option<String> {
    let re = Regex::new(r"(?:watch\?v=|youtu\.be/)([\w-]+)").unwrap();
    re.captures(url).and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}

fn create_output_directory(base_name: &str) -> io::Result<PathBuf> {
    let mut path = PathBuf::from(base_name);
    if path.exists() {
        let mut i = 2;
        loop {
            let new_name = format!("{}_{:02}", base_name, i);
            path = PathBuf::from(new_name);
            if !path.exists() {
                break;
            }
            i += 1;
        }
    }
    fs::create_dir_all(&path)?;
    Ok(path)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let base_name = if let Some(local_file) = &args.local_file {
        PathBuf::from(local_file).file_stem().unwrap_or_default().to_string_lossy().to_string()
    } else if let Some(youtube_url) = &args.youtube_url {
        get_youtube_id(youtube_url).ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "Invalid YouTube URL"))?
    } else {
        eprintln!("Usage: srtgram -l <FILE> | -y <URL>");
        return Err(io::Error::new(ErrorKind::InvalidInput, "No input specified."));
    };

    let output_dir = create_output_directory(&base_name)?;
    println!("Output will be saved in: {}", output_dir.display());

    let (srt_path, youtube_url_opt, html_title) = if let Some(local_file) = &args.local_file {
        let path = PathBuf::from(local_file);
        let new_srt_path = output_dir.join(path.file_name().unwrap());
        fs::copy(&path, &new_srt_path)?;
        let title = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        (new_srt_path, None, title)
    } else if let Some(youtube_url) = &args.youtube_url {
        let downloaded_srt_path = youtube_downloader::download_youtube_subtitles(youtube_url, &output_dir).await?;
        let title = youtube_downloader::get_youtube_video_title(youtube_url).await?;
        (downloaded_srt_path, Some(youtube_url.clone()), title)
    } else {
        // This branch is technically unreachable due to the check above, but included for completeness
        return Err(io::Error::new(ErrorKind::InvalidInput, "No input specified."));
    };


    let sentences_json_path = parser::process_srt_file(&srt_path, &output_dir)?;

    analyzer::analyze_sentences_from_json(&sentences_json_path, args.model, &output_dir, args.limit).await.map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;

    let analysis_jsonl_path = output_dir.join("analysis.jsonl");
    html_generator::generate_html_from_jsonl(&analysis_jsonl_path, youtube_url_opt.as_deref(), &output_dir, &html_title)?;

    println!("\nAll steps completed.");
    println!("You can now open {} in your web browser.", output_dir.join("index.html").display());

    Ok(())
}