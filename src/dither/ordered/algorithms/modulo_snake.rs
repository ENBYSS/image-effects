use crate::dither::ordered::tools::{gen_n_size_matrix, normalize_matrix, Matrix};

pub fn generate_modulosnake(
    n: usize,
    increment_by: f64,
    modulo: usize,
    iterations: usize,
) -> Matrix {
    let mut matrix = gen_n_size_matrix(n);

    for _ in 0..iterations {
        for (n, pixel) in matrix.iter_mut().enumerate() {
            *pixel = ((n as f64 * increment_by) as usize % modulo) as f64;
        }
    }

    normalize_matrix(matrix)
}
