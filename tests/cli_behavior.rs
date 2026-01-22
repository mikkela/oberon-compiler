use std::{fs, process::Command};
use tempfile::tempdir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_oberon-compiler")
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("failed to run binary")
}

#[test]
fn missing_file_exits_with_error() {
    let out = run(&["no_such_file.Mod"]);

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("error:"), "stderr was: {stderr}");
}

#[test]
fn empty_file_produces_structured_diagnostic() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("Empty.Mod");
    fs::write(&input, "").unwrap();

    let out = run(&[input.to_str().unwrap()]);

    assert!(!out.status.success());

    let stderr = String::from_utf8_lossy(&out.stderr);
    // Message
    assert!(stderr.contains("Input file is empty."), "stderr was: {stderr}");
    // Note
    assert!(stderr.contains("note:"), "stderr was: {stderr}");
    // Location marker (path:line:col)
    assert!(stderr.contains("-->"), "stderr was: {stderr}");
}

#[test]
fn default_output_is_input_dot_bin() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("Hello.Mod");
    fs::write(&input, "MODULE Hello; END Hello.\n").unwrap();

    let out = run(&[input.to_str().unwrap()]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

    let expected_out = dir.path().join("Hello.bin");
    assert!(expected_out.exists(), "Expected output file to exist: {expected_out:?}");

    let data = fs::read(&expected_out).unwrap();
    assert!(data.is_empty(), "Scaffolding stage writes empty output for now");
}

#[test]
fn output_flag_overrides_output_path() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("Hello.Mod");
    fs::write(&input, "MODULE Hello; END Hello.\n").unwrap();

    let out_path = dir.path().join("custom_output.bin");

    let out = run(&[
        input.to_str().unwrap(),
        "--output",
        out_path.to_str().unwrap(),
    ]);

    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(out_path.exists(), "Expected output to exist at custom path");
}

#[test]
fn verbose_prints_extra_information() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("Hello.Mod");
    fs::write(&input, "MODULE Hello; END Hello.\n").unwrap();

    let out = run(&[input.to_str().unwrap(), "--verbose"]);

    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Loaded:"), "stderr was: {stderr}");
    assert!(stderr.contains("Will write output to:"), "stderr was: {stderr}");
}

#[test]
fn invalid_utf8_is_reported_as_encoding_error() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("Bad.Mod");

    // Write invalid UTF-8 bytes
    fs::write(&input, vec![0xFF, 0xFE, 0xFF]).unwrap();

    let out = run(&[input.to_str().unwrap()]);

    assert!(!out.status.success());

    let stderr = String::from_utf8_lossy(&out.stderr);
    // Comes from CompilerError::Encoding display
    assert!(stderr.contains("not valid UTF-8"), "stderr was: {stderr}");
    assert!(stderr.contains("error:"), "stderr was: {stderr}");
}
