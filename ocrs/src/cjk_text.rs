/// Returns `true` if `c` belongs to a CJK (Chinese, Japanese, or Korean) Unicode block.
pub const fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{1100}'..='\u{11FF}'   // Hangul Jamo
        | '\u{3000}'..='\u{303F}' // CJK Symbols and Punctuation
        | '\u{3040}'..='\u{309F}' // Hiragana
        | '\u{30A0}'..='\u{30FF}' // Katakana
        | '\u{3100}'..='\u{312F}' // Bopomofo
        | '\u{3130}'..='\u{318F}' // Hangul Compatibility Jamo (ㄱ ㄴ ㅏ etc.)
        | '\u{3190}'..='\u{319F}' // Kanbun
        | '\u{31A0}'..='\u{31BF}' // Bopomofo Extended
        | '\u{3300}'..='\u{33FF}' // CJK Compatibility (㍉ ㎝ etc.)
        | '\u{3400}'..='\u{4DBF}' // CJK Unified Ideographs Extension A
        | '\u{4E00}'..='\u{9FFF}' // CJK Unified Ideographs
        | '\u{A960}'..='\u{A97F}' // Hangul Jamo Extended-A
        | '\u{AC00}'..='\u{D7A3}' // Hangul Syllables (U+D7A4..U+D7AF are unassigned)
        | '\u{D7B0}'..='\u{D7FF}' // Hangul Jamo Extended-B
        | '\u{F900}'..='\u{FAFF}' // CJK Compatibility Ideographs
        | '\u{FE30}'..='\u{FE4F}' // CJK Compatibility Forms (vertical punctuation)
        // U+FF00..U+FFEF: includes full-width Latin/punctuation and half-width Katakana/Hangul.
        // Full-width forms are standard in CJK documents, so the full block is intentional.
        | '\u{FF00}'..='\u{FFEF}' // Halfwidth and Fullwidth Forms
        | '\u{20000}'..='\u{2A6DF}' // CJK Unified Ideographs Extension B
        | '\u{2A700}'..='\u{2B73F}' // CJK Unified Ideographs Extension C
        | '\u{2B740}'..='\u{2B81F}' // CJK Unified Ideographs Extension D
        | '\u{2B820}'..='\u{2CEAF}' // CJK Unified Ideographs Extension E
        | '\u{2CEB0}'..='\u{2EBEF}' // CJK Unified Ideographs Extension F
    )
}

/// Returns the largest byte index `<= byte_idx` that is a valid UTF-8 char boundary in `s`.
///
/// Use this instead of direct byte slicing (`&s[..n]`) to avoid panics on
/// multibyte CJK characters.
#[inline]
pub fn floor_char_boundary(s: &str, byte_idx: usize) -> usize {
    if byte_idx >= s.len() {
        return s.len();
    }
    // Scan backwards at most 3 bytes: UTF-8 multibyte sequences are at most 4 bytes,
    // so the nearest boundary is guaranteed within 3 bytes of any interior byte.
    let lo = byte_idx.saturating_sub(3);
    for i in (lo..=byte_idx).rev() {
        if s.is_char_boundary(i) {
            return i;
        }
    }
    0
}

#[derive(Clone, Copy)]
struct SegmentIter<'a> {
    remaining: &'a str,
}

impl<'a> Iterator for SegmentIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // Single pass: skip whitespace, capture first non-WS char and its byte offset.
        let mut chars_iter = self.remaining.char_indices();
        let (start, first_char) = loop {
            match chars_iter.next() {
                None => return None,
                Some((_, c)) if c.is_whitespace() => {}
                Some(pair) => break pair,
            }
        };
        let first_is_cjk = is_cjk(first_char);

        // Continue the same iterator — no reset, no second pass.
        let seg_end = chars_iter
            .find(|(_, c)| c.is_whitespace() || is_cjk(*c) != first_is_cjk)
            .map_or(self.remaining.len(), |(i, _)| i);

        // Slice original `remaining` using byte offsets from the single iterator.
        let segment = &self.remaining[start..seg_end];
        self.remaining = &self.remaining[seg_end..];
        Some(segment)
    }
}

/// Segments `s` into borrowed script-homogeneous runs, yielding `&str` slices with no allocation.
///
/// Whitespace is a split point but is not yielded. CJK and non-CJK runs are separate segments.
///
/// ```
/// # use ocrs_cjk::cjk_text::segment;
/// let parts: Vec<&str> = segment("Hello 世界!").collect();
/// assert_eq!(parts, ["Hello", "世界", "!"]);
/// ```
pub fn segment(s: &str) -> impl Iterator<Item = &str> {
    SegmentIter { remaining: s }
}

/// Returns an iterator over all Hiragana characters (U+3040..=U+309F).
pub fn hiragana() -> impl Iterator<Item = char> {
    (0x3040u32..=0x309F).filter_map(char::from_u32)
}

/// Returns an iterator over all Katakana characters (U+30A0..=U+30FF).
pub fn katakana() -> impl Iterator<Item = char> {
    (0x30A0u32..=0x30FF).filter_map(char::from_u32)
}

/// Returns an iterator over CJK Unified Ideographs (U+4E00..=U+9FFF).
pub fn cjk_unified() -> impl Iterator<Item = char> {
    (0x4E00u32..=0x9FFF).filter_map(char::from_u32)
}

/// Returns an iterator over CJK Unified Ideographs Extension A (U+3400..=U+4DBF).
pub fn cjk_unified_ext_a() -> impl Iterator<Item = char> {
    (0x3400u32..=0x4DBF).filter_map(char::from_u32)
}

/// Returns an iterator over Hangul Syllables (U+AC00..=U+D7A3).
pub fn hangul() -> impl Iterator<Item = char> {
    (0xAC00u32..=0xD7A3).filter_map(char::from_u32)
}

/// Builds a CJK alphabet string containing Hiragana, Katakana, and CJK Unified Ideographs.
///
/// Use as `OcrEngineParams { alphabet: Some(cjk_text::cjk_alphabet()), .. }`.
/// For Korean support, chain [`hangul()`] before collecting:
/// ```
/// # use ocrs_cjk::cjk_text::{hiragana, katakana, cjk_unified, hangul};
/// let full: String = hiragana().chain(katakana()).chain(cjk_unified()).chain(hangul()).collect();
/// ```
pub fn cjk_alphabet() -> String {
    // All three ranges consist solely of 3-byte UTF-8 codepoints.
    // Pre-allocating avoids 3-4 intermediate reallocations during collect().
    const TOTAL: usize = (0x309F - 0x3040 + 1) + (0x30FF - 0x30A0 + 1) + (0x9FFF - 0x4E00 + 1);
    let mut s = String::with_capacity(TOTAL * 3);
    hiragana().chain(katakana()).chain(cjk_unified()).for_each(|c| s.push(c));
    s
}

/// Builds a CJK alphabet as a `Vec<char>` (Hiragana + Katakana + CJK Unified Ideographs).
///
/// Prefer this over `cjk_alphabet()` when passing to `OcrEngineParams::alphabet`, because
/// `OcrEngine` immediately decomposes an alphabet `String` into `Vec<char>` internally.
/// This function skips the intermediate UTF-8 `String` entirely.
pub fn cjk_alphabet_chars() -> Vec<char> {
    const TOTAL: usize = (0x309F - 0x3040 + 1) + (0x30FF - 0x30A0 + 1) + (0x9FFF - 0x4E00 + 1);
    let mut v = Vec::with_capacity(TOTAL);
    hiragana().chain(katakana()).chain(cjk_unified()).for_each(|c| v.push(c));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- is_cjk ----

    #[test]
    fn test_is_cjk_hiragana() {
        assert!(is_cjk('あ')); // U+3042
        assert!(is_cjk('ん')); // U+3093
    }

    #[test]
    fn test_is_cjk_katakana() {
        assert!(is_cjk('ア')); // U+30A2
        assert!(is_cjk('ン')); // U+30F3
    }

    #[test]
    fn test_is_cjk_unified() {
        assert!(is_cjk('缘')); // U+7F18 — the exact char from CLAUDE.md Issue #15647
        assert!(is_cjk('漢')); // U+6F22
        assert!(is_cjk('字')); // U+5B57
    }

    #[test]
    fn test_is_cjk_extension_a() {
        assert!(is_cjk('\u{3400}')); // first char of Extension A
        assert!(is_cjk('\u{4DBF}')); // last char of Extension A
    }

    #[test]
    fn test_is_cjk_hangul() {
        assert!(is_cjk('가')); // U+AC00
        assert!(is_cjk('힣')); // U+D7A3
    }

    #[test]
    fn test_is_cjk_extension_b() {
        assert!(is_cjk('\u{20000}')); // first char of Extension B
    }

    #[test]
    fn test_is_cjk_bopomofo() {
        assert!(is_cjk('\u{3105}')); // ㄅ — first Bopomofo
        assert!(is_cjk('\u{312F}')); // last Bopomofo
        assert!(is_cjk('\u{31A0}')); // first Bopomofo Extended
        assert!(is_cjk('\u{31BF}')); // last Bopomofo Extended
    }

    #[test]
    fn test_is_cjk_hangul_compatibility_jamo() {
        assert!(is_cjk('ㄱ')); // U+3131 — Hangul Compatibility Jamo
        assert!(is_cjk('ㅏ')); // U+314F
        assert!(is_cjk('\u{3130}')); // first of block
        assert!(is_cjk('\u{318F}')); // last of block
    }

    #[test]
    fn test_is_cjk_hangul_jamo_extended() {
        assert!(is_cjk('\u{A960}')); // Hangul Jamo Extended-A first
        assert!(is_cjk('\u{A97F}')); // Hangul Jamo Extended-A last
        assert!(is_cjk('\u{D7B0}')); // Hangul Jamo Extended-B first
        assert!(is_cjk('\u{D7FF}')); // Hangul Jamo Extended-B last
    }

    #[test]
    fn test_is_cjk_compatibility_blocks() {
        assert!(is_cjk('\u{3300}')); // CJK Compatibility first (㌀)
        assert!(is_cjk('\u{33FF}')); // CJK Compatibility last
        assert!(is_cjk('\u{FE30}')); // CJK Compatibility Forms first
        assert!(is_cjk('\u{FE4F}')); // CJK Compatibility Forms last
    }

    #[test]
    fn test_is_not_cjk() {
        assert!(!is_cjk('A'));
        assert!(!is_cjk('z'));
        assert!(!is_cjk('0'));
        assert!(!is_cjk(' '));
        assert!(!is_cjk('é'));
    }

    // ---- floor_char_boundary ----

    // Directly tests the panic scenario from CLAUDE.md §C (Issue #15647):
    // '缘' occupies bytes 3..6 in "abc缘def", so byte indices 4 and 5 are
    // inside the character and must be floored to index 3.
    #[test]
    fn test_floor_char_boundary_inside_cjk() {
        let s = "abc缘def";
        // '缘' starts at byte 3 and is 3 bytes wide (bytes 3, 4, 5).
        assert_eq!(floor_char_boundary(s, 3), 3); // boundary itself
        assert_eq!(floor_char_boundary(s, 4), 3); // inside '缘'
        assert_eq!(floor_char_boundary(s, 5), 3); // still inside '缘'
        assert_eq!(floor_char_boundary(s, 6), 6); // 'd' starts here
    }

    #[test]
    fn test_floor_char_boundary_ascii() {
        let s = "abcdef";
        assert_eq!(floor_char_boundary(s, 0), 0);
        assert_eq!(floor_char_boundary(s, 3), 3);
        assert_eq!(floor_char_boundary(s, 6), 6); // past end → len
    }

    #[test]
    fn test_floor_char_boundary_past_end() {
        let s = "abc";
        assert_eq!(floor_char_boundary(s, 100), 3);
    }

    #[test]
    fn test_floor_char_boundary_empty() {
        assert_eq!(floor_char_boundary("", 0), 0);
    }

    #[test]
    fn test_floor_char_boundary_4byte_emoji() {
        // '😀' is U+1F600, encoded as 4 bytes [F0 9F 98 80].
        // "a😀" → 'a' at byte 0, '😀' at bytes 1..5.
        let s = "a😀";
        assert_eq!(floor_char_boundary(s, 0), 0); // 'a' boundary
        assert_eq!(floor_char_boundary(s, 1), 1); // '😀' start = boundary
        assert_eq!(floor_char_boundary(s, 2), 1); // inside '😀' → floor to 1
        assert_eq!(floor_char_boundary(s, 3), 1); // inside '😀' → floor to 1
        assert_eq!(floor_char_boundary(s, 4), 1); // inside '😀' → floor to 1
        assert_eq!(floor_char_boundary(s, 5), 5); // past end → len
    }

    // ---- segment ----

    #[test]
    fn test_segment_mixed() {
        let parts: Vec<&str> = segment("Hello 世界!").collect();
        assert_eq!(parts, ["Hello", "世界", "!"]);
    }

    #[test]
    fn test_segment_cjk_only() {
        let parts: Vec<&str> = segment("日本語テスト").collect();
        assert_eq!(parts, ["日本語テスト"]);
    }

    #[test]
    fn test_segment_latin_only() {
        let parts: Vec<&str> = segment("foo bar baz").collect();
        assert_eq!(parts, ["foo", "bar", "baz"]);
    }

    #[test]
    fn test_segment_no_spaces_mixed() {
        let parts: Vec<&str> = segment("OCR認識エンジン").collect();
        assert_eq!(parts, ["OCR", "認識エンジン"]);
    }

    #[test]
    fn test_segment_whitespace_only() {
        let parts: Vec<&str> = segment("   \t\n  ").collect();
        assert!(parts.is_empty());
    }

    #[test]
    fn test_segment_empty() {
        let parts: Vec<&str> = segment("").collect();
        assert!(parts.is_empty());
    }

    #[test]
    fn test_segment_borrows_input() {
        // Verify zero-copy: the yielded slices point into the original string.
        let s = String::from("Hello世界");
        let s_ptr = s.as_ptr();
        let parts: Vec<&str> = segment(&s).collect();
        assert_eq!(parts[0].as_ptr(), s_ptr); // "Hello" borrows from s
    }

    #[test]
    fn test_segment_hangul_latin() {
        let parts: Vec<&str> = segment("한국어 test").collect();
        assert_eq!(parts, ["한국어", "test"]);
    }

    // ---- alphabet helpers ----

    #[test]
    fn test_hiragana_range() {
        let chars: Vec<char> = hiragana().collect();
        assert_eq!(chars.len(), 0x309F - 0x3040 + 1);
        assert!(chars.contains(&'あ'));
        assert!(chars.contains(&'ん'));
    }

    #[test]
    fn test_katakana_range() {
        let chars: Vec<char> = katakana().collect();
        assert_eq!(chars.len(), 0x30FF - 0x30A0 + 1);
        assert!(chars.contains(&'ア'));
        assert!(chars.contains(&'ン'));
    }

    #[test]
    fn test_cjk_unified_range() {
        let chars: Vec<char> = cjk_unified().collect();
        assert_eq!(chars.len(), 0x9FFF - 0x4E00 + 1);
        assert!(chars.contains(&'漢'));
        assert!(chars.contains(&'字'));
        assert!(chars.contains(&'缘')); // the exact char from CLAUDE.md Issue #15647
    }

    #[test]
    fn test_cjk_unified_ext_a_range() {
        let chars: Vec<char> = cjk_unified_ext_a().collect();
        assert_eq!(chars.len(), 0x4DBF - 0x3400 + 1);
        assert!(chars.contains(&'\u{3400}'));
        assert!(chars.contains(&'\u{4DBF}'));
    }

    #[test]
    fn test_hangul_range() {
        let chars: Vec<char> = hangul().collect();
        assert_eq!(chars.len(), 0xD7A3 - 0xAC00 + 1); // 11172; U+D7A4..U+D7AF are unassigned
        assert!(chars.contains(&'가'));
        assert!(chars.contains(&'힣')); // U+D7A3, last valid Hangul syllable
        // Unassigned codepoints must NOT be included
        assert!(!chars.contains(&'\u{D7A4}'));
        assert!(!chars.contains(&'\u{D7AF}'));
    }

    #[test]
    fn test_cjk_alphabet_contains_samples() {
        let alpha = cjk_alphabet();
        assert!(alpha.contains('あ')); // hiragana
        assert!(alpha.contains('ア')); // katakana
        assert!(alpha.contains('漢')); // cjk unified
        assert!(!alpha.contains('가')); // hangul not included by default
    }

    #[test]
    fn test_alphabet_helpers_chain() {
        let full: String = hiragana()
            .chain(katakana())
            .chain(cjk_unified())
            .chain(hangul())
            .collect();
        assert!(full.contains('が'));
        assert!(full.contains('가'));
    }

    #[test]
    fn test_cjk_alphabet_byte_length() {
        // Verifies the with_capacity constant in cjk_alphabet() is exact.
        const TOTAL: usize = (0x309F - 0x3040 + 1) + (0x30FF - 0x30A0 + 1) + (0x9FFF - 0x4E00 + 1);
        let alpha = cjk_alphabet();
        assert_eq!(alpha.chars().count(), TOTAL);
        assert_eq!(alpha.len(), TOTAL * 3); // all chars are 3-byte UTF-8
    }
}
