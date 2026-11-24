use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;

use escher_cli::{Status, run};
use escher_domain::FundamentalDomain;
use escher_group::{Colouring, IndexLimit, SearchBudget, free_classes_of_index};
use escher_render::{WholeFace, Wythoff};
use escher_signature::Signature;

pub(crate) struct Outcome {
    pub(crate) status: Status,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) fn run_os(args: Vec<OsString>) -> Outcome {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut all = vec![OsString::from("escher")];
    all.extend(args);
    let status = run(all, &mut stdout, &mut stderr);
    Outcome {
        status,
        stdout: String::from_utf8(stdout).expect("standard output is UTF-8"),
        stderr: String::from_utf8(stderr).expect("standard error is UTF-8"),
    }
}

pub(crate) fn run_args(args: &[&str]) -> Outcome {
    run_os(args.iter().map(OsString::from).collect())
}

pub(crate) fn snapshot(args: &[&str]) -> String {
    let outcome = run_args(args);
    format!(
        "status: {:?}\n--- stdout\n{}--- stderr\n{}--- end\n",
        outcome.status,
        without_statistics(&without_residuals(&outcome.stdout)),
        outcome.stderr
    )
}

fn without_residuals(text: &str) -> String {
    let mut redacted = String::new();
    for line in text.split_inclusive('\n') {
        let parts = line
            .strip_prefix("Check ")
            .and_then(|rest| rest.split_once("residual "))
            .and_then(|(head, rest)| Some((head, rest.split_once(',')?.1)));
        match parts {
            Some((head, tail)) => {
                redacted.push_str("Check ");
                redacted.push_str(head);
                redacted.push_str("residual <residual>,");
                redacted.push_str(tail);
            }
            None => redacted.push_str(line),
        }
    }
    redacted
}

fn without_statistics(text: &str) -> String {
    let mut redacted = String::new();
    for line in text.split_inclusive('\n') {
        let label = ["Mean folds", "Maximum folds", "Unconverged samples", "Time"]
            .into_iter()
            .find(|label| line.starts_with(&format!("{label}: ")));
        match label {
            Some(label) => {
                redacted.push_str(label);
                redacted.push_str(": <value>\n");
            }
            None => redacted.push_str(line),
        }
    }
    redacted
}

pub(crate) const NEVER_WRITTEN: &str = concat!(
    env!("CARGO_TARGET_TMPDIR"),
    "/",
    env!("CARGO_PKG_NAME"),
    "/never-written.png"
);

pub(crate) fn output_path(test: &str) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(env!("CARGO_PKG_NAME"))
        .join(test);
    match std::fs::remove_dir_all(&directory) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => {
            panic!("cannot empty the directory of the test: {error}")
        }
        _ => {}
    }
    std::fs::create_dir_all(&directory).expect("the directory of the test is created");
    directory.join("tiling.png")
}

pub(crate) fn failure_with_no_file_change(command: &str, test: &str, args: &[&str]) -> String {
    failure_with_no_mask_change(command, test, args, 0)
}

pub(crate) fn failure_with_no_mask_change(
    command: &str,
    test: &str,
    args: &[&str],
    masks: u32,
) -> String {
    let existing = output_path(test);
    let missing = existing.with_file_name("missing.png");
    let mask_names = |stem: &str| -> Vec<PathBuf> {
        (0..masks)
            .map(|mask| existing.with_file_name(format!("{stem}-{mask}.png")))
            .collect()
    };
    let existing_files: Vec<PathBuf> = [existing.clone()]
        .into_iter()
        .chain(mask_names("tiling"))
        .collect();
    let missing_files: Vec<PathBuf> = [missing.clone()]
        .into_iter()
        .chain(mask_names("missing"))
        .collect();
    for file in &existing_files {
        std::fs::write(file, b"old").expect("the existing file is written");
    }
    let diagnostics: Vec<String> = [&existing, &missing]
        .into_iter()
        .map(|output| {
            let mut all = vec![command];
            all.extend_from_slice(args);
            all.extend_from_slice(&["-o", path_text(output)]);
            let outcome = run_args(&all);
            assert_eq!(outcome.status, Status::Failure, "{args:?}");
            assert_eq!(outcome.stdout, "", "{args:?}");
            outcome.stderr
        })
        .collect();
    for file in &existing_files {
        assert_eq!(
            std::fs::read(file).expect("the existing file stays"),
            b"old",
            "{args:?} {}",
            file.display()
        );
    }
    for file in &missing_files {
        assert!(!file.exists(), "{args:?} {}", file.display());
    }
    assert_eq!(diagnostics[0], diagnostics[1], "{args:?}");
    diagnostics[0].clone()
}

pub(crate) fn regular_arrangement(
    signature: &str,
    marks: [bool; 3],
    id: &str,
) -> (FundamentalDomain, WholeFace, Colouring) {
    let signature: Signature = signature.parse().expect("the signature is valid");
    let domain = FundamentalDomain::build(&signature).expect("the signature has a domain");
    let face = WholeFace::new(
        &domain,
        Wythoff::new(&domain, marks).expect("the construction is built"),
    )
    .expect("the tiling is regular");
    let id: escher_group::ColouringId = id.parse().expect("the id is valid");
    let arrangement = free_classes_of_index(
        face.presentation(),
        face.stabilizer(),
        IndexLimit::new(id.colours()).expect("the index is within the limit"),
        SearchBudget::DEFAULT,
    )
    .expect("the search runs")
    .find(id)
    .expect("the arrangement exists")
    .clone();
    (domain, face, arrangement)
}

pub(crate) fn path_text(path: &std::path::Path) -> &str {
    path.to_str().expect("the path of the test is UTF-8")
}

#[derive(Debug, Default)]
pub(crate) struct FailingWriter {
    pub(crate) write_error: Option<io::ErrorKind>,
    pub(crate) flush_error: Option<io::ErrorKind>,
    pub(crate) written: Vec<u8>,
}

impl FailingWriter {
    pub(crate) fn failing_write(kind: io::ErrorKind) -> Self {
        Self {
            write_error: Some(kind),
            ..Self::default()
        }
    }

    pub(crate) fn failing_flush(kind: io::ErrorKind) -> Self {
        Self {
            flush_error: Some(kind),
            ..Self::default()
        }
    }
}

impl Write for FailingWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        match self.write_error {
            Some(kind) => Err(io::Error::from(kind)),
            None => self.written.write(buffer),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.flush_error {
            Some(kind) => Err(io::Error::from(kind)),
            None => Ok(()),
        }
    }
}

pub(crate) const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

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

pub(crate) fn assert_png_bytes(file: &[u8], width: u32, height: u32) {
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

pub(crate) fn write_image(path: &std::path::Path, width: u32, height: u32, rgb: [u8; 3]) {
    let pixels = rgb.repeat(usize::try_from(width * height).expect("the image fits a usize"));
    let file = escher_render::encode_png(width, height, &pixels).expect("the image is encoded");
    std::fs::write(path, file).expect("the image file is written");
}
