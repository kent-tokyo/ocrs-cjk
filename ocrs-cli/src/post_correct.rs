use ocrs_cjk::{TextChar, TextItem, TextLine};

/// Apply Japanese OCR confusion corrections to recognized text lines.
///
/// Only characters with `confidence < threshold` are candidates for correction.
/// Returns a new Vec with corrected `TextLine` objects.
///
/// Corrections applied:
/// - Latin-digit boundary confusions (0↔O, 1↔l) based on surrounding character context
/// - Unambiguous rare misreads (曰→日)
pub fn apply_ja(lines: &[Option<TextLine>], threshold: f32) -> Vec<Option<TextLine>> {
    lines
        .iter()
        .map(|line| {
            line.as_ref().map(|line| {
                let chars = line.chars();
                let corrected: Vec<TextChar> = chars
                    .iter()
                    .enumerate()
                    .map(|(i, tc)| {
                        if tc.confidence >= threshold {
                            return tc.clone();
                        }
                        let prev = if i > 0 { Some(chars[i - 1].char) } else { None };
                        let next = chars.get(i + 1).map(|c| c.char);
                        TextChar {
                            char: correct_char(tc.char, prev, next),
                            ..tc.clone()
                        }
                    })
                    .collect();
                TextLine::new(corrected)
            })
        })
        .collect()
}

/// Return `true` if `prev` or `next` is an ASCII digit.
fn is_digit_context(prev: Option<char>, next: Option<char>) -> bool {
    [prev, next]
        .iter()
        .any(|c| c.map_or(false, |c| c.is_ascii_digit()))
}

/// Return `true` if `prev` or `next` is an ASCII alphabetic character.
fn is_letter_context(prev: Option<char>, next: Option<char>) -> bool {
    [prev, next]
        .iter()
        .any(|c| c.map_or(false, |c| c.is_ascii_alphabetic()))
}

/// Apply a single-character confusion correction.
///
/// Conservative: only correct when the target is unambiguous given the context.
fn correct_char(c: char, prev: Option<char>, next: Option<char>) -> char {
    match c {
        // Digit zero misread as letter O (or vice versa)
        '0' if is_letter_context(prev, next) => 'O',
        'O' if is_digit_context(prev, next) => '0',
        // Digit one misread as lowercase l (or vice versa)
        '1' if is_letter_context(prev, next) => 'l',
        'l' if is_digit_context(prev, next) => '1',
        // 曰 (U+66F0, classical "to say") is extremely rare in modern Japanese;
        // almost always a misread of 日 (sun/day/Japan).
        '曰' => '日',
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ocrs_cjk::TextItem;
    use rten_imageproc::Rect;

    fn make_char(c: char, confidence: f32) -> TextChar {
        TextChar {
            char: c,
            rect: Rect::from_tlhw(0, 0, 10, 10),
            confidence,
        }
    }

    fn make_line(chars: Vec<TextChar>) -> TextLine {
        TextLine::new(chars)
    }

    #[test]
    fn test_zero_to_o_in_letter_context() {
        // "H0llo" with low-confidence '0' → "HOllo"
        let line = make_line(vec![
            make_char('H', 0.99),
            make_char('0', 0.50),
            make_char('l', 0.99),
            make_char('l', 0.99),
            make_char('o', 0.99),
        ]);
        let result = apply_ja(&[Some(line)], 0.7);
        let corrected = result[0].as_ref().unwrap().to_string();
        assert_eq!(corrected, "HOllo");
    }

    #[test]
    fn test_o_to_zero_in_digit_context() {
        // "1O3" with low-confidence 'O' → "103"
        let line = make_line(vec![
            make_char('1', 0.99),
            make_char('O', 0.40),
            make_char('3', 0.99),
        ]);
        let result = apply_ja(&[Some(line)], 0.7);
        let corrected = result[0].as_ref().unwrap().to_string();
        assert_eq!(corrected, "103");
    }

    #[test]
    fn test_high_confidence_not_corrected() {
        // '0' with high confidence in letter context must not be corrected.
        let line = make_line(vec![
            make_char('H', 0.99),
            make_char('0', 0.95),  // high confidence → keep
            make_char('l', 0.99),
        ]);
        let result = apply_ja(&[Some(line)], 0.7);
        let corrected = result[0].as_ref().unwrap().to_string();
        assert_eq!(corrected, "H0l");
    }

    #[test]
    fn test_kuriwaku_to_hi() {
        // '曰' at any confidence → '日'
        let line = make_line(vec![
            make_char('曰', 0.60),
            make_char('本', 0.99),
        ]);
        let result = apply_ja(&[Some(line)], 0.7);
        let corrected = result[0].as_ref().unwrap().to_string();
        assert_eq!(corrected, "日本");
    }

    #[test]
    fn test_none_lines_preserved() {
        let input: Vec<Option<TextLine>> = vec![None];
        let result = apply_ja(&input, 0.7);
        assert!(result[0].is_none());
    }
}
