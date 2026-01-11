//! EXIF orientation parsing and image normalization.
//!
//! Best-effort EXIF orientation extraction for JPEG images.
//! Other formats (PNG, GIF, WebP) don't carry EXIF orientation and return Normal.

use image::DynamicImage;

/// EXIF orientation values (1-8).
/// See: https://exiftool.org/TagNames/EXIF.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    /// Normal (no transformation needed)
    #[default]
    Normal = 1,
    /// Flip horizontal
    FlipHorizontal = 2,
    /// Rotate 180°
    Rotate180 = 3,
    /// Flip vertical
    FlipVertical = 4,
    /// Rotate 90° CW + flip horizontal
    Rotate90FlipH = 5,
    /// Rotate 90° CW
    Rotate90 = 6,
    /// Rotate 90° CCW + flip horizontal
    Rotate270FlipH = 7,
    /// Rotate 90° CCW (270° CW)
    Rotate270 = 8,
}

impl From<u16> for Orientation {
    fn from(value: u16) -> Self {
        match value {
            1 => Orientation::Normal,
            2 => Orientation::FlipHorizontal,
            3 => Orientation::Rotate180,
            4 => Orientation::FlipVertical,
            5 => Orientation::Rotate90FlipH,
            6 => Orientation::Rotate90,
            7 => Orientation::Rotate270FlipH,
            8 => Orientation::Rotate270,
            _ => Orientation::Normal,
        }
    }
}

/// Extract EXIF orientation from image bytes.
///
/// Currently only supports JPEG. Other formats return `Orientation::Normal`.
pub fn extract_orientation(bytes: &[u8]) -> Orientation {
    // Check for JPEG magic bytes
    if bytes.len() < 2 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return Orientation::Normal;
    }

    // Parse JPEG segments looking for APP1 (EXIF)
    parse_jpeg_exif(bytes).unwrap_or(Orientation::Normal)
}

/// Parse JPEG EXIF data to find orientation tag.
fn parse_jpeg_exif(bytes: &[u8]) -> Option<Orientation> {
    let mut pos = 2; // Skip SOI marker

    while pos + 4 <= bytes.len() {
        // Each segment starts with 0xFF
        if bytes[pos] != 0xFF {
            return None;
        }

        let marker = bytes[pos + 1];

        // Skip padding bytes
        if marker == 0xFF {
            pos += 1;
            continue;
        }

        // End of image or start of scan
        if marker == 0xD9 || marker == 0xDA {
            return None;
        }

        // Get segment length (big-endian, includes length bytes)
        if pos + 4 > bytes.len() {
            return None;
        }
        let length = u16::from_be_bytes([bytes[pos + 2], bytes[pos + 3]]) as usize;

        // APP1 marker (0xE1) contains EXIF
        if marker == 0xE1 {
            let segment_start = pos + 4;
            let segment_end = pos + 2 + length;
            if segment_end <= bytes.len() {
                let segment = &bytes[segment_start..segment_end];
                if let Some(orientation) = parse_exif_segment(segment) {
                    return Some(orientation);
                }
            }
        }

        // Move to next segment
        pos += 2 + length;
    }

    None
}

/// Parse EXIF segment to find orientation tag.
fn parse_exif_segment(segment: &[u8]) -> Option<Orientation> {
    // Check for "Exif\0\0" header
    if segment.len() < 14 || &segment[0..6] != b"Exif\0\0" {
        return None;
    }

    let tiff_data = &segment[6..];

    // Parse TIFF header to determine endianness
    let (is_little_endian, ifd_offset) = parse_tiff_header(tiff_data)?;

    // Parse IFD0 for orientation tag
    parse_ifd_for_orientation(tiff_data, ifd_offset as usize, is_little_endian)
}

/// Parse TIFF header, returns (is_little_endian, ifd_offset).
fn parse_tiff_header(data: &[u8]) -> Option<(bool, u32)> {
    if data.len() < 8 {
        return None;
    }

    let is_little_endian = match &data[0..2] {
        b"II" => true,  // Intel byte order (little-endian)
        b"MM" => false, // Motorola byte order (big-endian)
        _ => return None,
    };

    let read_u16 = |offset: usize| -> u16 {
        if is_little_endian {
            u16::from_le_bytes([data[offset], data[offset + 1]])
        } else {
            u16::from_be_bytes([data[offset], data[offset + 1]])
        }
    };

    let read_u32 = |offset: usize| -> u32 {
        if is_little_endian {
            u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ])
        } else {
            u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ])
        }
    };

    // Check TIFF magic number (42)
    if read_u16(2) != 42 {
        return None;
    }

    let ifd_offset = read_u32(4);
    Some((is_little_endian, ifd_offset))
}

/// Parse IFD looking for orientation tag.
const ORIENTATION_TAG: u16 = 0x0112;

fn parse_ifd_for_orientation(
    data: &[u8],
    ifd_offset: usize,
    is_little_endian: bool,
) -> Option<Orientation> {
    if ifd_offset + 2 > data.len() {
        return None;
    }

    let read_u16 = |offset: usize| -> u16 {
        if is_little_endian {
            u16::from_le_bytes([data[offset], data[offset + 1]])
        } else {
            u16::from_be_bytes([data[offset], data[offset + 1]])
        }
    };

    let entry_count = read_u16(ifd_offset) as usize;
    let entries_start = ifd_offset + 2;

    // Each IFD entry is 12 bytes
    for i in 0..entry_count {
        let entry_offset = entries_start + i * 12;
        if entry_offset + 12 > data.len() {
            break;
        }

        let tag = read_u16(entry_offset);

        // Orientation tag
        if tag == ORIENTATION_TAG {
            // Value is at offset + 8 (for SHORT type, value is inline)
            let value = read_u16(entry_offset + 8);
            return Some(Orientation::from(value));
        }
    }

    None
}

/// Apply orientation transform to normalize image.
///
/// Transforms the image so it displays correctly regardless of how it was
/// captured/stored.
pub fn normalize_orientation(img: DynamicImage, orientation: Orientation) -> DynamicImage {
    match orientation {
        Orientation::Normal => img,
        Orientation::FlipHorizontal => img.fliph(),
        Orientation::Rotate180 => img.rotate180(),
        Orientation::FlipVertical => img.flipv(),
        Orientation::Rotate90FlipH => img.rotate90().fliph(),
        Orientation::Rotate90 => img.rotate90(),
        Orientation::Rotate270FlipH => img.rotate270().fliph(),
        Orientation::Rotate270 => img.rotate270(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orientation_from_value() {
        assert_eq!(Orientation::from(1), Orientation::Normal);
        assert_eq!(Orientation::from(6), Orientation::Rotate90);
        assert_eq!(Orientation::from(3), Orientation::Rotate180);
        assert_eq!(Orientation::from(8), Orientation::Rotate270);
        assert_eq!(Orientation::from(99), Orientation::Normal); // Invalid -> Normal
    }

    #[test]
    fn test_orientation_default() {
        assert_eq!(Orientation::default(), Orientation::Normal);
    }

    #[test]
    fn test_extract_orientation_non_jpeg() {
        // PNG magic bytes
        let png_bytes = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(extract_orientation(&png_bytes), Orientation::Normal);

        // Empty bytes
        assert_eq!(extract_orientation(&[]), Orientation::Normal);

        // Too short
        assert_eq!(extract_orientation(&[0xFF]), Orientation::Normal);
    }

    #[test]
    fn test_extract_orientation_jpeg_no_exif() {
        // Minimal JPEG with no APP1 segment
        let jpeg_bytes = [
            0xFF, 0xD8, // SOI
            0xFF, 0xD9, // EOI
        ];
        assert_eq!(extract_orientation(&jpeg_bytes), Orientation::Normal);
    }

    #[test]
    fn test_normalize_identity() {
        let img = DynamicImage::new_rgba8(10, 20);
        let normalized = normalize_orientation(img.clone(), Orientation::Normal);
        assert_eq!(normalized.width(), 10);
        assert_eq!(normalized.height(), 20);
    }

    #[test]
    fn test_normalize_rotate90() {
        let img = DynamicImage::new_rgba8(10, 20);
        let normalized = normalize_orientation(img, Orientation::Rotate90);
        // 90° rotation swaps dimensions
        assert_eq!(normalized.width(), 20);
        assert_eq!(normalized.height(), 10);
    }

    #[test]
    fn test_normalize_rotate180() {
        let img = DynamicImage::new_rgba8(10, 20);
        let normalized = normalize_orientation(img, Orientation::Rotate180);
        // 180° rotation keeps dimensions
        assert_eq!(normalized.width(), 10);
        assert_eq!(normalized.height(), 20);
    }

    #[test]
    fn test_normalize_rotate270() {
        let img = DynamicImage::new_rgba8(10, 20);
        let normalized = normalize_orientation(img, Orientation::Rotate270);
        // 270° rotation swaps dimensions
        assert_eq!(normalized.width(), 20);
        assert_eq!(normalized.height(), 10);
    }

    #[test]
    fn test_normalize_flip_horizontal() {
        let img = DynamicImage::new_rgba8(10, 20);
        let normalized = normalize_orientation(img, Orientation::FlipHorizontal);
        assert_eq!(normalized.width(), 10);
        assert_eq!(normalized.height(), 20);
    }

    #[test]
    fn test_normalize_flip_vertical() {
        let img = DynamicImage::new_rgba8(10, 20);
        let normalized = normalize_orientation(img, Orientation::FlipVertical);
        assert_eq!(normalized.width(), 10);
        assert_eq!(normalized.height(), 20);
    }
}
