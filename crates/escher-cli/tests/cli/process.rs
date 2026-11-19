use std::process::Command;

use crate::common::output_path;

#[test]
fn binary_renders_a_file_and_exits_with_status_zero() {
    let output = output_path("smoke");
    let result = Command::new(env!("CARGO_BIN_EXE_escher"))
        .args(["render", "*732", "--size", "32x32", "-o"])
        .arg(&output)
        .output()
        .expect("the binary starts");
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(result.stdout, b"");
    assert_eq!(result.stderr, b"");
    let file = std::fs::read(&output).expect("the output file is written");
    assert!(file.starts_with(b"\x89PNG\r\n\x1a\n"));
}
