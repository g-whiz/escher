use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use escher_cli::Status;
use escher_domain::FundamentalDomain;
use escher_group::{ColouringSet, IndexLimit, SearchBudget, colourings_of_index};
use escher_render::{
    Colour, CosetFill, Fill, Image, Masks, Paint, RenderOptions, Renderer, WholeFace,
    WholeFaceFill, encode_png,
};
use escher_signature::Signature;

use crate::common::{
    Outcome, assert_png_bytes, failure_with_no_file_change, failure_with_no_mask_change,
    failure_with_no_named_mask_change, output_path, path_text, run_args, snapshot,
    whole_face_arrangement,
};
use crate::info::{CHECKERBOARD, THREE_TRIANGLES, TRANSLATIONS_ID, TRANSLATIONS_LINE};

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

#[test]
fn render_masks_diagnostic_says_wythoff_needs_an_arrangement() {
    let output = mask_path("masks_wythoff_without_arrangement").with_file_name("face.png");
    insta::assert_snapshot!(masks_snapshot(&["*632", "--wythoff", "010"], &output));
    assert_eq!(file_names(directory(&output)), names(&[]));
}

#[test]
fn render_masks_refuses_an_arrangement_without_wythoff() {
    let output = mask_path("masks_arrangement_without_wythoff");
    let outcome = render_masks(&["*632", "--arrangement", "12a"], &output);
    assert_eq!(outcome.status, Status::Failure);
    assert_eq!(
        outcome.stderr,
        "error: cannot render the masks of *632: --arrangement needs --wythoff\n"
    );
    assert_eq!(file_names(directory(&output)), names(&[]));
}

struct Tiling {
    signature: &'static str,
    marks: [bool; 3],
    marks_text: &'static str,
}

const HEXAGONS: Tiling = Tiling {
    signature: "*632",
    marks: [false, true, false],
    marks_text: "010",
};

const HEPTAGONS: Tiling = Tiling {
    signature: "*732",
    marks: [false, true, false],
    marks_text: "010",
};

#[test]
fn render_masks_refuses_a_face_mask_with_the_diagnostic_of_render_and_changes_no_file() {
    for (test, args, diagnostic) in [
        (
            "masks_face_missing_id",
            &["*632", "--wythoff", "010", "--arrangement", "12h"][..],
            "error: cannot render the masks of *632: the arrangement 12h does not exist, because the tiling has 7 arrangements with 1 picture\n",
        ),
        (
            "masks_face_none",
            &["*732", "--wythoff", "100", "--arrangement", "6a"],
            "error: cannot render the masks of *732: the tiling has no arrangement that puts one picture on every polygon\n",
        ),
        (
            "masks_face_index_limit",
            &["*(33)32", "--wythoff", "010", "--arrangement", "12a"],
            "error: cannot render the masks of *(33)32\n  caused by: cannot search subgroups: the index limit 66 is not between 1 and 64\n",
        ),
        (
            "masks_face_colouring",
            &[
                "*632",
                "--wythoff",
                "010",
                "--arrangement",
                "12a",
                "--colouring",
                "2a",
            ],
            "error: cannot render the masks of *632: --wythoff cannot be combined with --colouring\n",
        ),
        (
            "masks_face_not_regular",
            &["*522", "--wythoff", "010", "--arrangement", "10a"],
            "error: cannot render the masks of *522\n  caused by: cannot build whole face: the construction is not a regular tiling\n",
        ),
        (
            "masks_face_without_arrangement",
            &["*632", "--wythoff", "010"],
            "error: cannot render the masks of *632: --wythoff needs --arrangement\n",
        ),
        (
            "masks_face_size",
            &[
                "*632",
                "--wythoff",
                "010",
                "--arrangement",
                "12a",
                "--size",
                "0x16",
            ],
            "error: cannot render the masks of *632\n  caused by: cannot build render options: the image size 0x16 is empty\n",
        ),
        (
            "masks_face_set",
            &[
                "*632",
                "--wythoff",
                "010",
                "--arrangement",
                "12a",
                "--set",
                "aspect=2",
            ],
            "error: cannot render the masks of *632: the signature has no shape parameter aspect, nor any other\n",
        ),
    ] {
        assert_eq!(
            failure_with_no_file_change("render-masks", test, args),
            diagnostic,
            "{test}"
        );
    }
}

impl Tiling {
    fn arrangement(&self, id: &str) -> (FundamentalDomain, WholeFace, escher_group::Colouring) {
        whole_face_arrangement(self.signature, self.marks, id)
    }
}

fn library_whole_face_png(domain: &FundamentalDomain, fill: WholeFaceFill, size: u32) -> Vec<u8> {
    let options = RenderOptions::builder(Fill::WholeFace(fill))
        .size(size, size)
        .samples(1)
        .border_width(0.0)
        .build()
        .expect("the options are valid");
    let (pixels, _) = Renderer::new(domain, options)
        .expect("the renderer is built")
        .render();
    encode_png(size, size, &pixels).expect("the tiling is encoded")
}

fn written_face_mask(output: &Path, tiling: &Tiling, id: &str, size: u32) -> Vec<u8> {
    let size_text = format!("{size}x{size}");
    let outcome = render_masks(
        &[
            tiling.signature,
            "--wythoff",
            tiling.marks_text,
            "--arrangement",
            id,
            "--size",
            &size_text,
        ],
        output,
    );
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    assert_eq!(outcome.stdout, "");
    assert_eq!(outcome.stderr, "");
    let (_, face, arrangement) = tiling.arrangement(id);
    let pixels = face
        .render_mask(&arrangement, 0, size, size)
        .expect("the face mask is rendered");
    assert_eq!(
        std::fs::read(output).expect("the face mask is written"),
        encode_png(size, size, &pixels).expect("the face mask is encoded"),
        "{id}"
    );
    pixels
}

#[test]
fn render_masks_writes_exactly_the_face_mask_of_escher_render_to_the_output() {
    let output = mask_path("masks_face_mask").with_file_name("face.png");
    std::fs::write(&output, b"old").expect("the existing file is written");
    written_face_mask(&output, &HEXAGONS, "12a", 40);
    assert_png_bytes(
        &std::fs::read(&output).expect("the face mask is written"),
        40,
        40,
    );
    assert_eq!(file_names(directory(&output)), names(&["face.png"]));
}

#[test]
fn render_masks_names_the_face_mask_that_cannot_be_written() {
    let output = mask_path("masks_face_mask_missing_directory")
        .with_file_name("missing")
        .join("face.png");
    let outcome = render_masks(
        &[
            "*632",
            "--wythoff",
            "010",
            "--arrangement",
            "12a",
            "--size",
            "16x16",
        ],
        &output,
    );
    assert_eq!(outcome.status, Status::Failure);
    assert_eq!(outcome.stdout, "");
    assert_eq!(
        outcome.stderr,
        "error: cannot write the face mask\n  caused by: No such file or directory (os error 2)\n"
    );
}

fn picture_over(mask: &[u8], size: u32, blue: u8) -> Vec<u8> {
    let row = usize::try_from(size).expect("the size fits a usize");
    mask.as_chunks::<3>()
        .0
        .iter()
        .enumerate()
        .flat_map(|(index, &pixel)| {
            let shade =
                |value: usize| u8::try_from(value * 255 / row).expect("a shade fits a byte");
            if pixel == WHITE {
                [shade(index % row), shade(index / row), blue]
            } else {
                pixel
            }
        })
        .collect()
}

fn arrangement_listed(id: &str) -> bool {
    let outcome = run_args(&["info", "*632", "--max-colours", "1", "--wythoff", "010"]);
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    outcome
        .stdout
        .lines()
        .any(|line| line.starts_with(&format!("Arrangement: {id}, ")))
}

#[test]
fn render_with_a_picture_painted_over_the_face_mask_draws_the_whole_face_fill_of_escher_render() {
    let (mask_size, tiling_size) = (192, 128);
    let test = "masks_painted_face";
    let mut files = Vec::new();
    for id in ["12a", "12e"] {
        assert!(arrangement_listed(id), "{id}");
        let mask = mask_path(test).with_file_name(format!("face-{id}.png"));
        let pixels = written_face_mask(&mask, &HEXAGONS, id, mask_size);
        let picture = mask.with_file_name(format!("picture-{id}.png"));
        std::fs::write(
            &picture,
            encode_png(
                mask_size,
                mask_size,
                &picture_over(&pixels, mask_size, 0x80),
            )
            .expect("the picture is encoded"),
        )
        .expect("the picture is written");
        let tiling = mask.with_file_name(format!("tiling-{id}.png"));
        let size_text = format!("{tiling_size}x{tiling_size}");
        let colours = format!("image:{}", path_text(&picture));
        let outcome = run_args(&[
            "render",
            "*632",
            "--wythoff",
            "010",
            "--arrangement",
            id,
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
        let (domain, face, arrangement) = HEXAGONS.arrangement(id);
        let image = Image::decode_png(
            std::fs::read(&picture)
                .expect("the picture is read")
                .as_slice(),
        )
        .expect("the picture decodes");
        let fill = WholeFaceFill::new(face, arrangement, vec![Paint::Image(image)])
            .expect("the fill is valid");
        let written = std::fs::read(&tiling).expect("the tiling is written");
        assert_eq!(
            written,
            library_whole_face_png(&domain, fill, tiling_size),
            "{id}"
        );
        files.push(written);
    }
    assert_ne!(files[0], files[1]);
}

struct SeveralPictures {
    test: &'static str,
    signature: &'static str,
    marks: [bool; 3],
    marks_text: &'static str,
    id: &'static str,
    pictures: u32,
    listed: &'static str,
    masks: &'static [&'static str],
    swapped: (usize, usize),
}

const SQUARES_TWO: SeveralPictures = SeveralPictures {
    test: "masks_squares_two",
    signature: "*442",
    marks: [false, true, false],
    marks_text: "010",
    id: "16ab",
    pictures: 2,
    listed: CHECKERBOARD,
    masks: &["0", "1"],
    swapped: (0, 1),
};

const TRIANGLES_THREE: SeveralPictures = SeveralPictures {
    test: "masks_triangles_three",
    signature: "*732",
    marks: [true, false, false],
    marks_text: "100",
    id: "18a",
    pictures: 3,
    listed: THREE_TRIANGLES,
    masks: &["0", "1", "2"],
    swapped: (0, 1),
};

const TRANSLATIONS_OF_KINDS: SeveralPictures = SeveralPictures {
    test: "masks_translations_of_kinds",
    signature: "*442",
    marks: [true, true, true],
    marks_text: "111",
    id: TRANSLATIONS_ID,
    pictures: 4,
    listed: TRANSLATIONS_LINE,
    masks: &["0A", "1A", "2A", "2B"],
    swapped: (1, 2),
};

const PICTURE_BLUES: [u8; 4] = [0x80, 0x10, 0xf0, 0x40];

fn usize_of(value: u32) -> usize {
    usize::try_from(value).expect("a number of pictures fits a usize")
}

impl SeveralPictures {
    fn tiling_args<'a>(&'a self, extra: &[&'a str]) -> Vec<&'a str> {
        let mut args = vec![
            self.signature,
            "--wythoff",
            self.marks_text,
            "--arrangement",
            self.id,
        ];
        args.extend_from_slice(extra);
        args
    }

    fn mask_names(&self, stem: &str) -> Vec<String> {
        self.masks
            .iter()
            .map(|picture| format!("{stem}-{picture}.png"))
            .collect()
    }

    fn rendered(&self, output: &Path, extra: &[&str]) -> Vec<u8> {
        let mut args = vec!["render"];
        args.extend(self.tiling_args(extra));
        args.extend_from_slice(&["-o", path_text(output)]);
        let outcome = run_args(&args);
        assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
        std::fs::read(output).expect("the tiling is written")
    }
}

fn assert_flow_of_several_pictures(case: &SeveralPictures) {
    let (mask_size, tiling_size) = (96, 96);
    let listing = run_args(&["info", case.signature, "--wythoff", case.marks_text]);
    assert_eq!(listing.status, Status::Success, "{}", listing.stderr);
    assert!(
        listing.stdout.lines().any(|line| line == case.listed),
        "{}",
        case.id
    );
    let mask = mask_path(case.test).with_file_name("face.png");
    let size_text = format!("{mask_size}x{mask_size}");
    let outcome = render_masks(&case.tiling_args(&["--size", &size_text]), &mask);
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    assert_eq!(outcome.stdout, "");
    assert_eq!(outcome.stderr, "");
    let mask_names = case.mask_names("face");
    assert_eq!(
        file_names(directory(&mask)),
        mask_names.iter().cloned().collect()
    );
    let (domain, face, arrangement) = whole_face_arrangement(case.signature, case.marks, case.id);
    let mut items = Vec::new();
    let mut images = Vec::new();
    for (picture, name) in (0..case.pictures).zip(&mask_names) {
        let pixels = face
            .render_mask(&arrangement, picture, mask_size, mask_size)
            .expect("the face mask is rendered");
        let written = std::fs::read(mask.with_file_name(name)).expect("the face mask is written");
        assert_png_bytes(&written, mask_size, mask_size);
        assert_eq!(
            written,
            encode_png(mask_size, mask_size, &pixels).expect("the face mask is encoded"),
            "{name}"
        );
        let painted = picture_over(&pixels, mask_size, PICTURE_BLUES[usize_of(picture)]);
        let file = encode_png(mask_size, mask_size, &painted).expect("the picture is encoded");
        let path = mask.with_file_name(format!("picture-{picture}.png"));
        std::fs::write(&path, &file).expect("the picture is written");
        items.push(format!("image:{}", path_text(&path)));
        images.push(Paint::Image(
            Image::decode_png(file.as_slice()).expect("the picture decodes"),
        ));
    }
    let tiling_text = format!("{tiling_size}x{tiling_size}");
    let options = [
        "--size",
        &tiling_text,
        "--samples",
        "1",
        "--border-width",
        "0",
    ];
    let colours = items.join(",");
    let mut extra = vec!["--colours", &colours];
    extra.extend_from_slice(&options);
    let written = case.rendered(&mask.with_file_name("tiling.png"), &extra);
    let fill = WholeFaceFill::new(face, arrangement, images).expect("the fill is valid");
    assert_eq!(
        written,
        library_whole_face_png(&domain, fill, tiling_size),
        "{}",
        case.id
    );
    items.swap(case.swapped.0, case.swapped.1);
    let swapped = items.join(",");
    let mut extra = vec!["--colours", &swapped];
    extra.extend_from_slice(&options);
    assert_ne!(
        case.rendered(&mask.with_file_name("swapped.png"), &extra),
        written,
        "{}",
        case.id
    );
}

#[test]
fn the_flow_of_two_pictures_on_the_checkerboard_of_the_squares_draws_each_painted_mask() {
    assert_flow_of_several_pictures(&SQUARES_TWO);
}

#[test]
fn the_flow_of_three_pictures_on_the_triangles_of_732_draws_each_painted_mask() {
    assert_flow_of_several_pictures(&TRIANGLES_THREE);
}

#[test]
fn the_flow_of_the_four_pictures_of_the_translations_of_442_with_111_draws_each_painted_mask() {
    assert_flow_of_several_pictures(&TRANSLATIONS_OF_KINDS);
}

fn assert_refusals_of_several_pictures(case: &SeveralPictures, missing: &str, count: &str) {
    let test = format!("{}_refused", case.test);
    let default_colours = ["#e9c46a", "#2a9d8f", "#913671"][..usize_of(case.pictures)].join(",");
    let output = output_path(&format!("{test}_default"));
    assert_eq!(
        case.rendered(&output, &["--size", "24x24"]),
        case.rendered(
            &output.with_file_name("given.png"),
            &["--size", "24x24", "--colours", &default_colours]
        )
    );
    let (image, unreadable) = {
        let image = output.with_file_name("fill.png");
        let item = format!("image:{}", path_text(&image));
        (image, item)
    };
    std::fs::write(&image, b"not a PNG image").expect("the file is written");
    let fewer = ["#000", "#111", "#222", "#333"][..usize_of(case.pictures) - 1].join(",");
    let more = ["#000", "#111", "#222", "#333"][..=usize_of(case.pictures)].join(",");
    let bad_image = ["#000", unreadable.as_str(), "#222"][..usize_of(case.pictures)].join(",");
    let pictures = case.pictures;
    let render_failed = format!("error: cannot render {}: ", case.signature);
    let masks_failed = format!("error: cannot render the masks of {}: ", case.signature);
    let missing_text = format!(
        "the arrangement {missing} does not exist, because the tiling has {count} with {pictures} pictures\n"
    );
    let missing_args = [
        case.signature,
        "--wythoff",
        case.marks_text,
        "--arrangement",
        missing,
    ];
    for (name, command, args, diagnostic) in [
        (
            "fewer",
            "render",
            case.tiling_args(&["--colours", &fewer]),
            format!(
                "{render_failed}the arrangement {} needs {pictures} colours, and {} given\n",
                case.id,
                if pictures == 2 {
                    "1 was".to_owned()
                } else {
                    format!("{} were", pictures - 1)
                }
            ),
        ),
        (
            "more",
            "render",
            case.tiling_args(&["--colours", &more]),
            format!(
                "{render_failed}the arrangement {} needs {pictures} colours, and {} were given\n",
                case.id,
                pictures + 1
            ),
        ),
        (
            "missing",
            "render",
            missing_args.to_vec(),
            format!("{render_failed}{missing_text}"),
        ),
        (
            "image",
            "render",
            case.tiling_args(&["--colours", &bad_image]),
            format!(
                "{render_failed}the image of colour 2 of --colours cannot be used\n  caused by: cannot decode image: the data is not a valid PNG image\n  caused by: Invalid PNG signature.\n"
            ),
        ),
        (
            "masks_missing",
            "render-masks",
            missing_args.to_vec(),
            format!("{masks_failed}{missing_text}"),
        ),
    ] {
        assert_eq!(
            failure_with_no_mask_change(command, &format!("{test}_{name}"), &args, pictures),
            diagnostic,
            "{name}"
        );
    }
    let no_file = format!("{masks_failed}the output path names no file\n");
    let empty_stem = case.mask_names("");
    assert_eq!(
        stemless_refusal(case, &format!("{test}_stemless_existing"), &empty_stem),
        no_file
    );
    assert_eq!(
        stemless_refusal(case, &format!("{test}_stemless_missing"), &[]),
        no_file
    );
}

fn stemless_refusal(case: &SeveralPictures, test: &str, existing: &[String]) -> String {
    let directory = directory(&output_path(test)).to_path_buf();
    let inner = directory.join("inner");
    std::fs::create_dir(&inner).expect("the inner directory is created");
    for name in existing {
        std::fs::write(directory.join(name), b"old").expect("the existing file is written");
    }
    let outcome = render_masks(&case.tiling_args(&[]), &inner.join(".."));
    assert_eq!(outcome.status, Status::Failure, "{test}");
    assert_eq!(outcome.stdout, "", "{test}");
    let mut expected: BTreeSet<String> = existing.iter().cloned().collect();
    expected.insert("inner".to_owned());
    assert_eq!(file_names(&directory), expected, "{test}");
    assert_eq!(file_names(&inner), names(&[]), "{test}");
    for name in existing {
        assert_eq!(
            std::fs::read(directory.join(name)).expect("the existing file stays"),
            b"old",
            "{test} {name}"
        );
    }
    outcome.stderr
}

#[test]
fn several_pictures_on_the_checkerboard_of_the_squares_are_refused_without_a_file_change() {
    assert_refusals_of_several_pictures(&SQUARES_TWO, "16ad", "29 arrangements");
}

#[test]
fn several_pictures_on_the_triangles_of_732_are_refused_without_a_file_change() {
    assert_refusals_of_several_pictures(&TRIANGLES_THREE, "18b", "1 arrangement");
}

#[test]
fn render_masks_writes_the_masks_before_the_picture_that_cannot_be_written_and_none_after() {
    let output = mask_path("masks_three_pictures_directory").with_file_name("face.png");
    std::fs::create_dir(output.with_file_name("face-1.png")).expect("the directory is created");
    let outcome = render_masks(&TRIANGLES_THREE.tiling_args(&["--size", "16x16"]), &output);
    assert_eq!(outcome.status, Status::Failure);
    assert_eq!(outcome.stdout, "");
    assert_eq!(
        outcome.stderr,
        "error: cannot write the face mask of picture B\n  caused by: Is a directory (os error 21)\n"
    );
    let (_, face, arrangement) = whole_face_arrangement("*732", [true, false, false], "18a");
    let pixels = face
        .render_mask(&arrangement, 0, 16, 16)
        .expect("the face mask is rendered");
    assert_eq!(
        std::fs::read(output.with_file_name("face-0.png")).expect("the mask of A is written"),
        encode_png(16, 16, &pixels).expect("the face mask is encoded")
    );
    assert_eq!(
        file_names(directory(&output)),
        names(&["face-0.png", "face-1.png"])
    );
}

#[test]
fn render_masks_diagnostic_says_the_output_path_of_several_pictures_names_no_file() {
    insta::assert_snapshot!(masks_snapshot(
        &["*732", "--wythoff", "100", "--arrangement", "18a"],
        Path::new("/")
    ));
}

#[test]
fn render_masks_refuses_an_output_path_without_a_file_stem_before_a_bad_size() {
    let outcome = render_masks(
        &TRIANGLES_THREE.tiling_args(&["--size", "0x16"]),
        Path::new("/"),
    );
    assert_eq!(outcome.status, Status::Failure);
    assert_eq!(outcome.stdout, "");
    assert_eq!(
        outcome.stderr,
        "error: cannot render the masks of *732: the output path names no file\n"
    );
}

#[test]
fn render_masks_refuses_a_tiling_that_is_not_regular_before_a_path_without_a_file_stem() {
    let outcome = render_masks(
        &["*522", "--wythoff", "010", "--arrangement", "20a"],
        Path::new("/"),
    );
    assert_eq!(outcome.status, Status::Failure);
    assert_eq!(outcome.stdout, "");
    assert_eq!(
        outcome.stderr,
        "error: cannot render the masks of *522\n  caused by: cannot build whole face: the construction is not a regular tiling\n"
    );
}

#[test]
fn render_masks_names_the_face_masks_of_three_pictures_by_the_rule_of_the_polygon_masks() {
    for (test, output, expected) in [
        (
            "masks_three_pictures_extension",
            "face.png",
            ["face-0.png", "face-1.png", "face-2.png"],
        ),
        (
            "masks_three_pictures_no_extension",
            "face",
            ["face-0", "face-1", "face-2"],
        ),
        (
            "masks_three_pictures_two_dots",
            "face.tar.png",
            ["face.tar-0.png", "face.tar-1.png", "face.tar-2.png"],
        ),
    ] {
        let output = mask_path(test).with_file_name(output);
        let outcome = render_masks(&TRIANGLES_THREE.tiling_args(&["--size", "8x8"]), &output);
        assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
        assert_eq!(file_names(directory(&output)), names(&expected), "{test}");
    }
}

#[test]
fn render_masks_diagnostic_says_the_tiling_has_no_arrangement_with_2_pictures() {
    let output = mask_path("masks_no_arrangement_of_two_pictures");
    insta::assert_snapshot!(masks_snapshot(
        &["*732", "--wythoff", "100", "--arrangement", "12a"],
        &output
    ));
    assert_eq!(file_names(directory(&output)), names(&[]));
}

#[test]
fn an_arrangement_with_one_picture_writes_one_mask_at_the_output_and_renders_with_one_paint() {
    let size = 24;
    let size_text = format!("{size}x{size}");
    for (tiling, id) in [(&HEPTAGONS, "14a"), (&HEXAGONS, "12a"), (&HEXAGONS, "12g")] {
        let output = mask_path(&format!("masks_one_picture_{id}")).with_file_name("face.png");
        written_face_mask(&output, tiling, id, size);
        assert_eq!(file_names(directory(&output)), names(&["face.png"]), "{id}");
        let rendered = output.with_file_name("tiling.png");
        let outcome = run_args(&[
            "render",
            tiling.signature,
            "--wythoff",
            tiling.marks_text,
            "--arrangement",
            id,
            "--colours",
            "#123456",
            "--size",
            &size_text,
            "--samples",
            "1",
            "--border-width",
            "0",
            "-o",
            path_text(&rendered),
        ]);
        assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
        let (domain, face, arrangement) = tiling.arrangement(id);
        let paint = Paint::Solid(Colour::from_srgb8(0x12, 0x34, 0x56));
        let fill = WholeFaceFill::new(face, arrangement, vec![paint]).expect("the fill is valid");
        assert_eq!(
            std::fs::read(&rendered).expect("the tiling is written"),
            library_whole_face_png(&domain, fill, size),
            "{id}"
        );
    }
}

#[test]
fn render_masks_of_several_face_types_writes_the_masks_before_picture_1b_that_cannot_be_written() {
    let output = mask_path("masks_kinds_directory").with_file_name("face.png");
    std::fs::create_dir(output.with_file_name("face-1B.png")).expect("the directory is created");
    let outcome = render_masks(
        &[
            "*632",
            "--wythoff",
            "111",
            "--arrangement",
            "12a",
            "--size",
            "16x16",
        ],
        &output,
    );
    assert_eq!(outcome.status, Status::Failure);
    assert_eq!(outcome.stdout, "");
    assert_eq!(
        outcome.stderr,
        "error: cannot write the face mask of picture 1B\n  caused by: Is a directory (os error 21)\n"
    );
    let (_, face, arrangement) = whole_face_arrangement("*632", [true, true, true], "12a");
    for (picture, name) in [(0, "face-0A.png"), (1, "face-1A.png")] {
        let pixels = face
            .render_mask(&arrangement, picture, 16, 16)
            .expect("the face mask is rendered");
        assert_eq!(
            std::fs::read(output.with_file_name(name)).expect("the mask is written"),
            encode_png(16, 16, &pixels).expect("the face mask is encoded"),
            "{name}"
        );
    }
    assert_eq!(
        file_names(directory(&output)),
        names(&["face-0A.png", "face-1A.png", "face-1B.png"])
    );
}

#[test]
fn render_masks_of_several_face_types_refuses_an_output_path_without_a_file_stem() {
    let case = &TRANSLATIONS_OF_KINDS;
    let no_file = "error: cannot render the masks of *442: the output path names no file\n";
    assert_eq!(
        stemless_refusal(case, "masks_kinds_stemless_existing", &case.mask_names("")),
        no_file
    );
    assert_eq!(
        stemless_refusal(case, "masks_kinds_stemless_missing", &[]),
        no_file
    );
}

#[test]
fn render_masks_of_several_face_types_refuses_an_id_off_the_least_index_without_a_file_change() {
    assert_eq!(
        failure_with_no_named_mask_change(
            "render-masks",
            "masks_kinds_off_least_index",
            &["*442", "--wythoff", "111", "--arrangement", "12a"],
            TRANSLATIONS_OF_KINDS.masks
        ),
        "error: cannot render the masks of *442: the arrangement 12a does not exist, because the tiling has 3 arrangements with 4 pictures\n"
    );
}

fn polygon_area(pixels: &[u8], size: u32) -> f64 {
    let row = usize_of(size);
    let mut points: Vec<(f64, f64)> = pixels
        .as_chunks::<3>()
        .0
        .iter()
        .enumerate()
        .filter(|(_, pixel)| pixel.iter().any(|&channel| channel > 0x40))
        .map(|(index, _)| {
            let column = u32::try_from(index % row).expect("a column fits a u32");
            let line = u32::try_from(index / row).expect("a row fits a u32");
            (f64::from(column), f64::from(line))
        })
        .collect();
    points.sort_by(|a, b| a.partial_cmp(b).expect("pixel positions are finite"));
    let cross = |o: (f64, f64), a: (f64, f64), b: (f64, f64)| {
        (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
    };
    let mut hull: Vec<(f64, f64)> = Vec::new();
    for pass in [points.clone(), points.into_iter().rev().collect()] {
        let start = hull.len();
        for point in pass {
            while hull.len() >= start + 2
                && cross(hull[hull.len() - 2], hull[hull.len() - 1], point) <= 0.0
            {
                hull.pop();
            }
            hull.push(point);
        }
        hull.pop();
    }
    let twice: f64 = (0..hull.len())
        .map(|index| {
            let (a, b) = (hull[index], hull[(index + 1) % hull.len()]);
            a.0 * b.1 - b.0 * a.1
        })
        .sum();
    twice.abs() / 2.0
}

#[test]
fn the_masks_of_several_face_types_share_one_scale() {
    let size = 256;
    let output = mask_path("masks_kinds_scale").with_file_name("face.png");
    let size_text = format!("{size}x{size}");
    let outcome = render_masks(
        &TRANSLATIONS_OF_KINDS.tiling_args(&["--size", &size_text]),
        &output,
    );
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    let (_, face, arrangement) = whole_face_arrangement("*442", [true, true, true], "8b");
    let areas: Vec<f64> = (0..4)
        .map(|picture| {
            let pixels = face
                .render_mask(&arrangement, picture, size, size)
                .expect("the face mask is rendered");
            let name = format!(
                "face-{}.png",
                TRANSLATIONS_OF_KINDS.masks[usize_of(picture)]
            );
            assert_eq!(
                std::fs::read(output.with_file_name(&name)).expect("the mask is written"),
                encode_png(size, size, &pixels).expect("the face mask is encoded"),
                "{name}"
            );
            polygon_area(&pixels, size)
        })
        .collect();
    let octagon_over_square = 2.0 * (1.0 + core::f64::consts::SQRT_2);
    let close = |measured: f64, expected: f64| (measured / expected - 1.0).abs() < 0.02;
    assert!(close(areas[0] / areas[1], 1.0), "{areas:?}");
    assert!(close(areas[2] / areas[3], 1.0), "{areas:?}");
    assert!(close(areas[0] / areas[2], octagon_over_square), "{areas:?}");
    assert!(areas[0] > f64::from(size * size) / 4.0, "{areas:?}");
}
