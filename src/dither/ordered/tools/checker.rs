use std::ops::Div;

use crate::{
    dither::ordered::tools::{gen_n_size_matrix, properties::Checker, Matrix},
    utils::numops::abs_i_mod,
};

pub fn checker_matrix(matrix: &mut Matrix, checker: Checker) {
    let (y, x) = matrix.dim();
    let mut result = gen_n_size_matrix(x);

    match checker {
        Checker::From((y, x)) => {}
        Checker::Iter(f) => {}
    }

    for a in 0..y as isize {
        for b in 0..x as isize {
            let mut sum = 0.0;

            for ai in -(blur_amnt as isize)..blur_amnt as isize {
                for bi in -(blur_amnt as isize)..blur_amnt as isize {
                    let blur_target = (abs_i_mod(a + ai, y), abs_i_mod(b + bi, x));
                    sum += matrix.get(blur_target).unwrap();
                }
            }

            let target = result.get_mut((a as usize, b as usize)).unwrap();
            *target = sum;
        }
    }
}

fn checker_iter(matrix: &mut Matrix, f: usize) {
    for (i, pix) in matrix.iter_mut().enumerate() {
        *pix = pix.div((f as f64).div(i as f64 % f as f64) as f64)
    }
}

fn checker_from(matrix: &mut Matrix, (y, x): (usize, usize)) {}
