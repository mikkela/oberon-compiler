use std::process::Command;

#[test]
fn shows_error_on_missing_file() {
    let exe = env!("CARGO_BIN_EXE_oberon-compiler");
    let out = Command::new(exe)
        .arg("no_such_file.Mod")
        .output()
        .expect("run");

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("error:"));
}
