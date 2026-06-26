use rten_imageproc::Rect;
use serde_json::{json, Value};

use ocrs_cjk::{TextItem, TextLine};

pub struct Table {
    pub rows: Vec<Vec<String>>,
    pub num_cols: usize,
    pub bbox: Rect,
}

/// Detect table-like text regions from recognized lines.
///
/// A table is ≥2 consecutive lines sharing ≥1 column gutter (persistent
/// x-gap across all lines) that produces ≥2 columns.
pub fn detect_tables(lines: &[Option<TextLine>]) -> Vec<Table> {
    let mut tables = Vec::new();
    for group in group_lines(lines) {
        let median_h = median_line_height(&group);
        // ponytail: 0.8× line_height as min column gutter; tunable if false positives appear
        let min_gap = (0.8 * median_h as f32) as i32;
        let separators = find_column_separators(&group, min_gap);
        if separators.is_empty() {
            continue;
        }
        let col_bands = column_bands(&group, &separators);
        if col_bands.len() < 2 {
            continue;
        }
        tables.push(Table {
            num_cols: col_bands.len(),
            rows: build_rows(&group, &col_bands),
            bbox: group_bbox(&group),
        });
    }
    tables
}

/// Group consecutive Some(TextLine) entries where the inter-line gap is tight.
fn group_lines(lines: &[Option<TextLine>]) -> Vec<Vec<&TextLine>> {
    let mut groups: Vec<Vec<&TextLine>> = Vec::new();
    let mut current: Vec<&TextLine> = Vec::new();

    for line in lines {
        match line {
            None => flush(&mut current, &mut groups),
            Some(tl) => {
                if let Some(&prev) = current.last() {
                    let gap = tl.bounding_rect().top() - prev.bounding_rect().bottom();
                    // ponytail: 0.5× line_height max gap to stay in same table group
                    if gap > (0.5 * prev.bounding_rect().height() as f32) as i32 {
                        flush(&mut current, &mut groups);
                    }
                }
                current.push(tl);
            }
        }
    }
    flush(&mut current, &mut groups);
    groups
}

fn flush<'a>(current: &mut Vec<&'a TextLine>, groups: &mut Vec<Vec<&'a TextLine>>) {
    if current.len() >= 2 {
        groups.push(std::mem::take(current));
    } else {
        current.clear();
    }
}

/// Group text lines into logical blocks (paragraphs) for blocks-JSON output.
///
/// A new block starts when the vertical gap between consecutive lines exceeds
/// 1.5× the previous line's height. Single-line blocks are included.
pub fn group_into_blocks(lines: &[Option<TextLine>]) -> Vec<Vec<&TextLine>> {
    let mut blocks: Vec<Vec<&TextLine>> = Vec::new();
    let mut current: Vec<&TextLine> = Vec::new();
    for line in lines {
        match line {
            None => {
                if !current.is_empty() {
                    blocks.push(std::mem::take(&mut current));
                }
            }
            Some(tl) => {
                if let Some(&prev) = current.last() {
                    let gap = tl.bounding_rect().top() - prev.bounding_rect().bottom();
                    // ponytail: 1.5× line_height as inter-paragraph gap; tune if grouping is too coarse
                    if gap > (1.5 * prev.bounding_rect().height() as f32) as i32 {
                        if !current.is_empty() {
                            blocks.push(std::mem::take(&mut current));
                        }
                    }
                }
                current.push(tl);
            }
        }
    }
    if !current.is_empty() {
        blocks.push(current);
    }
    blocks
}

/// Find x-positions of column separators by merging char coverage and locating gaps.
fn find_column_separators(group: &[&TextLine], min_gap_width: i32) -> Vec<i32> {
    let mut intervals: Vec<(i32, i32)> = group
        .iter()
        .flat_map(|line| {
            line.chars()
                .iter()
                .map(|c| (c.rect.left(), c.rect.right()))
        })
        .filter(|(l, r)| l < r)
        .collect();

    if intervals.is_empty() {
        return vec![];
    }

    intervals.sort_unstable_by_key(|&(l, _)| l);

    // Merge overlapping intervals
    let mut merged: Vec<(i32, i32)> = Vec::new();
    for (l, r) in intervals {
        if let Some(last) = merged.last_mut() {
            if l <= last.1 {
                last.1 = last.1.max(r);
                continue;
            }
        }
        merged.push((l, r));
    }

    // Gaps between merged intervals with sufficient width become separators
    merged
        .windows(2)
        .filter_map(|pair| {
            let gap_start = pair[0].1;
            let gap_end = pair[1].0;
            let width = gap_end - gap_start;
            (width >= min_gap_width).then_some(gap_start + width / 2)
        })
        .collect()
}

/// Build column bands (x_left, x_right) from the overall x-range and separator midpoints.
fn column_bands(group: &[&TextLine], separators: &[i32]) -> Vec<(i32, i32)> {
    let x_min = group
        .iter()
        .flat_map(|l| l.chars().iter().map(|c| c.rect.left()))
        .min()
        .unwrap_or(0);
    let x_max = group
        .iter()
        .flat_map(|l| l.chars().iter().map(|c| c.rect.right()))
        .max()
        .unwrap_or(0);

    let mut bands = Vec::with_capacity(separators.len() + 1);
    let mut band_left = x_min;
    for &sep in separators {
        bands.push((band_left, sep));
        band_left = sep;
    }
    bands.push((band_left, x_max));
    bands
}

/// Assign each char in each line to a column band, building the row×col grid.
fn build_rows(group: &[&TextLine], col_bands: &[(i32, i32)]) -> Vec<Vec<String>> {
    group
        .iter()
        .map(|line| {
            let mut cells: Vec<String> = vec![String::new(); col_bands.len()];
            let mut sorted: Vec<_> = line.chars().iter().collect();
            sorted.sort_unstable_by_key(|c| c.rect.left());
            for c in sorted {
                let cx = c.rect.left() + c.rect.width() / 2;
                // Binary search: first band where cx <= right edge
                let col = col_bands.partition_point(|&(_, r)| cx > r);
                if col < col_bands.len() && cx >= col_bands[col].0 {
                    cells[col].push(c.char);
                }
            }
            cells.iter_mut().for_each(|s| {
                let t = s.trim().to_string();
                *s = t;
            });
            cells
        })
        .collect()
}

fn median_line_height(group: &[&TextLine]) -> i32 {
    let mut heights: Vec<i32> = group.iter().map(|l| l.bounding_rect().height()).collect();
    heights.sort_unstable();
    heights[heights.len() / 2]
}

fn group_bbox(group: &[&TextLine]) -> Rect {
    let top = group.iter().map(|l| l.bounding_rect().top()).min().unwrap();
    let left = group.iter().map(|l| l.bounding_rect().left()).min().unwrap();
    let bottom = group.iter().map(|l| l.bounding_rect().bottom()).max().unwrap();
    let right = group.iter().map(|l| l.bounding_rect().right()).max().unwrap();
    Rect::from_tlhw(top, left, bottom - top, right - left)
}

// ── Output formatters ────────────────────────────────────────────────────────

pub fn format_tables_csv(tables: &[Table]) -> String {
    let mut out = String::new();
    for (i, table) in tables.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        for row in &table.rows {
            let line: Vec<String> = row.iter().map(|c| csv_escape(c)).collect();
            out.push_str(&line.join(","));
            out.push('\n');
        }
    }
    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn format_tables_markdown(tables: &[Table]) -> String {
    let mut out = String::new();
    for table in tables {
        if table.rows.is_empty() {
            continue;
        }
        let n = table.num_cols;
        let md_row = |row: &[String]| -> String {
            let mut s = String::from("|");
            for i in 0..n {
                let cell = row.get(i).map(String::as_str).unwrap_or("");
                s.push_str(&format!(" {} |", cell.replace('|', "\\|")));
            }
            s
        };
        out.push_str(&md_row(&table.rows[0]));
        out.push('\n');
        // separator row
        out.push('|');
        for _ in 0..n {
            out.push_str(" --- |");
        }
        out.push('\n');
        for row in table.rows.iter().skip(1) {
            out.push_str(&md_row(row));
            out.push('\n');
        }
        out.push('\n');
    }
    out
}

pub fn tables_json(tables: &[Table]) -> Value {
    json!(tables
        .iter()
        .map(|t| json!({
            "rows": t.rows,
            "bbox": {
                "x": t.bbox.left(),
                "y": t.bbox.top(),
                "width": t.bbox.width(),
                "height": t.bbox.height(),
            }
        }))
        .collect::<Vec<_>>())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use rten_imageproc::Rect;

    use ocrs_cjk::TextLine;

    use super::*;

    fn make_line(cells: &[(&str, i32)], top: i32, height: i32) -> TextLine {
        // cells: (text, x_start), char_width inferred as 10
        let char_width = 10i32;
        let chars: Vec<_> = cells
            .iter()
            .flat_map(|(text, x_start)| {
                text.chars().enumerate().map(move |(i, ch)| ocrs_cjk::TextChar {
                    char: ch,
                    rect: Rect::from_tlhw(top, x_start + i as i32 * char_width, height, char_width),
                    confidence: 1.0,
                })
            })
            .collect();
        TextLine::new(chars)
    }

    #[test]
    fn test_detect_two_column_table() {
        // Col A: x=0–30, Col B: x=110–140, gutter x=30–110 = width 80 >> 0.8×25=20
        let row1 = make_line(&[("abc", 0), ("def", 110)], 0, 25);
        let row2 = make_line(&[("ghi", 0), ("jkl", 110)], 30, 25);
        let lines: Vec<Option<TextLine>> = vec![Some(row1), Some(row2)];

        let tables = detect_tables(&lines);
        assert_eq!(tables.len(), 1);
        let t = &tables[0];
        assert_eq!(t.num_cols, 2);
        assert_eq!(t.rows[0][0], "abc");
        assert_eq!(t.rows[0][1], "def");
        assert_eq!(t.rows[1][0], "ghi");
        assert_eq!(t.rows[1][1], "jkl");
    }

    #[test]
    fn test_empty_cell() {
        // Row 2 has text only in col A
        let row1 = make_line(&[("abc", 0), ("def", 110)], 0, 25);
        let row2 = make_line(&[("ghi", 0)], 30, 25);
        let lines: Vec<Option<TextLine>> = vec![Some(row1), Some(row2)];

        let tables = detect_tables(&lines);
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].rows[1][0], "ghi");
        assert_eq!(tables[0].rows[1][1], ""); // empty cell
    }

    #[test]
    fn test_no_table_single_column() {
        // No column gap — just a paragraph
        let row1 = make_line(&[("hello world", 0)], 0, 25);
        let row2 = make_line(&[("foo bar baz", 0)], 30, 25);
        let lines: Vec<Option<TextLine>> = vec![Some(row1), Some(row2)];

        let tables = detect_tables(&lines);
        assert!(tables.is_empty());
    }
}
