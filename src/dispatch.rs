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

impl<T> Null<T> {
    pub fn _only_use_if_needed() -> Self {
        Null { _data: PhantomData }
    }
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

impl<T> EffectEnum<T>
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
    pub fn affect(&self, item: T) -> T {
        match self {
            Self::HueRotate(fx) => fx.affect(item),
            Self::Contrast(fx) => fx.affect(item),
            Self::Brighten(fx) => fx.affect(item),
            Self::Saturate(fx) => fx.affect(item),
            Self::GradientMap(fx) => fx.affect(item),
            Self::QuantizeHue(fx) => fx.affect(item),
            Self::MultiplyHue(fx) => fx.affect(item),
            Self::Invert(fx) => fx.affect(item),
            Self::Ordered(fx) => fx.affect(item),
            Self::ErrorPropagator(fx) => fx.affect(item),
            Self::Null(fx) => fx.affect(item),
        }
    }
}
