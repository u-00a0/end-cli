use std::process::Command;
use tempfile::tempdir;

#[test]
fn init_creates_aic_toml() {
    let tmp = tempdir().expect("create temp dir");

    let output = Command::new(assert_cmd::cargo::cargo_bin!("end-cli"))
        .current_dir(tmp.path())
        .arg("init")
        .output()
        .expect("run end-cli init");

    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        tmp.path().join("aic.toml").exists(),
        "aic.toml was not created"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("wrote"),
        "stderr did not include write confirmation: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn solve_without_aic_fails() {
    let tmp = tempdir().expect("create temp dir");

    let output = Command::new(assert_cmd::cargo::cargo_bin!("end-cli"))
        .current_dir(tmp.path())
        .args(["solve", "--lang", "en"])
        .output()
        .expect("run end-cli solve");

    assert!(!output.status.success(), "solve unexpectedly succeeded");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found; run `end-cli init --aic aic.toml` to create it"),
        "stderr did not contain missing-file hint: {stderr}"
    );
}

#[test]
fn solve_with_initialized_aic_succeeds() {
    let tmp = tempdir().expect("create temp dir");

    let init_output = Command::new(assert_cmd::cargo::cargo_bin!("end-cli"))
        .current_dir(tmp.path())
        .arg("init")
        .output()
        .expect("run end-cli init");
    assert!(
        init_output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&init_output.stderr)
    );

    let solve_output = Command::new(assert_cmd::cargo::cargo_bin!("end-cli"))
        .current_dir(tmp.path())
        .args(["solve", "--lang", "en"])
        .output()
        .expect("run end-cli solve");
    assert!(
        solve_output.status.success(),
        "solve failed: {}",
        String::from_utf8_lossy(&solve_output.stderr)
    );

    let stdout = String::from_utf8_lossy(&solve_output.stdout);
    assert!(
        stdout.contains("Conclusion"),
        "stdout did not contain report heading: {stdout}"
    );
}
