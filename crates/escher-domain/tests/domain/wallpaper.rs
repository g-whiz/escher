use core::f64::consts::{PI, TAU};

use escher_domain::{FundamentalDomain, Orientation, PairingKind, parameters};
use escher_geometry::euclidean;
use glam::{DMat2, DMat3, DVec2, DVec3};
use proptest::prelude::{Just, Strategy, prop_assert, prop_oneof, proptest};

use crate::common::{LargestResiduals, largest_coordinate_error, proptest_config, signature};
use crate::reference::{Reference, Worst, div, error, int, matrix, mul, num, sub};
use crate::shared::{
    GRID_ASPECTS, GRID_SKEWS, build_with_aspect, built, built_with_aspect, relator_texts,
};

const RECTANGULAR: [&str; 6] = ["**", "xx", "*x", "22x", "22*", "2*22"];

const LATTICES: [&str; 2] = ["o", "2222"];

const KITES: [&str; 2] = ["4*2", "3*3"];

const MARGIN: f64 = 8.0;

struct Bounds {
    name: &'static str,
    vertex: f64,
    covector: f64,
    pairing: f64,
    length: f64,
    angle_sum: f64,
    area: f64,
    shear: f64,
}

const BOUNDS: Bounds = Bounds {
    name: "other than 2222",
    vertex: 4e-15,
    covector: 2e-15,
    pairing: 4e-15,
    length: 4e-15,
    angle_sum: 1e-14,
    area: 1e-15,
    shear: 2e-15,
};

const BOUNDS_OF_2222: Bounds = Bounds {
    name: "2222",
    vertex: 1e-14,
    covector: 1e-14,
    pairing: 1e-14,
    length: 1e-14,
    angle_sum: 1e-14,
    area: 1e-15,
    shear: 4e-15,
};

const BOUNDS_OF_KITES: Bounds = Bounds {
    name: "kites",
    vertex: 1e-15,
    covector: 4e-15,
    pairing: 1e-14,
    length: 1e-15,
    angle_sum: 2e-15,
    area: 1e-15,
    shear: 1e-15,
};

const MERGE_TOLERANCE: f64 = 1e-9;

const WALL_TOLERANCE: f64 = 1e-9;

const COVOLUME_TOLERANCE: f64 = 1e-12;

const CERTIFICATE_RADIUS: f64 = 6.0;

const SAMPLE_POINTS: usize = 400;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Setting {
    pub(crate) text: &'static str,
    aspect: Option<f64>,
    skew: Option<f64>,
}

impl Setting {
    const fn lattice(text: &'static str, aspect: f64, skew: f64) -> Self {
        Self {
            text,
            aspect: Some(aspect),
            skew: Some(skew),
        }
    }

    const fn torus(aspect: f64, skew: f64) -> Self {
        Self::lattice("o", aspect, skew)
    }

    const fn rectangular(text: &'static str, aspect: f64) -> Self {
        Self {
            text,
            aspect: Some(aspect),
            skew: None,
        }
    }

    const fn kite(text: &'static str) -> Self {
        Self {
            text,
            aspect: None,
            skew: None,
        }
    }

    const fn has_no_parameter(self) -> bool {
        self.aspect.is_none()
    }

    fn bound_set(self) -> usize {
        if self.text == "2222" {
            1
        } else if self.has_no_parameter() {
            2
        } else {
            0
        }
    }

    fn bounds(self) -> &'static Bounds {
        [&BOUNDS, &BOUNDS_OF_2222, &BOUNDS_OF_KITES][self.bound_set()]
    }

    pub(crate) fn label(self) -> String {
        match (self.aspect, self.skew) {
            (Some(aspect), Some(skew)) => format!("{} at aspect {aspect}, skew {skew}", self.text),
            (Some(aspect), None) => format!("{} at aspect {aspect}", self.text),
            (None, _) => self.text.to_owned(),
        }
    }

    fn built(self) -> escher_domain::Result<FundamentalDomain> {
        build_with_aspect(self.text, self.aspect(), self.skew)
    }

    pub(crate) fn domain(self) -> FundamentalDomain {
        built_with_aspect(self.text, self.aspect(), self.skew)
    }

    fn aspect(self) -> f64 {
        self.aspect.unwrap_or(1.0)
    }

    fn area(self) -> f64 {
        if self.text == "2222" { 2.0 } else { 1.0 }
    }

    fn flat_vertices(self) -> &'static [usize] {
        match self.text {
            "22*" | "2222" => &[0, 3],
            "2*22" => &[0],
            _ => &[],
        }
    }

    fn cycles(self) -> &'static [(u32, &'static [usize])] {
        match self.text {
            "o" => &[(1, &[0, 3, 2, 1])],
            "**" => &[(1, &[0, 0, 1, 1]), (1, &[2, 2, 3, 3])],
            "xx" => &[(1, &[0, 3, 1, 2])],
            "*x" => &[(1, &[0, 0, 2, 2]), (1, &[1, 3, 3, 1])],
            "22x" => &[(2, &[0, 2]), (2, &[1, 3])],
            "22*" => &[(2, &[0]), (1, &[1, 1, 5, 5]), (1, &[2, 4, 4, 2]), (2, &[3])],
            "2*22" => &[(2, &[0]), (1, &[1, 1, 4, 4]), (2, &[2, 2]), (2, &[3, 3])],
            "2222" => &[(2, &[0]), (2, &[1, 5]), (2, &[2, 4]), (2, &[3])],
            "4*2" => &[(2, &[0, 0]), (1, &[1, 1, 3, 3]), (4, &[2])],
            "3*3" => &[(3, &[0, 0]), (1, &[1, 1, 3, 3]), (3, &[2])],
            text => panic!("{text} has no listed cycles"),
        }
    }
}

fn kite_sources(text: &str) -> (&'static str, &'static str, [u32; 3]) {
    match text {
        "4*2" => ("*442", "442", [4, 4, 2]),
        "3*3" => ("*632", "632", [6, 3, 2]),
        text => panic!("{text} is not a kite"),
    }
}

fn expected_centre(setting: Setting) -> DVec3 {
    if setting.has_no_parameter() {
        built(kite_sources(setting.text).0).centre()
    } else {
        euclidean::ORIGIN
    }
}

fn grid() -> Vec<Setting> {
    let mut settings = Vec::new();
    for aspect in GRID_ASPECTS {
        for text in LATTICES {
            settings.extend(GRID_SKEWS.map(|skew| Setting::lattice(text, aspect, skew)));
        }
        settings.extend(RECTANGULAR.map(|text| Setting::rectangular(text, aspect)));
    }
    settings.extend(KITES.map(Setting::kite));
    settings
}

pub(crate) fn named_settings() -> Vec<Setting> {
    let mut settings = Vec::new();
    for aspect in [1.0, 0.125, 8.0] {
        settings.extend(LATTICES.map(|text| Setting::lattice(text, aspect, 0.0)));
        settings.extend(RECTANGULAR.map(|text| Setting::rectangular(text, aspect)));
    }
    for text in LATTICES {
        settings.extend([-0.5, 0.25, 0.5].map(|skew| Setting::lattice(text, 1.0, skew)));
    }
    settings.extend(KITES.map(Setting::kite));
    settings
}

fn expected_vertices(setting: Setting) -> Vec<DVec3> {
    let width = setting.aspect().sqrt();
    let height = 1.0 / width;
    let lean = setting.skew.unwrap_or(0.0) * width;
    let (x, y) = (0.5 * width, 0.5 * height);
    let point = |x: f64, y: f64| DVec3::new(x, y, 1.0);
    match setting.text {
        "o" => vec![
            point(0.5 * (width - lean), -y),
            point(f64::midpoint(width, lean), y),
            point(0.5 * (lean - width), y),
            point(-f64::midpoint(width, lean), -y),
        ],
        "22*" => vec![
            point(0.0, -y),
            point(x, -y),
            point(x, y),
            point(0.0, y),
            point(-x, y),
            point(-x, -y),
        ],
        "2*22" => vec![
            point(0.0, -y),
            point(x, -y),
            point(x, y),
            point(-x, y),
            point(-x, -y),
        ],
        "4*2" => built("442").vertices().to_vec(),
        "3*3" => built("632").vertices().to_vec(),
        "2222" => {
            let shift = 0.5 * lean;
            vec![
                point(x - shift, -y),
                point(3.0 * x - shift, -y),
                point(3.0 * x + shift, y),
                point(x + shift, y),
                point(-x + shift, y),
                point(-x - shift, -y),
            ]
        }
        _ => vec![point(x, -y), point(x, y), point(-x, y), point(-x, -y)],
    }
}

type TableRow = (
    Setting,
    &'static [usize],
    &'static [PairingKind],
    &'static [&'static str],
);

const ROTATION: PairingKind = PairingKind::Rotation;

const MIRROR: PairingKind = PairingKind::Mirror;

const TRANSLATION: PairingKind = PairingKind::Translation;

const GLIDE: PairingKind = PairingKind::GlideReflection;

const TWO_TWO_TWO_TWO_KINDS: &[PairingKind] = &[
    ROTATION,
    ROTATION,
    TRANSLATION,
    ROTATION,
    ROTATION,
    TRANSLATION,
];

fn table_rows() -> [TableRow; 12] {
    [
        (
            Setting::rectangular("**", 1.0),
            &[2, 1, 0, 3],
            &[TRANSLATION, MIRROR, TRANSLATION, MIRROR],
            &["aspect"],
        ),
        (
            Setting::rectangular("xx", 1.0),
            &[2, 3, 0, 1],
            &[GLIDE, TRANSLATION, GLIDE, TRANSLATION],
            &["aspect"],
        ),
        (
            Setting::rectangular("*x", 1.0),
            &[2, 1, 0, 3],
            &[GLIDE, MIRROR, GLIDE, MIRROR],
            &["aspect"],
        ),
        (
            Setting::rectangular("22x", 1.0),
            &[2, 3, 0, 1],
            &[GLIDE; 4],
            &["aspect"],
        ),
        (
            Setting::rectangular("22*", 1.0),
            &[1, 0, 2, 4, 3, 5],
            &[ROTATION, ROTATION, MIRROR, ROTATION, ROTATION, MIRROR],
            &["aspect"],
        ),
        (
            Setting::rectangular("2*22", 1.0),
            &[1, 0, 2, 3, 4],
            &[ROTATION, ROTATION, MIRROR, MIRROR, MIRROR],
            &["aspect"],
        ),
        (
            Setting::torus(1.0, 0.0),
            &[2, 3, 0, 1],
            &[TRANSLATION; 4],
            &["aspect", "skew"],
        ),
        (
            Setting::torus(1.0, 0.25),
            &[2, 3, 0, 1],
            &[TRANSLATION; 4],
            &["aspect", "skew"],
        ),
        (
            Setting::lattice("2222", 1.0, 0.0),
            &[1, 0, 5, 4, 3, 2],
            TWO_TWO_TWO_TWO_KINDS,
            &["aspect", "skew"],
        ),
        (
            Setting::lattice("2222", 1.0, 0.25),
            &[1, 0, 5, 4, 3, 2],
            TWO_TWO_TWO_TWO_KINDS,
            &["aspect", "skew"],
        ),
        (
            Setting::kite("4*2"),
            &[0, 1, 3, 2],
            &[MIRROR, MIRROR, ROTATION, ROTATION],
            &[],
        ),
        (
            Setting::kite("3*3"),
            &[0, 1, 3, 2],
            &[MIRROR, MIRROR, ROTATION, ROTATION],
            &[],
        ),
    ]
}

#[test]
fn wallpaper_domains_have_the_vertices_sides_and_parameters_of_their_table() {
    for (setting, partners, kinds, names) in table_rows() {
        let label = setting.label();
        let domain = setting.domain();
        assert!(domain.verify().passed(), "{label}");
        let expected_vertices = expected_vertices(setting);
        assert_eq!(domain.vertices().len(), expected_vertices.len(), "{label}");
        for (vertex, expected) in domain.vertices().iter().zip(expected_vertices) {
            let difference = (*vertex - expected).abs().max_element();
            assert!(
                difference <= 2.0 * setting.bounds().vertex,
                "{label}: {difference:e}"
            );
        }
        let sides = domain.sides();
        assert_eq!(
            sides.iter().map(|side| side.partner()).collect::<Vec<_>>(),
            partners,
            "{label}"
        );
        assert_eq!(
            sides.iter().map(|side| side.kind()).collect::<Vec<_>>(),
            kinds,
            "{label}"
        );
        for side in sides {
            let expected = match side.kind() {
                PairingKind::Mirror | PairingKind::GlideReflection => Orientation::Reversing,
                _ => Orientation::Preserving,
            };
            assert_eq!(side.orientation(), expected, "{label}");
        }
        let angles = domain.corner_angles();
        for (vertex, &angle) in angles.iter().enumerate() {
            if setting.flat_vertices().contains(&vertex) {
                assert_eq!(angle, PI, "{label}: flat vertex {vertex}");
            } else {
                assert!(
                    (angle - PI).abs() > 1e-3,
                    "{label}: vertex {vertex} is flat"
                );
            }
        }
        for &(order, cycle) in setting.cycles() {
            let sum: f64 = cycle.iter().map(|&vertex| angles[vertex]).sum();
            let expected = TAU / f64::from(order);
            assert!(
                (sum - expected).abs() <= setting.bounds().angle_sum,
                "{label}: cycle {cycle:?} sums to {sum}"
            );
        }
        let count = domain.vertices().len();
        for index in
            (0..count).filter(|index| !setting.flat_vertices().contains(&((index + 1) % count)))
        {
            let [first, second, third] =
                [0, 1, 2].map(|step| domain.vertices()[(index + step) % count].truncate());
            let turn = (second - first).perp_dot(third - first);
            assert!(turn > 0.0, "{label}: vertices from {index} turn {turn:e}");
        }
        assert_eq!(domain.centre(), expected_centre(setting), "{label}");
        assert_eq!(
            domain.reverses_orientation(),
            !LATTICES.contains(&setting.text),
            "{label}"
        );
        let found: Vec<String> = parameters(&signature(setting.text))
            .iter()
            .map(|spec| spec.name().to_owned())
            .collect();
        assert_eq!(found, names, "{label}");
    }
}

#[test]
fn wallpaper_signatures_typed_in_another_order_build_the_domain_of_their_canonical_form() {
    for (typed, canonical) in [("x*", "*x"), ("x22", "22x"), ("2x2", "22x")] {
        assert_eq!(signature(typed), signature(canonical), "{typed}");
        assert_eq!(
            Setting::rectangular(typed, 1.0).domain(),
            Setting::rectangular(canonical, 1.0).domain(),
            "{typed}"
        );
    }
}

#[test]
fn wallpaper_checks_pass_with_a_margin_over_the_grid() {
    for text in LATTICES.into_iter().chain(RECTANGULAR).chain(KITES) {
        let mut largest = LargestResiduals::new();
        let settings: Vec<Setting> = grid()
            .into_iter()
            .filter(|setting| setting.text == text)
            .collect();
        for setting in &settings {
            let verification = setting.domain().verify();
            for check in [
                verification.isometries(),
                verification.pairings(),
                verification.walls(),
                verification.relators(),
                verification.angles(),
                verification.area(),
            ] {
                assert!(
                    check.residual() * MARGIN <= check.tolerance(),
                    "{}: {check:?}",
                    setting.label()
                );
            }
            largest.record_verification(&verification, &setting.label());
        }
        println!("{text} over the grid:");
        largest.print_and_assert_samples(settings.len());
    }
}

fn setting_strategy() -> impl Strategy<Value = Setting> {
    let aspect = prop_oneof![
        3 => 0.125..=8.0_f64,
        1 => prop_oneof![Just(0.125), Just(8.0)],
    ];
    let skew = prop_oneof![
        3 => -0.5..=0.5_f64,
        1 => prop_oneof![Just(-0.5), Just(0.5)],
    ];
    (aspect, skew, 0..RECTANGULAR.len() + LATTICES.len()).prop_map(|(aspect, skew, choice)| {
        RECTANGULAR.get(choice).map_or_else(
            || Setting::lattice(LATTICES[choice - RECTANGULAR.len()], aspect, skew),
            |&text| Setting::rectangular(text, aspect),
        )
    })
}

proptest! {
    #![proptest_config(proptest_config())]

    #[test]
    fn random_wallpaper_settings_build_and_pass(setting in setting_strategy()) {
        let built = setting.built();
        prop_assert!(built.is_ok(), "{}: {:?}", setting.label(), built);
        if let Ok(domain) = built {
            prop_assert!(domain.verify().passed(), "{}", setting.label());
        }
    }
}

#[test]
fn kites_build_and_pass() {
    for setting in KITES.map(Setting::kite) {
        let built = setting.built();
        assert!(built.is_ok(), "{}: {:?}", setting.label(), built);
        if let Ok(domain) = built {
            assert!(domain.verify().passed(), "{}", setting.label());
        }
    }
}

#[test]
fn wallpaper_domains_match_their_closed_forms() {
    let mut reference = Reference::new();
    let one = int(1);
    let tau = mul(&reference.pi(), &int(2));
    let mut sets = [&BOUNDS, &BOUNDS_OF_2222, &BOUNDS_OF_KITES].map(|bounds| {
        (
            bounds,
            [
                "vertex",
                "covector",
                "pairing",
                "side length",
                "angle sum of a cycle",
                "area",
            ]
            .map(|quantity| Worst::new(format!("{}, {quantity}", bounds.name), 1)),
        )
    });
    let mut flat_count = 0_usize;
    for setting in &grid() {
        let label = setting.label();
        let domain = setting.domain();
        let exact = if setting.has_no_parameter() {
            reference.kite(kite_sources(setting.text).2)
        } else {
            crate::reference::wallpaper(setting.text, setting.aspect(), setting.skew.unwrap_or(0.0))
        };
        assert_eq!(domain.centre(), expected_centre(*setting), "{label}");
        let [vertex, covector, pairing, length, angle_sum, area] = &mut sets[setting.bound_set()].1;
        let lengths = domain.side_lengths();
        for (index, side) in domain.sides().iter().enumerate() {
            vertex.record(
                &largest_coordinate_error(domain.vertices()[index], &exact.vertices[index]),
                &one,
                || format!("{label}, vertex {index}"),
            );
            covector.record(
                &largest_coordinate_error(side.covector(), &exact.covectors[index]),
                &one,
                || format!("{label}, side {index}"),
            );
            let computed = matrix(side.pairing().to_cols_array());
            let mut largest = int(0);
            for (row, exact_row) in computed.iter().zip(&exact.pairings[index]) {
                for (entry, exact_entry) in row.iter().zip(exact_row) {
                    largest = crate::reference::max(&largest, &sub(entry, exact_entry).abs());
                }
            }
            pairing.record(&largest, &one, || format!("{label}, side {index}"));
            length.record(&error(lengths[index], &exact.lengths[index]), &one, || {
                format!("{label}, side {index}")
            });
        }
        let angles = domain.corner_angles();
        for &vertex in setting.flat_vertices() {
            assert_eq!(angles[vertex], PI, "{label}: flat vertex {vertex}");
            flat_count += 1;
        }
        for (position, &(order, cycle)) in setting.cycles().iter().enumerate() {
            let sum = cycle.iter().fold(0.0, |sum, &vertex| sum + angles[vertex]);
            angle_sum.record(&error(sum, &div(&tau, &int(order))), &one, || {
                format!("{label}, cycle {position}")
            });
        }
        let exact_area = if setting.has_no_parameter() {
            exact.area.clone()
        } else {
            num(setting.area())
        };
        area.record(&error(domain.area(), &exact_area), &one, || label.clone());
    }
    for (bounds, worsts) in &sets {
        let limits = [
            bounds.vertex,
            bounds.covector,
            bounds.pairing,
            bounds.length,
            bounds.angle_sum,
            bounds.area,
        ];
        for (worst, bound) in worsts.iter().zip(limits) {
            worst.assert_at_most(&mut reference, bound);
        }
    }
    assert!(flat_count > 0);
    println!("flat vertices: {flat_count}, each measured exactly as the f64 nearest pi");
    println!(
        "centre: exactly the origin for every domain but the kites, which keep the centre of their triangle"
    );
}

#[test]
fn skewed_domains_are_shears_of_their_unskewed_domains() {
    let mut reference = Reference::new();
    let one = int(1);
    for text in LATTICES {
        let bounds = Setting::lattice(text, 1.0, 0.0).bounds();
        let mut vertex = Worst::new(format!("{text}, sheared vertex"), 1);
        let mut area = Worst::new(format!("{text}, sheared area"), 1);
        for aspect in GRID_ASPECTS {
            let unskewed = Setting::lattice(text, aspect, 0.0).domain();
            for skew in GRID_SKEWS.into_iter().filter(|&skew| skew != 0.0) {
                let setting = Setting::lattice(text, aspect, skew);
                let label = setting.label();
                let skewed = setting.domain();
                let factor = mul(&num(skew), &num(aspect));
                for (index, (&moved, &start)) in skewed
                    .vertices()
                    .iter()
                    .zip(unskewed.vertices())
                    .enumerate()
                {
                    let [x, y, z] = [start.x, start.y, start.z].map(num);
                    let exact = [crate::reference::add(&x, &mul(&factor, &y)), y, z];
                    vertex.record(&largest_coordinate_error(moved, &exact), &one, || {
                        format!("{label}, vertex {index}")
                    });
                }
                area.record(&error(skewed.area(), &num(unskewed.area())), &one, || {
                    label.clone()
                });
            }
        }
        vertex.assert_at_most(&mut reference, bounds.shear);
        area.assert_at_most(&mut reference, bounds.area);
    }
}

struct Element {
    matrix: DMat3,
}

impl Element {
    fn linear(&self) -> DMat2 {
        DMat2::from_cols(self.matrix.x_axis.truncate(), self.matrix.y_axis.truncate())
    }

    fn shift(&self) -> DVec2 {
        self.matrix.z_axis.truncate()
    }

    fn apply(&self, point: DVec2) -> DVec2 {
        self.linear() * point + self.shift()
    }
}

fn close(first: DMat3, second: DMat3) -> bool {
    (first - second)
        .to_cols_array()
        .into_iter()
        .all(|entry| entry.abs() <= MERGE_TOLERANCE)
}

fn close_linear(first: DMat2, second: DMat2) -> bool {
    (first - second)
        .to_cols_array()
        .into_iter()
        .all(|entry| entry.abs() <= MERGE_TOLERANCE)
}

fn enumerate_group(domain: &FundamentalDomain) -> Vec<Element> {
    let centre = domain.centre().truncate();
    let generators: Vec<DMat3> = domain
        .sides()
        .iter()
        .flat_map(|side| [side.pairing(), side.pairing().inverse()])
        .collect();
    let explored = 2.0 * CERTIFICATE_RADIUS;
    let mut found = vec![DMat3::IDENTITY];
    let mut next = 0;
    while next < found.len() {
        let element = found[next];
        next += 1;
        for &generator in &generators {
            let product = element * generator;
            let moved = (product * centre.extend(1.0)).truncate();
            if moved.distance(centre) > explored {
                continue;
            }
            if !found.iter().any(|&known| close(known, product)) {
                found.push(product);
            }
        }
    }
    found
        .into_iter()
        .filter(|&element| {
            (element * centre.extend(1.0)).truncate().distance(centre) <= CERTIFICATE_RADIUS
        })
        .map(|matrix| Element { matrix })
        .collect()
}

fn samples() -> Vec<DVec2> {
    let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let bits = u32::try_from(state >> 40).unwrap_or(0);
        f64::from(bits) / f64::from(1_u32 << 24)
    };
    (0..SAMPLE_POINTS)
        .map(|_| DVec2::new(4.0 * next() - 2.0, 4.0 * next() - 2.0))
        .collect()
}

struct Classes {
    point_group: u32,
    covolume: f64,
    reflection: bool,
    glide_off_mirrors: bool,
    cone_orders: Vec<u32>,
    corner_orders: Vec<u32>,
}

struct Centre {
    point: DVec2,
    order: u32,
    on_mirror: bool,
}

fn centre_classes(
    elements: &[Element],
    centre: DVec2,
    on_mirror: &dyn Fn(DVec2, Option<DMat2>) -> bool,
) -> (Vec<u32>, Vec<u32>) {
    let mut centres: Vec<Centre> = Vec::new();
    for element in elements {
        let linear = element.linear();
        if linear.determinant() < 0.0 || close_linear(linear, DMat2::IDENTITY) {
            continue;
        }
        let angle = linear.x_axis.y.atan2(linear.x_axis.x).abs();
        let Some(order) =
            (2..=6_u32).find(|&order| (f64::from(order) * angle - TAU).abs() <= MERGE_TOLERANCE)
        else {
            panic!("a rotation through {angle} has no order from 2 to 6");
        };
        let point = (DMat2::IDENTITY - linear).inverse() * element.shift();
        if point.distance(centre) > 0.5 * CERTIFICATE_RADIUS {
            continue;
        }
        match centres
            .iter_mut()
            .find(|known| known.point.distance(point) <= MERGE_TOLERANCE)
        {
            Some(known) => known.order = known.order.max(order),
            None => centres.push(Centre {
                point,
                order,
                on_mirror: on_mirror(point, None),
            }),
        }
    }
    let mut class_of: Vec<usize> = (0..centres.len()).collect();
    for (index, first) in centres.iter().enumerate() {
        for (other, second) in centres.iter().enumerate().skip(index + 1) {
            let related = elements.iter().any(|element| {
                element.apply(first.point).distance(second.point) <= MERGE_TOLERANCE
            });
            if related {
                let (from, to) = (class_of[other], class_of[index]);
                for class in &mut class_of {
                    if *class == from {
                        *class = to;
                    }
                }
            }
        }
    }
    let (mut cone_orders, mut corner_orders) = (Vec::new(), Vec::new());
    for (index, found) in centres.iter().enumerate() {
        if class_of[index] != index {
            continue;
        }
        if found.on_mirror {
            corner_orders.push(found.order);
        } else {
            cone_orders.push(found.order);
        }
    }
    cone_orders.sort_unstable();
    corner_orders.sort_unstable();
    (cone_orders, corner_orders)
}

fn classify(elements: &[Element], centre: DVec2) -> Classes {
    let mut linear: Vec<DMat2> = Vec::new();
    for element in elements {
        if !linear
            .iter()
            .any(|&known| close_linear(known, element.linear()))
        {
            linear.push(element.linear());
        }
    }
    let translations: Vec<DVec2> = elements
        .iter()
        .filter(|element| close_linear(element.linear(), DMat2::IDENTITY))
        .map(Element::shift)
        .filter(|shift| shift.length() > MERGE_TOLERANCE)
        .collect();
    let mut covolume = f64::INFINITY;
    for (index, first) in translations.iter().enumerate() {
        for second in &translations[index + 1..] {
            let area = first.perp_dot(*second).abs();
            if area > MERGE_TOLERANCE {
                covolume = covolume.min(area);
            }
        }
    }
    let reversing = |element: &&Element| element.linear().determinant() < 0.0;
    let mirrors: Vec<&Element> = elements
        .iter()
        .filter(reversing)
        .filter(|element| close(element.matrix * element.matrix, DMat3::IDENTITY))
        .collect();
    let on_mirror = |point: DVec2, direction: Option<DMat2>| {
        mirrors.iter().any(|mirror| {
            mirror.apply(point).distance(point) <= MERGE_TOLERANCE
                && direction.is_none_or(|linear| close_linear(linear, mirror.linear()))
        })
    };
    let glide_off_mirrors = elements
        .iter()
        .filter(reversing)
        .filter(|element| !close(element.matrix * element.matrix, DMat3::IDENTITY))
        .any(|element| {
            let shift = element.shift();
            let along = 0.5 * (element.linear() * shift + shift);
            !on_mirror(0.5 * (shift - along), Some(element.linear()))
        });
    let (cone_orders, corner_orders) = centre_classes(elements, centre, &on_mirror);
    Classes {
        point_group: u32::try_from(linear.len()).unwrap_or(u32::MAX),
        covolume,
        reflection: !mirrors.is_empty(),
        glide_off_mirrors,
        cone_orders,
        corner_orders,
    }
}

fn cone_and_corner_orders(text: &str) -> (Vec<u32>, Vec<u32>) {
    let parsed = signature(text);
    let mut cones: Vec<u32> = parsed
        .cone_orders()
        .iter()
        .filter_map(|order| order.finite())
        .collect();
    let mut corners: Vec<u32> = parsed
        .boundaries()
        .iter()
        .flat_map(|boundary| {
            boundary
                .corner_orders()
                .iter()
                .filter_map(|order| order.finite())
        })
        .collect();
    cones.sort_unstable();
    corners.sort_unstable();
    (cones, corners)
}

fn is_inside(domain: &FundamentalDomain, point: DVec2) -> bool {
    domain
        .sides()
        .iter()
        .all(|side| euclidean::side(side.covector(), point.extend(1.0)) <= WALL_TOLERANCE)
}

fn point_group_order(text: &str) -> u32 {
    match text {
        "o" => 1,
        "**" | "xx" | "*x" | "2222" => 2,
        "333" => 3,
        "22x" | "22*" | "2*22" | "442" => 4,
        "3*3" | "*333" | "632" => 6,
        "4*2" | "*442" => 8,
        "*632" => 12,
        text => panic!("{text} has no listed point group"),
    }
}

fn certified(
    text: &str,
    label: &str,
    domain: &FundamentalDomain,
    points: &[DVec2],
) -> (usize, Classes) {
    let elements = enumerate_group(domain);
    for (index, &point) in points.iter().enumerate() {
        let images = elements
            .iter()
            .filter(|element| is_inside(domain, element.apply(point)))
            .count();
        assert_eq!(images, 1, "{label}: sample {index} at {point:?}");
    }
    let classes = classify(&elements, domain.centre().truncate());
    let expected_order = point_group_order(text);
    assert_eq!(classes.point_group, expected_order, "{label}");
    let expected_covolume = f64::from(expected_order) * domain.area();
    assert!(
        (classes.covolume - expected_covolume).abs() <= COVOLUME_TOLERANCE,
        "{label}: covolume {} against {expected_covolume}",
        classes.covolume
    );
    let (cones, corners) = cone_and_corner_orders(text);
    assert_eq!(classes.cone_orders, cones, "{label}");
    assert_eq!(classes.corner_orders, corners, "{label}");
    assert_eq!(
        classes.reflection,
        !signature(text).boundaries().is_empty(),
        "{label}"
    );
    (elements.len(), classes)
}

#[test]
fn wallpaper_groups_have_the_point_groups_lattices_and_isometries_of_their_signatures() {
    let points = samples();
    for setting in named_settings() {
        let label = setting.label();
        let domain = setting.domain();
        let (elements, classes) = certified(setting.text, &label, &domain, &points);
        assert_eq!(
            classes.glide_off_mirrors,
            matches!(
                setting.text,
                "xx" | "*x" | "22x" | "22*" | "2*22" | "4*2" | "3*3"
            ),
            "{label}"
        );
        let expected_covolume = f64::from(classes.point_group) * domain.area();
        println!(
            "{label}: {elements} elements within {CERTIFICATE_RADIUS}, point group of order {}, reflection {}, glide reflection off the mirror lines {}, cone orders {:?}, corner orders {:?}, covolume error {:e}",
            classes.point_group,
            classes.reflection,
            classes.glide_off_mirrors,
            classes.cone_orders,
            classes.corner_orders,
            (classes.covolume - expected_covolume).abs()
        );
    }
}

fn bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|value| value.to_bits()).collect()
}

fn domain_bits(domain: &FundamentalDomain) -> Vec<u64> {
    let mut found = Vec::new();
    for vertex in domain.vertices() {
        found.extend(bits(&vertex.to_array()));
    }
    for side in domain.sides() {
        found.extend(bits(&side.normal().to_array()));
        found.extend(bits(&side.covector().to_array()));
        found.extend(bits(&side.pairing().to_cols_array()));
    }
    found.extend(bits(&domain.centre().to_array()));
    let verification = domain.verify();
    for check in [
        verification.isometries(),
        verification.pairings(),
        verification.walls(),
        verification.relators(),
        verification.angles(),
        verification.area(),
    ] {
        found.extend(bits(&[check.residual(), check.tolerance()]));
    }
    found
}

fn presentation_text(domain: &FundamentalDomain, label: &str) -> Vec<String> {
    match domain.side_presentation() {
        Ok(presentation) => relator_texts(&presentation),
        Err(error) => panic!("{label} has no side presentation: {error}"),
    }
}

#[test]
fn skew_of_zero_leaves_the_domain_of_2222_bit_for_bit() {
    for aspect in [0.125, 1.0, 1.618, 8.0] {
        let unset = built_with_aspect("2222", aspect, None);
        for skew in [0.0, -0.0] {
            let label = format!("2222 at aspect {aspect}, skew {skew:?}");
            let set = built_with_aspect("2222", aspect, Some(skew));
            assert_eq!(domain_bits(&set), domain_bits(&unset), "{label}");
            for (side, kept) in set.sides().iter().zip(unset.sides()) {
                assert_eq!(
                    (side.partner(), side.kind(), side.orientation()),
                    (kept.partner(), kept.kind(), kept.orientation()),
                    "{label}"
                );
            }
            assert_eq!(
                presentation_text(&set, &label),
                presentation_text(&unset, &label),
                "{label}"
            );
        }
    }
}

#[test]
fn kites_keep_the_entries_of_their_triangles_and_doubles_bit_for_bit() {
    for text in KITES {
        let (reflection, rotation, _) = kite_sources(text);
        let kite = built(text);
        let triangle = built(reflection);
        let double = built(rotation);
        assert_eq!(
            domain_bits_of_vertices(&kite),
            domain_bits_of_vertices(&double),
            "{text}"
        );
        assert_eq!(
            kite.centre().to_array().map(f64::to_bits),
            triangle.centre().to_array().map(f64::to_bits),
            "{text}"
        );
        assert_eq!(
            double.centre().to_array().map(f64::to_bits),
            triangle.centre().to_array().map(f64::to_bits),
            "{text}"
        );
        for (index, (side, doubled)) in kite.sides().iter().zip(double.sides()).enumerate() {
            assert_eq!(
                bits(&side.normal().to_array()),
                bits(&doubled.normal().to_array()),
                "{text}, side {index}"
            );
            assert_eq!(
                bits(&side.covector().to_array()),
                bits(&doubled.covector().to_array()),
                "{text}, side {index}"
            );
        }
        let pairing_bits = |matrix: DMat3| bits(&matrix.to_cols_array());
        let sides = kite.sides();
        assert_eq!(
            pairing_bits(sides[0].pairing()),
            pairing_bits(triangle.sides()[0].pairing()),
            "{text}, side 0"
        );
        assert_eq!(
            pairing_bits(sides[1].pairing()),
            pairing_bits(euclidean::reflection(sides[1].covector())),
            "{text}, side 1"
        );
        for index in [2, 3] {
            assert_eq!(
                pairing_bits(sides[index].pairing()),
                pairing_bits(double.sides()[index].pairing()),
                "{text}, side {index}"
            );
        }
        for (index, side) in sides.iter().enumerate().take(2) {
            assert_eq!(side.kind(), PairingKind::Mirror, "{text}, side {index}");
            assert_eq!(side.partner(), index, "{text}, side {index}");
        }
    }
}

fn domain_bits_of_vertices(domain: &FundamentalDomain) -> Vec<u64> {
    domain
        .vertices()
        .iter()
        .flat_map(|vertex| bits(&vertex.to_array()))
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Feature {
    PointGroup,
    Reflection,
    CentresOffMirrors,
    CentresOnMirrors,
}

const FEATURES: [Feature; 4] = [
    Feature::PointGroup,
    Feature::Reflection,
    Feature::CentresOffMirrors,
    Feature::CentresOnMirrors,
];

fn differs(feature: Feature, first: &Classes, second: &Classes) -> bool {
    match feature {
        Feature::PointGroup => first.point_group != second.point_group,
        Feature::Reflection => first.reflection != second.reflection,
        Feature::CentresOffMirrors => first.cone_orders != second.cone_orders,
        Feature::CentresOnMirrors => first.corner_orders != second.corner_orders,
    }
}

type Control = (&'static str, &'static [Feature]);

#[test]
fn wallpaper_certificate_tells_the_kites_from_the_groups_of_their_triangles() {
    use Feature::{CentresOffMirrors, CentresOnMirrors, PointGroup, Reflection};
    let points = samples();
    let classes_of = |text: &str| {
        let (_, classes) = certified(text, text, &built(text), &points);
        println!(
            "{text}: point group of order {}, reflection {}, rotation centres off the mirror lines {:?}, on the mirror lines {:?}",
            classes.point_group, classes.reflection, classes.cone_orders, classes.corner_orders
        );
        classes
    };
    let cases: [(&str, &[Control]); 2] = [
        (
            "4*2",
            &[
                ("*442", &[CentresOffMirrors, CentresOnMirrors]),
                (
                    "442",
                    &[PointGroup, Reflection, CentresOffMirrors, CentresOnMirrors],
                ),
            ],
        ),
        (
            "3*3",
            &[
                ("*333", &[CentresOffMirrors, CentresOnMirrors]),
                (
                    "333",
                    &[PointGroup, Reflection, CentresOffMirrors, CentresOnMirrors],
                ),
                ("632", &[Reflection, CentresOffMirrors, CentresOnMirrors]),
                ("*632", &[PointGroup, CentresOffMirrors, CentresOnMirrors]),
            ],
        ),
    ];
    for (kite, controls) in cases {
        let found = classes_of(kite);
        assert!(found.reflection, "{kite}");
        assert!(!found.cone_orders.is_empty(), "{kite}");
        for &(control, expected) in controls {
            let other = classes_of(control);
            for feature in FEATURES {
                assert_eq!(
                    differs(feature, &found, &other),
                    expected.contains(&feature),
                    "{kite} against {control}: {feature:?}"
                );
            }
        }
    }
}
