#![expect(
    dead_code,
    reason = "the unit tests of the crate and the integration tests each use a different part of this module"
)]

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

pub(crate) fn matrix(columns: [f64; 9]) -> Matrix {
    let entry = |row: usize, column: usize| num(columns[3 * column + row]);
    [0, 1, 2].map(|row| [0, 1, 2].map(|column| entry(row, column)))
}

pub(crate) fn dot(x: &Vector, y: &Vector) -> BigFloat {
    add(
        &add(&mul(&x[0], &y[0]), &mul(&x[1], &y[1])),
        &mul(&x[2], &y[2]),
    )
}

pub(crate) fn form(x: &Vector, y: &Vector) -> BigFloat {
    sub(
        &add(&mul(&x[0], &y[0]), &mul(&x[1], &y[1])),
        &mul(&x[2], &y[2]),
    )
}

pub(crate) fn apply(matrix: &Matrix, x: &Vector) -> Vector {
    [0, 1, 2].map(|row| {
        let first = mul(&matrix[row][0], &x[0]);
        let second = mul(&matrix[row][1], &x[1]);
        let third = mul(&matrix[row][2], &x[2]);
        add(&add(&first, &second), &third)
    })
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

pub(crate) fn scale(factor: &BigFloat, x: &Vector) -> Vector {
    [mul(factor, &x[0]), mul(factor, &x[1]), mul(factor, &x[2])]
}

pub(crate) fn difference(x: &Vector, y: &Vector) -> Vector {
    [sub(&x[0], &y[0]), sub(&x[1], &y[1]), sub(&x[2], &y[2])]
}

pub(crate) fn largest_magnitude(x: &Vector) -> BigFloat {
    max(&max(&x[0].abs(), &x[1].abs()), &x[2].abs())
}

pub(crate) fn normalize(x: &Vector) -> Vector {
    scale(&div(&int(1), &sqrt(&form(x, x).neg())), x)
}

pub(crate) fn from_disk(position: [f64; 2]) -> Vector {
    let [first, second] = position.map(num);
    from_disk_exact(&first, &second)
}

pub(crate) fn from_disk_exact(first: &BigFloat, second: &BigFloat) -> Vector {
    let squared = add(&mul(first, first), &mul(second, second));
    let denominator = sub(&int(1), &squared);
    [
        div(&mul(&int(2), first), &denominator),
        div(&mul(&int(2), second), &denominator),
        div(&add(&int(1), &squared), &denominator),
    ]
}

pub(crate) fn plane_distance(x: &Vector, y: &Vector) -> BigFloat {
    let first = sub(&div(&x[0], &x[2]), &div(&y[0], &y[2]));
    let second = sub(&div(&x[1], &x[2]), &div(&y[1], &y[2]));
    sqrt(&add(&mul(&first, &first), &mul(&second, &second)))
}

pub(crate) fn plane_wall_distance(covector: &Vector, point: &Vector) -> BigFloat {
    div(&dot(covector, point), &point[2]).abs()
}

pub(crate) fn plane_border_distance(covectors: &[Vector], point: [f64; 3], pixel: f64) -> BigFloat {
    let point = vector(point);
    let mut nearest: Option<BigFloat> = None;
    for covector in covectors {
        let pixels = div(&plane_wall_distance(covector, &point), &num(pixel));
        nearest = Some(match nearest {
            Some(nearest) if nearest <= pixels => nearest,
            _ => pixels,
        });
    }
    nearest.expect("a domain has sides")
}

pub(crate) struct Wall {
    pub(crate) covector: Vector,
    pub(crate) pairing: Matrix,
}

pub(crate) struct ExactFold {
    pub(crate) sides: Vec<usize>,
    pub(crate) converged: bool,
    pub(crate) point: Vector,
}

pub(crate) fn fold(walls: &[Wall], point: [f64; 3], cap: u32) -> ExactFold {
    let mut x = vector(point);
    let mut sides = Vec::new();
    loop {
        let violated = walls
            .iter()
            .position(|wall| dot(&wall.covector, &x).is_positive());
        let Some(index) = violated else {
            return ExactFold {
                sides,
                converged: true,
                point: x,
            };
        };
        if sides.len() == usize::try_from(cap).unwrap_or(usize::MAX) {
            return ExactFold {
                sides,
                converged: false,
                point: x,
            };
        }
        x = apply(&walls[index].pairing, &x);
        sides.push(index);
    }
}

pub(crate) fn cross(x: &Vector, y: &Vector) -> Vector {
    [
        sub(&mul(&x[1], &y[2]), &mul(&x[2], &y[1])),
        sub(&mul(&x[2], &y[0]), &mul(&x[0], &y[2])),
        sub(&mul(&x[0], &y[1]), &mul(&x[1], &y[0])),
    ]
}

pub(crate) fn length(x: &Vector) -> BigFloat {
    sqrt(&dot(x, x))
}

pub(crate) fn transpose_apply(matrix: &Matrix, x: &Vector) -> Vector {
    [0, 1, 2].map(|column| {
        let first = mul(&matrix[0][column], &x[0]);
        let second = mul(&matrix[1][column], &x[1]);
        let third = mul(&matrix[2][column], &x[2]);
        add(&add(&first, &second), &third)
    })
}

pub(crate) fn screen_point_exact(first: &BigFloat, second: &BigFloat) -> Vector {
    let squared = add(&mul(first, first), &mul(second, second));
    [first.clone(), second.clone(), sqrt(&sub(&int(1), &squared))]
}

pub(crate) fn replay(walls: &[Wall], point: [f64; 3], sides: &[usize]) -> Vector {
    let mut x = vector(point);
    for &index in sides {
        x = apply(&walls[index].pairing, &x);
    }
    x
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

    pub(crate) fn atan(&mut self, value: &BigFloat) -> BigFloat {
        value.atan(PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn angle(&mut self, x: &Vector, y: &Vector) -> BigFloat {
        let across = length(&cross(x, y));
        let along = dot(x, y);
        if across.is_zero() {
            return int(0);
        }
        assert!(
            along.is_positive(),
            "the reference measures only angles below a quarter turn"
        );
        self.atan(&div(&across, &along))
    }

    pub(crate) fn viewing(&mut self, [yaw, pitch, roll]: [f64; 3]) -> Matrix {
        let [yaw, pitch, roll] = [yaw, pitch, roll].map(num);
        let (sin_yaw, cos_yaw) = (self.sin(&yaw), self.cos(&yaw));
        let (sin_pitch, cos_pitch) = (self.sin(&pitch), self.cos(&pitch));
        let (sin_roll, cos_roll) = (self.sin(&roll), self.cos(&roll));
        let zero = int(0);
        let one = int(1);
        let turn_yaw = [
            [cos_yaw.clone(), zero.clone(), sin_yaw.clone()],
            [zero.clone(), one.clone(), zero.clone()],
            [sin_yaw.neg(), zero.clone(), cos_yaw],
        ];
        let turn_pitch = [
            [one.clone(), zero.clone(), zero.clone()],
            [zero.clone(), cos_pitch.clone(), sin_pitch.clone()],
            [zero.clone(), sin_pitch.neg(), cos_pitch],
        ];
        let turn_roll = [
            [cos_roll.clone(), sin_roll.neg(), zero.clone()],
            [sin_roll, cos_roll, zero.clone()],
            [zero.clone(), zero, one],
        ];
        product(&turn_roll, &product(&turn_pitch, &turn_yaw))
    }

    pub(crate) fn lift(&mut self, position: [f64; 2], angles: [f64; 3]) -> Vector {
        let [first, second] = position.map(num);
        let viewing = self.viewing(angles);
        transpose_apply(&viewing, &screen_point_exact(&first, &second))
    }

    pub(crate) fn ball_tangents(&mut self, position: [f64; 2], pixel: f64) -> [Vector; 2] {
        let step = self.parse("1e-40");
        let half = div(&num(pixel), &mul(&int(2), &step));
        let [first, second] = position.map(num);
        let partial = |plus: Vector, minus: Vector| scale(&half, &difference(&plus, &minus));
        [
            partial(
                screen_point_exact(&add(&first, &step), &second),
                screen_point_exact(&sub(&first, &step), &second),
            ),
            partial(
                screen_point_exact(&first, &add(&second, &step)),
                screen_point_exact(&first, &sub(&second, &step)),
            ),
        ]
    }

    pub(crate) fn shade(&mut self, normal: [f64; 3], light: [f64; 3], channel: f64) -> BigFloat {
        let normal = vector(normal);
        let light = vector(light);
        let channel = num(channel);
        let facing = dot(&normal, &light);
        let diffuse = max(&facing, &int(0));
        let highlight = if facing.is_positive() {
            let reflected = sub(&mul(&mul(&int(2), &facing), &normal[2]), &light[2]);
            (0..5).fold(max(&reflected, &int(0)), |power, _| mul(&power, &power))
        } else {
            int(0)
        };
        let ambient = self.parse("0.25");
        let specular_weight = self.parse("0.25");
        let weight = add(&ambient, &mul(&self.parse("0.75"), &diffuse));
        let specular = mul(&mul(&specular_weight, &highlight), &sub(&int(1), &channel));
        add(&mul(&channel, &weight), &specular)
    }

    pub(crate) fn sinh(&mut self, value: &BigFloat) -> BigFloat {
        value.sinh(PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn asinh(&mut self, value: &BigFloat) -> BigFloat {
        value.asinh(PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn acosh(&mut self, value: &BigFloat) -> BigFloat {
        value.acosh(PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn pow(&mut self, base: &BigFloat, exponent: &BigFloat) -> BigFloat {
        base.pow(exponent, PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn parse(&mut self, text: &str) -> BigFloat {
        BigFloat::parse(text, Radix::Dec, PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn approximate(&mut self, value: &BigFloat) -> f64 {
        let text = value
            .format(Radix::Dec, ROUNDING, &mut self.consts)
            .expect("the reference formats a number");
        text.parse().unwrap_or(f64::NAN)
    }

    pub(crate) fn distance(&mut self, x: &Vector, y: &Vector) -> BigFloat {
        let c = form(&normalize(x), &normalize(y)).neg();
        self.acosh(&max(&int(1), &c))
    }

    pub(crate) fn wall_distance(&mut self, covector: &Vector, point: &Vector) -> BigFloat {
        self.asinh(&dot(covector, &normalize(point)).abs())
    }

    pub(crate) fn sphere_wall_distance(&mut self, normal: &Vector, point: &Vector) -> BigFloat {
        let sine = div(
            &dot(normal, point).abs(),
            &mul(&length(normal), &length(point)),
        );
        let cosine = sqrt(&sub(&int(1), &mul(&sine, &sine)));
        if cosine.is_zero() {
            return div(&self.pi(), &int(2));
        }
        self.atan(&div(&sine, &cosine))
    }

    pub(crate) fn border_distance(
        &mut self,
        covectors: &[Vector],
        point: [f64; 3],
        position: [f64; 2],
        pixel: f64,
    ) -> BigFloat {
        let point = vector(point);
        let [first, second] = position.map(num);
        let squared = add(&mul(&first, &first), &mul(&second, &second));
        let factor = div(&sub(&int(1), &squared), &mul(&int(2), &num(pixel)));
        let mut nearest: Option<BigFloat> = None;
        for covector in covectors {
            let pixels = mul(&self.wall_distance(covector, &point), &factor);
            nearest = Some(match nearest {
                Some(nearest) if nearest <= pixels => nearest,
                _ => pixels,
            });
        }
        nearest.expect("a domain has sides")
    }

    pub(crate) fn tangents(&mut self, position: [f64; 2], pixel: f64) -> [Vector; 2] {
        let step = self.parse("1e-40");
        let half = div(&num(pixel), &mul(&int(2), &step));
        let [first, second] = position.map(num);
        let partial = |plus: Vector, minus: Vector| scale(&half, &difference(&plus, &minus));
        [
            partial(
                from_disk_exact(&add(&first, &step), &second),
                from_disk_exact(&sub(&first, &step), &second),
            ),
            partial(
                from_disk_exact(&first, &add(&second, &step)),
                from_disk_exact(&first, &sub(&second, &step)),
            ),
        ]
    }

    pub(crate) fn srgb_decode(&mut self, byte: u8) -> BigFloat {
        let encoded = div(&int(u32::from(byte)), &int(255));
        self.srgb_decode_unit(&encoded)
    }

    pub(crate) fn srgb_decode_unit(&mut self, encoded: &BigFloat) -> BigFloat {
        if *encoded <= self.parse("0.04045") {
            div(encoded, &self.parse("12.92"))
        } else {
            let base = div(&add(encoded, &self.parse("0.055")), &self.parse("1.055"));
            let exponent = self.parse("2.4");
            self.pow(&base, &exponent)
        }
    }

    pub(crate) fn srgb_encode(&mut self, linear: &BigFloat) -> BigFloat {
        if *linear <= self.parse("0.0031308") {
            mul(&self.parse("12.92"), linear)
        } else {
            let exponent = div(&int(1), &self.parse("2.4"));
            let power = self.pow(linear, &exponent);
            sub(&mul(&self.parse("1.055"), &power), &self.parse("0.055"))
        }
    }
}

pub(crate) fn count_bound(measured: usize) -> usize {
    (8 * measured).max(8)
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

    pub(crate) fn largest(&self, reference: &mut Reference) -> f64 {
        reference.approximate(&self.ratio)
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
fn reference_arithmetic_reproduces_known_values() {
    let mut reference = Reference::new();
    let tolerance = reference.parse("1e-70");
    let pi = reference.pi();
    let literal = reference.parse(
        "3.14159265358979323846264338327950288419716939937510582097494459230781640628620899",
    );
    let third = div(&pi, &int(3));
    let root = sqrt(&int(2));
    let one = int(1);
    let asinh_one = reference.parse(
        "0.88137358701954302523260932497979230902816032826163541075329560865337718422202608",
    );
    let half = div(&one, &int(2));
    let power = reference.pow(&int(2), &half);
    let e = reference.parse(
        "2.71828182845904523536028747135266249775724709369995957496696762772407663035354759",
    );
    let cosh_one = div(&add(&e, &div(&one, &e)), &int(2));
    let checks = [
        ("pi", sub(&pi, &literal)),
        ("cos(pi / 3)", sub(&reference.cos(&third), &half)),
        ("sqrt(2)^2", sub(&mul(&root, &root), &int(2))),
        ("asinh(1)", sub(&reference.asinh(&one), &asinh_one)),
        ("sinh(asinh(1))", sub(&reference.sinh(&asinh_one), &one)),
        ("acosh(cosh(1))", sub(&reference.acosh(&cosh_one), &one)),
        ("2^(1/2)", sub(&power, &root)),
    ];
    for (name, difference) in checks {
        let size = difference.abs();
        let shown = reference.approximate(&size);
        println!("{name}: error {shown:e}");
        assert!(size <= tolerance, "{name}: error {shown:e}");
    }
}
