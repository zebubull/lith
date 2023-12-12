/// Convert an sRGB value s to its linear RGB equivalent
pub fn srgb_to_linear(s: f32) -> f32 {
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// Get the luminance Y of an sRGB pixel slice
pub fn srgb_to_luminance(pixel: &[u8]) -> f32 {
    assert_eq!(
        pixel.len(),
        3,
        "Pixel length was not 3. Did you really pass in a pixel slice?"
    );
    let (r, g, b) = (
        pixel[0] as f32 / 255.0,
        pixel[1] as f32 / 255.0,
        pixel[2] as f32 / 255.0,
    );
    srgb_to_linear(r) * 0.2126 + srgb_to_linear(g) * 0.7152 + srgb_to_linear(b) * 0.0722
}

/// Convert a gamma value on the interval \[0, 255] to a percieved lightness value.
pub fn luminance_to_lightness(y: f32) -> f32 {
    // https://stackoverflow.com/a/56678483
    if y < 216.0 / 24389.0 {
        y * 24389.0 / 27.0
    } else {
        (y.powf(1.0 / 3.0) * 116.0) - 16.0
    }
}
