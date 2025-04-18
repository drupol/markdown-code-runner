#[test]
fn test_all_scenarios_with_input_reset_and_cleanup() {
    use std::fs;
    use std::path::PathBuf;

    let scenarios_root = PathBuf::from("tests/fixtures/scenarios");

    for entry in fs::read_dir(&scenarios_root).unwrap() {
        let scenario_path = entry.unwrap().path();
        if !scenario_path.is_dir() {
            continue;
        }

        let name = scenario_path.file_name().unwrap().to_string_lossy();

        let config = scenario_path.join("config.toml");
        let input = scenario_path.join("test.input.md");
        let test = scenario_path.join("test.md");
        let expected = scenario_path.join("test.expected.md");

        assert!(config.exists(), "Missing config.toml in `{}`", name);
        assert!(input.exists(), "Missing test.input.md in `{}`", name);
        assert!(expected.exists(), "Missing test.expected.md in `{}`", name);

        fs::copy(&input, &test).expect("Failed to copy test.input.md -> test.md");

        let output = std::process::Command::new("cargo")
            .args([
                "run",
                "--quiet",
                "--",
                test.to_str().unwrap(),
                "--config",
                config.to_str().unwrap(),
            ])
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "mdcr failed in `{}`\nstdout: {}\nstderr: {}",
            name,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );

        let actual = fs::read_to_string(&test).unwrap();
        let expected_str = fs::read_to_string(&expected).unwrap();

        fs::remove_file(&test).expect("Failed to delete test.md");

        assert_eq!(actual.trim(), expected_str.trim(), "Mismatch in `{}`", name);
    }
}
