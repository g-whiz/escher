use astro_float_num::{BigFloat, Consts, Radix, RoundingMode};

pub(crate) const PRECISION: usize = 256;
const ROUNDING: RoundingMode = RoundingMode::ToEven;

pub(crate) type Vector = [BigFloat; 3];

pub(crate) fn num(value: f64) -> BigFloat {
    BigFloat::from_f64(value, PRECISION)
}

pub(crate) fn int(value: i32) -> BigFloat {
    num(f64::from(value))
}

pub(crate) fn add(left: &BigFloat, right: &BigFloat) -> BigFloat {
    left.add(right, PRECISION, ROUNDING)
}

pub(crate) fn sub(left: &BigFloat, right: &BigFloat) -> BigFloat {
    left.sub(right, PRECISION, ROUNDING)
}

pub(crate) fn mul(left: &BigFloat, right: &BigFloat) -> BigFloat {
    left.mul(right, PRECISION, ROUNDING)
}

pub(crate) fn div(left: &BigFloat, right: &BigFloat) -> BigFloat {
    left.div(right, PRECISION, ROUNDING)
}

pub(crate) fn sqrt(value: &BigFloat) -> BigFloat {
    value.sqrt(PRECISION, ROUNDING)
}

pub(crate) fn max(left: &BigFloat, right: &BigFloat) -> BigFloat {
    if left >= right {
        left.clone()
    } else {
        right.clone()
    }
}

pub(crate) fn vector(values: [f64; 3]) -> Vector {
    values.map(num)
}

pub(crate) fn form(x: &Vector, y: &Vector) -> BigFloat {
    sub(
        &add(&mul(&x[0], &y[0]), &mul(&x[1], &y[1])),
        &mul(&x[2], &y[2]),
    )
}

pub(crate) fn scale(factor: &BigFloat, x: &Vector) -> Vector {
    [mul(factor, &x[0]), mul(factor, &x[1]), mul(factor, &x[2])]
}

pub(crate) fn difference(x: &Vector, y: &Vector) -> Vector {
    [sub(&x[0], &y[0]), sub(&x[1], &y[1]), sub(&x[2], &y[2])]
}

pub(crate) fn epsilon() -> BigFloat {
    num(f64::EPSILON)
}

pub(crate) fn error(value: f64, reference: &BigFloat) -> BigFloat {
    sub(&num(value), reference).abs()
}

pub(crate) struct Reference {
    consts: Consts,
}

impl Reference {
    pub(crate) fn new() -> Self {
        Self {
            consts: Consts::new().expect("the constant cache of the reference allocates"),
        }
    }

    pub(crate) fn pi(&mut self) -> BigFloat {
        self.consts.pi(PRECISION, ROUNDING)
    }

    pub(crate) fn cos(&mut self, value: &BigFloat) -> BigFloat {
        value.cos(PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn sin(&mut self, value: &BigFloat) -> BigFloat {
        value.sin(PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn cosh(&mut self, value: &BigFloat) -> BigFloat {
        value.cosh(PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn sinh(&mut self, value: &BigFloat) -> BigFloat {
        value.sinh(PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn acosh(&mut self, value: &BigFloat) -> BigFloat {
        value.acosh(PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn asinh(&mut self, value: &BigFloat) -> BigFloat {
        value.asinh(PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn parse(&mut self, text: &str) -> BigFloat {
        BigFloat::parse(text, Radix::Dec, PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn distance(&mut self, x: &Vector, y: &Vector) -> BigFloat {
        let c = form(x, y).neg();
        self.acosh(&max(&int(1), &c))
    }

    pub(crate) fn approximate(&mut self, value: &BigFloat) -> f64 {
        let text = value
            .format(Radix::Dec, ROUNDING, &mut self.consts)
            .expect("the reference formats a number");
        text.parse().unwrap_or(f64::NAN)
    }
}

pub(crate) struct Worst {
    name: &'static str,
    minimum: usize,
    count: usize,
    ratio: BigFloat,
    input: String,
}

impl Worst {
    pub(crate) fn new(name: &'static str, minimum: usize) -> Self {
        Self {
            name,
            minimum,
            count: 0,
            ratio: int(0),
            input: String::new(),
        }
    }

    pub(crate) fn record(
        &mut self,
        error: &BigFloat,
        scale: &BigFloat,
        input: impl FnOnce() -> String,
    ) {
        self.count += 1;
        let ratio = if error.is_zero() {
            int(0)
        } else {
            div(error, scale)
        };
        assert!(
            !ratio.is_nan(),
            "{}: the error is NaN for {}",
            self.name,
            input()
        );
        if ratio > self.ratio {
            self.ratio = ratio;
            self.input = input();
        }
    }

    pub(crate) fn assert_at_most(&self, reference: &mut Reference, bound: f64) {
        let ratio = reference.approximate(&self.ratio);
        println!(
            "{}: largest error {ratio:.3e} of the scale, bound {bound:e}, margin {:.1}, {} samples, at {}",
            self.name,
            bound / ratio,
            self.count,
            self.input
        );
        assert!(
            self.count >= self.minimum,
            "{}: measured {} samples, fewer than {}",
            self.name,
            self.count,
            self.minimum
        );
        assert!(
            self.ratio <= num(bound),
            "{}: largest error {ratio} of the scale exceeds {bound} at {}",
            self.name,
            self.input
        );
    }
}
