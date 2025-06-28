use crate::dither::ordered::tools::{gen_n_size_matrix, normalize_matrix, Matrix};

pub fn generate_broken_spiral_matrix(
    n: usize,
    base_step: (f64, f64),
    oob_threshold: usize,
    increment_by: f64,
    increment_every: usize,
) -> Matrix {
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
            *point += get_magnitude(increment_by, increment_every, moves);
            // println!("m: {m}, point: {loc_i:?}, point-value: {}", *point);
        }

        while o <= oob_threshold {
            let disps = [
                (-base_step.0 * i as f64, 0.),
                (0., base_step.1 * i as f64),
                (base_step.0 * (i + 1) as f64, 0.),
                (0., -base_step.1 * (i + 1) as f64),
            ];

            for disp in disps {
                let og_loc = loc;

                loc = (loc.0 + disp.0, loc.1 + disp.1);

                // if og_loc.0 == (n as f64) / 2. {
                //     println!("m: {m}, i: {i}, new_loc: {loc:?}");
                // }

                let loc_i = (loc.0 as isize, loc.1 as isize);

                if loc_i.0 < 0 || loc_i.1 < 0 || loc_i.0 >= n as isize || loc_i.1 >= n as isize {
                    o += 1;
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
                    let p_coord = if move_in_x {
                        (other_coord as usize, draw_loc as usize)
                    } else {
                        (draw_loc as usize, other_coord as usize)
                    };
                    let point = matrix.get_mut(p_coord).unwrap();
                    *point = (*point + get_magnitude(increment_by, increment_every, moves)).abs();
                    // println!("{og_loc:?} - {loc:?} | incrementing: {p_coord:?} to {point}");
                    draw_loc += if move_in_x {
                        base_step.1 / m
                    } else {
                        base_step.0 / m
                    };
                    moves += 1;
                }
                // println!("line-done!");
            }

            i += 2;
        }
        if i < o {
            break;
        }

        m += 1.;
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
