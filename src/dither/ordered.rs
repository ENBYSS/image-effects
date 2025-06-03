use ndarray::{array, concatenate, Array, Axis, Dim};
use palette::Srgb;

use crate::{utils::image::RgbImageRepr, colour::utils::quantize_rgb, effect::Effect};

/// Represents the _ordered_ method of dithering. Compared to error propagation, this method is less accurate - however it
/// results in a pattern that can be visually appealing.
/// 
/// In addition it only modifies each pixel on its own without needing to simultaneously touch/affect other pixels, making it 
/// easily possible to parallellize.
pub struct Bayer {
    matrix_size: usize,
    palette: Vec<Srgb>,
}

impl Bayer {

    /// Creates a new `Bayer` ditherer with the given matrix size.
    pub fn new(matrix_size: usize, palette: Vec<Srgb>) -> Self {
        Self { matrix_size, palette }
    }

    /// Creates a clone of the ditherer with a different matrix size.
    pub fn with_matrix_size(&self, matrix_size: usize) -> Self {
        Self { matrix_size, palette: self.palette.clone() }
    }

    fn dither_matrix(n: usize) -> Array<f64, Dim<[usize; 2]>> {
        if n == 1 {
            return Array::<f64, _>::zeros((1, 1));
        }

        let nested_matrix = Self::dither_matrix(n / 2);
        let multiplier = n.pow(2) as f64;

        let first = multiplier * nested_matrix.clone();
        let second = multiplier * nested_matrix.clone() + 2.;
        let third = multiplier * nested_matrix.clone() + 3.;
        let fourth = multiplier * nested_matrix.clone() + 1.;

        let first_col = concatenate(Axis(0), &[first.view(), third.view()]).unwrap();
        let second_col = concatenate(Axis(0), &[second.view(), fourth.view()]).unwrap();

        (1. / multiplier) * concatenate(Axis(1), &[first_col.view(), second_col.view()]).unwrap()
    }
}

impl Effect<RgbImageRepr> for Bayer {
    fn affect(&self, mut image: RgbImageRepr) -> RgbImageRepr {
        let matrix = Self::dither_matrix(self.matrix_size);
        apply_ordered_matrix_to_image(image, matrix, self.matrix_size, &self.palette)
    }
}

pub struct Diamonds {
    matrix_size: usize,
    palette: Vec<Srgb>
}

impl Diamonds {

    /// Creates a new `Bayer` ditherer with the given matrix size.
    pub fn new(matrix_size: usize, palette: Vec<Srgb>) -> Self {
        Self { matrix_size, palette }
    }

        /// Creates a clone of the ditherer with a different matrix size.
    pub fn with_matrix_size(&self, matrix_size: usize) -> Self {
        Self { matrix_size, palette: self.palette.clone() }
    }

    fn dither_matrix(n: usize) -> Array<f64, Dim<[usize; 2]>> {
        let n_half = n / 2;
        
        let mut matrix =  Array::<f64, _>::zeros((n, n));
        let step_size = 1.0 / (n_half as f64); // from center to edge, plus one step

        for x in 0..n {
            for y in 0..n {
                let distance_x = x.abs_diff(n_half);
                let distance_y = y.abs_diff(n_half);

                // if distance_y > 0 { distance_y = distance_y - 1; }

                let factor = (distance_x + distance_y) as f64 * step_size;
                let point = matrix.get_mut((x, y)).unwrap();
                *point = (1.0 - factor as f64).abs();
            }
        }

        matrix
    }
}

impl Effect<RgbImageRepr> for Diamonds {
    fn affect(&self, mut image: RgbImageRepr) -> RgbImageRepr {
        let matrix = Self::dither_matrix(self.matrix_size);
        apply_ordered_matrix_to_image(image, matrix, self.matrix_size, &self.palette)
    }
}

pub struct Stars {
    palette: Vec<Srgb>
}

impl Stars {

    /// Creates a new `Bayer` ditherer with the given matrix size.
    pub fn new(palette: Vec<Srgb>) -> Self {
        Self { palette }
    }

    fn dither_matrix() -> Array<f64, Dim<[usize; 2]>> {
        array![
            [0.875, 0.625, 0.5, 0.375, 0.25, 0.125, 0.25, 0.125, 0.25, 0.375, 0.5, 0.625],
            [0.75, 0.5, 0.375, 0.125, 0.0, 0.125, 0.375, 0.125, 0.0, 0.125, 0.375, 0.5],
            [0.625, 0.375, 0.25, 0.0, 0.125, 0.25, 0.5, 0.25, 0.125, 0.0, 0.25, 0.375],
            [0.5, 0.25, 0.125, 0.0, 0.25, 0.375, 0.625, 0.375, 0.25, 0.0, 0.125, 0.25],
            [0.375, 0.125, 0.0, 0.125, 0.375, 0.5, 0.75, 0.5, 0.375, 0.125, 0.0, 0.125],
            [0.25, 0.125, 0.25, 0.375, 0.5, 0.625, 0.875, 0.625, 0.5, 0.375, 0.25, 0.125],
            [0.125, 0.25, 0.375, 0.5, 0.625, 0.875, 1.0, 0.875, 0.625, 0.5, 0.375, 0.25],
            [0.25, 0.125, 0.25, 0.375, 0.5, 0.625, 0.875, 0.625, 0.5, 0.375, 0.25, 0.125],
            [0.375, 0.125, 0.0, 0.125, 0.375, 0.5, 0.75, 0.5, 0.375, 0.125, 0.0, 0.125],
            [0.5, 0.25, 0.125, 0.0, 0.25, 0.375, 0.625, 0.375, 0.25, 0.0, 0.125, 0.25],
            [0.625, 0.375, 0.25, 0.0, 0.125, 0.25, 0.5, 0.25, 0.125, 0.0, 0.25, 0.375],
            [0.75, 0.5, 0.375, 0.125, 0.0, 0.125, 0.375, 0.125, 0.0, 0.125, 0.375, 0.5]
        ].reversed_axes()
    }
}

impl Effect<RgbImageRepr> for Stars {
    fn affect(&self, mut image: RgbImageRepr) -> RgbImageRepr {
        let matrix = Self::dither_matrix();
        apply_ordered_matrix_to_image(image, matrix, 12, &self.palette)
    }
}

pub struct CheckeredDiamonds {
    matrix_size: usize,
    palette: Vec<Srgb>
}

impl CheckeredDiamonds {

    /// Creates a new `Bayer` ditherer with the given matrix size.
    pub fn new(matrix_size: usize, palette: Vec<Srgb>) -> Self {
        Self { matrix_size, palette }
    }

        /// Creates a clone of the ditherer with a different matrix size.
    pub fn with_matrix_size(&self, matrix_size: usize) -> Self {
        Self { matrix_size, palette: self.palette.clone() }
    }

    fn dither_matrix(n: usize) -> Array<f64, Dim<[usize; 2]>> {
        let n_half = n / 2;
        
        let mut matrix =  Array::<f64, _>::zeros((n, n));
        let step_size = 1.0 / (n_half as f64); // from center to edge, plus one step

        for x in 0..n {
            for y in 0..n {
                let distance_x = x.abs_diff(n_half);
                let distance_y = y.abs_diff(n_half);

                // if distance_y > 0 { distance_y = distance_y - 1; }

                let factor = (distance_x + distance_y) as f64 * step_size;
                let point = matrix.get_mut((x, y)).unwrap();
                
                let lum = (1.0 - factor as f64).abs();

                if x % 2 == 0 && y % 2 == 0 {
                    *point = lum.round();
                } else {
                    *point = (1.0 - factor as f64).abs();
                }
            }
        }

        matrix
    }
}

impl Effect<RgbImageRepr> for CheckeredDiamonds {
    fn affect(&self, mut image: RgbImageRepr) -> RgbImageRepr {
        let matrix = Self::dither_matrix(self.matrix_size);
        apply_ordered_matrix_to_image(image, matrix, self.matrix_size, &self.palette)
    }
}

pub struct NewStars {
    palette: Vec<Srgb>
}

impl NewStars {

    /// Creates a new `Bayer` ditherer with the given matrix size.
    pub fn new(palette: Vec<Srgb>) -> Self {
        Self { palette }
    }

    fn dither_matrix() -> Array<f64, Dim<[usize; 2]>> {
        let stars_arr = array![
            [4., 3., 4., 3., 3., 4., 4., 5., 5., 4., 4., 4.],
            [3., 4., 2., 1., 2., 5., 5., 2., 1., 2., 0., 4.],
            [4., 1., 0., 0., 6., 3., 0., 6., 0., 1., 2., 3.],
            [4., 2., 1., 6., 2., 0., 5., 0., 6., 6., 0., 3.],
            [5., 1., 0., 6., 4., 3., 0., 7., 5., 4., 6., 4.],
            [5., 2., 1., 2., 7., 5., 8., 6., 4., 3., 4., 5.],
            [4., 5., 2., 0., 1., 8., 9., 8., 6., 2., 3., 5.],
            [3., 5., 3., 2., 1., 0., 8., 0., 7., 4., 2., 4.],
            [3., 0., 6., 3., 2., 7., 3., 2., 0., 6., 2., 3.],
            [4., 2., 1., 6., 6., 3., 2., 1., 0., 6., 0., 3.],
            [3., 4., 2., 0., 1., 2., 1., 0., 6., 1., 2., 4.],
            [4., 3., 4., 4., 3., 4., 5., 5., 4., 3., 3., 4.],
        ].reversed_axes();

            //         [0, 0, 4, 0, 0, 0, 0, 5, 5, 4, 4, 0],
            // [0, 4, 0, 0, 0, 5, 5, 0, 0, 0, 0, 4],
            // [4, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0],
            // [4, 0, 0, 6, 0, 0, 0, 0, 6, 6, 0, 0],
            // [5, 0, 0, 6, 0, 0, 0, 7, 0, 0, 6, 0],
            // [5, 0, 0, 0, 7, 0, 8, 0, 0, 0, 0, 5],
            // [0, 5, 0, 0, 0, 8, 9, 8, 0, 0, 0, 5],
            // [0, 5, 0, 0, 0, 0, 8, 0, 7, 0, 0, 0],
            // [0, 0, 6, 0, 0, 7, 0, 0, 0, 6, 0, 0],
            // [4, 0, 0, 6, 6, 0, 0, 0, 0, 6, 0, 0],
            // [0, 4, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0],
            // [0, 0, 4, 4, 0, 0, 5, 5, 0, 0, 0, 4],
        stars_arr / 9.
    }
}

impl Effect<RgbImageRepr> for NewStars {
    fn affect(&self, mut image: RgbImageRepr) -> RgbImageRepr {
        let matrix = Self::dither_matrix();
        apply_ordered_matrix_to_image(image, matrix, 12, &self.palette)
    }
}

pub fn apply_ordered_matrix_to_image(mut image: RgbImageRepr, matrix: Array<f64, Dim<[usize; 2]>>, matrix_size: usize, palette: &Vec<Srgb>) -> RgbImageRepr {
    let ydim = image.len();
    let xdim = image.get(0).map(|row| row.len()).unwrap_or(0);

    for y in 0..ydim {
        for x in 0..xdim {
            let mut color = Srgb::from(image[y][x]).into_format::<f32>();
    
            let offset = (1.0 / 3.0)
                * (matrix
                    .get((x % matrix_size, y % matrix_size))
                    .unwrap_or(&0.0)
                    - 0.5) as f32;
    
            color.red = color.red + offset;
            color.blue = color.blue + offset;
            color.green = color.green + offset;
    
            image[y][x] = quantize_rgb(color, palette).into_format().into();
        }
    }

    image
}