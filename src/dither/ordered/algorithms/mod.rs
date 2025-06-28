use std::f64::consts::PI;

use ndarray::{concatenate, Array, Axis, Dim};

use crate::{
    dither::ordered::tools::{gen_n_size_matrix, gen_n_size_visitor_matrix, normalize_matrix},
    utils::numops::abs_f_mod,
};

pub mod bayer;
pub mod bouncing_bowtie;
pub mod broken_spiral;
pub mod checkered_diamonds;
pub mod curve_path;
pub mod diagonal_tiles;
pub mod diagonals_n;
pub mod diamonds;
pub mod hardcoded;
pub mod marble_tile;
pub mod modulo_snake;
pub mod properties;
pub mod scanline;
pub mod shiny_bowtie;
pub mod starburst;
pub mod zigzag;
