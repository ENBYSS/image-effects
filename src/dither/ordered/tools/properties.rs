use std::usize;

#[derive(Debug, Clone)]
pub enum Rotation {
    Right,
    Left,
    Half,
    None,
}

impl Rotation {
    pub fn rotate_coords(&self, matrix_size: usize, coords: (usize, usize)) -> (usize, usize) {
        let (y, x) = coords;

        match self {
            Rotation::Right => (matrix_size - x, y),
            Rotation::Half => (matrix_size - x, matrix_size - y),
            Rotation::Left => (x, matrix_size - y),
            Rotation::None => coords,
        }
    }
}

pub enum Checker {
    Iter(usize),
    From((usize, usize)),
}
