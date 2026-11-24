use std::io;
use std::path::{Path, PathBuf};

use escher_cli::{Status, run};
use escher_domain::FundamentalDomain;
use escher_group::{ColouringSet, IndexLimit, SearchBudget, colourings_of_index};
use escher_render::{
    Colour, CosetFill, Fill, HyperbolicModel, Paint, RenderOptions, RenderOptionsBuilder, Renderer,
    WholeFaceFill, Wythoff, WythoffFill, encode_png,
};
use escher_signature::Signature;

use crate::common::{
    FailingWriter, NEVER_WRITTEN, PNG_SIGNATURE, assert_png_bytes, failure_with_no_file_change,
    failure_with_no_mask_change, output_path, path_text, regular_arrangement, run_args, snapshot,
    write_image,
};

fn render_snapshot(test: &str, args: &[&str]) -> String {
    snapshot_and_output(test, args).0
}

fn snapshot_and_output(test: &str, args: &[&str]) -> (String, PathBuf) {
    let output = output_path(test);
    let mut all = vec!["render"];
    all.extend_from_slice(args);
    all.extend_from_slice(&["-o", path_text(&output)]);
    (snapshot(&all), output)
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
    insta::assert_snapshot!(render_snapshot("order_limit", &["*(101)32"]));
}

#[test]
fn render_diagnostic_says_parity_does_not_exist_without_a_valid_colouring_with_two_colours() {
    insta::assert_snapshot!(render_snapshot(
        "parity_without_valid_colouring",
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
        &["*642", "--colouring", "3a"]
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
        &["*(12)(12)6", "--colouring", "12a"]
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
fn render_diagnostic_gives_the_cause_of_an_intrinsic_border_width_over_the_limit() {
    insta::assert_snapshot!(render_snapshot(
        "intrinsic_border_limit",
        &[
            "*732",
            "--border-unit",
            "intrinsic",
            "--border-width",
            "1025"
        ]
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
fn render_diagnostic_gives_the_cause_of_a_two_mirror_order_over_its_limit() {
    insta::assert_snapshot!(render_snapshot("two_mirror_limit", &["2*(51)"]));
}

#[test]
fn render_diagnostic_gives_the_cause_of_parity_on_the_whole_sphere() {
    insta::assert_snapshot!(render_snapshot(
        "whole_sphere_parity",
        &["1", "--colouring", "parity"]
    ));
}

#[test]
fn render_diagnostic_gives_the_cause_of_a_setting_on_a_rotary_lune() {
    insta::assert_snapshot!(render_snapshot(
        "rotary_lune_setting",
        &["5x", "--set", "aspect=2"]
    ));
}

#[test]
fn render_diagnostic_gives_the_cause_of_a_setting_on_a_spherical_kite() {
    insta::assert_snapshot!(render_snapshot(
        "spherical_kite_setting",
        &["3*2", "--set", "aspect=2"]
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
fn render_diagnostic_names_an_unknown_setting_and_the_parameter_of_the_signature() {
    insta::assert_snapshot!(render_snapshot(
        "unknown_setting",
        &["*2222", "--set", "foo=1"]
    ));
}

#[test]
fn render_diagnostic_names_a_parameter_of_another_signature() {
    insta::assert_snapshot!(render_snapshot(
        "skew_on_rectangle",
        &["*2222", "--set", "skew=0.1"]
    ));
}

#[test]
fn render_diagnostic_names_the_two_parameters_of_the_torus() {
    insta::assert_snapshot!(render_snapshot(
        "unknown_setting_on_torus",
        &["o", "--set", "skew=0.1", "--set", "Aspect=2"]
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
fn render_draws_each_kite_with_one_colour_as_its_rotation_signature() {
    for (kite, rotation, name) in [("4*2", "442", "quarter"), ("3*3", "632", "third")] {
        let one = small_png(
            &format!("{name}_kite_one_file"),
            &[kite, "--colouring", "one"],
        );
        let doubled = small_png(
            &format!("{name}_double_file"),
            &[rotation, "--colouring", "one"],
        );
        assert_eq!(one, doubled, "{kite}");
    }
    let quarter = small_png("quarter_kite_default_file", &["4*2"]);
    let quarter_parity = small_png("quarter_kite_parity_file", &["4*2", "--colouring", "2a"]);
    assert_eq!(quarter, quarter_parity);
    let third = small_png("third_kite_default_file", &["3*3"]);
    let third_one = small_png(
        "third_kite_default_one_file",
        &["3*3", "--colouring", "one"],
    );
    assert_eq!(third, third_one);
}

#[test]
fn render_writes_a_png_file_of_a_spherical_kite_with_mirrors() {
    small_png("three_mirror_two_file", &["3*2"]);
}

#[test]
fn render_writes_a_png_file_of_a_lune_paired_by_rotary_reflections() {
    small_png("rotary_lune_file", &["5x"]);
}

#[test]
fn render_writes_a_png_file_of_the_whole_sphere() {
    small_png("whole_sphere_file", &["1"]);
}

#[test]
fn render_draws_the_lune_groups_by_parity_as_their_reflection_groups() {
    for (group, reflection, name) in [("5x", "*55", "rotary_lune"), ("x", "*", "cross_cap")] {
        let default = small_png(&format!("{name}_default_file"), &[group]);
        let mirrored = small_png(&format!("{name}_mirrored_file"), &[reflection]);
        assert_eq!(default, mirrored, "{group} and {reflection}");
    }
}

#[test]
fn render_of_a_rotary_lune_does_not_depend_on_the_number_of_threads() {
    let files: Vec<Vec<u8>> = ["1", "4"]
        .into_iter()
        .map(|threads| {
            square_image(
                "5x",
                &format!("rotary_lune_threads_{threads}"),
                &["--threads", threads],
            )
        })
        .collect();
    assert_png_bytes(&files[0], 64, 64);
    assert_eq!(files[0], files[1]);
}

#[test]
fn render_draws_each_spherical_kite_with_one_colour_as_its_double() {
    for (kite, double, name) in [
        ("3*2", "432", "three_mirror_two"),
        ("2*", "222", "two_cone_mirror"),
        ("3*", "322", "three_cone_mirror"),
        ("5*", "522", "five_cone_mirror"),
        ("2*2", "422", "two_mirror_two"),
        ("2*3", "622", "two_mirror_three"),
        ("2*5", "(10)22", "two_mirror_five"),
    ] {
        let one = small_png(&format!("{name}_one_file"), &[kite, "--colouring", "one"]);
        let doubled = small_png(
            &format!("{name}_double_file"),
            &[double, "--colouring", "one"],
        );
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
            "4a",
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
        ColouringSet::ValidClasses,
        IndexLimit::new(4).expect("the limit is valid"),
        SearchBudget::DEFAULT,
    )
    .expect("the search runs");
    let colouring = found
        .find("4a".parse().expect("the id is valid"))
        .expect("the colouring 4a exists")
        .clone();
    let paints = vec![
        Paint::Solid(Colour::from_srgb8(0xe9, 0xc4, 0x6a)),
        Paint::Solid(Colour::from_srgb8(0x2a, 0x9d, 0x8f)),
        Paint::Solid(Colour::from_srgb8(0x91, 0x36, 0x71)),
        Paint::Solid(Colour::from_srgb8(0x91, 0xab, 0x3b)),
    ];
    let fill = Fill::Cosets(CosetFill::new(&domain, colouring, paints).expect("the fill is valid"));
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
    failure_with_no_file_change("render", test, args);
}

#[test]
fn render_without_a_domain_changes_no_file() {
    assert_no_file_changes("no_domain_file", &["4*3"]);
}

#[test]
fn render_with_parity_for_a_domain_without_a_valid_colouring_with_two_colours_changes_no_file() {
    assert_no_file_changes("parity_rotation_file", &["732", "--colouring", "parity"]);
}

#[test]
fn render_with_parity_on_the_whole_sphere_changes_no_file() {
    assert_no_file_changes("whole_sphere_parity_file", &["1", "--colouring", "parity"]);
}

#[test]
fn render_of_a_two_mirror_order_over_its_limit_changes_no_file() {
    assert_no_file_changes("two_mirror_limit_file", &["2*(51)"]);
}

#[test]
fn render_of_the_torus_with_a_skew_outside_its_range_changes_no_file() {
    assert_no_file_changes("torus_skew_file", &["o", "--set", "skew=0.6"]);
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
        small_file("default_rotation_triangle", &["732"]),
        small_file("one_rotation_triangle", &["732", "--colouring", "one"])
    );
}

#[test]
fn render_colours_a_domain_without_a_valid_colouring_with_two_colours_with_one_colour_by_default() {
    for (test, signature) in [("whole_sphere", "1"), ("kite", "3*3"), ("sphere", "532")] {
        assert_eq!(
            small_file(&format!("default_{test}"), &[signature]),
            small_file(&format!("one_{test}"), &[signature, "--colouring", "one"]),
            "{signature}"
        );
    }
}

#[test]
fn render_colours_a_domain_that_is_not_all_reversing_by_its_valid_colouring_with_two_colours() {
    for (test, signature) in [("torus", "o"), ("rotation", "442"), ("mixed", "**")] {
        let default = small_file(&format!("default_{test}"), &[signature]);
        assert_eq!(
            default,
            small_file(&format!("id_{test}"), &[signature, "--colouring", "2a"]),
            "{signature}"
        );
        assert_ne!(
            default,
            small_file(&format!("one_{test}"), &[signature, "--colouring", "one"]),
            "{signature}"
        );
    }
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
        ("parity_torus", "o"),
        ("parity_rotation", "442"),
        ("parity_mixed", "4*2"),
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
        first_error_line(&["4*3"]),
        (
            Status::Failure,
            "error: cannot render 4*3: rendering this signature is not supported".to_owned()
        )
    );
}

#[test]
fn render_of_a_two_mirror_order_over_its_limit_returns_failure() {
    assert_eq!(
        first_error_line(&["2*(51)"]),
        (Status::Failure, "error: cannot render 2*(51)".to_owned())
    );
}

#[test]
fn render_with_parity_on_the_whole_sphere_returns_failure() {
    assert_eq!(
        first_error_line(&["1", "--colouring", "parity"]),
        (
            Status::Failure,
            "error: cannot render 1: the colouring parity does not exist, because the signature has no colouring with 2 colours"
                .to_owned()
        )
    );
}

#[test]
fn render_of_a_rotary_lune_with_a_setting_returns_failure() {
    assert_eq!(
        first_error_line(&["5x", "--set", "aspect=2"]),
        (
            Status::Failure,
            "error: cannot render 5x: the signature has no shape parameter aspect, nor any other"
                .to_owned()
        )
    );
}

#[test]
fn render_of_a_spherical_kite_with_a_setting_returns_failure() {
    assert_eq!(
        first_error_line(&["3*2", "--set", "aspect=2"]),
        (
            Status::Failure,
            "error: cannot render 3*2: the signature has no shape parameter aspect, nor any other"
                .to_owned()
        )
    );
}

#[test]
fn render_with_parity_for_a_domain_without_a_valid_colouring_with_two_colours_returns_failure() {
    assert_eq!(
        first_error_line(&["732", "--colouring", "parity"]),
        (
            Status::Failure,
            "error: cannot render 732: the colouring parity does not exist, because the signature has no colouring with 2 colours"
                .to_owned()
        )
    );
}

#[test]
fn render_with_an_id_of_a_colouring_that_is_not_valid_returns_failure() {
    assert_eq!(
        first_error_line(&["*632", "--colouring", "2b"]),
        (
            Status::Failure,
            "error: cannot render *632: the colouring 2b does not exist, because the signature has 1 colouring with 2 colours"
                .to_owned()
        )
    );
}

#[test]
fn render_with_a_skew_outside_its_range_returns_failure() {
    assert_eq!(
        first_error_line(&["o", "--set", "skew=0.6"]),
        (
            Status::Failure,
            "error: cannot render o: the value 0.6 of the parameter skew is outside its range of -0.5 to 0.5"
                .to_owned()
        )
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
            "error: cannot render *732: colour 2 of --colours is not of the form #rgb, #rrggbb, or image:FILE"
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
            "error: invalid value '3A' for '--colouring <COLOURING>': expected one, parity, or a colouring id such as 4a"
                .to_owned()
        )
    );
}

#[test]
fn render_with_a_colouring_id_reports_a_bad_colour_before_the_search() {
    assert_eq!(
        first_error_line(&["*(12)(12)6", "--colouring", "12a", "--colours", "oops"]),
        (
            Status::Failure,
            "error: cannot render *(12)(12)6: the colouring 12a needs 12 colours, and 1 was given"
                .to_owned()
        )
    );
}

#[test]
fn render_with_a_colouring_id_reports_more_than_twelve_colours_before_the_search() {
    assert_eq!(
        first_error_line(&["*(12)(12)6", "--colouring", "13a"]),
        (
            Status::Failure,
            "error: cannot render *(12)(12)6: the colouring 13a needs 13 colours, more than the 12 default colours, so give 13 colours with --colours"
                .to_owned()
        )
    );
}

#[test]
fn render_with_a_colouring_id_reports_an_empty_size_before_the_search() {
    let outcome = run_args(&[
        "render",
        "*(12)(12)6",
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
        "error: cannot render *(12)(12)6\n  caused by: cannot build render options: the image size 0x0 is empty\n"
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

#[test]
fn render_with_the_disk_model_writes_the_bytes_of_the_default() {
    let default = small_png("model_disk_default", &["*732"]);
    let disk = small_png("model_disk_given", &["*732", "--model", "disk"]);
    assert_eq!(disk, default);
}

#[test]
fn render_in_each_other_model_writes_another_image_of_a_hyperbolic_tiling() {
    let disk = small_png("model_other_disk", &["*732"]);
    for model in ["half-plane", "band", "klein"] {
        let test = format!("model_other_{model}");
        let file = small_png(&test, &["*732", "--model", model]);
        assert_ne!(file, disk, "{model}");
    }
}

#[test]
fn render_with_a_model_writes_the_bytes_of_the_default_for_a_euclidean_or_spherical_tiling() {
    for (signature, name) in [("*632", "euclidean"), ("*532", "spherical")] {
        let default = small_png(&format!("model_none_{name}_default"), &[signature]);
        let band = small_png(
            &format!("model_none_{name}_band"),
            &[signature, "--model", "band"],
        );
        assert_eq!(band, default, "{signature}");
    }
}

#[test]
fn render_with_the_pixel_border_unit_writes_the_bytes_of_the_default() {
    let default = small_png("border_unit_pixels_default", &["*732"]);
    let pixels = small_png(
        "border_unit_pixels_given",
        &["*732", "--border-unit", "pixels"],
    );
    assert_eq!(pixels, default);
}

#[test]
fn render_with_the_intrinsic_border_unit_writes_another_image_in_each_geometry() {
    for (signature, name) in [
        ("*732", "hyperbolic"),
        ("*632", "euclidean"),
        ("*532", "spherical"),
    ] {
        let default = small_png(&format!("border_unit_{name}_default"), &[signature]);
        let intrinsic = small_png(
            &format!("border_unit_{name}_intrinsic"),
            &[
                signature,
                "--border-unit",
                "intrinsic",
                "--border-width",
                "0.05",
            ],
        );
        assert_ne!(intrinsic, default, "{signature}");
    }
}

#[test]
fn render_with_the_intrinsic_border_unit_and_no_width_draws_the_width_0_05() {
    let given = small_png(
        "border_unit_width_given",
        &[
            "*732",
            "--border-unit",
            "intrinsic",
            "--border-width",
            "0.05",
        ],
    );
    let default = small_png(
        "border_unit_width_default",
        &["*732", "--border-unit", "intrinsic"],
    );
    assert_eq!(default, given);
}

#[test]
fn render_diagnostic_gives_the_cause_of_a_centre_corner_that_the_domain_does_not_have() {
    insta::assert_snapshot!(refused_snapshot(
        "centre_outside",
        &["*732", "--centre", "3"]
    ));
}

#[test]
fn render_diagnostic_says_the_domain_of_1_has_no_corner() {
    insta::assert_snapshot!(refused_snapshot(
        "centre_no_corner",
        &["1", "--centre", "0"]
    ));
}

#[test]
fn render_centred_on_each_corner_writes_another_image_of_the_requested_size() {
    let images: Vec<Vec<u8>> = [None, Some("0"), Some("1"), Some("2")]
        .into_iter()
        .map(|corner| {
            let mut args = vec!["*732"];
            args.extend(
                corner
                    .map(|corner| ["--centre", corner])
                    .into_iter()
                    .flatten(),
            );
            small_png(&format!("centre_each_{}", corner.unwrap_or("none")), &args)
        })
        .collect();
    for (index, first) in images.iter().enumerate() {
        for (other, second) in images.iter().enumerate().skip(index + 1) {
            assert_ne!(first, second, "{index} {other}");
        }
    }
}

#[test]
fn render_centred_writes_a_rotation_domain_a_euclidean_tiling_and_a_turned_ball() {
    for (test, args) in [
        ("centre_rotation", &["732", "--centre", "1"][..]),
        ("centre_euclidean", &["*632", "--centre", "0"][..]),
        (
            "centre_ball",
            &["*532", "--centre", "2", "--rotation", "20,35,0"][..],
        ),
    ] {
        small_png(test, args);
    }
}

#[test]
fn render_centred_combines_with_a_model_wythoff_and_an_arrangement() {
    let image = image_item("centre_arrangement", [0x20, 0x40, 0x80]);
    for (test, args) in [
        ("centre_band", vec!["--model", "band"]),
        ("centre_wythoff", vec!["--wythoff", "010"]),
        (
            "centre_arrangement",
            vec![
                "--wythoff",
                "010",
                "--arrangement",
                "14a",
                "--colours",
                &image,
            ],
        ),
    ] {
        let mut all = vec!["*732", "--centre", "0"];
        all.extend(args);
        small_png(test, &all);
    }
}

#[test]
fn render_with_a_colouring_id_reports_a_centre_corner_before_the_search() {
    let outcome = run_args(&[
        "render",
        "*(12)(12)6",
        "--colouring",
        "12a",
        "--centre",
        "3",
        "-o",
        NEVER_WRITTEN,
    ]);
    assert_eq!(outcome.status, Status::Failure);
    assert_eq!(
        outcome.stderr,
        "error: cannot render *(12)(12)6\n  caused by: cannot build renderer: the corner 3 is not below the 3 corners of the domain\n"
    );
}

fn image_file(test: &str, name: &str) -> (PathBuf, String) {
    let output = output_path(test);
    let image = output.with_file_name(name);
    let item = format!("image:{}", path_text(&image));
    (image, item)
}

fn image_item(test: &str, rgb: [u8; 3]) -> String {
    let (image, item) = image_file(&format!("{test}_image"), "fill.png");
    write_image(&image, 6, 4, rgb);
    item
}

#[test]
fn render_diagnostic_says_an_image_item_without_a_path_is_not_a_colour() {
    insta::assert_snapshot!(render_snapshot(
        "image_without_path",
        &["*732", "--colours", "image:,#e9c46a"]
    ));
}

#[test]
fn render_diagnostic_says_an_image_file_that_does_not_exist_cannot_be_opened() {
    let (_, item) = image_file("missing_image", "missing.png");
    let colours = format!("#e9c46a,{item}");
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "--colours",
        &colours,
        "-o",
        NEVER_WRITTEN
    ]));
}

#[test]
fn render_diagnostic_says_a_file_that_is_not_a_png_image_cannot_be_used() {
    let (image, item) = image_file("image_not_png", "fill.png");
    std::fs::write(&image, b"not a PNG image").expect("the file is written");
    let colours = format!("#e9c46a,{item}");
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "--colours",
        &colours,
        "-o",
        NEVER_WRITTEN
    ]));
}

#[test]
fn render_diagnostic_gives_the_cause_of_an_image_over_the_size_limit() {
    let (image, item) = image_file("image_size_limit", "wide.png");
    write_image(&image, 16385, 1, [0, 0, 0]);
    let colours = format!("{item},#e9c46a");
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*732",
        "--colours",
        &colours,
        "-o",
        NEVER_WRITTEN
    ]));
}

#[test]
fn render_diagnostic_says_the_lune_has_no_mask_polygon_for_an_image_item() {
    let item = image_item("image_lune", [0x2a, 0x9d, 0x8f]);
    let colours = format!("{item},#e9c46a");
    insta::assert_snapshot!(snapshot(&[
        "render",
        "*22",
        "--colours",
        &colours,
        "-o",
        NEVER_WRITTEN
    ]));
}

#[test]
fn render_with_an_image_item_and_a_colour_writes_a_file() {
    let item = image_item("image_and_colour", [0x2a, 0x9d, 0x8f]);
    let colours = format!("#e9c46a,{item}");
    let file = rendered(
        "image_and_colour",
        &[
            "*732",
            "--colours",
            &colours,
            "--size",
            "48x32",
            "--samples",
            "2",
        ],
    );
    assert_png_bytes(&file, 48, 32);
}

#[test]
fn render_with_an_image_item_draws_one_and_parity_through_the_coset_fill() {
    let one = image_item("image_one", [0x91, 0x36, 0x71]);
    let file = small_png(
        "image_one",
        &["*632", "--colouring", "one", "--colours", &one],
    );
    let solid = small_png(
        "image_one_solid",
        &["*632", "--colouring", "one", "--colours", "#913671"],
    );
    assert_eq!(file, solid);
    let parity = image_item("image_parity", [0x91, 0x36, 0x71]);
    let colours = format!("{parity},#e9c46a");
    let file = small_png(
        "image_parity",
        &["*632", "--colouring", "parity", "--colours", &colours],
    );
    let solid = small_png(
        "image_parity_solid",
        &[
            "*632",
            "--colouring",
            "parity",
            "--colours",
            "#913671,#e9c46a",
        ],
    );
    assert_eq!(file, solid);
}

#[test]
fn render_draws_the_parity_colouring_with_image_items_as_parity() {
    for (test, signature) in [
        ("image_parity_reflection", "*632"),
        ("image_parity_rotation", "442"),
    ] {
        let first = image_item(&format!("{test}_first"), [0x91, 0x36, 0x71]);
        let second = image_item(&format!("{test}_second"), [0x3b, 0x70, 0xc2]);
        let colours = format!("{first},{second}");
        let id = small_png(
            &format!("{test}_id"),
            &[signature, "--colouring", "2a", "--colours", &colours],
        );
        let alias = small_png(
            &format!("{test}_alias"),
            &[signature, "--colouring", "parity", "--colours", &colours],
        );
        assert_eq!(id, alias, "{signature}");
    }
}

#[test]
fn render_with_an_image_that_cannot_be_read_changes_no_file() {
    let (image, item) = image_file("image_unreadable_source", "fill.png");
    std::fs::write(&image, b"not a PNG image").expect("the file is written");
    let colours = format!("#e9c46a,{item}");
    assert_no_file_changes("image_unreadable_file", &["*732", "--colours", &colours]);
    let (_, missing) = image_file("image_missing_source", "missing.png");
    let colours = format!("{missing},#e9c46a");
    assert_no_file_changes("image_missing_file", &["*732", "--colours", &colours]);
    let (wide, item) = image_file("image_wide_source", "wide.png");
    write_image(&wide, 16385, 1, [0, 0, 0]);
    let colours = format!("{item},#e9c46a");
    assert_no_file_changes("image_wide_file", &["*732", "--colours", &colours]);
}

#[test]
fn render_with_parity_and_image_items_for_a_domain_without_a_valid_colouring_with_two_colours_returns_failure()
 {
    let first = image_item("image_parity_missing_first", [0x91, 0x36, 0x71]);
    let second = image_item("image_parity_missing_second", [0x3b, 0x70, 0xc2]);
    let colours = format!("{first},{second}");
    assert_eq!(
        first_error_line(&["732", "--colouring", "parity", "--colours", &colours]),
        (
            Status::Failure,
            "error: cannot render 732: the colouring parity does not exist, because the signature has no colouring with 2 colours"
                .to_owned()
        )
    );
}

#[test]
fn render_reports_an_image_that_cannot_be_opened_before_the_search_for_an_id() {
    let (_, missing) = image_file("image_before_search", "missing.png");
    let mut items = vec!["#e9c46a"; 11];
    items.push(&missing);
    let colours = items.join(",");
    assert_eq!(
        first_error_line(&["*(12)(12)6", "--colouring", "12a", "--colours", &colours]),
        (
            Status::Failure,
            "error: cannot render *(12)(12)6: the image of colour 12 of --colours cannot be opened"
                .to_owned()
        )
    );
}

fn refused_snapshot(test: &str, args: &[&str]) -> String {
    let (outcome, output) = snapshot_and_output(test, args);
    assert!(outcome.starts_with("status: Failure\n"), "{outcome}");
    assert!(!output.exists(), "{args:?}");
    outcome
}

#[test]
fn render_diagnostic_says_wythoff_marks_all_off_give_no_face() {
    insta::assert_snapshot!(refused_snapshot(
        "wythoff_no_marks",
        &["*732", "--wythoff", "000"]
    ));
}

#[test]
fn render_diagnostic_says_wythoff_needs_a_reflection_triangle() {
    insta::assert_snapshot!(refused_snapshot(
        "wythoff_rotation_triangle",
        &["732", "--wythoff", "010"]
    ));
}

#[test]
fn render_diagnostic_says_wythoff_marks_of_a_digon_give_no_face() {
    insta::assert_snapshot!(refused_snapshot(
        "wythoff_digon",
        &["*522", "--wythoff", "100"]
    ));
}

#[test]
fn render_diagnostic_refuses_wythoff_with_a_colouring() {
    insta::assert_snapshot!(refused_snapshot(
        "wythoff_colouring",
        &["*732", "--wythoff", "010", "--colouring", "parity"]
    ));
}

#[test]
fn render_diagnostic_counts_the_colours_of_the_face_types_of_wythoff() {
    insta::assert_snapshot!(refused_snapshot(
        "wythoff_colour_count",
        &["*732", "--wythoff", "111", "--colours", "#000,#fff"]
    ));
}

#[test]
fn render_diagnostic_says_an_image_item_is_not_a_colour_with_wythoff() {
    insta::assert_snapshot!(refused_snapshot(
        "wythoff_image_item",
        &["*732", "--wythoff", "010", "--colours", "image:fill.png"]
    ));
}

const MARKINGS: [&str; 7] = ["100", "010", "001", "110", "101", "011", "111"];

#[test]
fn render_draws_each_wythoff_marking_of_a_hyperbolic_triangle_as_another_image() {
    let files: Vec<Vec<u8>> = MARKINGS
        .iter()
        .map(|marks| {
            small_png(
                &format!("wythoff_marking_{marks}"),
                &["*732", "--wythoff", marks],
            )
        })
        .collect();
    for (first, file) in files.iter().enumerate() {
        for (second, other) in files.iter().enumerate().skip(first + 1) {
            assert_ne!(file, other, "{} and {}", MARKINGS[first], MARKINGS[second]);
        }
    }
    let tiles = small_png("wythoff_marking_tiles", &["*732"]);
    assert_ne!(files[1], tiles);
}

#[test]
fn render_draws_wythoff_on_a_euclidean_and_a_spherical_triangle() {
    for (signature, marks, name) in [("*632", "111", "euclidean"), ("*532", "110", "spherical")] {
        let faces = small_png(&format!("wythoff_{name}"), &[signature, "--wythoff", marks]);
        let tiles = small_png(&format!("wythoff_{name}_tiles"), &[signature]);
        assert_ne!(faces, tiles, "{signature}");
    }
}

fn library_wythoff_png(signature: &str, marks: [bool; 3], colours: Vec<Colour>) -> Vec<u8> {
    let signature: Signature = signature.parse().expect("the signature is valid");
    let domain = FundamentalDomain::build(&signature).expect("the domain is built");
    let wythoff = Wythoff::new(&domain, marks).expect("the construction is built");
    let fill = WythoffFill::new(wythoff, colours).expect("the fill is valid");
    let options = RenderOptions::builder(Fill::Wythoff(fill))
        .size(48, 32)
        .samples(2)
        .build()
        .expect("the options are valid");
    let (pixels, _) = Renderer::new(&domain, options)
        .expect("the renderer is built")
        .render();
    encode_png(48, 32, &pixels).expect("the image is encoded")
}

#[test]
fn render_draws_wythoff_with_one_given_colour_for_each_face_type() {
    let gold = Colour::from_srgb8(0xe9, 0xc4, 0x6a);
    let teal = Colour::from_srgb8(0x2a, 0x9d, 0x8f);
    let plum = Colour::from_srgb8(0x91, 0x36, 0x71);
    let files: Vec<Vec<u8>> = [
        (
            "in_order",
            "#e9c46a,#2a9d8f,#913671",
            vec![gold, teal, plum],
        ),
        (
            "reordered",
            "#913671,#e9c46a,#2a9d8f",
            vec![plum, gold, teal],
        ),
    ]
    .into_iter()
    .map(|(name, text, colours)| {
        let file = small_png(
            &format!("wythoff_given_colours_{name}"),
            &["*732", "--wythoff", "110", "--colours", text],
        );
        assert_eq!(
            file,
            library_wythoff_png("*732", [true, true, false], colours),
            "{text}"
        );
        file
    })
    .collect();
    assert_ne!(files[0], files[1]);
}

#[test]
fn render_draws_wythoff_without_colours_in_the_first_default_colours() {
    for (marks, colours) in [("010", "#e9c46a"), ("111", "#e9c46a,#2a9d8f,#913671")] {
        let default = small_png(
            &format!("wythoff_default_colours_{marks}"),
            &["*732", "--wythoff", marks],
        );
        let given = small_png(
            &format!("wythoff_given_default_colours_{marks}"),
            &["*732", "--wythoff", marks, "--colours", colours],
        );
        assert_eq!(default, given, "{marks}");
    }
}

fn arrangement_snapshot(test: &str, args: &[&str]) -> String {
    let (snapshot, output) = snapshot_and_output(test, args);
    assert!(!output.exists(), "{test} wrote the output file");
    snapshot
}

#[test]
fn render_diagnostic_says_an_arrangement_needs_wythoff() {
    insta::assert_snapshot!(arrangement_snapshot(
        "arrangement_without_wythoff",
        &["*632", "--arrangement", "12a"]
    ));
}

#[test]
fn render_diagnostic_refuses_an_arrangement_of_a_tiling_that_is_not_regular() {
    insta::assert_snapshot!(arrangement_snapshot(
        "arrangement_not_regular",
        &["*732", "--wythoff", "111", "--arrangement", "14a"]
    ));
}

#[test]
fn render_diagnostic_says_an_arrangement_does_not_exist() {
    insta::assert_snapshot!(arrangement_snapshot(
        "arrangement_missing",
        &["*632", "--wythoff", "010", "--arrangement", "12h"]
    ));
}

#[test]
fn render_diagnostic_gives_the_index_limit_for_the_arrangements_of_a_polygon_of_33_sides() {
    insta::assert_snapshot!(arrangement_snapshot(
        "arrangement_index_limit",
        &["*(33)32", "--wythoff", "010", "--arrangement", "12a"]
    ));
}

#[test]
fn render_diagnostic_counts_one_colour_for_an_arrangement() {
    insta::assert_snapshot!(arrangement_snapshot(
        "arrangement_two_colours",
        &[
            "*632",
            "--wythoff",
            "010",
            "--arrangement",
            "12a",
            "--colours",
            "#000,#fff"
        ]
    ));
}

#[test]
fn render_diagnostic_says_a_tiling_without_arrangements_has_none() {
    insta::assert_snapshot!(arrangement_snapshot(
        "arrangement_none",
        &["*732", "--wythoff", "100", "--arrangement", "6a"]
    ));
}

#[test]
fn render_draws_an_arrangement_with_an_image_item_or_one_colour() {
    let image = image_item("arrangement_picture", [0x20, 0x40, 0x80]);
    let [picture, colour, default] = [
        ("arrangement_with_image", Some(image.as_str())),
        ("arrangement_with_colour", Some("#e9c46a")),
        ("arrangement_with_default_colour", None),
    ]
    .map(|(test, colours)| {
        let mut args = vec!["*632", "--wythoff", "010", "--arrangement", "12a"];
        args.extend(
            colours
                .map(|colours| ["--colours", colours])
                .into_iter()
                .flatten(),
        );
        small_png(test, &args)
    });
    assert_eq!(default, colour);
    assert_ne!(default, picture);
}

#[test]
fn render_with_an_arrangement_reports_an_image_that_cannot_be_opened_before_the_search() {
    let (_, item) = image_file("arrangement_missing_image", "missing.png");
    assert_eq!(
        first_error_line(&[
            "*(33)32",
            "--wythoff",
            "010",
            "--arrangement",
            "12a",
            "--colours",
            &item
        ]),
        (
            Status::Failure,
            "error: cannot render *(33)32: the image of colour 1 of --colours cannot be opened"
                .to_owned()
        )
    );
}

#[test]
fn render_with_an_arrangement_that_fails_gives_its_diagnostic_and_changes_no_file() {
    let (image, unreadable) = image_file("arrangement_refused_image_source", "fill.png");
    std::fs::write(&image, b"not a PNG image").expect("the file is written");
    let hexagons = ["*632", "--wythoff", "010", "--arrangement"];
    for (test, extra, diagnostic) in [
        (
            "arrangement_refused_missing_id",
            &["12h"][..],
            "error: cannot render *632: the arrangement 12h does not exist, because the tiling has 7 arrangements with 1 picture\n",
        ),
        (
            "arrangement_refused_image",
            &["12a", "--colours", &unreadable],
            "error: cannot render *632: the image of colour 1 of --colours cannot be used\n  caused by: cannot decode image: the data is not a valid PNG image\n  caused by: Invalid PNG signature.\n",
        ),
        (
            "arrangement_refused_two_colours",
            &["12a", "--colours", "#000,#fff"],
            "error: cannot render *632: the arrangement 12a needs 1 colour, and 2 were given\n",
        ),
        (
            "arrangement_refused_colouring",
            &["12a", "--colouring", "2a"],
            "error: cannot render *632: --wythoff cannot be combined with --colouring\n",
        ),
        (
            "arrangement_refused_setting",
            &["12a", "--set", "p=3"],
            "error: cannot render *632: the signature has no shape parameter p, nor any other\n",
        ),
    ] {
        let mut args = hexagons.to_vec();
        args.extend_from_slice(extra);
        assert_eq!(
            failure_with_no_file_change("render", test, &args),
            diagnostic,
            "{test}"
        );
    }
    assert_eq!(
        failure_with_no_file_change(
            "render",
            "arrangement_refused_none",
            &["*732", "--wythoff", "100", "--arrangement", "6a"]
        ),
        "error: cannot render *732: the tiling has no arrangement that puts one picture on every polygon\n"
    );
}

type Adjust = fn(RenderOptionsBuilder) -> RenderOptionsBuilder;

fn library_heptagons_png(width: u32, height: u32, adjust: Adjust) -> Vec<u8> {
    let (domain, face, arrangement) = regular_arrangement("*732", [false, true, false], "14a");
    let paint = Paint::Solid(Colour::from_srgb8(0xe9, 0xc4, 0x6a));
    let fill = WholeFaceFill::new(face, arrangement, vec![paint]).expect("the fill is valid");
    let options = adjust(
        RenderOptions::builder(Fill::WholeFace(fill))
            .size(width, height)
            .samples(2),
    )
    .build()
    .expect("the options are valid");
    let (pixels, _) = Renderer::new(&domain, options)
        .expect("the renderer is built")
        .render();
    encode_png(width, height, &pixels).expect("the image is encoded")
}

#[test]
fn render_applies_the_model_the_border_width_and_the_size_to_an_arrangement() {
    let heptagons = [
        "*732",
        "--wythoff",
        "010",
        "--arrangement",
        "14a",
        "--samples",
        "2",
    ];
    let mut args = heptagons.to_vec();
    args.extend_from_slice(&["--size", "48x32"]);
    let default = rendered("arrangement_options_default", &args);
    assert_eq!(default, library_heptagons_png(48, 32, |builder| builder));
    let cases: [(&str, &[&str], u32, u32, Adjust); 3] = [
        (
            "model",
            &["--size", "48x32", "--model", "half-plane"],
            48,
            32,
            |builder| builder.model(HyperbolicModel::HalfPlane),
        ),
        (
            "border",
            &["--size", "48x32", "--border-width", "0"],
            48,
            32,
            |builder| builder.border_width(0.0),
        ),
        ("size", &["--size", "40x40"], 40, 40, |builder| builder),
    ];
    for (name, extra, width, height, adjust) in cases {
        let mut args = heptagons.to_vec();
        args.extend_from_slice(extra);
        let file = rendered(&format!("arrangement_options_{name}"), &args);
        assert_png_bytes(&file, width, height);
        assert_eq!(file, library_heptagons_png(width, height, adjust), "{name}");
        assert_ne!(file, default, "{name}");
    }
}

#[test]
fn render_diagnostic_counts_three_colours_for_an_arrangement_of_three_pictures() {
    insta::assert_snapshot!(arrangement_snapshot(
        "arrangement_three_pictures_one_colour",
        &[
            "*732",
            "--wythoff",
            "100",
            "--arrangement",
            "18a",
            "--colours",
            "#000"
        ]
    ));
}

#[test]
fn render_diagnostic_says_an_arrangement_of_two_pictures_does_not_exist() {
    insta::assert_snapshot!(arrangement_snapshot(
        "arrangement_two_pictures_missing",
        &["*632", "--wythoff", "010", "--arrangement", "24zz"]
    ));
}

#[test]
fn render_diagnostic_says_the_tiling_has_no_arrangement_with_2_pictures() {
    insta::assert_snapshot!(arrangement_snapshot(
        "arrangement_none_of_two_pictures",
        &["*732", "--wythoff", "100", "--arrangement", "12a"]
    ));
}

#[test]
fn an_arrangement_id_above_the_index_limit_on_a_regular_tiling_fails_before_its_colours_are_read() {
    let without: &[&str] = &[];
    let with = &["--colours", "#000"][..];
    for (command, task, extras) in [
        ("render", "*732", &[without, with][..]),
        ("render-masks", "the masks of *732", &[without][..]),
    ] {
        for (id, index) in [("70a", 70), ("66a", 66)] {
            for (number, extra) in extras.iter().enumerate() {
                let mut args = vec!["*732", "--wythoff", "010", "--arrangement", id];
                args.extend_from_slice(extra);
                assert_eq!(
                    failure_with_no_mask_change(
                        command,
                        &format!("index_above_limit_{command}_{id}_{number}"),
                        &args,
                        5
                    ),
                    format!(
                        "error: cannot render {task}\n  caused by: cannot search subgroups: the index limit {index} is not between 1 and 64\n"
                    ),
                    "{args:?}"
                );
            }
        }
    }
}
