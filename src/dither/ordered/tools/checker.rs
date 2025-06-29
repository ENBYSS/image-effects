use std::ops::Div;

use crate::{
    dither::ordered::tools::{
        gen_n_size_matrix,
        properties::{CheckerType, Factor, Source},
        Matrix,
    },
    utils::numops::abs_i_mod,
};

pub fn checker_matrix(matrix: &mut Matrix, checker: CheckerType) {
    match checker {
        CheckerType::From {
            source,
            factor,
            modulo,
        } => checker_from(matrix, source, factor, modulo),
        CheckerType::Iter(f) => checker_iter(matrix, f),
    }
}

fn checker_iter(matrix: &mut Matrix, f: usize) {
    for (i, pix) in matrix.iter_mut().enumerate() {
        *pix = pix.div((f as f64).div(i as f64 % f as f64))
    }
}

fn checker_from(matrix: &mut Matrix, source: Source, factor: Factor, modulo: Option<usize>) {
    let (y, x) = source.get(matrix.dim().0);

    let (n, _) = matrix.dim();

    fn dist(from: (usize, usize), to: (usize, usize), modulo: Option<usize>) -> usize {
        let d = (from.0 as isize - to.0 as isize).unsigned_abs()
            + (from.1 as isize - to.1 as isize).unsigned_abs();

        if let Some(modulo) = modulo {
            d % modulo
        } else {
            d
        }
    }

    for a in 0..n {
        for b in 0..n {
            let d = dist((a, b), (y, x), modulo) as f64;

            let to_checker = matrix.get_mut((a, b)).unwrap();

            match factor {
                Factor::Exponential(factor) => *to_checker *= factor.powf(d),
                Factor::Linear => *to_checker *= 1.0 / (d + 1.),
            }
        }
    }
}
