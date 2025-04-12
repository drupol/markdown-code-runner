mod helpers {
    use std::fs;
    use std::path::PathBuf;

    pub struct TestEnv {
        pub md_path: PathBuf,
        pub cfg_path: PathBuf,
    }

    impl TestEnv {
        pub fn new(markdown: &str, lang: &str, config_json: &str) -> Self {
            let markdown = format!(
                r#"```{}
{}
```
"#,
                lang, markdown
            );

            Self::from_raw_markdown(&markdown, config_json)
        }

        pub fn from_raw_markdown(markdown: &str, config_json: &str) -> Self {
            let dir = tempfile::Builder::new()
                .prefix("mdcr-test-")
                .tempdir()
                .unwrap()
                .into_path();
            let md_path = dir.join("test.md");
            let cfg_path = dir.join("config.json");

            fs::write(&md_path, markdown).unwrap();
            fs::write(&cfg_path, config_json).unwrap();

            Self { md_path, cfg_path }
        }

        pub fn run(&self, args: &[&str]) -> std::process::Output {
            let _ = env_logger::builder().is_test(true).try_init();

            let mut full_args = vec!["run", "--quiet", "--"];
            full_args.extend_from_slice(args);
            std::process::Command::new("cargo")
                .args(full_args)
                .output()
                .unwrap()
        }
    }
}

use helpers::TestEnv;

#[test]
fn test_dry_run_does_not_fail_on_error() {
    let env = TestEnv::new(
        "echo error",
        "sh",
        r#"{
            "presets": {
                "shell": {
                    "language": "sh",
                    "command": [ "sh", "-c", "'exit 42'"],
                    "input_mode": "stdin",
                    "mode": "check"
                }
            }
        }"#,
    );

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--config",
        env.cfg_path.to_str().unwrap(),
        "--dry-run",
        "--log",
        "debug",
    ]);

    assert!(output.status.success());
}

#[test]
fn test_rewrites_code_block() {
    let env = TestEnv::new(
        "echo hello",
        "sh",
        r#"{
            "presets": {
                "shell": {
                    "language": "sh",
                    "command": [ "echo", "hello"],
                    "input_mode": "stdin",
                    "mode": "replace"
                }
            }
        }"#,
    );

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--config",
        env.cfg_path.to_str().unwrap(),
    ]);

    assert!(output.status.success());
    let updated = std::fs::read_to_string(&env.md_path).unwrap();
    assert!(updated.contains("hello"));
}

#[test]
fn test_check_mode_detects_differences() {
    let env = TestEnv::new(
        "echo something-else",
        "sh",
        r#"{
            "presets": {
                "shell": {
                    "language": "sh",
                    "command": [ "echo", "hello"],
                    "input_mode": "stdin",
                    "mode": "replace"
                }
            }
        }"#,
    );

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--check",
        "--log",
        "debug",
        "--config",
        env.cfg_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Code block mismatch"));
}

#[test]
fn test_prints_warning_on_failure() {
    let env = TestEnv::new(
        "echo bad",
        "sh",
        r#"{
            "presets": {
                "bad-sh": {
                    "language": "sh",
                    "command": [ "sh", "-c", "'exit 1'"],
                    "input_mode": "stdin",
                    "mode": "check"
                }
            }
        }"#,
    );

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--config",
        env.cfg_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error executing command for preset `bad-sh`"));
}

#[test]
fn test_unsupported_language_is_skipped() {
    let env = TestEnv::new(
        "console.log('hi');",
        "javascript",
        r#"{
            "presets": {
                "shell": {
                    "language": "sh",
                    "command": ["echo", "hello"],
                    "input_mode": "stdin",
                    "mode": "replace"
                }
            }
        }"#,
    );

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--config",
        env.cfg_path.to_str().unwrap(),
    ]);

    assert!(output.status.success());
    let updated = std::fs::read_to_string(&env.md_path).unwrap();
    assert!(updated.contains("console.log('hi');"));
}

#[test]
fn test_check_mode_no_changes_returns_zero() {
    let env = TestEnv::new(
        "hello",
        "sh",
        r#"{
            "presets": {
                "shell": {
                    "language": "sh",
                    "command": [ "echo", "hello"],
                    "input_mode": "stdin",
                    "mode": "replace"
                }
            }
        }"#,
    );

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--check",
        "--config",
        env.cfg_path.to_str().unwrap(),
    ]);

    assert!(output.status.success());
}

#[test]
fn test_multiple_code_blocks() {
    let env = TestEnv::from_raw_markdown(
        r#"
# bar

```sh
echo one
```

# foo

```sh
echo two
```
        "#,
        r#"{
            "presets": {
                "shell": {
                    "language": "sh",
                    "command": [ "sh", "-c", "exit 0"],
                    "input_mode": "stdin",
                    "mode": "check"
                }
            }
        }"#,
    );

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--check",
        "--config",
        env.cfg_path.to_str().unwrap(),
    ]);

    assert!(output.status.success());
}

#[test]
fn test_dry_run_allows_command_failure() {
    let env = TestEnv::new(
        "Hello",
        "sh",
        r#"{
            "presets": {
                "fail": {
                    "language": "sh",
                    "command": [ "sh", "-c", "exit 1"],
                    "input_mode": "stdin",
                    "mode": "check"
                }
            }
        }"#,
    );

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--config",
        env.cfg_path.to_str().unwrap(),
        "--dry-run",
    ]);

    assert!(
        output.status.success(),
        "Dry-run should not fail, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_check_mode_fails_on_change_but_does_not_write() {
    let env = TestEnv::new(
        "echo outdated",
        "sh",
        r#"{
            "presets": {
                "shell": {
                    "language": "sh",
                    "command": ["echo", "hello"],
                    "input_mode": "stdin",
                    "mode": "replace"
                }
            }
        }"#,
    );

    let original = std::fs::read_to_string(&env.md_path).unwrap();

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--check",
        "--config",
        env.cfg_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Code block mismatch detected"),);

    let after = std::fs::read_to_string(&env.md_path).unwrap();
    assert_eq!(original, after, "Check mode must not alter the file");
}

#[test]
fn test_preserves_indentation_in_code_block() {
    let markdown = r#"
# Title

- Foo
  ```sh
  Foobar
  ```
- Bar
    "#;

    let config = r#"{
        "presets": {
            "shell": {
                "language": "sh",
                "command": ["echo", "Hello"],
                "input_mode": "stdin",
                "mode": "replace"
            }
        }
    }"#;

    let env = TestEnv::from_raw_markdown(markdown, config);

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--config",
        env.cfg_path.to_str().unwrap(),
    ]);

    assert!(output.status.success());

    let updated = std::fs::read_to_string(&env.md_path).unwrap();
    let expected = r#"
# Title

- Foo
  ```sh
  Hello
  ```
- Bar
    "#;

    assert_eq!(updated.trim(), expected.trim());
}

#[test]
fn test_dry_run_outputs_warning_but_does_not_write() {
    let env = TestEnv::new(
        "echo hello",
        "sh",
        r#"{
            "presets": {
                "shell": {
                    "language": "sh",
                    "command": ["echo", "changed"],
                    "input_mode": "stdin",
                    "mode": "replace"
                }
            }
        }"#,
    );

    let original = std::fs::read_to_string(&env.md_path).unwrap();

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--config",
        env.cfg_path.to_str().unwrap(),
        "--dry-run",
        "--log",
        "debug",
    ]);

    assert!(output.status.success());

    let final_content = std::fs::read_to_string(&env.md_path).unwrap();
    assert_eq!(original, final_content);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Processing preset `shell` for language `sh` in mode `Replace`"),);
}
