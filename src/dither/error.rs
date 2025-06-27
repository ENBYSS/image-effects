use std::marker::PhantomData;


use palette::Srgb;

use crate::{
    utils::{image::{get_dimensions_of_matrix, RgbImageRepr}},
    colour::utils::{quantize_rgb, compute_rgb_error}, effect::Effect,
};

/// Every `ErrorPropagator` starts with a state of `Base`.
/// 
/// Base `ErrorPropagator`s _cannot_ be used as an `Effect`,
/// as they require a palette. This can be done with `.with_palette`.
pub struct Base;

/// Once an `ErrorPropagator` acquires a colour palette, it enters the `WithPalette` state.
/// 
/// With this state, it can now be used as an effect.
pub struct WithPalette;

mod private {
    pub trait Sealed {}

    impl Sealed for super::Base {}
    impl Sealed for super::WithPalette {}
}

/// Note: This trait is *sealed* and should not be used externally.
/// 
/// It is used specifically as a type-state for an `ErrorPropagator`.
pub trait PropagatorState: private::Sealed {}
impl PropagatorState for Base {}
impl PropagatorState for WithPalette {}

/// This struct defines an error propagation algorithm. For existing algorithms, the constants should be used instead.
/// 
/// An `ErrorPropagator` doesn't start out as an effect, as it requires a colour palette to actually perform the dithering.
/// 
/// This can be done by simply calling `.with_palette`, which will generate a configured version of the propagator.
pub struct ErrorPropagator<'name, 'matrix, S: PropagatorState> {
    /// The name of the algorithm in question.
    pub name: &'name str,

    /// The error propagation matrix, in the form of (dx, dy, portion).
    /// 
    /// For example, (1, 0, 1) will propagate `1/portion` of the error
    /// to the next pixel on the right.
    pub matrix: &'matrix [(i8, i8, u8)],

    /// The total amount of portions. Note that this doesn't need to 
    /// equal the sum of portions in the `matrix`. 
    /// 
    /// For example, Atkinson doesn't distribute all portions - and 
    /// one could theoretically _over-propagate_ by distributing more
    /// portions in the `matrix` than listed here.
    pub portions: u16,

    /// The colour palette that the error propagator has been configured with.
    /// Required to function as an effect.
    palette: Option<Vec<Srgb>>,

    /// Phantom data to own the state.
    _phantom: PhantomData<S>,
}

impl<'a, 'b> ErrorPropagator<'a, 'b, Base> {
    pub const fn new(name: &'a str, matrix: &'b [(i8, i8, u8)], portions: u16) -> Self {
        ErrorPropagator {
            name,
            matrix,
            portions,
            palette: None,
            _phantom: PhantomData,
        }
    }
}

impl<'a, 'b, S: PropagatorState> ErrorPropagator<'a, 'b, S> {
    pub fn with_palette(&self, palette: Vec<Srgb>) -> ErrorPropagator<'a, 'b, WithPalette> {
        ErrorPropagator {
            name: self.name,
            matrix: self.matrix,
            portions: self.portions,
            palette: Some(palette),
            _phantom: PhantomData,
        }
    }
}

impl<'a, 'b> Effect<RgbImageRepr> for ErrorPropagator<'a, 'b, WithPalette> {
    fn affect(&self, mut image: RgbImageRepr) -> RgbImageRepr {
        let (xdim, ydim) = get_dimensions_of_matrix(&image);

        if xdim == 0 || ydim == 0 {
            return image;
        }

        for y in 0..ydim {
            for x in 0..xdim {
                let error = {
                    let rgb = Srgb::from(image[y][x]).into_format::<f32>();
                    let quantized = quantize_rgb(rgb, self.palette.as_ref().unwrap());
                    image[y][x] = quantized.into_format().into();
                    compute_rgb_error(rgb, quantized)
                };

                let error = (
                    error.0 * 255.0,
                    error.1 * 255.0,
                    error.2 * 255.0,
                );

                for (x_off, y_off, portion) in self.matrix.iter() {
                    let (x_err, y_err) = (
                        (x as i64 + *x_off as i64) as usize,
                        (y as i64 + *y_off as i64) as usize,
                    );

                    let pixel = image
                        .get_mut(y_err)
                        .and_then(|row| row.get_mut(x_err));

                    if let Some(pixel) = pixel {
                        *pixel = [
                            (pixel[0] as f32 + (error.0 * *portion as f32) / self.portions as f32).clamp(0.0, 255.0) as u8,
                            (pixel[1] as f32 + (error.1 * *portion as f32) / self.portions as f32).clamp(0.0, 255.0) as u8,
                            (pixel[2] as f32 + (error.2 * *portion as f32) / self.portions as f32).clamp(0.0, 255.0) as u8,
                        ]
                    }
                }
            }
        }
        image
    }
}

type ConstErrorPropagator = ErrorPropagator<'static, 'static, Base>;

/// The Floyd-Steinberg error propagation method.
/// 
/// Distributes the entire error.
/// 
/// ```notrust
/// - x 7
/// 5 3 1
/// ```
pub const FLOYD_STEINBERG: ConstErrorPropagator = ErrorPropagator::new(
    "floyd-steinberg",
    &[
                            (1, 0, 7),
       (-1, 1, 5),(0, 1, 3),(1, 1, 1),
    ],
    16,
);

/// The Jarvis-Judice-Ninke error propagation method.
/// 
/// Distributes the entire error.
/// 
/// ```notrust
/// - - x 7 5
/// 3 5 7 5 3
/// 1 3 5 3 1
/// ```
pub const JARVIS_JUDICE_NINKE: ConstErrorPropagator = ErrorPropagator::new(
    "jarvis-judice-ninke",
    &[
                                        (1, 0, 7),(2, 0, 5),
        (-2, 1, 3),(-1, 1, 5),(0, 1, 7),(1, 1, 5),(2, 1, 3),
        (-2, 2, 1),(-1, 2, 3),(0, 2, 5),(1, 2, 3),(2, 2, 1),
    ],
    48,
);

/// The Atkinson error propagation method.
/// 
/// Doesn't distribute the entire error - only 6/8ths of it.
/// 
/// ```notrust
/// - - x 1 1
/// - 1 1 1 -
/// - - 1 - -
/// ```
pub const ATKINSON: ConstErrorPropagator = ErrorPropagator::new(
    "atkinson",
    &[
                             (1, 0, 1),(2, 0, 1),
        (-1, 1, 1),(0, 1, 1),(1, 1, 1),
                   (0, 2, 1)
    ],
    8,
);

/// The Burkes error propagation method.
/// 
/// Distributes the entire error.
/// 
/// ```notrust
/// - - x 8 4
/// 2 4 8 4 2
/// ```
pub const BURKES: ConstErrorPropagator = ErrorPropagator::new(
    "burkes",
    &[
                                           (1, 0, 8), (2, 0, 4),
        (-2, 1, 2), (-1, 1, 4), (0, 1, 8), (1, 1, 4), (2, 1, 2),
    ],
    32,
);

/// The Stucki error propagation method.
/// 
/// Distributes the entire error.
/// 
/// ```notrust
/// - - x 8 4
/// 2 4 8 4 2
/// 1 2 4 2 1
/// ```
pub const STUCKI: ConstErrorPropagator = ErrorPropagator::new(
    "stucki",
    &[
                                        (1, 0, 8),(2, 0, 4),
        (-2, 1, 2),(-1, 1, 4),(0, 1, 8),(1, 1, 4),(2, 1, 2),
        (-2, 2, 1),(-1, 2, 2),(0, 2, 4),(1, 2, 2),(2, 2, 1),
    ],
    42,
);

/// The Sierra error propagation method.
/// 
/// Distributes the entire error.
/// 
/// ```notrust
/// - - x 5 3
/// 2 4 5 4 2
/// - 2 3 2 -
/// ```
pub const SIERRA: ConstErrorPropagator = ErrorPropagator::new(
    "sierra",
    &[
                                        (1, 0, 5),(2, 0, 3),
        (-2, 1, 2),(-1, 1, 4),(0, 1, 5),(1, 1, 4),(2, 1, 2),
                   (-1, 2, 2),(0, 2, 3),(1, 2, 2),
    ],
    32,
);

/// The Sierra (Two Row) error propagation method. A variant of Sierra.
/// 
/// Distributes the entire error.
/// 
/// ```notrust
/// - - x 4 3
/// 1 2 3 2 1
/// ```
pub const SIERRA_TWO_ROW: ConstErrorPropagator = ErrorPropagator::new(
    "sierra-two-row",
    &[
                                        (1, 0, 4),(2, 0, 3),
        (-2, 1, 1),(-1, 1, 2),(0, 1, 3),(1, 1, 2),(2, 1, 1),
    ],
    16,
);

/// The Sierra (lite) error propagation method. A variant of Sierra.
/// 
/// Distributes the entire error.
/// 
/// ```notrust
/// - x 2
/// 1 1 -
/// ```
pub const SIERRA_LITE: ConstErrorPropagator = ErrorPropagator::new(
    "sierra-lite",
    &[
                            (1, 0, 2),
        (-1, 1, 1),(0, 1, 1)
    ],
    4,
);