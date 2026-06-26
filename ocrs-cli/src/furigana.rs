use rten_imageproc::Rect;

use ocrs_cjk::{TextItem, TextLine};

pub struct RubyLine {
    pub text: String,
    pub ruby: Option<String>,
    pub bbox: Rect,
}

fn is_kana(c: char) -> bool {
    matches!(c, '\u{3040}'..='\u{309F}' | '\u{30A0}'..='\u{30FF}')
}

/// Detect and separate furigana (ルビ) from main text lines.
///
/// Returns a list of `RubyLine` where furigana lines are folded into
/// their parent line's `ruby` field. Lines without associated furigana
/// have `ruby = None`.
///
/// Detection heuristics:
/// - A line is furigana if its median character height < 0.6 × document
///   median character height AND more than 60% of its characters are kana.
/// # ponytail
/// 0.6/0.6 thresholds work for standard Japanese typesetting. Tune if
/// manga with large kana or mixed-size text causes false positives.
pub fn detect_ruby(lines: &[Option<TextLine>]) -> Vec<RubyLine> {
    let valid_lines: Vec<&TextLine> = lines.iter().flatten().collect();
    if valid_lines.is_empty() {
        return vec![];
    }

    // Step 1: compute document-wide reference height as median of per-line medians.
    // Using per-line medians avoids furigana characters pulling down the overall
    // median and making the threshold too small to detect them.
    let mut line_medians: Vec<i32> = valid_lines
        .iter()
        .filter_map(|l| {
            let mut heights: Vec<i32> = l
                .chars()
                .iter()
                .filter(|c| c.char != ' ')
                .map(|c| c.rect.height())
                .collect();
            if heights.is_empty() {
                return None;
            }
            heights.sort_unstable();
            Some(heights[heights.len() / 2])
        })
        .collect();
    if line_medians.is_empty() {
        return valid_lines
            .iter()
            .map(|l| RubyLine {
                text: l.to_string(),
                ruby: None,
                bbox: l.bounding_rect(),
            })
            .collect();
    }
    line_medians.sort_unstable();
    let doc_median_h = line_medians[line_medians.len() / 2] as f32;

    // Step 2: classify each line as furigana or main text.
    let is_furigana: Vec<bool> = valid_lines
        .iter()
        .map(|line| {
            let non_space: Vec<_> = line.chars().iter().filter(|c| c.char != ' ').collect();
            if non_space.is_empty() {
                return false;
            }
            // Median char height for this line
            let mut heights: Vec<i32> = non_space.iter().map(|c| c.rect.height()).collect();
            heights.sort_unstable();
            let line_h = heights[heights.len() / 2] as f32;
            let kana_count = non_space.iter().filter(|c| is_kana(c.char)).count();
            let kana_ratio = kana_count as f32 / non_space.len() as f32;
            line_h < 0.6 * doc_median_h && kana_ratio > 0.6
        })
        .collect();

    // Step 3: for each furigana line, find the closest non-furigana parent
    // that has horizontal overlap.
    let mut ruby_assignments: Vec<Option<usize>> = vec![None; valid_lines.len()];
    for (fi, line) in valid_lines.iter().enumerate() {
        if !is_furigana[fi] {
            continue;
        }
        let fb = line.bounding_rect();
        let f_cx = fb.top() + fb.height() / 2;
        let mut best_idx: Option<usize> = None;
        let mut best_dist = i32::MAX;
        for (mi, main) in valid_lines.iter().enumerate() {
            if is_furigana[mi] {
                continue;
            }
            let mb = main.bounding_rect();
            // Check horizontal overlap
            if fb.right() <= mb.left() || mb.right() <= fb.left() {
                continue;
            }
            let m_cx = mb.top() + mb.height() / 2;
            let dist = (f_cx - m_cx).abs();
            if dist < best_dist {
                best_dist = dist;
                best_idx = Some(mi);
            }
        }
        ruby_assignments[fi] = best_idx;
    }

    // Step 4: build RubyLine output.
    // For each main line, collect associated furigana strings.
    let mut ruby_map: Vec<Vec<String>> = vec![vec![]; valid_lines.len()];
    for (fi, &parent) in ruby_assignments.iter().enumerate() {
        if let Some(mi) = parent {
            ruby_map[mi].push(valid_lines[fi].to_string());
        }
    }

    valid_lines
        .iter()
        .enumerate()
        .filter(|(i, _)| !is_furigana[*i])
        .map(|(i, line)| {
            let ruby = if ruby_map[i].is_empty() {
                None
            } else {
                Some(ruby_map[i].join(""))
            };
            RubyLine {
                text: line.to_string(),
                ruby,
                bbox: line.bounding_rect(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rten_imageproc::Rect;

    use ocrs_cjk::{TextChar, TextLine};

    use super::*;

    fn make_line(chars: &[(char, i32, i32, i32, i32)]) -> TextLine {
        // (char, top, left, height, width)
        let chars: Vec<TextChar> = chars
            .iter()
            .map(|&(ch, top, left, h, w)| TextChar {
                char: ch,
                rect: Rect::from_tlhw(top, left, h, w),
                confidence: 1.0,
            })
            .collect();
        TextLine::new(chars)
    }

    #[test]
    fn test_no_furigana_in_normal_text() {
        let line = make_line(&[('東', 0, 0, 20, 20), ('京', 0, 20, 20, 20)]);
        let lines = vec![Some(line)];
        let result = detect_ruby(&lines);
        assert_eq!(result.len(), 1);
        assert!(result[0].ruby.is_none());
    }

    #[test]
    fn test_furigana_detected_and_associated() {
        // Main text: kanji, height=20; furigana: kana, height=8 (< 0.6*20=12), above main
        let main = make_line(&[('漢', 30, 0, 20, 20), ('字', 30, 20, 20, 20)]);
        let ruby = make_line(&[('か', 0, 0, 8, 8), ('ん', 0, 8, 8, 8), ('じ', 0, 16, 8, 8)]);
        let lines = vec![Some(main), Some(ruby)];
        let result = detect_ruby(&lines);
        // Furigana line should be consumed, leaving only 1 output line
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, "漢字");
        assert_eq!(result[0].ruby.as_deref(), Some("かんじ"));
    }
}
