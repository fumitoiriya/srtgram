# SRT Grammar Analyzer with ollama

## 概要

このプロジェクトは、SRT字幕ファイルまたはYouTube動画の字幕からテキストを抽出し、各文の文法をollamaを介して解析し、その結果をインタラクティブなHTMLビューアとして出力するRustアプリケーションです。生成されるHTMLは、YouTube動画のタイトルまたはローカルファイル名をタイトルと見出しに表示します。

## 機能

-   SRTファイルからタイムスタンプや行番号を除去し、整形されたテキストを抽出します。
-   YouTube動画から字幕をダウンロードし、そのタイトルを取得する機能を追加しました。
-   抽出された各文をollamaのAPIに送信し、日本語での文法解説を取得します。
-   取得した英文と解説をJSONL形式で保存します。
-   JSONL形式の解析結果を、クリックで解説の表示/非表示を切り替えられるインタラクティブなHTMLページとして出力します。このHTMLページのタイトルと見出し（`<h1>`タグ）は、YouTube動画の場合はそのタイトル、ローカルファイルの場合はファイル名を表示します。

## 必要要件

-   **Rust**: プログラムのビルドと実行にはRustとCargoが必要です。
    -   [Rust公式ウェブサイト](https://www.rust-lang.org/tools/install) からインストールできます。
-   **ollama**: 文法解析にはollamaが必要です。
    -   `curl -fsSL https://ollama.com/install.sh | sh`でインストールしてください。
    -   ollamaでモデルをロードし、APIサーバー（通常 `http://localhost:11434`）を起動しておく必要があります。デフォルトのモデルは `gemma3:12b` ですが、`-m` オプションで変更可能です。
-   **yt-dlp**: YouTube字幕のダウンロードおよび動画タイトルの取得機能を使用する場合に必要です。
    -   [yt-dlp GitHubリポジリ](https://github.com/yt-dlp/yt-dlp) からインストールできます（例: `pip install yt-dlp` または `sudo apt install yt-dlp`）。

## セットアップ

1.  このリポジトリをクローンします。
    ```bash
    git clone <リポジトリのURL> # もしGitリポジトリなら
    cd srtgram
    ```
    (もしGitリポジトリでなければ、プロジェクトフォルダに移動します。)
2.  プロジェクトの依存関係をビルドします。
    ```bash
    cargo build --release
    cp target/release/srtgram <your bin path>
    ```

## 使い方

1.  ollamaを起動し、使用したいモデルをロードし、APIサーバー（通常 `http://localhost:11434`）を開始します。
2.  以下のコマンドを実行します。入力方法に応じて `-l` または `-y` オプションを使用してください。

    ### ローカルSRTファイルを処理する場合 (`-l` オプション)

    ```bash
    srtgram -l <ローカルSRTファイルのパス> [-m <モデル名>] [--limit <解析する文の数>]
    ```
    例:
    ```bash
    srtgram -l captions.srt -m gemma3:12b --limit 10
    ```
    (もし `srt_processor` ディレクトリから実行する場合は、`../captions.srt` のように相対パスを指定します。)

    ### YouTube動画の字幕を処理する場合 (`-y` オプション)

    ```bash
    srtgram -y <YouTube動画のURL> [-m <モデル名>] [--limit <解析する文の数>]
    ```
    例:
    ```bash
    srtgram -y https://www.youtube.com/watch?v=zYKJdzyAviE -m llama3 --limit 5
    ```

### 出力ファイル

プログラムの実行後、一意に作成された出力ディレクトリ内に以下のファイルが生成されます。

-   `sentences.json`: SRTファイルから抽出された各文とタイムスタンプを格納したJSONファイル。
-   `analysis.jsonl`: ollamaによる文法解析結果（元の文、タイムスタンプ、解説）をJSONL形式で格納したファイル。
-   `index.html`: 解析結果をインタラクティブに表示するHTMLビューア。このHTMLファイルのタイトルと見出し（`<h1>`タグ）は、YouTube動画の場合はそのタイトル、ローカルファイルの場合はファイル名を表示します。
-   `thumbnail.png`: YouTube動画の場合、動画のサムネイルが保存されます。

## プロジェクト構造

-   `src/main.rs`: コマンドラインインターフェースのエントリーポイント。`clap`クレートを使用して引数を解析し、`parser`, `analyzer`, `html_generator`, `youtube_downloader` モジュールを統合して、SRT処理とHTML生成の全体のフローを管理します。
-   `src/parser.rs`: SRTファイルを読み込み、タイムスタンプとテキストを抽出し、各文に分割します。結果は `sentences.json` として出力されます。
-   `src/analyzer.rs`: `sentences.json` を読み込み、各文をOllama API (`http://localhost:11434/api/generate`) に送信して日本語での文法解説を取得します。結果は `analysis.jsonl` として出力されます。
-   `src/html_generator.rs`: `analysis.jsonl` を読み込み、`pulldown-cmark` を使用してMarkdown形式の解説をHTMLに変換し、インタラクティブなHTMLビューア (`index.html`) を生成します。YouTube動画が処理された場合は、動画埋め込みとサムネイル表示も行います。
-   `src/youtube_downloader.rs`: `yt-dlp` ツールを使用してYouTube動画の字幕をダウンロードし、動画のタイトルを取得する機能を提供します。

## ライセンス

MIT License