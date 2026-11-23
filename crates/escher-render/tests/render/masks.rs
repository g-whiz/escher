use escher_domain::FundamentalDomain;
use escher_group::Colouring;
use escher_render::{CosetFill, ErrorKind, Fill, Masks, Paint, RenderOptions, Renderer};

use crate::golden_images::{
    colouring, domain_with_settings, golden_colouring, image_of, mask, mask_domains, masks,
};

fn refusal(domain: &FundamentalDomain, colouring: &Colouring) -> (ErrorKind, String) {
    match Masks::new(domain, colouring) {
        Ok(_) => panic!("the colouring {} is accepted", colouring.id()),
        Err(error) => (error.kind(), error.to_string()),
    }
}

#[test]
fn masks_refuse_a_colouring_with_another_number_of_generators_or_a_relator_that_moves_a_coset() {
    let triangle = domain_with_settings("*732", &[]);
    let lune = colouring(&domain_with_settings("22", &[]), "2a");
    assert_eq!(
        refusal(&triangle, &lune),
        (
            ErrorKind::InvalidOption,
            "cannot build masks: the colouring has 2 generators, and the domain has 3 sides"
                .to_owned()
        )
    );
    let other = colouring(&domain_with_settings("*642", &[]), "3a");
    assert_eq!(
        refusal(&triangle, &other),
        (
            ErrorKind::InvalidOption,
            "cannot build masks: relator 3 of the side presentation of the domain moves a coset of the colouring"
                .to_owned()
        )
    );
}

fn render_refusal(masks: &Masks, colour: u32, width: u32, height: u32) -> (ErrorKind, String) {
    match masks.render(colour, width, height) {
        Ok(_) => panic!("the mask of colour {colour} at {width}x{height} is drawn"),
        Err(error) => (error.kind(), error.to_string()),
    }
}

#[test]
fn render_refuses_a_colour_at_or_above_the_count_and_a_size_outside_the_limits() {
    let triangle = domain_with_settings("*642", &[]);
    let four = masks(&triangle, &golden_colouring(&triangle, "4a"));
    assert_eq!(four.count(), 4);
    let one = masks(&triangle, &colouring(&triangle, "1a"));
    assert_eq!(one.count(), 1);
    let refusals = [
        (
            render_refusal(&four, 4, 8, 8),
            ErrorKind::InvalidOption,
            "cannot render mask: the colour 4 is not below the 4 colours of the colouring",
        ),
        (
            render_refusal(&one, 1, 8, 8),
            ErrorKind::InvalidOption,
            "cannot render mask: the colour 1 is not below the 1 colour of the colouring",
        ),
        (
            render_refusal(&four, u32::MAX, 0, 512),
            ErrorKind::InvalidOption,
            "cannot render mask: the colour 4294967295 is not below the 4 colours of the colouring",
        ),
        (
            render_refusal(&four, 3, 0, 512),
            ErrorKind::InvalidOption,
            "cannot render mask: the image size 0x512 is empty",
        ),
        (
            render_refusal(&four, 3, 16385, 1),
            ErrorKind::LimitExceeded,
            "cannot render mask: the image size 16385x1 is larger than 16384 pixels on a side",
        ),
        (
            render_refusal(&four, 3, 16384, 8192),
            ErrorKind::LimitExceeded,
            "cannot render mask: the image size 16384x8192 has more than 67108864 pixels",
        ),
    ];
    for ((kind, text), expected_kind, expected_text) in refusals {
        assert_eq!((kind, text.as_str()), (expected_kind, expected_text));
    }
    assert_eq!(mask(&four, 3, 1, 1).len(), 3);
    assert_eq!(mask(&four, 0, 16384, 1).len(), 3 * 16384);
}

const NO_POLYGON: [&str; 11] = [
    "*22", "*55", "22", "55", "5x", "x", "*", "1", "*222", "222", "2*",
];

fn image_fill(domain: &FundamentalDomain, colouring: &Colouring) -> Fill {
    let colours = usize::try_from(colouring.colours()).unwrap_or(usize::MAX);
    let paints = vec![Paint::Image(image_of(2, 2, &[90; 12])); colours];
    Fill::Cosets(
        CosetFill::new(domain, colouring.clone(), paints).unwrap_or_else(|error| {
            panic!("the image fill of {} is rejected: {error}", colouring.id())
        }),
    )
}

fn renderer_refusal(domain: &FundamentalDomain, colouring: &Colouring) -> (ErrorKind, String) {
    let options = RenderOptions::builder(image_fill(domain, colouring))
        .size(8, 8)
        .build()
        .unwrap_or_else(|error| panic!("{error}"));
    match Renderer::new(domain, options) {
        Ok(_) => panic!("the renderer accepts an image fill of {}", colouring.id()),
        Err(error) => (error.kind(), error.to_string()),
    }
}

#[test]
fn masks_and_image_fills_refuse_every_domain_without_a_mask_polygon() {
    for signature in NO_POLYGON {
        let domain = domain_with_settings(signature, &[]);
        let trivial = colouring(&domain, "1a");
        assert_eq!(
            refusal(&domain, &trivial),
            (
                ErrorKind::InvalidOption,
                "cannot build masks: the domain has no mask polygon".to_owned()
            ),
            "{signature}"
        );
        assert_eq!(
            renderer_refusal(&domain, &trivial),
            (
                ErrorKind::InvalidOption,
                "cannot build renderer: the domain has no mask polygon".to_owned()
            ),
            "{signature}"
        );
        let solid = Fill::Cosets(
            CosetFill::new(
                &domain,
                trivial,
                vec![Paint::Solid(escher_render::Colour::WHITE)],
            )
            .unwrap_or_else(|error| panic!("{signature}: {error}")),
        );
        let options = RenderOptions::builder(solid)
            .size(8, 8)
            .build()
            .unwrap_or_else(|error| panic!("{error}"));
        assert!(Renderer::new(&domain, options).is_ok(), "{signature}");
    }
}

#[test]
fn masks_check_the_colouring_before_the_mask_polygon() {
    let lune = domain_with_settings("*22", &[]);
    let other = colouring(&domain_with_settings("*732", &[]), "1a");
    assert_eq!(
        refusal(&lune, &other),
        (
            ErrorKind::InvalidOption,
            "cannot build masks: the colouring has 3 generators, and the domain has 2 sides"
                .to_owned()
        )
    );
}

#[test]
fn masks_and_image_fills_accept_every_domain_with_a_mask_polygon() {
    let domains = mask_domains();
    for (name, domain) in &domains {
        let trivial = colouring(domain, "1a");
        let pixels = mask(&masks(domain, &trivial), 0, 64, 64);
        let pixels = pixels.as_chunks::<3>().0;
        assert_eq!(pixels.len(), 64 * 64, "{name}");
        assert_eq!(pixels[0], [0; 3], "{name}: the margin is not black");
        assert!(
            pixels.iter().any(|pixel| *pixel != [0; 3]),
            "{name}: the mask shows no polygon"
        );
        let options = RenderOptions::builder(image_fill(domain, &trivial))
            .size(8, 8)
            .build()
            .unwrap_or_else(|error| panic!("{error}"));
        assert!(Renderer::new(domain, options).is_ok(), "{name}");
    }
    assert!(domains.len() >= 90, "{}", domains.len());
}
