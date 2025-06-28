use crate::{
    dither::ordered::{
        algorithms::properties::Wrapping,
        tools::{gen_n_size_matrix, gen_n_size_visitor_matrix, normalize_matrix, Matrix},
    },
    utils::numops::abs_f_mod,
};

pub fn generate_zigzag_matrix(
    n: usize,
    halt_threshold: usize,
    wrapping: Wrapping,
    magnitude: (f64, f64),
    promotion: (f64, f64),
) -> Matrix {
    let (magnitude_y, magnitude_x) = magnitude;
    let (promotion_y, promotion_x) = promotion;

    let mut matrix = gen_n_size_matrix(n);
    let mut visitor_m = gen_n_size_visitor_matrix(n);

    let mut h = 0;
    let mut curr = (0., 0.);
    let mut disp = (magnitude_y, magnitude_x);

    while h < halt_threshold {
        let (mut y, mut x) = curr;

        let wrap_x = x / n as f64;
        let wrap_y = y / n as f64;

        let (yi, xi) = match wrapping {
            Wrapping::Horizontal => (y as usize, abs_f_mod(x, n)),
            Wrapping::Vertical => (abs_f_mod(y, n), x as usize),
            Wrapping::All => (abs_f_mod(y, n), abs_f_mod(x, n)),
            Wrapping::None => (y as usize, x as usize),
        };

        let point = matrix.get_mut((yi, xi));

        // println!("ITERATION {i}: pure_coords: ({y}, {x}), mag: ({magnitude_y}, {magnitude_x})");

        // if point.is_none() {
        //     println!("INDEX ERROR: current: {curr:?}, displacement: {disp:?}, n = {n}");
        // }

        let point = point.unwrap();
        *point += 1.;

        if visitor_m[yi][xi] {
            h += 1;
        } else {
            h = 0;
        }
        visitor_m[yi][xi] = true;

        match wrapping {
            Wrapping::Horizontal => {
                if y + disp.0 < 0.0 {
                    disp.0 = magnitude_y;
                    y = 0.;
                } else if yi >= (n - 1) {
                    disp.0 = -magnitude_y;
                }

                let magnitude_x = magnitude_x + wrap_x * promotion_x;
                disp.1 = magnitude_x;

                curr = ((y + disp.0).max(0.0).min(n as f64 - 1.0), (x + disp.1));
            }
            Wrapping::Vertical => {
                if x + disp.1 < 0.0 {
                    disp.1 = magnitude_x;
                    x = 0.;
                } else if xi >= (n - 1) {
                    disp.1 = -magnitude_x;
                }

                let magnitude_y = magnitude_y + wrap_y * promotion_y;
                disp.0 = magnitude_y;

                curr = ((y + disp.0), (x + disp.1).max(0.0).min(n as f64 - 1.0));
            }
            Wrapping::All => {
                let magnitude_x = magnitude_x + wrap_x * promotion_x;
                let magnitude_y = magnitude_y + wrap_y * promotion_y;

                disp = (magnitude_y, magnitude_x);
                curr = (y + disp.0, x + disp.1);
            }
            Wrapping::None => {
                if y + disp.0 < 0.0 {
                    disp.0 = magnitude_y;
                    y = 0.;
                } else if yi >= (n - 1) {
                    disp.0 = -magnitude_y;
                }

                if x + disp.1 < 0.0 {
                    disp.1 = magnitude_x;
                    x = 0.;
                } else if xi >= (n - 1) {
                    disp.1 = -magnitude_x;
                }

                curr = (
                    (y + disp.0).max(0.0).min(n as f64 - 1.0),
                    (x + disp.1).max(0.0).min(n as f64 - 1.0),
                );
            }
        }
    }

    normalize_matrix(matrix)
}
