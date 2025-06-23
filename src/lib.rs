//! This crate provides multiple effects that can be applied on an image.
//!
//! Currently there's two classes of effects:
//!
//! 1. [**Dithering**](./dither/index.html) - Limiting the colour palette of a given image while still
//!    retaining more detail than a purely quantized approach would.
//! 2. [**Filtering**](./filter/algorithms/index.html) - Some more common effects applied on an image, such as brightness,
//!    contrast, gradient mapping, and more.
//!
//! The library lets these effects work on a variety of types, including some from the `image` crate. If you're not using
//! `image` however, you can rely on the implementations on intermediate types:
//! 
//! - For pixels, it's `[u8; 3]` for RGB and `[u8; 4]` for RGBA.
//! - For images, it's `Vec<Vec<{Pixel}>>` - where `Pixel` is anything listed above.
//!
//! The *prelude* is useful for importing some common functionality, like the algorithms themselves
//! alongside some traits.
//! 
//! **Note:** Colour palettes _currently_ require the `palette` crate - as 
//! they are defined using its `Srgb` type.
//! 
//! # Usage
//! 
//! This usage is simplified to focus on the logic in this crate, rather than on `image` or `palette`.
//! 
//! ```ignore
//! use image::DynamicImage;
//! use image_effects::{
//!     prelude::*
//!     dither::FLOYD_STEINBERG,
//!     palette::named,
//! }
//! 
//! fn get_image() -> DynamicImage { /* ... */ }
//! 
//! fn main() {
//!     let image = get_image();
//!     let palette = vec![
//!         named::BLACK, named::WHITE
//!     ];
//! 
//!     image
//!         .apply(&filters::HueRotate(180.0))
//!         .apply(&FLOYD_STEINBERG.with_palette(palette))
//!         .save("image.png");
//! }
//! ```
//! 
//! # Effects
//! 
//! [`Effect<T>`](effect/trait.Effect.html) is the trait to implement here, since
//! [`Affectable<T, E>`](effect/trait.Affectable.html) will be automatically implemented.
//! 
//! Basically, `Effect<T>` defines an effect that can be applied on `T` - so in turn `Affectable` can depend on
//! this implementation to attach an `.apply` method _onto_ `T` that accepts `Effect<T>`.
//! 
//! In other words, if I implement `Effect<Image>` on an effect called `Brighten`, I can then call `.apply` on
//! any `Image` and pass in a reference to `Brighten`.
//! 
//! This also means that although you can define your own effects easily, the same isn't for new kinds of image.
//! You can implement `Affectable`, but not `Effect` due to the external trait rule:
//! 
//! > When implementing a trait, you must either own _the trait_ or _the struct_.
//! 
//! Since external crates don't own `Effect<T>` _nor_ the effects provided by this library, this effectively locks
//! you out of defining new `T`s directly.
//! 
//! However, since most effects get implemented using an intermediate format, such as `[u8; 3]` for an RGB pixel or
//! `Vec<Vec<[u8; 3]>>` for an image, theoretically you just need to convert whatever image/medium you'd like to apply
//! effects on into one of these intermediate formats.
//! 
//! Also, when creating an effect you don't need to define every single image it's compatible too - as auto-implementation
//! happens here as well. For example, if you implement an effect that can be applied on `[u8; 3]`, this will also result 
//! in implementations for RGBA `[u8; 4]`, images, and beyond. As a result, it's always best to define an `Effect` on the simplest
//! possible type.


/// Various dithering algorithms, including both error propagation and ordered.
/// 
/// For error propagation, existing algorithms are setup as constants. These aren't
/// directly usable as an effect since they need to be configured with a palette.
/// 
/// As for `Bayer`, it requires a palette to be initialized with, so it doesn't face
/// this issue.
pub mod dither;

/// Filters that can be applied to the image - such as brightness, contrast, and more.
pub mod filter;

/// Databending methods, which work by modifying the raw bytes.
pub mod databend;

/// Utilities. Mostly just for the test cases - will probably be removed.
mod utils;

/// Colour related logic, such as distance functions, palettes, gradient generation, etc.
pub mod colour;

/// Traits and implementations for _effects_ and anything that can be affected by them.
pub mod effect;

/// Prelude for including the useful elements from the library - including algorithms, traits, and constants.
pub mod prelude {
    // algorithms
    pub use crate::dither;
    pub use crate::filter::filters;
    
    // traits
    pub use crate::effect::Effect;
    pub use crate::effect::Affectable;
    pub use crate::colour::gradient::{
        IntoGradient,
        IntoGradientHsl,
        IntoGradientLch,
        IntoGradientOklch,
    };

    // constants
    pub use crate::colour::colours::srgb as SrgbColour;
    pub use crate::colour::palettes;
}

#[macro_export]
/// Helps construct a gradient map from colours.
///
/// You *could* construct the map yourself, however the purpose of this is mostly to
/// provide an easily usable and *clean* way to construct a gradient map.
///
/// The following is an example usage of this macro:
/// ```ignore
/// let hsl: GradientMap<Hsl<Srgb>> = gradient_map!(
///     0.00 => Hsl::new(0.0, 0.0, 0.0),
///     1.00 => Hsl::new(0.0, 0.0, 1.0),
/// );
/// ```
macro_rules! gradient_map {
    [$($threshold:expr => $color:expr),*] => {
        &[
            $(
                ($color, $threshold)
            ),*
        ]
    };
}

pub type GradientMap<'a, Color> = &'a [(Color, f32)];


#[cfg(test)]
mod test {
    use std::{error::Error, f64::consts::PI};

    use image::{DynamicImage, ImageResult, GenericImageView, imageops};
    use ndarray::{array, Array};
    use num::{Float, Signed};
    use palette::{Srgb, named};

    use crate::{
        colour::utils::ONE_BIT, dither::{ordered::{Ordered, OrderedStrategy::*, Orientation}, ATKINSON, BURKES, FLOYD_STEINBERG, JARVIS_JUDICE_NINKE, SIERRA, SIERRA_LITE, SIERRA_TWO_ROW, STUCKI}, prelude::{palettes::{EIGHT_BIT, WEB_SAFE}, *}
    };

    type UtilResult<T> = Result<T,Box<dyn Error>>;

    // From Unsplash, and more specifically Ravi Sharma.
    // https://unsplash.com/photos/sun-peeping-on-ice-mountain-hNv5s6NEYig
    const IMAGE_URL: &'static str = "https://images.unsplash.com/photo-1580826237584-fda5b612e1bc?q=80&w=1827&auto=format&fit=crop&ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D";
    const MAX_DIM: Option<usize> = Some(500);

    fn get_image() -> UtilResult<DynamicImage> {
        let img_bytes = reqwest::blocking::get(IMAGE_URL)?.bytes()?;
        let image = image::load_from_memory(&img_bytes)?;
        
        let image = if let Some(max_dim) = MAX_DIM {
            let (x, y) = image.dimensions();
            if max_dim < x.max(y) as usize {
                let image = &image;let factor = max_dim as f32 / x.max(y) as f32;
                let mul = |int: u32, float: f32| (int as f32 * float) as u32;
                image.resize(mul(x, factor), mul(y, factor), imageops::Nearest)
            } else { image }
        } else { image };

        Ok(image)
    }

    #[test]
    fn dither_test() -> UtilResult<()> {
        let image = get_image()?;

        let palette = [
            named::PURPLE.into_format().build_gradient_lch(10),
            named::GOLD.into_format().build_gradient_lch(10),
        ].concat();

        dither(&image, ONE_BIT.to_vec(), Some("-mono"))?;
        dither(&image, WEB_SAFE.to_vec(), Some("-web-safe"))?;
        dither(&image, EIGHT_BIT.to_vec(), Some("-8-bit"))?;
        dither(&image, palette, Some("-custom-palette"))?;

        Ok(())
    }

    #[test]
    fn experiment_test() -> UtilResult<()> {
        let stars_arr = array![
            [7., 6., 5., 4., 3., 2., 1., 2., 3., 4., 5., 6., 7.],
            [6., 4., 2., 1., 2., 1., 2., 1., 2., 1., 2., 4., 6.],
            [5., 4., 1., 0., 1., 0., 3., 0., 1., 0., 1., 4., 5.],
            [4., 3., 2., 1., 0., 1., 4., 1., 0., 1., 2., 3., 4.],
            [3., 2., 1., 0., 1., 2., 5., 2., 1., 0., 1., 2., 3.],
            [2., 1., 0., 1., 2., 4., 6., 4., 2., 1., 0., 1., 2.],
            [1., 2., 3., 4., 5., 6., 7., 6., 5., 4., 3., 2., 1.],
            [2., 1., 0., 1., 2., 4., 6., 4., 2., 1., 0., 1., 2.],
            [3., 2., 1., 0., 1., 2., 5., 2., 1., 0., 1., 2., 3.],
            [4., 3., 2., 1., 0., 1., 4., 1., 0., 1., 2., 3., 4.],
            [5., 4., 1., 0., 1., 0., 3., 0., 1., 0., 1., 4., 5.],
            [6., 4., 2., 1., 2., 1., 2., 1., 2., 1., 2., 4., 6.],
            [7., 6., 5., 4., 3., 2., 1., 2., 3., 4., 5., 6., 7.],
        ];

        let normalized = stars_arr / 7.;

        Ok(())
    }

    #[test]
    fn filter_effects_test() -> UtilResult<()> {
        let image = get_image()?;

        image
            .clone()
            .apply(&filters::HueRotate(180.0))
            .save("data/colour/rotate-hue-180.png")?;
        image
            .clone()
            .apply(&filters::Brighten( 0.2))
            .save("data/colour/brighten+0.2.png")?;
        image
            .clone()
            .apply(&filters::Brighten(-0.2))
            .save("data/colour/brighten-0.2.png")?;
        image
            .clone()
            .apply(&filters::Saturate( 0.2))
            .save("data/colour/saturate+0.2.png")?;
        image
            .clone()
            .apply(&filters::Saturate(-0.2))
            .save("data/colour/saturate-0.2.png")?;
        image
            .clone()
            .apply(&filters::Contrast(0.5))
            .save("data/colour/contrast.0.5.png")?;
        image
            .clone()
            .apply(&filters::Contrast(1.5))
            .save("data/colour/contrast.1.5.png")?;

        let _gradient_map = [
            (Srgb::new(0.0, 0.0, 1.0), 0.00),
            (Srgb::new(1.0, 0.0, 0.0), 0.50),
            (Srgb::new(0.0, 1.0, 0.0), 1.00),
        ];

        let mut gradient_map = filters::GradientMap::new();
        gradient_map
            .add_entry(Srgb::new(0.0, 0.0, 1.0), 0.00)
            .add_entry(Srgb::new(1.0, 0.0, 0.0), 0.50)
            .add_entry(Srgb::new(0.0, 1.0, 0.0), 1.00);

        image
            .clone()
            .apply(&gradient_map)
            .save("data/colour/gradient-mapped.png")?;

        let hue_palette = vec![180.0, 300.0];

        image
            .clone()
            .apply(&filters::QuantizeHue::with_hues(hue_palette))
            .save("data/colour/quantize-hue.png")?;

        image
            .clone()
            .apply(&filters::MultiplyHue(4.0))
            .save("data/colour/multiply-hue.4.0.png")?;

        image
            .clone()
            .apply(&filters::MultiplyHue(12.0))
            .save("data/colour/multiply-hue.12.0.png")?;

        image.clone()
            .apply(&filters::Invert)
            .save("data/colour/invert.png")?;

        Ok(())
    }

    fn dither(
        image: &DynamicImage,
        palette: Vec<Srgb>,
        opt_postfix: Option<&str>,
    ) -> ImageResult<()> {
        let postfix = opt_postfix.unwrap_or("");

        let error_propagators = vec![
            FLOYD_STEINBERG,
            JARVIS_JUDICE_NINKE,
            STUCKI,
            ATKINSON,
            BURKES,
            SIERRA,
            SIERRA_TWO_ROW,
            SIERRA_LITE
        ];

        for propagator in error_propagators.into_iter() {
            image.clone()
                .apply(&propagator.with_palette(palette.clone()))
                .save(format!("data/dither/{}{}.png", propagator.name, postfix))?;
        }

        image.clone().apply(&Ordered::new(palette.clone(), Bayer(2)))
            .save(format!("data/dither/bayer-2x2{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), Bayer(4)))
            .save(format!("data/dither/bayer-4x4{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), Bayer(8)))
            .save(format!("data/dither/bayer-8x8{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), Bayer(16)))
            .save(format!("data/dither/bayer-16x16{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), Diamonds(8)))
            .save(format!("data/dither/diamonds-8x8{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), Diamonds(12)))
            .save(format!("data/dither/diamonds-12x12{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), Diamonds(16)))
            .save(format!("data/dither/diamonds-16x16{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), CheckeredDiamonds(8)))
            .save(format!("data/dither/checkered-diamonds-8x8{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), CheckeredDiamonds(12)))
            .save(format!("data/dither/checkered-diamonds-12x12{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), CheckeredDiamonds(16)))
            .save(format!("data/dither/checkered-diamonds-16x16{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), DiagonalTiles(2)))
            .save(format!("data/dither/diagonal-tiles-2x2{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), DiagonalTiles(4)))
            .save(format!("data/dither/diagonal-tiles-4x4{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), DiagonalTiles(8)))
            .save(format!("data/dither/diagonal-tiles-8x8{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), DiagonalTiles(16)))
            .save(format!("data/dither/diagonal-tiles-16x16{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), BouncingBowtie(2)))
            .save(format!("data/dither/bouncing-bowtie-2x2{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), BouncingBowtie(4)))
            .save(format!("data/dither/bouncing-bowtie-4x4{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), BouncingBowtie(8)))
            .save(format!("data/dither/bouncing-bowtie-8x8{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), BouncingBowtie(16)))
            .save(format!("data/dither/bouncing-bowtie-16x16{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), ScanLine(2, dither::ordered::Orientation::Horizontal)))
            .save(format!("data/dither/scanline-horizontal-2x2{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), ScanLine(4, dither::ordered::Orientation::Horizontal)))
            .save(format!("data/dither/scanline-horizontal-4x4{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), ScanLine(8, dither::ordered::Orientation::Horizontal)))
            .save(format!("data/dither/scanline-horizontal-8x8{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), ScanLine(16, dither::ordered::Orientation::Horizontal)))
            .save(format!("data/dither/scanline-horizontal-16x16{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), ScanLine(2, dither::ordered::Orientation::Vertical)))
            .save(format!("data/dither/scanline-vertical-2x2{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), ScanLine(4, dither::ordered::Orientation::Vertical)))
            .save(format!("data/dither/scanline-vertical-4x4{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), ScanLine(8, dither::ordered::Orientation::Vertical)))
            .save(format!("data/dither/scanline-vertical-8x8{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), ScanLine(16, dither::ordered::Orientation::Vertical)))
            .save(format!("data/dither/scanline-vertical-16x16{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), Starburst(2)))
            .save(format!("data/dither/starburst-2x2{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), Starburst(4)))
            .save(format!("data/dither/starburst-4x4{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), Starburst(8)))
            .save(format!("data/dither/starburst-8x8{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), Starburst(16)))
            .save(format!("data/dither/starburst-16x16{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), ShinyBowtie(2)))
            .save(format!("data/dither/shiny-bowtie-2x2{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), ShinyBowtie(4)))
            .save(format!("data/dither/shiny-bowtie-4x4{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), ShinyBowtie(8)))
            .save(format!("data/dither/shiny-bowtie-8x8{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), ShinyBowtie(16)))
            .save(format!("data/dither/shiny-bowtie-16x16{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), MarbleTile(2)))
            .save(format!("data/dither/marble-tile-2x2{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), MarbleTile(4)))
            .save(format!("data/dither/marble-tile-4x4{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), MarbleTile(8)))
            .save(format!("data/dither/marble-tile-8x8{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), MarbleTile(16)))
            .save(format!("data/dither/marble-tile-16x16{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), CurvePath { n: 2, amplitude: 0.3, promotion: 0.005, halt_threshold: 100 }))
            .save(format!("data/dither/curve-path-2x2{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), CurvePath { n: 4, amplitude: 0.3, promotion: 0.005, halt_threshold: 100 }))
            .save(format!("data/dither/curve-path-4x4{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), CurvePath { n: 8, amplitude: 0.3, promotion: 0.005, halt_threshold: 100 }))
            .save(format!("data/dither/curve-path-8x8{}.png", postfix))?;
        image.clone().apply(&Ordered::new(palette.clone(), CurvePath { n: 16, amplitude: 0.3, promotion: 0.005, halt_threshold: 100 }))
            .save(format!("data/dither/curve-path-16x16{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), Stars))
            .save(format!("data/dither/stars{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), NewStars))
            .save(format!("data/dither/new-stars{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), Grid))
            .save(format!("data/dither/grid{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), Trail))
            .save(format!("data/dither/trail{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), Crisscross))
            .save(format!("data/dither/crisscross{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), Static))
            .save(format!("data/dither/static{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), Wavy(dither::ordered::Orientation::Vertical)))
            .save(format!("data/dither/wavy-vertical{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), Wavy(dither::ordered::Orientation::Horizontal)))
            .save(format!("data/dither/wavy-horizontal{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), BootlegBayer))
            .save(format!("data/dither/bootleg-bayer{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), Diagonals))
            .save(format!("data/dither/diagonals{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), DiagonalsBig))
            .save(format!("data/dither/diagonals-big{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), DiagonalsN {
             n: 8,
             direction: dither::ordered::DiagonalDirection::DownRight,
             increase: dither::ordered::Increase::Linear(1) 
        })).save(format!("data/dither/diagonals-n-dr+lin1-8x8{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), DiagonalsN {
             n: 32,
             direction: dither::ordered::DiagonalDirection::DownRight,
             increase: dither::ordered::Increase::Linear(1) 
        })).save(format!("data/dither/diagonals-n-dr+lin1-32x32{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), DiagonalsN {
             n: 8,
             direction: dither::ordered::DiagonalDirection::DownRight,
             increase: dither::ordered::Increase::Exponential(2) 
        })).save(format!("data/dither/diagonals-n-dr+exp2-8x8{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), DiagonalsN {
             n: 32,
             direction: dither::ordered::DiagonalDirection::DownRight,
             increase: dither::ordered::Increase::Exponential(2)
        })).save(format!("data/dither/diagonals-n-dr+exp2-32x32{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), DiamondGrid))
            .save(format!("data/dither/diamond-grid{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), SpeckleSquares))
            .save(format!("data/dither/speckle-squares{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), Scales))
            .save(format!("data/dither/scales{}.png", postfix))?;

        image.clone().apply(&Ordered::new(palette.clone(), TrailScales))
            .save(format!("data/dither/trail-scales{}.png", postfix))?;

        // Custom testing

        let n = 32;
        let amplitude = 0.4;
        let promotion = 0.01;
        let halt_threshold = 100;

        let mut matrix =  Array::<f64, _>::zeros((n, n));
        let mut visitor_m = vec![vec![false; n]; n];
        let curve_end = PI / 2.;
        let c_factor = curve_end / n as f64;

        let mut i = 0;
        let mut h = 0;

        while h < halt_threshold || (i % n != 0) {
            let h_wrap = i / n;
            let a = amplitude + (h_wrap as f64*promotion);
            let y_wrap_off = h_wrap as f64 * a;

            let y = (y_wrap_off as f64 + f64::sin(((i%n) as f64 * c_factor * a) % curve_end)) * n as f64;
            let x = i % n;

            let y = y.round() as usize % n;

            let point = matrix.get_mut((y, x)).unwrap();
            *point = *point + 1.;

            if visitor_m[y][x] {
                h = h + 1;
            } else {
                h = 0;
            }

            visitor_m[y][x] = true;
            i = i + 1;
        }

        let mut max = 0.0;
        for cell in matrix.iter() {
            if *cell > max {
                max = *cell;
            }
        }

        matrix = matrix / max;

        // println!("MATRIX:\n {matrix:#?}\nh = {h} - i = {i} - max = {max}");
        println!("h = {h} - i = {i} - max = {max}");

        image.clone().apply(&Ordered::new(palette.clone(), Custom(matrix)))
            .save(format!("data/dither/custom{}.png", postfix))?;

        Ok(())
    }

}
