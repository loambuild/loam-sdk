use crate::util::TestEnv;

#[test]
fn build_command_runs_init() {
    TestEnv::from("soroban-init-boilerplate", |env| {
        env.set_environments_toml(
            r#"
development.accounts = [
{ name = "alice" },
{ name = "bob" },
]

[development.network]
rpc-url = "http://localhost:8000/rpc"
network-passphrase = "Standalone Network ; February 2017"

[development.contracts]
hello_world.client = false
soroban_increment_contract.client = false
soroban_custom_types_contract.client = false
soroban_auth_contract.client = false

[development.contracts.soroban_token_contract]
client = true
init = """
initialize --symbol ABND --decimal 7 --name abundance --admin alice
mint --amount 2000000 --to alice
"""
"#,
        );

        let output = env
            .loam_env("development", true)
            .output()
            .expect("Failed to execute command");

        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        // ensure the invoke commands are run with the proper source account
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr)
            .contains(" -- initialize --symbol ABND --decimal 7 --name abundance --admin alice"));
        assert!(String::from_utf8_lossy(&output.stderr)
            .contains(" -- mint --amount 2000000 --to alice"));
        assert!(String::from_utf8_lossy(&output.stderr).contains(
            "✅ Initialization script for \"soroban_token_contract\" completed successfully"
        ));
        // ensure setting STELLAR_ACCOUNT works
        env.set_environments_toml(
            r#"
development.accounts = [
{ name = "alice" },
{ name = "bob" },
]

[development.network]
rpc-url = "http://localhost:8000/rpc"
network-passphrase = "Standalone Network ; February 2017"

[development.contracts]
hello_world.client = false
soroban_increment_contract.client = false
soroban_custom_types_contract.client = false
soroban_auth_contract.client = false

[development.contracts.soroban_token_contract]
client = true
init = """
STELLAR_ACCOUNT=bob initialize --symbol ABND --decimal 7 --name abundance --admin bob 
STELLAR_ACCOUNT=bob mint --amount 2000000 --to bob 
"""
"#,
        );
        let output = env
            .loam_env("development", true)
            .output()
            .expect("Failed to execute command");

        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        // ensure the invoke commands are run with the proper source account
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr)
            .contains("--source-account bob -- initialize --symbol ABND --decimal 7 --name abundance --admin bob"));
        assert!(String::from_utf8_lossy(&output.stderr)
            .contains("--source-account bob -- mint --amount 2000000 --to bob"));
        assert!(String::from_utf8_lossy(&output.stderr).contains(
            "✅ Initialization script for \"soroban_token_contract\" completed successfully"
        ));
    });
}

#[test]
fn init_handles_quotations_and_subcommands_in_script() {
    TestEnv::from("soroban-init-boilerplate", |env| {
        let binary_path = env
            .find_binary("stellar")
            .expect("Stellar binary not found. Test cannot proceed.");

        let binary_path_str = binary_path.to_string_lossy();
        env.set_environments_toml(&format!(
            r#"
    development.accounts = [
    {{ name = "me" }},
    ]

    [development.network]
    rpc-url = "http://localhost:8000/rpc"
    network-passphrase = "Standalone Network ; February 2017"

    [development.contracts]
    hello_world.client = false
    soroban_increment_contract.client = false
    soroban_auth_contract.client = false
    soroban_token_contract.client = false

    [development.contracts.soroban_custom_types_contract]
    client = true
    init = """
    test_init --resolution 300000 --assets '[{{"Stellar": "$({} contract id asset --asset native)"}} ]' --decimals 14 --base '{{"Stellar":"$({} contract id asset --asset native)"}}'
    """
    "#,
            binary_path_str, binary_path_str
        ));

        let output = env
            .loam_env("development", true)
            .output()
            .expect("Failed to execute command");

        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

        // Ensure the command executed successfully
        assert!(output.status.success());

        // Check for the presence of the initialization commands in the output
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(" -- test_init --resolution 300000")
        );

        // Check for successful initialization message
        assert!(String::from_utf8_lossy(&output.stderr).contains(
            "✅ Initialization script for \"soroban_custom_types_contract\" completed successfully"
        ));
    })
}
