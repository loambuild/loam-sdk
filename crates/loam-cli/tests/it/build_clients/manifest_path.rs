use crate::util::{AssertExt, TestEnv};

#[test]
fn uses_manifest_path_for_build_command() {
    TestEnv::from("soroban-init-boilerplate", |env| {
        env.set_environments_toml(
            r#"
development.accounts = [
    { name = "alice" },
]

[development.network]
rpc-url = "http://localhost:8000/rpc"
network-passphrase = "Standalone Network ; February 2017"

[development.contracts]
hello_world.client = false
soroban_increment_contract.client = false
soroban_custom_types_contract.client = false
soroban_auth_contract.client = false
soroban_token_contract.client = false
"#,
        );

        let stderr = env
            .loam("build")
            .current_dir(env.cwd.join(".."))
            .args(["--manifest-path", "./soroban-init-boilerplate/Cargo.toml"])
            .assert()
            .success()
            .stderr_as_str();

        assert!(stderr.contains("🌐 using network at http://localhost:8000/rpc\n"));
    });
}
