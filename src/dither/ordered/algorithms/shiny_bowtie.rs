use crate::dither::ordered::tools::{gen_n_size_matrix, Matrix};

pub fn generate_shiny_bowtie(n: usize) -> Matrix {
    let mut matrix = gen_n_size_matrix(n);

    for x in 0..n {
        for y in 0..n {
            let max_c = x.max(y);
            let min_c = x.min(y);
            let bigger_coord = (max_c.pow(2) as f64 / (min_c.pow(2) + 1) as f64).abs();

            let point = matrix.get_mut((x, y)).unwrap();

            *point = bigger_coord;
        }
    }

    matrix / (n - 1).pow(2) as f64
}
