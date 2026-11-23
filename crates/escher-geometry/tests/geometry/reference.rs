use astro_float_num::BigFloat;
use escher_reference::{
    Complex, Matrix, Reference, Vector, add, column, complex_mul, complex_sub, div, dot, form, int,
    mul, normalize, num, product, rotation_matrix, scale, sqrt, squared_modulus, sub, unit,
};

const STEP_EXPONENT: i32 = -100;

const SCAN_POINTS: u32 = 4096;

const REFINE_STEPS: usize = 200;

const HYPERBOLIC_SCAN_HALF_WIDTH: f64 = 40.0;

const EUCLIDEAN_SCAN_HALF_WIDTH: f64 = 256.0;

const HYPERBOLIC: Family = Family {
    even: Reference::cosh,
    odd: Reference::sinh,
    circular: false,
};

const CIRCULAR: Family = Family {
    even: Reference::cos,
    odd: Reference::sin,
    circular: true,
};

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

pub(crate) fn spherical_wall_sine(c: &Vector, x: &Vector) -> BigFloat {
    div(&dot(c, x).abs(), &sqrt(&dot(x, x)))
}

pub(crate) fn euclidean_wall_distance(c: &Vector, x: &Vector) -> BigFloat {
    dot(c, x).abs()
}

fn combination(first: &BigFloat, a: &Vector, second: &BigFloat, b: &Vector) -> Vector {
    let (left, right) = (scale(first, a), scale(second, b));
    [0, 1, 2].map(|index| add(&left[index], &right[index]))
}

fn translation_matrix(first: &BigFloat, second: &BigFloat) -> Matrix {
    [
        [int(1), int(0), first.clone()],
        [int(0), int(1), second.clone()],
        [int(0), int(0), int(1)],
    ]
}

fn golden_ratio_part() -> BigFloat {
    div(&sub(&sqrt(&int(5)), &int(1)), &int(2))
}

fn minimizer(
    grid: &[BigFloat],
    low: &BigFloat,
    step: &BigFloat,
    objective: &mut dyn FnMut(&BigFloat) -> BigFloat,
) -> BigFloat {
    let mut best = (0, &grid[0]);
    for (index, value) in (0..).zip(grid) {
        if value < best.1 {
            best = (index, value);
        }
    }
    let centre = add(low, &mul(&int(best.0), step));
    let (mut a, mut b) = (sub(&centre, step), add(&centre, step));
    let ratio = golden_ratio_part();
    let mut c = sub(&b, &mul(&ratio, &sub(&b, &a)));
    let mut d = add(&a, &mul(&ratio, &sub(&b, &a)));
    let (mut fc, mut fd) = (objective(&c), objective(&d));
    for _ in 0..REFINE_STEPS {
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = sub(&b, &mul(&ratio, &sub(&b, &a)));
            fc = objective(&c);
        } else {
            a = c;
            c = d;
            fc = fd;
            d = add(&a, &mul(&ratio, &sub(&b, &a)));
            fd = objective(&d);
        }
    }
    div(&add(&a, &b), &int(2))
}

fn scan_step(half_width: &BigFloat) -> BigFloat {
    div(&mul(&int(2), half_width), &int(SCAN_POINTS - 1))
}

struct Family {
    even: fn(&mut Reference, &BigFloat) -> BigFloat,
    odd: fn(&mut Reference, &BigFloat) -> BigFloat,
    circular: bool,
}

fn wall_map(reference: &mut Reference, family: &Family, length: f64, angle: f64) -> Matrix {
    let (length, angle) = (num(length), num(angle));
    let (even, odd) = (
        (family.even)(reference, &length),
        (family.odd)(reference, &length),
    );
    let lower = if family.circular {
        odd.neg()
    } else {
        odd.clone()
    };
    let boost = [
        [even.clone(), int(0), odd],
        [int(0), int(1), int(0)],
        [lower, int(0), even],
    ];
    let rotation = rotation_matrix(&reference.cos(&angle), &reference.sin(&angle));
    product(&boost, &rotation)
}

fn nearest_wall_point(
    reference: &mut Reference,
    family: &Family,
    half_width: &BigFloat,
    along: [&Vector; 2],
    weights: [&BigFloat; 2],
) -> Vector {
    let [a, b] = along;
    let [alpha, beta] = weights;
    let low = half_width.neg();
    let step = scan_step(half_width);
    let (step_even, step_odd) = (
        (family.even)(reference, &step),
        (family.odd)(reference, &step),
    );
    let (mut even, mut odd) = (
        (family.even)(reference, &low),
        (family.odd)(reference, &low),
    );
    let mut grid = Vec::new();
    for _ in 0..SCAN_POINTS {
        grid.push(add(&mul(alpha, &odd), &mul(beta, &even)).neg());
        let (first, second) = (mul(&even, &step_even), mul(&odd, &step_odd));
        let next_even = if family.circular {
            sub(&first, &second)
        } else {
            add(&first, &second)
        };
        odd = add(&mul(&odd, &step_even), &mul(&even, &step_odd));
        even = next_even;
    }
    let s = minimizer(&grid, &low, &step, &mut |s| {
        let odd = (family.odd)(reference, s);
        add(&mul(alpha, &odd), &mul(beta, &(family.even)(reference, s))).neg()
    });
    let (odd, even) = ((family.odd)(reference, &s), (family.even)(reference, &s));
    combination(&odd, a, &even, b)
}

pub(crate) struct ReferenceWall {
    pub(crate) covector: Vector,
    map: Matrix,
}

pub(crate) trait ReferenceExt {
    fn band_scale_factor(&mut self, w: &Complex) -> BigFloat;
    fn hyperbolic_wall_distance(&mut self, c: &Vector, x: &Vector) -> BigFloat;
    fn spherical_wall_distance(&mut self, c: &Vector, x: &Vector) -> BigFloat;
    fn hyperbolic_wall(&mut self, length: f64, angle: f64) -> ReferenceWall;
    fn euclidean_wall(&mut self, target: [f64; 2], angle: f64) -> ReferenceWall;
    fn spherical_wall(&mut self, length: f64, angle: f64) -> ReferenceWall;
    fn hyperbolic_point_off_wall(
        &mut self,
        wall: &ReferenceWall,
        along: f64,
        offset: f64,
    ) -> Vector;
    fn spherical_point_off_wall(&mut self, wall: &ReferenceWall, along: f64, offset: f64)
    -> Vector;
    fn hyperbolic_minimum_distance(&mut self, wall: &ReferenceWall, x: &Vector) -> BigFloat;
    fn spherical_minimum_distance(&mut self, wall: &ReferenceWall, x: &Vector) -> BigFloat;
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

    fn hyperbolic_wall_distance(&mut self, c: &Vector, x: &Vector) -> BigFloat {
        let q = form(x, x).neg();
        self.asinh(&div(&dot(c, x).abs(), &sqrt(&q)))
    }

    fn spherical_wall_distance(&mut self, c: &Vector, x: &Vector) -> BigFloat {
        self.asin(&spherical_wall_sine(c, x))
    }

    fn hyperbolic_wall(&mut self, length: f64, angle: f64) -> ReferenceWall {
        let map = wall_map(self, &HYPERBOLIC, length, angle);
        let normal = column(&map, 0);
        let covector = [normal[0].clone(), normal[1].clone(), normal[2].neg()];
        ReferenceWall { covector, map }
    }

    fn euclidean_wall(&mut self, target: [f64; 2], angle: f64) -> ReferenceWall {
        let [first, second] = target.map(num);
        let angle = num(angle);
        let (cos, sin) = (self.cos(&angle), self.sin(&angle));
        let map = product(
            &translation_matrix(&first, &second),
            &rotation_matrix(&cos, &sin),
        );
        let inverse = product(
            &rotation_matrix(&cos, &sin.neg()),
            &translation_matrix(&first.neg(), &second.neg()),
        );
        ReferenceWall {
            covector: inverse[0].clone(),
            map,
        }
    }

    fn spherical_wall(&mut self, length: f64, angle: f64) -> ReferenceWall {
        let map = wall_map(self, &CIRCULAR, length, angle);
        ReferenceWall {
            covector: column(&map, 0),
            map,
        }
    }

    fn hyperbolic_point_off_wall(
        &mut self,
        wall: &ReferenceWall,
        along: f64,
        offset: f64,
    ) -> Vector {
        let (along, offset) = (num(along), num(offset));
        let cosh_along = self.cosh(&along);
        let standard = [
            mul(&cosh_along, &self.sinh(&offset)),
            self.sinh(&along),
            mul(&cosh_along, &self.cosh(&offset)),
        ];
        escher_reference::apply(&wall.map, &standard)
    }

    fn spherical_point_off_wall(
        &mut self,
        wall: &ReferenceWall,
        along: f64,
        offset: f64,
    ) -> Vector {
        let (along, offset) = (num(along), num(offset));
        let cos_offset = self.cos(&offset);
        let standard = [
            self.sin(&offset),
            mul(&self.sin(&along), &cos_offset),
            mul(&self.cos(&along), &cos_offset),
        ];
        escher_reference::apply(&wall.map, &standard)
    }

    fn hyperbolic_minimum_distance(&mut self, wall: &ReferenceWall, x: &Vector) -> BigFloat {
        let point = normalize(x);
        let (a, b) = (column(&wall.map, 1), column(&wall.map, 2));
        let (alpha, beta) = (form(&point, &a), form(&point, &b));
        let half_width = num(HYPERBOLIC_SCAN_HALF_WIDTH);
        let nearest = nearest_wall_point(self, &HYPERBOLIC, &half_width, [&a, &b], [&alpha, &beta]);
        self.distance(&point, &nearest)
    }

    fn spherical_minimum_distance(&mut self, wall: &ReferenceWall, x: &Vector) -> BigFloat {
        let point = unit(x);
        let (a, b) = (column(&wall.map, 1), column(&wall.map, 2));
        let (alpha, beta) = (dot(&point, &a), dot(&point, &b));
        let half_width = self.pi();
        let nearest = nearest_wall_point(self, &CIRCULAR, &half_width, [&a, &b], [&alpha, &beta]);
        self.acos(&dot(&point, &nearest))
    }
}

pub(crate) fn euclidean_minimum_distance(wall: &ReferenceWall, x: &Vector) -> BigFloat {
    let (a, b) = (column(&wall.map, 1), column(&wall.map, 2));
    let squared = |s: &BigFloat| {
        let nearest = combination(s, &a, &int(1), &b);
        let (first, second) = (sub(&x[0], &nearest[0]), sub(&x[1], &nearest[1]));
        add(&mul(&first, &first), &mul(&second, &second))
    };
    let half_width = num(EUCLIDEAN_SCAN_HALF_WIDTH);
    let low = half_width.neg();
    let step = scan_step(&half_width);
    let grid: Vec<BigFloat> = (0..SCAN_POINTS)
        .map(|index| squared(&add(&low, &mul(&int(index), &step))))
        .collect();
    let s = minimizer(&grid, &low, &step, &mut |s| squared(s));
    sqrt(&squared(&s))
}
