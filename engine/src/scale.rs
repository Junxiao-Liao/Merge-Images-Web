use image::{DynamicImage, imageops::FilterType};

/// Scales an image to the specified dimensions using a deterministic filter.
///
/// Uses Lanczos3 filter for high-quality, deterministic resampling.
///
/// # Panics
/// Panics if new_width or new_height is zero.
pub fn scale_image(img: &DynamicImage, new_width: u32, new_height: u32) -> DynamicImage {
    assert!(
        new_width > 0 && new_height > 0,
        "Scale dimensions must be non-zero"
    );

    let current_width = img.width();
    let current_height = img.height();

    // Skip resize if dimensions match
    if current_width == new_width && current_height == new_height {
        return img.clone();
    }

    img.resize_exact(new_width, new_height, FilterType::Lanczos3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_scale_image_basic() {
        // Create a 10x10 red image
        let mut img = DynamicImage::new_rgba8(10, 10);
        if let Some(rgba) = img.as_mut_rgba8() {
            for pixel in rgba.pixels_mut() {
                *pixel = Rgba([255, 0, 0, 255]);
            }
        }

        let scaled = scale_image(&img, 20, 20);
        assert_eq!(scaled.width(), 20);
        assert_eq!(scaled.height(), 20);
    }

    #[test]
    fn test_scale_image_downscale() {
        let img = DynamicImage::new_rgba8(100, 100);
        let scaled = scale_image(&img, 50, 50);
        assert_eq!(scaled.width(), 50);
        assert_eq!(scaled.height(), 50);
    }

    #[test]
    fn test_scale_image_no_change() {
        let img = DynamicImage::new_rgba8(100, 200);
        let scaled = scale_image(&img, 100, 200);
        assert_eq!(scaled.width(), 100);
        assert_eq!(scaled.height(), 200);
    }

    #[test]
    fn test_scale_image_aspect_change() {
        // This is resize_exact, so aspect ratio can change
        let img = DynamicImage::new_rgba8(100, 100);
        let scaled = scale_image(&img, 200, 100);
        assert_eq!(scaled.width(), 200);
        assert_eq!(scaled.height(), 100);
    }

    #[test]
    #[should_panic]
    fn test_scale_image_zero_dimensions() {
        let img = DynamicImage::new_rgba8(100, 100);
        scale_image(&img, 0, 100);
    }
}
