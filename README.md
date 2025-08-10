# SRT Grammar Analyzer with ollama

## 概要

このプロジェクトは、SRT字幕ファイルまたはYouTube動画の字幕からテキストを抽出し、各文の文法をollamaを介して解析し、その結果をインタラクティブなHTMLビューアとして出力するRustアプリケーションです。

## 機能

-   SRTファイルからタイムスタンプや行番号を除去し、整形されたテキストを抽出します。
-   YouTube動画から字幕をダウンロードする機能を追加しました。
-   抽出された各文をollamaのAPIに送信し、文法解説（日本語）を取得します。
-   取得した英文と解説をJSON形式で保存します。
-   JSON形式の解析結果を、クリックで解説の表示/非表示を切り替えられるインタラクティブなHTMLページとして出力します。

## 必要要件

-   **Rust**: プログラムのビルドと実行にはRustとCargoが必要です。
    -   [Rust公式ウェブサイト](https://www.rust-lang.org/tools/install) からインストールできます。
-   **ollama**: 文法解析にはollamaが必要です。
    -   `curl -fsSL https://ollama.com/install.sh | sh`でインストールしてください。
    -   ollamaでモデルをロードし、APIサーバー（通常 `http://localhost:11434`）を起動しておく必要があります（上記でインストールすると自動的にサービスが登録されてるはず）。
-   **yt-dlp**: YouTube字幕のダウンロード機能を使用する場合に必要です。
    -   [yt-dlp GitHubリポジトリ](https://github.com/yt-dlp/yt-dlp) からインストールできます（例: `pip install yt-dlp` または `sudo apt install yt-dlp`）。

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
    srtgram -l <ローカルSRTファイルのパス>
    ```
    例:
    ```bash
    srtgram -l captions.srt
    ```
    (もし `srt_processor` ディレクトリから実行する場合は、`../captions.srt` のように相対パスを指定します。)

    ### YouTube動画の字幕を処理する場合 (`-y` オプション)

    ```bash
    srtgram -y <YouTube動画のURL>
    ```
    例:
    ```bash
    srtgram -y https://www.youtube.com/watch?v=zYKJdzyAviE
    ```

### 出力ファイル

プログラムの実行後、入力SRTファイル（またはダウンロードされた字幕ファイル）と同じディレクトリに以下のファイルが生成されます。

-   `<input_file_base_name>.txt`: 整形された字幕テキスト（各文が1行）。
-   `<input_file_base_name>.analysis.json`: ollamaによる文法解析結果を格納したJSONファイル。
-   `<input_file_base_name>.analysis.html`: 解析結果をインタラクティブに表示するHTMLビューア。

## プロジェクト構造

-   `src/main.rs`: コマンドラインインターフェースのエントリーポイント。`parser`, `analyzer`, `html_generator`, `youtube_downloader` モジュールを統合し、全体の処理フローを管理します。
-   `src/parser.rs`: SRTファイルを読み込み、整形されたテキストを抽出する機能を提供します。
-   `src/analyzer.rs`: 整形されたテキストをollama APIに送信し、文法解説を取得してJSONを生成する機能を提供します。
-   `src/html_generator.rs`: JSON形式の解析結果を読み込み、MarkdownをHTMLに変換してインタラクティブなHTMLビューアを生成する機能を提供します。
-   **`src/youtube_downloader.rs`**: YouTube動画から字幕をダウンロードする機能を提供します。

## 今後の改善点

-   ollama APIのエンドポイントやモデル名をコマンドライン引数で指定できるようにする。
-   エラーハンドリングの強化。
-   より高度なテキスト前処理（例: 句読点の正規化）。
-   UIの改善（例: 検索、フィルタリング機能）。

## ライセンス

MIT License