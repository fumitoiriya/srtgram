use regex::Regex;
use std::fs;
use std::io::{self};
use std::path::{Path, PathBuf};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Subtitle {
    pub timestamp: String,
    pub text: String,
}

pub fn process_srt_file(input_path: &Path, output_dir: &Path) -> io::Result<PathBuf> {
    let output_path = output_dir.join("sentences.json");
    let srt_content = fs::read_to_string(input_path)?;

    let re = Regex::new(r"\d+\r?\n(\d{2}:\d{2}:\d{2},\d{3}) --> .*\r?\n([\s\S]+?)(?:\r?\n\r?\n|\z)").unwrap();

    let mut subtitles = Vec::new();
    let mut current_sentence_parts: Vec<(String, String)> = Vec::new(); // (text_part, start_time)
    
    for cap in re.captures_iter(&srt_content) {
        let block_start_time = cap[1].to_string();
        let block_text = cap[2].replace("\r\n", " ").replace('\n', " ").trim().to_string();

        // 現在のブロックのテキストとタイムスタンプをパーツとして追加
        current_sentence_parts.push((block_text, block_start_time));

        // 現在のブロックのテキストが文の区切りで終わるか
        if current_sentence_parts.last().map_or(false, |(text, _)| text.ends_with('.') || text.ends_with('?') || text.ends_with('!')) {
            let combined_text: String = current_sentence_parts.iter().map(|(text, _)| text.clone()).collect::<Vec<String>>().join(" ");
            
            let sentences: Vec<&str> = combined_text.split_inclusive(&['.', '?', '!'][..]).collect();
            
            let mut current_char_index = 0;
            for sentence_slice in sentences {
                let trimmed_sentence = sentence_slice.trim();
                if !trimmed_sentence.is_empty() {
                    // この文の開始タイムスタンプを特定する
                    let mut sentence_start_time = String::new();
                    let mut found_start_time = false;

                    let mut temp_char_count = 0;
                    for (part_text, part_time) in &current_sentence_parts {
                        if current_char_index >= temp_char_count && current_char_index < temp_char_count + part_text.len() {
                            sentence_start_time = part_time.clone();
                            found_start_time = true;
                            break;
                        }
                        temp_char_count += part_text.len() + 1; // +1 for space added when joining
                    }
                    // もし見つからなければ、最初のブロックのタイムスタンプを使う（フォールバック）
                    if !found_start_time && !current_sentence_parts.is_empty() {
                        sentence_start_time = current_sentence_parts[0].1.clone();
                    }


                    subtitles.push(Subtitle {
                        timestamp: sentence_start_time,
                        text: trimmed_sentence.to_string(),
                    });
                }
                current_char_index += sentence_slice.len(); // 次の文の開始位置を更新
            }
            current_sentence_parts.clear(); // 処理が終わったのでクリア
        }
    }

    // ループの最後に残ったテキストを処理 (同様のロジックを適用)
    if !current_sentence_parts.is_empty() {
        let combined_text: String = current_sentence_parts.iter().map(|(text, _)| text.clone()).collect::<Vec<String>>().join(" ");
        let sentences: Vec<&str> = combined_text.split_inclusive(&['.', '?', '!'][..]).collect();
        
        let mut current_char_index = 0;
        for sentence_slice in sentences {
            let trimmed_sentence = sentence_slice.trim();
            if !trimmed_sentence.is_empty() {
                let mut sentence_start_time = String::new();
                let mut found_start_time = false;

                let mut temp_char_count = 0;
                for (part_text, part_time) in &current_sentence_parts {
                    if current_char_index >= temp_char_count && current_char_index < temp_char_count + part_text.len() {
                        sentence_start_time = part_time.clone();
                        found_start_time = true;
                        break;
                    }
                    temp_char_count += part_text.len() + 1;
                }
                if !found_start_time && !current_sentence_parts.is_empty() {
                    sentence_start_time = current_sentence_parts[0].1.clone();
                }

                subtitles.push(Subtitle {
                    timestamp: sentence_start_time,
                    text: trimmed_sentence.to_string(),
                });
            }
            current_char_index += sentence_slice.len();
        }
    }

    let json_output = serde_json::to_string_pretty(&subtitles)?;
    fs::write(&output_path, json_output)?;

    println!("Successfully created sentences file at {}", output_path.display());
    Ok(output_path)
}