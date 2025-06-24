#[inline] pub fn sum(nums: &[u8]) -> usize {
    return nums.iter().fold(0, |acc, &e| acc + e as usize);
}

#[inline] pub fn average(nums: &[u8]) -> f64 {
    return sum(nums) as f64 / nums.len() as f64;
}

#[inline] pub fn map_to_2d(cell_no: usize, xdim: usize) -> (usize, usize) {
    (
        cell_no % xdim,
        cell_no / xdim,
    )
}

pub fn f_mod(num: f64, modulo: usize) -> usize {
    let mut num = num as isize;
    while num < 0 {
        num = num + modulo as isize;
    }
    num as usize % modulo
}