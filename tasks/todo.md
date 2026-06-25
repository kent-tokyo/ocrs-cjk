# TODO

## High Priority

- [x] **rayon / WASM**: Fixed by adding `#[cfg(not(target_arch = "wasm32"))]` guards around the `rayon` and `thread_pool` imports, and adding a sequential `into_iter()` fallback path for `wasm32-unknown-unknown`. Both native and WASM builds pass.

- [x] **PaddleOCR detection model support**: Implemented `DetectorKind` enum in `detection.rs` (auto-detected from `input_shape[1]`). PaddleOCR DB path: resizes to nearest-32-multiple (max 960 px long side), accepts dynamic input dimensions. `OcrInput` now carries an optional `color_image: Option<NdTensor<f32,3>>` with ImageNet normalization. `preprocess.rs` provides `prepare_image_imagenet()`.

## Medium Priority

- [x] **Layout analysis CJK `median_word_spacing == 0` bug**: Fixed in `layout_analysis.rs:145`. When `median_word_spacing == 0` (CJK has near-zero inter-glyph gaps), `min_width` now falls back to `(median_height / 2).max(1)` instead of degenerating to 0.

- [ ] **Layout analysis O(n²)**: `group_into_lines` and paragraph grouping in `layout_analysis.rs` use `Vec::remove` (O(n) shift) in inner loops, giving O(n²) overall. Replace with `Vec<Option<T>>` slot-based approach for O(n) behavior on dense pages.

- [ ] **`OcrEngineParams::alphabet_chars`**: Currently callers who have a `Vec<char>` (e.g. from `cjk_alphabet_chars()`) must convert to `String` to set `OcrEngineParams::alphabet`. Add an `alphabet_chars: Option<Vec<char>>` field to `OcrEngineParams` to skip the UTF-8 roundtrip entirely.

- [ ] **WASM API for `cjk_text`**: `wasm_api.rs` does not expose `cjk_text::segment()`, `is_cjk()`, or the alphabet helpers to JavaScript. Add WASM bindings for downstream JS NLP consumers.

## Low Priority

- [ ] **Scratch buffer reuse in recognition**: `prepare_text_line` allocates a fresh `NdTensor` per line in a batch. Pre-allocating a scratch buffer sized to the group's max dimensions and reusing across lines would reduce allocations on dense pages.

- [ ] **`OcrEngineParamsImpl` simplification**: Consider whether the generic internal params struct can be collapsed once the test model API stabilizes.

- [ ] **`--features onnx` by default in CLI**: Currently users must pass `--features onnx` at build time to load `.onnx` models. Consider making ONNX the default and `.rten` opt-in, or document this more prominently in the CLI help text.

## Completed

- [x] **Per-character confidence scores**: `TextChar` now has a `confidence: f32` field in [0, 1]. `TextItem` exposes a `confidence()` method (mean over chars). JSON output (`-j`) includes `"confidence"` per word. Confidence is computed from the raw CTC decode output via `exp(log_prob).clamp(0,1)`.

- [x] **hOCR and ALTO XML output**: `--hocr` emits hOCR HTML (standard `ocr_page`/`ocr_line`/`ocrx_word` with `bbox` and `x_wconf`). `--alto` emits ALTO v4 XML with `TextLine`/`String` elements and `WC` confidence. Both are in `ocrs-cli/src/output.rs`.

- [x] **PaddleOCR ONNX recognition model support**: Auto-detect input channels from `input_shape[1]` and output layout from `batch_size` vs `dim[0]`. Implemented in `recognition.rs`. Tested with PP-OCRv5 (Japanese confirmed working, Chinese expected working — same model).

- [x] **`--alphabet-file` CLI option**: Added to `ocrs-cli/src/main.rs`. Avoids shell-escaping issues when passing large CJK alphabets (18,384 characters for PP-OCRv5). Takes precedence over `--alphabet` if both are given.

- [x] **CJK OCR end-to-end test**: PP-OCRv5 recognition model + built-in detection model successfully recognized `東京オリンピック2024` from a synthetic test image. Alphabet extraction script documented in all READMEs.
