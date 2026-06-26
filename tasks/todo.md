# TODO

## High Priority

- [x] **rayon / WASM**: Fixed by adding `#[cfg(not(target_arch = "wasm32"))]` guards around the `rayon` and `thread_pool` imports, and adding a sequential `into_iter()` fallback path for `wasm32-unknown-unknown`. Both native and WASM builds pass.

- [x] **PaddleOCR detection model support**: Implemented `DetectorKind` enum in `detection.rs` (auto-detected from `input_shape[1]`). PaddleOCR DB path: resizes to nearest-32-multiple (max 960 px long side), accepts dynamic input dimensions. `OcrInput` now carries an optional `color_image: Option<NdTensor<f32,3>>` with ImageNet normalization. `preprocess.rs` provides `prepare_image_imagenet()`.

## Medium Priority

- [x] **Layout analysis CJK `median_word_spacing == 0` bug**: Fixed in `layout_analysis.rs:145`. When `median_word_spacing == 0` (CJK has near-zero inter-glyph gaps), `min_width` now falls back to `(median_height / 2).max(1)` instead of degenerating to 0.

- [x] **Layout analysis O(n²)**: `group_into_lines` and paragraph grouping now use `Vec<Option<T>>` slot-based approach in `layout_analysis.rs`. O(n) behavior on dense pages.

- [x] **`OcrEngineParams::alphabet_chars`**: Added `alphabet_chars: Option<Vec<char>>` field to `OcrEngineParams`. Callers with `Vec<char>` (e.g. `cjk_alphabet_chars()`) skip the UTF-8 roundtrip entirely.

- [x] **WASM API for `cjk_text`**: `wasm_api.rs` now exposes `cjk_text::segment()`, `is_cjk()`, and alphabet helpers to JavaScript.

## Low Priority

- [ ] **Scratch buffer reuse in recognition**: `prepare_text_line` allocates a fresh `NdTensor` per line in a batch. Pre-allocating a scratch buffer sized to the group's max dimensions and reusing across lines would reduce allocations on dense pages.

- [ ] **`OcrEngineParamsImpl` simplification**: Consider whether the generic internal params struct can be collapsed once the test model API stabilizes.

- [ ] **`--features onnx` by default in CLI**: Currently users must pass `--features onnx` at build time to load `.onnx` models. Consider making ONNX the default and `.rten` opt-in, or document this more prominently in the CLI help text.

- [ ] **clippy warnings**: 13 warnings remain (`while_let_loop`, `needless_range_loop`, `incompatible_msrv`, `unnecessary_cast`, `map_or`, `collapsible_if`). `cargo clippy --fix` applies most. `incompatible_msrv` requires bumping `rust-version` or using older API.

## Completed

- [x] **Per-character confidence scores**: `TextChar` now has a `confidence: f32` field in [0, 1]. `TextItem` exposes a `confidence()` method (mean over chars). JSON output (`-j`) includes `"confidence"` per word.

- [x] **hOCR and ALTO XML output**: `--hocr` and `--alto` in `ocrs-cli/src/output.rs`.

- [x] **PaddleOCR ONNX recognition model support**: Auto-detect input channels/layout in `recognition.rs`. PP-OCRv5 Japanese/Chinese confirmed working.

- [x] **`--alphabet-file` CLI option**: Large CJK alphabets (18,384 chars for PP-OCRv5) via file path.

- [x] **CJK OCR end-to-end test**: PP-OCRv5 + built-in detection confirmed on `東京オリンピック2024`.

- [x] **`--mark-low-confidence`**: Flag uncertain OCR words in output with configurable threshold.

- [x] **`ocrs doctor`**: Health check subcommand — validates model files, reports config.

- [x] **`--deskew`**: Projection profile skew detection and correction (`deskew.rs`).

- [x] **`--post-correct-ja`**: Japanese OCR confusion correction below a confidence threshold.

- [x] **`--region x,y,w,h`**: Filter OCR output to lines intersecting a pixel region.

- [x] **`--markdown / -m`**: Markdown output format.

- [x] **`--find-text <query>`**: Filter OCR output to matching lines.

- [x] **`--tsv`**: Per-word TSV with bounding box, text, and confidence.

- [x] **`--auto-rotate`**: Auto-detect/correct 90°/180°/270° rotation via projection variance + density heuristic (`deskew.rs`).

- [x] **`--tables`**: Table-like layout detection, CSV/Markdown/JSON output (`table.rs`).

- [x] **`--furigana-separate`**: Separate furigana (ruby text) from base text (`furigana.rs`).

- [x] **`--confidence-heatmap <path>`**: Red/green confidence heatmap PNG.

- [x] **`--skip-text-pages`**: Skip OCR for PDF pages that already have a text layer.

- [x] **`--normalize-ja`**: Full-width ASCII → half-width.

- [x] **`--blocks-json`**: JSON output with paragraph block structure.

- [x] **`--review-json`**: JSON with per-word confidence and `needs_review` flags.

- [x] **GitHub security hardening**: SECURITY.md, Dependabot weekly (cargo + github-actions), `cargo audit` CI, `.cargo/audit.toml`, `.githooks/pre-push` (`build + clippy + audit` before every push).

- [x] **Dependency vulnerability fixes**: lopdf 0.34→0.42 (RUSTSEC-2026-0187 stack overflow), ring 0.17.7→0.17.14 (RUSTSEC-2025-0009 AES panic). rustls-webpki 0.102.x acknowledged in `.cargo/audit.toml` (blocked on ureq upstream).

- [x] **Upstream sync**: Merged robertknight/ocrs — serde_json 1.0.150, arboard `wayland-data-control` feature for Linux clipboard.
