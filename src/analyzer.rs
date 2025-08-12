use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use crate::parser::Subtitle;

#[derive(Serialize)]
struct ApiRequest {
    model: String,
    prompt: String,
    temperature: f32,
    stream: bool,
}

#[derive(Deserialize)]
struct ApiResponse {
    response: String,
}

#[derive(Serialize, Deserialize)]
pub struct AnalysisResult {
    pub timestamp: String,
    pub original_sentence: String,
    pub explanation: String,
}

pub async fn analyze_sentences_from_json(
    json_path: &Path,
    model_name: Option<String>,
    output_dir: &Path,
    limit: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .no_proxy()
        .http1_only()
        .build()?;

    let json_content = fs::read_to_string(json_path)?;
    let mut subtitles: Vec<Subtitle> = serde_json::from_str(&json_content)?;

    if let Some(l) = limit {
        subtitles.truncate(l);
        println!("Analyzing first {} sentences.", l);
    }

    let output_path = output_dir.join("analysis.jsonl");
    fs::write(&output_path, "")?;
    let mut output_file = OpenOptions::new().append(true).open(&output_path)?;

    println!("Starting analysis with Ollama (/api/generate). This may take a while...");

    let actual_model_name = model_name.unwrap_or_else(|| "gemma3:12b".to_string());

    for (index, subtitle) in subtitles.iter().enumerate() {
        let sentence = &subtitle.text;
        if sentence.trim().is_empty() {
            continue;
        }

        println!("Analyzing sentence {}...: \"{}\"", index + 1, &sentence);

        let system_prompt = "あなたは優秀な英文法学者です。最初に和訳を示してから、文法の解説をしてください。";
        
        let mut full_prompt = String::new();
        full_prompt.push_str(system_prompt);
        full_prompt.push_str("\n\nSentence: \"");
        full_prompt.push_str(sentence);
        full_prompt.push('"');

        let request_body = ApiRequest {
            model: actual_model_name.clone(),
            prompt: full_prompt,
            temperature: 0.7,
            stream: false,
        };

        let res = client
            .post("http://localhost:11434/api/generate")
            .json(&request_body)
            .send()
            .await;

        let result = match res {
            Ok(response) => {
                if response.status().is_success() {
                    let api_response = response.json::<ApiResponse>().await?;
                    AnalysisResult {
                        timestamp: subtitle.timestamp.clone(),
                        original_sentence: sentence.clone(),
                        explanation: api_response.response,
                    }
                } else {
                    let err_msg = format!("Failed to get explanation. Status: {}", response.status());
                    eprintln!("Error for sentence '{}': {}", sentence, err_msg);
                    AnalysisResult {
                        timestamp: subtitle.timestamp.clone(),
                        original_sentence: sentence.clone(),
                        explanation: err_msg,
                    }
                }
            }
            Err(e) => {
                let err_msg = format!("API connection error: {}", e);
                eprintln!("Failed to connect to API for sentence '{}': {}.", sentence, e);
                AnalysisResult {
                    timestamp: subtitle.timestamp.clone(),
                    original_sentence: sentence.clone(),
                    explanation: err_msg,
                }
            }
        };

        let json_line = serde_json::to_string(&result)?;
        writeln!(output_file, "{}", json_line)?;
    }

    println!("Analysis complete. Output written to {}", output_path.display());

    Ok(())
}
