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

    pub(crate) fn sinh(&mut self, value: &BigFloat) -> BigFloat {
        value.sinh(PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn asinh(&mut self, value: &BigFloat) -> BigFloat {
        value.asinh(PRECISION, ROUNDING, &mut self.consts)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Model {
    Hyperbolic,
    Euclidean,
    Spherical,
}

pub(crate) struct Double {
    pub(crate) vertices: Vec<Vector>,
    pub(crate) walls: Vec<Vector>,
    pub(crate) lengths: Vec<BigFloat>,
    pub(crate) area: BigFloat,
}

fn reflected_point(model: Model, wall: &Vector, x: &Vector) -> Vector {
    match model {
        Model::Hyperbolic => without_twice(x, &form(x, wall), wall),
        Model::Spherical => without_twice(x, &dot(x, wall), wall),
        Model::Euclidean => without_twice(
            x,
            &dot(wall, x),
            &[wall[0].clone(), wall[1].clone(), int(0)],
        ),
    }
}

fn reflected_wall(model: Model, wall: &Vector, x: &Vector) -> Vector {
    match model {
        Model::Hyperbolic | Model::Spherical => reflected_point(model, wall, x),
        Model::Euclidean => {
            let along = add(&mul(&x[0], &wall[0]), &mul(&x[1], &wall[1]));
            without_twice(x, &along, wall)
        }
    }
}

fn without_twice(x: &Vector, amount: &BigFloat, direction: &Vector) -> Vector {
    let twice = mul(&int(2), amount);
    [0, 1, 2].map(|index| sub(&x[index], &mul(&twice, &direction[index])))
}

impl Reference {
    pub(crate) fn double(
        &mut self,
        model: Model,
        vertices: &[Vector],
        walls: &[Vector],
        generating_area: &BigFloat,
    ) -> Double {
        let count = vertices.len();
        let wall = &walls[1];
        let mut doubled = vec![vertices[0].clone()];
        for index in 1..count - 1 {
            doubled.push(reflected_point(model, wall, &vertices[count - index]));
        }
        doubled.push(vertices[1].clone());
        for index in 1..count - 1 {
            doubled.push(vertices[1 + index].clone());
        }
        let total = doubled.len();
        let sides = (0..total)
            .map(|index| {
                if index == 0 {
                    walls[0].clone()
                } else if index < count {
                    reflected_wall(model, wall, &walls[(count + 1 - index) % count])
                } else {
                    walls[index + 2 - count].clone()
                }
            })
            .collect();
        let lengths = (0..total)
            .map(|index| {
                let (start, end) = (&doubled[(index + total - 1) % total], &doubled[index]);
                match model {
                    Model::Hyperbolic => self.distance(start, end),
                    Model::Spherical => self.acos(&dot(start, end)),
                    Model::Euclidean => {
                        let along = [sub(&end[0], &start[0]), sub(&end[1], &start[1])];
                        sqrt(&add(&mul(&along[0], &along[0]), &mul(&along[1], &along[1])))
                    }
                }
            })
            .collect();
        let area = match model {
            Model::Hyperbolic | Model::Spherical => mul(&int(2), generating_area),
            Model::Euclidean => {
                let mut twice = int(0);
                for index in 0..total {
                    let (start, end) = (&doubled[(index + total - 1) % total], &doubled[index]);
                    twice = add(
                        &twice,
                        &sub(&mul(&start[0], &end[1]), &mul(&end[0], &start[1])),
                    );
                }
                div(&twice, &int(2)).abs()
            }
        };
        Double {
            vertices: doubled,
            walls: sides,
            lengths,
            area,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FeatureKind {
    Corner,
    Length(usize),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Feature {
    pub(crate) kind: FeatureKind,
    pub(crate) first: usize,
    pub(crate) last: usize,
}

pub(crate) struct Cuts {
    pub(crate) pairs: usize,
    pub(crate) sheets: Vec<[Feature; 3]>,
    pub(crate) slice_walls: Vec<(usize, usize)>,
}

impl Cuts {
    pub(crate) fn of(orders: &[u32]) -> Self {
        let count = orders.len();
        let mut list = std::collections::VecDeque::new();
        let mut pairs = 0;
        let mut corner = 0;
        while corner < count {
            if orders[corner] == 2 && corner + 1 < count && orders[corner + 1] == 2 {
                list.push_back(Feature {
                    kind: FeatureKind::Length(pairs),
                    first: corner,
                    last: corner + 1,
                });
                pairs += 1;
                corner += 2;
            } else {
                list.push_back(Feature {
                    kind: FeatureKind::Corner,
                    first: corner,
                    last: corner,
                });
                corner += 1;
            }
        }
        let mut sheets = Vec::new();
        let mut slice_walls = Vec::new();
        while list.len() > 3 {
            let (Some(x), Some(y)) = (list.pop_back(), list.pop_back()) else {
                panic!("{orders:?}: the list is shorter than it says");
            };
            let kind = FeatureKind::Length(pairs + slice_walls.len());
            slice_walls.push((y.first, (x.last + 1) % count));
            sheets.push([
                y,
                x,
                Feature {
                    kind,
                    first: (x.last + 1) % count,
                    last: (y.first + count - 1) % count,
                },
            ]);
            list.push_front(Feature {
                kind,
                first: y.first,
                last: x.last,
            });
        }
        sheets.push([list[0], list[1], list[2]]);
        Self {
            pairs,
            sheets,
            slice_walls,
        }
    }
}

pub(crate) struct ReflectionPolygon {
    pub(crate) walk: Vec<Vector>,
    pub(crate) vertices: Vec<Vector>,
    pub(crate) normals: Vec<Vector>,
    pub(crate) lengths: Vec<BigFloat>,
    pub(crate) angles: Vec<BigFloat>,
}

pub(crate) struct Inscribed {
    pub(crate) radius: BigFloat,
    pub(crate) vertices: Vec<Vector>,
}

pub(crate) struct Weld {
    pub(crate) vertices: Vec<Vector>,
    pub(crate) closures: Vec<BigFloat>,
    pub(crate) slice_ends: Vec<BigFloat>,
}

struct Trig {
    corner: bool,
    cos: BigFloat,
    sin: BigFloat,
}

struct LaidOut {
    points: Vec<(Feature, usize, Vector, Matrix)>,
    closure: BigFloat,
}

const POINT_STEPS: u32 = 170;

pub(crate) fn boost_matrix(cosh: &BigFloat, sinh: &BigFloat) -> Matrix {
    [
        [cosh.clone(), int(0), sinh.clone()],
        [int(0), int(1), int(0)],
        [sinh.clone(), int(0), cosh.clone()],
    ]
}

pub(crate) fn rotation_matrix(cos: &BigFloat, sin: &BigFloat) -> Matrix {
    [
        [cos.clone(), sin.neg(), int(0)],
        [sin.clone(), cos.clone(), int(0)],
        [int(0), int(0), int(1)],
    ]
}

pub(crate) fn isometry_inverse(matrix: &Matrix) -> Matrix {
    [0, 1, 2].map(|row| {
        [0, 1, 2].map(|column| {
            let entry = matrix[column][row].clone();
            if (row == 2) == (column == 2) {
                entry
            } else {
                entry.neg()
            }
        })
    })
}

pub(crate) fn apply(matrix: &Matrix, x: &Vector) -> Vector {
    [0, 1, 2].map(|row| {
        add(
            &add(&mul(&matrix[row][0], &x[0]), &mul(&matrix[row][1], &x[1])),
            &mul(&matrix[row][2], &x[2]),
        )
    })
}

fn origin() -> Vector {
    [int(0), int(0), int(1)]
}

fn column_two(matrix: &Matrix) -> Vector {
    [
        matrix[0][2].clone(),
        matrix[1][2].clone(),
        matrix[2][2].clone(),
    ]
}

pub(crate) fn hyperbolic_normal(start: &Vector, end: &Vector) -> Vector {
    let cross = [
        sub(&mul(&start[1], &end[2]), &mul(&start[2], &end[1])),
        sub(&mul(&start[2], &end[0]), &mul(&start[0], &end[2])),
        sub(&mul(&start[0], &end[1]), &mul(&start[1], &end[0])),
    ];
    let normal = [cross[0].clone(), cross[1].clone(), cross[2].neg()];
    let unit = scale(&div(&int(1), &sqrt(&form(&normal, &normal))), &normal);
    if form(&unit, &origin()).is_positive() {
        scale(&int(1).neg(), &unit)
    } else {
        unit
    }
}

fn normalized(x: &Vector) -> Vector {
    scale(&div(&int(1), &sqrt(&form(x, x).neg())), x)
}

fn golden_minimum(
    steps: u32,
    mut low: BigFloat,
    mut high: BigFloat,
    mut value: impl FnMut(&BigFloat) -> BigFloat,
) -> BigFloat {
    let ratio = div(&sub(&sqrt(&int(5)), &int(1)), &int(2));
    let mut left = sub(&high, &mul(&ratio, &sub(&high, &low)));
    let mut right = add(&low, &mul(&ratio, &sub(&high, &low)));
    let (mut at_left, mut at_right) = (value(&left), value(&right));
    for _ in 0..steps {
        if at_left < at_right {
            high = right;
            right = left;
            at_right = at_left;
            left = sub(&high, &mul(&ratio, &sub(&high, &low)));
            at_left = value(&left);
        } else {
            low = left;
            left = right;
            at_left = at_right;
            right = add(&low, &mul(&ratio, &sub(&high, &low)));
            at_right = value(&right);
        }
    }
    if at_left < at_right {
        at_left
    } else {
        at_right
    }
}

fn geodesic(start: &Vector, end: &Vector) -> (Vector, Vector) {
    let middle = normalized(&sum(start, end));
    let along = sum(end, &scale(&form(end, &middle), &middle));
    let unit = scale(&div(&int(1), &sqrt(&form(&along, &along))), &along);
    (middle, unit)
}

fn open_interval() -> (BigFloat, BigFloat) {
    (int(1).neg(), int(1))
}

impl Reference {
    fn trig(&mut self, orders: &[u32], values: &[f64], feature: Feature) -> Trig {
        match feature.kind {
            FeatureKind::Corner => {
                let angle = div(&self.pi(), &int(orders[feature.first]));
                Trig {
                    corner: true,
                    cos: self.cos(&angle),
                    sin: self.sin(&angle),
                }
            }
            FeatureKind::Length(index) => {
                let length = num(values[index]);
                Trig {
                    corner: false,
                    cos: self.cosh(&length),
                    sin: self.sinh(&length),
                }
            }
        }
    }

    pub(crate) fn gap(
        &mut self,
        orders: &[u32],
        values: &[f64],
        sheet: &[Feature; 3],
        start: usize,
    ) -> BigFloat {
        let [f, g, h] = [0, 1, 2].map(|offset| sheet[(start + offset) % 3]);
        let (f, g, h) = (
            self.trig(orders, values, f),
            self.trig(orders, values, g),
            self.trig(orders, values, h),
        );
        let argument = div(&add(&mul(&f.cos, &g.cos), &h.cos), &mul(&f.sin, &g.sin));
        if f.corner == g.corner {
            self.acosh(&argument)
        } else {
            self.asinh(&argument)
        }
    }

    pub(crate) fn gap_sums(&mut self, orders: &[u32], values: &[f64]) -> Vec<BigFloat> {
        let count = orders.len();
        let cuts = Cuts::of(orders);
        let mut sums = vec![int(0); count];
        for sheet in &cuts.sheets {
            for start in 0..3 {
                let (f, g) = (sheet[start], sheet[(start + 1) % 3]);
                assert_eq!(
                    (f.last + 1) % count,
                    g.first,
                    "{orders:?}: a gap between {f:?} and {g:?}"
                );
                let side = g.first;
                let gap = self.gap(orders, values, sheet, start);
                sums[side] = add(&sums[side], &gap);
            }
        }
        sums
    }

    pub(crate) fn reflection_polygon(
        &mut self,
        orders: &[u32],
        values: &[f64],
    ) -> ReflectionPolygon {
        let count = orders.len();
        let cuts = Cuts::of(orders);
        let gap_sums = self.gap_sums(orders, values);
        let mut lengths = gap_sums.clone();
        for sheet in &cuts.sheets {
            for feature in sheet {
                if let FeatureKind::Length(index) = feature.kind
                    && index < cuts.pairs
                {
                    lengths[feature.last] = num(values[index]);
                }
            }
        }
        let pi = self.pi();
        let mut frame = identity();
        let mut walk = vec![origin()];
        for (side, &order) in lengths.iter().zip(orders).skip(1) {
            frame = product(&frame, &boost_matrix(&self.cosh(side), &self.sinh(side)));
            walk.push(column_two(&frame));
            let angle = div(&pi, &int(order));
            frame = product(
                &frame,
                &rotation_matrix(&self.cos(&angle).neg(), &self.sin(&angle)),
            );
        }
        let vertices = placed(&walk);
        let normals: Vec<Vector> = (0..count)
            .map(|index| {
                hyperbolic_normal(&vertices[(index + count - 1) % count], &vertices[index])
            })
            .collect();
        let distances = (0..count)
            .map(|index| self.distance(&vertices[(index + count - 1) % count], &vertices[index]))
            .collect();
        let angles = (0..count)
            .map(|index| {
                self.angle(
                    &vertices[(index + count - 1) % count],
                    &vertices[index],
                    &vertices[(index + 1) % count],
                )
            })
            .collect();
        ReflectionPolygon {
            walk,
            vertices,
            normals,
            lengths: distances,
            angles,
        }
    }

    pub(crate) fn angle(&mut self, before: &Vector, vertex: &Vector, after: &Vector) -> BigFloat {
        let cosh_of = |x: &Vector, y: &Vector| form(x, y).neg();
        let sinh_of = |cosh: &BigFloat| sqrt(&sub(&mul(cosh, cosh), &int(1)));
        let (a, b, c) = (
            cosh_of(vertex, before),
            cosh_of(vertex, after),
            cosh_of(before, after),
        );
        let cosine = div(&sub(&mul(&a, &b), &c), &mul(&sinh_of(&a), &sinh_of(&b)));
        self.acos(&cosine)
    }

    fn angle_sum(&mut self, halves: &[BigFloat], radius: &BigFloat) -> BigFloat {
        let cosh = self.cosh(radius);
        let mut total = int(0);
        for half in halves {
            total = add(&total, &self.asin(&div(half, &cosh)));
        }
        total
    }

    pub(crate) fn asin(&mut self, value: &BigFloat) -> BigFloat {
        value.asin(PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn atan(&mut self, value: &BigFloat) -> BigFloat {
        value.atan(PRECISION, ROUNDING, &mut self.consts)
    }

    pub(crate) fn inscribed(&mut self, orders: &[u32]) -> Inscribed {
        let pi = self.pi();
        let halves: Vec<BigFloat> = orders
            .iter()
            .map(|&order| self.cos(&div(&pi, &int(2 * order))))
            .collect();
        let mut high = int(1);
        while self.angle_sum(&halves, &high) >= pi {
            high = mul(&high, &int(2));
        }
        let mut low = int(0);
        let width = self.parse("1e-70");
        while sub(&high, &low) > width {
            let middle = div(&add(&low, &high), &int(2));
            if self.angle_sum(&halves, &middle) > pi {
                low = middle;
            } else {
                high = middle;
            }
        }
        let radius = div(&add(&low, &high), &int(2));
        let cosh = self.cosh(&radius);
        let mut direction = int(0);
        let mut previous: Option<BigFloat> = None;
        let mut vertices = Vec::new();
        for (&order, half) in orders.iter().zip(&halves) {
            let sine = div(half, &cosh);
            let alpha = self.asin(&sine);
            if let Some(previous) = previous {
                direction = add(&add(&direction, &previous), &alpha);
            }
            let cosine = sqrt(&sub(&int(1), &mul(&sine, &sine)));
            let half_angle = div(&pi, &int(2 * order));
            let cosh_rho = div(&mul(&cosine, half), &mul(&sine, &self.sin(&half_angle)));
            let sinh_rho = sqrt(&sub(&mul(&cosh_rho, &cosh_rho), &int(1)));
            vertices.push([
                mul(&sinh_rho, &self.cos(&direction)),
                mul(&sinh_rho, &self.sin(&direction)),
                cosh_rho,
            ]);
            previous = Some(alpha);
        }
        Inscribed { radius, vertices }
    }

    pub(crate) fn distance_to_segment(
        &mut self,
        point: &Vector,
        start: &Vector,
        end: &Vector,
    ) -> BigFloat {
        let smallest = golden_minimum(POINT_STEPS, int(0), int(1), |fraction| {
            let along = sum(
                &scale(&sub(&int(1), fraction), start),
                &scale(fraction, end),
            );
            div(
                &form(point, &along).neg(),
                &sqrt(&form(&along, &along).neg()),
            )
        });
        self.acosh(&max(&int(1), &smallest))
    }

    pub(crate) fn distance_between_walls(
        &mut self,
        first: [&Vector; 2],
        second: [&Vector; 2],
    ) -> BigFloat {
        let (middle_a, along_a) = geodesic(first[0], first[1]);
        let (middle_b, along_b) = geodesic(second[0], second[1]);
        let (low, high) = open_interval();
        let squared = golden_minimum(POINT_STEPS, low, high, |s| {
            let point = sum(&middle_a, &scale(s, &along_a));
            let towards_middle = form(&point, &middle_b).neg();
            let towards_along = form(&point, &along_b).neg();
            div(
                &sub(
                    &mul(&towards_middle, &towards_middle),
                    &mul(&towards_along, &towards_along),
                ),
                &sub(&int(1), &mul(s, s)),
            )
        });
        self.acosh(&max(&int(1), &sqrt(&squared)))
    }

    fn lay_out(&mut self, orders: &[u32], values: &[f64], sheet: &[Feature; 3]) -> LaidOut {
        let pi = self.pi();
        let quarter = rotation_matrix(&int(0), &int(1));
        let mut frame = identity();
        let mut points = Vec::new();
        for start in 0..3 {
            let feature = sheet[start];
            match feature.kind {
                FeatureKind::Corner => {
                    points.push((feature, 0, column_two(&frame), frame.clone()));
                    let angle = div(&pi, &int(orders[feature.first]));
                    let turn = rotation_matrix(&self.cos(&angle).neg(), &self.sin(&angle));
                    frame = product(&frame, &turn);
                }
                FeatureKind::Length(index) => {
                    frame = product(&frame, &quarter);
                    points.push((feature, 0, column_two(&frame), frame.clone()));
                    let length = num(values[index]);
                    frame = product(
                        &frame,
                        &boost_matrix(&self.cosh(&length), &self.sinh(&length)),
                    );
                    points.push((feature, 1, column_two(&frame), frame.clone()));
                    frame = product(&frame, &quarter);
                }
            }
            let gap = self.gap(orders, values, sheet, start);
            frame = product(&frame, &boost_matrix(&self.cosh(&gap), &self.sinh(&gap)));
        }
        LaidOut {
            points,
            closure: largest_entry_from_identity(&frame),
        }
    }

    pub(crate) fn weld(&mut self, orders: &[u32], values: &[f64]) -> Weld {
        let cuts = Cuts::of(orders);
        let laid: Vec<LaidOut> = cuts
            .sheets
            .iter()
            .map(|sheet| self.lay_out(orders, values, sheet))
            .collect();
        let half_turn = rotation_matrix(&int(1).neg(), &int(0));
        let mut placements: Vec<Option<Matrix>> = vec![None; laid.len()];
        let last = laid.len() - 1;
        placements[last] = Some(identity());
        let holder = |slice: usize, except: usize| {
            laid.iter()
                .enumerate()
                .filter(|&(index, _)| index != except)
                .find_map(|(index, sheet)| {
                    sheet
                        .points
                        .iter()
                        .position(|(feature, end, _, _)| {
                            feature.kind == FeatureKind::Length(cuts.pairs + slice) && *end == 0
                        })
                        .map(|position| (index, position))
                })
        };
        let mut slice_ends = Vec::new();
        for index in (0..last).rev() {
            let Some((other, position)) = holder(index, index) else {
                panic!("{orders:?}: slice {index} has no second sheet");
            };
            let Some(placement) = placements[other].clone() else {
                panic!("{orders:?}: sheet {other} is welded after sheet {index}");
            };
            let held = &laid[other].points;
            let own = &laid[index].points;
            let Some(start) = own.iter().position(|(feature, end, _, _)| {
                feature.kind == FeatureKind::Length(cuts.pairs + index) && *end == 0
            }) else {
                panic!("{orders:?}: sheet {index} does not hold slice {index}");
            };
            let target = product(&product(&placement, &held[position + 1].3), &half_turn);
            let map = product(&target, &isometry_inverse(&own[start].3));
            let (held_count, own_count) = (held.len(), own.len());
            for (end, before, after) in [
                (
                    position,
                    apply(
                        &placement,
                        &held[(position + held_count - 1) % held_count].2,
                    ),
                    apply(&map, &own[(start + 2) % own_count].2),
                ),
                (
                    position + 1,
                    apply(&map, &own[(start + own_count - 1) % own_count].2),
                    apply(&placement, &held[(position + 2) % held_count].2),
                ),
            ] {
                let inverse = isometry_inverse(&product(&placement, &held[end].3));
                let (before, after) = (apply(&inverse, &before), apply(&inverse, &after));
                let cross = sub(&mul(&before[0], &after[1]), &mul(&before[1], &after[0])).abs();
                let dot = add(&mul(&before[0], &after[0]), &mul(&before[1], &after[1]));
                assert!(
                    dot.is_negative(),
                    "{orders:?}: the sides at an end of slice {index} turn back"
                );
                slice_ends.push(self.atan(&div(&cross, &dot.neg())));
            }
            placements[index] = Some(map);
        }
        Weld {
            vertices: welded_vertices(orders, cuts.pairs, &laid, &placements),
            closures: laid.into_iter().map(|sheet| sheet.closure).collect(),
            slice_ends,
        }
    }
}

fn welded_vertices(
    orders: &[u32],
    pairs: usize,
    laid: &[LaidOut],
    placements: &[Option<Matrix>],
) -> Vec<Vector> {
    let mut vertices = vec![None; orders.len()];
    for (sheet, placement) in laid.iter().zip(placements) {
        let Some(placement) = placement else {
            panic!("{orders:?}: a sheet is not welded");
        };
        for (feature, end, point, _) in &sheet.points {
            let corner = match feature.kind {
                FeatureKind::Corner => feature.first,
                FeatureKind::Length(index) if index < pairs => {
                    if *end == 0 {
                        feature.first
                    } else {
                        feature.last
                    }
                }
                FeatureKind::Length(_) => continue,
            };
            vertices[corner] = Some(apply(placement, point));
        }
    }
    vertices
        .into_iter()
        .enumerate()
        .map(|(corner, vertex)| match vertex {
            Some(vertex) => vertex,
            None => panic!("{orders:?}: corner {corner} has no vertex"),
        })
        .collect()
}

fn placed(walk: &[Vector]) -> Vec<Vector> {
    let mut total = [int(0), int(0), int(0)];
    for point in walk {
        total = sum(&total, point);
    }
    let centre = normalized(&total);
    let h = sqrt(&add(
        &mul(&centre[0], &centre[0]),
        &mul(&centre[1], &centre[1]),
    ));
    let (cos_phi, sin_phi) = (div(&centre[0], &h), div(&centre[1], &h));
    let (sinh_rho, cosh_rho) = (h, centre[2].clone());
    let moved: Vec<Vector> = walk
        .iter()
        .map(|x| {
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
        })
        .collect();
    let first = &moved[0];
    let r = sqrt(&add(&mul(&first[0], &first[0]), &mul(&first[1], &first[1])));
    let (cos, sin) = (div(&first[0], &r), div(&first[1], &r).neg());
    moved
        .iter()
        .map(|x| {
            [
                sub(&mul(&cos, &x[0]), &mul(&sin, &x[1])),
                add(&mul(&sin, &x[0]), &mul(&cos, &x[1])),
                x[2].clone(),
            ]
        })
        .collect()
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

    pub(crate) fn count(&self) -> usize {
        self.count
    }

    pub(crate) fn print(&self, reference: &mut Reference) {
        let ratio = reference.approximate(&self.ratio);
        println!(
            "{}: largest error {ratio:.3e}, {} samples, at {}",
            self.name, self.count, self.input
        );
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
