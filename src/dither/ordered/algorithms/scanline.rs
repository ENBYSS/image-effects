use crate::dither::ordered::{
    algorithms::properties::Orientation,
    tools::{gen_n_size_matrix, Matrix},
};

pub fn generate_scanline(n: usize, orientation: Orientation) -> Matrix {
    let mut matrix = gen_n_size_matrix(n);

    for x in 0..n {
        for y in 0..n {
            let main_coord = if let Orientation::Vertical = orientation {
                x
            } else {
                y
            };
            let bigger_coord = main_coord.max(n - main_coord);

            let point = matrix.get_mut((x, y)).unwrap();

            *point = bigger_coord as f64;
        }
    }

    matrix / n as f64
}
