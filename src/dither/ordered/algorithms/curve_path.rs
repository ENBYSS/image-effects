use std::f64::consts::PI;

use crate::dither::ordered::tools::{
    gen_n_size_matrix, gen_n_size_visitor_matrix, normalize_matrix, Matrix,
};

pub fn generate_curve_path_matrix(
    n: usize,
    halt_threshold: usize,
    amplitude: f64,
    promotion: f64,
) -> Matrix {
    let mut matrix = gen_n_size_matrix(n);
    let mut visitor_m = gen_n_size_visitor_matrix(n);
    let curve_end = PI / 2.;
    let c_factor = curve_end / n as f64;

    let mut i = 0;
    let mut h = 0;

    while h < halt_threshold || (i % n != 0) {
        let h_wrap = i / n;
        let a = amplitude + (h_wrap as f64 * promotion);
        let y_wrap_off = h_wrap as f64 * a;

        let y = (y_wrap_off + f64::sin(((i % n) as f64 * c_factor * a) % curve_end)) * n as f64;
        let x = i % n;

        let y = y.round() as usize % n;

        let point = matrix.get_mut((y, x)).unwrap();
        *point += 1.;

        if visitor_m[y][x] {
            h += 1;
        } else {
            h = 0;
        }

        visitor_m[y][x] = true;
        i += 1;
    }

    normalize_matrix(matrix)
}
