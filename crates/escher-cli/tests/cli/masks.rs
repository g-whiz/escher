use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use escher_cli::Status;
use escher_domain::FundamentalDomain;
use escher_group::{ColouringSet, IndexLimit, SearchBudget, colourings_of_index};
use escher_render::{
    Colour, CosetFill, Fill, Image, Masks, Paint, RenderOptions, Renderer, encode_png,
};
use escher_signature::Signature;

use crate::common::{Outcome, assert_png_bytes, output_path, path_text, run_args, snapshot};

fn mask_path(test: &str) -> PathBuf {
    output_path(test).with_file_name("mask.png")
}

fn masks_args<'a>(args: &[&'a str], output: &'a Path) -> Vec<&'a str> {
    let mut all = vec!["render-masks"];
    all.extend_from_slice(args);
    all.extend_from_slice(&["-o", path_text(output)]);
    all
}

fn render_masks(args: &[&str], output: &Path) -> Outcome {
    run_args(&masks_args(args, output))
}

fn masks_snapshot(args: &[&str], output: &Path) -> String {
    snapshot(&masks_args(args, output))
}

fn file_names(directory: &Path) -> BTreeSet<String> {
    std::fs::read_dir(directory)
        .expect("the directory of the test is read")
        .map(|entry| {
            entry
                .expect("the entry is read")
                .file_name()
                .into_string()
                .expect("the file name is UTF-8")
        })
        .collect()
}

fn names(texts: &[&str]) -> BTreeSet<String> {
    texts.iter().map(|text| (*text).to_owned()).collect()
}

fn directory(output: &Path) -> &Path {
    output.parent().expect("the output path has a directory")
}

#[test]
fn render_masks_diagnostic_says_the_output_path_names_no_file() {
    let output = mask_path("masks_no_stem").with_file_name("..");
    insta::assert_snapshot!(masks_snapshot(&["*732"], &output));
}

#[test]
fn render_masks_diagnostic_says_a_colouring_id_does_not_exist() {
    let output = mask_path("masks_missing_id");
    insta::assert_snapshot!(masks_snapshot(&["*642", "--colouring", "3a"], &output));
}

#[test]
fn render_masks_diagnostic_says_the_lune_has_no_mask_polygon() {
    let output = mask_path("masks_lune");
    insta::assert_snapshot!(masks_snapshot(&["*22"], &output));
}

#[test]
fn render_masks_diagnostic_names_the_colour_of_a_mask_that_cannot_be_written() {
    let output = mask_path("masks_missing_directory")
        .with_file_name("missing")
        .join("mask.png");
    insta::assert_snapshot!(masks_snapshot(&["*732", "--size", "16x16"], &output));
}

#[test]
fn render_masks_writes_one_png_file_of_the_size_for_each_colour() {
    let output = mask_path("masks_four_colours");
    let outcome = render_masks(&["*642", "--colouring", "4a", "--size", "40x30"], &output);
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    assert_eq!(outcome.stdout, "");
    assert_eq!(outcome.stderr, "");
    let written = ["mask-0.png", "mask-1.png", "mask-2.png", "mask-3.png"];
    assert_eq!(file_names(directory(&output)), names(&written));
    for name in written {
        let file = std::fs::read(output.with_file_name(name)).expect("the mask is written");
        assert_png_bytes(&file, 40, 30);
    }
}

#[test]
fn render_masks_writes_files_without_an_extension_for_an_output_without_one() {
    let output = mask_path("masks_no_extension").with_file_name("masks");
    let outcome = render_masks(&["*632", "--size", "16x16"], &output);
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    assert_eq!(
        file_names(directory(&output)),
        names(&["masks-0", "masks-1"])
    );
}

#[test]
fn render_masks_replaces_an_existing_file() {
    let output = mask_path("masks_existing_file");
    let existing = output.with_file_name("mask-0.png");
    std::fs::write(&existing, b"old").expect("the existing file is written");
    let outcome = render_masks(&["*632", "--size", "16x16"], &output);
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    let file = std::fs::read(&existing).expect("the mask is written");
    assert_png_bytes(&file, 16, 16);
}

#[test]
fn render_masks_with_a_failed_check_changes_no_file() {
    let output = mask_path("masks_failed_check");
    let existing = output.with_file_name("mask-0.png");
    std::fs::write(&existing, b"old").expect("the existing file is written");
    for args in [
        &["*642", "--colouring", "3a"][..],
        &["*642", "--size", "0x16"],
        &["*22"],
        &["4*3"],
        &["*2222", "--set", "aspect=9"],
    ] {
        let outcome = render_masks(args, &output);
        assert_eq!(outcome.status, Status::Failure, "{args:?}");
        assert_eq!(outcome.stdout, "", "{args:?}");
    }
    assert_eq!(file_names(directory(&output)), names(&["mask-0.png"]));
    assert_eq!(
        std::fs::read(&existing).expect("the existing file stays"),
        b"old"
    );
}

#[test]
fn render_masks_with_parity_for_a_domain_without_a_valid_colouring_with_two_colours_returns_failure()
 {
    let output = mask_path("masks_parity_rotation");
    let outcome = render_masks(&["732", "--colouring", "parity"], &output);
    assert_eq!(outcome.status, Status::Failure);
    assert_eq!(outcome.stdout, "");
    assert_eq!(
        outcome.stderr,
        "error: cannot render the masks of 732: the colouring parity does not exist, because the signature has no colouring with 2 colours\n"
    );
    assert_eq!(file_names(directory(&output)), names(&[]));
}

#[test]
fn render_masks_stops_at_the_first_mask_that_cannot_be_written() {
    let output = mask_path("masks_failed_write");
    std::fs::create_dir(output.with_file_name("mask-2.png")).expect("the directory is created");
    let outcome = render_masks(&["*642", "--colouring", "4a", "--size", "16x16"], &output);
    assert_eq!(outcome.status, Status::Failure);
    assert_eq!(outcome.stdout, "");
    assert_eq!(
        outcome.stderr,
        "error: cannot write the mask of colour 2\n  caused by: Is a directory (os error 21)\n"
    );
    assert_eq!(
        file_names(directory(&output)),
        names(&["mask-0.png", "mask-1.png", "mask-2.png"])
    );
    for name in ["mask-0.png", "mask-1.png"] {
        let file = std::fs::read(output.with_file_name(name)).expect("the mask is written");
        assert_png_bytes(&file, 16, 16);
    }
    assert!(output.with_file_name("mask-2.png").is_dir());
}

#[test]
fn render_masks_writes_one_file_for_each_colour_of_the_default_colouring_and_of_one() {
    for (test, args, count) in [
        ("masks_default_parity", &["*632"][..], 2),
        ("masks_default_rotation_parity", &["442"], 2),
        ("masks_default_one", &["632"], 1),
        ("masks_one", &["*632", "--colouring", "one"], 1),
        ("masks_trivial", &["*632", "--colouring", "1a"], 1),
    ] {
        let output = mask_path(test);
        let mut all = args.to_vec();
        all.extend_from_slice(&["--size", "16x16"]);
        let outcome = render_masks(&all, &output);
        assert_eq!(
            outcome.status,
            Status::Success,
            "{test}: {}",
            outcome.stderr
        );
        let expected: BTreeSet<String> = (0..count)
            .map(|colour| format!("mask-{colour}.png"))
            .collect();
        assert_eq!(file_names(directory(&output)), expected, "{test}");
    }
}

fn domain(text: &str) -> FundamentalDomain {
    let signature: Signature = text.parse().expect("the signature is valid");
    FundamentalDomain::build(&signature).expect("the signature has a domain")
}

fn colouring(domain: &FundamentalDomain, id: &str) -> escher_group::Colouring {
    let id: escher_group::ColouringId = id.parse().expect("the id is valid");
    let presentation = domain
        .side_presentation()
        .expect("the domain has a side presentation");
    let colours = IndexLimit::new(id.colours()).expect("the colour count is valid");
    colourings_of_index(
        &presentation,
        ColouringSet::ValidClasses,
        colours,
        SearchBudget::DEFAULT,
    )
    .expect("the search runs")
    .find(id)
    .expect("the colouring exists")
    .clone()
}

fn written_masks(test: &str, signature: &str, id: &str, size: u32) -> Vec<Vec<u8>> {
    let output = mask_path(test);
    let size_text = format!("{size}x{size}");
    let outcome = render_masks(
        &[signature, "--colouring", id, "--size", &size_text],
        &output,
    );
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    let domain = domain(signature);
    let masks = Masks::new(&domain, &colouring(&domain, id)).expect("the domain has masks");
    (0..masks.count())
        .map(|colour| {
            let pixels = masks
                .render(colour, size, size)
                .expect("the mask is rendered");
            let file = std::fs::read(output.with_file_name(format!("mask-{colour}.png")))
                .expect("the mask is written");
            let expected = encode_png(size, size, &pixels).expect("the mask is encoded");
            assert_eq!(file, expected, "colour {colour}");
            pixels
        })
        .collect()
}

#[test]
fn render_masks_writes_the_masks_of_escher_render() {
    assert_eq!(written_masks("masks_bytes", "*642", "4a", 48).len(), 4);
}

const PAINTS: [[u8; 3]; 3] = [[0xdc, 0x14, 0x3c], [0xff, 0xd7, 0x00], [0x00, 0x00, 0x80]];

const WHITE: [u8; 3] = [0xff, 0xff, 0xff];

const RADIUS: usize = 2;

fn painted(pixels: &[u8], paint: [u8; 3]) -> Vec<u8> {
    pixels
        .as_chunks::<3>()
        .0
        .iter()
        .flat_map(|&pixel| if pixel == WHITE { paint } else { pixel })
        .collect()
}

fn library_render(domain: &FundamentalDomain, paints: Vec<Paint>, size: u32) -> Vec<u8> {
    let fill = CosetFill::new(domain, colouring(domain, "3a"), paints).expect("the fill is valid");
    let options = RenderOptions::builder(Fill::Cosets(fill))
        .size(size, size)
        .samples(1)
        .border_width(0.0)
        .build()
        .expect("the options are valid");
    Renderer::new(domain, options)
        .expect("the renderer is built")
        .render()
        .0
}

fn pixel(pixels: &[u8], index: usize) -> Option<&[u8]> {
    pixels.get(index * 3..index * 3 + 3)
}

fn is_interior(pixels: &[u8], size: usize, index: usize) -> bool {
    let (column, row) = (index % size, index / size);
    let centre = pixel(pixels, index);
    (row.saturating_sub(RADIUS)..=row + RADIUS).all(|other_row| {
        (column.saturating_sub(RADIUS)..=column + RADIUS).all(|other_column| {
            other_row < size
                && other_column < size
                && pixel(pixels, other_row * size + other_column) == centre
        })
    })
}

#[test]
fn render_with_painted_masks_fills_each_tile_with_the_paint_of_its_colour() {
    let (mask_size, tiling_size) = (256, 192);
    let test = "masks_painted";
    let masks = written_masks(test, "*632", "3a", mask_size);
    let output = mask_path(test);
    let mut items = Vec::new();
    for ((colour, pixels), paint) in masks.iter().enumerate().zip(PAINTS) {
        let path = output.with_file_name(format!("fill-{colour}.png"));
        let file = encode_png(mask_size, mask_size, &painted(pixels, paint))
            .expect("the painted mask is encoded");
        std::fs::write(&path, file).expect("the painted mask is written");
        items.push(format!("image:{}", path_text(&path)));
    }
    let tiling = output.with_file_name("tiling.png");
    let size_text = format!("{tiling_size}x{tiling_size}");
    let colours = items.join(",");
    let outcome = run_args(&[
        "render",
        "*632",
        "--colouring",
        "3a",
        "--colours",
        &colours,
        "--size",
        &size_text,
        "--samples",
        "1",
        "--border-width",
        "0",
        "-o",
        path_text(&tiling),
    ]);
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    let domain = domain("*632");
    let images = (0..PAINTS.len())
        .map(|colour| {
            let file = std::fs::read(output.with_file_name(format!("fill-{colour}.png")))
                .expect("the painted mask is read");
            Paint::Image(Image::decode_png(file.as_slice()).expect("the painted mask decodes"))
        })
        .collect();
    let filled = library_render(&domain, images, tiling_size);
    assert_eq!(
        std::fs::read(&tiling).expect("the tiling is written"),
        encode_png(tiling_size, tiling_size, &filled).expect("the tiling is encoded")
    );
    let solid = library_render(
        &domain,
        PAINTS
            .iter()
            .map(|&[red, green, blue]| Paint::Solid(Colour::from_srgb8(red, green, blue)))
            .collect(),
        tiling_size,
    );
    let side = usize::try_from(tiling_size).expect("the size fits a usize");
    let interior: Vec<usize> = (0..side * side)
        .filter(|&index| is_interior(&solid, side, index))
        .collect();
    assert!(
        interior.len() > side * side / 4,
        "{} interior pixels",
        interior.len()
    );
    for index in interior {
        assert_eq!(pixel(&filled, index), pixel(&solid, index), "pixel {index}");
    }
}
