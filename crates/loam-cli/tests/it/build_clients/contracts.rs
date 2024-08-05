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
development.accounts = [
    {{ name = "alice" }},
]

[development.network]
rpc-url = "http://localhost:8000/rpc"
network-passphrase = "Standalone Network ; February 2017"

[development.contracts]
{}
"#,
                contracts
                    .iter()
                    .map(|c| format!("{c}.client = true"))
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

            // check that contracts are actually deployed, bound, and imported
            assert!(env.cwd.join(format!("packages/{c}")).exists());
            assert!(env.cwd.join(format!("src/contracts/{c}.ts")).exists());
        }
    });
}

#[test]
fn contracts_built_by_default() {
    let contracts = [
        "soroban_auth_contract",
        "soroban_custom_types_contract",
        "hello_world",
        "soroban_increment_contract",
    ];
    TestEnv::from("soroban-init-boilerplate", |env| {
        env.set_environments_toml(
            r#"
development.accounts = [
    { name = "alice" },
]

[development.network]
rpc-url = "http://localhost:8000/rpc"
network-passphrase = "Standalone Network ; February 2017"

"#,
        );
        let stderr = env.loam("build").assert().success().stderr_as_str();
        assert!(stderr.contains("creating keys for \"alice\"\n"));
        assert!(stderr.contains("using network at http://localhost:8000/rpc\n"));

        for c in contracts {
            assert!(stderr.contains(&format!("installing \"{c}\" wasm bytecode on-chain")));
            assert!(stderr.contains(&format!("instantiating \"{c}\" smart contract")));
            assert!(stderr.contains(&format!("binding \"{c}\" contract")));
            assert!(stderr.contains(&format!("importing \"{c}\" contract")));

            // check that contracts are actually deployed, bound, and imported
            assert!(env.cwd.join(format!("packages/{c}")).exists());
            assert!(env.cwd.join(format!("src/contracts/{c}.ts")).exists());
        }
    });
}

#[test]
fn contract_with_bad_name_prints_useful_error() {
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
hello.client = true
soroban_increment_contract.client = false
soroban_custom_types_contract.client = false
soroban_auth_contract.client = false
soroban_token_contract.client = false
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
hello_world.client = true
soroban_increment_contract.client = false
soroban_custom_types_contract.client = false
soroban_auth_contract.client = false
soroban_token_contract.client = false
"#,
        );

        let output = env
            .loam_env("development", true)
            .output()
            .expect("Failed to execute command");

        //println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        // ensure it imports
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr)
            .contains("🍽️ importing \"hello_world\" contract"));

        let output2 = env
            .loam_env("development", false)
            .output()
            .expect("Failed to execute command");

        //println!("stderr: {}", String::from_utf8_lossy(&output2.stderr));
        // ensure alias retrieval works
        assert!(output2.status.success());
        assert!(String::from_utf8_lossy(&output2.stderr)
            .contains("✅ Contract \"hello_world\" is up to date"));

        let output3 = env
            .loam_env("development", true)
            .output()
            .expect("Failed to execute command");

        //println!("stderr: {}", String::from_utf8_lossy(&output3.stderr));
        //println!("stdout: {}", String::from_utf8_lossy(&output3.stdout));
        // ensure contract hash change check works, should update in dev mode
        assert!(output3.status.success());
        let message = String::from_utf8_lossy(&output3.stderr);
        assert!(message.contains("🔄 Updating contract \"hello_world\""));
        let Some(contract_id) = extract_contract_id(&message) else {
            panic!("Could not find contract ID in stderr");
        };
        env.set_environments_toml(format!(
            r#"
production.accounts = [
    {{ name = "alice" }},
]

[production.network]
rpc-url = "http://localhost:8000/rpc"
network-passphrase = "Standalone Network ; February 2017"

[production.contracts]
{}.client = true
"#,
            contract_id
        ));

        // ensure production can identify via contract ID
        env.loam_build("production", true).assert().success();

        env.set_environments_toml(
            r#"
production.accounts = [
    { name = "alice" },
]

[production.network]
rpc-url = "http://localhost:8000/rpc"
network-passphrase = "Standalone Network ; February 2017"

[production.contracts]
hello_world.client = true
soroban_custom_types_contract.client = false
soroban_auth_contract.client = false
soroban_token_contract.client = false
"#,
        );

        let output4 = env
            .loam_build("production", true)
            .output()
            .expect("Failed to execute command");

        //println!("stderr: {}", String::from_utf8_lossy(&output4.stderr));
        // ensure contract hash change check works, should throw error in production
        assert!(!output4.status.success());
        assert!(String::from_utf8_lossy(&output4.stderr)
            .contains("️Contract must be identified by its ID in production or staging"));
    });
}

fn extract_contract_id(stderr: &str) -> Option<String> {
    stderr
        .lines()
        .find(|line| line.contains("contract_id:"))
        .and_then(|line| {
            line.split_whitespace()
                .last()
                .map(|id| id.trim().to_string())
        })
}
