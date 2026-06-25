# ocrs-cjk

> **Pure Rust CJK OCRエンジン — スキャンPDFを検索可能PDFに変換。PaddleOCR検出・ONNX認識・WASM対応・オフラインファースト。**
> [ocrs](https://github.com/robertknight/ocrs) のフォーク。CJK（中国語・日本語・韓国語）に完全対応：PaddleOCRモデルのフルサポート、CJK対応セグメンテーション、コンフィデンススコア、検索可能PDF出力（ToUnicode CMap）、構造化出力フォーマット（hOCR、ALTO XML、JSON）。C/C++依存ゼロ、ネイティブ `wasm32-unknown-unknown` 対応。
> アップストリーム（`robertknight/ocrs`）の変更は定期的にマージされます。

**言語:** [English](README.md) | [日本語](README_ja.md) | [简体中文](README_zh.md) | [繁體中文](README_zh-tw.md) | [한국어](README_kr.md)

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
- **PaddleOCRモデル完全対応**: 検出（DBモデル、3ch RGB、動的サイズ）と認識（PP-OCRv5 ONNX）をモデルメタデータから自動判定
- **構造化出力**: `--hocr`（hOCR HTML）、`--alto`（ALTO v4 XML）、`-j`（JSON）— すべて単語単位バウンディングボックスとコンフィデンスを含む
- **コンフィデンススコア**: `TextItem::confidence()` で文字・単語単位の認識信頼度を取得可能
- **CJK対応セグメンテーション**: `TextLine::segments()` がスペースなしでスクリプト境界（ラテン ↔ CJK）で分割
- **アルファベットヘルパー**: `hiragana()`, `katakana()`, `cjk_unified()`, `hangul()`, `cjk_alphabet()`, `cjk_alphabet_chars()`
- **UTF-8安全**: すべての文字列操作は `char_indices`・`chars` などのchar境界対応メソッドを使用（バイトスライスなし）
- **WASM完全対応**: `recognize_text` の rayon パニックを修正済み — `wasm32-unknown-unknown` でフルパイプラインが動作

アップストリームの ocrs はラテン文字のみ対応しています。元の言語サポートロードマップは [upstream issue](https://github.com/robertknight/ocrs/issues/8) を参照してください。

## CJK認識の動作確認

`ocrs-cli/test-data/cjk/` のテスト画像を PP-OCRv5 認識モデル + 付属検出モデルで処理した結果：

| 言語 | テスト画像 | OCR出力 | 結果 |
|------|-----------|--------|------|
| 日本語 | `test_ja.png`（600×80、合成画像） | `東京オリンピック2024` | PASS |
| 中国語 | `test_zh.png`（600×80、合成画像） | `人工智能技2024` | PASS |

再現方法：
```sh
./tools/test-e2e-cjk.sh models/
```

## 他のOCRソリューションとの比較

| ソリューション | ランタイム | CJK (JA/ZH/KO) | ネイティブWASM | C/C++不要 | オフライン | hOCR/ALTO | ライセンス |
|---|---|---|---|---|---|---|---|
| **ocrs-cjk**（このフォーク） | Pure Rust | Yes / Yes / Yes | Yes | Yes | Yes | Yes | Apache-2.0 / MIT |
| [ocrs](https://github.com/robertknight/ocrs)（upstream） | Pure Rust | No（ラテン文字のみ） | Yes | Yes | Yes | No | Apache-2.0 / MIT |
| [Tesseract](https://github.com/tesseract-ocr/tesseract) | C++（`tesseract-sys` FFI） | Yes / Yes / Yes | 部分的¹ | No | Yes | Yes | Apache-2.0 |
| [PaddleOCR](https://github.com/PaddlePaddle/PaddleOCR) | Python / C++ | Yes / Yes / Yes | 部分的² | No | Yes | No | Apache-2.0 |
| [EasyOCR](https://github.com/JaidedAI/EasyOCR) | Python / PyTorch | Yes / Yes / Yes | No | No | Yes | No | Apache-2.0 |
| [RapidOCR](https://github.com/RapidAI/RapidOCR) | Python / ONNX | Yes / Yes / Unknown | No | No | Yes | No | Apache-2.0 |
| [manga-ocr](https://github.com/kha-white/manga-ocr) | Python / PyTorch | 日本語のみ | 非公式³ | 任意 | Yes | No | Apache-2.0 |

¹ `tesseract-wasm` は別プロジェクト（JS）。CJK tessdata の別途ロードが必要。`wasm32-unknown-unknown` ネイティブではない。  
² PaddleOCR には JS ブラウザ SDK があるが、Rust ネイティブ WASM ではない。  
³ コミュニティ製 Chrome 拡張のみ。プロダクション品質ではない。

**ocrs-cjk は Pure Rust・C/C++ 依存ゼロ・ネイティブ `wasm32-unknown-unknown`・完全オフライン・CJK 認識をすべて兼ね備えた唯一のソリューションです。**

### 精度の参考値（PP-OCRv5、PaddleOCR 内部ベンチマーク）
- 簡体字中国語（印刷）: 認識率 約90%
- 日本語: 認識率 約74%
- ocrs-cjk 実測 CER（合成画像）: ひらがな・カタカナ・漢字・簡体字・混合 CJK+Latin・長文 = 0%; 繁体字稀少字形（`臺灣` など）= 約67%

## 外部モデルを使ったCJK OCR

CJK OCRを実際に動かすには、以下の2つのモデルが必要です：

| ステージ | 役割 | 状態 |
|---|---|---|
| **検出モデル** | 画像内のテキスト領域を見つける | Yes PaddleOCR DB検出モデル対応済み（3ch RGB、動的サイズ、ImageNet正規化 — モデルメタデータから自動検出）。付属のラテン文字学習済みモデルもフォールバックとして使用可能 |
| **認識モデル** | 検出領域の文字を読む | Yes PaddleOCR ONNX形式に対応済み（3チャンネル入力・バッチファースト出力を自動検出） |

このリポジトリにはCJK学習済みモデルは含まれていません。別途入手する必要があります。

### ステップ1 — 認識モデルのダウンロード

[PP-OCRv5](https://github.com/PaddlePaddle/PaddleOCR) は簡体字中国語・繁体字中国語・日本語・英語を1つのモデルでサポートしています。Hugging Face にONNX変換済みファイルがあります。以下のPythonスクリプトで一度だけ実行してください：

```sh
pip install huggingface-hub pyyaml
```

```python
from huggingface_hub import hf_hub_download

hf_hub_download(
    repo_id="marsena/paddleocr-onnx-models",
    filename="PP-OCRv5_server_rec_infer.onnx",
    local_dir="./models",
)
hf_hub_download(
    repo_id="marsena/paddleocr-onnx-models",
    filename="PP-OCRv5_server_rec_infer.yml",
    local_dir="./models",
)
```

### ステップ2 — 文字辞書の取り出し

認識モデルはラベルのインデックスを出力します。`OcrEngineParams::alphabet` に対応する文字リストを渡す必要があります。以下のスクリプトで `alphabet.txt` を生成してください：

```python
import yaml

with open("models/PP-OCRv5_server_rec_infer.yml") as f:
    cfg = yaml.safe_load(f)

chars = cfg["PostProcess"]["character_dict"]

# 一部のエントリ（国旗絵文字  など）は2つのUnicodeコードポイントを持ちます。
# ocrs は1ラベル = 1文字として扱うため、最初のコードポイントに丸めます。
# これらのエントリはCJK OCRの出力には現れないため実用上の問題はありません。
fixed = [c[0] if len(c) > 1 else c for c in chars]

# PaddleOCRはデフォルトでスペースラベルを末尾に追加します。
fixed.append(" ")

with open("models/alphabet.txt", "w", encoding="utf-8") as f:
    f.write("".join(fixed))

print(f"{len(fixed)} 文字を models/alphabet.txt に書き出しました")
```

> **確認済み:** PP-OCRv5 は辞書 18383 文字 + スペース = 18384 文字です。
> `18384 + 1 (CTCブランク) = 18385` がモデルの出力次元と一致します。

### ステップ3 — CLIで実行

ONNXサポートを有効にしてビルドし、`--alphabet-file` で辞書ファイルを渡します（大きな文字セットをシェル引数で渡すとエスケープの問題が起きるため）：

```sh
cargo build -p ocrs-cli --release --features onnx

./target/release/ocrs \
  --rec-model  models/PP-OCRv5_server_rec_infer.onnx \
  --alphabet-file models/alphabet.txt \
  image.png
```

### ステップ4 — Rustライブラリとしての使い方

```rust
use ocrs::{OcrEngine, OcrEngineParams};
use rten::Model;

// PaddleOCR認識モデルを読み込む（チャンネル数と出力レイアウトは
// モデルのinput_shapeから自動検出されます）
let rec_model = Model::load_file("models/PP-OCRv5_server_rec_infer.onnx")?;

// ステップ2で生成した辞書ファイルを読み込む
let alphabet = std::fs::read_to_string("models/alphabet.txt")?;

let engine = OcrEngine::new(OcrEngineParams {
    recognition_model: Some(rec_model),
    // detection_model を省略すると付属のラテン文字学習済みモデルを使用します。
    alphabet: Some(&alphabet),
    ..Default::default()
})?;
```

### 既知の制限事項

- **ONNXフィーチャーフラグ**: `.onnx` ファイルを読み込むには `--features onnx` が必要です（rtenのデフォルト形式は `.rten`）。
- **アルファベット不一致**: `alphabet` 文字列がモデルの学習辞書と順番・長さが一致しない場合、認識結果が文字化けします。必ずモデルのYAML設定から取り出した辞書を使用してください。
- **検出精度**: CJK学習済みの検出モデルは同梱されていません。PaddleOCR DB検出ONNXを指定することで、デフォルトのラテン文字学習済みモデルより高精度なCJKレイアウト検出が可能です。

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

テキストをhOCR形式で抽出する（バウンディングボックスと単語ごとのコンフィデンス付き、多くのドキュメントツールで読み込み可能）：

```sh
$ ocrs image.png --hocr -o output.hocr
```

テキストをALTO XML形式で抽出する（アーカイブ標準、デジタルライブラリや文書管理システムで使用）：

```sh
$ ocrs image.png --alto -o output.xml
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
