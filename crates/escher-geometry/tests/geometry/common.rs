use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use escher_geometry::hyperbolic::{
    ORIGIN, boost, covector, distance, inverse, normalize_point, reflection, rotation, side,
    translation, wall_through,
};
use glam::{DMat3, DVec3};
use proptest::test_runner::Config;

pub(crate) const CASES: usize = 100_000;

const ATTEMPTS: usize = 1000;

pub(crate) const DOMAIN_TRIPLES: [[u32; 3]; 10] = [
    [7, 3, 2],
    [1000, 3, 2],
    [1000, 1000, 1000],
    [995, 987, 976],
    [994, 991, 970],
    [995, 983, 970],
    [997, 996, 970],
    [998, 4, 4],
    [928, 4, 3],
    [999, 7, 6],
];

pub(crate) fn proptest_config() -> Config {
    if cfg!(miri) {
        let mut config = Config::with_cases(8);
        config.failure_persistence = None;
        config
    } else {
        Config::default()
    }
}

pub(crate) struct Rng {
    state: u64,
}

impl Rng {
    pub(crate) fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    pub(crate) fn unit(&mut self) -> f64 {
        let bits = (self.next() >> 12) | 0x3ff0_0000_0000_0000;
        f64::from_bits(bits) - 1.0
    }

    pub(crate) fn uniform(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.unit()
    }

    pub(crate) fn log_uniform(&mut self, low: f64, high: f64) -> f64 {
        (low.ln() + (high.ln() - low.ln()) * self.unit()).exp()
    }

    pub(crate) fn sign(&mut self) -> f64 {
        if self.next() & 1 == 0 { 1.0 } else { -1.0 }
    }

    pub(crate) fn choice(&mut self, count: u64) -> u64 {
        self.next() % count
    }

    pub(crate) fn angle(&mut self) -> f64 {
        self.uniform(-PI, PI)
    }

    pub(crate) fn point_within(&mut self, radius: f64) -> DVec3 {
        let distance = self.uniform(0.0, radius);
        let angle = self.angle();
        polar_point(distance, angle)
    }

    pub(crate) fn factor(&mut self, longest_boost: f64) -> Factor {
        match self.choice(3) {
            0 => Factor::Boost(self.uniform(-longest_boost, longest_boost)),
            1 => Factor::Rotation(self.angle()),
            _ => Factor::Reflection(self.point_within(3.0), self.point_within(3.0)),
        }
    }

    pub(crate) fn product(&mut self, longest_boost: f64) -> Product {
        for _ in 0..ATTEMPTS {
            let count = 1 + self.choice(3);
            let factors: Vec<Factor> = (0..count).map(|_| self.factor(longest_boost)).collect();
            if let Some(product) = Product::of(&factors) {
                return product;
            }
        }
        panic!("no product of factors spans its walls after {ATTEMPTS} attempts");
    }

    pub(crate) fn wall(&mut self) -> Wall {
        for _ in 0..ATTEMPTS {
            if let Some(wall) = Wall::through(self.point_within(3.0), self.point_within(3.0)) {
                return wall;
            }
        }
        panic!("no two points span a wall after {ATTEMPTS} attempts");
    }

    pub(crate) fn separated_pair(&mut self) -> (DVec3, DVec3) {
        for _ in 0..ATTEMPTS {
            let (first, second) = (self.point_within(3.0), self.point_within(3.0));
            if distance(first, second) >= 0.1 {
                return (first, second);
            }
        }
        panic!("no two points are at least 0.1 apart after {ATTEMPTS} attempts");
    }
}

pub(crate) fn polar_point(distance: f64, angle: f64) -> DVec3 {
    DVec3::new(
        angle.cos() * distance.sinh(),
        angle.sin() * distance.sinh(),
        distance.cosh(),
    )
}

pub(crate) fn boost_map(length: f64, x: DVec3) -> DVec3 {
    DVec3::new(
        x.x * length.cosh() + x.z * length.sinh(),
        x.y,
        x.z * length.cosh() + x.x * length.sinh(),
    )
}

pub(crate) fn rotation_map(angle: f64, x: DVec3) -> DVec3 {
    DVec3::new(
        x.x * angle.cos() - x.y * angle.sin(),
        x.y * angle.cos() + x.x * angle.sin(),
        x.z,
    )
}

pub(crate) fn pair(radius: f64, angle: f64, separation: f64, direction: f64) -> (DVec3, DVec3) {
    let first = polar_point(radius, angle);
    let second = rotation_map(angle, boost_map(radius, polar_point(separation, direction)));
    (first, second)
}

pub(crate) fn wall_normal(distance: f64, direction: f64) -> DVec3 {
    rotation_map(direction, boost_map(distance, DVec3::X))
}

pub(crate) fn point_off_wall(distance: f64, direction: f64, along: f64, offset: f64) -> DVec3 {
    let foot = polar_point(along, FRAC_PI_2);
    rotation_map(direction, boost_map(distance, boost_map(offset, foot)))
}

pub(crate) fn table_points() -> Vec<DVec3> {
    let mut points = Vec::new();
    for distance in [0.0, 0.1, 1.0, 5.0, 10.0, 20.0] {
        for step in 0..8 {
            points.push(polar_point(distance, 0.1 + f64::from(step) * FRAC_PI_4));
        }
    }
    points
}

pub(crate) fn table_vectors() -> Vec<DVec3> {
    let mut vectors = Vec::new();
    for point in table_points() {
        for factor in [0.5, 1.0, 2.0] {
            vectors.push(point * factor);
        }
    }
    vectors
}

pub(crate) fn largest_entry(matrix: DMat3) -> f64 {
    matrix.abs().to_cols_array().into_iter().fold(0.0, f64::max)
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Factor {
    Boost(f64),
    Rotation(f64),
    Reflection(DVec3, DVec3),
}

impl Factor {
    pub(crate) fn matrix(self) -> Option<DMat3> {
        match self {
            Self::Boost(length) => Some(boost(length)),
            Self::Rotation(angle) => Some(rotation(angle)),
            Self::Reflection(first, second) => wall_through(first, second).ok().map(reflection),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Product {
    pub(crate) matrix: DMat3,
    pub(crate) factor_bound: f64,
    pub(crate) factors: Vec<Factor>,
}

impl Product {
    pub(crate) fn of(factors: &[Factor]) -> Option<Self> {
        let mut matrix = DMat3::IDENTITY;
        let mut factor_bound = 1.0;
        for factor in factors {
            let next = factor.matrix()?;
            factor_bound *= largest_entry(next);
            matrix *= next;
        }
        Some(Self {
            matrix,
            factor_bound,
            factors: factors.to_vec(),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Wall {
    pub(crate) first: DVec3,
    pub(crate) second: DVec3,
    pub(crate) normal: DVec3,
    pub(crate) matrix: DMat3,
}

impl Wall {
    pub(crate) fn through(first: DVec3, second: DVec3) -> Option<Self> {
        let normal = wall_through(first, second).ok()?;
        Some(Self {
            first,
            second,
            normal,
            matrix: reflection(normal),
        })
    }

    pub(crate) fn points(&self) -> Vec<DVec3> {
        let mut points = vec![self.first, self.second];
        points.extend(normalize_point(self.first + self.second).ok());
        points
    }
}

pub(crate) struct Triangle {
    pub(crate) vertices: [DVec3; 3],
    pub(crate) centre: DVec3,
    pub(crate) target: DVec3,
    pub(crate) normals: [DVec3; 3],
}

impl Triangle {
    pub(crate) fn of(orders: [u32; 3]) -> Self {
        let angles = orders.map(|order| PI / f64::from(order));
        let (sines, cosines) = (angles.map(f64::sin), angles.map(f64::cos));
        let first_length = ((cosines[1] + cosines[2] * cosines[0]) / (sines[2] * sines[0])).acosh();
        let second_length =
            ((cosines[0] + cosines[2] * cosines[1]) / (sines[2] * sines[1])).acosh();
        let placed = [
            boost(first_length) * ORIGIN,
            rotation(angles[2]) * (boost(second_length) * ORIGIN),
            ORIGIN,
        ];
        let target = normalize_point(placed[0] + placed[1] + placed[2])
            .expect("the vertices of a triangle sum to a timelike vector");
        let move_to_origin = inverse(translation(target));
        let vertices = placed.map(|vertex| move_to_origin * vertex);
        let normals = [0_usize, 1, 2].map(|index| {
            let normal = wall_through(vertices[(index + 2) % 3], vertices[index])
                .expect("two vertices of a triangle span a wall");
            if side(covector(normal), vertices[(index + 1) % 3]) > 0.0 {
                -normal
            } else {
                normal
            }
        });
        let centre = normalize_point(vertices[0] + vertices[1] + vertices[2])
            .expect("the vertices of a triangle sum to a timelike vector");
        Self {
            vertices,
            centre,
            target,
            normals,
        }
    }

    pub(crate) fn points(&self) -> [DVec3; 4] {
        [
            self.vertices[0],
            self.vertices[1],
            self.vertices[2],
            self.centre,
        ]
    }
}
