use std::time::{Duration, Instant};

use escher_domain::{FundamentalDomain, PairingKind};
use escher_geometry::{hyperbolic, spherical};
use escher_group::{
    CosetLimit, IndexLimit, Letter, Presentation, Relator, SearchBudget, Word, conway_presentation,
    count_colourings, enumerate,
};
use escher_signature::Geometry;
use glam::{DMat3, DVec3};

use crate::common::{
    EXAMPLE_TRIPLES, Kite, LIMIT_ORDER, SMALL_ORDER, TWO_MIRROR_LIMIT_ORDER, built,
    clear_rotation_polygons, cone_text, order_text, signature, small_kites, starred, test_set,
    text, triangles,
};
use crate::reference::{
    Matrix, Reference, add, div, identity, int, largest_entry_from_identity as largest_entry,
    matrix, mul, power, product as exact_product, sqrt, sub, vector,
};
use crate::shared::{NAMED_POLYGONS, relator_text, relator_texts};
use crate::wallpaper::named_settings;

const PAIRING_TOLERANCE: f64 = 1e-9;

const RELATOR_TOLERANCE: f64 = 2e-6;

const ACCURATE_RELATOR_VERTICES: usize = 5;

fn presentation(text: &str) -> (FundamentalDomain, Presentation) {
    let domain = built(text);
    match domain.side_presentation() {
        Ok(presentation) => (domain, presentation),
        Err(error) => panic!("{text} has no side presentation: {error}"),
    }
}

fn side_names(count: usize) -> Vec<String> {
    (0..count).map(|side| format!("s{side}")).collect()
}

fn lune(order: u32) -> String {
    let order = order_text(order);
    format!("{order}{order}")
}

fn spherical_rotations(orders: impl Iterator<Item = u32> + Clone) -> Vec<String> {
    let mut texts: Vec<String> = ["332", "432", "532"].map(str::to_owned).to_vec();
    texts.extend(orders.clone().map(|order| cone_text([order, 2, 2])));
    texts.extend(orders.map(lune));
    texts
}

fn with_reflections(rotations: &[String]) -> Vec<String> {
    let mut texts: Vec<String> = rotations.iter().map(|text| starred(text)).collect();
    texts.extend(rotations.iter().cloned());
    texts
}

fn spherical_signatures() -> Vec<String> {
    with_reflections(&spherical_rotations(2..=SMALL_ORDER))
}

fn limit_signatures() -> Vec<String> {
    with_reflections(&[cone_text([LIMIT_ORDER, 2, 2]), lune(LIMIT_ORDER)])
}

fn small_spherical_kites() -> Vec<String> {
    small_kites().into_iter().map(Kite::text).collect()
}

fn spherical_kites() -> Vec<String> {
    test_set().into_iter().map(Kite::text).collect()
}

const CYCLE_RELATOR_KITES: [&str; 4] = ["3*2", "3*", "2*3", "2*"];

#[test]
fn side_presentations_have_the_documented_text() {
    let cases: [(&str, &[&str]); 5] = [
        (
            "*532",
            &[
                "s0^2",
                "s1^2",
                "s2^2",
                "(s0 s1)^5",
                "(s1 s2)^3",
                "(s2 s0)^2",
            ],
        ),
        ("532", &["s0 s1", "s2 s3", "s1^5", "(s2 s0)^2", "s3^3"]),
        (
            "3333",
            &[
                "s0 s1",
                "s2 s5",
                "s3 s4",
                "s1^3",
                "(s2 s0)^3",
                "(s3 s5)^3",
                "s4^3",
            ],
        ),
        (
            "2222",
            &[
                "s0 s1",
                "s2 s5",
                "s3 s4",
                "s1^2",
                "(s2 s0)^2",
                "(s3 s5)^2",
                "s4^2",
            ],
        ),
        ("22", &["s0 s1", "s1^2", "s0^2"]),
    ];
    for (text, relators) in cases {
        let (domain, presentation) = presentation(text);
        assert_eq!(
            presentation.generator_names(),
            side_names(domain.sides().len()),
            "{text}"
        );
        assert_eq!(relator_texts(&presentation), relators, "{text}");
    }
}

#[test]
fn side_presentations_have_one_generator_for_each_side_and_their_relators() {
    let cases = [
        ("*732", 3, 6),
        ("732", 4, 5),
        ("*2222", 4, 8),
        ("*632", 3, 6),
        ("632", 4, 5),
        ("*55", 2, 4),
        ("55", 2, 3),
        ("*3333", 4, 8),
        ("3333", 6, 7),
        ("3222", 6, 7),
        ("22222", 8, 9),
        ("22222222", 14, 15),
    ];
    for (text, generators, relators) in cases {
        let (domain, presentation) = presentation(text);
        assert_eq!(domain.sides().len(), generators, "{text}");
        assert_eq!(
            presentation.generator_names(),
            side_names(generators),
            "{text}"
        );
        assert_eq!(presentation.relators().len(), relators, "{text}");
    }
    let rotations = [
        "732", "542", "444", "433", "(12)32", "632", "442", "333", "2222", "332", "432", "532",
        "222", "722", "22", "55",
    ];
    for rotation in rotations {
        for text in [rotation.to_owned(), starred(rotation)] {
            let domain = built(&text);
            if let Err(error) = domain.side_presentation() {
                panic!("{text} has no side presentation: {error}");
            }
        }
    }
}

fn assert_side_order(text: &str) -> Duration {
    let (_, presentation) = presentation(text);
    let Some(order) = signature(text).group_order() else {
        panic!("{text} has no group order");
    };
    let started = Instant::now();
    let table = match enumerate(&presentation, &[], CosetLimit::DEFAULT) {
        Ok(table) => table,
        Err(error) => panic!("{text}: {error}"),
    };
    let elapsed = started.elapsed();
    assert_eq!(
        u64::try_from(table.index()).ok(),
        Some(order.get()),
        "{text}"
    );
    elapsed
}

#[test]
fn side_presentations_give_the_group_order_on_the_sphere() {
    let mut texts = spherical_signatures();
    texts.extend(limit_signatures());
    assert_eq!(texts.len(), 54);
    let mut slowest = (Duration::ZERO, String::new());
    for text in texts {
        let elapsed = assert_side_order(&text);
        if elapsed > slowest.0 {
            slowest = (elapsed, text);
        }
    }
    println!(
        "slowest side order: {} in {:.3} ms",
        slowest.1,
        slowest.0.as_secs_f64() * 1e3
    );
}

#[test]
fn side_presentation_of_22_gives_its_group_order() {
    assert_side_order("22");
}

#[derive(Clone, Copy)]
enum Kind {
    Square,
    Corner,
    Pair,
    Cycle,
}

impl Kind {
    const fn name(self) -> &'static str {
        match self {
            Self::Square => "square",
            Self::Corner => "corner",
            Self::Pair => "pair",
            Self::Cycle => "cycle",
        }
    }

    const fn tolerance(self) -> f64 {
        match self {
            Self::Square | Self::Pair => PAIRING_TOLERANCE,
            Self::Corner | Self::Cycle => RELATOR_TOLERANCE,
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Square => 0,
            Self::Corner => 1,
            Self::Pair => 2,
            Self::Cycle => 3,
        }
    }
}

const KINDS: [Kind; 4] = [Kind::Square, Kind::Corner, Kind::Pair, Kind::Cycle];

fn sides_of(base: &Word) -> Vec<usize> {
    base.letters()
        .iter()
        .map(|&letter| {
            assert_eq!(letter, Letter::generator(letter.index()), "{base:?}");
            usize::from(letter.index())
        })
        .collect()
}

fn letter_matrix(domain: &FundamentalDomain, side: usize) -> DMat3 {
    let pairing = domain.sides()[side].pairing();
    match domain.geometry() {
        Geometry::Hyperbolic => hyperbolic::inverse(pairing),
        Geometry::Spherical => spherical::inverse(pairing),
        Geometry::Euclidean => pairing.inverse(),
    }
}

fn largest_entry_from_identity(matrix: DMat3) -> f64 {
    (matrix - DMat3::IDENTITY)
        .to_cols_array()
        .into_iter()
        .map(f64::abs)
        .fold(0.0, f64::max)
}

fn exact_frame(vertex: DVec3) -> (Matrix, Matrix) {
    let [x, y, t] = vector(vertex.to_array());
    let norm = sqrt(&sub(&sub(&mul(&t, &t), &mul(&x, &x)), &mul(&y, &y)));
    let [x, y, t] = [x, y, t].map(|coordinate| div(&coordinate, &norm));
    let factor = div(&int(1), &add(&int(1), &t));
    let mixed = mul(&mul(&x, &y), &factor);
    let first = add(&int(1), &mul(&mul(&x, &x), &factor));
    let second = add(&int(1), &mul(&mul(&y, &y), &factor));
    let frame = [
        [first.clone(), mixed.clone(), x.clone()],
        [mixed.clone(), second.clone(), y.clone()],
        [x.clone(), y.clone(), t.clone()],
    ];
    let inverse = [
        [first, mixed.clone(), sub(&int(0), &x)],
        [mixed, second, sub(&int(0), &y)],
        [sub(&int(0), &x), sub(&int(0), &y), t],
    ];
    (frame, inverse)
}

fn exact_framed_residual(
    domain: &FundamentalDomain,
    relator: &Relator,
    vertex: usize,
    reference: &mut Reference,
) -> f64 {
    let product = sides_of(relator.base())
        .into_iter()
        .fold(identity(), |product, side| {
            exact_product(
                &product,
                &matrix(letter_matrix(domain, side).to_cols_array()),
            )
        });
    let (frame, inverse) = exact_frame(domain.vertices()[vertex]);
    let element = exact_product(&exact_product(&inverse, &product), &frame);
    let exponent = relator.exponent();
    let residual = div(&largest_entry(&power(&element, exponent)), &int(exponent));
    reference.approximate(&residual)
}

fn relator_residual(
    domain: &FundamentalDomain,
    relator: &Relator,
    kind: Kind,
    vertex: usize,
    reference: &mut Reference,
) -> f64 {
    let product = sides_of(relator.base())
        .into_iter()
        .fold(DMat3::IDENTITY, |product, side| {
            product * letter_matrix(domain, side)
        });
    let framed = match kind {
        Kind::Corner | Kind::Cycle => domain.geometry() == Geometry::Hyperbolic,
        Kind::Square | Kind::Pair => false,
    };
    if framed && domain.vertices().len() >= ACCURATE_RELATOR_VERTICES {
        return exact_framed_residual(domain, relator, vertex, reference);
    }
    let element = if framed {
        let frame = hyperbolic::translation(domain.vertices()[vertex]);
        hyperbolic::inverse(frame) * product * frame
    } else {
        product
    };
    let exponent = relator.exponent();
    let mut power = element;
    for _ in 1..exponent {
        power *= element;
    }
    let residual = largest_entry_from_identity(power);
    match kind {
        Kind::Square | Kind::Pair => residual,
        Kind::Corner | Kind::Cycle => residual / f64::from(exponent),
    }
}

fn classified(domain: &FundamentalDomain, presentation: &Presentation) -> Vec<(Kind, usize)> {
    let count = domain.sides().len();
    let relators = presentation.relators();
    let all_mirrors = domain
        .sides()
        .iter()
        .all(|side| side.kind() == PairingKind::Mirror);
    if all_mirrors {
        assert_eq!(relators.len(), 2 * count);
        return relators
            .iter()
            .enumerate()
            .map(|(position, relator)| {
                if position < count {
                    (Kind::Square, position)
                } else {
                    let first = sides_of(relator.base())[0];
                    assert_eq!(first, position - count);
                    (Kind::Corner, first)
                }
            })
            .collect();
    }
    let heads: Vec<Kind> = domain
        .sides()
        .iter()
        .enumerate()
        .filter_map(|(index, side)| {
            if side.kind() == PairingKind::Mirror {
                Some(Kind::Square)
            } else {
                (side.partner() > index).then_some(Kind::Pair)
            }
        })
        .collect();
    relators
        .iter()
        .enumerate()
        .map(|(position, relator)| {
            let first = sides_of(relator.base())[0];
            match heads.get(position) {
                Some(&kind) => (kind, first),
                None => (Kind::Cycle, (first + count - 1) % count),
            }
        })
        .collect()
}

struct Largest {
    residual: f64,
    input: String,
    samples: usize,
}

#[test]
fn side_relators_evaluate_to_the_identity() {
    let mut texts: Vec<String> = triangles().map(text).collect();
    texts.extend(triangles().map(cone_text));
    texts.extend(["*632", "*442", "*333", "*2222", "632", "442", "333", "2222"].map(str::to_owned));
    texts.extend(spherical_signatures());
    texts.extend(limit_signatures());
    texts.extend(NAMED_POLYGONS.map(str::to_owned));
    texts.extend(clear_rotation_polygons());
    texts.extend(spherical_kites());
    let mut reference = Reference::new();
    let empty = || {
        KINDS.map(|_| Largest {
            residual: 0.0,
            input: String::new(),
            samples: 0,
        })
    };
    let (mut largest, mut polygons, mut wallpapers) = (empty(), empty(), empty());
    let mut domains: Vec<(String, FundamentalDomain)> = texts
        .iter()
        .map(|text| (text.clone(), built(text)))
        .collect();
    domains.extend(
        named_settings()
            .into_iter()
            .map(|setting| (setting.label(), setting.domain())),
    );
    for (position, (text, domain)) in domains.iter().enumerate() {
        let wallpaper = position >= texts.len();
        let presentation = match domain.side_presentation() {
            Ok(presentation) => presentation,
            Err(error) => panic!("{text} has no side presentation: {error}"),
        };
        for ((kind, vertex), relator) in classified(domain, &presentation)
            .into_iter()
            .zip(presentation.relators())
        {
            let residual = relator_residual(domain, relator, kind, vertex, &mut reference);
            assert!(
                residual <= kind.tolerance(),
                "{text}: {} {residual:e} above {:e}",
                relator_text(&presentation, relator),
                kind.tolerance()
            );
            let reflection_polygon = NAMED_POLYGONS.contains(&text.as_str());
            for (table, kept) in [
                (&mut largest, true),
                (&mut polygons, reflection_polygon),
                (&mut wallpapers, wallpaper),
            ] {
                let entry = &mut table[kind.index()];
                if kept {
                    entry.samples += 1;
                    if residual > entry.residual || entry.input.is_empty() {
                        entry.residual = residual;
                        entry.input.clone_from(text);
                    }
                }
            }
        }
    }
    for (label, table) in [
        ("", &largest),
        ("reflection polygons, ", &polygons),
        ("wallpaper domains, ", &wallpapers),
    ] {
        for kind in KINDS {
            let entry = &table[kind.index()];
            if entry.samples == 0 {
                assert!(!label.is_empty(), "no {} relator", kind.name());
                continue;
            }
            println!(
                "{label}{} relators: largest residual {:e} at {}, tolerance {:e}, margin {:.1}, over {} relators",
                kind.name(),
                entry.residual,
                entry.input,
                kind.tolerance(),
                kind.tolerance() / entry.residual,
                entry.samples
            );
        }
    }
}

fn corner_words(count: usize) -> Vec<Word> {
    (0..count)
        .map(|corner| {
            [corner, (corner + 1) % count]
                .into_iter()
                .map(|side| match u8::try_from(side) {
                    Ok(side) => Letter::generator(side),
                    Err(error) => panic!("side {side} has no letter: {error}"),
                })
                .collect()
        })
        .collect()
}

#[test]
fn corner_words_of_a_reflection_domain_generate_a_subgroup_of_index_two() {
    let mut texts: Vec<String> = EXAMPLE_TRIPLES.into_iter().map(text).collect();
    texts.extend(
        [
            "*632", "*442", "*333", "*2222", "*532", "*432", "*332", "*722", "*55",
        ]
        .map(str::to_owned),
    );
    for text in texts {
        let (domain, presentation) = presentation(&text);
        let subgroup = corner_words(domain.sides().len());
        let table = match enumerate(&presentation, &subgroup, CosetLimit::DEFAULT) {
            Ok(table) => table,
            Err(error) => panic!("{text}: {error}"),
        };
        assert_eq!(table.index(), 2, "{text}");
    }
}

#[test]
fn side_presentations_of_the_wallpaper_domains_have_the_documented_text() {
    for setting in named_settings() {
        let label = setting.label();
        let expected: &[&str] = match setting.text {
            "o" => &["s0 s2", "s1 s3", "s1 s0 s3 s2"],
            "**" => &["s0 s2", "s1^2", "s3^2", "s1 s0 s1 s2", "s3 s2 s3 s0"],
            "xx" => &["s0 s2", "s1 s3", "s1 s0 s1 s2"],
            "*x" => &["s0 s2", "s1^2", "s3^2", "s1 s0 s3 s2", "s2 s3 s0 s1"],
            "22x" => &["s0 s2", "s1 s3", "(s1 s2)^2", "(s2 s3)^2"],
            "22*" => &[
                "s0 s1",
                "s2^2",
                "s3 s4",
                "s5^2",
                "s1^2",
                "s2 s1 s5 s0",
                "s3 s5 s4 s2",
                "s4^2",
            ],
            "2*22" => &[
                "s0 s1",
                "s2^2",
                "s3^2",
                "s4^2",
                "s1^2",
                "s2 s1 s4 s0",
                "(s3 s2)^2",
                "(s4 s3)^2",
            ],
            "2222" => &[
                "s0 s1",
                "s2 s5",
                "s3 s4",
                "s1^2",
                "(s2 s0)^2",
                "(s3 s5)^2",
                "s4^2",
            ],
            "4*2" => &["s0^2", "s1^2", "s2 s3", "(s1 s0)^2", "s2 s0 s3 s1", "s3^4"],
            "3*3" => &["s0^2", "s1^2", "s2 s3", "(s1 s0)^3", "s2 s0 s3 s1", "s3^3"],
            text => panic!("{text} has no documented side presentation"),
        };
        let domain = setting.domain();
        let presentation = match domain.side_presentation() {
            Ok(presentation) => presentation,
            Err(error) => panic!("{label} has no side presentation: {error}"),
        };
        assert_eq!(
            presentation.generator_names(),
            side_names(domain.sides().len()),
            "{label}"
        );
        assert_eq!(relator_texts(&presentation), expected, "{label}");
    }
}

const WALLPAPER_COLOUR_LIMIT: u32 = 16;

#[test]
fn side_and_conway_presentations_of_the_wallpaper_domains_give_equal_colouring_counts() {
    let Ok(limit) = IndexLimit::new(WALLPAPER_COLOUR_LIMIT) else {
        panic!("{WALLPAPER_COLOUR_LIMIT} colours are out of range");
    };
    for (text, first_counts) in [
        ("o", [1, 3, 4, 7, 6]),
        ("**", [1, 7, 4, 19, 6]),
        ("xx", [1, 3, 4, 7, 6]),
        ("*x", [1, 3, 4, 11, 6]),
        ("22x", [1, 3, 6, 11, 10]),
        ("22*", [1, 7, 6, 23, 10]),
        ("2*22", [1, 7, 6, 31, 10]),
        ("2222", [1, 7, 12, 31, 30]),
        ("4*2", [1, 3, 0, 11, 0]),
        ("3*3", [1, 1, 4, 4, 0]),
    ] {
        let side = match built(text).side_presentation() {
            Ok(presentation) => presentation,
            Err(error) => panic!("{text} has no side presentation: {error}"),
        };
        let conway = match conway_presentation(&signature(text)) {
            Ok(presentation) => presentation,
            Err(error) => panic!("{text} has no Conway presentation: {error}"),
        };
        let counts = [("side", &side), ("Conway", &conway)].map(|(name, presentation)| {
            let started = Instant::now();
            let counts = match count_colourings(presentation, limit, SearchBudget::DEFAULT) {
                Ok(counts) => counts,
                Err(error) => panic!("{text}: the search of the {name} presentation fails: {error}"),
            };
            println!(
                "{text}: search of the {name} presentation to {WALLPAPER_COLOUR_LIMIT} colours in {:.3} s",
                started.elapsed().as_secs_f64()
            );
            counts
        });
        let found: Vec<[Option<u64>; 2]> = (1..=WALLPAPER_COLOUR_LIMIT)
            .map(|colours| counts.clone().map(|count| count.colourings(colours)))
            .collect();
        println!(
            "{text}: colourings with 1 to {WALLPAPER_COLOUR_LIMIT} colours, side and Conway: {found:?}"
        );
        for (colours, [by_sides, by_conway]) in (1..=WALLPAPER_COLOUR_LIMIT).zip(&found) {
            assert!(by_sides.is_some(), "{text}: {colours} colours not counted");
            assert_eq!(by_sides, by_conway, "{text}: {colours} colours");
        }
        let first: Vec<Option<u64>> = found
            .iter()
            .take(5)
            .map(|[by_sides, _]| *by_sides)
            .collect();
        assert_eq!(first, first_counts.map(Some), "{text}");
    }
}

#[test]
fn side_presentations_of_the_spherical_kites_have_the_documented_text() {
    let mut cases: Vec<(String, Vec<String>)> = vec![(
        "3*2".to_owned(),
        ["s0^2", "s1^2", "s2 s3", "(s1 s0)^2", "s2 s0 s3 s1", "s3^3"]
            .map(str::to_owned)
            .to_vec(),
    )];
    for order in (2..=SMALL_ORDER).chain([LIMIT_ORDER]) {
        cases.push((
            Kite::ConeMirror(order).text(),
            vec![
                "s0 s1".to_owned(),
                "s2^2".to_owned(),
                "s3^2".to_owned(),
                format!("s1^{order}"),
                "s2 s1 s3 s0".to_owned(),
                "s3 s2".to_owned(),
            ],
        ));
    }
    for order in (2..=SMALL_ORDER).chain([TWO_MIRROR_LIMIT_ORDER]) {
        cases.push((
            Kite::TwoMirror(order).text(),
            vec![
                "s0^2".to_owned(),
                "s1^2".to_owned(),
                "s2 s3".to_owned(),
                format!("(s1 s0)^{order}"),
                "s2 s0 s3 s1".to_owned(),
                "s3^2".to_owned(),
            ],
        ));
    }
    for (text, relators) in cases {
        let (domain, presentation) = presentation(&text);
        assert_eq!(
            presentation.generator_names(),
            side_names(domain.sides().len()),
            "{text}"
        );
        assert_eq!(relator_texts(&presentation), relators, "{text}");
    }
}

fn without_relator(presentation: &Presentation, dropped: usize) -> Presentation {
    let relators: Vec<Relator> = presentation
        .relators()
        .iter()
        .enumerate()
        .filter(|&(position, _)| position != dropped)
        .map(|(_, relator)| relator.clone())
        .collect();
    match Presentation::new(presentation.generator_names().to_vec(), relators) {
        Ok(presentation) => presentation,
        Err(error) => panic!("relator {dropped} cannot be dropped: {error}"),
    }
}

fn cycle_relators(domain: &FundamentalDomain) -> std::ops::Range<usize> {
    let heads = domain
        .sides()
        .iter()
        .enumerate()
        .filter(|&(index, side)| side.kind() == PairingKind::Mirror || side.partner() > index)
        .count();
    heads..heads + 3
}

#[test]
fn side_presentations_of_the_spherical_kites_give_the_group_order() {
    let mut slowest = (Duration::ZERO, String::new());
    let started = Instant::now();
    for text in spherical_kites() {
        let elapsed = assert_side_order(&text);
        if elapsed > slowest.0 {
            slowest = (elapsed, text);
        }
    }
    println!(
        "slowest side order of a spherical kite: {} in {:.3} ms, all in {:.3} s",
        slowest.1,
        slowest.0.as_secs_f64() * 1e3,
        started.elapsed().as_secs_f64()
    );
    for text in CYCLE_RELATOR_KITES {
        let (domain, presentation) = presentation(text);
        let order = signature(text).group_order().map(core::num::NonZero::get);
        for dropped in cycle_relators(&domain) {
            let reduced = without_relator(&presentation, dropped);
            let index = enumerate(&reduced, &[], CosetLimit::DEFAULT)
                .ok()
                .and_then(|table| u64::try_from(table.index()).ok());
            println!(
                "{text} without {}: index {index:?}",
                relator_text(&presentation, &presentation.relators()[dropped])
            );
            assert_ne!(index, order, "{text} without relator {dropped}");
        }
    }
}

const SPHERICAL_COLOUR_LIMIT: u32 = 16;

const DROPPED_RELATOR_COLOUR_LIMIT: u32 = 6;

fn colouring_counts(
    text: &str,
    name: &str,
    presentation: &Presentation,
    largest: u32,
) -> Vec<Option<u64>> {
    let colours = signature(text)
        .group_order()
        .and_then(|order| u32::try_from(order.get()).ok())
        .map_or(largest, |order| order.min(largest));
    let Ok(limit) = IndexLimit::new(colours) else {
        panic!("{colours} colours are out of range");
    };
    let started = Instant::now();
    let counts = match count_colourings(presentation, limit, SearchBudget::DEFAULT) {
        Ok(counts) => counts,
        Err(error) => panic!("{text}: the search of the {name} presentation fails: {error}"),
    };
    let found: Vec<Option<u64>> = (1..=colours)
        .map(|count| counts.colourings(count))
        .collect();
    println!(
        "{text}: {name} presentation, colourings with 1 to {colours} colours {found:?}, in {:.3} s",
        started.elapsed().as_secs_f64()
    );
    found
}

#[test]
fn side_and_conway_presentations_of_the_spherical_groups_give_equal_colouring_counts() {
    for text in small_spherical_kites() {
        let side = presentation(&text).1;
        let conway = match conway_presentation(&signature(&text)) {
            Ok(presentation) => presentation,
            Err(error) => panic!("{text} has no Conway presentation: {error}"),
        };
        let by_sides = colouring_counts(&text, "side", &side, SPHERICAL_COLOUR_LIMIT);
        let by_conway = colouring_counts(&text, "Conway", &conway, SPHERICAL_COLOUR_LIMIT);
        assert!(by_sides.iter().all(Option::is_some), "{text}");
        assert_eq!(by_sides, by_conway, "{text}");
        if CYCLE_RELATOR_KITES.contains(&text.as_str()) {
            let domain = built(&text);
            for dropped in cycle_relators(&domain) {
                let reduced = without_relator(&side, dropped);
                let name = format!(
                    "side without {}",
                    relator_text(&side, &side.relators()[dropped])
                );
                let limit = DROPPED_RELATOR_COLOUR_LIMIT;
                assert_ne!(
                    colouring_counts(&text, &name, &reduced, limit),
                    colouring_counts(&text, "Conway", &conway, limit),
                    "{text}: {name}"
                );
            }
        }
    }
}
