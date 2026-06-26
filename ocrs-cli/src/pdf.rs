//! PDF image extraction and searchable PDF generation.
//!
//! Supports image-PDF (scanned) input: extracts raster images from each page,
//! runs them through OCR, then writes an invisible text overlay back to the
//! original PDF so the result is searchable in standard PDF viewers.

use std::collections::HashMap;

use anyhow::Context;
use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, ObjectId, Stream, StringFormat};
use ocrs_cjk::TextItem;
use rten_tensor::NdTensor;

/// A raster image extracted from one PDF page, plus the page's dimensions
/// in PDF points (1 pt = 1/72 inch).
pub struct PageImage {
    /// HWC u8 tensor (RGB).
    pub tensor: NdTensor<u8, 3>,
    /// Page width in PDF points.
    pub width_pts: f32,
    /// Page height in PDF points.
    pub height_pts: f32,
    /// True if the page has a native text layer (Font resources in page dictionary).
    pub has_text_layer: bool,
}

/// Return true if the page has a non-empty Font resource dictionary, indicating
/// it contains rendered text (as opposed to a pure scanned raster image).
///
/// # ponytail
/// Font-resource presence is a reliable proxy. Upgrade to BT/ET content-stream
/// scanning if PDFs with decorative fonts but no actual text cause false positives.
pub fn page_has_text_layer(doc: &Document, page_id: ObjectId) -> bool {
    let Ok(page_dict) = doc.get_dictionary(page_id) else {
        return false;
    };
    let Ok(resources_obj) = page_dict.get(b"Resources") else {
        return false;
    };
    // Resources may be a direct dictionary or an indirect reference.
    let font_obj = match resources_obj {
        Object::Dictionary(d) => d.get(b"Font").ok(),
        Object::Reference(id) => {
            let Some(obj) = doc.get_object(*id).ok() else { return false };
            let Some(d) = obj.as_dict().ok() else { return false };
            d.get(b"Font").ok()
        }
        _ => return false,
    };
    let Some(font) = font_obj else { return false };
    // Font dict must be non-empty to avoid false positives from empty/inherited dicts.
    match font {
        Object::Dictionary(d) => !d.is_empty(),
        Object::Reference(id) => doc
            .get_object(*id)
            .ok()
            .and_then(|o| o.as_dict().ok())
            .map(|d| !d.is_empty())
            .unwrap_or(false),
        _ => false,
    }
}

/// OCR result for one PDF page, ready for text overlay embedding.
pub struct PageOcrResult {
    pub text_lines: Vec<Option<ocrs_cjk::TextLine>>,
    /// [height_pixels, width_pixels] of the source image.
    pub image_hw: [usize; 2],
    /// [width_pts, height_pts] of the PDF page.
    pub page_wh_pts: [f32; 2],
}

/// Extract raster images from each page of an image/scanned PDF.
///
/// Only `DCTDecode` (JPEG) and `FlateDecode` (raw pixels, DeviceRGB/DeviceGray)
/// are supported. Pages with unsupported image filters are skipped with a warning.
pub fn extract_page_images(path: &str) -> anyhow::Result<Vec<PageImage>> {
    let doc = Document::load(path)
        .with_context(|| format!("failed to open PDF: {path}"))?;

    let mut page_images = Vec::new();

    for (_page_num, page_id) in doc.get_pages() {
        let (width_pts, height_pts) = get_page_media_box(&doc, page_id);
        let has_text_layer = page_has_text_layer(&doc, page_id);

        let images = match doc.get_page_images(page_id) {
            Ok(imgs) => imgs,
            Err(e) => {
                eprintln!("warning: could not read images from page {_page_num}: {e}");
                continue;
            }
        };

        if images.is_empty() {
            if has_text_layer {
                eprintln!(
                    "note: page {_page_num} has a native text layer — no raster images to OCR. \
                     Extract text directly from the PDF instead, or use --skip-text-pages."
                );
            } else {
                eprintln!("warning: page {_page_num} has no embedded images — skipping");
            }
            continue;
        }

        // Use the largest image per page by pixel area.
        // ponytail: area heuristic covers 95%+ of real PDFs; upgrade to content-stream
        //           position parsing if PDFs with equal-size multi-image pages appear.
        let img = images
            .iter()
            .max_by_key(|i| (
                (i.width.max(0) as usize) * (i.height.max(0) as usize),
                i.content.len(),
            ))
            .unwrap(); // safe: images is non-empty (checked above)
        if images.len() > 1 {
            eprintln!(
                "note: page {_page_num} has {} images; using largest ({}×{})",
                images.len(),
                img.width,
                img.height
            );
        }
        let filters = img.filters.as_deref().unwrap_or(&[]);
        let first_filter = filters.first().map(|s| s.as_str()).unwrap_or("");

        let tensor: NdTensor<u8, 3> = match first_filter {
            "DCTDecode" | "DCTD" => {
                // Raw JPEG bytes → decode with the `image` crate.
                let dyn_img = image::load_from_memory(img.content)
                    .context("failed to decode DCTDecode (JPEG) image")?
                    .into_rgb8();
                let (w, h) = dyn_img.dimensions();
                let data = dyn_img.into_raw();
                NdTensor::from_data([h as usize, w as usize, 3], data)
            }
            "FlateDecode" | "" => {
                // Attempt: zlib-decompress and interpret as raw pixels.
                let stream = doc
                    .get_object(img.id)
                    .and_then(Object::as_stream)
                    .context("could not read FlateDecode stream")?;
                let raw = stream
                    .decompressed_content()
                    .context("failed to decompress FlateDecode stream")?;

                let mut w = img.width as usize;
                let mut h = img.height as usize;

                // lopdf's PDFImage may return 0 for width/height on some PDFs;
                // fall back to reading /Width and /Height from the stream dict directly.
                if w == 0 {
                    w = stream.dict.get(b"Width").ok()
                        .and_then(|o| o.as_i64().ok())
                        .map(|v| v.max(0) as usize)
                        .unwrap_or(0);
                }
                if h == 0 {
                    h = stream.dict.get(b"Height").ok()
                        .and_then(|o| o.as_i64().ok())
                        .map(|v| v.max(0) as usize)
                        .unwrap_or(0);
                }

                let channels: usize = match img.color_space.as_deref() {
                    Some("DeviceGray") => 1,
                    _ => 3,
                };

                if w == 0 || h == 0 || raw.len() < w * h * channels {
                    eprintln!(
                        "warning: page {_page_num} FlateDecode size mismatch ({} < {}×{}×{}) — skipping",
                        raw.len(), h, w, channels
                    );
                    continue;
                }

                if channels == 1 {
                    // Expand greyscale to RGB.
                    let mut rgb = vec![0u8; w * h * 3];
                    for (i, &grey) in raw[..w * h].iter().enumerate() {
                        rgb[i * 3] = grey;
                        rgb[i * 3 + 1] = grey;
                        rgb[i * 3 + 2] = grey;
                    }
                    NdTensor::from_data([h, w, 3], rgb)
                } else {
                    NdTensor::from_data([h, w, 3], raw[..w * h * 3].to_vec())
                }
            }
            other => {
                eprintln!(
                    "warning: page {_page_num} uses unsupported image filter '{other}' — skipping"
                );
                continue;
            }
        };

        page_images.push(PageImage { tensor, width_pts, height_pts, has_text_layer });
    }

    Ok(page_images)
}

/// Read the `/MediaBox` of a page from its dictionary.
/// Falls back to A4 (595 × 842 pt) if not found.
fn get_page_media_box(doc: &Document, page_id: ObjectId) -> (f32, f32) {
    let try_get = || -> Option<(f32, f32)> {
        let page_dict = doc.get_dictionary(page_id).ok()?;
        let media_box = page_dict.get(b"MediaBox").ok()?.as_array().ok()?;
        if media_box.len() < 4 {
            return None;
        }
        let x_min = media_box[0].as_float().ok()?;
        let y_min = media_box[1].as_float().ok()?;
        let x_max = media_box[2].as_float().ok()?;
        let y_max = media_box[3].as_float().ok()?;
        Some((x_max - x_min, y_max - y_min))
    };
    try_get().unwrap_or((595.0, 842.0))
}

/// Build a searchable PDF by appending an invisible text overlay to the
/// original PDF for each page.
///
/// The text is rendered in mode 3 (invisible), so it does not obscure the
/// scanned image. A `Type0`/`CIDFontType2` font with a `ToUnicode` CMap is
/// embedded so PDF viewers can correctly extract and search CJK characters.
pub fn build_searchable_pdf(
    source_path: &str,
    pages: &[PageOcrResult],
    output_path: &str,
    min_confidence: Option<f32>,
) -> anyhow::Result<()> {
    let mut doc = Document::load(source_path)
        .with_context(|| format!("failed to re-open PDF for writing: {source_path}"))?;

    // Collect every unique character across all pages to build the font tables.
    let mut all_chars: Vec<char> = pages
        .iter()
        .flat_map(|p| p.text_lines.iter().flatten())
        .flat_map(|l| l.chars().iter().map(|c| c.char))
        .collect();
    all_chars.sort_unstable();
    all_chars.dedup();

    // Assign sequential glyph IDs (1-based; 0 is reserved as "no glyph").
    let char_to_glyph: HashMap<char, u16> = all_chars
        .iter()
        .enumerate()
        .map(|(i, &c)| (c, (i + 1) as u16))
        .collect();

    // Build and register the CJK-capable font.
    let font_id = build_type0_font(&mut doc, &all_chars, &char_to_glyph)?;

    // Add text overlay to each page.
    let page_ids: Vec<ObjectId> = doc.get_pages().into_values().collect();
    for (page_result, &page_id) in pages.iter().zip(page_ids.iter()) {
        let [img_h, img_w] = page_result.image_hw;
        let [page_w_pts, page_h_pts] = page_result.page_wh_pts;

        // Add font to this page's resource dictionary.
        add_font_to_page(doc.get_object_mut(page_id).and_then(Object::as_dict_mut)?, b"F0", font_id)?;

        // Build the invisible text content stream.
        let content_bytes = build_text_stream(
            page_result,
            img_h,
            img_w,
            page_w_pts,
            page_h_pts,
            &char_to_glyph,
            min_confidence,
        )?;

        doc.add_page_contents(page_id, content_bytes)
            .context("failed to add text layer to page")?;
    }

    doc.compress();
    doc.save(output_path)
        .with_context(|| format!("failed to write PDF to {output_path}"))?;

    Ok(())
}

/// Add a `/Font /F0 <ref>` entry to a page dictionary's Resources.
fn add_font_to_page(
    page: &mut lopdf::Dictionary,
    font_name: &[u8],
    font_id: ObjectId,
) -> anyhow::Result<()> {
    // Ensure /Resources exists inline (if it's a Reference we handle below).
    if !page.has(b"Resources") {
        page.set("Resources", lopdf::Dictionary::new());
    }

    let resources = page
        .get_mut(b"Resources")
        .and_then(Object::as_dict_mut)
        .context("could not get Resources dict")?;

    if !resources.has(b"Font") {
        resources.set("Font", lopdf::Dictionary::new());
    }

    let font_dict = resources
        .get_mut(b"Font")
        .and_then(Object::as_dict_mut)
        .context("could not get Font dict")?;

    font_dict.set(font_name, Object::Reference(font_id));
    Ok(())
}

/// Build the invisible text content stream for one page.
fn build_text_stream(
    page: &PageOcrResult,
    img_h: usize,
    img_w: usize,
    page_w_pts: f32,
    page_h_pts: f32,
    char_to_glyph: &HashMap<char, u16>,
    min_confidence: Option<f32>,
) -> anyhow::Result<Vec<u8>> {
    let scale_x = if img_w > 0 { page_w_pts / img_w as f32 } else { 1.0 };
    let scale_y = if img_h > 0 { page_h_pts / img_h as f32 } else { 1.0 };

    let mut ops: Vec<Operation> = vec![
        Operation::new("BT", vec![]),
        // Render mode 3 = invisible (no fill, no stroke).
        Operation::new("Tr", vec![Object::Integer(3)]),
        // Use font F0 at 10pt. Size matters for selection rectangles but not
        // for visibility, since text is invisible.
        Operation::new("Tf", vec![Object::Name(b"F0".to_vec()), Object::Integer(10)]),
    ];

    for line in page.text_lines.iter().flatten() {
        for word in line.words() {
            let text = word.to_string();
            if text.trim().is_empty() {
                continue;
            }
            // Skip words below the confidence threshold.
            if min_confidence.map(|t| word.confidence() < t).unwrap_or(false) {
                continue;
            }

            let bbox = word.bounding_rect();

            // Map pixel coordinates to PDF points.
            // PDF Y-axis is bottom-up; image Y-axis is top-down.
            let pdf_x = bbox.left() as f32 * scale_x;
            let pdf_y = page_h_pts - (bbox.bottom() as f32 * scale_y);

            // Encode text as 2-byte glyph IDs (UTF-16BE identity mapping).
            let encoded: Vec<u8> = text
                .chars()
                .flat_map(|c| {
                    let gid = char_to_glyph.get(&c).copied().unwrap_or(0);
                    gid.to_be_bytes()
                })
                .collect();

            ops.push(Operation::new(
                "Tm",
                vec![
                    Object::Real(1.0),
                    Object::Real(0.0),
                    Object::Real(0.0),
                    Object::Real(1.0),
                    Object::Real(pdf_x),
                    Object::Real(pdf_y),
                ],
            ));
            ops.push(Operation::new(
                "Tj",
                vec![Object::String(encoded, StringFormat::Hexadecimal)],
            ));
        }
    }

    ops.push(Operation::new("ET", vec![]));

    Content { operations: ops }
        .encode()
        .context("failed to encode PDF content stream")
}

/// Create a `Type0`/`CIDFontType2` font with a `ToUnicode` CMap and register
/// it in the document. Returns the `ObjectId` of the `Type0` font dictionary.
fn build_type0_font(
    doc: &mut Document,
    all_chars: &[char],
    char_to_glyph: &HashMap<char, u16>,
) -> anyhow::Result<ObjectId> {
    // 1. ToUnicode CMap stream (no Filter — doc.compress() will add FlateDecode).
    let cmap_text = build_tounicode_cmap(all_chars, char_to_glyph);
    let tounicode_id = doc.add_object(Stream::new(
        dictionary! {},
        cmap_text.into_bytes(),
    ));

    // 2. CIDFont (descendant).
    let cid_font_dict = dictionary! {
        "Type" => "Font",
        "Subtype" => "CIDFontType2",
        "BaseFont" => "OCRText",
        "CIDSystemInfo" => dictionary! {
            "Registry" => Object::string_literal("Adobe"),
            "Ordering" => Object::string_literal("Identity"),
            "Supplement" => Object::Integer(0),
        },
        // DW = default glyph width (1000 units = full em).
        "DW" => Object::Integer(1000),
    };
    let cid_font_id = doc.add_object(cid_font_dict);

    // 3. Type0 composite font.
    let type0_dict = dictionary! {
        "Type" => "Font",
        "Subtype" => "Type0",
        "BaseFont" => "OCRText",
        "Encoding" => "Identity-H",
        "DescendantFonts" => vec![Object::Reference(cid_font_id)],
        "ToUnicode" => Object::Reference(tounicode_id),
    };
    let type0_id = doc.add_object(type0_dict);

    Ok(type0_id)
}

/// Build the PostScript-like `ToUnicode` CMap string that maps each glyph ID
/// to its Unicode code point, enabling PDF viewers to extract and search text.
fn build_tounicode_cmap(all_chars: &[char], char_to_glyph: &HashMap<char, u16>) -> String {
    let mut cmap = String::from(
        "/CIDInit /ProcSet findresource begin\n\
         12 dict begin\n\
         begincmap\n\
         /CIDSystemInfo <<\n\
           /Registry (Adobe)\n\
           /Ordering (UCS)\n\
           /Supplement 0\n\
         >> def\n\
         /CMapName /Adobe-Identity-UCS def\n\
         /CMapType 2 def\n\
         1 begincodespacerange\n\
         <0000> <FFFF>\n\
         endcodespacerange\n",
    );

    // PDF limits bfchar sections to 100 entries each.
    const CHUNK_SIZE: usize = 100;
    for chunk in all_chars.chunks(CHUNK_SIZE) {
        cmap.push_str(&format!("{} beginbfchar\n", chunk.len()));
        for &c in chunk {
            let gid = char_to_glyph.get(&c).copied().unwrap_or(0);
            let cp = c as u32;
            if cp <= 0xFFFF {
                cmap.push_str(&format!("<{gid:04X}> <{cp:04X}>\n"));
            } else {
                // Supplementary plane: encode as UTF-16 surrogate pair.
                let cp = cp - 0x10000;
                let high = 0xD800 + (cp >> 10);
                let low = 0xDC00 + (cp & 0x3FF);
                cmap.push_str(&format!("<{gid:04X}> <{high:04X}{low:04X}>\n"));
            }
        }
        cmap.push_str("endbfchar\n");
    }

    cmap.push_str(
        "endcmap\n\
         CMapName currentdict /CMap defineresource pop\n\
         end\n\
         end\n",
    );
    cmap
}
