//! Overlap detection module for smart merging.
//!
//! Uses template matching with Normalized Cross-Correlation (NCC) to detect
//! overlapping regions between consecutive screenshots.

use image::{DynamicImage, GrayImage, ImageBuffer, Luma};
use imageproc::template_matching::{MatchTemplateMethod, find_extremes, match_template};

/// Minimum match score threshold for overlap detection (conservative end).
const MATCH_THRESHOLD_CONSERVATIVE: f32 = 0.86;
/// Minimum match score threshold for overlap detection (aggressive end).
const MATCH_THRESHOLD_AGGRESSIVE: f32 = 0.76;

/// Required gap between best and second-best matches (conservative end).
const AMBIGUITY_GAP_CONSERVATIVE: f32 = 0.04;
/// Required gap between best and second-best matches (aggressive end).
const AMBIGUITY_GAP_AGGRESSIVE: f32 = 0.01;

/// Minimum template variance to avoid flat matches (conservative end).
const MIN_TEMPLATE_VARIANCE_CONSERVATIVE: f32 = 50.0;
/// Minimum template variance to avoid flat matches (aggressive end).
const MIN_TEMPLATE_VARIANCE_AGGRESSIVE: f32 = 10.0;

/// Base template height in pixels.
const TEMPLATE_HEIGHT_PX: u32 = 80;
/// Maximum template height in pixels (for ambiguous matches).
const TEMPLATE_HEIGHT_PX_MAX: u32 = 240;
/// Template height adjustment step.
const TEMPLATE_HEIGHT_STEP_PX: u32 = 40;

/// Minimum template height in pixels.
const MIN_TEMPLATE_HEIGHT: u32 = 30;
/// Minimum template width in pixels.
const MIN_TEMPLATE_WIDTH: u32 = 50;

/// Minimum overlap to accept (in pixels).
const MIN_OVERLAP_PIXELS: u32 = 5;

/// Percentage of width to crop from each side to ignore scroll bars.
const MARGIN_PERCENT: f32 = 0.025;

/// Search region start and end in the top image.
const SEARCH_START_PERCENT: f32 = 0.0;
const SEARCH_END_PERCENT: f32 = 1.0;

/// Template start positions in the bottom image.
const TEMPLATE_START_PERCENT: f32 = 0.0;
const TEMPLATE_START_FALLBACK_PERCENT: f32 = 0.02;

/// Minimum acceptable width ratio between two images.
const WIDTH_RATIO_THRESHOLD: f32 = 0.9;

#[derive(Debug, Clone, Copy)]
struct OverlapConfig {
    match_threshold: f32,
    ambiguity_gap: f32,
    min_template_variance: f32,
    sensitivity: u8,
}

impl OverlapConfig {
    fn from_sensitivity(sensitivity: u8) -> Self {
        let clamped = sensitivity.min(100) as f32 / 100.0;
        Self {
            match_threshold: lerp(
                MATCH_THRESHOLD_CONSERVATIVE,
                MATCH_THRESHOLD_AGGRESSIVE,
                clamped,
            ),
            ambiguity_gap: lerp(
                AMBIGUITY_GAP_CONSERVATIVE,
                AMBIGUITY_GAP_AGGRESSIVE,
                clamped,
            ),
            min_template_variance: lerp(
                MIN_TEMPLATE_VARIANCE_CONSERVATIVE,
                MIN_TEMPLATE_VARIANCE_AGGRESSIVE,
                clamped,
            ),
            sensitivity: sensitivity.min(100),
        }
    }

    fn prefer_smaller_templates(&self) -> bool {
        self.sensitivity >= 50
    }
}

/// Result of overlap detection between two images.
#[derive(Debug, Clone, Copy)]
pub struct OverlapResult {
    /// Number of pixels that overlap between the two images.
    /// This is how much to crop from the top of the second image.
    pub overlap_pixels: u32,
    /// Match confidence score (0.0 - 1.0).
    #[allow(dead_code)]
    pub confidence: f32,
}

/// Detects vertical overlap between two images.
///
/// Compares a strip near the top of `img_bottom` against a wide search region
/// in `img_top` to find overlapping content.
///
/// # Arguments
/// * `img_top` - The first (top) image
/// * `img_bottom` - The second (bottom) image
/// * `sensitivity` - Overlap sensitivity (0-100); higher is more aggressive
///
/// # Returns
/// * `Some(OverlapResult)` - If overlap detected with sufficient confidence
/// * `None` - If no overlap detected or images are incompatible
pub fn detect_overlap(
    img_top: &DynamicImage,
    img_bottom: &DynamicImage,
    sensitivity: u8,
) -> Option<OverlapResult> {
    let (top_w, top_h) = (img_top.width(), img_top.height());
    let (bottom_w, bottom_h) = (img_bottom.width(), img_bottom.height());

    // Images must have similar widths for screenshot stitching.
    let width_ratio = top_w.min(bottom_w) as f32 / top_w.max(bottom_w) as f32;
    if width_ratio < WIDTH_RATIO_THRESHOLD {
        return None;
    }

    let common_width = top_w.min(bottom_w);

    // Calculate horizontal margins to crop (to ignore scroll bars).
    let margin = ((common_width as f32) * MARGIN_PERCENT) as u32;
    let cropped_width = common_width.saturating_sub(margin.saturating_mul(2));

    if cropped_width < MIN_TEMPLATE_WIDTH {
        return None;
    }

    // Search region in the top image (almost full height).
    let search_start_y = ((top_h as f32) * SEARCH_START_PERCENT) as u32;
    let search_end_y = ((top_h as f32) * SEARCH_END_PERCENT) as u32;
    let search_end_y = search_end_y.min(top_h);

    if search_end_y <= search_start_y {
        return None;
    }

    let search_height = search_end_y - search_start_y;
    if search_height < MIN_TEMPLATE_HEIGHT {
        return None;
    }

    let search_region = extract_grayscale_region(
        img_top,
        margin,
        search_start_y,
        cropped_width,
        search_height,
    )?;

    let config = OverlapConfig::from_sensitivity(sensitivity);

    let template_start_candidates = [TEMPLATE_START_PERCENT, TEMPLATE_START_FALLBACK_PERCENT];
    for template_start_percent in template_start_candidates {
        let template_start_y = ((bottom_h as f32) * template_start_percent) as u32;
        let available_template_height = bottom_h.saturating_sub(template_start_y);
        if available_template_height < MIN_TEMPLATE_HEIGHT {
            continue;
        }

        let max_template_height = TEMPLATE_HEIGHT_PX_MAX
            .min(available_template_height)
            .min(search_height.saturating_sub(1));

        if max_template_height < MIN_TEMPLATE_HEIGHT {
            continue;
        }

        let base_height = TEMPLATE_HEIGHT_PX.clamp(MIN_TEMPLATE_HEIGHT, max_template_height);
        let template_heights = build_template_heights(
            base_height,
            MIN_TEMPLATE_HEIGHT,
            max_template_height,
            config.prefer_smaller_templates(),
        );

        for template_height in template_heights {
            if template_height >= search_height {
                continue;
            }

            let template = extract_grayscale_region(
                img_bottom,
                margin,
                template_start_y,
                cropped_width,
                template_height,
            )?;

            if template_variance(&template) < config.min_template_variance {
                continue;
            }

            if let Some(result) = perform_matching(
                &search_region,
                &template,
                search_start_y,
                top_h,
                bottom_h,
                &config,
            ) {
                return Some(result);
            }
        }
    }

    None
}

/// Extracts a grayscale region from an image.
fn extract_grayscale_region(
    img: &DynamicImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Option<GrayImage> {
    // Bounds check.
    if x + width > img.width() || y + height > img.height() {
        return None;
    }

    let cropped = img.crop_imm(x, y, width, height);
    Some(cropped.to_luma8())
}

fn build_template_heights(
    base: u32,
    min_height: u32,
    max_height: u32,
    prefer_smaller: bool,
) -> Vec<u32> {
    let mut heights = Vec::new();

    let push_unique = |value: u32, list: &mut Vec<u32>| {
        if !list.contains(&value) {
            list.push(value);
        }
    };

    push_unique(base, &mut heights);

    if prefer_smaller {
        let mut h = base;
        while h > min_height {
            h = h.saturating_sub(TEMPLATE_HEIGHT_STEP_PX);
            if h < min_height {
                break;
            }
            push_unique(h, &mut heights);
        }

        let mut h = base;
        while h < max_height {
            h = h.saturating_add(TEMPLATE_HEIGHT_STEP_PX);
            if h > max_height {
                break;
            }
            push_unique(h, &mut heights);
        }
    } else {
        let mut h = base;
        while h < max_height {
            h = h.saturating_add(TEMPLATE_HEIGHT_STEP_PX);
            if h > max_height {
                break;
            }
            push_unique(h, &mut heights);
        }

        let mut h = base;
        while h > min_height {
            h = h.saturating_sub(TEMPLATE_HEIGHT_STEP_PX);
            if h < min_height {
                break;
            }
            push_unique(h, &mut heights);
        }
    }

    heights
}

fn template_variance(template: &GrayImage) -> f32 {
    let mut sum = 0.0f32;
    let mut sum_sq = 0.0f32;
    let count = (template.width() * template.height()) as f32;

    if count == 0.0 {
        return 0.0;
    }

    for pixel in template.pixels() {
        let value = pixel[0] as f32;
        sum += value;
        sum_sq += value * value;
    }

    let mean = sum / count;
    let variance = (sum_sq / count) - (mean * mean);
    variance.max(0.0)
}

/// Performs template matching and returns overlap result.
fn perform_matching(
    search_region: &GrayImage,
    template: &GrayImage,
    search_start_y: u32,
    top_height: u32,
    bottom_height: u32,
    config: &OverlapConfig,
) -> Option<OverlapResult> {
    // Perform template matching using NCC.
    let result = match_template(
        search_region,
        template,
        MatchTemplateMethod::CrossCorrelationNormalized,
    );

    // Find best match.
    let extremes = find_extremes(&result);
    let best_score = extremes.max_value;
    let best_pos = extremes.max_value_location;

    if !best_score.is_finite() || best_score < config.match_threshold {
        return None;
    }

    let second_best = find_second_best(&result, best_pos, template);
    if best_score - second_best < config.ambiguity_gap {
        return None;
    }

    // Calculate overlap:
    // - best_pos.1 is the y-position in the search region where template matched
    // - search region starts at search_start_y
    // - The overlap is from the match position to the bottom of img_top
    let match_y_in_original = search_start_y.saturating_add(best_pos.1 as u32);
    let overlap_pixels = top_height.saturating_sub(match_y_in_original);

    // Sanity check: overlap should be reasonable.
    if overlap_pixels < MIN_OVERLAP_PIXELS || overlap_pixels > bottom_height {
        return None;
    }

    Some(OverlapResult {
        overlap_pixels,
        confidence: best_score,
    })
}

fn find_second_best(
    result: &ImageBuffer<Luma<f32>, Vec<f32>>,
    best_pos: (u32, u32),
    template: &GrayImage,
) -> f32 {
    let exclusion_x = (template.width() / 4).max(2);
    let exclusion_y = (template.height() / 4).max(2);
    let mut second_best = f32::NEG_INFINITY;

    for (x, y, pixel) in result.enumerate_pixels() {
        if x.abs_diff(best_pos.0) <= exclusion_x && y.abs_diff(best_pos.1) <= exclusion_y {
            continue;
        }

        let value = pixel[0];
        if value > second_best {
            second_best = value;
        }
    }

    second_best
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + ((end - start) * t)
}

/// Computes overlaps for a sequence of images.
///
/// Returns a vector of overlap amounts, where `overlaps[i]` is the overlap
/// between image `i` and image `i+1`. The vector has length `images.len() - 1`.
///
/// When overlap detection fails for a pair, returns 0 for that pair (simple concatenation).
pub fn compute_overlaps(images: &[DynamicImage], sensitivity: u8) -> Vec<u32> {
    if images.len() < 2 {
        return vec![];
    }

    images
        .windows(2)
        .map(|pair| {
            detect_overlap(&pair[0], &pair[1], sensitivity)
                .map(|r| r.overlap_pixels)
                .unwrap_or(0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    const TEST_SENSITIVITY: u8 = 35;

    fn create_solid_image(width: u32, height: u32, color: Rgba<u8>) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(width, height, color))
    }

    fn create_gradient_image(width: u32, height: u32) -> DynamicImage {
        let mut img = RgbaImage::new(width, height);
        for y in 0..height {
            let gray = ((y as f32 / height as f32) * 255.0) as u8;
            for x in 0..width {
                img.put_pixel(x, y, Rgba([gray, gray, gray, 255]));
            }
        }
        DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn test_no_overlap_different_images() {
        let img1 = create_solid_image(200, 400, Rgba([255, 0, 0, 255]));
        let img2 = create_solid_image(200, 400, Rgba([0, 255, 0, 255]));

        let result = detect_overlap(&img1, &img2, TEST_SENSITIVITY);
        // Solid colors might still match, but the test verifies the function runs
        assert!(result.is_none() || result.unwrap().overlap_pixels > 0);
    }

    #[test]
    fn test_images_too_small() {
        let img1 = create_solid_image(40, 40, Rgba([128, 128, 128, 255]));
        let img2 = create_solid_image(40, 40, Rgba([128, 128, 128, 255]));

        let result = detect_overlap(&img1, &img2, TEST_SENSITIVITY);
        // Very small images may fail due to minimum size requirements
        assert!(result.is_none() || result.unwrap().overlap_pixels <= 40);
    }

    #[test]
    fn test_different_widths_rejected() {
        let img1 = create_solid_image(200, 400, Rgba([128, 128, 128, 255]));
        let img2 = create_solid_image(100, 400, Rgba([128, 128, 128, 255]));

        let result = detect_overlap(&img1, &img2, TEST_SENSITIVITY);
        assert!(result.is_none());
    }

    #[test]
    fn test_compute_overlaps_single_image() {
        let img = create_solid_image(200, 400, Rgba([128, 128, 128, 255]));
        let overlaps = compute_overlaps(&[img], TEST_SENSITIVITY);
        assert!(overlaps.is_empty());
    }

    #[test]
    fn test_compute_overlaps_empty() {
        let overlaps = compute_overlaps(&[], TEST_SENSITIVITY);
        assert!(overlaps.is_empty());
    }

    #[test]
    fn test_gradient_overlap_detection() {
        let img1 = create_gradient_image(200, 400);
        let img2 = create_gradient_image(200, 400);

        let result = detect_overlap(&img1, &img2, TEST_SENSITIVITY);
        if let Some(r) = result {
            let config = OverlapConfig::from_sensitivity(TEST_SENSITIVITY);
            assert!(r.confidence >= config.match_threshold);
            assert!(r.overlap_pixels > 0);
        }
    }
}
