use enum_dispatch::enum_dispatch;
use image::{DynamicImage, Frame, ImageBuffer, Rgb, Rgba};

use crate::utils::{
    image::{get_dimensions_of_matrix, RgbImageRepr, RgbPixelRepr, RgbaImageRepr, RgbaPixelRepr},
    numops::map_to_2d,
};

/// Defines an effect that can be applied onto `T`.
///
/// Implementing this auto-implements `Affectable<T, F>` for `T`,
/// where `F` is this effect.
#[enum_dispatch]
pub trait Effect<T> {
    /// Affects `T` using `self`.
    fn affect(&self, item: T) -> T;
}

/// Defines something that can be affected.
///
/// This is auto-implemented for any `T` and `F` where `F`
/// is an effect that can be applied on `T`.
///
/// As a result it should almost never be implemented directly, like `Into<T>` in the
/// standard library.
pub trait Affectable<T, F: Effect<T>> {
    /// Applies an effect on `self`.
    fn apply(self, effect: &F) -> Self;
}

impl<T, F> Affectable<T, F> for T
where
    F: Effect<T>,
{
    fn apply(self, effect: &F) -> Self {
        effect.affect(self)
    }
}

impl<F> Effect<RgbaPixelRepr> for F
where
    F: Effect<RgbPixelRepr>,
{
    fn affect(&self, item: RgbaPixelRepr) -> RgbaPixelRepr {
        let [r, g, b, a] = item;
        let [r, g, b] = self.affect([r, g, b]);
        [r, g, b, a]
    }
}

impl<F> Effect<RgbImageRepr> for F
where
    F: Effect<RgbPixelRepr>,
{
    fn affect(&self, mut item: RgbImageRepr) -> RgbImageRepr {
        for row in item.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = self.affect(*pixel);
            }
        }
        item
    }
}

impl<F> Effect<RgbaImageRepr> for F
where
    F: Effect<RgbImageRepr>,
{
    fn affect(&self, item: RgbaImageRepr) -> RgbaImageRepr {
        let (xs, ys) = get_dimensions_of_matrix(&item);

        let mut rgb_repr = vec![vec![[0_u8; 3]; xs]; ys];
        let mut output = vec![vec![[0_u8; 4]; xs]; ys];

        for y in 0..ys {
            for x in 0..xs {
                let [r, g, b, _] = item[y][x];
                rgb_repr[y][x] = [r, g, b];
            }
        }

        let rgb_repr = self.affect(rgb_repr);

        for y in 0..ys {
            for x in 0..xs {
                let [r, g, b] = rgb_repr[y][x];
                output[y][x] = [r, g, b, item[y][x][3]];
            }
        }

        output
    }
}

impl<F> Effect<ImageBuffer<Rgb<u8>, Vec<u8>>> for F
where
    F: Effect<RgbImageRepr>,
{
    fn affect(&self, item: ImageBuffer<Rgb<u8>, Vec<u8>>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let (xs, ys) = item.dimensions();
        let (xs, ys) = (xs as usize, ys as usize);

        let mut img_matrix = vec![vec![[0_u8; 3]; xs]; ys];

        for (i, pixel) in item.pixels().enumerate() {
            let (x, y) = map_to_2d(i, xs);
            img_matrix[y][x] = pixel.0;
        }

        img_matrix = self.affect(img_matrix);

        let (xdim, ydim) = get_dimensions_of_matrix(&img_matrix);

        ImageBuffer::from_fn(xdim as u32, ydim as u32, |x, y| {
            image::Rgb(img_matrix[y as usize][x as usize])
        })
    }
}

impl<F> Effect<ImageBuffer<Rgba<u8>, Vec<u8>>> for F
where
    F: Effect<RgbaImageRepr>,
{
    fn affect(&self, item: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (xs, ys) = item.dimensions();
        let (xs, ys) = (xs as usize, ys as usize);

        let mut img_matrix = vec![vec![[0_u8; 4]; xs]; ys];

        for (i, pixel) in item.pixels().enumerate() {
            let (x, y) = map_to_2d(i, xs);
            img_matrix[y][x] = pixel.0;
        }

        img_matrix = self.affect(img_matrix);

        let (xdim, ydim) = get_dimensions_of_matrix(&img_matrix);

        ImageBuffer::from_fn(xdim as u32, ydim as u32, |x, y| {
            image::Rgba(img_matrix[y as usize][x as usize])
        })
    }
}

impl<F> Effect<DynamicImage> for F
where
    F: Effect<ImageBuffer<Rgb<u8>, Vec<u8>>> + Effect<ImageBuffer<Rgba<u8>, Vec<u8>>>,
{
    fn affect(&self, item: DynamicImage) -> DynamicImage {
        match item {
            DynamicImage::ImageRgb8(img) => DynamicImage::from(self.affect(img)),
            DynamicImage::ImageRgba8(img) => DynamicImage::from(self.affect(img)),
            _ => DynamicImage::ImageRgb8(self.affect(item.into_rgb8())),
        }
    }
}

impl<F> Effect<Frame> for F
where
    F: Effect<ImageBuffer<Rgba<u8>, Vec<u8>>>,
{
    fn affect(&self, item: Frame) -> Frame {
        let left = item.left();
        let top = item.top();
        let delay = item.delay();

        let new_buf = self.affect(item.into_buffer());
        Frame::from_parts(new_buf, left, top, delay)
    }
}
