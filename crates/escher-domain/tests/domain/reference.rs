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

    pub(crate) fn acos(&mut self, value: &BigFloat) -> BigFloat {
        value.acos(PRECISION, ROUNDING, &mut self.consts)
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

pub(crate) struct Polygon {
    pub(crate) vertices: Vec<Vector>,
    pub(crate) covectors: Vec<Vector>,
    pub(crate) pairings: Vec<Matrix>,
    pub(crate) lengths: Vec<BigFloat>,
    pub(crate) angles: Vec<BigFloat>,
    pub(crate) area: BigFloat,
    pub(crate) mean: [BigFloat; 2],
}

impl Reference {
    pub(crate) fn euclidean_triangle(&mut self, orders: [u32; 3]) -> Polygon {
        let pi = self.pi();
        let angles = orders.map(|order| div(&pi, &int(order)));
        let length = div(&self.sin(&angles[0]), &self.sin(&angles[1]));
        let placed = vec![
            [int(1), int(0)],
            [
                mul(&length, &self.cos(&angles[2])),
                mul(&length, &self.sin(&angles[2])),
            ],
            [int(0), int(0)],
        ];
        polygon(&placed, angles.to_vec())
    }

    pub(crate) fn rectangle(&mut self, aspect: f64) -> Polygon {
        let width = sqrt(&num(aspect));
        let height = div(&int(1), &width);
        let (x, y) = (div(&width, &int(2)), div(&height, &int(2)));
        let placed = vec![
            [x.clone(), y.neg()],
            [x.clone(), y.clone()],
            [x.neg(), y.clone()],
            [x.neg(), y.neg()],
        ];
        let right = div(&self.pi(), &int(2));
        polygon(&placed, vec![right; 4])
    }
}

fn polygon(placed: &[[BigFloat; 2]], angles: Vec<BigFloat>) -> Polygon {
    let count = int(u32::try_from(placed.len()).unwrap_or(0));
    let mut sum = [int(0), int(0)];
    for vertex in placed {
        sum = [add(&sum[0], &vertex[0]), add(&sum[1], &vertex[1])];
    }
    let mean = [div(&sum[0], &count), div(&sum[1], &count)];
    let vertices: Vec<Vector> = placed
        .iter()
        .map(|vertex| [sub(&vertex[0], &mean[0]), sub(&vertex[1], &mean[1]), int(1)])
        .collect();
    let total = vertices.len();
    let mut covectors = Vec::new();
    let mut pairings = Vec::new();
    let mut lengths = Vec::new();
    let mut twice_area = int(0);
    for index in 0..total {
        let (start, end) = (&vertices[(index + total - 1) % total], &vertices[index]);
        let along = [sub(&end[0], &start[0]), sub(&end[1], &start[1])];
        let length = sqrt(&add(&mul(&along[0], &along[0]), &mul(&along[1], &along[1])));
        let mut normal = [div(&along[1], &length), div(&along[0], &length).neg()];
        let mut offset = add(&mul(&normal[0], &end[0]), &mul(&normal[1], &end[1]));
        if offset.is_negative() {
            normal = [normal[0].neg(), normal[1].neg()];
            offset = offset.neg();
        }
        let entry = |row: usize, column: usize| {
            let identity = int(u32::from(row == column));
            match column {
                2 => add(&identity, &mul(&mul(&int(2), &offset), &normal[row])),
                _ => sub(
                    &identity,
                    &mul(&mul(&int(2), &normal[row]), &normal[column]),
                ),
            }
        };
        pairings.push([
            [entry(0, 0), entry(0, 1), entry(0, 2)],
            [entry(1, 0), entry(1, 1), entry(1, 2)],
            [int(0), int(0), int(1)],
        ]);
        covectors.push([normal[0].clone(), normal[1].clone(), offset.neg()]);
        lengths.push(length);
        twice_area = add(
            &twice_area,
            &sub(&mul(&start[0], &end[1]), &mul(&end[0], &start[1])),
        );
    }
    Polygon {
        vertices,
        covectors,
        pairings,
        lengths,
        angles,
        area: div(&twice_area, &int(2)).abs(),
        mean,
    }
}

pub(crate) fn dot(x: &Vector, y: &Vector) -> BigFloat {
    add(
        &add(&mul(&x[0], &y[0]), &mul(&x[1], &y[1])),
        &mul(&x[2], &y[2]),
    )
}

fn unit(x: &Vector) -> Vector {
    scale(&div(&int(1), &sqrt(&dot(x, x))), x)
}

fn without(x: &Vector, direction: &Vector) -> Vector {
    let along = scale(&dot(x, direction), direction);
    [
        sub(&x[0], &along[0]),
        sub(&x[1], &along[1]),
        sub(&x[2], &along[2]),
    ]
}

pub(crate) struct SphericalPolygon {
    pub(crate) vertices: Vec<Vector>,
    pub(crate) normals: Vec<Vector>,
    pub(crate) lengths: Vec<BigFloat>,
    pub(crate) angles: Vec<BigFloat>,
    pub(crate) area: BigFloat,
    pub(crate) centre: Vector,
}

impl Reference {
    pub(crate) fn spherical_triangle(&mut self, orders: [u32; 3]) -> SphericalPolygon {
        let pi = self.pi();
        let angles = orders.map(|order| div(&pi, &int(order)));
        let [s0, s1, s2] = [0, 1, 2].map(|index| self.sin(&angles[index]));
        let [c0, c1, c2] = [0, 1, 2].map(|index| self.cos(&angles[index]));
        let opposite_one = div(&add(&c1, &mul(&c2, &c0)), &mul(&s2, &s0));
        let opposite_two = div(&add(&c2, &mul(&c0, &c1)), &mul(&s0, &s1));
        let opposite_zero = div(&add(&c0, &mul(&c2, &c1)), &mul(&s2, &s1));
        let sin_of = |cos: &BigFloat| sqrt(&sub(&int(1), &mul(cos, cos)));
        let placed = [
            [sin_of(&opposite_one), int(0), opposite_one.clone()],
            [
                mul(&c2, &sin_of(&opposite_zero)),
                mul(&s2, &sin_of(&opposite_zero)),
                opposite_zero.clone(),
            ],
            [int(0), int(0), int(1)],
        ];
        let target = unit(&sum(&sum(&placed[0], &placed[1]), &placed[2]));
        let h = sqrt(&add(
            &mul(&target[0], &target[0]),
            &mul(&target[1], &target[1]),
        ));
        let (cos_phi, sin_phi) = (div(&target[0], &h), div(&target[1], &h));
        let (sin_rho, cos_rho) = (h, target[2].clone());
        let move_to_origin = |x: &Vector| {
            let turned = [
                add(&mul(&cos_phi, &x[0]), &mul(&sin_phi, &x[1])),
                sub(&mul(&cos_phi, &x[1]), &mul(&sin_phi, &x[0])),
                x[2].clone(),
            ];
            let moved = [
                sub(&mul(&cos_rho, &turned[0]), &mul(&sin_rho, &turned[2])),
                turned[1].clone(),
                add(&mul(&cos_rho, &turned[2]), &mul(&sin_rho, &turned[0])),
            ];
            [
                sub(&mul(&cos_phi, &moved[0]), &mul(&sin_phi, &moved[1])),
                add(&mul(&sin_phi, &moved[0]), &mul(&cos_phi, &moved[1])),
                moved[2].clone(),
            ]
        };
        let vertices: Vec<Vector> = placed.iter().map(move_to_origin).collect();
        let normals = (0..3)
            .map(|index| {
                let first = unit(&vertices[(index + 2) % 3]);
                let second = unit(&without(&vertices[index], &first));
                let off = without(&without(&vertices[(index + 1) % 3], &first), &second);
                scale(&int(1).neg(), &unit(&off))
            })
            .collect();
        let lengths = vec![
            self.acos(&opposite_one),
            self.acos(&opposite_two),
            self.acos(&opposite_zero),
        ];
        let area = sub(&add(&add(&angles[0], &angles[1]), &angles[2]), &pi);
        let centre = unit(&sum(&sum(&vertices[0], &vertices[1]), &vertices[2]));
        SphericalPolygon {
            vertices,
            normals,
            lengths,
            angles: angles.to_vec(),
            area,
            centre,
        }
    }

    pub(crate) fn lune(&mut self, order: u32) -> SphericalPolygon {
        let pi = self.pi();
        let angle = div(&pi, &int(order));
        let half = div(&angle, &int(2));
        let (sin, cos) = (self.sin(&half), self.cos(&half));
        SphericalPolygon {
            vertices: vec![[int(0), int(1), int(0)], [int(0), int(1).neg(), int(0)]],
            normals: vec![
                [cos.clone(), int(0), sin.neg()],
                [cos.neg(), int(0), sin.neg()],
            ],
            lengths: vec![pi.clone(), pi],
            angles: vec![angle.clone(), angle.clone()],
            area: mul(&int(2), &angle),
            centre: [int(0), int(0), int(1)],
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
