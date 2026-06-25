use std::fmt;
use std::fmt::Write;

use rten_imageproc::{bounding_rect, min_area_rect, Point, Rect, RotatedRect, Vec2};

use crate::cjk_text;

/// A non-empty sequence of recognized characters ([TextChar]) that constitute a
/// logical unit of a document such as a word or line.
pub trait TextItem {
    /// Return the sequence of characters that make up this item.
    fn chars(&self) -> &[TextChar];

    /// Return the mean recognition confidence across all characters, in [0, 1].
    fn confidence(&self) -> f32 {
        let chars = self.chars();
        if chars.is_empty() {
            return 0.0;
        }
        chars.iter().map(|c| c.confidence).sum::<f32>() / chars.len() as f32
    }

    /// Return the bounding rectangle of all characters in this item.
    fn bounding_rect(&self) -> Rect {
        bounding_rect(self.chars().iter().map(|c| &c.rect)).expect("expected valid rect")
    }

    /// Return the oriented bounding rectangle of all characters in this item.
    fn rotated_rect(&self) -> RotatedRect {
        let chars = self.chars();
        let mut points = Vec::with_capacity(chars.len() * 4);
        points.extend(chars.iter().flat_map(|c| c.rect.corners()).map(Point::to_f32));
        let rect = min_area_rect(&points).expect("expected valid rect");

        // Give the rect a predictable orientation. We currently assume the
        // text is horizontal and upright (ie. rotation angle < 180°).
        rect.orient_towards(Vec2::from_yx(-1., 0.))
    }
}

fn fmt_text_item<TI: TextItem>(item: &TI, f: &mut fmt::Formatter) -> fmt::Result {
    for c in item.chars().iter().map(|c| c.char) {
        f.write_char(c)?;
    }
    Ok(())
}

impl fmt::Display for TextLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_text_item(self, f)
    }
}

/// Details of a single character that was recognized.
#[derive(Clone)]
pub struct TextChar {
    /// Character that was recognized.
    pub char: char,

    /// Approximate bounding rectangle of character in input image.
    pub rect: Rect,

    /// Recognition confidence in [0, 1].
    pub confidence: f32,
}

/// Result of recognizing a line of text.
///
/// This includes the sequence of characters that were found and associated
/// metadata (eg. bounding boxes).
#[derive(Clone)]
pub struct TextLine {
    chars: Vec<TextChar>,
}

impl TextLine {
    /// Create a new text line which contains the given characters.
    ///
    /// Word boundaries are inferred from the presence of characters with
    /// [TextChar::char] values that are ASCII spaces.
    pub fn new(chars: Vec<TextChar>) -> TextLine {
        assert!(!chars.is_empty(), "Text lines must not be empty");
        TextLine { chars }
    }

    /// Return an iterator over words in this line.
    pub fn words(&self) -> impl Iterator<Item = TextWord<'_>> {
        self.chars()
            .split(|c| c.char == ' ')
            .filter(|chars| !chars.is_empty())
            .map(TextWord::new)
    }

    /// Return an iterator over CJK-aware segments of this line.
    ///
    /// Unlike [`words`](Self::words), this splits at script transitions
    /// (Latin↔CJK) without requiring a space delimiter. ASCII spaces are still
    /// treated as split points and are not included in any segment.
    ///
    /// For Latin-only text the output is identical to `words()`.
    pub fn segments(&self) -> impl Iterator<Item = TextWord<'_>> {
        TextSegmentIter {
            remaining: self.chars(),
        }
    }
}

impl TextItem for TextLine {
    /// Return the bounding rects of each character in the line.
    fn chars(&self) -> &[TextChar] {
        &self.chars
    }
}

struct TextSegmentIter<'a> {
    remaining: &'a [TextChar],
}

impl<'a> Iterator for TextSegmentIter<'a> {
    type Item = TextWord<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Single pass: find first non-space char and its index simultaneously.
        let start = self.remaining.iter().position(|c| c.char != ' ')?;
        let first_is_cjk = cjk_text::is_cjk(self.remaining[start].char);

        // Continue from start+1 — avoids re-scanning the leading spaces a second time.
        let end = self.remaining[start + 1..]
            .iter()
            .position(|c| c.char == ' ' || cjk_text::is_cjk(c.char) != first_is_cjk)
            .map(|p| start + 1 + p)
            .unwrap_or(self.remaining.len());

        let (seg, rest) = self.remaining.split_at(end);
        self.remaining = rest;
        Some(TextWord::new(&seg[start..]))
    }
}

/// Subsequence of a [TextLine] that contains a sequence of non-space characters.
pub struct TextWord<'a> {
    chars: &'a [TextChar],
}

impl<'a> TextWord<'a> {
    fn new(chars: &'a [TextChar]) -> TextWord<'a> {
        assert!(!chars.is_empty(), "Text words must not be empty");
        TextWord { chars }
    }
}

impl TextItem for TextWord<'_> {
    fn chars(&self) -> &[TextChar] {
        self.chars
    }
}

impl fmt::Display for TextWord<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_text_item(self, f)
    }
}

#[cfg(test)]
mod tests {
    use rten_imageproc::{BoundingRect, Point, Rect, Vec2};

    use super::{TextChar, TextItem, TextLine, TextWord};

    fn gen_text_chars(text: &str, width: i32) -> Vec<TextChar> {
        text.chars()
            .enumerate()
            .map(|(i, char)| TextChar {
                char,
                rect: Rect::from_tlhw(0, i as i32 * width, 25, width),
                confidence: 1.0,
            })
            .collect()
    }

    #[test]
    fn test_item_display() {
        let chars = gen_text_chars("foo bar baz", 10 /* char_width */);
        let line = TextLine::new(chars);
        assert_eq!(line.to_string(), "foo bar baz");
    }

    #[test]
    fn test_item_rotated_rect() {
        // Horizontal word case. The rotated rect and bounding rect are the same.
        let char_width = 10;
        let chars = gen_text_chars("foo", char_width);
        let word = TextWord::new(&chars);

        assert_eq!(
            word.bounding_rect(),
            Rect::from_tlhw(0, 0, 25, char_width * 3)
        );

        let rot_rect = word.rotated_rect();
        assert_eq!(rot_rect.bounding_rect(), word.bounding_rect().to_f32());
        assert_eq!(rot_rect.up_axis(), Vec2::from_yx(-1., 0.));
        assert_eq!(
            word.rotated_rect().corners(),
            [(25, 30), (25, 0), (0, 0), (0, 30)].map(|(y, x)| Point::from_yx(y as f32, x as f32))
        );

        // TODO - Add cases for non-horizontal words.
    }

    #[test]
    fn test_line_words() {
        let char_width = 10;
        let chars = gen_text_chars("foo bar  baz ", char_width);
        let line = TextLine::new(chars);
        let words: Vec<_> = line.words().collect();

        assert_eq!(words.len(), 3);
        assert_eq!(words[0].to_string(), "foo");
        assert_eq!(
            words[0].bounding_rect(),
            Rect::from_tlhw(0, 0, 25, char_width * 3)
        );

        assert_eq!(words[1].to_string(), "bar");
        assert_eq!(
            words[1].bounding_rect(),
            Rect::from_tlhw(0, char_width * 4, 25, char_width * 3)
        );

        assert_eq!(words[2].to_string(), "baz");
        assert_eq!(
            words[2].bounding_rect(),
            Rect::from_tlhw(0, char_width * 9, 25, char_width * 3)
        );
    }

    // ---- segments() ----

    #[test]
    fn test_segments_latin_matches_words() {
        let chars = gen_text_chars("foo bar baz", 10);
        let line = TextLine::new(chars);
        let segs: Vec<_> = line.segments().map(|w| w.to_string()).collect();
        let words: Vec<_> = line.words().map(|w| w.to_string()).collect();
        assert_eq!(segs, words);
    }

    #[test]
    fn test_segments_cjk_only() {
        let chars = gen_text_chars("日本語", 20);
        let line = TextLine::new(chars);
        let segs: Vec<_> = line.segments().collect();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].to_string(), "日本語");
    }

    #[test]
    fn test_segments_mixed_space_delimited() {
        let chars = gen_text_chars("Hello 世界", 15);
        let line = TextLine::new(chars);
        let segs: Vec<_> = line.segments().collect();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].to_string(), "Hello");
        assert_eq!(segs[1].to_string(), "世界");
    }

    #[test]
    fn test_segments_adjacent_script_boundary() {
        // No space — script boundary alone triggers split.
        let chars = gen_text_chars("OCR認識", 15);
        let line = TextLine::new(chars);
        let segs: Vec<_> = line.segments().collect();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].to_string(), "OCR");
        assert_eq!(segs[1].to_string(), "認識");
    }

    #[test]
    fn test_segments_rapid_transitions() {
        let chars = gen_text_chars("a世b", 15);
        let line = TextLine::new(chars);
        let segs: Vec<_> = line.segments().collect();
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0].to_string(), "a");
        assert_eq!(segs[1].to_string(), "世");
        assert_eq!(segs[2].to_string(), "b");
    }

    #[test]
    fn test_segments_double_space() {
        let chars = gen_text_chars("foo  bar", 10);
        let line = TextLine::new(chars);
        let segs: Vec<_> = line.segments().collect();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].to_string(), "foo");
        assert_eq!(segs[1].to_string(), "bar");
    }

    #[test]
    fn test_segments_bounding_rect_cjk() {
        let width = 20;
        let chars = gen_text_chars("ab日本cd", width);
        let line = TextLine::new(chars);
        let segs: Vec<_> = line.segments().collect();
        // "ab", "日本", "cd"
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[1].bounding_rect().left(), 2 * width);
        assert_eq!(segs[1].bounding_rect().right(), 4 * width);
    }
}
