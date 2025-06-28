use crate::dither::ordered::tools::{gen_n_size_matrix, Matrix};

pub fn generate_diagonal_tiles(n: usize) -> Matrix {
    let mut matrix = gen_n_size_matrix(n);

    for a in 0..n {
        let min = a;
        let max = n - a;

        for i in min..max {
            let to_mark = matrix.get_mut((a, i)).unwrap();
            *to_mark = (n - i) as f64;
            let to_mark = matrix.get_mut((n - a - 1, i)).unwrap();
            *to_mark = (n - i) as f64;

            let to_mark = matrix.get_mut((i, a)).unwrap();
            *to_mark = (n - i) as f64;
            let to_mark = matrix.get_mut((i, n - a - 1)).unwrap();
            *to_mark = (n - i) as f64;
        }
    }

    matrix / (n as f64)
}
