use palette::{Srgb, FromColor, Lch};

use super::comparisons::rgb_weighted_euclidean;

#[inline] pub fn collapse_angle(angle: f32) -> f32 {
    ((angle % 360.0) + 360.0) % 360.0
}

pub fn quantize_hue(original_hue: f32, hues: &[f32]) -> f32 {
    let mut closest_dist = f32::MAX;
    let pixel_hue = collapse_angle(original_hue);
    let mut current_hue = pixel_hue;

    for hue in hues.iter() {
        let normalized = collapse_angle(*hue);
        let distance = (normalized - pixel_hue).abs();
        if distance < closest_dist {
            closest_dist = distance;
            current_hue = normalized;
        }
    }

    current_hue
}

pub const ONE_BIT: &[Srgb] = &[
    Srgb::new(0.0, 0.0, 0.0),
    Srgb::new(1.0, 1.0, 1.0),
];

type DistanceFn = fn((f32, f32, f32), (f32, f32, f32)) -> f32; 

fn quantize_colour(
    original: (f32, f32, f32),
    palette: &[(f32, f32, f32)],
    distance_fn: DistanceFn,
) -> (f32, f32, f32) {
    let mut closest_distance = f32::MAX;
    let mut current_colour = &original;

    for colour in palette.iter() {
        let distance = distance_fn(
            original,
            *colour,
        );
        if distance < closest_distance {
            current_colour = colour;
            closest_distance = distance;
        };
    }

    *current_colour
}

pub fn quantize_rgb(original_rgb: Srgb, palette: &[Srgb]) -> Srgb {
    let srgb = quantize_colour(
        original_rgb.into_components(),
        &palette.iter().map(|&col| col.into_components()).collect::<Vec<_>>(),
        rgb_weighted_euclidean
    );

    Srgb::from_components(srgb)
}

pub fn compute_rgb_error(main: Srgb, other: Srgb) -> (f32, f32, f32) {
    (main - other).into_components()
}

pub fn grayscale_rgb(rgb: Srgb) -> Srgb {
    let mut lch = Lch::from_color(rgb);
    lch.chroma = 0.0;
    Srgb::from_color(lch)
}

pub fn hexcode_to_srgb(value: &str) -> Srgb {
    let r = u8::from_str_radix(&value[0..=1], 16);
    let g = u8::from_str_radix(&value[2..=3], 16);
    let b = u8::from_str_radix(&value[4..=5], 16);

    if let (Ok(r), Ok(g), Ok(b)) = (r, g, b) {
        let (r, g, b) = (
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
        );
        Srgb::new(r, g, b)
    } else {
        println!(
            "WARNING! Couldn't convert {value} into an RGB value. Returning black."
        );
        Srgb::new(0.0, 0.0, 0.0)
    }
}