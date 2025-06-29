use ndarray::{Array, Dim};
use palette::Srgb;

use crate::{
    dither::ordered::{
        algorithms::{
            bayer::dither_bayer,
            bouncing_bowtie::bouncing_bowtie,
            broken_spiral::generate_broken_spiral_matrix,
            checkered_diamonds::generate_checkered_diamonds,
            curve_path::generate_curve_path_matrix,
            diagonal_tiles::generate_diagonal_tiles,
            diagonals_n::generate_diagonals_n,
            diamonds::generate_diamonds,
            hardcoded::{
                get_bootleg_bayer, get_crisscross, get_diagonals, get_diagonals_big,
                get_diamond_grid, get_grid, get_new_stars, get_scales, get_speckle_squares,
                get_stars, get_static, get_trail, get_trail_scales, get_wavy,
            },
            marble_tile::generate_marble_tile,
            modulo_snake::generate_modulosnake,
            properties::{DiagonalDirection, Increase, Orientation, Wrapping},
            scanline::generate_scanline,
            shiny_bowtie::generate_shiny_bowtie,
            starburst::starburst,
            zigzag::generate_zigzag_matrix,
        },
        tools::{
            apply_ordered_matrix_to_image,
            blur::blur_matrix,
            checker::checker_matrix,
            exponentiate::exponentiate_matrix,
            mirror::MirrorLine,
            normalize_matrix,
            properties::{CheckerType, Rotation},
            rotate::rotate_matrix,
        },
    },
    effect::Effect,
    utils::image::RgbImageRepr,
};

pub mod algorithms;
pub mod tools;

/// Represents the _ordered_ method of dithering. Compared to error propagation, this method is less accurate - however it
/// results in a pattern that can be visually appealing.
///
/// In addition it only modifies each pixel on its own without needing to simultaneously touch/affect other pixels, making it
/// easily possible to parallellize.
#[derive(Debug, Clone)]
pub enum OrderedStrategy {
    Bayer(usize),
    Static,
    Crisscross,
    Trail,
    Grid,
    Stars,
    NewStars,
    CheckeredDiamonds(usize),
    Diamonds(usize),
    Wavy(Orientation),
    BootlegBayer,
    Diagonals,
    DiagonalsBig,
    DiagonalsN {
        n: usize,
        direction: DiagonalDirection,
        increase: Increase,
    },
    DiamondGrid,
    SpeckleSquares,
    Scales,
    TrailScales,
    DiagonalTiles(usize),
    BouncingBowtie(usize),
    ScanLine(usize, Orientation),
    Starburst(usize),
    ShinyBowtie(usize),
    MarbleTile(usize),
    CurvePath {
        n: usize,
        amplitude: f64,
        promotion: f64,
        halt_threshold: usize,
    },
    ZigZag {
        n: usize,
        halt_threshold: usize,
        wrapping: Wrapping,
        magnitude: (f64, f64),
        promotion: (f64, f64),
    },
    BrokenSpiral {
        n: usize,
        base_step: (f64, f64),
        oob_threshold: usize,
        increment_by: f64,
        increment_in: usize,
    },
    ModuloSnake {
        n: usize,
        increment_by: f64,
        modulo: usize,
        iterations: usize,
    },

    // modifiers
    Invert(Box<OrderedStrategy>),
    Mirror(Box<OrderedStrategy>, MirrorLine),
    Blur(Box<OrderedStrategy>, usize),
    Exponentiate(Box<OrderedStrategy>, f64),
    Rotate(Box<OrderedStrategy>, Rotation),
    Checker(Box<OrderedStrategy>, CheckerType),
    Custom(Array<f64, Dim<[usize; 2]>>),
}

impl OrderedStrategy {
    fn get_matrix(self) -> Array<f64, Dim<[usize; 2]>> {
        match self {
            Self::Bayer(size) => dither_bayer(size),
            Self::Diamonds(n) => generate_diamonds(n),
            Self::CheckeredDiamonds(n) => generate_checkered_diamonds(n),
            Self::Stars => get_stars().reversed_axes(),
            Self::NewStars => get_new_stars(),
            Self::Grid => get_grid(),
            Self::Trail => get_trail(),
            Self::Crisscross => get_crisscross(),
            Self::Static => get_static(),
            Self::Wavy(orientation) => get_wavy(orientation),
            Self::BootlegBayer => get_bootleg_bayer(),
            Self::Diagonals => get_diagonals(),
            Self::DiagonalsBig => get_diagonals_big(),
            Self::DiagonalsN {
                n,
                direction,
                increase,
            } => generate_diagonals_n(n, direction, increase),
            Self::DiamondGrid => get_diamond_grid(),
            Self::SpeckleSquares => get_speckle_squares(),
            Self::Scales => get_scales(),
            Self::TrailScales => get_trail_scales(),
            Self::DiagonalTiles(n) => generate_diagonal_tiles(n),
            Self::BouncingBowtie(n) => bouncing_bowtie(n),
            Self::ScanLine(n, orientation) => generate_scanline(n, orientation),
            Self::Starburst(n) => starburst(n),
            Self::ShinyBowtie(n) => generate_shiny_bowtie(n),
            Self::MarbleTile(n) => generate_marble_tile(n),
            Self::CurvePath {
                n,
                amplitude,
                promotion,
                halt_threshold,
            } => generate_curve_path_matrix(n, halt_threshold, amplitude, promotion),
            Self::ZigZag {
                n,
                halt_threshold,
                wrapping,
                magnitude,
                promotion,
            } => generate_zigzag_matrix(n, halt_threshold, wrapping, magnitude, promotion),
            Self::BrokenSpiral {
                n,
                base_step,
                oob_threshold,
                increment_by,
                increment_in,
            } => generate_broken_spiral_matrix(
                n,
                base_step,
                oob_threshold,
                increment_by,
                increment_in,
            ),
            Self::ModuloSnake {
                n,
                increment_by,
                modulo,
                iterations,
            } => generate_modulosnake(n, increment_by, modulo, iterations),
            Self::Invert(strategy) => 1.0 - &strategy.get_matrix(),
            Self::Mirror(strategy, mirrorline) => {
                let mut matrix = strategy.get_matrix().clone();
                mirrorline.mirror(&mut matrix);
                normalize_matrix(matrix)
            }
            Self::Blur(strategy, n) => {
                let mut matrix = strategy.get_matrix().clone();
                blur_matrix(&mut matrix, n);
                normalize_matrix(matrix)
            }
            Self::Exponentiate(strategy, factor) => {
                let mut matrix = strategy.get_matrix().clone();
                exponentiate_matrix(&mut matrix, factor);
                normalize_matrix(matrix)
            }
            Self::Rotate(strategy, rotation) => {
                let mut matrix = strategy.get_matrix();
                rotate_matrix(&mut matrix, rotation);
                normalize_matrix(matrix)
            }
            Self::Checker(strategy, checker_type) => {
                let mut matrix = strategy.get_matrix();
                checker_matrix(&mut matrix, checker_type);
                normalize_matrix(matrix)
            }
            Self::Custom(matrix) => matrix.clone(),
        }
    }

    pub fn invert(self) -> Self {
        Self::Invert(Box::new(self))
    }

    pub fn mirror(self, mirror_line: MirrorLine) -> Self {
        Self::Mirror(Box::new(self), mirror_line)
    }

    pub fn blur(self, blur_amnt: usize) -> Self {
        Self::Blur(Box::new(self), blur_amnt)
    }

    pub fn exponentiate(self, factor: f64) -> Self {
        Self::Exponentiate(Box::new(self), factor)
    }

    pub fn rotate(self, rotation: Rotation) -> Self {
        Self::Rotate(Box::new(self), rotation)
    }

    pub fn checker(self, checker_type: CheckerType) -> Self {
        Self::Checker(Box::new(self), checker_type)
    }
}

#[derive(Debug, Clone)]
pub struct Ordered {
    palette: Vec<Srgb>,
    strategy: OrderedStrategy,
}

impl Ordered {
    /// Creates a new `Bayer` ditherer with the given matrix size.
    pub fn new(palette: Vec<Srgb>, strategy: OrderedStrategy) -> Self {
        Self { palette, strategy }
    }

    fn dither_matrix(self) -> Array<f64, Dim<[usize; 2]>> {
        self.strategy.get_matrix()
    }
}

impl Effect<RgbImageRepr> for Ordered {
    fn affect(&self, image: RgbImageRepr) -> RgbImageRepr {
        let matrix = self.clone().dither_matrix();
        let matrix_size = matrix.dim().0;
        apply_ordered_matrix_to_image(image, matrix, matrix_size, &self.palette)
    }
}
