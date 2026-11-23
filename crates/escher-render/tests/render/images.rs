use std::path::PathBuf;

use escher_render::{
    BorderUnit, Colour, CosetFill, ErrorKind, Fill, HyperbolicModel, RenderOptions,
    RenderOptionsBuilder, RenderStats, Renderer, encode_png,
};

use crate::common::{
    CONVERGENCE_SIZE, DEFAULT_CAP, SPHERICAL_GROUP_DOMAINS, decode_png, domain, one_fill, options,
    render_image_in_bands, render_in_bands, spherical_group_bounds,
};
use crate::golden_images::{
    ALLOWED_PIXELS, COSET_GOLDEN_IMAGES, GOLDEN_FILL, GOLDEN_GRID, GOLDEN_IMAGES, GOLDEN_SIZE,
    GoldenImage, MODEL_MAPS, POLYGON_DOMAINS, Shape, TRANSLATION_TOLERANCE, WALLPAPER_DOMAINS,
    colouring, colourings_of, compare, coset_fill, domain_with_aspect, domain_with_settings,
    golden_colouring, linear_table, renderer, shifted_pairs, translation,
};

const UPDATE: &str = "ESCHER_UPDATE_GOLDEN";

const RECIPE: &str = "just update-golden";

fn golden_pixels(image: &GoldenImage) -> Vec<u8> {
    let domain = image.domain();
    renderer(&domain, image.options(&domain)).render().0
}

fn encoded(pixels: &[u8]) -> Vec<u8> {
    match encode_png(GOLDEN_SIZE, GOLDEN_SIZE, pixels) {
        Ok(file) => file,
        Err(error) => panic!("{error}"),
    }
}

fn write(path: &std::path::Path, file: &[u8]) {
    std::fs::write(path, file)
        .unwrap_or_else(|error| panic!("cannot write {}: {error}", path.display()));
}

fn check(
    image: &GoldenImage,
    path: &std::path::Path,
    pixels: &[u8],
    linear: &[f64],
) -> Option<String> {
    let name = format!("{} in {}", image.signature, image.file);
    let file = match std::fs::read(path) {
        Ok(file) => file,
        Err(error) => {
            return Some(format!(
                "{name}: cannot read {}: {error}. Run `{RECIPE}` to write it",
                path.display()
            ));
        }
    };
    let decoded = match decode_png(&file) {
        Ok(decoded) => decoded,
        Err(error) => {
            return Some(format!(
                "{name}: cannot decode {}: {error}. Run `{RECIPE}` to write it again",
                path.display()
            ));
        }
    };
    let output = &decoded.output;
    let expected = (
        GOLDEN_SIZE,
        GOLDEN_SIZE,
        png::ColorType::Rgb,
        png::BitDepth::Eight,
    );
    let found = (
        output.width,
        output.height,
        output.color_type,
        output.bit_depth,
    );
    if found != expected || decoded.pixels.len() != pixels.len() {
        return Some(format!(
            "{name}: {} is a {}x{} image of colour type {:?} and bit depth {:?}, not a {GOLDEN_SIZE}x{GOLDEN_SIZE} RGB image of bit depth 8. Run `{RECIPE}` to write it again",
            path.display(),
            found.0,
            found.1,
            found.2,
            found.3
        ));
    }
    let comparison = compare(&decoded.pixels, pixels, GOLDEN_GRID, linear);
    println!("{name}: {comparison:?}, allowed {ALLOWED_PIXELS} differing pixels");
    if comparison.within_allowance() {
        return None;
    }
    let output = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let actual = output.join(format!("actual-{}", image.file));
    let difference = output.join(format!("difference-{}", image.file));
    let differences: Vec<u8> = decoded
        .pixels
        .iter()
        .zip(pixels)
        .map(|(left, right)| left.abs_diff(*right))
        .collect();
    write(&actual, &encoded(pixels));
    write(&difference, &encoded(&differences));
    Some(format!(
        "{name}: {comparison:?}, allowed {ALLOWED_PIXELS} differing pixels, see {} and {}. If the change to the image is intended, run `{RECIPE}`",
        actual.display(),
        difference.display()
    ))
}

#[test]
fn golden_images_match_the_committed_files() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/render/golden");
    let update = std::env::var_os(UPDATE).is_some_and(|value| value == "1");
    let linear = linear_table();
    let mut failures = Vec::new();
    for image in GOLDEN_IMAGES.iter().chain(&COSET_GOLDEN_IMAGES) {
        let pixels = golden_pixels(image);
        let path = directory.join(image.file);
        if update {
            write(&path, &encoded(&pixels));
        } else if let Some(failure) = check(image, &path, &pixels, &linear) {
            failures.push(failure);
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn pixel_unit_given_explicitly_gives_the_bytes_of_the_default_on_every_golden_configuration() {
    let mut compared = 0;
    for image in GOLDEN_IMAGES
        .iter()
        .chain(&COSET_GOLDEN_IMAGES)
        .filter(|image| image.border_unit == BorderUnit::Pixels)
    {
        let domain = image.domain();
        let render = |builder: RenderOptionsBuilder| {
            render_image_in_bands(image.file, &domain, options(builder)).0
        };
        let default = render(image.builder_without_unit(&domain));
        let explicit = render(
            image
                .builder_without_unit(&domain)
                .border_unit(BorderUnit::Pixels),
        );
        assert!(default == explicit, "{}", image.file);
        compared += 1;
    }
    println!("{compared} golden configurations give the same bytes with the explicit pixel unit");
    assert_eq!(
        compared,
        GOLDEN_IMAGES.len() + COSET_GOLDEN_IMAGES.len() - 3
    );
}

#[test]
fn render_equals_render_rows_over_any_split_into_bands() {
    let [yaw, pitch, roll] = [20.0_f64, 35.0, 0.0].map(f64::to_radians);
    let options = options(
        RenderOptions::builder(Fill::Parity([
            Colour::WHITE,
            Colour::from_srgb8(220, 20, 60),
        ]))
        .size(64, 48)
        .samples(3)
        .rotation(yaw, pitch, roll),
    );
    for domain in [domain([7, 3, 2]), domain_with_aspect("*532", None)] {
        assert_bands_equal_the_whole_image(&renderer(&domain, options.clone()));
    }
}

fn assert_bands_equal_the_whole_image(renderer: &Renderer) {
    let (whole, whole_stats) = renderer.render();
    let row = 3 * 64;
    let splits: [&[u32]; 3] = [&[1; 48], &[7, 1, 23, 2, 15], &[48]];
    for split in splits {
        let mut pixels = Vec::new();
        let mut stats = RenderStats::default();
        let mut first = 0;
        for &rows in split {
            let mut band = vec![0; row * usize::try_from(rows).unwrap_or(usize::MAX)];
            match renderer.render_rows(first, &mut band) {
                Ok(band_stats) => stats = stats.merged(band_stats),
                Err(error) => panic!("{error}"),
            }
            pixels.extend(band);
            first += rows;
        }
        assert_eq!(first, 48);
        assert!(pixels == whole, "{split:?}");
        assert_eq!(stats, whole_stats, "{split:?}");
    }
}

#[test]
fn render_rows_rejects_a_buffer_that_does_not_fit_the_image() {
    let options = options(RenderOptions::builder(one_fill()).size(512, 512).samples(1));
    let renderer = renderer(&domain([7, 3, 2]), options);
    let row = 3 * 512;
    let cases: [(u32, usize, &str); 4] = [
        (
            0,
            row - 1,
            "cannot render rows: the buffer of 1535 bytes does not hold whole rows of 1536 bytes",
        ),
        (
            0,
            0,
            "cannot render rows: the buffer of 0 bytes does not hold whole rows of 1536 bytes",
        ),
        (
            510,
            4 * row,
            "cannot render rows: the rows 510 to 513 are outside the image of 512 rows",
        ),
        (
            u32::MAX,
            row,
            "cannot render rows: the rows 4294967295 to 4294967295 are outside the image of 512 rows",
        ),
    ];
    for (first, length, text) in cases {
        let mut buffer = vec![7; length];
        match renderer.render_rows(first, &mut buffer) {
            Ok(stats) => panic!("{length} bytes from row {first} are accepted: {stats:?}"),
            Err(error) => {
                assert_eq!(error.kind(), ErrorKind::InvalidBuffer);
                assert_eq!(error.to_string(), text);
                assert!(buffer.iter().all(|&byte| byte == 7));
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Axis {
    First,
    Second,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Expected {
    Equal,
    WithinAllowance,
}

struct Period {
    signature: &'static str,
    aspect: Option<f64>,
    word: &'static [usize],
    axis: Axis,
    shift: i64,
    expected: Expected,
}

const PERIODS: [Period; 8] = [
    Period {
        signature: "*2222",
        aspect: Some(1.0),
        word: &[1, 3],
        axis: Axis::First,
        shift: 128,
        expected: Expected::Equal,
    },
    Period {
        signature: "*2222",
        aspect: Some(1.0),
        word: &[0, 2],
        axis: Axis::Second,
        shift: 128,
        expected: Expected::Equal,
    },
    Period {
        signature: "*2222",
        aspect: Some(4.0),
        word: &[1, 3],
        axis: Axis::First,
        shift: 256,
        expected: Expected::Equal,
    },
    Period {
        signature: "*2222",
        aspect: Some(4.0),
        word: &[0, 2],
        axis: Axis::Second,
        shift: 64,
        expected: Expected::Equal,
    },
    Period {
        signature: "*442",
        aspect: None,
        word: &[2, 1, 0, 1],
        axis: Axis::First,
        shift: 128,
        expected: Expected::WithinAllowance,
    },
    Period {
        signature: "*442",
        aspect: None,
        word: &[0, 1, 2, 1],
        axis: Axis::Second,
        shift: 128,
        expected: Expected::WithinAllowance,
    },
    Period {
        signature: "*632",
        aspect: None,
        word: &[2, 1, 0, 1, 0, 1],
        axis: Axis::First,
        shift: 128,
        expected: Expected::WithinAllowance,
    },
    Period {
        signature: "*333",
        aspect: None,
        word: &[2, 0, 1, 2, 0, 1],
        axis: Axis::First,
        shift: 192,
        expected: Expected::WithinAllowance,
    },
];

const PERIOD_SIZE: u32 = 512;

#[test]
fn euclidean_images_equal_themselves_shifted_by_a_translation_of_the_group() {
    let [even, odd] = GOLDEN_FILL.map(|[red, green, blue]| Colour::from_srgb8(red, green, blue));
    let options = options(
        RenderOptions::builder(Fill::Parity([even, odd]))
            .size(PERIOD_SIZE, PERIOD_SIZE)
            .samples(GOLDEN_GRID),
    );
    let pixel = 8.0 / f64::from(PERIOD_SIZE);
    let linear = linear_table();
    let mut rendered: Vec<(&str, Option<u64>, Vec<u8>)> = Vec::new();
    for period in &PERIODS {
        let name = format!(
            "{} aspect {:?}, word {:?}",
            period.signature, period.aspect, period.word
        );
        let domain = domain_with_aspect(period.signature, period.aspect);
        let found = translation(&domain, period.word);
        let (along, across) = match period.axis {
            Axis::First => (found.offset.x, found.offset.y),
            Axis::Second => (found.offset.y, found.offset.x),
        };
        println!(
            "{name}: offset {:?}, linear defect {:e}, last row defect {:e}, off-axis component {:e}",
            found.offset, found.linear_defect, found.last_row_defect, across
        );
        assert!(found.linear_defect <= TRANSLATION_TOLERANCE, "{name}");
        assert!(found.last_row_defect <= TRANSLATION_TOLERANCE, "{name}");
        assert!(across.abs() <= TRANSLATION_TOLERANCE, "{name}");
        #[expect(
            clippy::cast_possible_truncation,
            reason = "the shift is a whole number of pixels within the image"
        )]
        let shift = (along / pixel).round() as i64;
        assert_eq!(shift.abs(), period.shift, "{name}: {along}");
        let (columns, rows) = match period.axis {
            Axis::First => (shift, 0),
            Axis::Second => (0, -shift),
        };
        let key = (period.signature, period.aspect.map(f64::to_bits));
        let found = rendered
            .iter()
            .position(|(signature, aspect, _)| (*signature, *aspect) == key);
        let index = found.unwrap_or_else(|| {
            let (pixels, _) = renderer(&domain, options.clone()).render();
            rendered.push((key.0, key.1, pixels));
            rendered.len() - 1
        });
        let pixels = &rendered[index].2;
        let (here, there) = shifted_pairs(pixels, PERIOD_SIZE, PERIOD_SIZE, columns, rows);
        let comparison = compare(&here, &there, GOLDEN_GRID, &linear);
        println!(
            "{name}: shifted by {columns} columns and {rows} rows, {comparison:?}, expected {:?}",
            period.expected
        );
        match period.expected {
            Expected::Equal => assert!(here == there, "{name}: {comparison:?}"),
            Expected::WithinAllowance => {
                assert!(comparison.within_allowance(), "{name}: {comparison:?}");
            }
        }
    }
}

#[test]
fn square_tiling_at_one_sample_per_pixel_has_no_border_pixel_and_at_two_by_two_has_some() {
    let fill = Colour::from_srgb8(70, 130, 180);
    let domain = domain_with_aspect("*2222", Some(1.0));
    let mut counts = [0; 2];
    for (count, grid) in counts.iter_mut().zip([1, 2]) {
        let options = options(
            RenderOptions::builder(Fill::One(fill))
                .size(512, 512)
                .samples(grid)
                .border_width(1.0),
        );
        let (pixels, _) = renderer(&domain, options).render();
        *count = pixels
            .as_chunks::<3>()
            .0
            .iter()
            .filter(|pixel| **pixel != fill.to_srgb8())
            .count();
    }
    println!(
        "*2222 aspect 1 at 512 by 512 with border width 1: {} pixels other than the fill with one sample, {} with 2 by 2 samples",
        counts[0], counts[1]
    );
    assert_eq!(counts[0], 0);
    assert!(counts[1] > 0);
}

fn assert_folds_within_the_group_order(text: &str) {
    let signature: escher_signature::Signature = text
        .parse()
        .unwrap_or_else(|error| panic!("{text:?} is rejected: {error}"));
    let Some(order) = signature.group_order() else {
        panic!("{text} has no group order");
    };
    let options =
        options(RenderOptions::builder(one_fill()).size(CONVERGENCE_SIZE, CONVERGENCE_SIZE));
    let (_, stats) = renderer(&domain_with_aspect(text, None), options).render();
    let inside = stats.samples() - stats.outside();
    #[expect(
        clippy::cast_precision_loss,
        reason = "the counts are far below 2 to the power 52"
    )]
    let mean = stats.folds() as f64 / inside as f64;
    println!(
        "{text} at {CONVERGENCE_SIZE} by {CONVERGENCE_SIZE} with 4 by 4 samples: {inside} samples inside the ball, mean steps {mean:.2}, largest {}, group order {order}, {} unconverged",
        stats.largest_fold(),
        stats.unconverged()
    );
    assert!(u64::from(stats.largest_fold()) < order.get(), "{text}");
    assert_eq!(stats.unconverged(), 0, "{text}");
}

#[test]
fn spherical_golden_domains_fold_every_sample_within_the_group_order() {
    let mut texts: Vec<&str> = Vec::new();
    for image in &GOLDEN_IMAGES {
        let domain = image.domain();
        let spherical = domain.geometry() == escher_signature::Geometry::Spherical;
        let rotation = !domain.reverses_orientation();
        if spherical && !rotation && !texts.contains(&image.signature) {
            texts.push(image.signature);
        }
    }
    for text in texts {
        assert_folds_within_the_group_order(text);
    }
    let small = options(RenderOptions::builder(one_fill()).size(4, 4).samples(1));
    let (_, stats) = renderer(&domain_with_aspect("*532", None), small).render();
    assert_eq!(stats.samples(), 16);
    assert_eq!(stats.outside(), 4);
}

#[test]
fn triangle_of_order_100_folds_every_sample_within_the_group_order() {
    assert_folds_within_the_group_order("*(100)22");
}

#[test]
fn lune_of_order_100_folds_every_sample_within_the_group_order() {
    assert_folds_within_the_group_order("*(100)(100)");
}

#[test]
fn parity_fill_of_a_rotation_domain_equals_its_first_colour() {
    let first = Colour::from_srgb8(245, 222, 179);
    let second = Colour::from_srgb8(70, 130, 180);
    for text in ["732", "55"] {
        let domain = domain_with_aspect(text, None);
        let render = |fill| {
            let options = options(RenderOptions::builder(fill).size(64, 64).samples(2));
            renderer(&domain, options).render().0
        };
        assert!(
            render(Fill::Parity([first, second])) == render(Fill::One(first)),
            "{text}"
        );
    }
}

fn parity_colouring(domain: &escher_domain::FundamentalDomain) -> escher_group::Colouring {
    let found = colourings_of(domain, 2);
    let moves_coset_zero = |colouring: &escher_group::Colouring| {
        domain.sides().iter().enumerate().all(|(index, side)| {
            let reverses = side.orientation() == escher_domain::Orientation::Reversing;
            let index = u8::try_from(index).unwrap_or(u8::MAX);
            let image = colouring
                .table()
                .image(0, escher_group::Letter::generator(index));
            image == Some(u32::from(reverses))
        })
    };
    let mut parity = found.iter().filter(|colouring| moves_coset_zero(colouring));
    let colouring = parity
        .next()
        .unwrap_or_else(|| panic!("the domain has no parity colouring"));
    assert!(
        parity.next().is_none(),
        "the domain has two parity colourings"
    );
    colouring.clone()
}

#[test]
fn coset_fills_of_the_trivial_and_the_parity_colouring_equal_the_one_and_the_parity_fill() {
    let first = Colour::from_srgb8(245, 222, 179);
    let second = Colour::from_srgb8(70, 130, 180);
    let render = |domain: &escher_domain::FundamentalDomain, fill| {
        let options = options(RenderOptions::builder(fill).size(64, 64).samples(2));
        renderer(domain, options).render()
    };
    for text in ["*732", "*642", "*2222", "*532", "732"] {
        let domain = domain_with_aspect(text, None);
        let trivial = colouring(&domain, "1a");
        let cosets = match CosetFill::new(&domain, trivial, vec![first]) {
            Ok(fill) => fill,
            Err(error) => panic!("{text}: {error}"),
        };
        assert!(
            render(&domain, Fill::Cosets(cosets)) == render(&domain, Fill::One(first)),
            "{text} with 1a"
        );
        if !text.starts_with('*') {
            continue;
        }
        let parity = parity_colouring(&domain);
        let id = parity.id();
        let cosets = match CosetFill::new(&domain, parity, vec![first, second]) {
            Ok(fill) => fill,
            Err(error) => panic!("{text}: {error}"),
        };
        assert!(
            render(&domain, Fill::Cosets(cosets)) == render(&domain, Fill::Parity([first, second])),
            "{text} with {id}"
        );
    }
}

#[test]
fn coset_fill_render_equals_render_rows_over_any_split_into_bands() {
    for (text, id) in [("*642", "3a"), ("532", "5a")] {
        let domain = domain_with_aspect(text, None);
        let fill = coset_fill(&domain, colouring(&domain, id));
        let options = options(
            RenderOptions::builder(Fill::Cosets(fill))
                .size(64, 48)
                .samples(3),
        );
        assert_bands_equal_the_whole_image(&renderer(&domain, options));
    }
}

#[test]
fn polygon_images_fold_every_sample_within_the_cap() {
    for shape in &POLYGON_DOMAINS {
        let name = shape.name();
        let options =
            options(RenderOptions::builder(one_fill()).size(CONVERGENCE_SIZE, CONVERGENCE_SIZE));
        let stats = render_in_bands(&name, &shape.domain(), options);
        let inside = stats.samples() - stats.outside();
        #[expect(
            clippy::cast_precision_loss,
            reason = "the counts are far below 2 to the power 52"
        )]
        let mean = stats.folds() as f64 / inside as f64;
        let allowed = crate::reference::count_bound(0);
        println!(
            "{name} at {CONVERGENCE_SIZE} by {CONVERGENCE_SIZE} with 4 by 4 samples: {inside} samples inside the disk, mean steps {mean:.2}, largest {}, {} unconverged, bound {allowed}",
            stats.largest_fold(),
            stats.unconverged()
        );
        assert!(
            stats.unconverged() <= u64::try_from(allowed).unwrap_or(u64::MAX),
            "{name}: {} unconverged samples",
            stats.unconverged()
        );
    }
}

#[test]
fn coset_golden_colouring_of_the_pentagon_fixes_the_numbering_of_its_sides() {
    let domain = domain_with_aspect("*22222", None);
    let colouring = golden_colouring(&domain, "8ekf");
    let sides = domain.sides();
    let colours: Vec<u32> = sides
        .iter()
        .enumerate()
        .map(|(index, side)| {
            let across = sides[side.partner()].pairing() * domain.centre();
            let mut visited = Vec::new();
            let result = escher_render::fold(&domain, across, DEFAULT_CAP, |side| {
                visited.push(u8::try_from(side).unwrap_or(u8::MAX));
            });
            assert!(result.converged(), "the tile across side {index}");
            assert_eq!(
                visited,
                [u8::try_from(index).unwrap_or(u8::MAX)],
                "the tile across side {index}"
            );
            colouring
                .colour_of(&visited)
                .unwrap_or_else(|| panic!("side {index} has no generator"))
        })
        .collect();
    assert_eq!(colours, [1, 2, 3, 4, 5]);
}

fn assert_wallpaper_images_converge(domains: core::ops::Range<usize>) {
    for shape in &WALLPAPER_DOMAINS[domains] {
        let name = shape.name();
        let options =
            options(RenderOptions::builder(one_fill()).size(CONVERGENCE_SIZE, CONVERGENCE_SIZE));
        let stats = render_in_bands(&name, &shape.domain(), options);
        #[expect(
            clippy::cast_precision_loss,
            reason = "the counts are far below 2 to the power 52"
        )]
        let mean = stats.folds() as f64 / stats.samples() as f64;
        println!(
            "{name} at {CONVERGENCE_SIZE} by {CONVERGENCE_SIZE} with 4 by 4 samples: mean steps {mean:.2}, largest {}, {} unconverged, {} outside",
            stats.largest_fold(),
            stats.unconverged(),
            stats.outside()
        );
        assert_eq!(stats.unconverged(), 0, "{name}");
        assert_eq!(stats.outside(), 0, "{name}");
    }
}

#[test]
fn wallpaper_images_of_the_torus_and_the_first_rectangular_groups_converge_at_every_sample() {
    assert_wallpaper_images_converge(0..13);
}

#[test]
fn wallpaper_images_of_the_other_groups_converge_at_every_sample() {
    assert_wallpaper_images_converge(13..WALLPAPER_DOMAINS.len());
}

struct WallpaperPeriod {
    signature: &'static str,
    settings: &'static [(&'static str, f64)],
    words: &'static [(&'static [usize], [i64; 2])],
    expected: Expected,
}

const fn period(
    signature: &'static str,
    settings: &'static [(&'static str, f64)],
    words: &'static [(&'static [usize], [i64; 2])],
    expected: Expected,
) -> WallpaperPeriod {
    WallpaperPeriod {
        signature,
        settings,
        words,
        expected,
    }
}

const EXACT_WALLPAPER_PERIODS: [WallpaperPeriod; 12] = [
    period(
        "**",
        &[("aspect", 1.0)],
        &[(&[1, 3], [128, 0]), (&[0], [0, 64])],
        Expected::Equal,
    ),
    period(
        "**",
        &[("aspect", 4.0)],
        &[(&[1, 3], [256, 0]), (&[0], [0, 32])],
        Expected::Equal,
    ),
    period(
        "xx",
        &[("aspect", 1.0)],
        &[(&[3], [64, 0]), (&[0, 0], [0, 128])],
        Expected::Equal,
    ),
    period(
        "xx",
        &[("aspect", 4.0)],
        &[(&[3], [128, 0]), (&[0, 0], [0, 64])],
        Expected::Equal,
    ),
    period(
        "*x",
        &[("aspect", 1.0)],
        &[(&[1, 3], [128, 0]), (&[0, 3], [64, 64])],
        Expected::Equal,
    ),
    period(
        "*x",
        &[("aspect", 4.0)],
        &[(&[1, 3], [256, 0]), (&[0, 3], [128, 32])],
        Expected::Equal,
    ),
    period(
        "22x",
        &[("aspect", 1.0)],
        &[(&[3, 3], [128, 0]), (&[0, 0], [0, 128])],
        Expected::Equal,
    ),
    period(
        "22x",
        &[("aspect", 4.0)],
        &[(&[3, 3], [256, 0]), (&[0, 0], [0, 64])],
        Expected::Equal,
    ),
    period(
        "22*",
        &[("aspect", 1.0)],
        &[(&[2, 5], [128, 0]), (&[3, 0], [0, 128])],
        Expected::Equal,
    ),
    period(
        "22*",
        &[("aspect", 4.0)],
        &[(&[2, 5], [256, 0]), (&[3, 0], [0, 64])],
        Expected::Equal,
    ),
    period(
        "2*22",
        &[("aspect", 1.0)],
        &[(&[2, 4], [128, 0]), (&[2, 3, 0], [64, 128])],
        Expected::Equal,
    ),
    period(
        "2*22",
        &[("aspect", 4.0)],
        &[(&[2, 4], [256, 0]), (&[2, 3, 0], [128, 64])],
        Expected::Equal,
    ),
];

const ROUNDED_WALLPAPER_PERIODS: [WallpaperPeriod; 7] = [
    period(
        "o",
        &[("aspect", 1.0), ("skew", 0.25)],
        &[(&[3], [64, 0]), (&[0], [16, 64])],
        Expected::WithinAllowance,
    ),
    period(
        "o",
        &[("aspect", 4.0), ("skew", 0.25)],
        &[(&[3], [128, 0]), (&[0], [32, 32])],
        Expected::WithinAllowance,
    ),
    period(
        "2222",
        &[("aspect", 1.0), ("skew", 0.25)],
        &[(&[5], [128, 0]), (&[3, 0], [32, 128])],
        Expected::WithinAllowance,
    ),
    period(
        "2222",
        &[("aspect", 4.0), ("skew", 0.25)],
        &[(&[5], [256, 0]), (&[3, 0], [64, 64])],
        Expected::WithinAllowance,
    ),
    period(
        "2222",
        &[("aspect", 4.0), ("skew", -0.5)],
        &[(&[5], [256, 0]), (&[3, 0], [-128, 64])],
        Expected::WithinAllowance,
    ),
    period(
        "4*2",
        &[],
        &[(&[1, 3, 1, 3], [128, 128]), (&[0, 3, 0, 3], [128, -128])],
        Expected::WithinAllowance,
    ),
    period(
        "3*3",
        &[],
        &[(&[1, 0, 3], [128, 0])],
        Expected::WithinAllowance,
    ),
];

const GLIDE_GROUPS: [&str; 3] = ["xx", "*x", "22x"];

fn assert_glide_symmetry(name: &str, domain: &escher_domain::FundamentalDomain, first: &[u8]) {
    let [even, odd] = GOLDEN_FILL.map(|[red, green, blue]| Colour::from_srgb8(red, green, blue));
    let size = i64::from(PERIOD_SIZE);
    let pixel = 8.0 / f64::from(PERIOD_SIZE);
    let options = options(
        RenderOptions::builder(Fill::Parity([odd, even]))
            .size(PERIOD_SIZE, PERIOD_SIZE)
            .samples(GOLDEN_GRID),
    );
    let (second, _) = render_image_in_bands(name, domain, options);
    let height = domain.side_lengths()[1];
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the height of the rectangle is a small whole number of pixels"
    )]
    let shift = (height / pixel).round() as i64;
    let index =
        |column: i64, row: i64| usize::try_from(3 * (row * size + column)).unwrap_or(usize::MAX);
    let mut compared = 0;
    let mut differing = 0;
    for row in shift..size {
        for column in 0..size {
            let here = index(column, row);
            let there = index(size - 1 - column, row - shift);
            compared += 1;
            if first[here..here + 3] != second[there..there + 3] {
                differing += 1;
            }
        }
    }
    println!(
        "{name}: mirrored and shifted by {shift} rows, {differing} of {compared} pixels differ from the render with the colours exchanged"
    );
    assert_eq!(
        differing, 0,
        "{name}: the image mirrored by the glide reflection differs from the render with the parity colours exchanged"
    );
}

fn assert_wallpaper_periods(periods: &[WallpaperPeriod]) {
    let [even, odd] = GOLDEN_FILL.map(|[red, green, blue]| Colour::from_srgb8(red, green, blue));
    let pixel = 8.0 / f64::from(PERIOD_SIZE);
    let linear = linear_table();
    for period in periods {
        let name = format!("{} with {:?}", period.signature, period.settings);
        let domain = domain_with_settings(period.signature, period.settings);
        let options = options(
            RenderOptions::builder(Fill::Parity([even, odd]))
                .size(PERIOD_SIZE, PERIOD_SIZE)
                .samples(GOLDEN_GRID),
        );
        let (pixels, _) = render_image_in_bands(&name, &domain, options);
        for &(word, shift) in period.words {
            let found = translation(&domain, word);
            #[expect(
                clippy::cast_precision_loss,
                reason = "the shift is a small whole number of pixels"
            )]
            let expected = shift.map(|pixels| pixels as f64 * pixel);
            let offset = [found.offset.x - expected[0], found.offset.y - expected[1]];
            println!(
                "{name}, word {word:?}: offset {:?}, difference from the shift {shift:?} {offset:?}, linear defect {:e}, last row defect {:e}",
                found.offset, found.linear_defect, found.last_row_defect
            );
            assert!(
                found.linear_defect <= TRANSLATION_TOLERANCE,
                "{name}, word {word:?}"
            );
            assert!(
                found.last_row_defect <= TRANSLATION_TOLERANCE,
                "{name}, word {word:?}"
            );
            assert!(
                offset
                    .iter()
                    .all(|difference| difference.abs() <= TRANSLATION_TOLERANCE),
                "{name}, word {word:?}: {offset:?}"
            );
            let (columns, rows) = (shift[0], -shift[1]);
            let (here, there) = shifted_pairs(&pixels, PERIOD_SIZE, PERIOD_SIZE, columns, rows);
            let comparison = compare(&here, &there, GOLDEN_GRID, &linear);
            println!(
                "{name}, word {word:?}: shifted by {columns} columns and {rows} rows, {comparison:?}, expected {:?}",
                period.expected
            );
            match period.expected {
                Expected::Equal => assert!(here == there, "{name}, word {word:?}: {comparison:?}"),
                Expected::WithinAllowance => {
                    assert!(
                        comparison.within_allowance(),
                        "{name}, word {word:?}: {comparison:?}"
                    );
                }
            }
        }
        if GLIDE_GROUPS.contains(&period.signature) {
            assert_glide_symmetry(&name, &domain, &pixels);
        }
    }
}

#[test]
fn wallpaper_images_of_dyadic_rectangles_equal_themselves_shifted_by_a_translation_and_mirrored_by_a_glide_reflection()
 {
    assert_wallpaper_periods(&EXACT_WALLPAPER_PERIODS);
}

#[test]
fn wallpaper_images_with_rounded_walls_equal_themselves_shifted_by_a_translation_within_the_golden_allowance()
 {
    assert_wallpaper_periods(&ROUNDED_WALLPAPER_PERIODS);
}

#[test]
fn wallpaper_coset_fills_of_the_trivial_and_the_parity_colouring_equal_the_one_and_the_parity_fill()
{
    let first = Colour::from_srgb8(245, 222, 179);
    let second = Colour::from_srgb8(70, 130, 180);
    let render = |domain: &escher_domain::FundamentalDomain, fill| {
        let options = options(RenderOptions::builder(fill).size(64, 64).samples(2));
        renderer(domain, options).render()
    };
    let cases = WALLPAPER_DOMAINS.iter().filter(|shape| match shape.shape {
        Shape::Named(settings) => settings.contains(&("aspect", 1.0)),
        Shape::Default | Shape::Every(_) => true,
    });
    for shape in cases {
        let name = shape.name();
        let domain = shape.domain();
        let trivial = colouring(&domain, "1a");
        let cosets = match CosetFill::new(&domain, trivial, vec![first]) {
            Ok(fill) => fill,
            Err(error) => panic!("{name}: {error}"),
        };
        assert!(
            render(&domain, Fill::Cosets(cosets)) == render(&domain, Fill::One(first)),
            "{name} with 1a"
        );
        if !domain.reverses_orientation() {
            println!("{name}: 1a equals the one fill, and no side reverses orientation");
            continue;
        }
        let parity = parity_colouring(&domain);
        let id = parity.id();
        let cosets = match CosetFill::new(&domain, parity, vec![first, second]) {
            Ok(fill) => fill,
            Err(error) => panic!("{name}: {error}"),
        };
        assert!(
            render(&domain, Fill::Cosets(cosets)) == render(&domain, Fill::Parity([first, second])),
            "{name} with {id}"
        );
        println!("{name}: 1a equals the one fill, and the parity colouring {id} the parity fill");
    }
}

#[test]
fn coset_golden_colourings_of_the_nine_wallpaper_groups_give_pairwise_distinct_images() {
    let images: Vec<(&str, Vec<u8>)> = COSET_GOLDEN_IMAGES
        .iter()
        .filter(|image| image.file.starts_with("cos-wp-"))
        .map(|image| {
            let domain = image.domain();
            let id = image
                .colouring
                .unwrap_or_else(|| panic!("{} has no colouring", image.file));
            let fill = coset_fill(&domain, golden_colouring(&domain, id));
            let options = options(
                RenderOptions::builder(Fill::Cosets(fill))
                    .size(64, 64)
                    .samples(2)
                    .border_width(1.0),
            );
            (image.file, renderer(&domain, options).render().0)
        })
        .collect();
    assert_eq!(images.len(), 9);
    for (index, (file, pixels)) in images.iter().enumerate() {
        for (other, other_pixels) in &images[..index] {
            let differing = pixels
                .as_chunks::<3>()
                .0
                .iter()
                .zip(other_pixels.as_chunks::<3>().0)
                .filter(|(left, right)| left != right)
                .count();
            println!("{file} and {other}: {differing} of 4096 pixels differ");
            assert!(pixels != other_pixels, "{file} and {other}");
        }
    }
}

const TURNED: [f64; 3] = [30.0, 50.0, 10.0];

const GROUP_IMAGE_SIZE: u32 = 256;

const DEGENERATE: [&str; 3] = ["*", "x", "1"];

fn turned_builder(fill: Fill, degrees: [f64; 3]) -> RenderOptionsBuilder {
    let [yaw, pitch, roll] = degrees.map(f64::to_radians);
    RenderOptions::builder(fill).rotation(yaw, pitch, roll)
}

fn group_order(text: &str) -> u64 {
    let signature: escher_signature::Signature = text
        .parse()
        .unwrap_or_else(|error| panic!("{text:?} is rejected: {error}"));
    signature
        .group_order()
        .unwrap_or_else(|| panic!("{text} has no group order"))
        .get()
}

#[test]
fn spherical_group_images_fold_every_sample_within_the_step_bound_and_the_group_order() {
    let cases = SPHERICAL_GROUP_DOMAINS
        .into_iter()
        .map(|text| (text, [0.0; 3]))
        .chain(DEGENERATE.map(|text| (text, TURNED)));
    for (text, degrees) in cases {
        let name = format!("{text} with rotation {degrees:?}");
        let order = group_order(text);
        let steps = spherical_group_bounds(text).steps;
        let options = options(
            turned_builder(one_fill(), degrees)
                .size(GROUP_IMAGE_SIZE, GROUP_IMAGE_SIZE)
                .samples(4),
        );
        let stats = render_in_bands(&name, &domain_with_settings(text, &[]), options);
        let inside = stats.samples() - stats.outside();
        #[expect(
            clippy::cast_precision_loss,
            reason = "the counts are far below 2 to the power 52"
        )]
        let mean = stats.folds() as f64 / inside as f64;
        println!(
            "{name} at {GROUP_IMAGE_SIZE} by {GROUP_IMAGE_SIZE} with 4 by 4 samples: {inside} samples inside the ball, mean steps {mean:.2}, largest {}, step bound {steps}, group order {order}, {} unconverged",
            stats.largest_fold(),
            stats.unconverged()
        );
        assert_eq!(stats.unconverged(), 0, "{name}");
        assert!(stats.largest_fold() <= steps, "{name}");
        assert!(u64::from(stats.largest_fold()) < order, "{name}");
    }
}

#[test]
fn whole_sphere_of_the_trivial_group_is_one_tile_without_a_border_in_every_view() {
    let domain = domain_with_settings("1", &[]);
    let [first, second] =
        GOLDEN_FILL.map(|[red, green, blue]| Colour::from_srgb8(red, green, blue));
    for degrees in [[0.0; 3], TURNED] {
        let render = |fill: Fill| {
            let options = options(
                turned_builder(fill, degrees)
                    .size(64, 64)
                    .samples(2)
                    .border_width(1.0)
                    .shading(escher_render::Shading::Flat),
            );
            renderer(&domain, options).render()
        };
        let (pixels, stats) = render(Fill::One(first));
        assert_eq!(stats.largest_fold(), 0, "{degrees:?}");
        assert_eq!(stats.unconverged(), 0, "{degrees:?}");
        let pixel = 2.0 / (15.0 / 16.0 * 64.0);
        let mut inside = 0;
        for (index, bytes) in pixels.as_chunks::<3>().0.iter().enumerate() {
            let index = u32::try_from(index).unwrap_or(u32::MAX);
            let (column, row) = (index % 64, index / 64);
            let corners = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)].map(|(across, down)| {
                let x = (f64::from(column) + across - 32.0) * pixel;
                let y = (32.0 - f64::from(row) - down) * pixel;
                x * x + y * y
            });
            if corners.iter().all(|&squared| squared < 1.0) {
                inside += 1;
                assert_eq!(*bytes, first.to_srgb8(), "{degrees:?} at ({column}, {row})");
            }
        }
        assert!(inside > 0, "{degrees:?}");
        println!(
            "1 with rotation {degrees:?}: {inside} pixels wholly inside the ball have the fill colour"
        );
        assert!(
            render(Fill::Parity([first, second])) == render(Fill::One(first)),
            "{degrees:?}"
        );
    }
}

const SHARED_VIEWS: [[f64; 3]; 3] = [[0.0; 3], [20.0, 35.0, 0.0], TURNED];

fn shared_render(text: &str, fill: Fill, degrees: [f64; 3]) -> Vec<u8> {
    let options = options(turned_builder(fill, degrees).size(64, 64).samples(2));
    renderer(&domain_with_settings(text, &[]), options)
        .render()
        .0
}

#[test]
fn spherical_groups_share_the_pictures_of_the_groups_whose_tiles_and_borders_they_share() {
    let [first, second] =
        GOLDEN_FILL.map(|[red, green, blue]| Colour::from_srgb8(red, green, blue));
    let parity = || Fill::Parity([first, second]);
    let one = || Fill::One(first);
    let order = |n: u32| {
        if n < 10 {
            n.to_string()
        } else {
            format!("({n})")
        }
    };
    let mut parity_pairs: Vec<(String, String)> = [2, 3, 5, 12, 100]
        .map(|n| (format!("{}x", order(n)), format!("*{0}{0}", order(n))))
        .to_vec();
    parity_pairs.push(("x".to_owned(), "*".to_owned()));
    let mut one_pairs: Vec<(String, String)> = vec![("x".to_owned(), "*".to_owned())];
    one_pairs
        .extend([2, 3, 5, 12, 100].map(|n| (format!("{}*", order(n)), format!("{}22", order(n)))));
    one_pairs.extend(
        [2, 3, 5, 12, 50].map(|n| (format!("2*{}", order(n)), format!("{}22", order(2 * n)))),
    );
    one_pairs.push(("3*2".to_owned(), "432".to_owned()));
    for degrees in SHARED_VIEWS {
        for (pairs, label) in [(&parity_pairs, "Fill::Parity"), (&one_pairs, "Fill::One")] {
            for (group, related) in pairs {
                let fill = || {
                    if label == "Fill::One" {
                        one()
                    } else {
                        parity()
                    }
                };
                assert!(
                    shared_render(group, fill(), degrees)
                        == shared_render(related, fill(), degrees),
                    "{group} and {related} with {label} and rotation {degrees:?}"
                );
            }
            println!(
                "rotation {degrees:?}: {} pairs equal byte for byte with {label}",
                pairs.len()
            );
        }
        let kite = shared_render("3*2", parity(), degrees);
        let rotation = shared_render("432", one(), degrees);
        assert!(kite != rotation, "3*2 with parity against 432, {degrees:?}");
    }
    let whole = shared_render("1", one(), [0.0; 3]);
    let cross_cap = shared_render("x", one(), [0.0; 3]);
    let differing = whole
        .as_chunks::<3>()
        .0
        .iter()
        .zip(cross_cap.as_chunks::<3>().0)
        .filter(|(left, right)| left != right)
        .count();
    println!("1 and x with Fill::One in the canonical view: {differing} of 4096 pixels differ");
    assert!(differing > 0);
}

#[test]
fn spherical_group_coset_fills_of_the_trivial_and_the_parity_colouring_equal_the_one_and_the_parity_fill()
 {
    let [first, second] =
        GOLDEN_FILL.map(|[red, green, blue]| Colour::from_srgb8(red, green, blue));
    let mut reversing = 0;
    let cases = SPHERICAL_GROUP_DOMAINS
        .into_iter()
        .map(|text| (text, [0.0; 3]))
        .chain(DEGENERATE.map(|text| (text, TURNED)));
    for (text, degrees) in cases {
        let domain = domain_with_settings(text, &[]);
        let render = |fill| {
            let options = options(turned_builder(fill, degrees).size(64, 64).samples(2));
            renderer(&domain, options).render()
        };
        let trivial = colouring(&domain, "1a");
        let cosets = match CosetFill::new(&domain, trivial, vec![first]) {
            Ok(fill) => fill,
            Err(error) => panic!("{text}: {error}"),
        };
        assert!(
            render(Fill::Cosets(cosets)) == render(Fill::One(first)),
            "{text} with 1a and rotation {degrees:?}"
        );
        if !domain.reverses_orientation() {
            println!(
                "{text}, rotation {degrees:?}: 1a equals the one fill, and no side reverses orientation"
            );
            continue;
        }
        let parity = parity_colouring(&domain);
        let id = parity.id();
        let cosets = match CosetFill::new(&domain, parity, vec![first, second]) {
            Ok(fill) => fill,
            Err(error) => panic!("{text}: {error}"),
        };
        assert!(
            render(Fill::Cosets(cosets)) == render(Fill::Parity([first, second])),
            "{text} with {id} and rotation {degrees:?}"
        );
        reversing += 1;
        println!(
            "{text}, rotation {degrees:?}: 1a equals the one fill, and the parity colouring {id} the parity fill"
        );
    }
    assert_eq!(reversing, 18 + 2);
}

#[test]
fn spherical_group_golden_images_give_pairwise_distinct_images() {
    let images: Vec<(&str, Vec<u8>)> = GOLDEN_IMAGES
        .iter()
        .chain(&COSET_GOLDEN_IMAGES)
        .filter(|image| image.file.starts_with("sg-") || image.file.starts_with("cos-sg-"))
        .map(|image| {
            let domain = image.domain();
            let fill = match image.colouring {
                None => Fill::Parity(
                    GOLDEN_FILL.map(|[red, green, blue]| Colour::from_srgb8(red, green, blue)),
                ),
                Some(id) => Fill::Cosets(coset_fill(&domain, golden_colouring(&domain, id))),
            };
            let options = options(
                RenderOptions::builder(fill)
                    .size(64, 64)
                    .samples(2)
                    .border_width(1.0),
            );
            (image.file, renderer(&domain, options).render().0)
        })
        .collect();
    assert_eq!(images.len(), 3);
    for (index, (file, pixels)) in images.iter().enumerate() {
        for (other, other_pixels) in &images[..index] {
            let differing = pixels
                .as_chunks::<3>()
                .0
                .iter()
                .zip(other_pixels.as_chunks::<3>().0)
                .filter(|(left, right)| left != right)
                .count();
            println!("{file} and {other}: {differing} of 4096 pixels differ");
            assert!(pixels != other_pixels, "{file} and {other}");
        }
    }
}

const CENSUS: [&str; 31] = [
    "o", "2222", "333", "442", "632", "*2222", "**", "xx", "*x", "22*", "22x", "2*22", "*333",
    "*442", "*632", "4*2", "3*3", "*532", "532", "*432", "432", "*332", "332", "3*2", "*522",
    "522", "2*5", "*55", "55", "5*", "5x",
];

#[test]
fn every_euclidean_and_spherical_type_renders_with_every_sample_converged() {
    for text in CENSUS {
        let options = options(RenderOptions::builder(one_fill()).size(128, 128).samples(2));
        let stats = render_in_bands(text, &domain_with_settings(text, &[]), options);
        println!(
            "{text} at 128 by 128 with 2 by 2 samples: largest fold {}, {} unconverged",
            stats.largest_fold(),
            stats.unconverged()
        );
        assert_eq!(stats.unconverged(), 0, "{text}");
    }
}

#[test]
fn model_changes_only_the_image_of_a_hyperbolic_domain() {
    let fill =
        Fill::Parity(GOLDEN_FILL.map(|[red, green, blue]| Colour::from_srgb8(red, green, blue)));
    let render = |text: &str, model: Option<HyperbolicModel>| {
        let builder = RenderOptions::builder(fill.clone()).size(64, 48).samples(2);
        let builder = match model {
            Some(model) => builder.model(model),
            None => builder,
        };
        renderer(&domain_with_settings(text, &[]), options(builder))
            .render()
            .0
    };
    for text in ["*632", "*532", "*732"] {
        let default = render(text, None);
        for model in [HyperbolicModel::Disk]
            .into_iter()
            .chain(MODEL_MAPS.iter().map(|map| map.model))
        {
            let pixels = render(text, Some(model));
            let same = pixels == default;
            let expected = text != "*732" || model == HyperbolicModel::Disk;
            println!("{text} with {model:?}: the bytes of the default {same}");
            assert_eq!(same, expected, "{text} with {model:?}");
        }
    }
}

#[test]
fn band_and_half_plane_images_have_no_pixel_of_the_background() {
    let background = Colour::from_srgb8(255, 0, 255);
    let fill =
        Fill::Parity(GOLDEN_FILL.map(|[red, green, blue]| Colour::from_srgb8(red, green, blue)));
    let domain = domain_with_settings("*732", &[]);
    for map in MODEL_MAPS.iter().filter(|map| map.fills_height) {
        for (width, height) in [(192, 192), (256, 128)] {
            let name = format!("*732 in the {:?} model at {width} by {height}", map.model);
            let options = options(
                RenderOptions::builder(fill.clone())
                    .size(width, height)
                    .samples(GOLDEN_GRID)
                    .model(map.model)
                    .background(background),
            );
            let (pixels, stats) = render_image_in_bands(&name, &domain, options);
            let shown = pixels
                .as_chunks::<3>()
                .0
                .iter()
                .filter(|pixel| **pixel == background.to_srgb8())
                .count();
            println!("{name}: {stats:?}, {shown} pixels of the background");
            assert_eq!(stats.outside(), 0, "{name}");
            assert_eq!(shown, 0, "{name}");
        }
    }
}
