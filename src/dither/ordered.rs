use ndarray::{array, concatenate, Array, Axis, Dim};
use palette::Srgb;

use crate::{utils::image::RgbImageRepr, colour::utils::quantize_rgb, effect::Effect};

/// Represents the _ordered_ method of dithering. Compared to error propagation, this method is less accurate - however it
/// results in a pattern that can be visually appealing.
/// 
/// In addition it only modifies each pixel on its own without needing to simultaneously touch/affect other pixels, making it 
/// easily possible to parallellize.
pub enum OrderedStrategy {
    Bayer(u8),
    Static,
    Crisscross,
    Trail,
    Grid,
    Stars,
    NewStars,
    CheckeredDiamonds(u8),
    Diamonds(u8),
    Wavy,
    BootlegBayer,
    Diagonals,
    DiagonalsBig,
    DiamondGrid,
    SpeckleSquares,
    Scales,
    TrailScales,
}

impl OrderedStrategy {
    fn get_matrix(&self) -> Array<f64, Dim<[usize; 2]>> {
        match self {
            Self::Bayer(size) => {
                fn dither_bayer(n: u8) -> Array<f64, Dim<[usize; 2]>> {
                    if n == 1 {
                        return Array::<f64, _>::zeros((1, 1));
                    }

                    let nested_matrix = dither_bayer(n / 2);
                    let multiplier = n.pow(2) as f64;

                    let first = multiplier * nested_matrix.clone();
                    let second = multiplier * nested_matrix.clone() + 2.;
                    let third = multiplier * nested_matrix.clone() + 3.;
                    let fourth = multiplier * nested_matrix.clone() + 1.;

                    let first_col = concatenate(Axis(0), &[first.view(), third.view()]).unwrap();
                    let second_col = concatenate(Axis(0), &[second.view(), fourth.view()]).unwrap();

                    (1. / multiplier) * concatenate(Axis(1), &[first_col.view(), second_col.view()]).unwrap()
                }

                dither_bayer(*size)
            },
            Self::Diamonds(n) => {
                let n = *n;
                let n_half = n / 2;
        
                let mut matrix =  Array::<f64, _>::zeros((n as usize, n as usize));
                let step_size = 1.0 / (n_half as f64); // from center to edge, plus one step

                for x in 0..n {
                    for y in 0..n {
                        let distance_x = x.abs_diff(n_half);
                        let distance_y = y.abs_diff(n_half);

                        // if distance_y > 0 { distance_y = distance_y - 1; }

                        let factor = (distance_x + distance_y) as f64 * step_size;
                        let point = matrix.get_mut((x as usize, y as usize)).unwrap();
                        *point = (1.0 - factor as f64).abs();
                    }
                }

                matrix
            },
            Self::CheckeredDiamonds(n) => {
                let n = *n;
                let n_half = n / 2;
        
                let mut matrix =  Array::<f64, _>::zeros((n as usize, n as usize));
                let step_size = 1.0 / (n_half as f64); // from center to edge, plus one step

                for x in 0..n {
                    for y in 0..n {
                        let distance_x = x.abs_diff(n_half);
                        let distance_y = y.abs_diff(n_half);

                        // if distance_y > 0 { distance_y = distance_y - 1; }

                        let factor = (distance_x + distance_y) as f64 * step_size;
                        let point = matrix.get_mut((x as usize, y as usize)).unwrap();
                        
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
            Self::Stars => {
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
            },
            Self::NewStars => {
                let stars_arr = array![
                    [4., 5., 3., 2., 1., 0., 1., 2., 3., 5.],
                    [3., 2., 1., 0., 0., 1., 0., 1., 2., 5.],
                    [2., 1., 0., 0., 1., 2., 0., 0., 1., 3.],
                    [1., 0., 0., 1., 2., 3., 1., 0., 0., 2.],
                    [0., 1., 2., 3., 4., 4., 2., 1., 0., 1.],
                    [1., 0., 1., 2., 4., 5., 3., 2., 1., 0.],
                    [2., 0., 0., 1., 3., 2., 1., 0., 0., 1.],
                    [3., 1., 0., 0., 2., 1., 0., 0., 1., 2.],
                    [4., 2., 1., 0., 1., 0., 0., 1., 2., 3.],
                    [5., 3., 2., 1., 0., 1., 2., 3., 5., 5.],
                ].reversed_axes();
                stars_arr / 5.
            },
            Self::Grid => {
                let stars_arr = array![
                    [0., 1., 0., 1., 0., 1., 0., 1., 0., 1.],
                    [1., 2., 3., 2., 3., 2., 3., 2., 3., 0.],
                    [0., 3., 4., 5., 4., 5., 4., 5., 2., 1.],
                    [1., 2., 5., 6., 7., 6., 7., 4., 3., 0.],
                    [0., 3., 4., 7., 8., 9., 6., 5., 2., 1.],
                    [1., 2., 5., 6., 9., 8., 7., 4., 3., 0.],
                    [0., 3., 4., 7., 6., 7., 6., 5., 2., 1.],
                    [1., 2., 5., 4., 5., 4., 5., 4., 3., 0.],
                    [0., 3., 2., 3., 2., 3., 2., 3., 2., 1.],
                    [1., 0., 1., 0., 1., 0., 1., 0., 1., 0.],
                ].reversed_axes();
                stars_arr / 9.
            },
            Self::Trail => {
                let stars_arr = array![
                    [9., 8., 7., 6., 5., 4., 3., 2., 1., 0.],
                    [8., 9., 8., 7., 6., 5., 4., 3., 2., 1.],
                    [7., 8., 9., 8., 7., 6., 5., 4., 3., 2.],
                    [6., 7., 8., 9., 8., 7., 6., 5., 4., 3.],
                    [5., 6., 7., 8., 9., 8., 7., 6., 5., 4.],
                    [4., 5., 6., 7., 8., 9., 8., 7., 6., 5.],
                    [3., 4., 5., 6., 7., 8., 9., 8., 7., 6.],
                    [2., 3., 4., 5., 6., 7., 8., 9., 8., 7.],
                    [1., 2., 3., 4., 5., 6., 7., 8., 9., 8.],
                    [0., 1., 2., 3., 4., 5., 6., 7., 8., 9.],
                ].reversed_axes();
                stars_arr / 9.
            },
            Self::Crisscross => {
                let stars_arr = array![
                    [8., 8., 8., 8., 8., 9., 8., 7., 6., 5., 4.],
                    [7., 7., 7., 7., 8., 9., 8., 7., 6., 5., 5.],
                    [6., 6., 6., 7., 8., 9., 8., 7., 6., 6., 6.],
                    [5., 5., 6., 7., 8., 9., 8., 7., 7., 7., 7.],
                    [4., 5., 6., 7., 8., 9., 8., 8., 8., 8., 8.],
                    [9., 9., 9., 9., 9., 9., 9., 9., 9., 9., 9.],
                    [8., 8., 8., 8., 8., 9., 8., 7., 6., 5., 4.],
                    [7., 7., 7., 7., 8., 9., 8., 7., 6., 5., 5.],
                    [6., 6., 6., 7., 8., 9., 8., 7., 6., 6., 6.],
                    [5., 5., 6., 7., 8., 9., 8., 7., 7., 7., 7.],
                    [4., 5., 6., 7., 8., 9., 8., 8., 8., 8., 8.],
                ].reversed_axes();
                (stars_arr-4.) / 5.
            },
            Self::Static => {
                let stars_arr = array![
                    [2., 1., 3., 4., 5., 9., 7., 8., 0., 6.],
                    [6., 3., 2., 8., 9., 5., 4., 1., 7., 0.],
                    [5., 9., 4., 3., 6., 0., 7., 2., 1., 8.],
                    [8., 0., 2., 1., 5., 6., 3., 7., 4., 9.],
                    [0., 2., 9., 6., 1., 4., 5., 8., 7., 3.],
                    [4., 1., 5., 8., 3., 2., 0., 7., 8., 6.],
                    [3., 6., 0., 2., 8., 5., 9., 1., 7., 4.],
                    [9., 8., 6., 0., 4., 1., 2., 3., 5., 7.],
                    [7., 4., 2., 9., 3., 6., 8., 0., 5., 1.],
                ].reversed_axes();
                stars_arr / 9.
            },
            Self::Wavy => {
                let stars_arr = array![
                    [1.0, 1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0],
                    [34.0, 21.0, 13.0, 8.0, 5.0, 3.0, 2.0, 1.0, 1.0],
                    [1.0, 1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0],
                    [34.0, 21.0, 13.0, 8.0, 5.0, 3.0, 2.0, 1.0, 1.0],
                    [1.0, 1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0],
                    [34.0, 21.0, 13.0, 8.0, 5.0, 3.0, 2.0, 1.0, 1.0],
                    [1.0, 1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0],
                    [34.0, 21.0, 13.0, 8.0, 5.0, 3.0, 2.0, 1.0, 1.0],
                    [1.0, 1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0, 34.0],
                ].reversed_axes();
                stars_arr / 34.
            },
            Self::BootlegBayer => {
                let stars_arr = array![
                    [1., 7., 3.],
                    [8., 2., 9.],
                    [5., 6., 4.],
                ].reversed_axes();
                stars_arr / 9.
            },
            Self::Diagonals => {
                let stars_arr = array![
                    [10., 5., 1.,],
                    [1., 10., 5.,],
                    [5., 1., 10.,],
                ].reversed_axes();
                stars_arr / 10.
            },
            Self::DiagonalsBig => {
                let stars_arr = array![
                    [256., 128., 64., 32., 16., 8., 4., 2., 1.],
                    [1., 256., 128., 64., 32., 16., 8., 4., 2.],
                    [2., 1., 256., 128., 64., 32., 16., 8., 4.],
                    [4., 2., 1., 256., 128., 64., 32., 16., 8.],
                    [8., 4., 2., 1., 256., 128., 64., 32., 16.],
                    [16., 8., 4., 2., 1., 256., 128., 64., 32.],
                    [32., 16., 8., 4., 2., 1., 256., 128., 64.],
                    [64., 32., 16., 8., 4., 2., 1., 256., 128.],
                    [128., 64., 32., 16., 8., 4., 2., 1., 256.],
                ].reversed_axes();
                stars_arr / 256.
            },
            Self::DiamondGrid => {
                let stars_arr = array![
                    [10., 6., 4., 6., 10.],
                    [8., 10., 6., 10., 8.],
                    [2., 8., 10., 8., 2.],
                    [8., 10., 6., 10., 8.],
                    [10., 6., 4., 6., 10.],
                ].reversed_axes();
                (stars_arr-2.) / 8.
            },
            Self::SpeckleSquares => {
                let stars_arr = array![
                    [1., 3., 1.],
                    [2., 3., 2.],
                    [1., 3., 1.],
                ].reversed_axes();
                stars_arr / 3.
            },
            Self::Scales => {
                let stars_arr = array![
                    [9., 8., 7., 6., 5., 4., 3., 2., 1.],
                    [8., 9., 8., 7., 6., 5., 4., 3., 2.],
                    [7., 8., 9., 8., 7., 6., 5., 4., 3.],
                    [6., 7., 8., 9., 8., 7., 6., 5., 4.],
                    [5., 6., 7., 8., 9., 8., 7., 6., 5.],
                    [4., 5., 6., 7., 8., 9., 8., 7., 6.],
                    [3., 4., 5., 6., 7., 8., 9., 8., 7.],
                    [2., 3., 4., 5., 6., 7., 8., 9., 8.],
                    [1., 2., 3., 4., 5., 6., 7., 8., 9.],
                ].reversed_axes();
                stars_arr / 9.
            },
            Self::TrailScales => {
                let stars_arr = array![
                    [0., 8., 7., 6., 5., 4., 3., 2., 1., 0.],
                    [8., 5., 8., 7., 6., 5., 4., 3., 5., 8.],
                    [7., 8., 6., 8., 7., 6., 5., 4., 3., 7.],
                    [6., 7., 8., 9., 8., 7., 6., 5., 4., 6.],
                    [5., 6., 7., 8., 9., 8., 7., 6., 5., 5.],
                    [4., 5., 6., 7., 8., 9., 8., 7., 6., 4.],
                    [3., 4., 5., 6., 7., 8., 9., 8., 7., 3.],
                    [2., 3., 4., 5., 6., 7., 8., 3., 8., 2.],
                    [1., 5., 3., 4., 5., 6., 6., 5., 3., 1.],
                    [0., 8., 7., 6., 5., 4., 3., 2., 1., 0.],
                ].reversed_axes();
                stars_arr / 9.
            },
        }
    }
}

pub struct Ordered {
    palette: Vec<Srgb>,
    strategy: OrderedStrategy,
}

impl Ordered {

    /// Creates a new `Bayer` ditherer with the given matrix size.
    pub fn new(palette: Vec<Srgb>, strategy: OrderedStrategy) -> Self {
        Self { palette, strategy }
    }


    fn dither_matrix(&self) -> Array<f64, Dim<[usize; 2]>> {
        self.strategy.get_matrix()
    }
}

impl Effect<RgbImageRepr> for Ordered {
    fn affect(&self, mut image: RgbImageRepr) -> RgbImageRepr {
        let matrix = self.dither_matrix();
        let matrix_size = matrix.dim().0;
        apply_ordered_matrix_to_image(image, matrix, matrix_size, &self.palette)
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