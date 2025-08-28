use super::conversions::lch_to_lab;

type Colour = (f32, f32, f32);

/// Calculates the distance between two RGB colours using the euclidean distance function.
///
/// Also includes weight to more accurately calculate the distance.
pub fn rgb_weighted_euclidean(rgb_a: Colour, rgb_b: Colour) -> f32 {
    let r_avg = (rgb_a.0.algebraic_add(rgb_b.0)).algebraic_div(2.0);
    let m: (f32, f32, f32) = if r_avg > 0.5 {
        (3.0, 4.0, 2.0)
    } else {
        (2.0, 4.0, 3.0)
    };

    fn algebraic_sq(a: f32) -> f32 {
        a.algebraic_mul(a)
    }

    let diff_r = algebraic_sq(m.0.algebraic_mul(rgb_a.0.algebraic_sub(rgb_b.0)));
    let diff_g = algebraic_sq(m.1.algebraic_mul(rgb_a.1.algebraic_sub(rgb_b.1)));
    let diff_b = algebraic_sq(m.2.algebraic_mul(rgb_a.2.algebraic_sub(rgb_b.2)));

    diff_r + diff_g + diff_b
}

pub fn cie76(lab_a: Colour, lab_b: Colour) -> f32 {
    ((lab_b.0 - lab_a.0).powi(2) + (lab_b.1 - lab_a.1).powi(2) + (lab_b.2 - lab_a.2).powi(2)).sqrt()
}

/// This function may not work correctly.
pub fn cie94(lch_a: Colour, lch_b: Colour) -> f32 {
    const K_L: f32 = 1.0;
    const K_C: f32 = 1.0;
    const K_H: f32 = 1.0;

    let delta_l = lch_a.0 - lch_b.0;
    let delta_c = lch_a.1 - lch_b.1;
    let delta_h = lch_a.2 - lch_b.2;

    let (s_l, s_c, s_h) = (1.0, 1.0 + 0.045 * lch_a.1, 1.0 + 0.015 * lch_a.1);

    ((delta_l / (K_L * s_l)).powi(2)
        + (delta_c / (K_C * s_c)).powi(2)
        + (delta_h / (K_H * s_h)).powi(2))
    .sqrt()
}

/// Calculates the distance between two LCH colours using CIEDE2000.
///
/// Not confirmed to be fully functional yet - however this algorithm is
/// proven to be the best, albeit significantly slower due to more computations.
pub fn ciede2000(lch_a: Colour, lch_b: Colour) -> f32 {
    // set up constants for formula
    // these are usually unity (1)
    let k_l: f32 = 1.0;
    let k_c: f32 = 1.0;
    let k_h: f32 = 1.0;

    // get LAB values - needed for formula
    let (_, a_1, b_1) = lch_to_lab(lch_a);
    let (_, a_2, b_2) = lch_to_lab(lch_b);

    // set up variables for formula
    let delta_l_mark = lch_b.0 - lch_a.0;

    let avg_l = (lch_b.0 + lch_a.0) / 2.0;
    let avg_c = (lch_b.1 + lch_a.1) / 2.0;

    let c_7_mul = 1.0 - (avg_c.powi(7) / (avg_c.powi(7) + 25_f32.powi(7)).sqrt());
    let a_1_mark = a_1 + (a_1 / 2.0) * c_7_mul;
    let a_2_mark = a_2 + (a_2 / 2.0) * c_7_mul;

    let c_1_mark = (a_1_mark.powi(2) + b_1.powi(2)).sqrt();
    let c_2_mark = (a_2_mark.powi(2) + b_2.powi(2)).sqrt();

    let delta_c_mark = c_2_mark - c_1_mark;
    let avg_c_mark = (c_2_mark + c_1_mark) / 2.0;

    let h_1_mark = b_1.atan2(a_1_mark).to_degrees() % 360.0;
    let h_2_mark = b_1.atan2(a_1_mark).to_degrees() % 360.0;

    let abs_diff_h_marks = (h_1_mark - h_2_mark).abs();
    let delta_h_mark = if c_1_mark == 0.0 || c_2_mark == 0.0 {
        0.0
    } else if abs_diff_h_marks <= 180.0 {
        h_2_mark - h_1_mark
    } else if abs_diff_h_marks > 180.0 && h_2_mark <= h_1_mark {
        h_2_mark - h_1_mark + 360.0
    } else {
        h_2_mark - h_1_mark - 360.0
    };

    let delta_big_h_mark =
        2.0 * (c_1_mark * c_2_mark).sqrt() * (delta_h_mark / 2.0).to_radians().sin();
    let avg_big_h_mark = if c_1_mark == 0.0 || c_2_mark == 0.0 {
        h_1_mark + h_2_mark
    } else if abs_diff_h_marks <= 180.0 {
        (h_1_mark + h_2_mark) / 2.0
    } else if abs_diff_h_marks > 180.0 && h_1_mark + h_2_mark < 360.0 {
        (h_1_mark + h_2_mark + 360.0) / 2.0
    } else {
        (h_1_mark + h_2_mark - 360.0) / 2.0
    };

    let t = 1.0 - 0.17 * (avg_big_h_mark - 30.0).to_radians().cos()
        + 0.24 * (avg_big_h_mark * 2.0).to_radians().cos()
        + 0.32 * (avg_big_h_mark * 3.0 + 6.0).to_radians().cos()
        - 0.20 * (avg_big_h_mark * 4.0 - 63.0).to_radians().cos();

    let s_l = 1.0 + (0.015 * (avg_l - 50.0).powi(2)) / (20.0 + (avg_l - 50.0).powi(2));

    let s_c = 1.0 + 0.045 * avg_c_mark;
    let s_h = 1.0 + 0.015 * avg_c_mark * t;

    let r_t = -2.0
        * (avg_c_mark.powi(7) / (avg_c_mark.powi(7) + 25_f32.powi(7))).sqrt()
        * (60.0 * (-((avg_big_h_mark - 275.0) / 25.0).powi(2)).exp())
            .to_radians()
            .sin();

    // the actual formula
    ((delta_l_mark / (k_l * s_l)).powi(2)
        + (delta_c_mark / (k_c * s_c)).powi(2)
        + (delta_big_h_mark / (k_h * s_h)).powi(2)
        + r_t * (delta_c_mark / (k_c * s_c)) * (delta_big_h_mark / (k_h * s_h)))
        .sqrt()
}
