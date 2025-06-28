use ndarray::{Array, Dim};
use palette::Srgb;

use crate::{colour::utils::quantize_rgb, utils::image::RgbImageRepr};

pub mod blur;
pub mod checker;
pub mod exponentiate;
pub mod mirror;
pub mod properties;
pub mod rotate;

pub type Matrix = Array<f64, Dim<[usize; 2]>>;

#[inline]
pub fn gen_n_size_matrix(n: usize) -> Matrix {
    Array::<f64, _>::zeros((n, n))
}

#[inline]
pub fn gen_n_size_visitor_matrix(n: usize) -> Vec<Vec<bool>> {
    vec![vec![false; n]; n]
}

pub fn normalize_matrix(matrix: Matrix) -> Matrix {
    let mut max = 0.0;
    for cell in matrix.iter() {
        if *cell > max {
            max = *cell;
        }
    }

    matrix / max
}

pub fn apply_ordered_matrix_to_image(
    mut image: RgbImageRepr,
    matrix: Array<f64, Dim<[usize; 2]>>,
    matrix_size: usize,
    palette: &[Srgb],
) -> RgbImageRepr {
    let ydim = image.len();
    let xdim = image.first().map(|row| row.len()).unwrap_or(0);

    for (x, rows) in image.iter_mut().enumerate().take(ydim) {
        for (y, cell) in rows.iter_mut().enumerate().take(xdim) {
            let mut color = Srgb::from(*cell).into_format::<f32>();

            let offset = (1.0 / 3.0)
                * (matrix
                    .get((x % matrix_size, y % matrix_size))
                    .unwrap_or(&0.0)
                    - 0.5) as f32;

            color.red += offset;
            color.blue += offset;
            color.green += offset;

            *cell = quantize_rgb(color, palette).into_format().into();
        }
    }

    image
}
