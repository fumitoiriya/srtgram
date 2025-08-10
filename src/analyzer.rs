use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::Path;
use std::time::Duration;

#[derive(Serialize)]
struct ApiRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Serialize)]
struct AnalysisResult {
    original_sentence: String,
    explanation: String,
}

pub async fn analyze_text_file(text_file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;

    let file = File::open(text_file_path)?;
    let reader = io::BufReader::new(file);

    let mut results: Vec<AnalysisResult> = Vec::new(); // ここに移動

    println!("Starting analysis with LM Studio. This may take a while...");

    for (index, line) in reader.lines().enumerate() {
        let sentence = line?;
        if sentence.trim().is_empty() {
            continue;
        }

        println!("Analyzing sentence {}...: \"{}\"", index + 1, &sentence);

        let system_prompt = "あなたは英語文法のエキスパートです。次のセンテンスについて、まず和訳を示し、その後文法について解説してください。".to_string();
        
        let request_body = ApiRequest {
            model: "gemma3:27b".to_string(),
            messages: vec![
                Message { role: "system".to_string(), content: system_prompt },
                Message { role: "user".to_string(), content: sentence.clone() },
            ],
            temperature: 0.7,
        };

        let res = client
            //.post("http://localhost:1234/v1/chat/completions")
            .post("http://localhost:11434/api/chat")
            .json(&request_body)
            .send()
            .await;

        match res {
            Ok(response) => {
                if response.status().is_success() {
                    let api_response = response.json::<ApiResponse>().await?;
                    let explanation = if let Some(choice) = api_response.choices.get(0) {
                        choice.message.content.clone()
                    } else {
                        "No explanation received.".to_string()
                    };
                    results.push(AnalysisResult {
                        original_sentence: sentence,
                        explanation,
                    });
                } else {
                     eprintln!("Error from API for sentence '{}': Status {}", sentence, response.status());
                     results.push(AnalysisResult {
                        original_sentence: sentence,
                        explanation: format!("Failed to get explanation. Status: {}", response.status()),
                    });
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to LM Studio API for sentence '{}': {}. Please ensure LM Studio is running and the server is on.", sentence, e);
                results.push(AnalysisResult {
                    original_sentence: sentence,
                    explanation: format!("API connection error: {}", e),
                });
            }
        }
    }

    let output_path = text_file_path.with_extension("analysis.json");
    let json_output = serde_json::to_string_pretty(&results)?;
    fs::write(&output_path, json_output)?;

    println!("Analysis complete. Output written to {}", output_path.display());

    Ok(())
}