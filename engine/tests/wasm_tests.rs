//! WASM boundary tests for the merge-images-engine.
//!
//! These tests verify the JS↔WASM contract and can be run via:
//! `wasm-pack test --headless --chrome`

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Test fixture: creates a minimal valid PNG image
fn create_test_png(width: u32, height: u32, r: u8, g: u8, b: u8) -> Vec<u8> {
    use image::{DynamicImage, Rgba, RgbaImage};

    let img = RgbaImage::from_pixel(width, height, Rgba([r, g, b, 255]));
    let mut bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut bytes);
    DynamicImage::ImageRgba8(img)
        .write_with_encoder(encoder)
        .unwrap();
    bytes
}

#[wasm_bindgen_test]
fn test_greet_initializes() {
    let result = merge_images_engine::greet();
    assert_eq!(result, "merge-images-engine initialized");
}

#[wasm_bindgen_test]
fn test_merge_empty_array_returns_error() {
    use js_sys::Array;

    let images = Array::new();
    let options = JsValue::undefined();

    let result = merge_images_engine::merge_images(&images, &options);
    assert!(result.is_err());

    // Verify error object structure
    let err = result.unwrap_err();
    let err_obj = js_sys::Object::from(err);
    let code = js_sys::Reflect::get(&err_obj, &JsValue::from_str("code")).unwrap();
    assert_eq!(code.as_string().unwrap(), "NO_IMAGES");
}

#[wasm_bindgen_test]
fn test_merge_vertical_two_images() {
    use js_sys::{Array, Object, Reflect, Uint8Array};

    // Create two 10x10 test images
    let red_png = create_test_png(10, 10, 255, 0, 0);
    let blue_png = create_test_png(10, 10, 0, 0, 255);

    let images = Array::new();
    images.push(&Uint8Array::from(red_png.as_slice()));
    images.push(&Uint8Array::from(blue_png.as_slice()));

    // Create options object
    let options = Object::new();
    Reflect::set(&options, &JsValue::from_str("direction"), &JsValue::from_str("vertical")).unwrap();

    let result = merge_images_engine::merge_images(&images, &options.into());
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.length() > 0);

    // Decode output to verify dimensions
    let output_bytes: Vec<u8> = output.to_vec();
    let cursor = std::io::Cursor::new(output_bytes);
    let reader = image::ImageReader::new(cursor)
        .with_guessed_format()
        .unwrap();
    let decoded = reader.decode().unwrap();

    // 10x10 + 10x10 vertical = 10x20
    assert_eq!(decoded.width(), 10);
    assert_eq!(decoded.height(), 20);
}

#[wasm_bindgen_test]
fn test_merge_horizontal_two_images() {
    use js_sys::{Array, Object, Reflect, Uint8Array};

    // Create two 10x10 test images
    let red_png = create_test_png(10, 10, 255, 0, 0);
    let blue_png = create_test_png(10, 10, 0, 0, 255);

    let images = Array::new();
    images.push(&Uint8Array::from(red_png.as_slice()));
    images.push(&Uint8Array::from(blue_png.as_slice()));

    // Create options object
    let options = Object::new();
    Reflect::set(&options, &JsValue::from_str("direction"), &JsValue::from_str("horizontal")).unwrap();

    let result = merge_images_engine::merge_images(&images, &options.into());
    assert!(result.is_ok());

    let output = result.unwrap();

    // Decode output to verify dimensions
    let output_bytes: Vec<u8> = output.to_vec();
    let cursor = std::io::Cursor::new(output_bytes);
    let reader = image::ImageReader::new(cursor)
        .with_guessed_format()
        .unwrap();
    let decoded = reader.decode().unwrap();

    // 10x10 + 10x10 horizontal = 20x10
    assert_eq!(decoded.width(), 20);
    assert_eq!(decoded.height(), 10);
}

#[wasm_bindgen_test]
fn test_merge_with_background_color() {
    use js_sys::{Array, Object, Reflect, Uint8Array};

    // Create a 10x10 test image
    let red_png = create_test_png(10, 10, 255, 0, 0);

    let images = Array::new();
    images.push(&Uint8Array::from(red_png.as_slice()));

    // Create options with custom background
    let options = Object::new();
    let bg = Object::new();
    Reflect::set(&bg, &JsValue::from_str("r"), &JsValue::from_f64(0.0)).unwrap();
    Reflect::set(&bg, &JsValue::from_str("g"), &JsValue::from_f64(255.0)).unwrap();
    Reflect::set(&bg, &JsValue::from_str("b"), &JsValue::from_f64(0.0)).unwrap();
    Reflect::set(&bg, &JsValue::from_str("a"), &JsValue::from_f64(255.0)).unwrap();
    Reflect::set(&options, &JsValue::from_str("background"), &bg).unwrap();

    let result = merge_images_engine::merge_images(&images, &options.into());
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_merge_invalid_input_type() {
    use js_sys::Array;

    let images = Array::new();
    // Push a string instead of Uint8Array
    images.push(&JsValue::from_str("not an image"));

    let result = merge_images_engine::merge_images(&images, &JsValue::undefined());
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_obj = js_sys::Object::from(err);
    let code = js_sys::Reflect::get(&err_obj, &JsValue::from_str("code")).unwrap();
    assert_eq!(code.as_string().unwrap(), "INVALID_INPUT");
}

#[wasm_bindgen_test]
fn test_merge_decode_error() {
    use js_sys::{Array, Uint8Array};

    let images = Array::new();
    // Push invalid image data
    images.push(&Uint8Array::from(&[0u8, 1, 2, 3][..]));

    let result = merge_images_engine::merge_images(&images, &JsValue::undefined());
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_obj = js_sys::Object::from(err);
    let code = js_sys::Reflect::get(&err_obj, &JsValue::from_str("code")).unwrap();
    assert_eq!(code.as_string().unwrap(), "DECODE_FAILED");
}

#[wasm_bindgen_test]
fn test_merge_default_options() {
    use js_sys::{Array, Uint8Array};

    // Create a 10x10 test image
    let red_png = create_test_png(10, 10, 255, 0, 0);

    let images = Array::new();
    images.push(&Uint8Array::from(red_png.as_slice()));

    // Use undefined/null options - should use defaults
    let result1 = merge_images_engine::merge_images(&images, &JsValue::undefined());
    assert!(result1.is_ok());

    let result2 = merge_images_engine::merge_images(&images, &JsValue::null());
    assert!(result2.is_ok());
}
