use rten_tensor::prelude::*;
use rten_tensor::{NdTensor, NdTensorView};

/// Max skew angle to detect (degrees, ±).
const MAX_ANGLE: f32 = 15.0;
/// Angular step for the search.
const ANGLE_STEP: f32 = 0.5;
/// Skip correction for angles below this threshold.
const MIN_CORRECTION: f32 = 0.1;
/// Max pixel dimension for the downsampled analysis image.
const SMALL_DIM: usize = 200;

/// Detect and correct image skew using projection profile analysis.
///
/// Returns `(corrected_image, detected_skew_degrees)`. If `|angle| < 0.1°`
/// no rotation is applied and the original pixels are returned unchanged.
pub fn deskew(img: NdTensorView<u8, 3>) -> (NdTensor<u8, 3>, f32) {
    let [h, w, channels] = img.shape();

    let (gray_small, sh, sw) = grayscale_downsample(img, SMALL_DIM);
    let angle = detect_skew_angle(&gray_small, sh, sw);

    if angle.abs() < MIN_CORRECTION {
        let data: Vec<u8> = img.iter().copied().collect();
        return (NdTensor::from_data([h, w, channels], data), 0.0);
    }

    (rotate_bilinear(img, -angle), angle)
}

/// Downsample and convert to grayscale (nearest-neighbour sampling).
fn grayscale_downsample(
    img: NdTensorView<u8, 3>,
    max_dim: usize,
) -> (Vec<u8>, usize, usize) {
    let [h, w, channels] = img.shape();

    let scale = (max_dim as f32 / h.max(w) as f32).min(1.0);
    let sh = ((h as f32 * scale).round() as usize).max(1);
    let sw = ((w as f32 * scale).round() as usize).max(1);

    let input: Vec<u8> = img.iter().copied().collect();

    let mut gray = Vec::with_capacity(sh * sw);
    for sy in 0..sh {
        for sx in 0..sw {
            let fy = ((sy as f32 / sh as f32) * h as f32) as usize;
            let fx = ((sx as f32 / sw as f32) * w as f32) as usize;
            let base = (fy.min(h - 1) * w + fx.min(w - 1)) * channels;
            let g = if channels == 1 {
                input[base]
            } else {
                let r = input[base] as u32;
                let g = input[base + 1] as u32;
                let b = input[base + 2] as u32;
                ((r * 299 + g * 587 + b * 114) / 1000) as u8
            };
            gray.push(g);
        }
    }
    (gray, sh, sw)
}

/// Search ±MAX_ANGLE in ANGLE_STEP increments; return the angle that
/// maximises horizontal projection variance (= text aligned with axis).
fn detect_skew_angle(gray: &[u8], h: usize, w: usize) -> f32 {
    let steps = ((2.0 * MAX_ANGLE / ANGLE_STEP).round() as usize) + 1;
    let mut best_var = f32::NEG_INFINITY;
    let mut best_angle = 0.0f32;

    for i in 0..steps {
        let angle_deg = -MAX_ANGLE + i as f32 * ANGLE_STEP;
        let var = projection_variance(gray, h, w, angle_deg.to_radians());
        if var > best_var {
            best_var = var;
            best_angle = angle_deg;
        }
    }
    best_angle
}

/// Compute variance of the horizontal projection profile.
///
/// Each dark pixel (< 128) is projected onto a rotated row index; we count
/// pixels per row then compute variance. Higher variance ↔ text rows aligned.
fn projection_variance(gray: &[u8], h: usize, w: usize, angle_rad: f32) -> f32 {
    let cos = angle_rad.cos();
    let sin = angle_rad.sin();
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let n_rows = h + w;
    let offset = w as f32 / 2.0;

    let mut counts = vec![0u32; n_rows];
    for y in 0..h {
        for x in 0..w {
            if gray[y * w + x] < 128 {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let proj = ((-sin * dx + cos * dy) + cy + offset).round() as i32;
                if proj >= 0 && (proj as usize) < n_rows {
                    counts[proj as usize] += 1;
                }
            }
        }
    }

    let mean = counts.iter().sum::<u32>() as f32 / n_rows as f32;
    counts
        .iter()
        .map(|&v| {
            let d = v as f32 - mean;
            d * d
        })
        .sum::<f32>()
        / n_rows as f32
}

/// Rotate an HWC u8 image by `angle_deg` degrees using bilinear interpolation.
///
/// Pixels outside source bounds are filled with white (255).
fn rotate_bilinear(img: NdTensorView<u8, 3>, angle_deg: f32) -> NdTensor<u8, 3> {
    let [h, w, channels] = img.shape();
    let input: Vec<u8> = img.iter().copied().collect();

    let angle_rad = angle_deg.to_radians();
    let cos = angle_rad.cos();
    let sin = angle_rad.sin();
    let cx = (w as f32 - 1.0) / 2.0;
    let cy = (h as f32 - 1.0) / 2.0;

    let mut out = vec![255u8; h * w * channels];

    for oy in 0..h {
        for ox in 0..w {
            // Inverse map: find the source pixel for output (ox, oy).
            let dx = ox as f32 - cx;
            let dy = oy as f32 - cy;
            let src_x = cos * dx + sin * dy + cx;
            let src_y = -sin * dx + cos * dy + cy;

            let x0 = src_x.floor() as i32;
            let y0 = src_y.floor() as i32;
            let x1 = x0 + 1;
            let y1 = y0 + 1;
            let fx = src_x - x0 as f32;
            let fy = src_y - y0 as f32;

            let get = |xi: i32, yi: i32, c: usize| -> f32 {
                if xi >= 0 && (xi as usize) < w && yi >= 0 && (yi as usize) < h {
                    input[(yi as usize * w + xi as usize) * channels + c] as f32
                } else {
                    255.0
                }
            };

            let base = (oy * w + ox) * channels;
            for c in 0..channels {
                let v = (1.0 - fx) * (1.0 - fy) * get(x0, y0, c)
                    + fx * (1.0 - fy) * get(x1, y0, c)
                    + (1.0 - fx) * fy * get(x0, y1, c)
                    + fx * fy * get(x1, y1, c);
                out[base + c] = v.round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    NdTensor::from_data([h, w, channels], out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rten_tensor::NdTensor;

    fn make_striped_image(h: usize, w: usize) -> NdTensor<u8, 3> {
        // Horizontal black stripes on white — perfect alignment → angle ≈ 0°
        let data: Vec<u8> = (0..h)
            .flat_map(|y| (0..w).map(move |_| if y % 8 < 4 { 0u8 } else { 255u8 }))
            .flat_map(|v| [v, v, v])
            .collect();
        NdTensor::from_data([h, w, 3], data)
    }

    #[test]
    fn test_no_skew_detected_for_horizontal_stripes() {
        let img = make_striped_image(100, 200);
        let (_, angle) = deskew(img.view());
        // Striped image is perfectly horizontal — detected skew should be near 0
        assert!(
            angle.abs() < 1.0,
            "expected near-zero skew for horizontal stripes, got {angle:.1}°"
        );
    }

    #[test]
    fn test_no_panic_on_tiny_image() {
        let data = vec![128u8; 4 * 4 * 3];
        let img = NdTensor::from_data([4, 4, 3], data);
        let _ = deskew(img.view()); // must not panic
    }

    #[test]
    fn test_rotation_roundtrip_identity() {
        // Rotating by 0° must return the original pixels.
        let img = make_striped_image(40, 60);
        let rotated = rotate_bilinear(img.view(), 0.0);
        let orig: Vec<u8> = img.iter().copied().collect();
        let got: Vec<u8> = rotated.iter().copied().collect();
        assert_eq!(orig, got);
    }
}
