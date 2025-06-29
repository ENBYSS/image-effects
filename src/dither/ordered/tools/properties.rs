use std::{arch::x86_64, usize};

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

#[derive(Debug, Clone)]
pub enum CheckerType {
    Iter(usize),
    From {
        source: Source,
        factor: Factor,
        modulo: Option<usize>,
    },
}

#[derive(Debug, Clone)]
pub enum Source {
    Center,
    Fixed(usize, usize),
}

impl Source {
    pub fn get(&self, matrix_size: usize) -> (usize, usize) {
        match self {
            Self::Center => (matrix_size / 2, matrix_size / 2),
            Self::Fixed(y, x) => (*y, *x),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Factor {
    Exponential(f64),
    Linear,
}
