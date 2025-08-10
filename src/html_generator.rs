use pulldown_cmark::{html, Parser};
use serde::Deserialize;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[derive(Deserialize)]
struct AnalysisResult {
    original_sentence: String,
    explanation: String,
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn generate_html_from_json(json_path: &Path) -> io::Result<()> {
    let json_content = fs::read_to_string(json_path)?;
    let results: Vec<AnalysisResult> = serde_json::from_str(&json_content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let output_path = json_path.with_extension("html");
    let mut file = fs::File::create(&output_path)?;

    // --- HTMLとCSSのヘッダー --- 
    let head_html = r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>SRT文法解析ビューア</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; line-height: 1.6; background-color: #f4f4f9; color: #333; margin: 0; padding: 20px; }
        h1 { color: #2c3e50; text-align: center; }
        #container { max-width: 800px; margin: 0 auto; background-color: #fff; padding: 25px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .entry { border-bottom: 1px solid #eee; padding: 15px 0; }
        .entry:last-child { border-bottom: none; }
        .sentence { font-weight: bold; font-size: 1.2em; color: #34495e; margin-bottom: 10px; cursor: pointer; user-select: none; position: relative; padding-left: 20px; }
        .sentence::before { content: '▶'; position: absolute; left: 0; top: 5px; font-size: 0.8em; color: #95a5a6; transition: transform 0.2s; }
        .sentence.active::before { transform: rotate(90deg); }
        .explanation { display: none; margin-top: 10px; background-color: #f8f9fa; padding: 15px; border-radius: 5px; border: 1px solid #ddd; word-wrap: break-word; }
        .explanation.visible { display: block; }
        .explanation h1, .explanation h2, .explanation h3 { color: #2c3e50; margin-top: 1em; margin-bottom: 0.5em; border-bottom: 1px solid #eaecef; padding-bottom: 0.3em; }
        .explanation p { margin-top: 0; margin-bottom: 1em; }
        .explanation ul, .explanation ol { padding-left: 2em; }
        .explanation li { margin-bottom: 0.5em; }
        .explanation code { background-color: #e1e4e8; padding: .2em .4em; margin: 0; font-size: 85%; border-radius: 3px; font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, Courier, monospace; }
        .explanation pre { background-color: #2d2d2d; color: #f1f1f1; padding: 1em; border-radius: 5px; overflow-x: auto; }
        .explanation pre code { background-color: transparent; padding: 0; }
        .explanation blockquote { padding: 0 1em; color: #6a737d; border-left: 0.25em solid #dfe2e5; }
    </style>
</head>
<body>
    <div id="container">
        <h1>SRT文法解析ビューア</h1>
        <p style="text-align:center;">各英文をクリックすると、解説が開閉します。</p>
        <div id="results">
"#;
    file.write_all(head_html.as_bytes())?;

    // --- 各エントリを処理 --- 
    for item in results {
        let parser = Parser::new(&item.explanation);
        let mut explanation_html = String::new();
        html::push_html(&mut explanation_html, parser);

        writeln!(file, "            <div class=\"entry\">")?;
        writeln!(file, "                <div class=\"sentence\">{}</div>", escape_html(&item.original_sentence))?;
        writeln!(file, "                <div class=\"explanation\">{}</div>", explanation_html)?;
        writeln!(file, "            </div>")?;
    }

    // --- HTMLのフッターとJavaScript --- 
    let foot_html = r#"
        </div>
    </div>
    <script>
        document.addEventListener('DOMContentLoaded', () => {
            const sentences = document.querySelectorAll('.sentence');
            sentences.forEach(sentence => {
                sentence.addEventListener('click', () => {
                    sentence.classList.toggle('active');
                    const explanation = sentence.nextElementSibling;
                    if (explanation) {
                        explanation.classList.toggle('visible');
                    }
                });
            });
        });
    </script>
</body>
</html>"#;
    file.write_all(foot_html.as_bytes())?;

    println!("Successfully generated interactive HTML file at {}", output_path.display());

    Ok(())
}
