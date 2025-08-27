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
            .map(|match_| format!("https://www.youtube.com/embed/{}?enablejsapi=1", match_.as_str()))
    })
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&#39;")
}

fn srt_time_to_seconds(time_str: &str) -> f64 {
    let parts: Vec<&str> = time_str.split(&[':', ','][..]).collect();
    if parts.len() == 4 {
        let h: f64 = parts[0].parse().unwrap_or(0.0);
        let m: f64 = parts[1].parse().unwrap_or(0.0);
        let s: f64 = parts[2].parse().unwrap_or(0.0);
        let ms: f64 = parts[3].parse().unwrap_or(0.0);
        h * 3600.0 + m * 60.0 + s + ms / 1000.0
    } else {
        0.0
    }
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

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);

    let entries_html: String = results
        .iter()
        .map(|item| {
            let parser = Parser::new_ext(&item.explanation, options);
            let mut explanation_html = String::new();
            html::push_html(&mut explanation_html, parser);
            let timestamp_sec = srt_time_to_seconds(&item.timestamp);

            format!(
                r###"                <div class="entry" data-timestamp-sec="{}">
                    <div class="sentence"><span class="timestamp">{}</span><span class="original-text">{}</span></div>
                    <div class="explanation">{}</div>
                </div>"###,
                timestamp_sec,
                escape_html(&item.timestamp),
                escape_html(&item.original_sentence),
                explanation_html
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let video_container_html = if let Some(url_str) = youtube_url {
        let iframe_html = if let Some(embed_url) = get_youtube_embed_url(url_str) {
            format!(
                r###"        <iframe id="youtube-player" src="{}" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>"###,
                embed_url
            )
        } else {
            String::from("")
        };
        format!(
            r###"    <div id="video-container">
{}
    </div>"###,
            iframe_html
        )
    } else {
        String::from("")
    };

    let full_html = format!(
        r###"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        html, body {{
            height: 100%;
            margin: 0;
            overflow: hidden;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            color: #333;
        }}
        #main-container {{
            display: flex;
            flex-direction: column;
            height: 100vh;
        }}
        #video-container {{
            display: flex;
            justify-content: center;
            align-items: center;
            flex-shrink: 0;
            background: #000;
            padding: 10px;
            gap: 10px;
        }}
        #video-container iframe {{
            width: 100%;
            max-width: 800px;
            aspect-ratio: 16 / 9;
            border: none;
        }}
        #video-container img {{
            max-width: 240px;
            max-height: 135px;
            object-fit: cover;
            border-radius: 8px;
        }}
        #results-wrapper {{
            flex-grow: 1;
            overflow-y: auto;
            background-color: #f4f4f9;
            scroll-behavior: smooth;
        }}
        #container {{
            max-width: 800px;
            margin: 0 auto;
            background-color: #fff;
            padding: 25px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            margin-top: 20px;
            margin-bottom: 20px;
        }}
        h1 {{
            color: #2c3e50;
            text-align: center;
        }}
        .entry {{
            border-bottom: 1px solid #eee;
            padding: 15px 5px;
            transition: background-color 0.3s;
            border-radius: 5px;
        }}
        .entry:last-child {{
            border-bottom: none;
        }}
        .entry.active-sentence {{
            background-color: #eaf4ff;
        }}
        .sentence {{
            font-weight: bold;
            font-size: 1.2em;
            color: #34495e;
            margin-bottom: 10px;
            cursor: pointer;
            user-select: none;
            position: relative;
            padding-left: 20px;
        }}
        .sentence::before {{
            content: '▶';
            position: absolute;
            left: 0;
            top: 5px;
            font-size: 0.8em;
            color: #95a5a6;
            transition: transform 0.2s;
        }}
        .sentence.active::before {{
            transform: rotate(90deg);
        }}
        .explanation {{
            display: none;
            margin-top: 10px;
            background-color: #f8f9fa;
            padding: 15px;
            border-radius: 5px;
            border: 1px solid #ddd;
            word-wrap: break-word;
        }}
        .explanation.visible {{
            display: block;
        }}
        .explanation h1, .explanation h2, .explanation h3 {{ color: #2c3e50; margin-top: 1em; margin-bottom: 0.5em; border-bottom: 1px solid #eaecef; padding-bottom: 0.3em; }}
        .explanation p {{ margin-top: 0; margin-bottom: 1em; }}
        .explanation ul, .explanation ol {{ padding-left: 2em; }}
        .explanation li {{ margin-bottom: 0.5em; }}
        .explanation code {{ background-color: #e1e4e8; padding: .2em .4em; margin: 0; font-size: 85%; border-radius: 3px; font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, Courier, monospace; }}
        .explanation pre {{ background-color: #2d2d2d; color: #f1f1f1; padding: 1em; border-radius: 5px; overflow-x: auto; }}
        .explanation pre code {{ background-color: transparent; padding: 0; }}
        .explanation blockquote {{ padding: 0 1em; color: #6a737d; border-left: 0.25em solid #dfe2e5; }}
        .timestamp {{ font-size: 0.8rem; color: #888; margin-right: 12px; font-weight: normal; background-color: #f0f0f0; padding: 2px 6px; border-radius: 4px; }}
        .original-text {{ font-weight: bold; }}
    </style>
</head>
<body>
<div id="main-container">
{video_container}
    <div id="results-wrapper">
        <div id="container">
            <h1>{title}</h1>
            <p style="text-align:center;">各英文をクリックすると、解説が開閉します。</p>
            <div id="results">
{entries}
            </div>
        </div>
    </div>
</div>
<script>
    // 1. YouTube Player APIのスクリプトを非同期で読み込む
    var tag = document.createElement('script');
    tag.src = "https://www.youtube.com/iframe_api";
    var firstScriptTag = document.getElementsByTagName('script')[0];
    firstScriptTag.parentNode.insertBefore(tag, firstScriptTag);

    var player;
    var timeUpdater;
    var sentenceEntries = [];
    var lastActiveEntry = null;

    // 2. APIが読み込まれた後に呼ばれるコールバック関数
    function onYouTubeIframeAPIReady() {{
        var playerElement = document.getElementById('youtube-player');
        if (!playerElement) return;

        player = new YT.Player('youtube-player', {{
            events: {{
                'onReady': onPlayerReady,
                'onStateChange': onPlayerStateChange
            }}
        }});
    }}

    // 3. プレイヤーの準備ができたときに呼ばれる
    function onPlayerReady(event) {{
        // 全ての文章要素とそのタイムスタンプを収集
        document.querySelectorAll('.entry[data-timestamp-sec]').forEach(entry => {{
            sentenceEntries.push({{ 
                element: entry,
                time: parseFloat(entry.getAttribute('data-timestamp-sec'))
            }});
        }});

        // クリックで文章の解説を開閉する従来の機能
        document.querySelectorAll('.sentence').forEach(sentence => {{
            sentence.addEventListener('click', () => {{
                sentence.classList.toggle('active');
                const explanation = sentence.nextElementSibling;
                if (explanation) {{
                    explanation.classList.toggle('visible');
                }}
            }});
        }});
    }}

    // 4. プレイヤーの状態が変わったときに呼ばれる
    function onPlayerStateChange(event) {{
        if (event.data == YT.PlayerState.PLAYING) {{
            // 再生が始まったら、定期的に時間をチェックするタイマーを開始
            timeUpdater = setInterval(updateActiveSentence, 500);
        }} else {{
            // 停止、一時停止、終了した場合はタイマーを停止
            clearInterval(timeUpdater);
        }}
    }}

    // 5. 現在の再生時間に基づいてアクティブな文章を更新する関数
    function updateActiveSentence() {{
        if (!player || typeof player.getCurrentTime !== 'function') return;

        const currentTime = player.getCurrentTime();
        let activeEntry = null;

        // 現在時刻に最も近い、過去のタイムスタンプを持つ文章を探す
        for (let i = sentenceEntries.length - 1; i >= 0; i--) {{
            if (currentTime >= sentenceEntries[i].time - 0.5) {{ // 0.5秒早くハイライト
                activeEntry = sentenceEntries[i];
                break;
            }}
        }}

        if (activeEntry && activeEntry !== lastActiveEntry) {{
            // 他のすべてのアクティブクラスを削除
            document.querySelectorAll('.entry.active-sentence').forEach(entry => {{
                entry.classList.remove('active-sentence');
            }});

            // 新しい文章をアクティブにする
            activeEntry.element.classList.add('active-sentence');
            
            // アクティブな文章が画面内に表示されるようにスクロール
            activeEntry.element.scrollIntoView({{
                behavior: 'smooth',
                block: 'center'
            }});

            lastActiveEntry = activeEntry;
        }}
    }}
</script>
</body>
</html>"###,
        title = escape_html(title),
        video_container = video_container_html,
        entries = entries_html
    );

    file.write_all(full_html.as_bytes())?;

    println!("Successfully generated interactive HTML file at {}", output_path.display());

    Ok(())
}
