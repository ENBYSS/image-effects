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

#[inline]
fn gen_n_size_matrix(n: usize) -> OrderedMatrix {
    Array::<f64, _>::zeros((n, n))
}

#[inline]
fn gen_n_size_visitor_matrix(n: usize) -> Vec<Vec<bool>> {
    vec![vec![false; n]; n]
}

fn normalize_matrix(matrix: OrderedMatrix) -> OrderedMatrix {
    let mut max = 0.0;
    for cell in matrix.iter() {
        if *cell > max {
            max = *cell;
        }
    }

    matrix / max
}

pub fn dither_bayer(n: usize) -> OrderedMatrix {
    if n == 1 {
        return gen_n_size_matrix(n);
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
    let mut matrix =  gen_n_size_matrix(n);
    let mut visitor_m = gen_n_size_visitor_matrix(n);
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

    normalize_matrix(matrix)
}

pub fn generate_zigzag_matrix(n: usize, halt_threshold: usize, wrapping: Wrapping, magnitude: (f64, f64), promotion: (f64, f64)) -> OrderedMatrix {
    let (magnitude_y, magnitude_x) = magnitude;
    let (promotion_y, promotion_x) = promotion;
    
    let mut matrix =  gen_n_size_matrix(n);
    let mut visitor_m = gen_n_size_visitor_matrix(n);

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

    normalize_matrix(matrix)
}

pub fn generate_broken_spiral_matrix(n: usize, base_step: (f64, f64), oob_threshold: usize, increment_by: f64, increment_every: usize) -> OrderedMatrix {
    let mut matrix = gen_n_size_matrix(n);

    let mut m = 1.;

    loop {
        let mut o = 0;
        let mut i = 1;
        let mut loc = (n as f64 / 2., n as f64 / 2.);
        let mut moves = 0;

        fn get_magnitude(i_by: f64, i_every: usize, moves: usize) -> f64 {
            1.0 + ((moves / i_every) + 1) as f64 * i_by
        }

        let base_step = (base_step.0 * m, base_step.1 * m);

        {
            let loc_i = (loc.0 as usize, loc.1 as usize);
            let point = matrix.get_mut((loc_i.0, loc_i.1)).unwrap();
            *point = *point + get_magnitude(increment_by, increment_every, moves);
            // println!("m: {m}, point: {loc_i:?}, point-value: {}", *point);
        }

        while o <= oob_threshold {
            let disps = [
                (-1. * base_step.0 * i as f64, 0.),
                (0., base_step.1 * i as f64),
                (base_step.0 * (i+1) as f64, 0.),
                (0., -1. * base_step.1 * (i+1) as f64),
            ];

            for disp in disps {
                let og_loc = loc;

                loc = (loc.0 + disp.0, loc.1 + disp.1);

                // if og_loc.0 == (n as f64) / 2. {
                //     println!("m: {m}, i: {i}, new_loc: {loc:?}");
                // }

                let loc_i = (loc.0 as isize, loc.1 as isize);

                if loc_i.0 < 0 || loc_i.1 < 0 || loc_i.0 >= n as isize || loc_i.1 >= n as isize {
                    o = o + 1;
                    continue;
                } else {
                    o = 0;
                }

                // let point = matrix.get_mut((loc_i.0 as usize, loc_i.1 as usize)).unwrap();
                // *point = *point + 1.;

                let move_in_x = disp.0 == 0.;

                let get_coord = |coords: (f64, f64)| if move_in_x { coords.1 } else { coords.0 };

                let min = get_coord(loc).min(get_coord(og_loc).min(n as f64).max(0.));
                let max = get_coord(loc).max(get_coord(og_loc).min(n as f64).max(0.));

                let other_coord = if move_in_x { og_loc.0 } else { og_loc.1 };

                let mut draw_loc = min;

                while draw_loc <= max {
                    let p_coord = if move_in_x { (other_coord as usize, draw_loc as usize) } else { (draw_loc as usize, other_coord as usize) };
                    let point = matrix.get_mut(p_coord).unwrap();
                    *point = *point + get_magnitude(increment_by, increment_every, moves);
                    // println!("{og_loc:?} - {loc:?} | incrementing: {p_coord:?} to {point}");
                    draw_loc = draw_loc + if move_in_x { base_step.1 / m } else { base_step.0 / m };
                    moves = moves + 1;
                }
                // println!("line-done!");
            }

            i = i + 2;
        }
        if i < o {
            break;
        }

        m = m + 1.;
    }

    // println!("{matrix:#?}");

    // let loc = (
    //     n as f64 / 2.0,
    //     n as f64 / 2.0,
    // );
    // let point = matrix.get_mut((loc.0 as usize, loc.1 as usize)).unwrap();
    // println!("m: {m}, point: {loc:?}, point-value: {}", *point);

    normalize_matrix(matrix)
}

pub fn generate_modulosnake(n: usize, increment_by: f64, modulo: usize, iterations: usize) -> OrderedMatrix {
    let mut matrix = gen_n_size_matrix(n);

    for _ in 0..iterations {
        for (n, pixel) in matrix.iter_mut().enumerate() {
            *pixel = ((n as f64 * increment_by) as usize % modulo) as f64;
        }
    }

    normalize_matrix(matrix)
}
// 1 up
// 1 right
// 2 down
// 2 left
// 3 up
// 3 right