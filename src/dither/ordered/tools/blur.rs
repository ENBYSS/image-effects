use crate::{
    dither::ordered::tools::{gen_n_size_matrix, Matrix},
    utils::numops::abs_i_mod,
};

pub fn blur_matrix(matrix: &mut Matrix, blur_amnt: usize) {
    let (y, x) = matrix.dim();
    let mut result = gen_n_size_matrix(x);

    for a in 0..y as isize {
        for b in 0..x as isize {
            let mut sum = 0.0;

            for ai in -(blur_amnt as isize)..blur_amnt as isize {
                for bi in -(blur_amnt as isize)..blur_amnt as isize {
                    let blur_target = (abs_i_mod(a + ai, y), abs_i_mod(b + bi, x));
                    sum += matrix.get(blur_target).unwrap();
                }
            }

            let target = result.get_mut((a as usize, b as usize)).unwrap();
            *target = sum;
        }
    }
}
