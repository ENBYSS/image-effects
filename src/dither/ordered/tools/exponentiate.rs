use crate::dither::ordered::tools::Matrix;

pub fn exponentiate_matrix(matrix: &mut Matrix, factor: f64) {
    for pix in matrix.iter_mut() {
        *pix = pix.powf(factor);
    }
}
