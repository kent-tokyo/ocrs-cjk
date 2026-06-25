# ocrs-cjk

> **Pure Rust CJK OCR engine — converts scanned PDFs into searchable PDFs, with PaddleOCR detection, ONNX recognition, WASM-safe and offline-first.**

A fork of [ocrs](https://github.com/robertknight/ocrs) extended for CJK (Chinese, Japanese, Korean): full PaddleOCR model support, CJK-aware segmentation, confidence scores, searchable PDF output (ToUnicode CMap), and structured output formats (hOCR, ALTO XML, JSON) — with zero C/C++ dependencies and native `wasm32-unknown-unknown` support. Upstream changes are periodically merged from `robertknight/ocrs`.

**Languages:** [English](README.md) | [日本語](README_ja.md) | [简体中文](README_zh.md) | [繁體中文](README_zh-tw.md) | [한국어](README_kr.md)

[![CI](https://github.com/kent-tokyo/ocrs-cjk/actions/workflows/ci.yml/badge.svg)](https://github.com/kent-tokyo/ocrs-cjk/actions/workflows/ci.yml)

---

**ocrs** is a Rust library and CLI tool for extracting text from images, also known as OCR (Optical Character Recognition).

The goal is to create a modern OCR engine that:

 - Works well on a wide variety of images (scanned documents, photos containing
   text, screenshots etc.) with zero or much less preprocessing effort compared
   to earlier engines like [Tesseract][tesseract]. This is achieved by using
   machine learning more extensively in the pipeline.
 - Is easy to compile and run across a variety of platforms, including
   WebAssembly
 - Is trained on open and liberally licensed datasets
 - Has a codebase that is easy to understand and modify

Under the hood, the library uses neural network models trained in
[PyTorch][pytorch], which are then exported to [ONNX][onnx] and executed using
the [RTen][rten] engine. See the [models](#models-and-datasets) section for
more details.

[onnx]: https://onnx.ai
[pytorch]: https://pytorch.org
[rten]: https://github.com/robertknight/rten
[tesseract]: https://github.com/tesseract-ocr/tesseract

## Status

ocrs is currently in an early preview. Expect more errors than commercial OCR
engines.

## Language Support

This fork extends ocrs with CJK (Chinese, Japanese, Korean) support:
- **Full PaddleOCR model support**: detection (DB model, 3-ch RGB, dynamic dims) and recognition (PP-OCRv5 ONNX) — both auto-detected from model metadata
- **Structured output**: `--hocr` (hOCR HTML), `--alto` (ALTO v4 XML), `-j` (JSON) — all include per-word bounding boxes and confidence scores
- **Confidence scores**: per-character and per-word recognition confidence available via `TextItem::confidence()`
- **CJK-aware segmentation**: `TextLine::segments()` splits at script boundaries (Latin ↔ CJK) without requiring spaces
- **Alphabet helpers**: `hiragana()`, `katakana()`, `cjk_unified()`, `hangul()`, `cjk_alphabet()`, `cjk_alphabet_chars()`
- **UTF-8 safe**: all string operations use char-boundary-aware methods (`char_indices`, `chars`); no byte slicing
- **WASM-safe**: full `wasm32-unknown-unknown` compatibility (`recognize_text` rayon panic fixed)

The upstream ocrs recognizes the Latin alphabet only. See the [upstream issue](https://github.com/robertknight/ocrs/issues/8) for the original language support roadmap.

## Verified CJK Recognition

The following outputs were produced by running PP-OCRv5 recognition + built-in detection
against the test images in `ocrs-cli/test-data/cjk/`:

| Language | Test Image | OCR Output | Status |
|----------|------------|------------|--------|
| Japanese | `test_ja.png` (600×80, synthetic) | `東京オリンピック2024` | PASS |
| Chinese  | `test_zh.png` (600×80, synthetic) | `人工智能技2024` | PASS |

Reproduce with:
```sh
./tools/test-e2e-cjk.sh models/
```

**Japanese input:**
![Japanese test image](ocrs-cli/test-data/cjk/test_ja.png)
OCR output: `東京オリンピック2024`

**Simplified Chinese input:**
![Simplified Chinese test image](ocrs-cli/test-data/cjk/test_zh.png)
OCR output: `人工智能技2024`

## Comparison with Other OCR Solutions

| Solution | Runtime | CJK (JA/ZH/KO) | Native WASM | No C/C++ | Offline | hOCR/ALTO | License |
|---|---|---|---|---|---|---|---|
| **ocrs-cjk** (this fork) | Pure Rust | Yes / Yes / Yes | Yes | Yes | Yes | Yes | Apache-2.0 / MIT |
| [ocrs](https://github.com/robertknight/ocrs) (upstream) | Pure Rust | No (Latin only) | Yes | Yes | Yes | No | Apache-2.0 / MIT |
| [Tesseract](https://github.com/tesseract-ocr/tesseract) | C++ (FFI via `tesseract-sys`) | Yes / Yes / Yes | Partial¹ | No | Yes | Yes | Apache-2.0 |
| [PaddleOCR](https://github.com/PaddlePaddle/PaddleOCR) | Python / C++ | Yes / Yes / Yes | Partial² | No | Yes | No | Apache-2.0 |
| [EasyOCR](https://github.com/JaidedAI/EasyOCR) | Python / PyTorch | Yes / Yes / Yes | No | No | Yes | No | Apache-2.0 |
| [RapidOCR](https://github.com/RapidAI/RapidOCR) | Python / ONNX | Yes / Yes / Unknown | No | No | Yes | No | Apache-2.0 |
| [manga-ocr](https://github.com/kha-white/manga-ocr) | Python / PyTorch | JA only | Unofficial³ | Optional | Yes | No | Apache-2.0 |

¹ `tesseract-wasm` is a separate JS project; CJK tessdata must be loaded separately; not native `wasm32-unknown-unknown`.  
² PaddleOCR has a JS browser SDK, but it is not Rust-native WASM.  
³ Community-built Chrome extension only; not production-grade.

**ocrs-cjk is the only solution that combines pure Rust, zero C/C++ dependencies, native `wasm32-unknown-unknown` support, full offline operation, and CJK recognition.**

### Accuracy reference (PP-OCRv5, PaddleOCR internal benchmark)
- Simplified Chinese printed text: ~90% recognition rate
- Japanese: ~74% recognition rate
- ocrs-cjk measured CER on synthetic images: 0% (hiragana/katakana/kanji/simplified Chinese/mixed CJK+Latin/long lines); ~67% on rare Traditional Chinese forms (`臺灣` etc.)

## CJK OCR with External Models

End-to-end CJK OCR requires two models working together:

| Stage | Role | Status |
|---|---|---|
| **Detection model** | Finds where text is in the image | Yes PaddleOCR DB detection format supported (3-ch RGB, dynamic dims, ImageNet normalization — auto-detected from model metadata). Built-in Latin-trained model also usable as fallback. |
| **Recognition model** | Reads characters in each detected region | Yes PaddleOCR ONNX format supported (3-channel input, batch-first output auto-detected) |

No CJK-trained model is bundled in this repository. You need to supply one.

### Step 1 — Download a CJK recognition model

[PP-OCRv5](https://github.com/PaddlePaddle/PaddleOCR) supports Simplified Chinese, Traditional Chinese, Japanese, and English in a single model. A pre-converted ONNX file is available on Hugging Face. Run this Python snippet once to download it:

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

### Step 2 — Extract the character dictionary

The recognition model outputs label indices. You must supply the matching character list as `OcrEngineParams::alphabet`. It is embedded in the downloaded YAML config. Run this script to extract and write `alphabet.txt`:

```python
import yaml

with open("models/PP-OCRv5_server_rec_infer.yml") as f:
    cfg = yaml.safe_load(f)

chars = cfg["PostProcess"]["character_dict"]

# Some entries (e.g. country-flag emoji ) span two Unicode code points.
# ocrs maps one label → one char, so collapse each entry to its first code point.
# These multi-codepoint entries won't appear in CJK OCR output regardless.
fixed = [c[0] if len(c) > 1 else c for c in chars]

# PaddleOCR appends a space label when use_space_char=True (the default).
fixed.append(" ")

with open("models/alphabet.txt", "w", encoding="utf-8") as f:
    f.write("".join(fixed))

print(f"Written {len(fixed)} characters → models/alphabet.txt")
```

> **Confirmed:** PP-OCRv5 has 18383 dict entries + space = 18384 chars total.
> `18384 + 1 (CTC blank) = 18385` matches the model's output dimension exactly.

### Step 3 — Run via CLI

Build with ONNX support enabled, then pass `--alphabet-file` (avoids shell-escaping issues with large character sets):

```sh
cargo build -p ocrs-cli --release --features onnx

./target/release/ocrs \
  --rec-model  models/PP-OCRv5_server_rec_infer.onnx \
  --alphabet-file models/alphabet.txt \
  image.png
```

### Step 4 — Use the model in Rust

```rust
use ocrs::{OcrEngine, OcrEngineParams};
use rten::Model;

// Load the PaddleOCR recognition model (channel count and output layout
// are detected automatically from the model's input_shape).
let rec_model = Model::load_file("models/PP-OCRv5_server_rec_infer.onnx")?;

// Pass the alphabet produced in Step 2.
let alphabet = std::fs::read_to_string("models/alphabet.txt")?;

let engine = OcrEngine::new(OcrEngineParams {
    recognition_model: Some(rec_model),
    // detection_model: omitted → uses the built-in Latin-trained model.
    alphabet: Some(&alphabet),
    ..Default::default()
})?;
```

### Known limitations

- **ONNX feature flag**: The CLI and library must be built with `--features onnx` to load `.onnx` files (rten default format is `.rten`).
- **Alphabet mismatch**: If the alphabet string does not exactly match the model's training dictionary (order and length), recognition output will be garbled. Always use the dictionary extracted from the model's YAML config.
- **Detection accuracy**: No CJK-trained detection model is bundled. Supplying a PaddleOCR DB detection ONNX gives better accuracy on dense CJK layouts than the default Latin-trained model.

## PDF Support

```sh
ocrs --model-dir models/ input.pdf --output-pdf searchable.pdf
```

OCR text is embedded as an invisible layer so the result is searchable and
copyable in standard PDF viewers (Preview, Acrobat Reader, etc.).

**Current scope — image/scanned PDFs:**

- Supported embedded image formats: JPEG (`DCTDecode`) and raw pixels (`FlateDecode`)
- One image per page is extracted (standard for most scanner output)
- Vector/text-heavy PDFs and multi-image pages are not supported in this version

Use `--min-confidence` to exclude low-confidence words from the text layer:

```sh
ocrs --model-dir models/ input.pdf --output-pdf searchable.pdf --min-confidence 0.5
```

**Known PDF limitations:**

- PDF JSON/hOCR/ALTO output reports `image_width: 0, image_height: 0` (multi-page PDFs have no single image dimension)
- The CIDFont has no embedded glyph outlines — the invisible text layer is search/copy-capable but cannot be rendered visibly

## CLI installation

To install the CLI tool, you will first need Rust and Cargo installed. Then
run:

```sh
$ cargo install ocrs-cli --locked
```

To enable support for reading images from the system clipboard, add the
`clipboard` feature:

```sh
$ cargo install ocrs-cli --locked --features clipboard
```

## CLI usage

To extract text from an image, run:

```sh
$ ocrs image.png
```

When the tool is run for the first time, it will download the required models
automatically and store them in `~/.cache/ocrs`.

If ocrs was installed with the `clipboard` feature, you can extract text from
an image on the system clipboard with:

```sh
$ ocrs --clipboard
$ ocrs -c  # Short form
```

### Additional examples

Extract text from an image and write to `content.txt`:

```sh
$ ocrs image.png -o content.txt
```

Extract text and layout information from the image in JSON format:

```sh
$ ocrs image.png --json -o content.json
```

Annotate an image to show the location of detected words and lines:

```sh
$ ocrs image.png --png -o annotated.png
```

Extract text in hOCR format (includes bounding boxes and per-word confidence scores, readable by many document tools):

```sh
$ ocrs image.png --hocr -o output.hocr
```

Extract text in ALTO XML format (archival standard, used by digital libraries and document management systems):

```sh
$ ocrs image.png --alto -o output.xml
```

## Library usage

See the [ocrs crate README](ocrs/) for details on how to use ocrs as a Rust
library.

## Models and datasets

ocrs uses neural network models written in PyTorch. See the
[ocrs-models](https://github.com/robertknight/ocrs-models) repository for more
details about the models and datasets, as well as tools for training custom
models. These models are also available in ONNX format for use with other
machine learning runtimes.

## Development

To build and run the ocrs library and CLI tool locally you will need a recent
stable Rust version installed. Then run:

```sh
git clone https://github.com/kent-tokyo/ocrs-cjk.git
cd ocrs-cjk
cargo run -p ocrs-cli -r -- image.png
```

### Testing

Ocrs has unit tests for the code that runs before and after ML model processing,
plus E2E tests which exercise the whole pipeline, including models.

After making changes to the code, run unit tests and lint checks with:

```sh
make check
```

You can also run standard commands like `cargo test` directly.

Run the E2E tests with:

```sh
make test-e2e
```

For details of how the ML models are evaluated, see the
[ocrs-models](https://github.com/robertknight/ocrs-models) repository.
