# ocrs-cjk

> **このリポジトリは [ocrs](https://github.com/robertknight/ocrs) のフォークです。CJK（中国語・日本語・韓国語）のテキスト認識に特化しています。**
> CJK対応アルファベット、CJK文字列セグメンテーション、そして完全オフライン / WebAssembly 動作を目標とし、C/C++依存（Tesseract、OpenCVなど）を一切持ちません。
> アップストリーム（`robertknight/ocrs`）の変更は定期的にマージされます。

---

**ocrs** は、画像からテキストを抽出するRustライブラリおよびCLIツールです（光学文字認識 / OCR）。

目指すのは、以下の特性を持つモダンなOCRエンジンです：

 - スキャン文書・写真・スクリーンショットなど多様な画像に対し、[Tesseract][tesseract] 等の従来エンジンよりも前処理なしで高精度に動作する（機械学習をパイプライン全体で積極的に活用）
 - WebAssembly を含む多様なプラットフォームで容易にコンパイル・実行できる
 - オープンで自由なライセンスのデータセットで学習されている
 - コードベースが理解・改変しやすい

内部的には [PyTorch][pytorch] で学習したニューラルネットワークモデルを [ONNX][onnx] 形式でエクスポートし、[RTen][rten] エンジンで実行しています。詳細は[モデルとデータセット](#モデルとデータセット)を参照してください。

[onnx]: https://onnx.ai
[pytorch]: https://pytorch.org
[rten]: https://github.com/robertknight/rten
[tesseract]: https://github.com/tesseract-ocr/tesseract

## ステータス

ocrs は現在アーリープレビュー段階です。商用OCRエンジンよりも誤認識が多い場合があります。

## 言語サポート

このフォークはCJK（中国語・日本語・韓国語）サポートを追加しています：
- `TextLine::segments()` によるCJK対応テキスト分割
- アルファベットヘルパー: `hiragana()`, `katakana()`, `cjk_unified()`, `hangul()`, `cjk_alphabet()`, `cjk_alphabet_chars()`
- `cjk_text` モジュールのUTF-8安全なバイト境界ユーティリティ

アップストリームの ocrs はラテン文字のみ対応しています。元の言語サポートロードマップは [upstream issue](https://github.com/robertknight/ocrs/issues/8) を参照してください。

> **WASM制限:** `OcrEngine::recognize_text` は並列処理に `rayon` を使用しており、`wasm32-unknown-unknown` ではランタイムパニックが発生します。これはアップストリームから引き継いだ既知の問題です。それ以外のAPI（`detect_words`, `find_text_lines`, `cjk_text` ユーティリティ）はWASM互換です。

## CLIのインストール

Rust と Cargo がインストールされていることを確認してから、以下を実行してください：

```sh
$ cargo install ocrs-cli --locked
```

クリップボード上の画像からテキストを抽出する機能を有効にするには、`clipboard` フィーチャーを追加します：

```sh
$ cargo install ocrs-cli --locked --features clipboard
```

## CLIの使い方

画像からテキストを抽出するには：

```sh
$ ocrs image.png
```

初回実行時、必要なモデルが自動的にダウンロードされ `~/.cache/ocrs` に保存されます。

`clipboard` フィーチャー付きでインストールした場合、クリップボード上の画像からテキストを抽出できます：

```sh
$ ocrs --clipboard
$ ocrs -c  # 短縮形
```

### 使用例

テキストを `content.txt` に書き出す：

```sh
$ ocrs image.png -o content.txt
```

テキストとレイアウト情報をJSON形式で抽出する：

```sh
$ ocrs image.png --json -o content.json
```

検出した単語・行の位置を注釈付き画像として出力する：

```sh
$ ocrs image.png --png -o annotated.png
```

## ライブラリとしての使い方

Rustライブラリとしての使い方は [ocrs クレートのREADME](ocrs/) を参照してください。

## モデルとデータセット

ocrs は PyTorch で記述されたニューラルネットワークモデルを使用しています。モデルとデータセットの詳細、およびカスタムモデルの学習ツールについては [ocrs-models](https://github.com/robertknight/ocrs-models) リポジトリを参照してください。モデルは他の機械学習ランタイムから利用できるよう ONNX 形式でも提供されています。

## 開発

ライブラリとCLIをローカルでビルド・実行するには、最新の安定版Rustが必要です：

```sh
git clone https://github.com/kent-tokyo/ocrs-cjk.git
cd ocrs-cjk
cargo run -p ocrs-cli -r -- image.png
```

### テスト

コード変更後、ユニットテストとlintチェックを実行するには：

```sh
make check
```

通常の `cargo test` コマンドも使用できます。

E2Eテストを実行するには：

```sh
make test-e2e
```

MLモデルの評価方法の詳細は [ocrs-models](https://github.com/robertknight/ocrs-models) リポジトリを参照してください。
