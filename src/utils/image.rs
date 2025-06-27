pub type RgbPixelRepr = [u8; 3];
pub type RgbaPixelRepr = [u8; 4];

pub type RgbImageRepr = Vec<Vec<RgbPixelRepr>>;
pub type RgbaImageRepr = Vec<Vec<RgbaPixelRepr>>;

pub(crate) fn get_dimensions_of_matrix<T>(
    matrix: &[Vec<T>]
) -> (usize, usize)
{
    let ydim = matrix.len();
    let xdim = matrix.first().map(|row| row.len()).unwrap_or(0);
    (xdim, ydim)
}