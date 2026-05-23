# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] — CJK Fork (kent-tokyo/ocrs-cjk)

### Added

- **PaddleOCR ONNX recognition model support**: `TextRecognizer` now auto-detects input channel count from `input_shape[1]` (1=grayscale for ocrs models, 3=RGB for PaddleOCR) and output layout from `batch_size` vs `dim[0]` (`[seq, batch, class]` → transpose; `[batch, seq, class]` → pass-through). No API changes; existing ocrs models work identically. Tested with PP-OCRv5 (Japanese CJK OCR confirmed working).
- **`--alphabet-file <path>` CLI option**: Read the recognition alphabet from a UTF-8 file instead of a command-line string. Avoids shell-escaping corruption when passing large CJK character sets (PP-OCRv5 uses 18,384 characters). Takes precedence over `--alphabet` if both are specified.
- `cjk_text` module with CJK-aware utilities:
  - `is_cjk(c: char) -> bool` (`const fn`) — covers Hiragana, Katakana, CJK Unified Ideographs (A–F), Hangul Syllables, Hangul Jamo, Hangul Compatibility Jamo, Hangul Jamo Extended-A/B, Bopomofo, Bopomofo Extended, CJK Symbols & Punctuation, CJK Compatibility, CJK Compatibility Forms, CJK Compatibility Ideographs, Halfwidth/Fullwidth Forms
  - `floor_char_boundary(s: &str, byte_idx: usize) -> usize` — UTF-8 safe byte boundary (avoids panic on multibyte CJK chars)
  - `segment(s: &str) -> impl Iterator<Item = &str>` — zero-copy script-homogeneous segmentation
  - `hiragana()`, `katakana()`, `cjk_unified()`, `cjk_unified_ext_a()`, `hangul()` — lazy char iterators
  - `cjk_alphabet() -> String` — Hiragana + Katakana + CJK Unified for use as `OcrEngineParams::alphabet`
  - `cjk_alphabet_chars() -> Vec<char>` — same without intermediate UTF-8 String allocation
- `TextLine::segments()` — CJK-aware segmentation that splits at script transitions (Latin↔CJK) without requiring space delimiters; `words()` is unchanged
- `OcrEngine::alphabet() -> &[char]` — exposes the active alphabet for downstream NLP inspection
- WASM `TextLine::segments()` binding in `wasm_api.rs`
- Re-exports at crate root: `ocrs::is_cjk`, `ocrs::segment`, `ocrs::cjk_alphabet`, `ocrs::cjk_alphabet_chars`
- Multilingual READMEs: `README_ja.md`, `README_zh.md`, `README_kr.md`

### Fixed

- `step.label - 1` integer underflow: CTC decode step with `label == 0` caused panic in debug builds and silent `u32::MAX` wraparound in release. Fixed with `checked_sub(1)`.
- `DEFAULT_ALPHABET`: placeholder `E` before `ABCDE` corrected to `€` (U+20AC) to match the trained model's alphabet.
- Hangul range off-by-one: `is_cjk` and `hangul()` iterator now correctly end at U+D7A3 (`힣`); U+D7A4–U+D7AF are unassigned and excluded.

### Changed (Performance)

- `OcrEngine` pre-collects `alphabet_chars: Vec<char>` at construction; eliminated per-call allocation (~168 KB for CJK alphabet) in `recognize_text`
- `allowed_chars` filtering uses `HashSet<char>` — O(alphabet + allowed) instead of O(alphabet × allowed)
- Alphabet label lookup changed from `alphabet.chars().nth(k)` (O(k)) to `alphabet_chars.get(k)` (O(1))
- Line group batching uses `Vec::split_off` (move) instead of `chunk.to_vec()` (clone), eliminating `Polygon` heap copies
- `get_text` streams output via `fmt::Write` instead of collecting intermediate `Vec<String>` before `join`
- `line_polygon` and `rotated_rect` pre-allocate with `Vec::with_capacity`
- `collect_text_words` shared helper eliminates duplication between `wasm_api::words()` and `wasm_api::segments()`

### Known Limitations

- `OcrEngine::recognize_text` uses `rayon` for parallelism and panics at runtime on `wasm32-unknown-unknown`. This is an upstream issue. Other APIs and all `cjk_text` utilities are WASM-compatible.
- No CJK-trained model is bundled. End-to-end CJK OCR is possible by supplying an external PaddleOCR ONNX recognition model (see README for step-by-step instructions). The detection model format (PaddleOCR) is not yet supported; the built-in Latin-trained detection model works in practice.
- Loading `.onnx` models requires building with `--features onnx` (rten default format is `.rten`).

## [0.12.2] - 2026-03-27

- Added support for reading from stdin in CLI (https://github.com/robertknight/ocrs/pull/241)

## [0.12.1] - 2026-02-14

- Added support for reading from the clipboard in the CLI (https://github.com/robertknight/ocrs/pull/229)

## [0.12.0] - 2025-12-27

- Updated to rten v0.24. This adds the ability to load custom models in
  ONNX format instead of `.rten` format (requires enabling the `onnx` crate feature)

- Improved image preprocessing performance (https://github.com/robertknight/ocrs/pull/217).
  Thanks @laundmo for initial suggestions and testing.

## [0.11.0] - 2025-09-11

- Updated to rten v0.22. This enables AVX-512 on stable Rust and updates the
  MSRV to Rust v1.89.

## [0.10.4] - 2025-08-05

- Update compatible rten version range to include v0.21

## [0.10.3] - 2025-05-08

- Improved performance on Arm by 20-30%, by updating to rten v0.18

## [0.10.2] - 2025-02-09

- Disabled IDNA support in ocrs-cli's `url` dependency. This removes various
  Unicode-related dependencies (https://github.com/robertknight/ocrs/pull/163)

## [0.10.1] - 2025-02-08

- Update ureq to v3.x. This removes a number of unnecessary indirect
  dependencies from ocrs-cli (https://github.com/robertknight/ocrs/pull/162)

## [0.10.0] - 2025-02-08

- Updated compatible rten version range to >= 0.14, < 0.17
  (https://github.com/robertknight/ocrs/pull/127,
  https://github.com/robertknight/ocrs/pull/161)

## [0.9.0] - 2024-10-03

- Added `allowed_chars` library option and `--allowed-chars` CLI option to
  filter the characters produced by text recognition
  (https://github.com/robertknight/ocrs/pull/119). Thanks @basic-bgnr.

- Improved error message if custom alphabet size does not match recognition
  model output (https://github.com/robertknight/ocrs/pull/126)

- Updated README to note the importance of building the project, or at least the
  rten dependencies, in release mode
  (https://github.com/robertknight/ocrs/pull/118). Thanks @ezkangaroo.

## [0.8.1] - 2024-08-01

- Added ability to customize the alphabet used by the recognition model
  (https://github.com/robertknight/ocrs/pull/100). Thanks @Phaired.

- Updated rten to v0.13.1. This enables running custom models in the V2
  [rten model format](https://github.com/robertknight/rten/blob/main/docs/rten-file-format.md)

## [0.8.0] - 2024-05-25

### Breaking changes

This release changes Ocrs's internal use of threads, which may affect consumers
that are using their own parallelism. Specifically Ocrs no longer uses the
global Rayon thread pool but instead a custom thread pool which is sized to
match the number of physical rather than logical cores. See
https://github.com/robertknight/ocrs/pull/79 for more details and information on
adapting.

### Changes

- Updated rten to v0.10.0. This improves performance when recognizing long lines
  of text (https://github.com/robertknight/ocrs/pull/79) and improves efficiency
  by setting the number of threads to match the number of physical cores.

- Errors that occur when running the text recognition model are now propagated
  to the caller instead of causing a panic (https://github.com/robertknight/ocrs/pull/77)

## [0.7.0] - 2024-05-16

### Breaking changes

The APIs for loading models and images have changed in this release to make them
more efficient and easier to use. See the updated
[hello_ocr](https://github.com/robertknight/ocrs/blob/main/ocrs/examples/hello_ocr.rs)
example.

### Changes

 - Updated rten to v0.9.0. This brings a simpler API for loading models from
   disk (`Model::load_file`) and improves performance
   (https://github.com/robertknight/ocrs/pull/76)

 - Updated image crate. This includes a much faster JPEG decoder
   (https://github.com/robertknight/ocrs/pull/58)

 - Re-designed the API for loading images to be easier to use and more
   efficient (https://github.com/robertknight/ocrs/pull/56).

## [0.6.0] - 2024-04-29

 - Updated rten to v0.8.0. This fixes a crash on x86-64 CPUs that don't support
   AVX2 instructions and includes several performance improvements
   [#53](https://github.com/robertknight/ocrs/pull/53).

 - Added `--text-mask` flag to CLI which saves a binarized version of the text
   probability mask as an image (https://github.com/robertknight/ocrs/pull/38)

 - Made it easier to run examples (https://github.com/robertknight/ocrs/pull/41)

## [0.5.0] - 2024-02-28

 - Improve recognition accuracy for long text lines, at the cost of longer
   inference times, by increasing max image width after preprocessing
   [#32](https://github.com/robertknight/ocrs/pull/32)

 - Added `--text-line-images` option to save previews of text lines after
   preprocessing. This is useful for debugging recognition accuracy issues
   [#29](https://github.com/robertknight/ocrs/pull/29),
   [#30](https://github.com/robertknight/ocrs/pull/30).

 - Added a note in the README about the importance of building ocrs (or at least
   the rten dependencies) in release mode
   [#28](https://github.com/robertknight/ocrs/pull/28)

 - Updated rten to 0.4.0. This includes optimizations for post-processing of
   the text segmentation mask [#23](https://github.com/robertknight/ocrs/pull/23).

## [0.4.0] - 2024-01-23

 - Updated rten to v0.3.1. This improves performance on Arm by ~30%.

 - Fix panic in layout analysis when average word spacing in a line is negative
   [#20](https://github.com/robertknight/ocrs/pull/20)

 - Added LICENSE files to repository (Apache 2, MIT)
   [#12](https://github.com/robertknight/ocrs/pull/12)

## [0.3.1] - 2024-01-03

 - Fix cache directory location on Windows [#9](https://github.com/robertknight/ocrs/pull/9)

 - Improved speed on ARM by ~16% [9224ac9](https://github.com/robertknight/ocrs/commit/9224ac9)

## [0.3.0] - 2024-01-02

 - Extract the ocrs project out of the [RTen](https://github.com/robertknight/rten)
   repository and into a standalone repo at https://github.com/robertknight/ocrs.

 - Improve the `--json` output format with extracted text and coordinates of
   the rotated bounding rect for each word and line (92f17fb).

## [0.2.1] - 2024-01-01

 - Update rten to fix incorrect output on non-x64 / wasm32 platforms

## [0.2.0] - 2024-01-01

 - Improve layout analysis (ce52b3a1, cefb6c3f). The longer term plan is to use
   machine learning for layout analysis, but these incremental tweaks address
   some of the most egregious errors.
 - Add `--version` flag to CLI (20055ee0)
 - Revise CLI flags for specifying output format (97c3a011). The output path
   is now specified with `-o`. Available formats are text (default), JSON
   (`--json`) or annotated PNG (`--png`).
 - Fixed slow OCR model downloads by changing hosting location
   (https://github.com/robertknight/rten/issues/22).

## [0.1.0] - 2023-12-31

Initial release.
