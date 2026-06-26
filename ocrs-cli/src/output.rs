use rten_imageproc::{min_area_rect, Painter, Point, PointF, Rgb, RotatedRect};
use rten_tensor::prelude::*;
use rten_tensor::{NdTensor, NdTensorView};
use serde_json::json;

use ocrs_cjk::{TextItem, TextLine};

pub enum OutputFormat {
    /// Output a PNG image containing a copy of the input image annotated with
    /// text bounding boxes.
    Png,

    /// Output extracted plain text in reading order.
    Text,

    /// Output text and layout information in JSON format.
    Json,

    /// Output text in hOCR HTML format (includes bounding boxes and confidence).
    Hocr,

    /// Output text in ALTO XML format (includes bounding boxes and confidence).
    Alto,
}

/// Return the coordinates of vertices of `rr` as an array of `[x, y]` points.
///
/// This matches the format of the "vertices" data in the HierText dataset.
/// See [RotatedRect::corners] for details of the vertex order.
fn rounded_vertex_coords(rr: &RotatedRect) -> [[i32; 2]; 4] {
    rr.corners()
        .map(|point| [point.x.round() as i32, point.y.round() as i32])
}

/// Build a `paragraphs` JSON value from a flat list of text lines.
fn paragraphs_json(text_lines: &[Option<TextLine>]) -> serde_json::Value {
    let line_items: Vec<_> = text_lines
        .iter()
        .filter_map(|line| line.as_ref())
        .map(|line| {
            let word_items: Vec<_> = line
                .words()
                .map(|word| {
                    json!({
                        "text": word.to_string(),
                        "confidence": (word.confidence() * 100.0).round() / 100.0,
                        "vertices": rounded_vertex_coords(&word.rotated_rect()),
                    })
                })
                .collect();
            json!({
                "text": line.to_string(),
                "words": word_items,
                "vertices": rounded_vertex_coords(&line.rotated_rect()),
            })
        })
        .collect();
    // nb. Since we haven't got layout analysis info here, we just put all
    // the lines on one paragraph.
    json!([{ "lines": serde_json::Value::Array(line_items) }])
}

/// Format extracted text and hierarchical layout information as JSON.
///
/// The JSON format roughly follows the structure of the ground truth data in
/// the [HierText](https://github.com/google-research-datasets/hiertext)
/// dataset, on which ocrs's models were trained.
fn ocr_json(args: FormatJsonArgs) -> serde_json::Value {
    let [height, width] = args.input_hw;
    json!({
        "url": args.input_path,
        "image_width": width,
        "image_height": height,
        "paragraphs": paragraphs_json(args.text_lines),
    })
}

fn hocr_header() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
     <!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\"\n\
       \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd\">\n\
     <html xmlns=\"http://www.w3.org/1999/xhtml\" xml:lang=\"en\" lang=\"en\">\n\
     <head>\n\
       <meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\"/>\n\
       <meta name=\"ocr-system\" content=\"ocrs-cjk\"/>\n\
       <meta name=\"ocr-capabilities\" content=\"ocr_page ocr_line ocrx_word\"/>\n\
     </head>\n\
     <body>\n"
    .to_string()
}

/// Render one hOCR `<div class="ocr_page">` block for a single page.
fn hocr_page_div(
    path: &str,
    hw: [usize; 2],
    lines: &[Option<TextLine>],
    page_idx: usize,
) -> String {
    let [height, width] = hw;
    let path = html_escape(path);
    let mut out = format!(
        "  <div class=\"ocr_page\" id=\"page_{page_idx}\" \
            title=\"image &quot;{path}&quot;; bbox 0 0 {width} {height}; ppageno {page_idx}\">\n"
    );
    for (line_idx, line) in lines.iter().flatten().enumerate() {
        let lb = line.bounding_rect();
        out.push_str(&format!(
            "    <span class=\"ocr_line\" id=\"p{page_idx}_line_{line_idx}\" \
                title=\"bbox {} {} {} {}\">\n",
            lb.left(),
            lb.top(),
            lb.right(),
            lb.bottom()
        ));
        for (word_idx, word) in line.words().enumerate() {
            let wb = word.bounding_rect();
            let conf = (word.confidence() * 100.0).round() as u32;
            let text = html_escape(&word.to_string());
            out.push_str(&format!(
                "      <span class=\"ocrx_word\" id=\"p{page_idx}_word_{line_idx}_{word_idx}\" \
                    title=\"bbox {} {} {} {}; x_wconf {conf}\">{text}</span>\n",
                wb.left(),
                wb.top(),
                wb.right(),
                wb.bottom(),
            ));
        }
        out.push_str("    </span>\n");
    }
    out.push_str("  </div>\n");
    out
}

/// Render one ALTO `<Page>` element for a single page.
fn alto_page_elem(hw: [usize; 2], lines: &[Option<TextLine>], page_idx: usize) -> String {
    let [height, width] = hw;
    let mut out = format!(
        "    <Page ID=\"page_{page_idx}\" WIDTH=\"{width}\" HEIGHT=\"{height}\">\n\
           <PrintSpace HPOS=\"0\" VPOS=\"0\" WIDTH=\"{width}\" HEIGHT=\"{height}\">\n\
             <TextBlock ID=\"p{page_idx}_block_0\" HPOS=\"0\" VPOS=\"0\" WIDTH=\"{width}\" HEIGHT=\"{height}\">\n"
    );
    for (line_idx, line) in lines.iter().flatten().enumerate() {
        let lb = line.bounding_rect();
        out.push_str(&format!(
            "               <TextLine ID=\"p{page_idx}_line_{line_idx}\" \
                HPOS=\"{}\" VPOS=\"{}\" WIDTH=\"{}\" HEIGHT=\"{}\">\n",
            lb.left(),
            lb.top(),
            lb.width(),
            lb.height()
        ));
        for (word_idx, word) in line.words().enumerate() {
            let wb = word.bounding_rect();
            let conf = (word.confidence() * 1000.0).round() / 1000.0;
            let content = html_escape(&word.to_string());
            out.push_str(&format!(
                "                 <String ID=\"p{page_idx}_word_{line_idx}_{word_idx}\" \
                    HPOS=\"{}\" VPOS=\"{}\" WIDTH=\"{}\" HEIGHT=\"{}\" \
                    WC=\"{conf:.3}\" CONTENT=\"{content}\"/>\n",
                wb.left(),
                wb.top(),
                wb.width(),
                wb.height(),
            ));
        }
        out.push_str("               </TextLine>\n");
    }
    out.push_str(
        "             </TextBlock>\n\
           </PrintSpace>\n\
         </Page>\n",
    );
    out
}

/// Input data for [format_json_output].
pub struct FormatJsonArgs<'a> {
    pub input_path: &'a str,
    pub input_hw: [usize; 2],

    /// Lines of text recognized by OCR engine.
    pub text_lines: &'a [Option<TextLine>],
}

/// Format OCR outputs as plain text.
pub fn format_text_output(text_lines: &[Option<TextLine>]) -> String {
    let lines: Vec<String> = text_lines
        .iter()
        .flatten()
        .map(|line| line.to_string())
        .collect();
    lines.join("\n")
}

/// Format OCR outputs as JSON.
pub fn format_json_output(args: FormatJsonArgs) -> String {
    let json_data = ocr_json(args);
    serde_json::to_string_pretty(&json_data).expect("JSON formatting failed")
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Format OCR outputs as hOCR HTML.
///
/// The hOCR format embeds recognized text and bounding boxes in an HTML
/// document using `ocr_page`, `ocr_line`, and `ocrx_word` CSS classes with
/// `title` attributes containing `bbox` and `x_wconf` metadata.
pub fn format_hocr_output(args: FormatJsonArgs) -> String {
    let mut out = hocr_header();
    out.push_str(&hocr_page_div(args.input_path, args.input_hw, args.text_lines, 0));
    out.push_str("</body>\n</html>\n");
    out
}

/// Format OCR outputs as ALTO XML.
///
/// Produces an ALTO v4 XML document with `TextLine` and `String` elements
/// containing bounding boxes (`HPOS`, `VPOS`, `WIDTH`, `HEIGHT`) and
/// word confidence (`WC`).
pub fn format_alto_output(args: FormatJsonArgs) -> String {
    let mut out =
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <alto xmlns=\"http://www.loc.gov/standards/alto/ns-v4#\">\n\
           <Layout>\n"
        .to_string();
    out.push_str(&alto_page_elem(args.input_hw, args.text_lines, 0));
    out.push_str("  </Layout>\n</alto>\n");
    out
}

/// Per-page data for PDF output functions.
pub struct PageInfo<'a> {
    pub image_hw: [usize; 2],
    pub text_lines: &'a [Option<TextLine>],
}

/// Format multi-page PDF OCR output as JSON with per-page dimensions.
///
/// Unlike [`format_json_output`], this produces a `"pages"` array so each
/// page can carry its own `image_width` / `image_height`.
pub fn format_json_pdf_output(input_path: &str, pages: &[PageInfo]) -> String {
    let page_values: Vec<serde_json::Value> = pages
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let [height, width] = p.image_hw;
            json!({
                "page": i + 1,
                "image_width": width,
                "image_height": height,
                "paragraphs": paragraphs_json(p.text_lines),
            })
        })
        .collect();
    let doc = json!({ "url": input_path, "pages": page_values });
    serde_json::to_string_pretty(&doc).expect("JSON formatting failed")
}

/// Format multi-page PDF OCR output as hOCR with one `ocr_page` div per page.
pub fn format_hocr_pdf_output(input_path: &str, pages: &[PageInfo]) -> String {
    let mut out = hocr_header();
    for (i, p) in pages.iter().enumerate() {
        out.push_str(&hocr_page_div(input_path, p.image_hw, p.text_lines, i));
    }
    out.push_str("</body>\n</html>\n");
    out
}

/// Format multi-page PDF OCR output as ALTO XML with one `<Page>` per page.
pub fn format_alto_pdf_output(pages: &[PageInfo]) -> String {
    let mut out =
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <alto xmlns=\"http://www.loc.gov/standards/alto/ns-v4#\">\n\
           <Layout>\n"
        .to_string();
    for (i, p) in pages.iter().enumerate() {
        out.push_str(&alto_page_elem(p.image_hw, p.text_lines, i));
    }
    out.push_str("  </Layout>\n</alto>\n");
    out
}

/// Arguments for [generate_annotated_png].
pub struct GeneratePngArgs<'a> {
    /// Input image as a (height, width, channels) tensor.
    pub img: NdTensorView<'a, u8, 3>,

    /// Lines of text detected by OCR engine.
    pub line_rects: &'a [Vec<RotatedRect>],

    /// Lines of text recognized by OCR engine.
    pub text_lines: &'a [Option<TextLine>],
}

/// Annotate OCR input image with detected text.
pub fn generate_annotated_png(args: GeneratePngArgs) -> NdTensor<f32, 3> {
    let GeneratePngArgs {
        img,
        line_rects,
        text_lines,
    } = args;
    // HWC u8 => CHW f32
    let mut annotated_img = img.permuted([2, 0, 1]).map(|pixel| *pixel as f32 / 255.0);
    let mut painter = Painter::new(annotated_img.view_mut());

    // Colors chosen from https://www.w3.org/wiki/CSS/Properties/color/keywords.
    //
    // Light colors for text detection outputs, darker colors for
    // corresponding text recognition outputs.
    const CORAL: Rgb = [255, 127, 80];
    const DARKSEAGREEN: Rgb = [143, 188, 143];
    const CORNFLOWERBLUE: Rgb = [100, 149, 237];

    const CRIMSON: Rgb = [220, 20, 60];
    const DARKGREEN: Rgb = [0, 100, 0];
    const DARKBLUE: Rgb = [0, 0, 139];

    const LIGHT_GRAY: Rgb = [200, 200, 200];

    let u8_to_f32 = |x: u8| x as f32 / 255.;
    let floor_point = |p: PointF| Point::from_yx(p.y as i32, p.x as i32);

    // Draw line bounding rects from layout analysis step.
    for line in line_rects {
        let line_points: Vec<_> = line
            .iter()
            .flat_map(|word_rect| word_rect.corners().into_iter())
            .collect();
        if let Some(line_rect) = min_area_rect(&line_points) {
            painter.set_stroke(LIGHT_GRAY.map(u8_to_f32));
            painter.draw_polygon(&line_rect.corners().map(floor_point));
        };
    }

    // Draw word bounding rects from text detection step, grouped by line.
    let colors = [CORAL, DARKSEAGREEN, CORNFLOWERBLUE];
    for (line, color) in line_rects.iter().zip(colors.into_iter().cycle()) {
        for word_rect in line {
            painter.set_stroke(color.map(u8_to_f32));
            painter.draw_polygon(&word_rect.corners().map(floor_point));
        }
    }

    // Draw word bounding rects from text recognition step. These may be
    // different as they are computed from the bounding boxes of recognized
    // characters.
    let colors = [CRIMSON, DARKGREEN, DARKBLUE];
    for (line, color) in text_lines.iter().zip(colors.into_iter().cycle()) {
        let Some(line) = line else {
            // Skip lines where recognition produced no output.
            continue;
        };
        for text_word in line.words() {
            painter.set_stroke(color.map(u8_to_f32));
            painter.draw_polygon(&text_word.rotated_rect().corners().map(floor_point));
        }
    }

    annotated_img
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use std::io;
    use std::path::PathBuf;

    use ocrs_cjk::{TextChar, TextItem, TextLine};
    use rten_imageproc::Rect;
    use rten_tensor::prelude::*;
    use rten_tensor::NdTensor;

    use super::{
        format_json_output, format_text_output, generate_annotated_png, FormatJsonArgs,
        GeneratePngArgs,
    };

    /// Generate dummy OCR output with the given text and character spacing.
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

    fn read_test_file(path: &str) -> Result<String, io::Error> {
        let mut abs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        abs_path.push("test-data/");
        abs_path.push(path);
        read_to_string(abs_path)
    }

    #[test]
    fn test_format_json_output() {
        let lines = &[
            Some(TextLine::new(gen_text_chars("line one", 10))),
            None,
            Some(TextLine::new(gen_text_chars("line two", 10))),
        ];

        let json = format_json_output(FormatJsonArgs {
            input_path: "image.jpeg",
            input_hw: [256, 256],
            text_lines: lines,
        });
        let parsed_json: serde_json::Value = serde_json::from_str(&json).unwrap();

        let expected_json = read_test_file("format-json-expected.json").unwrap();
        let expected: serde_json::Value = serde_json::from_str(&expected_json).unwrap();
        assert_eq!(parsed_json, expected);
    }

    #[test]
    fn test_format_text_output() {
        let lines = &[
            Some(TextLine::new(gen_text_chars("line one", 10))),
            None,
            Some(TextLine::new(gen_text_chars("line two", 10))),
        ];
        let formatted = format_text_output(lines);
        let formatted_lines: Vec<_> = formatted.lines().collect();

        assert_eq!(formatted_lines, ["line one", "line two",]);
    }

    #[test]
    fn test_generate_annotated_png() {
        let img = NdTensor::zeros([64, 64, 3]);
        let text_lines = &[
            Some(TextLine::new(gen_text_chars("line one", 10))),
            Some(TextLine::new(gen_text_chars("line one", 10))),
        ];

        let line_rects: Vec<_> = text_lines
            .iter()
            .filter_map(|line| line.clone().map(|l| vec![l.rotated_rect()]))
            .collect();

        let args = GeneratePngArgs {
            img: img.view(),
            line_rects: &line_rects,
            text_lines,
        };

        let annotated = generate_annotated_png(args);

        assert_eq!(annotated.shape(), img.permuted([2, 0, 1]).shape());
    }

    use super::{format_alto_output, format_hocr_output};

    #[test]
    fn test_hocr_cjk_no_panic() {
        // CJK characters must not cause panics in HTML generation or escaping.
        let lines = &[
            Some(TextLine::new(gen_text_chars("東京オリンピック2024", 20))),
            Some(TextLine::new(gen_text_chars("日本語テスト", 20))),
        ];
        let hocr = format_hocr_output(FormatJsonArgs {
            input_path: "test_cjk.png",
            input_hw: [80, 600],
            text_lines: lines,
        });
        assert!(hocr.contains("<?xml"));
        assert!(hocr.contains("ocr_page"));
        assert!(hocr.contains("ocrx_word"));
        // CJK text must appear verbatim (no garbling or escaping of CJK chars).
        assert!(hocr.contains("東京オリンピック2024"));
        assert!(hocr.contains("日本語テスト"));
        // Confidence must appear as x_wconf integer.
        assert!(hocr.contains("x_wconf 100"));
    }

    #[test]
    fn test_alto_cjk_no_panic() {
        // CJK characters must appear correctly in ALTO XML output.
        let lines = &[
            Some(TextLine::new(gen_text_chars("東京オリンピック2024", 20))),
        ];
        let alto = format_alto_output(FormatJsonArgs {
            input_path: "test_cjk.png",
            input_hw: [80, 600],
            text_lines: lines,
        });
        assert!(alto.contains("<?xml"));
        assert!(alto.contains("<alto"));
        assert!(alto.contains("<String"));
        assert!(alto.contains("東京オリンピック2024"));
        // WC confidence must be a float between 0 and 1.
        assert!(alto.contains("WC=\"1.000\""));
    }

    #[test]
    fn test_hocr_html_escape() {
        // Characters that need HTML escaping (<, >, &, ") must be escaped.
        let lines = &[Some(TextLine::new(gen_text_chars(
            "price < $5 & quality > \"good\"",
            10,
        )))];
        let hocr = format_hocr_output(FormatJsonArgs {
            input_path: "test.png",
            input_hw: [100, 600],
            text_lines: lines,
        });
        assert!(!hocr.contains(" < "), "raw '<' must be escaped");
        assert!(!hocr.contains(" & "), "raw '&' must be escaped");
        assert!(hocr.contains("&lt;"));
        assert!(hocr.contains("&amp;"));
    }

    #[test]
    fn test_confidence_aggregation() {
        use ocrs_cjk::TextItem;

        // Chars with varying confidence: mean should be computed correctly.
        let chars = vec![
            TextChar {
                char: '東',
                rect: Rect::from_tlhw(0, 0, 25, 20),
                confidence: 0.9,
            },
            TextChar {
                char: '京',
                rect: Rect::from_tlhw(0, 20, 25, 20),
                confidence: 0.8,
            },
        ];
        let line = TextLine::new(chars);
        let conf = line.confidence();
        assert!(
            (conf - 0.85).abs() < 1e-5,
            "expected mean confidence 0.85, got {conf}"
        );
    }

    #[test]
    fn test_json_confidence_field_present() {
        let lines = &[Some(TextLine::new(gen_text_chars("東京", 20)))];
        let json = format_json_output(FormatJsonArgs {
            input_path: "test.png",
            input_hw: [80, 100],
            text_lines: lines,
        });
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let words = &parsed["paragraphs"][0]["lines"][0]["words"];
        assert!(words.is_array());
        let first_word = &words[0];
        assert!(
            first_word.get("confidence").is_some(),
            "confidence field missing from JSON word"
        );
        let conf = first_word["confidence"].as_f64().unwrap();
        assert!(
            (0.0..=1.0).contains(&conf),
            "confidence {conf} out of range [0, 1]"
        );
    }
}
