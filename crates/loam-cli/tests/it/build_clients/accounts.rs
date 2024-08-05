use crate::util::{AssertExt, TestEnv};

#[test]
fn create_two_accounts() {
    TestEnv::from("soroban-init-boilerplate", |env| {
        env.set_environments_toml(r#"
[development]
network = { rpc-url = "http://localhost:8000/rpc", network-passphrase = "Standalone Network ; February 2017"}

accounts = [
    "alice",
    { name = "bob" },
]
[development.contracts]
hello_world.client = false
soroban_increment_contract.client = false
soroban_custom_types_contract.client = false
soroban_auth_contract.client = false
soroban_token_contract.client = false
"#);

        let stderr = env.loam("build").assert().success().stderr_as_str();
        assert!(stderr.contains("creating keys for \"alice\""));
        assert!(stderr.contains("creating keys for \"bob\""));
        assert!(env.cwd.join(".soroban/identity/alice.toml").exists());
        assert!(env.cwd.join(".soroban/identity/bob.toml").exists());

        // check that they dont get overwritten if build is run again
        let stderr = env.loam("build").assert().success().stderr_as_str();
        assert!(stderr.contains("account \"alice\" already exists"));
        assert!(stderr.contains("account \"bob\" already exists"));

        // check that they're actually funded
        let stderr = env
            .soroban("keys")
            .args([
                "fund",
                "alice",
                "--network-passphrase",
                "\"Standalone Network ; February 2017\"",
                "--rpc-url",
                "http://localhost:8000/soroban/rpc",
            ])
            .assert()
            .success()
            .stderr_as_str();
        assert!(stderr.contains("Account already exists"));
    });
}
