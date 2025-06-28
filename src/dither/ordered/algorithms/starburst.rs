use crate::dither::ordered::tools::{gen_n_size_matrix, Matrix};

pub fn starburst(n: usize) -> Matrix {
    let mut matrix = gen_n_size_matrix(n);

    for x in 0..n {
        for y in 0..n {
            let bigger_coord = x.min(n - x) * y.min(n - y);

            let point = matrix.get_mut((x, y)).unwrap();

            *point = bigger_coord as f64;
        }
    }

    matrix / ((n / 2).pow(2) as f64)
}
