use std::fs;
use std::io::{self};
use std::path::{Path, PathBuf};

pub fn process_srt_file(input_path: &Path) -> io::Result<PathBuf> {
    let output_path = input_path.with_extension("txt");
    let srt_content = fs::read_to_string(input_path)?;

    let text_lines: Vec<String> = srt_content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.contains("-->") || trimmed.parse::<u32>().is_ok() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect();

    let single_line_text = text_lines.join(" ");

    let mut formatted_text = single_line_text.replace(". ", ".\n");
    formatted_text = formatted_text.replace("? ", "?\n");

    let abbreviations = [
        "Mr.\n", "Mrs.\n", "Ms.\n", "Dr.\n", "Prof.\n", "Rev.\n", 
        "Sen.\n", "Gov.\n", "Gen.\n", "Capt.\n", "Sgt.\n", "St.\n",
        "etc.\n", "i.e.\n", "e.g.\n", "vs.\n", "a.m.\n", "p.m.\n"
    ];
    for &abbr in &abbreviations {
        let corrected_abbr_with_space = format!("{} ", &abbr[..abbr.len() - 1]);
        formatted_text = formatted_text.replace(abbr, &corrected_abbr_with_space);
    }

    fs::write(&output_path, formatted_text.as_bytes())?;

    Ok(output_path)
}
