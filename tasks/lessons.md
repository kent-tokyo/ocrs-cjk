# Lessons Learned

## Rust Language

- **`str::floor_char_boundary` is unstable** (feature `round_char_boundary`, issue #93743). Do not use it. Instead scan backwards with `str::is_char_boundary`: scan at most 3 positions back (UTF-8 sequences are at most 4 bytes, so the nearest boundary is within 3 bytes of any interior byte).

- **`matches!` works in `const fn`** since Rust 1.57. `match` expressions with char range patterns are also const-eval safe, so `pub const fn is_cjk(c: char) -> bool { matches!(c, '\u{...}'..='\u{...}' | ...) }` compiles without any unstable features.

- **`char` range syntax in iterators requires Rust ≥ 1.85** (`char: Step` stabilized in 1.85). For MSRV compatibility, use `(0x3040u32..=0x309Fu32).filter_map(char::from_u32)` instead of `('\u{3040}'..='\u{309F}')`.

- **`u32` subtraction before `as usize` can underflow**. `step.label - 1` panics in debug mode and wraps to `u32::MAX` in release when `step.label == 0`. Always use `checked_sub(1)` before casting.

- **Dead `#[derive(Clone)]` generates dead code**. Remove `Clone` from internal structs (`TextRecLine`) that are only moved, not cloned.

- **`#[derive(Default)]` on structs with `&[T]` fields is a footgun**. `Default` gives `&[]` (empty slice), which passes the borrow checker but causes a runtime model mismatch error (`alphabet size 0`). Remove `Default` from `RecognitionOpt` — it should be constructed explicitly.

## Unicode / CJK

- **Hangul Syllables end at U+D7A3, not U+D7AF**. U+D7A4–U+D7AF are unassigned codepoints. `char::from_u32` returns `None` for them, but `is_cjk` range patterns silently include them. The range must be `'\u{AC00}'..='\u{D7A3}'`.

- **`is_cjk()` must cover Hangul Compatibility Jamo (U+3130–U+318F)**. Individual Korean consonants/vowels (`ㄱ ㄴ ㅏ`) live here. Without this block, Korean text using Jamo is misclassified as Latin and merged into Latin segments by `TextSegmentIter`.

- **Bopomofo (U+3100–U+312F) and Bopomofo Extended (U+31A0–U+31BF)** are required for Traditional Chinese (Taiwan). Also missing from a naive CJK range.

## Performance

- **`alphabet.chars().nth(k)` is O(k)**, not O(1). For a 21,000-char CJK alphabet, this causes up to 21,000 byte scans per recognized character. Pre-collect to `Vec<char>` at engine construction and use `slice.get(k)`.

- **`String::contains(char)` is O(n)** linear scan. For `allowed_chars` filtering over a CJK alphabet, use `HashSet<char>` for O(1) lookup.

- **`Vec::remove(0)` is O(n)** (shifts the entire tail). In `layout_analysis.rs`, this appears inside nested loops, giving O(n²) layout analysis. Replace with `Vec<Option<T>>` slot-based iteration.

- **`chunk.to_vec()` on a slice of non-Copy structs clones every element**, including heap-allocated fields (`Polygon` → `Vec<Point>`). For batching owned `Vec`, use `Vec::split_off` to move without cloning.

## WASM

- **`rayon` panics at runtime on `wasm32-unknown-unknown`**. The code compiles (rayon has a wasm stub) but `into_par_iter` calls panic when executed. All parallelism in `recognize_text` must be guarded behind a `#[cfg(not(target_arch = "wasm32"))]` or replaced with sequential code for WASM targets.
