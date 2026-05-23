# ocrs-cjk

> **This is a fork of [ocrs](https://github.com/robertknight/ocrs) focused on CJK (Chinese, Japanese, Korean) text recognition.**
> The goal is to extend ocrs with a CJK-capable alphabet, CJK-aware text segmentation, and full offline / WebAssembly compatibility — without any C/C++ dependencies (no Tesseract, no OpenCV).
> Upstream changes are periodically merged from `robertknight/ocrs`.

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
- CJK-aware text segmentation via `TextLine::segments()`
- Alphabet helpers: `hiragana()`, `katakana()`, `cjk_unified()`, `hangul()`, `cjk_alphabet()`, `cjk_alphabet_chars()`
- UTF-8 safe boundary utilities in `cjk_text` module

The upstream ocrs recognizes the Latin alphabet only. See the [upstream issue](https://github.com/robertknight/ocrs/issues/8) for the original language support roadmap.

> **WASM limitation:** `OcrEngine::recognize_text` uses `rayon` for parallelism and will panic at runtime on `wasm32-unknown-unknown`. This is an upstream issue inherited from `ocrs`. The remaining API (`detect_words`, `find_text_lines`, `cjk_text` utilities) is WASM-compatible.

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
````

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
