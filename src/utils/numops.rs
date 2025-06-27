#[inline] pub fn map_to_2d(cell_no: usize, xdim: usize) -> (usize, usize) {
    (
        cell_no % xdim,
        cell_no / xdim,
    )
}

pub fn f_mod(num: f64, modulo: usize) -> usize {
    let mut num = num as isize;
    while num < 0 {
        num += modulo as isize;
    }
    num as usize % modulo
}