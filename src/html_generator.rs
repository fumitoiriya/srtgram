use pulldown_cmark::{html, Options, Parser};
use regex::Regex;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use crate::analyzer::AnalysisResult;

fn get_youtube_embed_url(url: &str) -> Option<String> {
    let re = Regex::new(r"(?:watch\?v=|youtu\.be/)([\w-]+)").unwrap();
    re.captures(url).and_then(|cap| {
        cap.get(1)
            .map(|match_| format!("https://www.youtube.com/embed/{}", match_.as_str()))
    })
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&#39;")
}

pub fn generate_html_from_jsonl(
    jsonl_path: &Path,
    youtube_url: Option<&str>,
    output_dir: &Path,
    title: &str,
) -> io::Result<()> {
    let file = File::open(jsonl_path)?;
    let reader = BufReader::new(file);

    let mut results: Vec<AnalysisResult> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(result) = serde_json::from_str(&line) {
            results.push(result);
        }
    }

    let output_path = output_dir.join("index.html");
    let mut file = fs::File::create(&output_path)?;

    let head_html = format!("<!DOCTYPE html>
<html lang=\"ja\">
<head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
    <title>{}</title>
    <style>
        html, body {{ height: 100%; margin: 0; overflow: hidden; font-family: -apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, Helvetica, Arial, sans-serif; color: #333; }}
        #main-container {{ display: flex; flex-direction: column; height: 100vh; }}
        #video-container {{ display: flex; justify-content: center; align-items: center; flex-shrink: 0; background: #000; padding: 10px; gap: 10px; }}
        #video-container iframe {{ width: 100%; max-width: 800px; aspect-ratio: 16 / 9; border: none; }}
        #video-container img {{ max-width: 240px; max-height: 135px; object-fit: cover; border-radius: 8px; }}
        #results-wrapper {{ flex-grow: 1; overflow-y: auto; background-color: #f4f4f9; }}
        #container {{ max-width: 800px; margin: 0 auto; background-color: #fff; padding: 25px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); margin-top: 20px; margin-bottom: 20px; }}
        h1 {{ color: #2c3e50; text-align: center; }}
        .entry {{ border-bottom: 1px solid #eee; padding: 15px 0; }}
        .entry:last-child {{ border-bottom: none; }}
        .sentence {{ font-weight: bold; font-size: 1.2em; color: #34495e; margin-bottom: 10px; cursor: pointer; user-select: none; position: relative; padding-left: 20px; }}
        .sentence::before {{ content: '▶'; position: absolute; left: 0; top: 5px; font-size: 0.8em; color: #95a5a6; transition: transform 0.2s; }}
        .sentence.active::before {{ transform: rotate(90deg); }}
        .explanation {{ display: none; margin-top: 10px; background-color: #f8f9fa; padding: 15px; border-radius: 5px; border: 1px solid #ddd; word-wrap: break-word; }}
        .explanation.visible {{ display: block; }}
        .explanation h1, .explanation h2, .explanation h3 {{ color: #2c3e50; margin-top: 1em; margin-bottom: 0.5em; border-bottom: 1px solid #eaecef; padding-bottom: 0.3em; }}
        .explanation p {{ margin-top: 0; margin-bottom: 1em; }}
        .explanation ul, .explanation ol {{ padding-left: 2em; }}
        .explanation li {{ margin-bottom: 0.5em; }}
        .explanation code {{ background-color: #e1e4e8; padding: .2em .4em; margin: 0; font-size: 85%; border-radius: 3px; font-family: \"SFMono-Regular\", Consolas, \"Liberation Mono\", Menlo, Courier, monospace; }}
        .explanation pre {{ background-color: #2d2d2d; color: #f1f1f1; padding: 1em; border-radius: 5px; overflow-x: auto; }}
        .explanation pre code {{ background-color: transparent; padding: 0; }}
        .explanation blockquote {{ padding: 0 1em; color: #6a737d; border-left: 0.25em solid #dfe2e5; }}
        .timestamp {{ font-size: 0.8rem; color: #888; margin-right: 12px; font-weight: normal; background-color: #f0f0f0; padding: 2px 6px; border-radius: 4px; }}
        .original-text {{ font-weight: bold; }}
    </style>
</head>
<body>
<div id=\"main-container\">
" , title);
    file.write_all(head_html.as_bytes())?;

    if let Some(url_str) = youtube_url {
        writeln!(file, "    <div id=\"video-container\">")?;
        let thumbnail_path = Path::new("thumbnail.png");
        if thumbnail_path.exists() {
            writeln!(file, "        <img src=\"thumbnail.png\" alt=\"Video Thumbnail\">")?;
        }
        if let Some(embed_url) = get_youtube_embed_url(url_str) {
            writeln!(
                file,
                "        <iframe src=\"{}\" allow=\"accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture\" allowfullscreen></iframe>",
                embed_url
            )?;
        }
        writeln!(file, "    </div>")?;
    }

    writeln!(file, "    <div id=\"results-wrapper\">")?;
    writeln!(file, "        <div id=\"container\">")?;
    writeln!(file, "            <h1>{}</h1>", title)?;
    writeln!(file, "            <p style=\"text-align:center;\">各英文をクリックすると、解説が開閉します。</p>
            <div id=\"results\">")?;

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);

    for item in results {
        let parser = Parser::new_ext(&item.explanation, options);
        let mut explanation_html = String::new();
        html::push_html(&mut explanation_html, parser);

        writeln!(file, "                <div class=\"entry\">")?;
        writeln!(
            file,
            "                    <div class=\"sentence\"><span class=\"timestamp\">{}</span><span class=\"original-text\">{}</span></div>",
            escape_html(&item.timestamp),
            escape_html(&item.original_sentence)
        )?;
        writeln!(file, "                    <div class=\"explanation\">{}</div>", explanation_html)?;
        writeln!(file, "                </div>")?;
    }

    let foot_html = r#"
            </div>
        </div>
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