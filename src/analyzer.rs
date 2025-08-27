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
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
}

#[derive(Deserialize)]
struct ApiResponse {
    response: String,
    #[serde(default)]
    eval_count: Option<u64>,
    #[serde(default)]
    eval_duration: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct AnalysisResult {
    pub timestamp: String,
    pub original_sentence: String,
    pub translation: String,
    pub explanation: String,
}

async fn call_ollama_api(
    client: &reqwest::Client,
    model_name: &str,
    prompt: String,
) -> Result<ApiResponse, String> {
    let request_body = ApiRequest {
        model: model_name.to_string(),
        prompt,
        temperature: 0.3,
        stream: false,
        format: None,
    };

    let res = client
        .post("http://localhost:11434/api/generate")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        res.json::<ApiResponse>().await.map_err(|e| e.to_string()) 
    } else {
        Err(format!("API request failed with status: {}", res.status()))
    }
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

    let total_sentences = if let Some(l) = limit {
        subtitles.truncate(l);
        println!("Analyzing first {} sentences.", l);
        l
    } else {
        subtitles.len()
    };

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

        println!("Analyzing sentence {}/{}...: \"{}\"", index + 1, total_sentences, &sentence);

        // 1. Get translation
        let start_time = Instant::now();
        let translation_prompt = format!(
            "あなたは優秀な翻訳家です。以下の英文を自然な日本語に翻訳してください。翻訳文のみを返してください。他の言葉は一切含めないでください。翻訳を\"「\"や\"」\"で囲む必要はありません。\n\nSentence: \"{}\"",
            sentence
        );
        
        let translation = match call_ollama_api(&client, &actual_model_name, translation_prompt).await {
            Ok(api_response) => {
                let elapsed_time = start_time.elapsed();
                print!("  Translation: Response time: {:.2?}", elapsed_time);
                if let (Some(eval_count), Some(eval_duration)) = (api_response.eval_count, api_response.eval_duration) {
                    if eval_duration > 0 {
                        let tokens_per_second = (eval_count as f64 / eval_duration as f64) * 1_000_000_000.0;
                        println!(", Tokens/s: {:.2}", tokens_per_second);
                    } else {
                        println!("");
                    }
                } else {
                    println!("");
                }
                api_response.response.trim().to_string()
            }
            Err(e) => {
                eprintln!("\nError getting translation for sentence '{}': {}", sentence, e);
                "Error: Failed to get translation.".to_string()
            }
        };

        // 2. Get explanation
        let start_time = Instant::now();
        let explanation_prompt = format!(
            "あなたは優秀な英文法学者です。以下の英文について、文法的な解説を日本語で提供してください。解説はマークダウン形式で記述してください。解説文のみを返してください。他の言葉は一切含めないでください。最初の横線も不要です。\n\nSentence: \"{}\"",
            sentence
        );
        
        let explanation = match call_ollama_api(&client, &actual_model_name, explanation_prompt).await {
            Ok(api_response) => {
                let elapsed_time = start_time.elapsed();
                print!("  Explanation: Response time: {:.2?}", elapsed_time);
                if let (Some(eval_count), Some(eval_duration)) = (api_response.eval_count, api_response.eval_duration) {
                    if eval_duration > 0 {
                        let tokens_per_second = (eval_count as f64 / eval_duration as f64) * 1_000_000_000.0;
                        println!(", Tokens/s: {:.2}", tokens_per_second);
                    } else {
                        println!("");
                    }
                } else {
                    println!("");
                }
                api_response.response
            }
            Err(e) => {
                eprintln!("\nError getting explanation for sentence '{}': {}", sentence, e);
                format!("Error: Failed to get explanation. Details: {}", e)
            }
        };

        let result = AnalysisResult {
            timestamp: subtitle.timestamp.clone(),
            original_sentence: sentence.clone(),
            translation,
            explanation,
        };

        let json_line = serde_json::to_string(&result)?;
        writeln!(output_file, "{}", json_line)?;
    }

    println!("Analysis complete. Output written to {}", output_path.display());

    Ok(())
}
