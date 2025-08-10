use clap::Parser;
use std::env;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

// すべてのモジュールを宣言
mod analyzer;
mod html_generator;
mod parser;
mod youtube_downloader;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to a local SRT file.
    #[arg(short = 'l', long, value_name = "FILE")]
    local_file: Option<String>,

    /// YouTube video URL to download subtitles from.
    #[arg(short = 'y', long, value_name = "URL")]
    youtube_url: Option<String>,

    /// Model to use for analysis.
    #[arg(short = 'm', long, value_name = "MODEL")]
    model: Option<String>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let input_path: PathBuf;

    if let Some(local_file_str) = args.local_file {
        input_path = PathBuf::from(local_file_str);
        if !input_path.exists() {
            eprintln!("Error: Local file not found at '{}'", input_path.display());
            return Err(io::Error::new(ErrorKind::NotFound, "Local file not found."));
        }
    } else if let Some(youtube_url) = args.youtube_url {
        input_path = youtube_downloader::download_youtube_subtitles(&youtube_url).await?;
    } else {
        eprintln!("Usage: {} -l <FILE> | -y <URL>", env::args().next().unwrap());
        return Err(io::Error::new(ErrorKind::InvalidInput, "No input specified."));
    }

    // 3. SRTファイルを解析してテキストファイルを生成
    println!("Step 1: Parsing SRT file...");
    let text_file_path = match parser::process_srt_file(&input_path) {
        Ok(path) => {
            println!("Successfully created text file at {}", path.display());
            path
        }
        Err(e) => {
            eprintln!("Failed to process SRT file: {}", e);
            return Err(e);
        }
    };

    // 4. 生成されたテキストファイルをollamaで解析し、JSONファイルを生成
    println!("\nStep 2: Analyzing text file with ollama...");
    let json_file_path = text_file_path.with_extension("analysis.json");
    if let Err(e) = analyzer::analyze_text_file(&text_file_path, args.model).await {
        eprintln!("An error occurred during analysis: {}", e);
        // エラーが発生しても続行する
    }

    // 5. 生成されたJSONファイルからHTMLファイルを生成
    println!("\nStep 3: Generating HTML viewer...");
    if let Err(e) = html_generator::generate_html_from_json(&json_file_path) {
        eprintln!("Failed to generate HTML file: {}", e);
        return Err(e);
    }

    println!("\nAll steps completed.");
    println!("You can now open {} in your web browser to view the analysis.", json_file_path.with_extension("html").display());

    Ok(())
}