use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Serialize)]
pub struct Metadata {
    title: String,
    video_url: Option<String>,
    duration: Option<String>,
    sentence_count: usize,
    thumbnail_path: Option<String>,
    report_path: String,
    creation_date: String,
    output_dir_name: String,
}

pub fn generate_and_save_metadata(
    output_dir: &Path,
    html_title: String,
    youtube_url_opt: Option<String>,
    duration_opt: Option<String>,
    sentence_count: usize,
    thumbnail_path_opt: Option<String>,
) -> io::Result<()> {
    let output_dir_name = output_dir.file_name().unwrap().to_string_lossy().to_string();

    let metadata = Metadata {
        title: html_title,
        video_url: youtube_url_opt,
        duration: duration_opt,
        sentence_count,
        thumbnail_path: thumbnail_path_opt,
        report_path: format!("{}/index.html", output_dir_name),
        creation_date: Utc::now().to_rfc3339(),
        output_dir_name,
    };

    let metadata_path = output_dir.join("metadata.json");
    let metadata_json = serde_json::to_string_pretty(&metadata)?;
    fs::write(&metadata_path, metadata_json)?;
    println!("Metadata file created at {}", metadata_path.display());

    Ok(())
}
