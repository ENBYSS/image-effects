use crate::dither::ordered::tools::{gen_n_size_matrix, Matrix};

pub fn generate_diamonds(n: usize) -> Matrix {
    let n_half = n / 2;

    let mut matrix = gen_n_size_matrix(n);
    let step_size = 1.0 / (n as f64); // from center to edge, plus one step

    for x in 0..n {
        for y in 0..n {
            let distance_x = x.abs_diff(n_half);
            let distance_y = y.abs_diff(n_half);

            // if distance_y > 0 { distance_y = distance_y - 1; }

            let factor = (distance_x + distance_y) as f64 * step_size;
            let point = matrix.get_mut((x, y)).unwrap();
            *point = (1.0 - factor).abs();
        }
    }

    matrix
}
