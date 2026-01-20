mod chrome_strip;
mod dimension;
mod error;
mod exif;
mod merge;
mod overlap;
mod scale;
mod types;

pub use error::MergeError;
pub use types::{BackgroundColor, Direction, MergeOptions};

use js_sys::{Array, Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;

/// Initialize the engine (for compatibility check).
#[wasm_bindgen]
pub fn greet() -> String {
    "merge-images-engine initialized".to_string()
}

/// Merges multiple images into a single output image.
///
/// # Arguments
/// * `images_data` - JS Array of Uint8Array, each containing raw image bytes
/// * `options` - JS Object with merge options:
///   - `direction`: "vertical" | "horizontal" | "smart"
///   - `background`: { r, g, b, a } (0-255 each)
///   - `overlapSensitivity`: 0-100 (smart mode only)
///
/// # Returns
/// * On success: Uint8Array containing PNG-encoded output
/// * On error: throws a JS error with structured details
#[wasm_bindgen]
pub fn merge_images(images_data: &Array, options: &JsValue) -> Result<Uint8Array, JsValue> {
    // Validate input array
    let length = images_data.length();
    if length == 0 {
        return Err(create_error_object(&MergeError::NoImages));
    }

    // Parse images array with bounds checking, avoiding unnecessary copies
    let mut images: Vec<Vec<u8>> = Vec::with_capacity(length as usize);
    for i in 0..length {
        let item = images_data.get(i);
        if !item.is_instance_of::<Uint8Array>() {
            let obj = Object::new();
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("code"),
                &JsValue::from_str("INVALID_INPUT"),
            );
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("message"),
                &JsValue::from_str("Expected Uint8Array at index"),
            );
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("fileIndex"),
                &JsValue::from_f64(i as f64),
            );
            return Err(obj.into());
        }
        let uint8_array = Uint8Array::new(&item);
        let len = uint8_array.length();
        let mut vec = vec![0u8; len as usize];
        uint8_array.copy_to(&mut vec);
        images.push(vec);
    }

    // Parse options
    let merge_options = parse_options(options)?;

    // Run merge
    match merge::merge(images, merge_options) {
        Ok(output_bytes) => {
            let result = Uint8Array::new_with_length(output_bytes.len() as u32);
            result.copy_from(&output_bytes);
            Ok(result)
        }
        Err(e) => Err(create_error_object(&e)),
    }
}

/// Parses JS options object into MergeOptions.
fn parse_options(options: &JsValue) -> Result<MergeOptions, JsValue> {
    let mut merge_options = MergeOptions::default();

    if options.is_undefined() || options.is_null() {
        return Ok(merge_options);
    }

    // Parse direction
    if let Ok(dir_val) = Reflect::get(options, &JsValue::from_str("direction"))
        && let Some(dir_str) = dir_val.as_string()
    {
        merge_options.direction = match dir_str.as_str() {
            "horizontal" => Direction::Horizontal,
            "smart" => Direction::Smart,
            _ => Direction::Vertical,
        };
    }

    // Parse background
    if let Ok(bg_val) = Reflect::get(options, &JsValue::from_str("background"))
        && !bg_val.is_undefined()
        && !bg_val.is_null()
    {
        let r = get_u8_field(&bg_val, "r").unwrap_or(255);
        let g = get_u8_field(&bg_val, "g").unwrap_or(255);
        let b = get_u8_field(&bg_val, "b").unwrap_or(255);
        let a = get_u8_field(&bg_val, "a").unwrap_or(255);
        merge_options.background = BackgroundColor::new(r, g, b, a);
    }

    if let Ok(sensitivity_val) = Reflect::get(options, &JsValue::from_str("overlapSensitivity"))
        && !sensitivity_val.is_undefined()
        && !sensitivity_val.is_null()
        && let Some(sensitivity) = sensitivity_val
            .as_f64()
            .filter(|value| value.is_finite())
            .map(|value| value.round() as i64)
    {
        merge_options.overlap_sensitivity = sensitivity.clamp(0, 100) as u8;
    }

    Ok(merge_options)
}

/// Gets a u8 field from a JS object.
fn get_u8_field(obj: &JsValue, field: &str) -> Option<u8> {
    Reflect::get(obj, &JsValue::from_str(field))
        .ok()
        .and_then(|v| v.as_f64())
        .filter(|n| n.is_finite())
        .map(|n| n.clamp(0.0, 255.0) as u8)
}

/// Creates a structured JS error object from a MergeError.
fn create_error_object(error: &MergeError) -> JsValue {
    let obj = Object::new();

    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("code"),
        &JsValue::from_str(error.code()),
    );
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("message"),
        &JsValue::from_str(&error.to_string()),
    );

    // Add error-specific details
    if let MergeError::DecodeError {
        index, file_name, ..
    } = error
    {
        let _ = Reflect::set(
            &obj,
            &JsValue::from_str("fileIndex"),
            &JsValue::from_f64(*index as f64),
        );
        if let Some(name) = file_name {
            let _ = Reflect::set(
                &obj,
                &JsValue::from_str("fileName"),
                &JsValue::from_str(name),
            );
        }
    }

    obj.into()
}
