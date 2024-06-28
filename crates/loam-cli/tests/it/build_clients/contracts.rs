use crate::util::{AssertExt, TestEnv};

#[test]
fn contracts_built() {
    let contracts = [
        "soroban_auth_contract",
        "soroban_custom_types_contract",
        "hello_world",
        "soroban_increment_contract",
    ];
    TestEnv::from("soroban-init-boilerplate", |env| {
        env.set_environments_toml(
            format!(
                r#"
production.accounts = [
    {{ name = "alice" }},
]

[production.network]
rpc-url = "http://localhost:8000/rpc"
network-passphrase = "Standalone Network ; February 2017"

[production.contracts]
{}
"#,
                contracts
                    .iter()
                    .map(|c| format!("{c}.workspace = true"))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
            .as_str(),
        );

        let stderr = env.loam("build").assert().success().stderr_as_str();
        assert!(stderr.contains("creating keys for \"alice\"\n"));
        assert!(stderr.contains("using network at http://localhost:8000/rpc\n"));

        for c in contracts {
            assert!(stderr.contains(&format!("installing \"{c}\" wasm bytecode on-chain")));
            assert!(stderr.contains(&format!("instantiating \"{c}\" smart contract")));
            assert!(stderr.contains(&format!("binding \"{c}\" contract")));
            assert!(stderr.contains(&format!("importing \"{c}\" contract")));
        }

        // check that contracts are actually deployed, bound, and imported
        for contract in contracts {
            assert!(env.cwd.join(format!("packages/{}", contract)).exists());
            assert!(env
                .cwd
                .join(format!("src/contracts/{}.ts", contract))
                .exists());
        }
    });
}

#[test]
fn contract_with_bad_name_prints_useful_error() {
    TestEnv::from("soroban-init-boilerplate", |env| {
        env.set_environments_toml(
            r#"
production.accounts = [
    { name = "alice" },
]

[production.network]
rpc-url = "http://localhost:8000/rpc"
network-passphrase = "Standalone Network ; February 2017"

[production.contracts]
hello.workspace = true
"#,
        );

        env.loam("build")
            .assert()
            .failure()
            .stderr(predicates::str::contains("No contract named \"hello\""));
    });
}

#[test]
fn contract_alias_skips_install() {
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
hello_world.workspace = true
"#,
        );
        let output = env
            .loam_env("build", "development")
            .output()
            .expect("Failed to execute command");

        // ensure it imports
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr)
            .contains("🍽️ importing \"hello_world\" contract"));

        let output2 = env
            .loam_env("build", "development")
            .output()
            .expect("Failed to execute command");

        // ensure alias retrieval works
        assert!(output2.status.success());
        assert!(String::from_utf8_lossy(&output2.stderr)
            .contains("✅ Contract \"hello_world\" is up to date"));

        let file = "contracts/increment/src/lib.rs";
        let file_replaced = "contracts/hello_world/src/lib.rs";
        env.replace_file(file, file_replaced);

        let output3 = env
            .loam_env("build", "development")
            .output()
            .expect("Failed to execute command");

        
        println!("stderr: {}",String::from_utf8_lossy(&output3.stderr));
        // ensure contract hash change check works, should update in dev mode
        assert!(output3.status.success());
        assert!(String::from_utf8_lossy(&output3.stderr)
            .contains("🔄 Updating contract \"hello_world\""));

        env.set_environments_toml(
            r#"
production.accounts = [
    { name = "alice" },
]

[production.network]
rpc-url = "http://localhost:8000/rpc"
network-passphrase = "Standalone Network ; February 2017"

[production.contracts]
hello_world.workspace = true
"#,
        );
        env.replace_file("contracts/auth/src/lib.rs", file_replaced);

        let output4 = env
            .loam("build")
            .output()
            .expect("Failed to execute command");

        // ensure contract hash change check works, should throw error in production
        assert!(!output4.status.success());
        assert!(String::from_utf8_lossy(&output4.stderr)
            .contains("⛔ ️Contract update not allowed in production for \"hello_world\""));
    });
}
