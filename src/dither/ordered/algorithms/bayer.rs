use ndarray::{concatenate, Axis};

use crate::dither::ordered::tools::{gen_n_size_matrix, Matrix};

pub fn dither_bayer(n: usize) -> Matrix {
    if n == 1 {
        return gen_n_size_matrix(n);
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
