use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};
use crate::parser::Subtitle;

#[derive(Serialize)]
struct ApiRequest {
    model: String,
    prompt: String,
    temperature: f32,
    stream: bool,
    format: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    response: String,
    #[serde(default)]
    eval_count: Option<u64>,
    #[serde(default)]
    eval_duration: Option<u64>,
}

// For parsing the JSON string within the response
#[derive(Serialize, Deserialize)]
struct LlmJsonResponse {
    translation: String,
    explanation: String,
}

#[derive(Serialize, Deserialize)]
pub struct AnalysisResult {
    pub timestamp: String,
    pub original_sentence: String,
    pub translation: String,
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

        let system_prompt = r###"あなたは優秀な英文法学者です。以下のJSON形式で、提供された英文の和訳と文法解説を日本語で生成してください。explanationフィールドにはマークダウンを使用してください。\n{ \"translation\": \"<ここに和訳>\", \"explanation\": \"<ここに文法解説>\" }\n最終応答は、"{"で始まり"}"で終わるJSONのみを出力し、JSON以外の文字は一切応答に含めないでください。"###;
        
        let mut full_prompt = String::new();
        full_prompt.push_str(system_prompt);
        full_prompt.push_str("\n\nSentence: \"");
        full_prompt.push_str(sentence);
        full_prompt.push('"');

        let request_body = ApiRequest {
            model: actual_model_name.clone(),
            prompt: full_prompt,
            temperature: 0.3,
            stream: false,
            format: "json".to_string(),
        };

        let start_time = Instant::now();
        let res = client
            .post("http://localhost:11434/api/generate")
            .json(&request_body)
            .send()
            .await;
        let elapsed_time = start_time.elapsed();

        let result = match res {
            Ok(response) => {
                if response.status().is_success() {
                    let api_response = response.json::<ApiResponse>().await?;
                    print!("  Response time: {:.2?},", elapsed_time);

                    if let (Some(eval_count), Some(eval_duration)) = (api_response.eval_count, api_response.eval_duration) {
                        if eval_duration > 0 {
                            let tokens_per_second = (eval_count as f64 / eval_duration as f64) * 1_000_000_000.0;
                            println!("  Tokens/s: {:.2}", tokens_per_second);
                        } else {
                            println!("  Tokens/s: N/A (eval_duration was zero)");
                        }
                    } else {
                        println!("  Tokens/s: N/A (eval_count or eval_duration not available)");
                    }

                    // Parse the JSON string from the response
                    match serde_json::from_str::<LlmJsonResponse>(&api_response.response) {
                        Ok(llm_json) => AnalysisResult {
                            timestamp: subtitle.timestamp.clone(),
                            original_sentence: sentence.clone(),
                            translation: llm_json.translation,
                            explanation: llm_json.explanation,
                        },
                        Err(e) => {
                            let err_msg = format!("Failed to parse LLM JSON response: {}", e);
                            eprintln!("Error for sentence '{}': {}", sentence, err_msg);
                            // Save the raw response for debugging
                            AnalysisResult {
                                timestamp: subtitle.timestamp.clone(),
                                original_sentence: sentence.clone(),
                                translation: "Error: Failed to parse LLM response.".to_string(),
                                explanation: api_response.response,
                            }
                        }
                    }
                } else {
                    let err_msg = format!("Failed to get explanation. Status: {}", response.status());
                    eprintln!("Error for sentence '{}': {}", sentence, err_msg);
                    AnalysisResult {
                        timestamp: subtitle.timestamp.clone(),
                        original_sentence: sentence.clone(),
                        translation: "Error: API request failed.".to_string(),
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
                    translation: "Error: API connection failed.".to_string(),
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