use image::{DynamicImage, ImageReader, Rgba, RgbaImage};
use std::io::Cursor;

use crate::dimension::{compute_output_size, compute_scaled_dimensions, compute_target_dimension};
use crate::error::MergeError;
use crate::exif::{extract_orientation, normalize_orientation};
use crate::overlap::compute_overlaps;
use crate::scale::scale_image;
use crate::types::{BackgroundColor, Direction, MergeOptions};

/// Decodes an image from raw bytes.
fn decode_image(bytes: &[u8]) -> Result<DynamicImage, String> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;

    reader.decode().map_err(|e| e.to_string())
}

/// Merges multiple images into a single output image.
///
/// # Arguments
/// * `images_data` - Vector of raw image bytes for each input image
/// * `options` - Merge options (direction, background)
///
/// # Returns
/// * `Ok(Vec<u8>)` - PNG-encoded output image bytes
/// * `Err(MergeError)` - Error details if merge fails
pub fn merge(images_data: Vec<Vec<u8>>, options: MergeOptions) -> Result<Vec<u8>, MergeError> {
    // Check for empty input
    if images_data.is_empty() {
        return Err(MergeError::NoImages);
    }

    // Step 1: Decode all images and normalize EXIF orientation
    let mut decoded_images: Vec<DynamicImage> = Vec::with_capacity(images_data.len());
    for (index, data) in images_data.iter().enumerate() {
        match decode_image(data) {
            Ok(img) => {
                // Extract EXIF orientation and normalize
                let orientation = extract_orientation(data);
                let normalized = normalize_orientation(img, orientation);
                decoded_images.push(normalized);
            }
            Err(message) => {
                return Err(MergeError::DecodeError {
                    index,
                    file_name: None,
                    message,
                });
            }
        }
    }

    // Step 2: Get dimensions (from normalized images)
    let dimensions: Vec<(u32, u32)> = decoded_images
        .iter()
        .map(|img| (img.width(), img.height()))
        .collect();

    // Step 3: Compute target dimension
    let target = compute_target_dimension(&dimensions, options.direction);
    if target == 0 {
        return Err(MergeError::NoImages);
    }

    // Step 4: Compute scaled dimensions for each image
    let scaled_dimensions: Vec<(u32, u32)> = dimensions
        .iter()
        .map(|(w, h)| compute_scaled_dimensions(*w, *h, target, options.direction))
        .collect();

    // Step 5: Compute output size
    // For Smart mode, we treat it as Vertical for dimension calculation initially
    let direction_for_sizing = match options.direction {
        Direction::Smart => Direction::Vertical,
        d => d,
    };
    let (output_width, output_height) =
        compute_output_size(&scaled_dimensions, direction_for_sizing);

    if output_width > u32::MAX as u64 || output_height > u32::MAX as u64 {
        return Err(MergeError::EncodeError {
            message: "Output dimensions exceed supported size".to_string(),
        });
    }

    let output_width = output_width as u32;
    let mut output_height = output_height as u32;

    // Step 7: Scale all images
    let scaled_images: Vec<DynamicImage> = decoded_images
        .iter()
        .zip(scaled_dimensions.iter())
        .map(|(img, (w, h))| scale_image(img, *w, *h))
        .collect();

    // Step 7.5: For Smart mode, compute overlaps and adjust output height
    let overlaps = if options.direction == Direction::Smart {
        let overlaps = compute_overlaps(&scaled_images, options.overlap_sensitivity);
        let total_overlap: u32 = overlaps.iter().sum();
        output_height = output_height.saturating_sub(total_overlap);
        overlaps
    } else {
        vec![]
    };

    // Step 8: Create output canvas with background color
    let mut output = RgbaImage::from_pixel(
        output_width,
        output_height,
        Rgba([
            options.background.r,
            options.background.g,
            options.background.b,
            options.background.a,
        ]),
    );

    // Step 9: Composite images onto canvas
    let mut offset: u32 = 0;
    for (i, (img, (w, h))) in scaled_images
        .iter()
        .zip(scaled_dimensions.iter())
        .enumerate()
    {
        let rgba_img = img.to_rgba8();

        match options.direction {
            Direction::Vertical => {
                // Center horizontally if width is smaller than output width
                let x_offset = (output_width - w) / 2;
                composite_image(
                    &mut output,
                    &rgba_img,
                    x_offset,
                    offset,
                    &options.background,
                );
                offset += h;
            }
            Direction::Horizontal => {
                // Center vertically if height is smaller than output height
                let y_offset = (output_height - h) / 2;
                composite_image(
                    &mut output,
                    &rgba_img,
                    offset,
                    y_offset,
                    &options.background,
                );
                offset += w;
            }
            Direction::Smart => {
                // Smart mode: vertical stacking with overlap removal
                let x_offset = (output_width - w) / 2;

                if i == 0 {
                    // First image: composite fully
                    composite_image(
                        &mut output,
                        &rgba_img,
                        x_offset,
                        offset,
                        &options.background,
                    );
                    // Subtract overlap from next image (if exists)
                    let overlap = overlaps.get(i).copied().unwrap_or(0);
                    offset += h.saturating_sub(overlap);
                } else {
                    // Subsequent images: skip the overlapping top portion
                    let prev_overlap = overlaps.get(i - 1).copied().unwrap_or(0);
                    composite_image_with_crop(
                        &mut output,
                        &rgba_img,
                        x_offset,
                        offset,
                        prev_overlap, // crop this many pixels from top
                        &options.background,
                    );
                    // For the next image
                    let next_overlap = overlaps.get(i).copied().unwrap_or(0);
                    offset += h.saturating_sub(prev_overlap).saturating_sub(next_overlap);
                }
            }
        }
    }

    // Step 10: Encode to PNG
    let mut output_bytes: Vec<u8> = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut output_bytes);
    DynamicImage::ImageRgba8(output)
        .write_with_encoder(encoder)
        .map_err(|e| MergeError::EncodeError {
            message: e.to_string(),
        })?;

    Ok(output_bytes)
}

/// Composites a source image onto a destination canvas at the given offset.
/// Handles alpha blending with the background color.
fn composite_image(
    dest: &mut RgbaImage,
    src: &RgbaImage,
    x_offset: u32,
    y_offset: u32,
    background: &BackgroundColor,
) {
    for (x, y, pixel) in src.enumerate_pixels() {
        let dest_x = x_offset + x;
        let dest_y = y_offset + y;

        if dest_x < dest.width() && dest_y < dest.height() {
            let blended = blend_with_background(*pixel, background);
            dest.put_pixel(dest_x, dest_y, blended);
        }
    }
}

/// Composites a source image onto a destination canvas, cropping the top portion.
/// Used for Smart merge mode to remove overlapping content.
fn composite_image_with_crop(
    dest: &mut RgbaImage,
    src: &RgbaImage,
    x_offset: u32,
    y_offset: u32,
    crop_top: u32,
    background: &BackgroundColor,
) {
    for (x, y, pixel) in src.enumerate_pixels() {
        // Skip the cropped top portion
        if y < crop_top {
            continue;
        }

        let dest_x = x_offset + x;
        let dest_y = y_offset + (y - crop_top);

        if dest_x < dest.width() && dest_y < dest.height() {
            let blended = blend_with_background(*pixel, background);
            dest.put_pixel(dest_x, dest_y, blended);
        }
    }
}

/// Blends a pixel with the background color based on alpha.
fn blend_with_background(pixel: Rgba<u8>, background: &BackgroundColor) -> Rgba<u8> {
    let alpha = pixel[3] as f32 / 255.0;

    if alpha >= 1.0 {
        return pixel;
    }

    if alpha <= 0.0 {
        return Rgba([background.r, background.g, background.b, background.a]);
    }

    let blend = |fg: u8, bg: u8| -> u8 {
        let fg_f = fg as f32;
        let bg_f = bg as f32;
        ((fg_f * alpha) + (bg_f * (1.0 - alpha))).round() as u8
    };

    Rgba([
        blend(pixel[0], background.r),
        blend(pixel[1], background.g),
        blend(pixel[2], background.b),
        background.a,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_png(width: u32, height: u32, color: Rgba<u8>) -> Vec<u8> {
        let img = RgbaImage::from_pixel(width, height, color);
        let mut bytes = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut bytes);
        DynamicImage::ImageRgba8(img)
            .write_with_encoder(encoder)
            .unwrap();
        bytes
    }

    #[test]
    fn test_merge_no_images() {
        let result = merge(vec![], MergeOptions::default());
        assert!(matches!(result, Err(MergeError::NoImages)));
    }

    #[test]
    fn test_merge_single_image() {
        let img_data = create_test_png(100, 200, Rgba([255, 0, 0, 255]));
        let result = merge(vec![img_data], MergeOptions::default());
        assert!(result.is_ok());

        let output_bytes = result.unwrap();
        let output_img = decode_image(&output_bytes).unwrap();
        assert_eq!(output_img.width(), 100);
        assert_eq!(output_img.height(), 200);
    }

    #[test]
    fn test_merge_vertical() {
        let img1 = create_test_png(100, 50, Rgba([255, 0, 0, 255]));
        let img2 = create_test_png(100, 50, Rgba([0, 255, 0, 255]));

        let options = MergeOptions {
            direction: Direction::Vertical,
            ..Default::default()
        };

        let result = merge(vec![img1, img2], options);
        assert!(result.is_ok());

        let output_bytes = result.unwrap();
        let output_img = decode_image(&output_bytes).unwrap();
        assert_eq!(output_img.width(), 100);
        assert_eq!(output_img.height(), 100); // 50 + 50
    }

    #[test]
    fn test_merge_horizontal() {
        let img1 = create_test_png(50, 100, Rgba([255, 0, 0, 255]));
        let img2 = create_test_png(50, 100, Rgba([0, 255, 0, 255]));

        let options = MergeOptions {
            direction: Direction::Horizontal,
            ..Default::default()
        };

        let result = merge(vec![img1, img2], options);
        assert!(result.is_ok());

        let output_bytes = result.unwrap();
        let output_img = decode_image(&output_bytes).unwrap();
        assert_eq!(output_img.width(), 100); // 50 + 50
        assert_eq!(output_img.height(), 100);
    }

    #[test]
    fn test_merge_vertical_different_widths() {
        // Images with different widths should be scaled to max width
        let img1 = create_test_png(100, 50, Rgba([255, 0, 0, 255]));
        let img2 = create_test_png(200, 50, Rgba([0, 255, 0, 255])); // wider

        let options = MergeOptions {
            direction: Direction::Vertical,
            ..Default::default()
        };

        let result = merge(vec![img1, img2], options);
        assert!(result.is_ok());

        let output_bytes = result.unwrap();
        let output_img = decode_image(&output_bytes).unwrap();
        assert_eq!(output_img.width(), 200); // max width
        // First image scaled from 100x50 to 200x100
        assert_eq!(output_img.height(), 150); // 100 + 50
    }

    #[test]
    fn test_merge_decode_error() {
        let valid_img = create_test_png(100, 100, Rgba([255, 0, 0, 255]));
        let invalid_img = vec![0u8, 1, 2, 3]; // Invalid image data

        let result = merge(vec![valid_img, invalid_img], MergeOptions::default());
        assert!(matches!(
            result,
            Err(MergeError::DecodeError { index: 1, .. })
        ));
    }

    #[test]
    fn test_blend_with_background_opaque() {
        let pixel = Rgba([100, 150, 200, 255]);
        let bg = BackgroundColor::white();
        let blended = blend_with_background(pixel, &bg);
        assert_eq!(blended, pixel);
    }

    #[test]
    fn test_blend_with_background_transparent() {
        let pixel = Rgba([100, 150, 200, 0]);
        let bg = BackgroundColor::white();
        let blended = blend_with_background(pixel, &bg);
        assert_eq!(blended, Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn test_blend_with_background_transparent_bg() {
        let pixel = Rgba([100, 150, 200, 0]);
        let bg = BackgroundColor::transparent();
        let blended = blend_with_background(pixel, &bg);
        assert_eq!(blended, Rgba([0, 0, 0, 0]));
    }

    #[test]
    fn test_blend_with_background_semi_transparent() {
        let pixel = Rgba([0, 0, 0, 128]); // 50% black
        let bg = BackgroundColor::white();
        let blended = blend_with_background(pixel, &bg);
        // Should be roughly 50% gray
        assert!(blended[0] > 100 && blended[0] < 150);
        assert_eq!(blended[3], 255); // White background is opaque
    }

    #[test]
    fn test_blend_with_background_semi_transparent_bg() {
        let pixel = Rgba([255, 0, 0, 128]); // 50% red
        let bg = BackgroundColor {
            r: 0,
            g: 0,
            b: 255,
            a: 128,
        }; // 50% blue
        let blended = blend_with_background(pixel, &bg);
        // Should blend red with blue, keeping background alpha
        assert!(blended[0] > 60 && blended[0] < 140); // red component from blend
        assert!(blended[2] > 60 && blended[2] < 140); // blue component from blend
        assert_eq!(blended[3], 128); // Preserves background alpha
    }
}
