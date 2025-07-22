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
                r#"```{lang}
{markdown}
```
"#
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
fn test_rewrites_code_block() {
    let env = TestEnv::new(
        "echo hello",
        "sh",
        r#"
        [presets.shell]
        language = "sh"
        command = ["echo", "hello"]
        input_mode = "stdin"
        output_mode = "replace"
        "#,
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
        r#"
        [presets.shell]
        language = "sh"
        command = ["echo", "hello"]
        input_mode = "stdin"
        output_mode = "replace"
        "#,
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
        r#"
        [presets.bad-sh]
        language = "sh"
        command = ["sh", "-c", "exit 123"]
        input_mode = "stdin"
        output_mode = "check"
        "#,
    );

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--config",
        env.cfg_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(
        "The command `sh -c exit 123` returned a non-zero exit status (123) for preset `bad-sh` in "
    ));
}

#[test]
fn test_unsupported_language_is_skipped() {
    let env = TestEnv::new(
        "console.log('hi');",
        "javascript",
        r#"
        [presets.shell]
        language = "sh"
        command = ["echo", "hello"]
        input_mode = "stdin"
        output_mode = "replace"
        "#,
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
        r#"
        [presets.shell]
        language = "sh"
        command = ["echo", "hello"]
        input_mode = "stdin"
        output_mode = "replace"
        "#,
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
        r#"
        [presets.shell]
        language = "sh"
        command = ["sh", "-c", "exit 0"]
        input_mode = "stdin"
        output_mode = "check"
        "#,
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
fn test_check_mode_fails_on_change_but_does_not_write() {
    let env = TestEnv::new(
        "echo outdated",
        "sh",
        r#"
        [presets.shell]
        language = "sh"
        command = ["echo", "hello"]
        input_mode = "stdin"
        output_mode = "replace"
        "#,
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
fn test_output_mode_check_fails_on_error() {
    let env = TestEnv::new(
        "echo broken",
        "sh",
        r#"
        [presets.shell]
        language = "sh"
        command = ["sh", "-c", "exit 1"]
        input_mode = "stdin"
        output_mode = "check"
        "#,
    );

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--config",
        env.cfg_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("returned a non-zero exit status"));
}

#[test]
fn test_no_code_blocks_means_no_changes() {
    let env = TestEnv::from_raw_markdown(
        r#"
# Hello world

This is a test document with no code blocks.

Enjoy!
        "#,
        r#"
        [presets.shell]
        language = "sh"
        command = ["echo", "hello"]
        "#,
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
fn test_file_placeholders_are_expanded() {
    let env = TestEnv::new(
        "echo hello",
        "sh",
        r#"
        [presets.test]
        language = "sh"
        command = ["sh", "-c", "echo File: {file}, Basename: {basename}, Dir: {dirname}, Suffix: {suffix}"]
        input_mode = "file"
        output_mode = "replace"
        "#,
    );

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--config",
        env.cfg_path.to_str().unwrap(),
    ]);

    assert!(output.status.success());
    let updated = std::fs::read_to_string(&env.md_path).unwrap();
    assert!(updated.contains("File:"));
    assert!(updated.contains("Suffix:"));
}

#[test]
fn test_invalid_config_fails_with_error() {
    let env = TestEnv::from_raw_markdown(
        "```sh\necho hi\n```",
        r#"
        this is not valid TOML
        "#,
    );

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--config",
        env.cfg_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("TOML parse error at line"));
}

#[test]
fn test_block_with_unknown_language_skipped() {
    let env = TestEnv::new(
        "print('hello')",
        "foobar",
        r#"
        [presets.py]
        language = "python"
        command = ["echo", "hi"]
        "#,
    );

    let output = env.run(&[
        env.md_path.to_str().unwrap(),
        "--config",
        env.cfg_path.to_str().unwrap(),
    ]);

    assert!(output.status.success());
    let contents = std::fs::read_to_string(&env.md_path).unwrap();
    assert!(contents.contains("print('hello')"));
}

#[test]
fn test_multiple_files_in_dir_one_with_issue() {
    use std::fs;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let file_with_issue = dir.path().join("file_with_issue.md");
    let file_ok = dir.path().join("file_ok.md");
    let config_path = dir.path().join("config.toml");

    fs::write(&file_with_issue, "```sh\necho something-wrong\n```").unwrap();

    fs::write(&file_ok, "```sh\nhello\n```").unwrap();

    fs::write(
        &config_path,
        r#"
        [presets.shell]
        language = "sh"
        command = ["echo", "hello"]
        input_mode = "stdin"
        output_mode = "replace"
        "#,
    )
    .unwrap();

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            dir.path().to_str().unwrap(),
            "--check",
            "--config",
            config_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());

    let content_issue = fs::read_to_string(&file_with_issue).unwrap();
    assert!(content_issue.contains("something-wrong"));

    let content_ok = fs::read_to_string(&file_ok).unwrap();
    assert!(content_ok.contains("hello"));
}

#[test]
fn test_multiple_files_in_dir_one_is_fixed() {
    use std::fs;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let file_with_issue = dir.path().join("file_with_issue.md");
    let file_ok = dir.path().join("file_ok.md");
    let config_path = dir.path().join("config.toml");

    fs::write(&file_with_issue, "```sh\necho something-wrong\n```").unwrap();

    fs::write(&file_ok, "```sh\nhello\n```").unwrap();

    fs::write(
        &config_path,
        r#"
        [presets.shell]
        language = "sh"
        command = ["echo", "hello"]
        input_mode = "stdin"
        output_mode = "replace"
        "#,
    )
    .unwrap();

    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            dir.path().to_str().unwrap(),
            "--config",
            config_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success());

    let updated_issue = fs::read_to_string(&file_with_issue).unwrap();
    assert!(updated_issue.contains("hello"));
    assert!(!updated_issue.contains("something-wrong"));

    let updated_ok = fs::read_to_string(&file_ok).unwrap();
    assert_eq!(updated_ok.trim(), "```sh\nhello\n```");
}
