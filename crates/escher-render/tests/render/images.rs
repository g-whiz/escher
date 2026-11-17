use std::path::PathBuf;

use escher_render::{Colour, ErrorKind, Fill, RenderOptions, RenderStats, Renderer, encode_png};

use crate::common::{decode_png, domain, one_fill, options};
use crate::golden_images::{
    ALLOWED_PIXELS, GOLDEN_FILL, GOLDEN_GRID, GOLDEN_IMAGES, GOLDEN_SIZE, GoldenImage,
    TRANSLATION_TOLERANCE, compare, domain_with_aspect, linear_table, shifted_pairs, translation,
};

const UPDATE: &str = "ESCHER_UPDATE_GOLDEN";

const RECIPE: &str = "just update-golden";

fn golden_pixels(image: &GoldenImage) -> Vec<u8> {
    Renderer::new(&image.domain(), image.options()).render().0
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
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter, and the test reads files"
)]
fn golden_images_match_the_committed_files() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/render/golden");
    let update = std::env::var_os(UPDATE).is_some_and(|value| value == "1");
    let linear = linear_table();
    let mut failures = Vec::new();
    for image in &GOLDEN_IMAGES {
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
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
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
        assert_bands_equal_the_whole_image(&Renderer::new(&domain, options.clone()));
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
    let renderer = Renderer::new(&domain([7, 3, 2]), options);
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
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
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
            let (pixels, _) = Renderer::new(&domain, options.clone()).render();
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
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
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
        let (pixels, _) = Renderer::new(&domain, options).render();
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
    let options = options(RenderOptions::builder(one_fill()));
    let (_, stats) = Renderer::new(&domain_with_aspect(text, None), options).render();
    let inside = stats.samples() - stats.outside();
    #[expect(
        clippy::cast_precision_loss,
        reason = "the counts are far below 2 to the power 52"
    )]
    let mean = stats.folds() as f64 / inside as f64;
    println!(
        "{text} at 512 by 512 with 4 by 4 samples: {inside} samples inside the ball, mean steps {mean:.2}, largest {}, group order {order}, {} unconverged",
        stats.largest_fold(),
        stats.unconverged()
    );
    assert!(u64::from(stats.largest_fold()) < order.get(), "{text}");
    assert_eq!(stats.unconverged(), 0, "{text}");
}

#[test]
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
fn spherical_golden_domains_fold_every_sample_within_the_group_order() {
    let mut texts: Vec<&str> = Vec::new();
    for image in &GOLDEN_IMAGES {
        let spherical = image.domain().geometry() == escher_signature::Geometry::Spherical;
        let reflection = image.signature.starts_with('*');
        if spherical && reflection && !texts.contains(&image.signature) {
            texts.push(image.signature);
        }
    }
    for text in texts {
        assert_folds_within_the_group_order(text);
    }
    let small = options(RenderOptions::builder(one_fill()).size(4, 4).samples(1));
    let (_, stats) = Renderer::new(&domain_with_aspect("*532", None), small).render();
    assert_eq!(stats.samples(), 16);
    assert_eq!(stats.outside(), 4);
}

#[test]
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
fn triangle_of_order_1000_folds_every_sample_within_the_group_order() {
    assert_folds_within_the_group_order("*(1000)22");
}

#[test]
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
fn lune_of_order_1000_folds_every_sample_within_the_group_order() {
    assert_folds_within_the_group_order("*(1000)(1000)");
}

#[test]
#[cfg_attr(
    miri,
    ignore = "rendering whole images runs loops too long for the interpreter"
)]
fn parity_fill_of_a_rotation_domain_equals_its_first_colour() {
    let first = Colour::from_srgb8(245, 222, 179);
    let second = Colour::from_srgb8(70, 130, 180);
    for text in ["732", "55"] {
        let domain = domain_with_aspect(text, None);
        let render = |fill| {
            let options = options(RenderOptions::builder(fill).size(64, 64).samples(2));
            Renderer::new(&domain, options).render().0
        };
        assert!(
            render(Fill::Parity([first, second])) == render(Fill::One(first)),
            "{text}"
        );
    }
}
