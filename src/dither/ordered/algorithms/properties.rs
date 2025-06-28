#[derive(Debug, Clone)]
pub enum Wrapping {
    Horizontal,
    Vertical,
    All,
    None,
}

#[derive(Debug, Clone)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone)]
pub enum DiagonalDirection {
    DownRight,
    UpRight,
}

#[derive(Debug, Clone)]
pub enum Increase {
    Linear(u8),
    Exponential(u8),
}
