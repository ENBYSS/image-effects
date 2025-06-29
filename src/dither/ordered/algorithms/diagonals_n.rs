use crate::dither::ordered::{
    algorithms::properties::{DiagonalDirection, Increase},
    tools::{gen_n_size_matrix, Matrix},
};

pub fn generate_diagonals_n(n: usize, direction: DiagonalDirection, increase: Increase) -> Matrix {
    let mut matrix = gen_n_size_matrix(n);

    let mut numerals = Vec::<f64>::new();

    for i in 0..n {
        numerals.push(match increase {
            Increase::Linear(f) => i as f64 * f as f64,
            Increase::Exponential(f) => f.pow(i as u32) as f64,
        });
    }

    if let DiagonalDirection::DownRight = direction {
        numerals.reverse();
    }

    for x in 0..n {
        for y in 0..n {
            let dot = matrix.get_mut((x, y)).unwrap();

            // This will iterate over the array, and then shift to the right with mapping.
            // "% n" handles the mapping.
            // x moves laterally through the numerals
            // (y * (n-1)) ensures that it will shift correctly.
            //
            // [0, 1, 2], [2, 0, 1], [1, 2, 0]
            // (0, 0) [0], (1, 0) [1], (2, 0) [2], (0, 1) [2], (1,1) [3 % 3 = 0]
            *dot = numerals[(y * (n - 1) + x) % n];
        }
    }

    matrix
        / *numerals
            .iter()
            .max_by(|a, b| a.total_cmp(b))
            .expect("[E001] Couldn't compute max in DiagonalsN.")
}
