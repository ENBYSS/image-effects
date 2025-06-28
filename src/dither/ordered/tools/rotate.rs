use crate::dither::ordered::tools::{gen_n_size_matrix, properties::Rotation, Matrix};

pub fn rotate_matrix(matrix: &mut Matrix, rotation: Rotation) {
    let (y, x) = matrix.dim();
    let mut result = gen_n_size_matrix(x);

    for a in 0..y {
        for b in 0..x {
            let source = { matrix.get((a, b)).unwrap() };

            let (a, b) = rotation.rotate_coords(y, (a, b));

            let target = result.get_mut((a, b)).unwrap();

            *target = *source;
        }
    }
}
