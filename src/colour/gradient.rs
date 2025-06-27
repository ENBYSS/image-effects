use palette::{IntoColor, Lch, Hsl, FromColor, Oklch};

// gradient logic
pub enum GradientMethod {
    LCH,
    HSL,
    OKLCH,
}

/// Defines a colour that can generate a gradient through Lch.
/// 
/// Should be auto-implemented by having a colour satisfy the trait bounds.
pub trait IntoGradientLch: Sized + IntoColor<Lch> + FromColor<Lch> + Copy {
    fn build_gradient_lch(self, shades: u16) -> Vec<Self> {
        let step_size = 100.0 / (shades+1) as f32;

        (1..shades)
            .map(|i| {
                let mut color: Lch = self.into_color();
                color.l = i as f32 * step_size;
                Self::from_color(color)
            })
            .collect()
    }
}

/// Defines a colour that can generate a gradient through Hsl.
/// 
/// Should be auto-implemented by having a colour satisfy the trait bounds.
pub trait IntoGradientHsl: Sized + IntoColor<Hsl> + FromColor<Hsl> + Copy {
    fn build_gradient_hsl(self, shades: u16) -> Vec<Self> {
        let step_size = 1.0 / (shades+1) as f32;

        (1..shades)
            .map(|i| {
                let mut color: Hsl = self.into_color();
                color.lightness = i as f32 * step_size;
                Self::from_color(color)
            })
            .collect()
    }
}

/// Defines a colour that can generate a gradient through Oklch. Tends to distort hues... unknown why.
/// 
/// Should be auto-implemented by having a colour satisfy the trait bounds.
pub trait IntoGradientOklch: Sized + IntoColor<Oklch> + FromColor<Oklch> + Copy {
    fn build_gradient_oklch(self, shades: u16) -> Vec<Self> {
        let step_size = 1.0 / (shades+1) as f32;

        (1..shades)
            .map(|i| {
                let mut color: Oklch = self.into_color();
                color.l = i as f32 * step_size;
                Self::from_color(color)
            })
            .collect()
    }
}

/// Defines a colour that can generate a gradient through all supported gradient methods.
/// 
/// Should be auto-implemented by having a colour satisfy the trait bounds.
pub trait IntoGradient: IntoGradientHsl + IntoGradientLch + IntoGradientOklch {
    fn build_gradient(self, shades: u16, method: GradientMethod) -> Vec<Self> {
        match method {
            GradientMethod::HSL => self.build_gradient_hsl(shades),
            GradientMethod::LCH => self.build_gradient_lch(shades),
            GradientMethod::OKLCH => self.build_gradient_oklch(shades),
        }
    }
}

// general implementations
impl<C> IntoGradientHsl for C where 
    C: Sized + IntoColor<Hsl> + FromColor<Hsl> + Copy {}

impl<C> IntoGradientLch for C where 
    C: Sized + IntoColor<Lch> + FromColor<Lch> + Copy {}

impl<C> IntoGradientOklch for C where 
    C: Sized + IntoColor<Oklch> + FromColor<Oklch> + Copy {}

impl<C> IntoGradient for C where
    C: IntoGradientHsl + IntoGradientLch + IntoGradientOklch {}