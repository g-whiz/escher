use astro_float_num::{BigFloat, Consts, Radix, RoundingMode};

pub(crate) const PRECISION: usize = 256;
const ROUNDING: RoundingMode = RoundingMode::ToEven;

pub(crate) type Vector = [BigFloat; 3];

pub(crate) type Matrix = [[BigFloat; 3]; 3];

pub(crate) fn num(value: f64) -> BigFloat {
    BigFloat::from_f64(value, PRECISION)
}

pub(crate) fn int(value: u32) -> BigFloat {
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

pub(crate) fn error(value: f64, reference: &BigFloat) -> BigFloat {
    sub(&num(value), reference).abs()
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

pub(crate) fn sum(x: &Vector, y: &Vector) -> Vector {
    [add(&x[0], &y[0]), add(&x[1], &y[1]), add(&x[2], &y[2])]
}

pub(crate) fn scale(factor: &BigFloat, x: &Vector) -> Vector {
    [mul(factor, &x[0]), mul(factor, &x[1]), mul(factor, &x[2])]
}

pub(crate) fn largest_magnitude(x: &Vector) -> BigFloat {
    max(&max(&x[0].abs(), &x[1].abs()), &x[2].abs())
}

pub(crate) fn matrix(columns: [f64; 9]) -> Matrix {
    let entry = |row: usize, column: usize| num(columns[3 * column + row]);
    [0, 1, 2].map(|row| [0, 1, 2].map(|column| entry(row, column)))
}

pub(crate) fn identity() -> Matrix {
    [0, 1, 2].map(|row| [0, 1, 2].map(|column| int(u32::from(row == column))))
}

pub(crate) fn product(left: &Matrix, right: &Matrix) -> Matrix {
    [0, 1, 2].map(|row| {
        [0, 1, 2].map(|column| {
            let first = mul(&left[row][0], &right[0][column]);
            let second = mul(&left[row][1], &right[1][column]);
            let third = mul(&left[row][2], &right[2][column]);
            add(&add(&first, &second), &third)
        })
    })
}

pub(crate) fn power(base: &Matrix, exponent: u32) -> Matrix {
    let mut result = identity();
    let mut square = base.clone();
    let mut remaining = exponent;
    while remaining > 0 {
        if remaining & 1 == 1 {
            result = product(&result, &square);
        }
        square = product(&square, &square);
        remaining >>= 1;
    }
    result
}

pub(crate) fn largest_entry_from_identity(matrix: &Matrix) -> BigFloat {
    let reference = identity();
    let mut largest = int(0);
    for row in 0..3 {
        for column in 0..3 {
            largest = max(
                &largest,
                &sub(&matrix[row][column], &reference[row][column]).abs(),
            );
        }
    }
    largest
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

    pub(crate) fn acosh(&mut self, value: &BigFloat) -> BigFloat {
        value.acosh(PRECISION, ROUNDING, &mut self.consts)
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

    pub(crate) fn triangle(&mut self, orders: [u32; 3]) -> Triangle {
        let pi = self.pi();
        let angles = orders.map(|order| div(&pi, &int(order)));
        let [s0, s1, s2] = [0, 1, 2].map(|index| self.sin(&angles[index]));
        let [c0, c1, c2] = [0, 1, 2].map(|index| self.cos(&angles[index]));
        let opposite_one = div(&add(&c1, &mul(&c2, &c0)), &mul(&s2, &s0));
        let opposite_two = div(&add(&c2, &mul(&c0, &c1)), &mul(&s0, &s1));
        let opposite_zero = div(&add(&c0, &mul(&c2, &c1)), &mul(&s2, &s1));
        let sinh_of = |cosh: &BigFloat| sqrt(&sub(&mul(cosh, cosh), &int(1)));
        let placed = [
            [sinh_of(&opposite_one), int(0), opposite_one.clone()],
            [
                mul(&c2, &sinh_of(&opposite_zero)),
                mul(&s2, &sinh_of(&opposite_zero)),
                opposite_zero.clone(),
            ],
            [int(0), int(0), int(1)],
        ];
        let total = sum(&sum(&placed[0], &placed[1]), &placed[2]);
        let centre = scale(&div(&int(1), &sqrt(&form(&total, &total).neg())), &total);
        let h = sqrt(&add(
            &mul(&centre[0], &centre[0]),
            &mul(&centre[1], &centre[1]),
        ));
        let (cos_phi, sin_phi) = (div(&centre[0], &h), div(&centre[1], &h));
        let (sinh_rho, cosh_rho) = (h, centre[2].clone());
        let move_to_origin = |x: &Vector| {
            let turned = [
                add(&mul(&cos_phi, &x[0]), &mul(&sin_phi, &x[1])),
                sub(&mul(&cos_phi, &x[1]), &mul(&sin_phi, &x[0])),
                x[2].clone(),
            ];
            let boosted = [
                sub(&mul(&cosh_rho, &turned[0]), &mul(&sinh_rho, &turned[2])),
                turned[1].clone(),
                sub(&mul(&cosh_rho, &turned[2]), &mul(&sinh_rho, &turned[0])),
            ];
            [
                sub(&mul(&cos_phi, &boosted[0]), &mul(&sin_phi, &boosted[1])),
                add(&mul(&sin_phi, &boosted[0]), &mul(&cos_phi, &boosted[1])),
                boosted[2].clone(),
            ]
        };
        let vertices = [
            move_to_origin(&placed[0]),
            move_to_origin(&placed[1]),
            move_to_origin(&placed[2]),
        ];
        let normals = [0, 1, 2].map(|index| {
            let first = &vertices[(index + 2) % 3];
            let second = &vertices[index];
            let cross = [
                sub(&mul(&first[1], &second[2]), &mul(&first[2], &second[1])),
                sub(&mul(&first[2], &second[0]), &mul(&first[0], &second[2])),
                sub(&mul(&first[0], &second[1]), &mul(&first[1], &second[0])),
            ];
            let normal = [cross[0].clone(), cross[1].clone(), cross[2].neg()];
            let unit = scale(&div(&int(1), &sqrt(&form(&normal, &normal))), &normal);
            if form(&unit, &vertices[(index + 1) % 3]).is_positive() {
                scale(&int(1).neg(), &unit)
            } else {
                unit
            }
        });
        let lengths = [
            self.acosh(&opposite_one),
            self.acosh(&opposite_two),
            self.acosh(&opposite_zero),
        ];
        let area = sub(&sub(&sub(&pi, &angles[0]), &angles[1]), &angles[2]);
        let standard_third_side = self.distance(&placed[0], &placed[1]);
        Triangle {
            vertices,
            normals,
            lengths,
            angles,
            area,
            standard_third_side,
        }
    }
}

pub(crate) struct Triangle {
    pub(crate) vertices: [Vector; 3],
    pub(crate) normals: [Vector; 3],
    pub(crate) lengths: [BigFloat; 3],
    pub(crate) angles: [BigFloat; 3],
    pub(crate) area: BigFloat,
    pub(crate) standard_third_side: BigFloat,
}

pub(crate) struct Worst {
    name: String,
    minimum: usize,
    count: usize,
    ratio: BigFloat,
    input: String,
}

impl Worst {
    pub(crate) fn new(name: impl Into<String>, minimum: usize) -> Self {
        Self {
            name: name.into(),
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
            "{}: largest error {ratio:.3e}, bound {bound:e}, margin {:.1}, {} samples, at {}",
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
            "{}: largest error {ratio} exceeds {bound} at {}",
            self.name,
            self.input
        );
    }
}

#[test]
#[cfg_attr(
    miri,
    ignore = "the high-precision arithmetic runs loops too long for the interpreter"
)]
fn reference_arithmetic_reproduces_known_values() {
    let mut reference = Reference::new();
    let tolerance = reference.parse("1e-70");
    let pi = reference.pi();
    let literal = reference.parse(
        "3.14159265358979323846264338327950288419716939937510582097494459230781640628620899",
    );
    let third = div(&pi, &int(3));
    let root = sqrt(&int(2));
    let inner = reference.acosh(&int(2));
    let checks = [
        ("pi", sub(&pi, &literal)),
        (
            "cos(pi / 3)",
            sub(&reference.cos(&third), &div(&int(1), &int(2))),
        ),
        ("cosh(acosh(2))", sub(&reference.cosh(&inner), &int(2))),
        ("sqrt(2)^2", sub(&mul(&root, &root), &int(2))),
    ];
    for (name, difference) in checks {
        let size = difference.abs();
        let shown = reference.approximate(&size);
        println!("{name}: error {shown:e}");
        assert!(size <= tolerance, "{name}: error {shown:e}");
    }
}
