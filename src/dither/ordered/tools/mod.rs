use ndarray::{Array, Dim};
use palette::Srgb;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelBridge,
    ParallelIterator,
};

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

    //      Performance counter stats for 'cargo test --release':

    //          24,182.73 msec task-clock:u                     #    1.626 CPUs utilized
    //                  0      context-switches:u               #    0.000 /sec
    //                  0      cpu-migrations:u                 #    0.000 /sec
    //            103,405      page-faults:u                    #    4.276 K/sec
    //    164,569,987,236      instructions:u                   #    1.89  insn per cycle
    //                                                          #    0.03  stalled cycles per insn
    //     86,987,525,609      cycles:u                         #    3.597 GHz
    //      4,812,809,355      stalled-cycles-frontend:u        #    5.53% frontend cycles idle
    //     13,425,734,897      branches:u                       #  555.179 M/sec
    //        114,572,758      branch-misses:u                  #    0.85% of all branches

    //       14.876598083 seconds time elapsed

    //       22.614836000 seconds user
    //        1.036618000 seconds sys
    image
        .iter_mut()
        .flat_map(|it| it.iter_mut())
        .enumerate()
        // .par_bridge()
        .for_each(|(idx, pixel)| {
            let mut color = Srgb::from(*pixel).into_format::<f32>();

            let y = idx / ydim;
            let x = idx / xdim;

            let offset = (1.0 / 3.0)
                * (matrix
                    .get((x % matrix_size, y % matrix_size))
                    .unwrap_or(&0.0)
                    - 0.5) as f32;

            color.red += offset;
            color.blue += offset;
            color.green += offset;

            *pixel = quantize_rgb(color, palette).into_format().into();
        });

    image
}
