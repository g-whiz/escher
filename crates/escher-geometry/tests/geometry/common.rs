use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use escher_geometry::hyperbolic::{
    ORIGIN, boost, covector, distance, inverse, normalize_point, reflection, rotation, side,
    translation, wall_through,
};
use escher_geometry::spherical;
use glam::{DMat3, DVec3};
use proptest::test_runner::Config;

pub(crate) const CASES: usize = 100_000;

pub(crate) const EUCLIDEAN_RADIUS: f64 = 64.0;

const LINE_ANGLES: u32 = 64;

const SMALLEST_SEPARATION: f64 = 0.1;

const LARGEST_SEPARATION: f64 = 128.0;

const ATTEMPTS: usize = 1000;

pub(crate) const SPHERE_SEPARATION_FLOOR: f64 = 1e-3;

pub(crate) const SPHERE_DOMAIN_ORDERS: [u32; 7] = [2, 3, 5, 12, 100, 999, 1000];

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

    pub(crate) fn euclidean_point_within(&mut self, radius: f64) -> DVec3 {
        let distance = self.uniform(0.0, radius);
        let angle = self.angle();
        euclidean_point(distance, angle)
    }

    pub(crate) fn line_covector(&mut self, index: u32) -> DVec3 {
        let angle = 2.0 * PI * f64::from(index % LINE_ANGLES) / f64::from(LINE_ANGLES);
        let offset = self.uniform(-EUCLIDEAN_RADIUS, EUCLIDEAN_RADIUS);
        line_covector(angle, offset)
    }

    pub(crate) fn euclidean_pair(&mut self) -> (DVec3, DVec3) {
        for _ in 0..ATTEMPTS {
            let first = self.euclidean_point_within(EUCLIDEAN_RADIUS);
            let separation = self.log_uniform(SMALLEST_SEPARATION, LARGEST_SEPARATION);
            let direction = self.angle();
            let second = first + euclidean_point(separation, direction) - DVec3::Z;
            if second.x.hypot(second.y) <= EUCLIDEAN_RADIUS {
                return (first, second);
            }
        }
        panic!("no second point lies within {EUCLIDEAN_RADIUS} after {ATTEMPTS} attempts");
    }

    pub(crate) fn sphere_point(&mut self) -> DVec3 {
        let height = self.uniform(-1.0, 1.0);
        sphere_point(height, self.angle())
    }

    pub(crate) fn sphere_separation(&mut self, floor: f64) -> f64 {
        let towards_end = self.log_uniform(floor, FRAC_PI_2);
        if self.choice(2) == 0 {
            towards_end
        } else {
            PI - towards_end
        }
    }

    pub(crate) fn sphere_pair(&mut self, floor: f64) -> (DVec3, DVec3) {
        let first = self.sphere_point();
        let separation = self.sphere_separation(floor);
        let second = sphere_step(first, self.angle(), separation);
        (first, second)
    }

    pub(crate) fn sphere_factor(&mut self) -> SphereFactor {
        match self.choice(3) {
            0 => SphereFactor::Boost(self.uniform(-10.0, 10.0)),
            1 => SphereFactor::Rotation(self.uniform(-10.0, 10.0)),
            _ => {
                let (first, second) = self.sphere_pair(SPHERE_SEPARATION_FLOOR);
                SphereFactor::Reflection(first, second)
            }
        }
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

pub(crate) fn sphere_point(height: f64, angle: f64) -> DVec3 {
    let radius = (1.0 - height * height).sqrt();
    DVec3::new(radius * angle.cos(), radius * angle.sin(), height)
}

pub(crate) fn tangent_basis(point: DVec3) -> (DVec3, DVec3) {
    let helper = if point.x.abs() < 0.5 {
        DVec3::X
    } else {
        DVec3::Y
    };
    let first = (helper - point * point.dot(helper)).normalize();
    (first, point.cross(first))
}

pub(crate) fn sphere_step(point: DVec3, direction: f64, distance: f64) -> DVec3 {
    let (first, second) = tangent_basis(point);
    let tangent = first * direction.cos() + second * direction.sin();
    point * distance.cos() + tangent * distance.sin()
}

pub(crate) fn euclidean_point(distance: f64, angle: f64) -> DVec3 {
    DVec3::new(distance * angle.cos(), distance * angle.sin(), 1.0)
}

pub(crate) fn line_covector(angle: f64, offset: f64) -> DVec3 {
    DVec3::new(angle.cos(), angle.sin(), -offset)
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

#[derive(Clone, Copy, Debug)]
pub(crate) enum SphereFactor {
    Boost(f64),
    Rotation(f64),
    Reflection(DVec3, DVec3),
}

impl SphereFactor {
    pub(crate) fn matrix(self) -> Option<DMat3> {
        match self {
            Self::Boost(angle) => Some(spherical::boost(angle)),
            Self::Rotation(angle) => Some(spherical::rotation(angle)),
            Self::Reflection(first, second) => spherical::wall_through(first, second)
                .ok()
                .map(spherical::reflection),
        }
    }
}

pub(crate) struct SphereDomain {
    pub(crate) name: String,
    pub(crate) normals: Vec<DVec3>,
    pub(crate) points: Vec<DVec3>,
}

impl SphereDomain {
    pub(crate) fn triangle(orders: [u32; 3]) -> Self {
        let angles = orders.map(|order| PI / f64::from(order));
        let (sines, cosines) = (angles.map(f64::sin), angles.map(f64::cos));
        let length =
            |numerator: f64, denominator: f64| (numerator / denominator).clamp(-1.0, 1.0).acos();
        let first_length = length(cosines[1] + cosines[2] * cosines[0], sines[2] * sines[0]);
        let second_length = length(cosines[0] + cosines[2] * cosines[1], sines[2] * sines[1]);
        let origin = spherical::ORIGIN;
        let placed = [
            spherical::boost(first_length) * origin,
            spherical::rotation(angles[2]) * (spherical::boost(second_length) * origin),
            origin,
        ];
        let target = spherical::normalize_point(placed[0] + placed[1] + placed[2])
            .expect("the vertices of a spherical triangle have a non-zero sum");
        let move_to_origin = spherical::inverse(spherical::translation(target));
        let vertices = placed.map(|vertex| move_to_origin * vertex);
        let normals = [0_usize, 1, 2].map(|index| {
            let normal = spherical::wall_through(vertices[(index + 2) % 3], vertices[index])
                .expect("two vertices of a spherical triangle span a great circle");
            if spherical::side(normal, vertices[(index + 1) % 3]) > 0.0 {
                -normal
            } else {
                normal
            }
        });
        let centre = spherical::normalize_point(vertices[0] + vertices[1] + vertices[2])
            .expect("the vertices of a spherical triangle have a non-zero sum");
        let [n0, n1, n2] = orders;
        Self {
            name: format!("*{n0},{n1},{n2}"),
            normals: normals.to_vec(),
            points: vec![vertices[0], vertices[1], vertices[2], centre],
        }
    }

    pub(crate) fn lune(order: u32) -> Self {
        let angle = PI / f64::from(order);
        let (sin, cos) = (angle / 2.0).sin_cos();
        Self {
            name: format!("*{order},{order}"),
            normals: vec![DVec3::new(cos, 0.0, -sin), DVec3::new(-cos, 0.0, -sin)],
            points: vec![DVec3::Y, -DVec3::Y, spherical::ORIGIN],
        }
    }

    pub(crate) fn all() -> Vec<Self> {
        let mut domains: Vec<Self> = [[3, 3, 2], [4, 3, 2], [5, 3, 2], [2, 2, 2]]
            .into_iter()
            .map(Self::triangle)
            .collect();
        for order in SPHERE_DOMAIN_ORDERS {
            domains.push(Self::triangle([order, 2, 2]));
            domains.push(Self::lune(order));
        }
        domains
    }
}
