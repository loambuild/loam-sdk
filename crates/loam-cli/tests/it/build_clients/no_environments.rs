use assert_cmd::Command;

#[test]
fn no_environments_toml_gives_helpful_message() {
    Command::cargo_bin("loam")
        .unwrap()
        .current_dir(
            env!("CARGO_MANIFEST_DIR").to_string() +
            "/tests/fixtures/no_environments",
        )
        .arg("build-clients")
        .assert()
        .success()
        .stdout("No environments.toml; nothing to do\n");
}
