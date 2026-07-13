/*
 * Copyright (c) 2026 DevEtte.
 *
 * This project is dual-licensed under both the MIT License and the
 * Apache License, Version 2.0 (the "License"). You may not use this
 * file except in compliance with one of these licenses.
 *
 * You may obtain a copy of the Licenses at:
 * - MIT: https://opensource.org
 * - Apache 2.0: http://apache.org
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the licenses.
 */

//! Integration tests for the `jsonette` CLI binary.
//!
//! Validates process input/output pipelines, command options overrides, config management,
//! and stderr formatting using `assert_cmd` and isolated environment directories.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

/// Helper function to instantiate the `jsonette` command with isolated environment variables.
///
/// Ensures the configuration dotfile is loaded/saved inside a temporary directory
/// to avoid overriding the user's real home folder preferences.
fn jsonette_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("jsonette").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path())
        .env("LOCALAPPDATA", temp_dir.path());
    cmd
}

/// **Test Case**: Pipeline formatting via standard input.
#[test]
fn test_cli_format_stdin() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);

    cmd.arg("format")
        .write_stdin(r#"{"b":2,"a":1}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"b\": 2"))
        .stdout(predicate::str::contains("\"a\": 1"));
}

/// **Test Case**: Key sorting override on formatting.
#[test]
fn test_cli_format_sort_keys() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);

    // Default configuration has sort_keys disabled
    cmd.arg("format")
        .arg("--sort-keys")
        .arg("true")
        .write_stdin(r#"{"b":2,"a":1}"#)
        .assert()
        .success()
        .stdout(predicate::str::starts_with("{\n  \"a\": 1,\n  \"b\": 2\n}"));
}

/// **Test Case**: Minified JSON output.
#[test]
fn test_cli_format_minify() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);

    cmd.arg("format")
        .arg("--minify")
        .write_stdin(
            r#"{
  "b": 2,
  "a": 1
}"#,
        )
        .assert()
        .success()
        .stdout(predicate::str::starts_with("{\"b\":2,\"a\":1}"));
}

/// **Test Case**: Formatting an input file.
#[test]
fn test_cli_format_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.json");
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(r#"{"x":1}"#.as_bytes()).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("format")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("{\n  \"x\": 1\n}"));
}

/// **Test Case**: In-place formatting of files.
#[test]
fn test_cli_format_inplace() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.json");
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(r#"{"x":1}"#.as_bytes()).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("format")
        .arg(&file_path)
        .arg("--in-place")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let content = fs::read_to_string(file_path).unwrap();
    assert_eq!(content, "{\n  \"x\": 1\n}\n");
}

/// **Test Case**: Compiler-style error diagnostic pointing.
#[test]
fn test_cli_format_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);

    cmd.arg("format")
        .write_stdin(r#"{"a": 1"#)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Error in <stdin>:1:7"))
        .stderr(predicate::str::contains("  |"))
        .stderr(predicate::str::contains("1 | {\"a\": 1"))
        .stderr(predicate::str::contains("  |       ^"));
}

/// **Test Case**: Config list, config set, and config get pipeline.
#[test]
fn test_cli_config_management() {
    let temp_dir = TempDir::new().unwrap();

    // 1. Verify default config listing works
    let mut list_cmd = jsonette_cmd(&temp_dir);
    list_cmd
        .arg("config")
        .arg("list")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"use_tabs\": false")
                .and(predicate::str::contains("\"indent\": 2")),
        );

    // 2. Modify global indent size to 4
    let mut set_cmd = jsonette_cmd(&temp_dir);
    set_cmd
        .arg("config")
        .arg("set")
        .arg("format.indent")
        .arg("4")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Configuration updated: format.indent = 4",
        ));

    // 3. Query the updated key
    let mut get_cmd = jsonette_cmd(&temp_dir);
    get_cmd
        .arg("config")
        .arg("get")
        .arg("format.indent")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("4"));

    // 4. Verify formatting uses the new global indent size of 4
    let mut format_cmd = jsonette_cmd(&temp_dir);
    format_cmd
        .arg("format")
        .write_stdin(r#"{"a":1}"#)
        .assert()
        .success()
        .stdout(predicate::str::starts_with("{\n    \"a\": 1\n}"));
}
