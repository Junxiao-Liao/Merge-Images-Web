//! Chrome-strip pre-pass for Smart merges.
//!
//! Many screenshot sources include repeated browser/app chrome (headers/footers)
//! on every capture. This confuses overlap detection because these repeated bars
//! can appear as strong matches. This module detects near-identical top/bottom
//! regions between adjacent images and trims them so the overlap detector sees
//! mostly content.

use image::{DynamicImage, GrayImage, imageops::FilterType};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChromeTrim {
    /// Pixels to trim from the top of this image.
    pub top: u32,
    /// Pixels to trim from the bottom of this image.
    pub bottom: u32,
}

const PROXY_WIDTH: u32 = 320;
const MARGIN_PERCENT: f32 = 0.025;

const PIXEL_DELTA: u8 = 12;
const ROW_MATCH_FRACTION: f32 = 0.97;
const ROW_MEAN_ABS_DELTA_MAX: f32 = 6.0;

const MAX_TRIM_PX: u32 = 240;
const MAX_TRIM_FRACTION: f32 = 0.20;
const MIN_CONTENT_PX: u32 = 50;

/// Computes chrome trims for each image in a sequence.
///
/// The returned vector has the same length as `images`. The first image will
/// have `top = 0` and the last image will have `bottom = 0`.
pub fn compute_chrome_trims(images: &[DynamicImage]) -> Vec<ChromeTrim> {
    let n = images.len();
    if n == 0 {
        return vec![];
    }

    let proxies: Vec<GrayImage> = images.iter().map(build_proxy).collect();
    let mut trims = vec![ChromeTrim::default(); n];

    for i in 0..n.saturating_sub(1) {
        let (prev, curr) = (&proxies[i], &proxies[i + 1]);
        let top_rows = count_common_rows_top(prev, curr);
        let bottom_rows = count_common_rows_bottom(prev, curr);

        // Apply top trim to the current image.
        let curr_px = proxy_rows_to_pixels(top_rows, images[i + 1].height(), curr.height());
        trims[i + 1].top = clamp_trim(curr_px, images[i + 1].height());

        // Apply bottom trim to the previous image.
        let prev_px = proxy_rows_to_pixels(bottom_rows, images[i].height(), prev.height());
        trims[i].bottom = clamp_trim(prev_px, images[i].height());
    }

    // Ensure we don't trim away the entire image.
    for (i, img) in images.iter().enumerate() {
        trims[i] = enforce_min_content(trims[i], img.height());
    }

    // First top and last bottom must survive.
    if let Some(first) = trims.first_mut() {
        first.top = 0;
    }
    if let Some(last) = trims.last_mut() {
        last.bottom = 0;
    }

    trims
}

fn build_proxy(img: &DynamicImage) -> GrayImage {
    let w = img.width().max(1);
    let h = img.height().max(1);

    let target_w = PROXY_WIDTH.min(w).max(1);
    let target_h =
        round_half_up_u64((h as u64) * (target_w as u64), w as u64).clamp(1, h as u64) as u32;

    // Convert to grayscale first, then scale deterministically.
    let gray = img.to_luma8();
    if gray.width() == target_w && gray.height() == target_h {
        return gray;
    }
    image::imageops::resize(&gray, target_w, target_h, FilterType::Triangle)
}

fn proxy_rows_to_pixels(rows: u32, orig_h: u32, proxy_h: u32) -> u32 {
    if rows == 0 || orig_h == 0 || proxy_h == 0 {
        return 0;
    }
    round_half_up_u64((rows as u64) * (orig_h as u64), proxy_h as u64) as u32
}

fn clamp_trim(trim: u32, height: u32) -> u32 {
    if height == 0 {
        return 0;
    }
    let max_by_fraction = ((height as f32) * MAX_TRIM_FRACTION).round() as u32;
    trim.min(MAX_TRIM_PX).min(max_by_fraction).min(height)
}

fn enforce_min_content(trim: ChromeTrim, height: u32) -> ChromeTrim {
    if height == 0 {
        return ChromeTrim::default();
    }
    let mut t = trim;
    let total = t.top.saturating_add(t.bottom);
    let min_content = MIN_CONTENT_PX.min(height);
    if total > height.saturating_sub(min_content) {
        // If we would trim too much, fall back to trimming nothing.
        t.top = 0;
        t.bottom = 0;
    }
    t
}

fn count_common_rows_top(a: &GrayImage, b: &GrayImage) -> u32 {
    let max_rows = a.height().min(b.height());
    if max_rows == 0 {
        return 0;
    }
    let (ax0, aw) = common_span(a.width(), b.width());
    if aw == 0 {
        return 0;
    }

    let mut rows = 0;
    for y in 0..max_rows {
        if !rows_similar(a, b, ax0, aw, y, y) {
            break;
        }
        rows += 1;
    }
    rows
}

fn count_common_rows_bottom(a: &GrayImage, b: &GrayImage) -> u32 {
    let max_rows = a.height().min(b.height());
    if max_rows == 0 {
        return 0;
    }
    let (ax0, aw) = common_span(a.width(), b.width());
    if aw == 0 {
        return 0;
    }

    let mut rows = 0;
    for i in 0..max_rows {
        let ay = a.height() - 1 - i;
        let by = b.height() - 1 - i;
        if !rows_similar(a, b, ax0, aw, ay, by) {
            break;
        }
        rows += 1;
    }
    rows
}

fn common_span(wa: u32, wb: u32) -> (u32, u32) {
    let common_w = wa.min(wb);
    if common_w == 0 {
        return (0, 0);
    }
    let margin = ((common_w as f32) * MARGIN_PERCENT) as u32;
    let x0 = margin.min(common_w);
    let x1 = common_w.saturating_sub(margin);
    let w = x1.saturating_sub(x0);
    (x0, w)
}

fn rows_similar(a: &GrayImage, b: &GrayImage, x0: u32, w: u32, ay: u32, by: u32) -> bool {
    if w == 0 {
        return false;
    }

    let mut match_count: u32 = 0;
    let mut sum_abs: u32 = 0;
    for x in x0..(x0 + w) {
        let av = a.get_pixel(x, ay)[0];
        let bv = b.get_pixel(x, by)[0];
        let diff = av.abs_diff(bv);
        if diff <= PIXEL_DELTA {
            match_count += 1;
        }
        sum_abs += diff as u32;
    }

    let denom = w as f32;
    let frac = (match_count as f32) / denom;
    if frac < ROW_MATCH_FRACTION {
        return false;
    }

    let mean_abs = (sum_abs as f32) / denom;
    mean_abs <= ROW_MEAN_ABS_DELTA_MAX
}

fn round_half_up_u64(num: u64, den: u64) -> u64 {
    if den == 0 {
        return 0;
    }
    (num + (den / 2)) / den
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, Rgba, RgbaImage};

    fn build_bar_image(width: u32, height: u32, top: u32, bottom: u32, seed: u32) -> DynamicImage {
        let mut img = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let px = if y < top || y >= height.saturating_sub(bottom) {
                    Rgba([20, 20, 20, 255])
                } else {
                    // Deterministic busy content.
                    let v = x.wrapping_mul(37) ^ y.wrapping_mul(131) ^ seed.wrapping_mul(7919);
                    let g = (v % 251) as u8;
                    Rgba([g, g, g, 255])
                };
                img.put_pixel(x, y, px);
            }
        }
        DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn test_compute_chrome_trims_detects_top_and_bottom() {
        let a = build_bar_image(200, 340, 20, 20, 1);
        let b = build_bar_image(200, 340, 20, 20, 2);
        let trims = compute_chrome_trims(&[a, b]);
        assert_eq!(trims.len(), 2);

        // First top and last bottom must remain.
        assert_eq!(trims[0].top, 0);
        assert_eq!(trims[1].bottom, 0);

        // Allow small tolerance due to proxy scaling.
        assert!(
            trims[0].bottom.abs_diff(20) <= 2,
            "bottom={}",
            trims[0].bottom
        );
        assert!(trims[1].top.abs_diff(20) <= 2, "top={}", trims[1].top);
    }
}
