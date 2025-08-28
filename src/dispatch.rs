use std::marker::PhantomData;

use enum_dispatch::enum_dispatch;

use crate::dither::error::{ErrorPropagator, WithPalette};
use crate::dither::ordered::Ordered;
use crate::effect::Effect;
use crate::prelude::filters::{
    Brighten, Contrast, GradientMap, HueRotate, Invert, MultiplyHue, QuantizeHue, Saturate,
};
use crate::utils::image::RgbPixelRepr;

pub struct Null<T> {
    _data: PhantomData<T>,
}

impl<T> Effect<RgbPixelRepr> for Null<T> {
    fn affect(&self, item: RgbPixelRepr) -> RgbPixelRepr {
        item
    }
}

#[enum_dispatch(Effect<T>)]
pub enum EffectEnum<T>
where
    HueRotate: Effect<T>,
    Contrast: Effect<T>,
    Brighten: Effect<T>,
    Saturate: Effect<T>,
    GradientMap: Effect<T>,
    QuantizeHue: Effect<T>,
    MultiplyHue: Effect<T>,
    Invert: Effect<T>,
    Ordered: Effect<T>,
    ErrorPropagator<'static, 'static, WithPalette>: Effect<T>,
    Null<T>: Effect<T>,
{
    HueRotate,
    Contrast,
    Brighten,
    Saturate,
    GradientMap,
    QuantizeHue,
    MultiplyHue,
    Invert,
    Ordered,
    ErrorPropagator(ErrorPropagator<'static, 'static, WithPalette>),
    Null(Null<T>),
}
