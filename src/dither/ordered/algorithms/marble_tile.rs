use crate::dither::ordered::tools::{gen_n_size_matrix, Matrix};

pub fn generate_marble_tile(n: usize) -> Matrix {
    let mut matrix = gen_n_size_matrix(n);

    for x in 0..n {
        for y in 0..n {
            let mag = x as isize - y as isize;
            let point = matrix.get_mut((x, y)).unwrap();
            *point = mag as f64;
        }
    }

    matrix /= n as f64;
    matrix += 1.;
    matrix * 0.5
}
