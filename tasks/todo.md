# TODO

## High Priority

- [ ] **rayon / WASM**: `OcrEngine::recognize_text` uses `rayon::into_par_iter` via `thread_pool().run(...)`, which panics at runtime on `wasm32-unknown-unknown`. Upstream issue. Needs investigation into `rten`'s WASM thread pool shim or a sequential fallback path.

- [ ] **PaddleOCR detection model support**: The detection model format used by PaddleOCR (different input normalization and output format) is not yet supported. Only the recognition model format has been implemented. Until this is done, users must rely on the built-in Latin-trained detection model, which works in practice but has unverified accuracy on complex CJK layouts.

## Medium Priority

- [ ] **Layout analysis O(n²)**: `group_into_lines` and paragraph grouping in `layout_analysis.rs` use `Vec::remove` (O(n) shift) in inner loops, giving O(n²) overall. Replace with `Vec<Option<T>>` slot-based approach for O(n) behavior on dense pages.

- [ ] **`OcrEngineParams::alphabet_chars`**: Currently callers who have a `Vec<char>` (e.g. from `cjk_alphabet_chars()`) must convert to `String` to set `OcrEngineParams::alphabet`. Add an `alphabet_chars: Option<Vec<char>>` field to `OcrEngineParams` to skip the UTF-8 roundtrip entirely.

- [ ] **WASM API for `cjk_text`**: `wasm_api.rs` does not expose `cjk_text::segment()`, `is_cjk()`, or the alphabet helpers to JavaScript. Add WASM bindings for downstream JS NLP consumers.

## Low Priority

- [ ] **Scratch buffer reuse in recognition**: `prepare_text_line` allocates a fresh `NdTensor` per line in a batch. Pre-allocating a scratch buffer sized to the group's max dimensions and reusing across lines would reduce allocations on dense pages.

- [ ] **`OcrEngineParamsImpl` simplification**: Consider whether the generic internal params struct can be collapsed once the test model API stabilizes.

- [ ] **`--features onnx` by default in CLI**: Currently users must pass `--features onnx` at build time to load `.onnx` models. Consider making ONNX the default and `.rten` opt-in, or document this more prominently in the CLI help text.

## Completed

- [x] **PaddleOCR ONNX recognition model support**: Auto-detect input channels from `input_shape[1]` and output layout from `batch_size` vs `dim[0]`. Implemented in `recognition.rs`. Tested with PP-OCRv5 (Japanese confirmed working, Chinese expected working — same model).

- [x] **`--alphabet-file` CLI option**: Added to `ocrs-cli/src/main.rs`. Avoids shell-escaping issues when passing large CJK alphabets (18,384 characters for PP-OCRv5). Takes precedence over `--alphabet` if both are given.

- [x] **CJK OCR end-to-end test**: PP-OCRv5 recognition model + built-in detection model successfully recognized `東京オリンピック2024` from a synthetic test image. Alphabet extraction script documented in all READMEs.
