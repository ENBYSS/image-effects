use std::f64::consts::PI;

use ndarray::{concatenate, Array, Axis, Dim};

use crate::utils::numops::f_mod;

#[derive(Debug, Clone)]
pub enum Wrapping {
    Horizontal,
    Vertical,
    All,
    None,
}

pub type OrderedMatrix = Array<f64, Dim<[usize; 2]>>;

pub fn dither_bayer(n: usize) -> OrderedMatrix {
    if n == 1 {
        return Array::<f64, _>::zeros((1, 1));
    }

    let nested_matrix = dither_bayer(n / 2);
    let multiplier = n.pow(2) as f64;

    let first = multiplier * nested_matrix.clone();
    let second = multiplier * nested_matrix.clone() + 2.;
    let third = multiplier * nested_matrix.clone() + 3.;
    let fourth = multiplier * nested_matrix.clone() + 1.;

    let first_col = concatenate(Axis(0), &[first.view(), third.view()]).unwrap();
    let second_col = concatenate(Axis(0), &[second.view(), fourth.view()]).unwrap();

    (1. / multiplier) * concatenate(Axis(1), &[first_col.view(), second_col.view()]).unwrap()
}

pub fn generate_curve_path_matrix(n: usize, halt_threshold: usize, amplitude: f64, promotion: f64) -> OrderedMatrix {
    let mut matrix =  Array::<f64, _>::zeros((n, n));
    let mut visitor_m = vec![vec![false; n]; n];
    let curve_end = PI / 2.;
    let c_factor = curve_end / n as f64;

    let mut i = 0;
    let mut h = 0;

    while h < halt_threshold || (i % n != 0) {
        let h_wrap = i / n;
        let a = amplitude + (h_wrap as f64*promotion);
        let y_wrap_off = h_wrap as f64 * a;

        let y = (y_wrap_off as f64 + f64::sin(((i%n) as f64 * c_factor * a) % curve_end)) * n as f64;
        let x = i % n;

        let y = y.round() as usize % n;

        let point = matrix.get_mut((y, x)).unwrap();
        *point = *point + 1.;

        if visitor_m[y][x] {
            h = h + 1;
        } else {
            h = 0;
        }

        visitor_m[y][x] = true;
        i = i + 1;
    }

    let mut max = 0.0;
    for cell in matrix.iter() {
        if *cell > max {
            max = *cell;
        }
    }

    matrix / max
}

pub fn generate_zigzag_matrix(n: usize, halt_threshold: usize, wrapping: Wrapping, magnitude: (f64, f64), promotion: (f64, f64)) -> OrderedMatrix {
    let (magnitude_y, magnitude_x) = magnitude;
    let (promotion_y, promotion_x) = promotion;
    
    let mut matrix =  Array::<f64, _>::zeros((n, n));
    let mut visitor_m = vec![vec![false; n]; n];

    let mut i = 0;
    let mut h = 0;
    let mut curr = (0., 0.);
    let mut disp = (magnitude_y, magnitude_x);

    while h < halt_threshold {
        let (mut y, mut x) = curr;

        let wrap_x = x / n as f64;
        let wrap_y = y / n as f64;

        let (yi, xi) = match wrapping {
            Wrapping::Horizontal => (y as usize, f_mod(x, n)),
            Wrapping::Vertical => (f_mod(y, n), x as usize),
            Wrapping::All => (f_mod(y, n), f_mod(x, n)),
            Wrapping::None => (y as usize, x as usize),
        };

        let point = matrix.get_mut((yi, xi));

        // println!("ITERATION {i}: pure_coords: ({y}, {x}), mag: ({magnitude_y}, {magnitude_x})");

        // if point.is_none() {
        //     println!("INDEX ERROR: current: {curr:?}, displacement: {disp:?}, n = {n}");
        // }

        let point = point.unwrap();
        *point = *point + 1.;

        if visitor_m[yi][xi] {
            h = h + 1;
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
                    disp.0 = magnitude_y * -1.;
                }

                let magnitude_x = magnitude_x + wrap_x * promotion_x;
                disp.1 = magnitude_x;

                curr = (
                    (y + disp.0).max(0.0).min(n as f64 - 1.0),
                    (x + disp.1),
                );
            },
            Wrapping::Vertical => {
                if x + disp.1 < 0.0 {
                    disp.1 = magnitude_x; 
                    x = 0.;
                } else if xi >= (n - 1) {
                    disp.1 = magnitude_x * -1.;
                }

                let magnitude_y = magnitude_y + wrap_y * promotion_y;
                disp.0 = magnitude_y;

                curr = (
                    (y + disp.0),
                    (x + disp.1).max(0.0).min(n as f64 - 1.0),
                );
            },
            Wrapping::All => {
                let magnitude_x = magnitude_x + wrap_x * promotion_x;
                let magnitude_y = magnitude_y + wrap_y * promotion_y;

                disp = (magnitude_y, magnitude_x);
                curr = (y + disp.0, x + disp.1);
            },
            Wrapping::None => {
                if y + disp.0 < 0.0 {
                    disp.0 = magnitude_y; 
                    y = 0.;
                } else if yi >= (n - 1) {
                    disp.0 = magnitude_y * -1.;
                }

                if x + disp.1 < 0.0 {
                    disp.1 = magnitude_x; 
                    x = 0.;
                } else if xi >= (n - 1) {
                    disp.1 = magnitude_x * -1.;
                }

                curr = (
                    (y + disp.0).max(0.0).min(n as f64 - 1.0),
                    (x + disp.1).max(0.0).min(n as f64 - 1.0),
                );
            },
        }

        i = i + 1;
    }

    let mut max = 0.0;
    for cell in matrix.iter() {
        if *cell > max {
            max = *cell;
        }
    }

    matrix / max
}