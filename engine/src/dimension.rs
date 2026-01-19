use crate::types::Direction;

/// Computes the target dimension for scaling.
///
/// - Vertical merge: returns maximum width among inputs
/// - Horizontal merge: returns maximum height among inputs
pub fn compute_target_dimension(dimensions: &[(u32, u32)], direction: Direction) -> u32 {
    if dimensions.is_empty() {
        return 0;
    }

    match direction {
        Direction::Vertical | Direction::Smart => {
            dimensions.iter().map(|(w, _)| *w).max().unwrap_or(0)
        }
        Direction::Horizontal => dimensions.iter().map(|(_, h)| *h).max().unwrap_or(0),
    }
}

/// Computes the new dimensions after scaling to the target.
///
/// - Vertical merge: scales to target width, computes height preserving aspect ratio
/// - Horizontal merge: scales to target height, computes width preserving aspect ratio
///
/// Uses deterministic rounding: `round(value)` via `(value + 0.5).floor()`.
pub fn compute_scaled_dimensions(
    width: u32,
    height: u32,
    target: u32,
    direction: Direction,
) -> (u32, u32) {
    if width == 0 || height == 0 || target == 0 {
        return (0, 0);
    }

    match direction {
        Direction::Vertical | Direction::Smart => {
            // Scale to target width
            let scale = target as f64 / width as f64;
            let new_height = round_half_up(height as f64 * scale);
            (target, new_height.max(1))
        }
        Direction::Horizontal => {
            // Scale to target height
            let scale = target as f64 / height as f64;
            let new_width = round_half_up(width as f64 * scale);
            (new_width.max(1), target)
        }
    }
}

/// Round half up: 0.5 rounds up to 1.
fn round_half_up(value: f64) -> u32 {
    (value + 0.5).floor() as u32
}

/// Computes total output dimensions given scaled image dimensions.
///
/// - Vertical merge: width = max width, height = sum of heights
/// - Horizontal merge: width = sum of widths, height = max height
pub fn compute_output_size(scaled_dimensions: &[(u32, u32)], direction: Direction) -> (u64, u64) {
    if scaled_dimensions.is_empty() {
        return (0, 0);
    }

    match direction {
        Direction::Vertical | Direction::Smart => {
            let width = scaled_dimensions
                .iter()
                .map(|(w, _)| *w as u64)
                .max()
                .unwrap_or(0);
            let height: u64 = scaled_dimensions.iter().map(|(_, h)| *h as u64).sum();
            (width, height)
        }
        Direction::Horizontal => {
            let width: u64 = scaled_dimensions.iter().map(|(w, _)| *w as u64).sum();
            let height = scaled_dimensions
                .iter()
                .map(|(_, h)| *h as u64)
                .max()
                .unwrap_or(0);
            (width, height)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_target_vertical() {
        let dims = vec![(100, 200), (150, 100), (80, 300)];
        let target = compute_target_dimension(&dims, Direction::Vertical);
        assert_eq!(target, 150); // max width
    }

    #[test]
    fn test_compute_target_horizontal() {
        let dims = vec![(100, 200), (150, 100), (80, 300)];
        let target = compute_target_dimension(&dims, Direction::Horizontal);
        assert_eq!(target, 300); // max height
    }

    #[test]
    fn test_compute_target_empty() {
        let dims: Vec<(u32, u32)> = vec![];
        assert_eq!(compute_target_dimension(&dims, Direction::Vertical), 0);
        assert_eq!(compute_target_dimension(&dims, Direction::Horizontal), 0);
    }

    #[test]
    fn test_scaled_dimensions_vertical() {
        // 100x200 scaled to width 150: height = 200 * 1.5 = 300
        let (w, h) = compute_scaled_dimensions(100, 200, 150, Direction::Vertical);
        assert_eq!(w, 150);
        assert_eq!(h, 300);
    }

    #[test]
    fn test_scaled_dimensions_horizontal() {
        // 100x200 scaled to height 400: width = 100 * 2 = 200
        let (w, h) = compute_scaled_dimensions(100, 200, 400, Direction::Horizontal);
        assert_eq!(w, 200);
        assert_eq!(h, 400);
    }

    #[test]
    fn test_scaled_dimensions_no_change() {
        // Already at target
        let (w, h) = compute_scaled_dimensions(100, 200, 100, Direction::Vertical);
        assert_eq!(w, 100);
        assert_eq!(h, 200);
    }

    #[test]
    fn test_scaled_dimensions_downscale() {
        // 200x400 scaled to width 100: height = 400 * 0.5 = 200
        let (w, h) = compute_scaled_dimensions(200, 400, 100, Direction::Vertical);
        assert_eq!(w, 100);
        assert_eq!(h, 200);
    }

    #[test]
    fn test_round_half_up() {
        // 100x150 scaled to width 200: height = 150 * 2 = 300 (exact)
        let (_, h) = compute_scaled_dimensions(100, 150, 200, Direction::Vertical);
        assert_eq!(h, 300);

        // 100x151 scaled to width 200: height = 151 * 2 = 302 (exact)
        let (_, h) = compute_scaled_dimensions(100, 151, 200, Direction::Vertical);
        assert_eq!(h, 302);

        // 100x101 scaled to width 150: height = 101 * 1.5 = 151.5 -> 152 (round up)
        let (_, h) = compute_scaled_dimensions(100, 101, 150, Direction::Vertical);
        assert_eq!(h, 152);
    }

    #[test]
    fn test_scaled_dimensions_zero() {
        assert_eq!(
            compute_scaled_dimensions(0, 100, 200, Direction::Vertical),
            (0, 0)
        );
        assert_eq!(
            compute_scaled_dimensions(100, 0, 200, Direction::Vertical),
            (0, 0)
        );
        assert_eq!(
            compute_scaled_dimensions(100, 200, 0, Direction::Vertical),
            (0, 0)
        );
    }

    #[test]
    fn test_output_size_vertical() {
        // Three images scaled to width 150
        let dims = vec![(150, 300), (150, 150), (150, 200)];
        let (w, h) = compute_output_size(&dims, Direction::Vertical);
        assert_eq!(w, 150);
        assert_eq!(h, 650); // 300 + 150 + 200
    }

    #[test]
    fn test_output_size_horizontal() {
        // Three images scaled to height 300
        let dims = vec![(200, 300), (150, 300), (100, 300)];
        let (w, h) = compute_output_size(&dims, Direction::Horizontal);
        assert_eq!(w, 450); // 200 + 150 + 100
        assert_eq!(h, 300);
    }

    #[test]
    fn test_output_size_empty() {
        let dims: Vec<(u32, u32)> = vec![];
        assert_eq!(compute_output_size(&dims, Direction::Vertical), (0, 0));
    }
}
