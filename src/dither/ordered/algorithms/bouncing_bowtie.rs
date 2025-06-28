use crate::dither::ordered::tools::{gen_n_size_matrix, Matrix};

pub fn bouncing_bowtie(n: usize) -> Matrix {
    let mut matrix = gen_n_size_matrix(n);

    for x in 0..n {
        for y in 0..n {
            let dot = matrix.get_mut((x, y)).unwrap();
            *dot = ((n - x - y - 1).pow(2) as isize).abs() as f64;
        }
    }

    matrix / (n.pow(2) as f64)
}
