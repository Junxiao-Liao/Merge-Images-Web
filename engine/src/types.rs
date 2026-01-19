use serde::{Deserialize, Serialize};

/// Merge direction - vertical stacks images top to bottom, horizontal stacks left to right.
/// Smart mode is vertical with automatic overlap detection and removal.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    #[default]
    Vertical,
    Horizontal,
    Smart,
}

/// Background fill color for transparent areas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for BackgroundColor {
    fn default() -> Self {
        // Default to opaque white
        BackgroundColor {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        }
    }
}

impl BackgroundColor {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        BackgroundColor { r, g, b, a }
    }

    pub fn white() -> Self {
        Self::default()
    }

    pub fn black() -> Self {
        BackgroundColor {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    pub fn transparent() -> Self {
        BackgroundColor {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}

/// Options for the merge operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeOptions {
    #[serde(default)]
    pub direction: Direction,
    #[serde(default)]
    pub background: BackgroundColor,
    #[serde(default = "default_overlap_sensitivity")]
    pub overlap_sensitivity: u8,
}

impl Default for MergeOptions {
    fn default() -> Self {
        MergeOptions {
            direction: Direction::default(),
            background: BackgroundColor::default(),
            overlap_sensitivity: default_overlap_sensitivity(),
        }
    }
}

fn default_overlap_sensitivity() -> u8 {
    35
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_default() {
        assert_eq!(Direction::default(), Direction::Vertical);
    }

    #[test]
    fn test_background_default() {
        let bg = BackgroundColor::default();
        assert_eq!(bg.r, 255);
        assert_eq!(bg.g, 255);
        assert_eq!(bg.b, 255);
        assert_eq!(bg.a, 255);
    }

    #[test]
    fn test_background_presets() {
        let black = BackgroundColor::black();
        assert_eq!(black.r, 0);
        assert_eq!(black.a, 255);

        let transparent = BackgroundColor::transparent();
        assert_eq!(transparent.a, 0);
    }

    #[test]
    fn test_options_default() {
        let opts = MergeOptions::default();
        assert_eq!(opts.direction, Direction::Vertical);
        assert_eq!(opts.overlap_sensitivity, default_overlap_sensitivity());
    }
}
