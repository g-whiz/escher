use std::path::PathBuf;

use escher_domain::FundamentalDomain;
use escher_render::{Colour, ErrorKind, Fill, RenderOptions, RenderStats, Renderer, encode_png};
use escher_signature::Signature;

use crate::common::{decode_png, domain, one_fill, options};
use crate::golden_images::{
    ALLOWED_PIXELS, GOLDEN_GRID, GOLDEN_IMAGES, GOLDEN_SIZE, GoldenImage, compare, golden_options,
    linear_table,
};

const UPDATE: &str = "ESCHER_UPDATE_GOLDEN";

const RECIPE: &str = "just update-golden";

fn golden_pixels(image: &GoldenImage) -> Vec<u8> {
    let signature: Signature = image
        .signature
        .parse()
        .unwrap_or_else(|error| panic!("{} is rejected: {error}", image.signature));
    let domain = FundamentalDomain::build(&signature)
        .unwrap_or_else(|error| panic!("{} has no domain: {error}", image.signature));
    Renderer::new(&domain, golden_options()).render().0
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
    let name = image.signature;
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
    let options = options(
        RenderOptions::builder(Fill::Parity([
            Colour::WHITE,
            Colour::from_srgb8(220, 20, 60),
        ]))
        .size(64, 48)
        .samples(3),
    );
    let renderer = Renderer::new(&domain([7, 3, 2]), options);
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
