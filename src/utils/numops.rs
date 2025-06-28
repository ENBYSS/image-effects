#[inline]
pub fn map_to_2d(cell_no: usize, xdim: usize) -> (usize, usize) {
    (cell_no % xdim, cell_no / xdim)
}

pub fn abs_f_mod(num: f64, modulo: usize) -> usize {
    let mut num = num as isize;
    while num < 0 {
        num += modulo as isize;
    }
    num as usize % modulo
}

pub fn abs_i_mod(mut num: isize, modulo: usize) -> usize {
    while num < 0 {
        num += modulo as isize;
    }
    num as usize % modulo
}
