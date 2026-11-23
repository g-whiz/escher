use astro_float_num::BigFloat;
use escher_reference::{
    Complex, Reference, add, complex_mul, complex_sub, div, int, mul, num, sqrt, squared_modulus,
    sub,
};

const STEP_EXPONENT: i32 = -100;

pub(crate) fn epsilon() -> BigFloat {
    num(f64::EPSILON)
}

pub(crate) fn complex(value: [f64; 2]) -> Complex {
    value.map(num)
}

pub(crate) fn central_jacobian(
    map: &mut dyn FnMut(&Complex) -> Complex,
    position: &Complex,
) -> [Complex; 2] {
    let step = num(2.0_f64.powi(STEP_EXPONENT));
    let width = mul(&int(2), &step);
    [0, 1].map(|coordinate| {
        let mut ahead = position.clone();
        let mut behind = position.clone();
        ahead[coordinate] = add(&ahead[coordinate], &step);
        behind[coordinate] = sub(&behind[coordinate], &step);
        let change = complex_sub(&map(&ahead), &map(&behind));
        [div(&change[0], &width), div(&change[1], &width)]
    })
}

pub(crate) trait ReferenceExt {
    fn band_scale_factor(&mut self, w: &Complex) -> BigFloat;
}

impl ReferenceExt for Reference {
    fn band_scale_factor(&mut self, w: &Complex) -> BigFloat {
        let tanh = self.band_to_disk(w);
        let one = [int(1), int(0)];
        let difference = complex_sub(&one, &complex_mul(&tanh, &tanh));
        mul(
            &div(&self.pi(), &int(4)),
            &sqrt(&squared_modulus(&difference)),
        )
    }
}
