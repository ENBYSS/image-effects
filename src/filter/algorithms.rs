
use palette::Srgb;

use crate::{utils::image::RgbPixelRepr, effect::Effect};

use super::raw::{contrast, gradient_map, quantize_hue, brighten, saturate, shift_hue, multiply_hue};

/// Rotates the hue based on the amount of degrees passed.
pub struct HueRotate(
    /// Amount of degrees to rotate the hue by.
    pub f32
);

/// Modifies the contrast of the image.
pub struct Contrast(
    /// A general factor to apply contrast.
    /// 
    /// - Anything higher than `1.0` will add contrast.
    /// - Anything between `0.0` and `1.0` will decrease the contrast.
    /// - Anything below `0.0` will start inverting the image - with `-1.0` being a 
    ///   total inversion while preserving contrast
    pub f32
);

/// Modifies the brightness of the image.
pub struct Brighten(
    /// Factor to change brightness by.
    /// 
    /// This value can range from `-1.0`, which turns all pixels black, and `1.0`, which makes
    /// all pixels white.
    pub f32
);

/// Modifies the saturation of the image.
pub struct Saturate(
    /// Factor to affect saturation by.
    /// 
    /// This value can range from `-1.0`, which removes all saturation, and `1.0`, which maximizes
    /// all saturation.
    /// 
    /// Internally, `Saturate(1.0)` would mean setting each pixel to `128.0 chroma` in LCH terms -
    /// despite Chroma being technically unbounded.
    /// 
    /// This may change in the future.
    pub f32
);

/// Applies a gradient map to the image.
///
/// The gradient map is defined as a slice of tuples containing the *colour* and its threshold.
/// Each pixel in the image will be mapped to the gradient using its luminance value.
///
/// The threshold must be between `0.0` and `1.0` - you can technically use other values but the results
/// may be a bit weird.
///
/// As an example, to turn an image grayscale you could pass the colour black at `0.0` and the colour
/// white at `1.0`.
#[derive(Default)]
pub struct GradientMap {
    map: Vec<(Srgb, f32)>
}

impl GradientMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new gradient map from an existing map.
    pub fn with_map(map: Vec<(Srgb, f32)>) -> Self {
        Self { map }
    }

    /// Add an entry into the gradient map.
    pub fn add_entry(&mut self, colour: Srgb, luminance: f32) -> &mut Self {
        self.map.push((colour, luminance));
        self
    }
}

/// Quantizes the hue of each pixel to one of the hues passed.
///
/// This *only* changes the hue - useful for defining a colour
/// scheme without losing luminance/saturation detail.
#[derive(Default)]
pub struct QuantizeHue {
    hues: Vec<f32>
}

impl QuantizeHue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a `QuantizeHue` effect with the given hues.
    pub fn with_hues(hues: Vec<f32>) -> Self {
        Self { hues }
    }

    /// Add a hue to the list.
    pub fn add_hue(&mut self, hue: f32) -> &mut Self {
        self.hues.push(hue);
        self
    }
}

/// Multiplies the hue of each pixel by the factor passed.
pub struct MultiplyHue(pub f32);

/// Inverts the colours of the image. Effectively the same as `Contrast(-1.0)`
pub struct Invert;

impl Effect<RgbPixelRepr> for HueRotate {
    fn affect(&self, item: RgbPixelRepr) -> RgbPixelRepr {
        shift_hue(item, self.0)
    }
}

impl Effect<RgbPixelRepr> for Contrast {
    fn affect(&self, item: RgbPixelRepr) -> RgbPixelRepr {
        contrast(item, self.0)
    }
}

impl Effect<RgbPixelRepr> for Brighten {
    fn affect(&self, item: RgbPixelRepr) -> RgbPixelRepr {
        brighten(item, self.0)
    }
}

impl Effect<RgbPixelRepr> for Saturate {
    fn affect(&self, item: RgbPixelRepr) -> RgbPixelRepr {
        saturate(item, self.0)
    }
}

impl Effect<RgbPixelRepr> for QuantizeHue {
    fn affect(&self, item: RgbPixelRepr) -> RgbPixelRepr {
        quantize_hue(item, &self.hues)
    }
}

impl Effect<RgbPixelRepr> for GradientMap {
    fn affect(&self, item: RgbPixelRepr) -> RgbPixelRepr {
        gradient_map(item, &self.map).map_or(item, |colour| colour.into_format().into())
    }
}

impl Effect<RgbPixelRepr> for MultiplyHue {
    fn affect(&self, item: RgbPixelRepr) -> RgbPixelRepr {
        multiply_hue(item, self.0)
    }
}

impl Effect<RgbPixelRepr> for Invert {
    fn affect(&self, item: RgbPixelRepr) -> RgbPixelRepr {
        Contrast(-1.0).affect(item)
    }
}