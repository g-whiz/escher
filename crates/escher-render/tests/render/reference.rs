#![expect(
    dead_code,
    reason = "the unit tests of the crate and the integration tests each use a different part of this module"
)]

use astro_float_num::BigFloat;
use escher_reference::{
    Complex, Matrix, Reference, Vector, add, apply, cross, difference, disk_to_half_plane,
    disk_to_klein, div, dot, half_plane_to_disk, int, klein_to_disk, max, mul, normalize, num,
    product, scale, sqrt, sub, unit, vector,
};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Model {
    HalfPlane,
    Band,
    Klein,
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
    fold_exact(walls, vector(point), cap)
}

pub(crate) fn fold_exact(walls: &[Wall], mut x: Vector, cap: u32) -> ExactFold {
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

fn central_differences(
    reference: &mut Reference,
    position: [f64; 2],
    pixel: f64,
    embedding: fn(&BigFloat, &BigFloat) -> Vector,
) -> [Vector; 2] {
    let step = reference.parse("1e-40");
    let half = div(&num(pixel), &mul(&int(2), &step));
    let [first, second] = position.map(num);
    let partial = |plus: Vector, minus: Vector| scale(&half, &difference(&plus, &minus));
    [
        partial(
            embedding(&add(&first, &step), &second),
            embedding(&sub(&first, &step), &second),
        ),
        partial(
            embedding(&first, &add(&second, &step)),
            embedding(&first, &sub(&second, &step)),
        ),
    ]
}

pub(crate) trait ReferenceExt {
    fn angle(&mut self, x: &Vector, y: &Vector) -> BigFloat;

    fn viewing(&mut self, angles: [f64; 3]) -> Matrix;

    fn lift(&mut self, position: [f64; 2], angles: [f64; 3]) -> Vector;

    fn ball_tangents(&mut self, position: [f64; 2], pixel: f64) -> [Vector; 2];

    fn shade(&mut self, normal: [f64; 3], light: [f64; 3], channel: f64) -> BigFloat;

    fn wall_distance(&mut self, covector: &Vector, point: &Vector) -> BigFloat;

    fn sphere_wall_distance(&mut self, normal: &Vector, point: &Vector) -> BigFloat;

    fn border_distance(
        &mut self,
        covectors: &[Vector],
        point: [f64; 3],
        position: [f64; 2],
        pixel: f64,
    ) -> BigFloat;

    fn tangents(&mut self, position: [f64; 2], pixel: f64) -> [Vector; 2];

    fn model_to_disk(&mut self, model: Model, w: &Complex) -> Complex;

    fn disk_to_model(&mut self, model: Model, z: [f64; 2]) -> [f64; 2];

    fn model_tangents(&mut self, model: Model, position: [f64; 2], pixel: f64) -> [Vector; 2];

    fn srgb_decode(&mut self, byte: u8) -> BigFloat;

    fn srgb_decode_unit(&mut self, encoded: &BigFloat) -> BigFloat;

    fn srgb_encode(&mut self, linear: &BigFloat) -> BigFloat;
}

impl ReferenceExt for Reference {
    fn angle(&mut self, x: &Vector, y: &Vector) -> BigFloat {
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

    fn viewing(&mut self, [yaw, pitch, roll]: [f64; 3]) -> Matrix {
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

    fn lift(&mut self, position: [f64; 2], angles: [f64; 3]) -> Vector {
        let [first, second] = position.map(num);
        let viewing = self.viewing(angles);
        transpose_apply(&viewing, &screen_point_exact(&first, &second))
    }

    fn ball_tangents(&mut self, position: [f64; 2], pixel: f64) -> [Vector; 2] {
        central_differences(self, position, pixel, screen_point_exact)
    }

    fn shade(&mut self, normal: [f64; 3], light: [f64; 3], channel: f64) -> BigFloat {
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

    fn wall_distance(&mut self, covector: &Vector, point: &Vector) -> BigFloat {
        self.asinh(&dot(covector, &normalize(point)).abs())
    }

    fn sphere_wall_distance(&mut self, normal: &Vector, point: &Vector) -> BigFloat {
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

    fn border_distance(
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

    fn tangents(&mut self, position: [f64; 2], pixel: f64) -> [Vector; 2] {
        central_differences(self, position, pixel, from_disk_exact)
    }

    fn model_to_disk(&mut self, model: Model, w: &Complex) -> Complex {
        match model {
            Model::HalfPlane => half_plane_to_disk(w),
            Model::Band => self.band_to_disk(w),
            Model::Klein => klein_to_disk(w),
        }
    }

    fn disk_to_model(&mut self, model: Model, z: [f64; 2]) -> [f64; 2] {
        let z = z.map(num);
        let exact = match model {
            Model::HalfPlane => disk_to_half_plane(&z),
            Model::Band => self.disk_to_band(&z),
            Model::Klein => disk_to_klein(&z),
        };
        exact.map(|coordinate| self.approximate(&coordinate))
    }

    fn model_tangents(&mut self, model: Model, position: [f64; 2], pixel: f64) -> [Vector; 2] {
        let step = self.parse("1e-40");
        let half = div(&num(pixel), &mul(&int(2), &step));
        let [first, second] = position.map(num);
        let mut embed = |w: Complex| {
            let [z1, z2] = self.model_to_disk(model, &w);
            from_disk_exact(&z1, &z2)
        };
        let mut partial = |ahead: Complex, behind: Complex| {
            let plus = embed(ahead);
            let minus = embed(behind);
            scale(&half, &difference(&plus, &minus))
        };
        [
            partial(
                [add(&first, &step), second.clone()],
                [sub(&first, &step), second.clone()],
            ),
            partial(
                [first.clone(), add(&second, &step)],
                [first, sub(&second, &step)],
            ),
        ]
    }

    fn srgb_decode(&mut self, byte: u8) -> BigFloat {
        let encoded = div(&int(u32::from(byte)), &int(255));
        self.srgb_decode_unit(&encoded)
    }

    fn srgb_decode_unit(&mut self, encoded: &BigFloat) -> BigFloat {
        if *encoded <= self.parse("0.04045") {
            div(encoded, &self.parse("12.92"))
        } else {
            let base = div(&add(encoded, &self.parse("0.055")), &self.parse("1.055"));
            let exponent = self.parse("2.4");
            self.pow(&base, &exponent)
        }
    }

    fn srgb_encode(&mut self, linear: &BigFloat) -> BigFloat {
        if *linear <= self.parse("0.0031308") {
            mul(&self.parse("12.92"), linear)
        } else {
            let exponent = div(&int(1), &self.parse("2.4"));
            let power = self.pow(linear, &exponent);
            sub(&mul(&self.parse("1.055"), &power), &self.parse("0.055"))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Measure {
    Hyperbolic,
    Euclidean,
    Spherical,
}

impl Measure {
    pub(crate) const fn of(geometry: escher_signature::Geometry) -> Self {
        match geometry {
            escher_signature::Geometry::Hyperbolic => Self::Hyperbolic,
            escher_signature::Geometry::Euclidean => Self::Euclidean,
            escher_signature::Geometry::Spherical => Self::Spherical,
        }
    }

    pub(crate) fn distance(self, reference: &mut Reference, x: &Vector, y: &Vector) -> BigFloat {
        match self {
            Self::Hyperbolic => reference.distance(&normalize(x), &normalize(y)),
            Self::Euclidean => plane_distance(x, y),
            Self::Spherical if dot(x, y).is_positive() => reference.angle(x, y),
            Self::Spherical => reference.pi(),
        }
    }

    pub(crate) fn wall_distance(
        self,
        reference: &mut Reference,
        covector: &Vector,
        point: &Vector,
    ) -> BigFloat {
        match self {
            Self::Hyperbolic => reference.wall_distance(covector, point),
            Self::Euclidean => plane_wall_distance(covector, point),
            Self::Spherical => reference.sphere_wall_distance(covector, point),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MaskRule {
    Plane,
    Triangle([u32; 3]),
    Klein,
    Double([u32; 3]),
    FaceTriangle { corner: usize, order: u32 },
}

pub(crate) type Flat = [BigFloat; 2];

pub(crate) struct ExactMask {
    pub(crate) measure: Measure,
    pub(crate) vertices: Vec<Vector>,
    covectors: Vec<Vector>,
    pub(crate) centre: Vector,
    chart_centre: Vector,
    pub(crate) polygon: Vec<Flat>,
    pub(crate) flat_centre: Flat,
    lengths: Vec<BigFloat>,
    charts: Vec<Vector>,
}

fn recentre(centre: &Vector) -> Matrix {
    let [x, y, z] = centre.clone();
    let lift = add(&int(1), &z);
    [
        [
            add(&int(1), &div(&mul(&x, &x), &lift)),
            div(&mul(&x, &y), &lift),
            x.neg(),
        ],
        [
            div(&mul(&x, &y), &lift),
            add(&int(1), &div(&mul(&y, &y), &lift)),
            y.neg(),
        ],
        [x.neg(), y.neg(), z],
    ]
}

pub(crate) fn face_direction(measure: Measure, vertices: &[Vector], corner: usize) -> Flat {
    let (centre, foot) = (&vertices[corner], &vertices[2]);
    let toward = match measure {
        Measure::Euclidean => {
            let [centre, foot] = [centre, foot].map(|point| scale(&div(&int(1), &point[2]), point));
            difference(&foot, &centre)
        }
        Measure::Hyperbolic => apply(&recentre(&normalize(centre)), &normalize(foot)),
        Measure::Spherical => {
            let (centre, foot) = (unit(centre), unit(foot));
            difference(&foot, &scale(&dot(&centre, &foot), &centre))
        }
    };
    let flat = [toward[0].clone(), toward[1].clone()];
    if flat.iter().all(BigFloat::is_zero) {
        return [int(0), int(1).neg()];
    }
    flat_scale(&div(&int(1), &sqrt(&flat_dot(&flat, &flat))), &flat)
}

pub(crate) fn flat_turn(direction: &Flat, point: &Flat) -> Flat {
    [
        sub(
            &mul(&direction[0], &point[0]),
            &mul(&direction[1], &point[1]),
        ),
        add(
            &mul(&direction[1], &point[0]),
            &mul(&direction[0], &point[1]),
        ),
    ]
}

fn flat_sub(left: &Flat, right: &Flat) -> Flat {
    [sub(&left[0], &right[0]), sub(&left[1], &right[1])]
}

fn flat_add(left: &Flat, right: &Flat) -> Flat {
    [add(&left[0], &right[0]), add(&left[1], &right[1])]
}

fn flat_scale(factor: &BigFloat, value: &Flat) -> Flat {
    [mul(factor, &value[0]), mul(factor, &value[1])]
}

fn flat_dot(left: &Flat, right: &Flat) -> BigFloat {
    add(&mul(&left[0], &right[0]), &mul(&left[1], &right[1]))
}

fn flat_triangle(reference: &mut Reference, orders: [u32; 3]) -> [Flat; 3] {
    let pi = reference.pi();
    let [first, second, third] = orders.map(|order| div(&pi, &int(order)));
    let angles = match orders {
        [_, second_order, 2] if second_order >= 3 => {
            [sub(&sub(&pi, &second), &third), second, third]
        }
        [_, 2, 2] => [first.clone(), sub(&sub(&pi, &first), &third), third],
        [_, _, third_order] if third_order >= 3 => [
            first.clone(),
            second.clone(),
            sub(&sub(&pi, &first), &second),
        ],
        _ => panic!("the corner orders {orders:?} have no flat triangle"),
    };
    assert!(
        angles.iter().all(BigFloat::is_positive),
        "the corner orders {orders:?} have no flat triangle"
    );
    let reach = div(&reference.sin(&angles[0]), &reference.sin(&angles[1]));
    let (cos, sin) = (reference.cos(&angles[2]), reference.sin(&angles[2]));
    [
        [int(1), int(0)],
        [mul(&reach, &cos), mul(&reach, &sin)],
        [int(0), int(0)],
    ]
}

fn reflect_flat(point: &Flat, start: &Flat, end: &Flat) -> Flat {
    let direction = flat_sub(end, start);
    let along = div(
        &flat_dot(&flat_sub(point, start), &direction),
        &flat_dot(&direction, &direction),
    );
    let foot = flat_add(start, &flat_scale(&along, &direction));
    flat_sub(&flat_scale(&int(2), &foot), point)
}

impl ExactMask {
    pub(crate) fn new(
        reference: &mut Reference,
        measure: Measure,
        vertices: &[[f64; 3]],
        covectors: &[[f64; 3]],
        rule: MaskRule,
    ) -> Self {
        let vertices: Vec<Vector> = vertices.iter().map(|&vertex| vector(vertex)).collect();
        let count = u32::try_from(vertices.len()).unwrap_or(u32::MAX);
        let sum = vertices
            .iter()
            .fold([int(0), int(0), int(0)], |total, vertex| {
                [0, 1, 2].map(|index| add(&total[index], &vertex[index]))
            });
        let centre = match measure {
            Measure::Hyperbolic => normalize(&sum),
            Measure::Spherical => unit(&sum),
            Measure::Euclidean => scale(&div(&int(1), &int(count)), &sum),
        };
        let polygon: Vec<Flat> = match rule {
            MaskRule::Plane => vertices
                .iter()
                .map(|vertex| [div(&vertex[0], &vertex[2]), div(&vertex[1], &vertex[2])])
                .collect(),
            MaskRule::Triangle(orders) => flat_triangle(reference, orders).to_vec(),
            MaskRule::Klein => {
                let recentre = recentre(&centre);
                vertices
                    .iter()
                    .map(|vertex| {
                        let moved = apply(&recentre, vertex);
                        [div(&moved[0], &moved[2]), div(&moved[1], &moved[2])]
                    })
                    .collect()
            }
            MaskRule::Double(orders) => {
                let [first, second, third] = flat_triangle(reference, orders);
                let mirrored = reflect_flat(&third, &first, &second);
                vec![first, mirrored, second, third]
            }
            MaskRule::FaceTriangle { corner, order } => {
                let direction = face_direction(measure, &vertices, corner);
                face_triangle(reference, corner, order, &direction).to_vec()
            }
        };
        assert_eq!(polygon.len(), vertices.len(), "{rule:?}");
        let flat_sum = polygon
            .iter()
            .fold([int(0), int(0)], |total, vertex| flat_add(&total, vertex));
        let flat_centre = flat_scale(&div(&int(1), &int(count)), &flat_sum);
        let mut mask = Self {
            measure,
            covectors: covectors.iter().map(|&covector| vector(covector)).collect(),
            chart_centre: centre.clone(),
            centre,
            polygon,
            flat_centre,
            lengths: Vec::new(),
            charts: Vec::new(),
            vertices,
        };
        mask.chart_centre = mask.chart(&mask.centre);
        mask.charts = mask
            .vertices
            .iter()
            .map(|vertex| difference(&mask.chart(vertex), &mask.chart_centre))
            .collect();
        mask.lengths = (0..mask.vertices.len())
            .map(|side| {
                let previous = mask.vertices[mask.previous(side)].clone();
                mask.distance(reference, &previous, &mask.vertices[side])
            })
            .collect();
        mask
    }

    fn previous(&self, side: usize) -> usize {
        (side + self.vertices.len() - 1) % self.vertices.len()
    }

    pub(crate) fn chart(&self, point: &Vector) -> Vector {
        let divisor = match self.measure {
            Measure::Hyperbolic | Measure::Euclidean => point[2].clone(),
            Measure::Spherical => dot(&self.centre, point),
        };
        assert!(divisor.is_positive(), "a point lies outside the chart");
        scale(&div(&int(1), &divisor), point)
    }

    fn turn(&self, first: &Vector, second: &Vector) -> BigFloat {
        match self.measure {
            Measure::Hyperbolic | Measure::Euclidean => {
                sub(&mul(&first[0], &second[1]), &mul(&first[1], &second[0]))
            }
            Measure::Spherical => dot(&self.centre, &cross(first, second)),
        }
    }

    fn unchart(&self, chart: &Vector) -> Vector {
        match self.measure {
            Measure::Hyperbolic => normalize(chart),
            Measure::Euclidean => chart.clone(),
            Measure::Spherical => unit(chart),
        }
    }

    pub(crate) fn distance(&self, reference: &mut Reference, x: &Vector, y: &Vector) -> BigFloat {
        match self.measure {
            Measure::Hyperbolic => reference.distance(&normalize(x), &normalize(y)),
            Measure::Euclidean => plane_distance(x, y),
            Measure::Spherical => {
                let cosine = dot(&unit(x), &unit(y));
                let clamped = max(&min(&cosine, &int(1)), &int(1).neg());
                reference.acos(&clamped)
            }
        }
    }

    pub(crate) fn along_side(
        &self,
        reference: &mut Reference,
        side: usize,
        fraction: &BigFloat,
    ) -> Vector {
        let first = &self.vertices[self.previous(side)];
        let second = &self.vertices[side];
        let length = &self.lengths[side];
        let rest = sub(&int(1), fraction);
        match self.measure {
            Measure::Euclidean => {
                let first = scale(&div(&int(1), &first[2]), first);
                let second = scale(&div(&int(1), &second[2]), second);
                add_vectors(&scale(&rest, &first), &scale(fraction, &second))
            }
            Measure::Hyperbolic => {
                let whole = reference.sinh(length);
                let near = div(&reference.sinh(&mul(&rest, length)), &whole);
                let far = div(&reference.sinh(&mul(fraction, length)), &whole);
                add_vectors(
                    &scale(&near, &normalize(first)),
                    &scale(&far, &normalize(second)),
                )
            }
            Measure::Spherical => {
                let whole = reference.sin(length);
                let near = div(&reference.sin(&mul(&rest, length)), &whole);
                let far = div(&reference.sin(&mul(fraction, length)), &whole);
                add_vectors(&scale(&near, &unit(first)), &scale(&far, &unit(second)))
            }
        }
    }

    pub(crate) fn map(&self, reference: &mut Reference, point: &Vector) -> Flat {
        let chart = self.chart(point);
        let ray = difference(&chart, &self.chart_centre);
        if ray.iter().all(BigFloat::is_zero) {
            return self.flat_centre.clone();
        }
        let count = self.vertices.len();
        let side = (0..count)
            .find(|&side| {
                let start = &self.charts[self.previous(side)];
                let end = &self.charts[side];
                !self.turn(start, &ray).is_negative() && self.turn(&ray, end).is_positive()
            })
            .unwrap_or_else(|| panic!("no sector of the chart holds the ray"));
        let covector = &self.covectors[side];
        let parameter = div(
            &dot(covector, &self.chart_centre).neg(),
            &dot(covector, &ray),
        );
        let boundary = self.unchart(&add_vectors(&self.chart_centre, &scale(&parameter, &ray)));
        let previous = self.previous(side);
        let start = self.vertices[previous].clone();
        let fraction = div(
            &self.distance(reference, &start, &boundary),
            &self.lengths[side],
        );
        let fraction = max(&min(&fraction, &int(1)), &int(0));
        let edge = flat_sub(&self.polygon[side], &self.polygon[previous]);
        let flat = flat_add(&self.polygon[previous], &flat_scale(&fraction, &edge));
        flat_add(
            &self.flat_centre,
            &flat_scale(
                &div(&int(1), &parameter),
                &flat_sub(&flat, &self.flat_centre),
            ),
        )
    }

    pub(crate) fn inverse(&self, reference: &mut Reference, point: &Flat) -> Vector {
        let ray = flat_sub(point, &self.flat_centre);
        let mut exit: Option<(BigFloat, usize)> = None;
        for side in 0..self.polygon.len() {
            let start = &self.polygon[self.previous(side)];
            let edge = flat_sub(&self.polygon[side], start);
            let outward = [edge[1].clone(), edge[0].neg()];
            let rate = flat_dot(&outward, &ray);
            if rate.is_positive() && !rate.is_zero() {
                let parameter = div(
                    &flat_dot(&outward, &flat_sub(start, &self.flat_centre)),
                    &rate,
                );
                if exit.as_ref().is_none_or(|(least, _)| parameter < *least) {
                    exit = Some((parameter, side));
                }
            }
        }
        let Some((parameter, side)) = exit else {
            return self.centre.clone();
        };
        let start = &self.polygon[self.previous(side)];
        let edge = flat_sub(&self.polygon[side], start);
        let boundary = flat_add(&self.flat_centre, &flat_scale(&parameter, &ray));
        let fraction = div(
            &flat_dot(&flat_sub(&boundary, start), &edge),
            &flat_dot(&edge, &edge),
        );
        let on_side = self.chart(&self.along_side(reference, side, &fraction));
        let chart = add_vectors(
            &self.chart_centre,
            &scale(
                &div(&int(1), &parameter),
                &difference(&on_side, &self.chart_centre),
            ),
        );
        self.unchart(&chart)
    }

    pub(crate) fn image_position(
        &self,
        reference: &mut Reference,
        point: &Flat,
        width: u32,
        height: u32,
    ) -> [f64; 2] {
        placement_position(reference, &self.polygon, point, width, height)
    }
}

pub(crate) fn placement_position(
    reference: &mut Reference,
    polygon: &[Flat],
    point: &Flat,
    width: u32,
    height: u32,
) -> [f64; 2] {
    ExactPlacement::new(polygon, width, height).position(reference, point)
}

pub(crate) struct ExactPlacement {
    factor: BigFloat,
    middle: Flat,
    half: Flat,
}

impl ExactPlacement {
    pub(crate) fn new(polygon: &[Flat], width: u32, height: u32) -> Self {
        let mut low = polygon[0].clone();
        let mut high = polygon[0].clone();
        for vertex in polygon {
            for axis in 0..2 {
                low[axis] = min(&low[axis], &vertex[axis]);
                high[axis] = max(&high[axis], &vertex[axis]);
            }
        }
        let shorter = int(width.min(height));
        let margin = div(&shorter, &int(16));
        let size = [int(width), int(height)];
        let scale_of = |axis: usize| div(&sub(&size[axis], &margin), &sub(&high[axis], &low[axis]));
        Self {
            factor: min(&scale_of(0), &scale_of(1)),
            middle: [0, 1].map(|axis| div(&add(&low[axis], &high[axis]), &int(2))),
            half: [0, 1].map(|axis| div(&size[axis], &int(2))),
        }
    }

    pub(crate) fn position(&self, reference: &mut Reference, point: &Flat) -> [f64; 2] {
        let across = add(
            &self.half[0],
            &mul(&self.factor, &sub(&point[0], &self.middle[0])),
        );
        let down = sub(
            &self.half[1],
            &mul(&self.factor, &sub(&point[1], &self.middle[1])),
        );
        [reference.approximate(&across), reference.approximate(&down)]
    }

    pub(crate) fn inside_edge(
        &self,
        reference: &mut Reference,
        [start, end]: [&Flat; 2],
        fraction: f64,
        depth: f64,
    ) -> [f64; 2] {
        let [across, down] = self.position(reference, &flat_between(start, end, &num(fraction)));
        let [first, last] =
            [start, end].map(|point| point.each_ref().map(|value| reference.approximate(value)));
        let run = [last[0] - first[0], last[1] - first[1]];
        let length = run[0].hypot(run[1]);
        [
            across - run[1] / length * depth,
            down - run[0] / length * depth,
        ]
    }
}

pub(crate) fn flat_between(start: &Flat, end: &Flat, fraction: &BigFloat) -> Flat {
    [0, 1].map(|axis| add(&start[axis], &mul(fraction, &sub(&end[axis], &start[axis]))))
}

pub(crate) type FlatMatrix = [[BigFloat; 2]; 2];

pub(crate) fn flat_identity() -> FlatMatrix {
    [[int(1), int(0)], [int(0), int(1)]]
}

pub(crate) fn flat_apply(matrix: &FlatMatrix, point: &Flat) -> Flat {
    [0, 1].map(|row| flat_dot(&matrix[row], point))
}

pub(crate) fn flat_product(left: &FlatMatrix, right: &FlatMatrix) -> FlatMatrix {
    [0, 1].map(|row| {
        [0, 1].map(|column| {
            add(
                &mul(&left[row][0], &right[0][column]),
                &mul(&left[row][1], &right[1][column]),
            )
        })
    })
}

pub(crate) fn flat_determinant(matrix: &FlatMatrix) -> BigFloat {
    sub(
        &mul(&matrix[0][0], &matrix[1][1]),
        &mul(&matrix[0][1], &matrix[1][0]),
    )
}

fn flat_reflection(through: &Flat) -> FlatMatrix {
    let squared = flat_dot(through, through);
    let entry =
        |first: &BigFloat, second: &BigFloat| div(&mul(&int(2), &mul(first, second)), &squared);
    [
        [
            sub(&entry(&through[0], &through[0]), &int(1)),
            entry(&through[0], &through[1]),
        ],
        [
            entry(&through[0], &through[1]),
            sub(&entry(&through[1], &through[1]), &int(1)),
        ],
    ]
}

pub(crate) fn flat_face(reference: &mut Reference, order: u32, direction: &Flat) -> Vec<Flat> {
    let pi = reference.pi();
    (0..order)
        .map(|vertex| {
            let angle = div(&mul(&int(2 * vertex + 1), &pi), &int(order));
            flat_turn(direction, &[reference.cos(&angle), reference.sin(&angle)])
        })
        .collect()
}

pub(crate) fn face_triangle(
    reference: &mut Reference,
    corner: usize,
    order: u32,
    direction: &Flat,
) -> [Flat; 3] {
    let face = flat_face(reference, order, direction);
    let pi = reference.pi();
    let apothem = reference.cos(&div(&pi, &int(order)));
    let mut triangle = [
        [int(0), int(0)],
        [int(0), int(0)],
        flat_scale(&apothem, direction),
    ];
    triangle[1 - corner] = if corner == 0 {
        face[face.len() - 1].clone()
    } else {
        face[0].clone()
    };
    triangle
}

pub(crate) fn centre_mirror(corner: usize) -> usize {
    if corner == 0 { 0 } else { 2 }
}

pub(crate) fn edge_side(corner: usize) -> usize {
    if corner == 0 { 2 } else { 0 }
}

pub(crate) struct FaceElement {
    pub(crate) matrix: Matrix,
    pub(crate) flat: FlatMatrix,
    pub(crate) reflection: bool,
}

pub(crate) struct ExactFace {
    pub(crate) corner: usize,
    pub(crate) elements: Vec<FaceElement>,
    pub(crate) triangle: [Flat; 3],
    pub(crate) polygon: Vec<Flat>,
}

impl ExactFace {
    pub(crate) fn new(
        reference: &mut Reference,
        measure: Measure,
        vertices: &[Vector],
        walls: &[Wall],
        corner: usize,
        order: u32,
    ) -> Self {
        let direction = face_direction(measure, vertices, corner);
        let triangle = face_triangle(reference, corner, order, &direction);
        let polygon = flat_face(reference, order, &direction);
        let centre_mirror = centre_mirror(corner);
        let flat_centre_mirror = flat_reflection(&triangle[2]);
        let flat_vertex_mirror = flat_reflection(&triangle[1 - corner]);
        let turn = product(&walls[centre_mirror].pairing, &walls[1].pairing);
        let flat_turn = flat_product(&flat_centre_mirror, &flat_vertex_mirror);
        let mut matrix = [0, 1, 2].map(|row| [0, 1, 2].map(|column| int(u32::from(row == column))));
        let mut flat = flat_identity();
        let mut elements = Vec::new();
        for _ in 0..order {
            elements.push(FaceElement {
                matrix: matrix.clone(),
                flat: flat.clone(),
                reflection: false,
            });
            elements.push(FaceElement {
                matrix: product(&matrix, &walls[centre_mirror].pairing),
                flat: flat_product(&flat, &flat_centre_mirror),
                reflection: true,
            });
            matrix = product(&matrix, &turn);
            flat = flat_product(&flat, &flat_turn);
        }
        Self {
            corner,
            elements,
            triangle,
            polygon,
        }
    }
}

pub(crate) struct ExactWythoff {
    pub(crate) measure: Measure,
    pub(crate) vertices: [Vector; 3],
    pub(crate) generator: Vector,
    pub(crate) perpendiculars: [Option<Vector>; 3],
    pub(crate) feet: [Option<Vector>; 3],
}

impl ExactWythoff {
    pub(crate) fn new(
        measure: Measure,
        walls: &[Wall],
        vertices: [[f64; 3]; 3],
        centre: [f64; 3],
        marks: [bool; 3],
    ) -> Self {
        let centre = vector(centre);
        let vertices = vertices.map(vector);
        let covectors = [0, 1, 2].map(|side| walls[side].covector.clone());
        let active = [0, 1, 2].map(|side| marks[(side + 1) % 3]);
        let on_sphere = |point: Vector| oriented(measure, &centre, &point);
        let generator = match active {
            [true, false, false] => vertices[1].clone(),
            [false, true, false] => vertices[2].clone(),
            [false, false, true] => vertices[0].clone(),
            [false, true, true] => on_sphere(cross(
                &covectors[0],
                &difference(&covectors[1], &covectors[2]),
            )),
            [true, false, true] => on_sphere(cross(
                &covectors[1],
                &difference(&covectors[2], &covectors[0]),
            )),
            [true, true, false] => on_sphere(cross(
                &covectors[2],
                &difference(&covectors[0], &covectors[1]),
            )),
            [true, true, true] => on_sphere(cross(
                &difference(&covectors[0], &covectors[1]),
                &difference(&covectors[1], &covectors[2]),
            )),
            [false, false, false] => panic!("every mark is off"),
        };
        let mut perpendiculars = [None, None, None];
        let mut feet = [None, None, None];
        for side in 0..3 {
            if active[side] {
                let image = apply(&walls[side].pairing, &generator);
                let perpendicular = wall_covector(measure, &cross(&generator, &image));
                feet[side] = Some(on_sphere(cross(&covectors[side], &perpendicular)));
                perpendiculars[side] = Some(perpendicular);
            }
        }
        Self {
            measure,
            vertices,
            generator,
            perpendiculars,
            feet,
        }
    }

    pub(crate) fn corner_holds(&self, corner: usize, point: &Vector) -> bool {
        [corner, (corner + 1) % 3].into_iter().all(|side| {
            self.perpendiculars[side]
                .as_ref()
                .is_none_or(|perpendicular| {
                    let reference = dot(perpendicular, &self.vertices[corner]);
                    let value = dot(perpendicular, point);
                    !(mul(&reference, &value).is_negative())
                })
        })
    }
}

pub(crate) fn oriented(measure: Measure, centre: &Vector, point: &Vector) -> Vector {
    match measure {
        Measure::Hyperbolic if point[2].is_negative() => normalize(&scale(&int(1).neg(), point)),
        Measure::Hyperbolic => normalize(point),
        Measure::Euclidean => scale(&div(&int(1), &point[2]), point),
        Measure::Spherical if dot(point, centre).is_negative() => {
            unit(&scale(&int(1).neg(), point))
        }
        Measure::Spherical => unit(point),
    }
}

fn wall_covector(measure: Measure, covector: &Vector) -> Vector {
    let squared = match measure {
        Measure::Hyperbolic => sub(
            &add(
                &mul(&covector[0], &covector[0]),
                &mul(&covector[1], &covector[1]),
            ),
            &mul(&covector[2], &covector[2]),
        ),
        Measure::Euclidean => add(
            &mul(&covector[0], &covector[0]),
            &mul(&covector[1], &covector[1]),
        ),
        Measure::Spherical => dot(covector, covector),
    };
    scale(&div(&int(1), &sqrt(&squared)), covector)
}

fn add_vectors(x: &Vector, y: &Vector) -> Vector {
    [0, 1, 2].map(|index| add(&x[index], &y[index]))
}

fn min(left: &BigFloat, right: &BigFloat) -> BigFloat {
    if left <= right {
        left.clone()
    } else {
        right.clone()
    }
}

pub(crate) fn count_bound(measured: usize) -> usize {
    (8 * measured).max(8)
}
