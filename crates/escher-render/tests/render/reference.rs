#![expect(
    dead_code,
    reason = "the unit tests of the crate and the integration tests each use a different part of this module"
)]

use astro_float_num::BigFloat;
use escher_reference::{
    Complex, Matrix, Reference, Vector, add, apply, cross, difference, disk_to_half_plane,
    disk_to_klein, div, dot, half_plane_to_disk, int, klein_to_disk, max, mul, normalize, num,
    product, scale, sqrt, sub, vector,
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

pub(crate) fn count_bound(measured: usize) -> usize {
    (8 * measured).max(8)
}
