use astro_float_num::BigFloat;
use escher_reference::{Reference, add, div, int, mul, num, sqrt, sub};

pub(crate) type Complex = [BigFloat; 2];

const STEP_EXPONENT: i32 = -100;

pub(crate) fn epsilon() -> BigFloat {
    num(f64::EPSILON)
}

pub(crate) fn complex(value: [f64; 2]) -> Complex {
    value.map(num)
}

fn complex_add(left: &Complex, right: &Complex) -> Complex {
    [add(&left[0], &right[0]), add(&left[1], &right[1])]
}

fn complex_sub(left: &Complex, right: &Complex) -> Complex {
    [sub(&left[0], &right[0]), sub(&left[1], &right[1])]
}

fn complex_mul(left: &Complex, right: &Complex) -> Complex {
    [
        sub(&mul(&left[0], &right[0]), &mul(&left[1], &right[1])),
        add(&mul(&left[0], &right[1]), &mul(&left[1], &right[0])),
    ]
}

fn complex_div(left: &Complex, right: &Complex) -> Complex {
    let norm = squared_modulus(right);
    [
        div(
            &add(&mul(&left[0], &right[0]), &mul(&left[1], &right[1])),
            &norm,
        ),
        div(
            &sub(&mul(&left[1], &right[0]), &mul(&left[0], &right[1])),
            &norm,
        ),
    ]
}

fn complex_scale(factor: &BigFloat, value: &Complex) -> Complex {
    [mul(factor, &value[0]), mul(factor, &value[1])]
}

pub(crate) fn squared_modulus(value: &Complex) -> BigFloat {
    add(&mul(&value[0], &value[0]), &mul(&value[1], &value[1]))
}

fn one() -> Complex {
    [int(1), int(0)]
}

fn i() -> Complex {
    [int(0), int(1)]
}

pub(crate) fn half_plane_to_disk(w: &Complex) -> Complex {
    complex_div(
        &complex_add(&complex_mul(&i(), w), &one()),
        &complex_add(w, &i()),
    )
}

pub(crate) fn disk_to_half_plane(z: &Complex) -> Complex {
    complex_div(
        &complex_add(z, &i()),
        &complex_add(&complex_mul(&i(), z), &one()),
    )
}

pub(crate) fn klein_to_disk(s: &Complex) -> Complex {
    let root = sqrt(&sub(&int(1), &squared_modulus(s)));
    complex_scale(&div(&int(1), &add(&int(1), &root)), s)
}

pub(crate) fn disk_to_klein(z: &Complex) -> Complex {
    complex_scale(&div(&int(2), &add(&int(1), &squared_modulus(z))), z)
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
    fn complex_tanh(&mut self, value: &Complex) -> Complex;

    fn log_of_right_half(&mut self, value: &Complex) -> Complex;

    fn band_to_disk(&mut self, w: &Complex) -> Complex;

    fn band_scale_factor(&mut self, w: &Complex) -> BigFloat;

    fn disk_to_band(&mut self, z: &Complex) -> Complex;
}

impl ReferenceExt for Reference {
    fn complex_tanh(&mut self, value: &Complex) -> Complex {
        let [x, y] = value;
        let (sinh, cosh) = (self.sinh(x), self.cosh(x));
        let (sin, cos) = (self.sin(y), self.cos(y));
        let numerator = [mul(&sinh, &cos), mul(&cosh, &sin)];
        let denominator = [mul(&cosh, &cos), mul(&sinh, &sin)];
        complex_div(&numerator, &denominator)
    }

    fn log_of_right_half(&mut self, value: &Complex) -> Complex {
        let [x, y] = value;
        assert!(
            *x > int(0),
            "internal error: the logarithm of a value with no positive real part"
        );
        let modulus = div(&self.ln(&squared_modulus(value)), &int(2));
        [modulus, self.atan(&div(y, x))]
    }

    fn band_to_disk(&mut self, w: &Complex) -> Complex {
        let quarter_pi = div(&self.pi(), &int(4));
        self.complex_tanh(&complex_scale(&quarter_pi, w))
    }

    fn band_scale_factor(&mut self, w: &Complex) -> BigFloat {
        let tanh = self.band_to_disk(w);
        let difference = complex_sub(&one(), &complex_mul(&tanh, &tanh));
        mul(
            &div(&self.pi(), &int(4)),
            &sqrt(&squared_modulus(&difference)),
        )
    }

    fn disk_to_band(&mut self, z: &Complex) -> Complex {
        let ahead = self.log_of_right_half(&complex_add(&one(), z));
        let behind = self.log_of_right_half(&complex_sub(&one(), z));
        let factor = div(&int(2), &self.pi());
        complex_scale(&factor, &complex_sub(&ahead, &behind))
    }
}
