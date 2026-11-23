use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use escher_group::SearchBudget;
use escher_render::{Colour, Fill, Masks, RenderOptions, encode_png};

use crate::args::MasksArgs;
use crate::render::{
    Task, build_domain, cannot_render, chosen_colouring, coset_colouring, parse_signature,
};

pub(crate) fn render_masks(args: &MasksArgs) -> anyhow::Result<()> {
    let signature = parse_signature(&args.tiling.signature)?;
    let task = Task::Masks(&signature);
    let names = MaskNames::new(&args.output)
        .ok_or_else(|| cannot_render(task, "the output path names no file"))?;
    let domain = build_domain(task, &signature, &args.tiling.set)?;
    let (colouring, found_parity) =
        chosen_colouring(&domain, task, args.tiling.colouring, SearchBudget::DEFAULT)?;
    let (width, height) = (args.tiling.size.width, args.tiling.size.height);
    RenderOptions::builder(Fill::One(Colour::WHITE))
        .size(width, height)
        .build()
        .with_context(|| task.failed())?;
    let colouring = coset_colouring(
        &domain,
        task,
        colouring,
        found_parity,
        SearchBudget::DEFAULT,
    )?;
    let masks = Masks::new(&domain, &colouring).with_context(|| task.failed())?;
    for colour in 0..masks.count() {
        let pixels = masks
            .render(colour, width, height)
            .with_context(|| task.failed())?;
        let png = encode_png(width, height, &pixels).with_context(|| task.failed())?;
        std::fs::write(names.name(colour), png)
            .with_context(|| format!("cannot write the mask of colour {colour}"))?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct MaskNames<'a> {
    output: &'a Path,
    stem: &'a OsStr,
    extension: Option<&'a OsStr>,
}

impl<'a> MaskNames<'a> {
    fn new(output: &'a Path) -> Option<Self> {
        Some(Self {
            output,
            stem: output.file_stem()?,
            extension: output.extension(),
        })
    }

    fn name(self, colour: u32) -> PathBuf {
        let mut file = self.stem.to_os_string();
        file.push("-");
        file.push(colour.to_string());
        if let Some(extension) = self.extension {
            file.push(".");
            file.push(extension);
        }
        self.output.with_file_name(file)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt as _;
    use std::path::{Path, PathBuf};

    use super::MaskNames;

    fn names(output: &Path) -> Option<Vec<PathBuf>> {
        MaskNames::new(output).map(|names| (0..3).map(|colour| names.name(colour)).collect())
    }

    fn paths(texts: [&str; 3]) -> Vec<PathBuf> {
        texts.into_iter().map(PathBuf::from).collect()
    }

    #[test]
    fn mask_names_add_the_colour_before_the_extension() {
        assert_eq!(
            names(Path::new("/tmp/masks/mask.png")),
            Some(paths([
                "/tmp/masks/mask-0.png",
                "/tmp/masks/mask-1.png",
                "/tmp/masks/mask-2.png"
            ]))
        );
        assert_eq!(
            names(Path::new("mask.png")),
            Some(paths(["mask-0.png", "mask-1.png", "mask-2.png"]))
        );
    }

    #[test]
    fn mask_names_of_a_stem_without_an_extension_have_no_extension() {
        assert_eq!(
            names(Path::new("/tmp/masks")),
            Some(paths(["/tmp/masks-0", "/tmp/masks-1", "/tmp/masks-2"]))
        );
    }

    #[test]
    fn mask_names_keep_every_dot_of_the_stem() {
        assert_eq!(
            names(Path::new("archive.tar.png")),
            Some(paths([
                "archive.tar-0.png",
                "archive.tar-1.png",
                "archive.tar-2.png"
            ]))
        );
    }

    #[test]
    fn mask_names_keep_the_bytes_of_a_stem_that_is_not_utf8() {
        let output = PathBuf::from(OsString::from_vec(b"/tmp/m\xffsk.png".to_vec()));
        let expected = (0..3_u8)
            .map(|colour| {
                let mut bytes = b"/tmp/m\xffsk-".to_vec();
                bytes.push(b'0' + colour);
                bytes.extend_from_slice(b".png");
                PathBuf::from(OsString::from_vec(bytes))
            })
            .collect();
        assert_eq!(names(&output), Some(expected));
    }

    #[test]
    fn mask_names_need_a_file_stem() {
        for text in ["/", "..", "/tmp/.."] {
            assert_eq!(names(Path::new(text)), None, "{text}");
        }
    }
}
