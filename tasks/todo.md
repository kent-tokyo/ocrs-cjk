# TODO

## High Priority

- [ ] **rayon / WASM**: `OcrEngine::recognize_text` uses `rayon::into_par_iter` via `thread_pool().run(...)`, which panics at runtime on `wasm32-unknown-unknown`. Upstream issue. Needs investigation into `rten`'s WASM thread pool shim or a sequential fallback path.

- [ ] **CJK-trained model**: The ML model itself must be trained on CJK characters. Without a CJK-trained detection+recognition model, the alphabet helpers and segmentation are ready but end-to-end CJK OCR is not possible. This is a training task, not a code task.

## Medium Priority

- [ ] **Layout analysis O(n²)**: `group_into_lines` and paragraph grouping in `layout_analysis.rs` use `Vec::remove` (O(n) shift) in inner loops, giving O(n²) overall. Replace with `Vec<Option<T>>` slot-based approach for O(n) behavior on dense pages.

- [ ] **`OcrEngineParams::alphabet_chars`**: Currently callers who have a `Vec<char>` (e.g. from `cjk_alphabet_chars()`) must convert to `String` to set `OcrEngineParams::alphabet`. Add an `alphabet_chars: Option<Vec<char>>` field to `OcrEngineParams` to skip the UTF-8 roundtrip entirely.

- [ ] **WASM API for `cjk_text`**: `wasm_api.rs` does not expose `cjk_text::segment()`, `is_cjk()`, or the alphabet helpers to JavaScript. Add WASM bindings for downstream JS NLP consumers.

## Low Priority

- [ ] **Scratch buffer reuse in recognition**: `prepare_text_line` allocates a fresh `NdTensor` per line in a batch. Pre-allocating a scratch buffer sized to the group's max dimensions and reusing across lines would reduce allocations on dense pages.

- [ ] **`OcrEngineParamsImpl` simplification**: Consider whether the generic internal params struct can be collapsed once the test model API stabilizes.
