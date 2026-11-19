use std::io;
use std::path::Path;

use escher_cli::{Status, run};
use escher_domain::FundamentalDomain;
use escher_group::{IndexLimit, SearchBudget, colourings_of_index};
use escher_render::{Colour, CosetFill, Fill, RenderOptions, Renderer, encode_png};
use escher_signature::Signature;

use crate::common::{FailingWriter, NEVER_WRITTEN, output_path, path_text, run_args, snapshot};

const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

fn render_snapshot(test: &str, args: &[&str]) -> String {
    let output = output_path(test);
    let mut all = vec!["render"];
    all.extend_from_slice(args);
    all.extend_from_slice(&["-o", path_text(&output)]);
    snapshot(&all)
}

#[test]
fn render_diagnostic_marks_a_bad_orbifold() {
    insta::assert_snapshot!(render_snapshot("bad_orbifold", &["23"]));
}

#[test]
fn render_diagnostic_says_a_non_compact_signature_is_not_supported() {
    insta::assert_snapshot!(render_snapshot("non_compact", &["3*(inf)"]));
}

#[test]
fn render_diagnostic_gives_the_cause_of_an_order_over_the_limit() {
    insta::assert_snapshot!(render_snapshot("order_limit", &["*(1001)32"]));
}

#[test]
fn render_diagnostic_says_parity_does_not_exist_without_a_reversing_side() {
    insta::assert_snapshot!(render_snapshot(
        "parity_without_reversing_side",
        &["732", "--colouring", "parity"]
    ));
}

#[test]
fn render_diagnostic_counts_two_colours_for_the_default_colouring_of_a_rotation_domain() {
    insta::assert_snapshot!(render_snapshot(
        "two_colours_for_rotation_domain",
        &["732", "--colours", "#e9c46a,#2a9d8f"]
    ));
}

#[test]
fn render_diagnostic_counts_three_colours_for_parity() {
    insta::assert_snapshot!(render_snapshot(
        "three_colours",
        &["*732", "--colours", "#e9c46a,#2a9d8f,#264653"]
    ));
}

#[test]
fn render_diagnostic_counts_one_colour_for_parity() {
    insta::assert_snapshot!(render_snapshot(
        "one_colour",
        &["*732", "--colours", "#e9c46a"]
    ));
}

#[test]
fn render_diagnostic_gives_the_position_of_a_colour_name() {
    insta::assert_snapshot!(render_snapshot(
        "colour_name",
        &["*732", "--colours", "#e9c46a,red"]
    ));
}

#[test]
fn render_diagnostic_says_a_colouring_id_does_not_exist() {
    insta::assert_snapshot!(render_snapshot(
        "missing_id",
        &["*642", "--colouring", "3d"]
    ));
}

#[test]
fn render_diagnostic_asks_for_the_colours_of_an_id_with_more_than_twelve() {
    insta::assert_snapshot!(render_snapshot(
        "fifteen_colours",
        &["*732", "--colouring", "15a"]
    ));
}

#[test]
fn render_diagnostic_counts_two_colours_for_an_id_with_three() {
    insta::assert_snapshot!(render_snapshot(
        "two_colours_for_id",
        &["*642", "--colouring", "3a", "--colours", "#e9c46a,#2a9d8f"]
    ));
}

#[test]
fn render_diagnostic_says_the_search_for_an_id_stopped_at_a_limit() {
    insta::assert_snapshot!(render_snapshot(
        "search_limit",
        &["*666", "--colouring", "12a"]
    ));
}

#[test]
fn render_diagnostic_names_a_bad_border_colour() {
    insta::assert_snapshot!(render_snapshot(
        "border_colour",
        &["*732", "--border-colour", "red"]
    ));
}

#[test]
fn render_diagnostic_names_a_bad_background() {
    insta::assert_snapshot!(render_snapshot(
        "background",
        &["*732", "--background", "#ffff"]
    ));
}

#[test]
fn render_diagnostic_gives_the_cause_of_an_empty_size() {
    insta::assert_snapshot!(render_snapshot("empty_size", &["*732", "--size", "0x0"]));
}

#[test]
fn render_diagnostic_gives_the_cause_of_a_size_over_the_limit() {
    insta::assert_snapshot!(render_snapshot(
        "size_limit",
        &["*732", "--size", "16385x512"]
    ));
}

#[test]
fn render_diagnostic_gives_the_cause_of_an_empty_sample_grid() {
    insta::assert_snapshot!(render_snapshot("empty_grid", &["*732", "--samples", "0"]));
}

#[test]
fn render_diagnostic_gives_the_cause_of_a_sample_grid_over_the_limit() {
    insta::assert_snapshot!(render_snapshot("grid_limit", &["*732", "--samples", "17"]));
}

#[test]
fn render_diagnostic_gives_the_cause_of_a_negative_border_width() {
    insta::assert_snapshot!(render_snapshot(
        "negative_border",
        &["*732", "--border-width=-1"]
    ));
}

#[test]
fn render_diagnostic_gives_the_cause_of_a_missing_directory() {
    let output = output_path("missing_directory");
    let missing = output.with_file_name("missing").join("tiling.png");
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "--size",
        "8x8",
        "--samples",
        "1",
        "-o",
        path_text(&missing),
    ]));
}

#[test]
fn render_diagnostic_names_the_positions_of_a_parameter_set_twice() {
    insta::assert_snapshot!(render_snapshot(
        "repeated_setting",
        &["*2222", "--set", "aspect=2", "--set", "aspect=3"]
    ));
}

#[test]
fn render_diagnostic_gives_the_cause_of_a_skew_outside_its_range() {
    insta::assert_snapshot!(render_snapshot("skew_range", &["o", "--set", "skew=0.6"]));
}

#[test]
fn render_diagnostic_gives_the_cause_of_a_skew_on_the_annulus() {
    insta::assert_snapshot!(render_snapshot(
        "annulus_skew",
        &["**", "--set", "skew=0.1"]
    ));
}

#[test]
fn render_diagnostic_says_parity_does_not_exist_for_the_torus() {
    insta::assert_snapshot!(render_snapshot(
        "torus_parity",
        &["o", "--colouring", "parity"]
    ));
}

#[test]
fn render_diagnostic_gives_the_cause_of_an_unknown_parameter() {
    insta::assert_snapshot!(render_snapshot(
        "unknown_parameter",
        &["*632", "--set", "aspect=2"]
    ));
}

#[test]
fn render_diagnostic_gives_the_cause_of_a_parameter_outside_its_range() {
    insta::assert_snapshot!(render_snapshot(
        "parameter_range",
        &["*2222", "--set", "aspect=9"]
    ));
}

#[test]
fn render_diagnostic_names_a_rotation_angle_that_is_not_finite() {
    insta::assert_snapshot!(render_snapshot(
        "rotation_nan",
        &["*532", "--rotation", "nan,0,0"]
    ));
}

#[test]
fn render_diagnostic_gives_the_cause_of_a_zero_light_direction() {
    insta::assert_snapshot!(render_snapshot("light_zero", &["*532", "--light", "0,0,0"]));
}

#[test]
fn render_diagnostic_gives_the_cause_of_an_infinite_light_direction() {
    insta::assert_snapshot!(render_snapshot(
        "light_infinite",
        &["*532", "--light", "inf,0,0"]
    ));
}

#[test]
fn render_diagnostic_gives_the_cause_of_a_parameter_of_a_spherical_signature() {
    insta::assert_snapshot!(render_snapshot(
        "spherical_parameter",
        &["*532", "--set", "aspect=2"]
    ));
}

const REFUSED_SHAPE: &[&str] = &[
    "*22222222",
    "--set",
    "cornerpair1.length=3.5",
    "--set",
    "cornerpair2.length=0.5",
    "--set",
    "cornerpair3.length=3.5",
    "--set",
    "cornerpair4.length=0.5",
    "--set",
    "slice1.length=0.5",
];

#[test]
fn render_diagnostic_gives_the_cause_of_a_polygon_parameter_outside_its_range() {
    insta::assert_snapshot!(render_snapshot(
        "polygon_parameter_range",
        &["*3333", "--set", "slice1.length=4"]
    ));
}

#[test]
fn render_diagnostic_gives_the_cause_of_a_parameter_of_a_doubled_polygon() {
    insta::assert_snapshot!(render_snapshot(
        "doubled_polygon_parameter",
        &["3333", "--set", "slice1.length=2"]
    ));
}

#[test]
fn render_diagnostic_says_a_polygon_shape_is_too_extreme() {
    insta::assert_snapshot!(render_snapshot("refused_shape", REFUSED_SHAPE));
}

#[test]
fn render_diagnostic_says_the_pairings_of_a_double_cannot_be_stored() {
    insta::assert_snapshot!(render_snapshot(
        "refused_double",
        &["(967)(967)(967)(967)(967)(967)(967)(967)"]
    ));
}

#[test]
fn render_diagnostic_gives_the_cause_of_more_than_eight_orders() {
    insta::assert_snapshot!(render_snapshot("nine_orders", &["222222222"]));
}

#[test]
fn render_statistics_hold_the_samples_and_masked_values() {
    insta::assert_snapshot!(render_snapshot(
        "statistics",
        &["*732", "--size", "16x16", "--samples", "1", "--stats"]
    ));
}

#[test]
fn render_statistics_of_a_spherical_tiling_hold_the_samples_and_masked_values() {
    insta::assert_snapshot!(render_snapshot(
        "statistics_spherical",
        &["*532", "--size", "16x16", "--samples", "1", "--stats"]
    ));
}

fn read_u32(bytes: &[u8], start: usize) -> u32 {
    let field = bytes
        .get(start..start + 4)
        .expect("the file holds the field");
    u32::from_be_bytes(field.try_into().expect("the field has four bytes"))
}

fn chunk_types(file: &[u8]) -> Vec<[u8; 4]> {
    let mut types = Vec::new();
    let mut start = PNG_SIGNATURE.len();
    while start < file.len() {
        let length = usize::try_from(read_u32(file, start)).expect("a chunk length fits a usize");
        let chunk_type = file
            .get(start + 4..start + 8)
            .expect("the file holds the chunk type");
        types.push(chunk_type.try_into().expect("a chunk type has four bytes"));
        start += 12 + length;
    }
    types
}

#[test]
fn render_writes_a_png_file_of_the_requested_size() {
    let output = output_path("written_file");
    let outcome = run_args(&[
        "render",
        "*732",
        "--size",
        "48x32",
        "--samples",
        "2",
        "-o",
        path_text(&output),
    ]);
    assert_eq!(outcome.status, Status::Success);
    assert_eq!(outcome.stdout, "");
    assert_eq!(outcome.stderr, "");
    assert_png_header(&output, 48, 32);
}

fn assert_png_header(output: &Path, width: u32, height: u32) {
    let file = std::fs::read(output).expect("the output file is written");
    assert_png_bytes(&file, width, height);
}

fn assert_png_bytes(file: &[u8], width: u32, height: u32) {
    assert_eq!(file.get(..8), Some(PNG_SIGNATURE));
    assert_eq!(read_u32(file, 8), 13);
    assert_eq!(file.get(12..16), Some(&b"IHDR"[..]));
    assert_eq!(read_u32(file, 16), width);
    assert_eq!(read_u32(file, 20), height);
    assert_eq!(file.get(24), Some(&8));
    assert_eq!(file.get(25), Some(&2));
    let types = chunk_types(file);
    let srgb = types.iter().position(|chunk| chunk == b"sRGB");
    let idat = types.iter().position(|chunk| chunk == b"IDAT");
    assert!(
        matches!((srgb, idat), (Some(srgb), Some(idat)) if srgb < idat),
        "{types:?}"
    );
}

#[test]
fn render_writes_a_png_file_of_a_euclidean_tiling() {
    let output = output_path("euclidean_file");
    let outcome = run_args(&[
        "render",
        "*442",
        "--size",
        "48x32",
        "--samples",
        "2",
        "-o",
        path_text(&output),
    ]);
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    assert_png_header(&output, 48, 32);
}

#[test]
fn render_writes_a_png_file_of_a_skewed_torus() {
    let output = output_path("skewed_torus_file");
    let outcome = run_args(&[
        "render",
        "o",
        "--set",
        "skew=0.25",
        "--size",
        "48x32",
        "--samples",
        "2",
        "-o",
        path_text(&output),
    ]);
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    assert_png_header(&output, 48, 32);
}

fn small_png(test: &str, args: &[&str]) -> Vec<u8> {
    let mut all = args.to_vec();
    all.extend_from_slice(&["--size", "48x32", "--samples", "2"]);
    let file = rendered(test, &all);
    assert_png_bytes(&file, 48, 32);
    file
}

#[test]
fn render_writes_a_png_file_of_a_polygon() {
    small_png("polygon_file", &["*22222"]);
}

#[test]
fn render_writes_a_png_file_of_a_doubled_polygon() {
    small_png("double_file", &["3333"]);
}

#[test]
fn render_fills_a_polygon_by_the_cosets_of_a_colouring_id() {
    let cosets = small_png("polygon_colouring_file", &["*3333", "--colouring", "3a"]);
    let default = small_png("polygon_default_file", &["*3333"]);
    let one = small_png("polygon_one_file", &["*3333", "--colouring", "one"]);
    assert_ne!(cosets, default);
    assert_ne!(cosets, one);
}

#[test]
fn render_writes_the_annulus_by_parity_by_default() {
    let default = small_png("annulus_file", &["**"]);
    let one = small_png("annulus_one_file", &["**", "--colouring", "one"]);
    assert_ne!(default, one);
}

#[test]
fn render_writes_the_klein_bottle_by_parity_by_default() {
    let default = small_png("klein_bottle_file", &["xx"]);
    let one = small_png("klein_bottle_one_file", &["xx", "--colouring", "one"]);
    assert_ne!(default, one);
}

#[test]
fn render_writes_a_png_file_of_two_half_turns_and_a_mirror() {
    small_png("two_two_mirror_file", &["22*"]);
}

#[test]
fn render_writes_a_png_file_of_a_half_turn_and_two_mirror_corners() {
    small_png("two_mirror_two_two_file", &["2*22"]);
}

#[test]
fn render_writes_a_png_file_of_the_kite_with_a_quarter_turn() {
    small_png("four_mirror_two_file", &["4*2"]);
}

#[test]
fn render_writes_a_png_file_of_the_kite_with_a_third_of_a_turn() {
    small_png("three_mirror_three_file", &["3*3"]);
}

#[test]
fn render_draws_each_kite_by_parity_and_with_one_colour_as_its_rotation_signature() {
    for (kite, rotation, name) in [("4*2", "442", "quarter"), ("3*3", "632", "third")] {
        let default = small_png(&format!("{name}_kite_default_file"), &[kite]);
        let one = small_png(
            &format!("{name}_kite_one_file"),
            &[kite, "--colouring", "one"],
        );
        let doubled = small_png(&format!("{name}_double_file"), &[rotation]);
        assert_ne!(default, doubled, "{kite}");
        assert_eq!(one, doubled, "{kite}");
    }
}

#[test]
fn render_draws_the_same_default_picture_for_groups_that_share_their_tiles() {
    for (index, group) in [&["*x", "22x", "*2222"][..], &["**", "22*"][..]]
        .into_iter()
        .enumerate()
    {
        let files: Vec<Vec<u8>> = group
            .iter()
            .enumerate()
            .map(|(position, signature)| {
                small_png(
                    &format!("shared_default_{index}_{position}_file"),
                    &[signature],
                )
            })
            .collect();
        for (signature, file) in group.iter().zip(&files).skip(1) {
            assert_eq!(*file, files[0], "{signature} against {}", group[0]);
        }
    }
}

#[test]
fn render_applies_the_skew_of_2222_and_leaves_its_default_at_a_skew_of_zero() {
    let default = square_image("2222", "sheared_default", &[]);
    let skewed = square_image("2222", "sheared_skew", &["--set", "skew=0.25"]);
    let zero = square_image("2222", "sheared_zero", &["--set", "skew=0"]);
    assert_png_bytes(&skewed, 64, 64);
    assert_ne!(skewed, default);
    assert_eq!(zero, default);
    for threads in ["1", "4"] {
        let threaded = square_image(
            "2222",
            &format!("sheared_threads_{threads}"),
            &["--set", "skew=0.25", "--threads", threads],
        );
        assert_eq!(threaded, skewed, "{threads} threads");
    }
}

fn square_image(signature: &str, test: &str, extra: &[&str]) -> Vec<u8> {
    let mut args = vec![signature, "--size", "64x64", "--samples", "2"];
    args.extend_from_slice(extra);
    rendered(test, &args)
}

#[test]
fn render_applies_a_polygon_setting_and_writes_the_same_bytes_for_one_thread_and_four() {
    let default = square_image("*3333", "polygon_default", &[]);
    let one = square_image(
        "*3333",
        "slice_one_thread",
        &["--set", "slice1.length=2.5", "--threads", "1"],
    );
    let four = square_image(
        "*3333",
        "slice_four_threads",
        &["--set", "slice1.length=2.5", "--threads", "4"],
    );
    assert_ne!(one, default);
    assert_eq!(one, four);
}

fn rectangle(test: &str, extra: &[&str]) -> Vec<u8> {
    let output = output_path(test);
    let mut args = vec!["render", "*2222", "--size", "64x64", "--samples", "2"];
    args.extend_from_slice(extra);
    args.extend_from_slice(&["-o", path_text(&output)]);
    let outcome = run_args(&args);
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    std::fs::read(&output).expect("the output file is written")
}

#[test]
fn render_applies_a_setting_and_writes_the_same_bytes_for_one_thread_and_four() {
    let square = rectangle("square", &[]);
    let one = rectangle(
        "aspect_one_thread",
        &["--set", "aspect=1.618", "--threads", "1"],
    );
    let four = rectangle(
        "aspect_four_threads",
        &["--set", "aspect=1.618", "--threads", "4"],
    );
    assert_ne!(one, square);
    assert_eq!(one, four);
}

fn rendered(test: &str, args: &[&str]) -> Vec<u8> {
    let output = output_path(test);
    let mut all = vec!["render"];
    all.extend_from_slice(args);
    all.extend_from_slice(&["-o", path_text(&output)]);
    let outcome = run_args(&all);
    assert_eq!(
        outcome.status,
        Status::Success,
        "{args:?}: {}",
        outcome.stderr
    );
    std::fs::read(&output).expect("the output file is written")
}

fn rendered_with_threads(test: &str, threads: &str) -> Vec<u8> {
    rendered(
        test,
        &[
            "*732",
            "--size",
            "64x64",
            "--samples",
            "2",
            "--threads",
            threads,
        ],
    )
}

#[test]
fn render_writes_the_same_bytes_for_one_thread_and_four() {
    let signature: Signature = "*732".parse().expect("the signature is valid");
    let domain = FundamentalDomain::build(&signature).expect("the domain is built");
    let fill = Fill::Parity([
        Colour::from_srgb8(0xe9, 0xc4, 0x6a),
        Colour::from_srgb8(0x2a, 0x9d, 0x8f),
    ]);
    let options = RenderOptions::builder(fill)
        .size(64, 64)
        .samples(2)
        .build()
        .expect("the options are valid");
    let (pixels, _) = Renderer::new(&domain, options)
        .expect("the renderer is built")
        .render();
    let whole_image = encode_png(64, 64, &pixels).expect("the image is encoded");
    assert_eq!(rendered_with_threads("one_thread", "1"), whole_image);
    assert_eq!(rendered_with_threads("four_threads", "4"), whole_image);
}

fn rendered_colouring_with_threads(test: &str, threads: &str) -> Vec<u8> {
    rendered(
        test,
        &[
            "*642",
            "--colouring",
            "3a",
            "--size",
            "64x64",
            "--samples",
            "2",
            "--threads",
            threads,
        ],
    )
}

#[test]
fn render_writes_the_same_bytes_of_a_colouring_for_one_thread_and_four() {
    let signature: Signature = "*642".parse().expect("the signature is valid");
    let domain = FundamentalDomain::build(&signature).expect("the domain is built");
    let presentation = domain
        .side_presentation()
        .expect("the domain has a side presentation");
    let found = colourings_of_index(
        &presentation,
        IndexLimit::new(3).expect("the limit is valid"),
        SearchBudget::DEFAULT,
    )
    .expect("the search runs");
    let colouring = found
        .find("3a".parse().expect("the id is valid"))
        .expect("the colouring 3a exists")
        .clone();
    let colours = vec![
        Colour::from_srgb8(0xe9, 0xc4, 0x6a),
        Colour::from_srgb8(0x2a, 0x9d, 0x8f),
        Colour::from_srgb8(0x91, 0x36, 0x71),
    ];
    let fill =
        Fill::Cosets(CosetFill::new(&domain, colouring, colours).expect("the fill is valid"));
    let options = RenderOptions::builder(fill)
        .size(64, 64)
        .samples(2)
        .build()
        .expect("the options are valid");
    let (pixels, _) = Renderer::new(&domain, options)
        .expect("the renderer is built")
        .render();
    let whole_image = encode_png(64, 64, &pixels).expect("the image is encoded");
    assert_eq!(
        rendered_colouring_with_threads("colouring_one_thread", "1"),
        whole_image
    );
    assert_eq!(
        rendered_colouring_with_threads("colouring_four_threads", "4"),
        whole_image
    );
}

#[test]
fn render_leaves_the_global_thread_pool_unused() {
    let output = output_path("global_pool");
    assert_eq!(small_render(&output), Status::Success);
    assert!(
        rayon::ThreadPoolBuilder::new().build_global().is_ok(),
        "the global thread pool was started"
    );
}

fn small_render(output: &Path) -> Status {
    run_args(&[
        "render",
        "*732",
        "--size",
        "8x8",
        "--samples",
        "1",
        "-o",
        path_text(output),
    ])
    .status
}

#[test]
fn render_replaces_an_existing_file() {
    let output = output_path("existing_file");
    std::fs::write(&output, b"old").expect("the existing file is written");
    assert_eq!(small_render(&output), Status::Success);
    let file = std::fs::read(&output).expect("the output file is written");
    assert!(file.starts_with(PNG_SIGNATURE));
}

#[test]
fn render_writes_through_a_symbolic_link() {
    let link = output_path("symbolic_link");
    let target = link.with_file_name("target.png");
    std::fs::write(&target, b"old").expect("the target file is written");
    std::os::unix::fs::symlink(&target, &link).expect("the link is created");
    assert_eq!(small_render(&link), Status::Success);
    let metadata = std::fs::symlink_metadata(&link).expect("the link exists");
    assert!(metadata.file_type().is_symlink());
    let file = std::fs::read(&target).expect("the target file is written");
    assert!(file.starts_with(PNG_SIGNATURE));
}

fn assert_no_file_changes(test: &str, args: &[&str]) {
    let existing = output_path(test);
    let missing = existing.with_file_name("missing.png");
    std::fs::write(&existing, b"old").expect("the existing file is written");
    for output in [&existing, &missing] {
        let mut all = vec!["render"];
        all.extend_from_slice(args);
        all.extend_from_slice(&["-o", path_text(output)]);
        assert_eq!(run_args(&all).status, Status::Failure);
    }
    assert_eq!(
        std::fs::read(&existing).expect("the existing file stays"),
        b"old"
    );
    assert!(!missing.exists());
}

#[test]
fn render_without_a_domain_changes_no_file() {
    assert_no_file_changes("no_domain_file", &["3*2"]);
}

#[test]
fn render_with_parity_for_a_domain_without_a_reversing_side_changes_no_file() {
    assert_no_file_changes("parity_rotation_file", &["732", "--colouring", "parity"]);
}

#[test]
fn render_of_the_torus_with_a_skew_outside_its_range_or_parity_changes_no_file() {
    assert_no_file_changes("torus_skew_file", &["o", "--set", "skew=0.6"]);
    assert_no_file_changes("torus_parity_file", &["o", "--colouring", "parity"]);
}

#[test]
fn render_with_a_parameter_outside_its_range_changes_no_file() {
    assert_no_file_changes("parameter_range_file", &["*2222", "--set", "aspect=9"]);
}

#[test]
fn render_with_a_polygon_parameter_outside_its_range_changes_no_file() {
    assert_no_file_changes(
        "polygon_parameter_range_file",
        &["*3333", "--set", "slice1.length=4"],
    );
}

#[test]
fn render_with_a_refused_polygon_shape_changes_no_file() {
    assert_no_file_changes("refused_shape_file", REFUSED_SHAPE);
}

#[test]
fn render_with_an_empty_size_changes_no_file() {
    assert_no_file_changes("empty_size_file", &["*732", "--size", "0x0"]);
}

#[test]
fn render_with_a_bad_colour_changes_no_file() {
    assert_no_file_changes("bad_colour_file", &["*732", "--colours", "#e9c46a,red"]);
}

#[test]
fn render_with_a_rotation_that_is_not_finite_changes_no_file() {
    assert_no_file_changes("rotation_nan_file", &["*532", "--rotation", "nan,0,0"]);
}

#[test]
fn render_with_a_zero_light_direction_changes_no_file() {
    assert_no_file_changes("light_zero_file", &["*532", "--light", "0,0,0"]);
}

fn statistics_to(test: &str, stdout: &mut FailingWriter) -> (Status, Vec<u8>, Vec<u8>) {
    let output = output_path(test);
    let mut stderr = Vec::new();
    let status = run(
        [
            "escher",
            "render",
            "*732",
            "--size",
            "8x8",
            "--samples",
            "1",
            "--stats",
            "-o",
            path_text(&output),
        ],
        stdout,
        &mut stderr,
    );
    let file = std::fs::read(&output).expect("the output file is written");
    (status, stderr, file)
}

#[test]
fn render_statistics_to_a_broken_pipe_fail_without_a_diagnostic() {
    let mut stdout = FailingWriter::failing_write(io::ErrorKind::BrokenPipe);
    let (status, stderr, file) = statistics_to("broken_pipe", &mut stdout);
    assert_eq!(status, Status::Failure);
    assert_eq!(stderr, b"");
    assert!(file.starts_with(PNG_SIGNATURE));
}

#[test]
fn render_statistics_to_a_failing_writer_give_a_diagnostic() {
    let mut stdout = FailingWriter::failing_write(io::ErrorKind::Other);
    let (status, stderr, file) = statistics_to("failing_writer", &mut stdout);
    assert_eq!(status, Status::Failure);
    assert_eq!(
        String::from_utf8(stderr).expect("standard error is UTF-8"),
        "error: cannot write to standard output\n  caused by: other error\n"
    );
    assert!(file.starts_with(PNG_SIGNATURE));
}

fn ball(test: &str, extra: &[&str]) -> Vec<u8> {
    let output = output_path(test);
    let mut args = vec!["render", "*532", "--size", "64x64", "--samples", "2"];
    args.extend_from_slice(extra);
    args.extend_from_slice(&["-o", path_text(&output)]);
    let outcome = run_args(&args);
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    assert_eq!(outcome.stdout, "");
    assert_eq!(outcome.stderr, "");
    std::fs::read(&output).expect("the output file is written")
}

#[test]
fn render_writes_a_png_file_of_a_spherical_tiling() {
    let output = output_path("spherical_file");
    let outcome = run_args(&[
        "render",
        "*532",
        "--size",
        "48x32",
        "--samples",
        "2",
        "-o",
        path_text(&output),
    ]);
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    assert_png_header(&output, 48, 32);
}

#[test]
fn render_writes_the_same_bytes_of_a_ball_for_one_thread_and_four() {
    let one = ball("ball_one_thread", &["--threads", "1"]);
    let four = ball("ball_four_threads", &["--threads", "4"]);
    assert_eq!(one, four);
}

#[test]
fn render_accepts_a_value_that_starts_with_a_hyphen_for_the_rotation_and_the_light() {
    let output = output_path("hyphen_values");
    let rotation = run_args(&[
        "render",
        "*532",
        "--size",
        "8x8",
        "--samples",
        "1",
        "--rotation",
        "-20,35,0",
        "-o",
        path_text(&output),
    ]);
    assert_eq!(rotation.status, Status::Success, "{}", rotation.stderr);
    let light = run_args(&[
        "render",
        "*532",
        "--size",
        "8x8",
        "--samples",
        "1",
        "--light=-1,1,2",
        "-o",
        path_text(&output),
    ]);
    assert_eq!(light.status, Status::Success, "{}", light.stderr);
}

#[test]
fn render_turns_the_ball_with_the_rotation() {
    let turned = ball("ball_turned", &["--rotation", "20,35,0"]);
    let default = ball("ball_default", &[]);
    assert_ne!(turned, default);
}

fn small_file(test: &str, args: &[&str]) -> Vec<u8> {
    let output = output_path(test);
    let mut all = vec!["render"];
    all.extend_from_slice(args);
    all.extend_from_slice(&[
        "--size",
        "32x32",
        "--samples",
        "1",
        "-o",
        path_text(&output),
    ]);
    let outcome = run_args(&all);
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    std::fs::read(&output).expect("the output file is written")
}

#[test]
fn render_colours_a_rotation_domain_with_one_colour_by_default() {
    assert_eq!(
        small_file("default_rotation", &["732"]),
        small_file("one_rotation", &["732", "--colouring", "one"])
    );
}

#[test]
fn render_colours_a_reflection_domain_by_parity_by_default() {
    assert_eq!(
        small_file("default_reflection", &["*732"]),
        small_file("parity_reflection", &["*732", "--colouring", "parity"])
    );
}

#[test]
fn render_writes_a_png_file_of_each_geometry_of_a_rotation_domain() {
    for (test, args) in [
        ("rotation_euclidean", &["442"][..]),
        ("rotation_rectangle", &["2222", "--set", "aspect=1.618"][..]),
        ("rotation_spherical", &["532"][..]),
    ] {
        let output = output_path(test);
        let mut all = vec!["render"];
        all.extend_from_slice(args);
        all.extend_from_slice(&[
            "--size",
            "48x32",
            "--samples",
            "2",
            "-o",
            path_text(&output),
        ]);
        let outcome = run_args(&all);
        assert_eq!(
            outcome.status,
            Status::Success,
            "{test}: {}",
            outcome.stderr
        );
        assert_png_header(&output, 48, 32);
    }
}

fn alias_file(test: &str, signature: &str, colouring: &str) -> Vec<u8> {
    rendered(
        test,
        &[
            signature,
            "--colouring",
            colouring,
            "--size",
            "48x32",
            "--samples",
            "2",
        ],
    )
}

fn parity_id(signature: &str) -> String {
    let outcome = run_args(&["info", "--colourings", "--max-colours", "2", signature]);
    assert_eq!(outcome.status, Status::Success, "{}", outcome.stderr);
    let ids: Vec<&str> = outcome
        .stdout
        .lines()
        .filter_map(|line| line.strip_prefix("Colouring: "))
        .filter_map(|value| value.strip_suffix(", 2 colours, alias parity"))
        .collect();
    let [id] = ids.as_slice() else {
        panic!("{signature} has one parity colouring: {}", outcome.stdout);
    };
    (*id).to_owned()
}

#[test]
fn render_draws_the_trivial_colouring_as_one() {
    for (test, signature) in [("trivial_reflection", "*732"), ("trivial_rotation", "732")] {
        assert_eq!(
            alias_file(&format!("{test}_id"), signature, "1a"),
            alias_file(&format!("{test}_alias"), signature, "one"),
            "{signature}"
        );
    }
}

#[test]
fn render_draws_the_parity_colouring_as_parity() {
    for (test, signature) in [
        ("parity_hyperbolic", "*732"),
        ("parity_kite", "*642"),
        ("parity_rectangle", "*2222"),
        ("parity_sphere", "*532"),
    ] {
        let id = parity_id(signature);
        assert_eq!(
            alias_file(&format!("{test}_id"), signature, &id),
            alias_file(&format!("{test}_alias"), signature, "parity"),
            "{signature} {id}"
        );
    }
}

fn first_error_line(args: &[&str]) -> (Status, String) {
    let never_written = Path::new(NEVER_WRITTEN);
    match std::fs::remove_file(never_written) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => {
            panic!("cannot remove a stale output file: {error}")
        }
        _ => {}
    }
    let mut all = vec!["render"];
    all.extend_from_slice(args);
    all.extend_from_slice(&["-o", NEVER_WRITTEN]);
    let outcome = run_args(&all);
    assert!(!never_written.exists(), "the output file was written");
    assert_eq!(outcome.stdout, "");
    let first = outcome.stderr.lines().next().unwrap_or_default().to_owned();
    (outcome.status, first)
}

#[test]
fn render_without_a_domain_returns_failure() {
    assert_eq!(
        first_error_line(&["3*2"]),
        (
            Status::Failure,
            "error: cannot render 3*2: rendering this signature is not supported".to_owned()
        )
    );
}

#[test]
fn render_with_parity_for_a_domain_without_a_reversing_side_returns_failure() {
    assert_eq!(
        first_error_line(&["732", "--colouring", "parity"]),
        (
            Status::Failure,
            "error: cannot render 732: the colouring parity does not exist, because no side pairing reverses orientation"
                .to_owned()
        )
    );
}

#[test]
fn render_with_parity_for_the_torus_returns_failure() {
    assert_eq!(
        first_error_line(&["o", "--colouring", "parity"]),
        (
            Status::Failure,
            "error: cannot render o: the colouring parity does not exist, because no side pairing reverses orientation"
                .to_owned()
        )
    );
}

#[test]
fn render_with_a_skew_outside_its_range_returns_failure() {
    assert_eq!(
        first_error_line(&["o", "--set", "skew=0.6"]),
        (Status::Failure, "error: cannot render o".to_owned())
    );
}

#[test]
fn render_with_two_colours_for_one_returns_failure() {
    assert_eq!(
        first_error_line(&["*732", "--colouring", "one", "--colours", "#e9c46a,#2a9d8f"]),
        (
            Status::Failure,
            "error: cannot render *732: the colouring one needs 1 colour, and 2 were given"
                .to_owned()
        )
    );
}

#[test]
fn render_with_a_colour_name_returns_failure() {
    assert_eq!(
        first_error_line(&["*732", "--colours", "#e9c46a,red"]),
        (
            Status::Failure,
            "error: cannot render *732: colour 2 of --colours is not of the form #rgb or #rrggbb"
                .to_owned()
        )
    );
}

#[test]
fn render_with_a_missing_colouring_id_returns_failure() {
    assert_eq!(
        first_error_line(&["22", "--colouring", "2b"]),
        (
            Status::Failure,
            "error: cannot render 22: the colouring 2b does not exist, because the signature has 1 colouring with 2 colours"
                .to_owned()
        )
    );
}

#[test]
fn render_with_one_colour_for_an_id_with_two_returns_failure() {
    assert_eq!(
        first_error_line(&["22", "--colouring", "2a", "--colours", "#e9c46a"]),
        (
            Status::Failure,
            "error: cannot render 22: the colouring 2a needs 2 colours, and 1 was given".to_owned()
        )
    );
}

#[test]
fn render_with_a_malformed_colouring_id_returns_usage() {
    assert_eq!(
        first_error_line(&["*732", "--colouring", "3A"]),
        (
            Status::Usage,
            "error: invalid value '3A' for '--colouring <COLOURING>': expected one, parity, or a colouring id such as 3a"
                .to_owned()
        )
    );
}

#[test]
fn render_with_a_colouring_id_reports_a_bad_colour_before_the_search() {
    assert_eq!(
        first_error_line(&["*666", "--colouring", "12a", "--colours", "oops"]),
        (
            Status::Failure,
            "error: cannot render *666: the colouring 12a needs 12 colours, and 1 was given"
                .to_owned()
        )
    );
}

#[test]
fn render_with_a_colouring_id_reports_more_than_twelve_colours_before_the_search() {
    assert_eq!(
        first_error_line(&["*666", "--colouring", "13a"]),
        (
            Status::Failure,
            "error: cannot render *666: the colouring 13a needs 13 colours, more than the 12 default colours, so give 13 colours with --colours"
                .to_owned()
        )
    );
}

#[test]
fn render_with_a_colouring_id_reports_an_empty_size_before_the_search() {
    let outcome = run_args(&[
        "render",
        "*666",
        "--colouring",
        "12a",
        "--size",
        "0x0",
        "-o",
        NEVER_WRITTEN,
    ]);
    assert_eq!(outcome.status, Status::Failure);
    assert_eq!(
        outcome.stderr,
        "error: cannot render *666\n  caused by: cannot build render options: the image size 0x0 is empty\n"
    );
}

#[test]
fn render_with_an_empty_size_returns_failure() {
    assert_eq!(
        first_error_line(&["*732", "--size", "0x0"]),
        (Status::Failure, "error: cannot render *732".to_owned())
    );
}

#[test]
fn render_without_an_output_returns_usage() {
    let outcome = run_args(&["render", "*732"]);
    assert_eq!(outcome.status, Status::Usage);
    assert_eq!(outcome.stdout, "");
    assert!(
        outcome
            .stderr
            .starts_with("error: the following required arguments were not provided:\n"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn render_reports_a_broken_pipe_on_the_output_file() {
    let fifo = output_path("output_fifo");
    let created = std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("mkfifo starts");
    assert!(created.success());
    let reader_path = fifo.clone();
    let reader = std::thread::spawn(move || {
        drop(std::fs::File::open(reader_path).expect("the reader opens the pipe"));
    });
    let outcome = run_args(&[
        "render",
        "*732",
        "--size",
        "1024x1024",
        "--samples",
        "1",
        "-o",
        path_text(&fifo),
    ]);
    reader.join().expect("the reader finishes");
    assert_eq!(outcome.status, Status::Failure);
    assert_eq!(outcome.stdout, "");
    assert_eq!(
        outcome.stderr,
        "error: cannot write the output file\n  caused by: Broken pipe (os error 32)\n"
    );
}
