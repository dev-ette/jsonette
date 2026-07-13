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

/// **Test Case**: Output option generating formatted output to a new file.
#[test]
fn test_cli_format_output() {
    let temp_dir = TempDir::new().unwrap();
    let out_path = temp_dir.path().join("out.json");

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("format")
        .arg("--output")
        .arg(&out_path)
        .write_stdin(r#"{"a":1}"#)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let content = fs::read_to_string(out_path).unwrap();
    assert_eq!(content, "{\n  \"a\": 1\n}\n");
}

/// **Test Case**: Conflicting output and in-place arguments.
#[test]
fn test_cli_format_output_and_inplace_conflict() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("format")
        .arg("dummy.json")
        .arg("--output")
        .arg("out.json")
        .arg("--in-place")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "Error: Cannot specify both --in-place and --output file.",
        ));
}

/// **Test Case**: Generation of shell autocompletion scripts.
#[test]
fn test_cli_completions() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("completions")
        .arg("zsh")
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef jsonette"))
        .stdout(predicate::str::contains("format"))
        .stdout(predicate::str::contains("query"))
        .stdout(predicate::str::contains("config"));
}

// ──────────────────────────── query subcommand ────────────────────────────────

/// **Test Case**: Dot-Key Path Extracts Named Property From File
///
/// ### Description
/// Verifies that `jsonette query` correctly evaluates a simple dot-key JSONPath
/// expression against a JSON file and prints the matched string value.
///
/// ### Test Procedure
/// 1. Write `{"name": "jsonette", "version": "0.1.0"}` to a temporary file.
/// 2. Run `jsonette query $.name <file>`.
///
/// ### Expected Result
/// Exits with code 0. Stdout contains the string `"jsonette"`.
#[test]
fn test_cli_query_dot_key() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"name": "jsonette", "version": "0.1.0"}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("query")
        .arg("$.name")
        .arg(json_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("jsonette"));
}

/// **Test Case**: Wildcard Path Collects All Matching Array Elements
///
/// ### Description
/// Verifies that a wildcard JSONPath expression traverses an array of objects
/// and collects the named property from every element in document order.
///
/// ### Test Procedure
/// 1. Write `{"users": [{"name": "Alice"}, {"name": "Bob"}]}` to a temporary file.
/// 2. Run `jsonette query $.users[*].name <file>`.
///
/// ### Expected Result
/// Exits with code 0. Stdout contains both `"Alice"` and `"Bob"`.
#[test]
fn test_cli_query_wildcard_array() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("users.json");
    fs::write(
        &json_file,
        r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#,
    )
    .unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("query")
        .arg("$.users[*].name")
        .arg(json_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob"));
}

/// **Test Case**: Non-Matching Path Prints Empty Array
///
/// ### Description
/// Verifies that a valid JSONPath expression that matches no nodes in the
/// document prints `[]` to stdout and exits successfully, consistent with
/// the RFC 9535 empty node-list convention.
///
/// ### Test Procedure
/// 1. Write `{"a": 1}` to a temporary file.
/// 2. Run `jsonette query $.nonexistent <file>`.
///
/// ### Expected Result
/// Exits with code 0. Stdout contains `[]`.
#[test]
fn test_cli_query_no_match_prints_empty_array() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"a": 1}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("query")
        .arg("$.nonexistent")
        .arg(json_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}

/// **Test Case**: Query Reads JSON Document From Standard Input
///
/// ### Description
/// Verifies that `jsonette query` accepts a JSON document on stdin when no
/// file argument is provided, matching the pipeline-friendly behaviour of the
/// format subcommand.
///
/// ### Test Procedure
/// 1. Pipe `{"x": 42}` on stdin.
/// 2. Run `jsonette query $.x` with no file argument.
///
/// ### Expected Result
/// Exits with code 0. Stdout contains the numeric value `42`.
#[test]
fn test_cli_query_stdin() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("query")
        .arg("$.x")
        .write_stdin(r#"{"x": 42}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

/// **Test Case**: Invalid JSONPath Expression Reports Error on Stderr
///
/// ### Description
/// Verifies that a syntactically invalid JSONPath expression is caught by the
/// pre-evaluation validation step and produces a clear error message on stderr
/// before any document processing occurs.
///
/// ### Test Procedure
/// 1. Pipe a valid JSON document on stdin.
/// 2. Supply an arbitrary non-JSONPath string as the path argument.
///
/// ### Expected Result
/// Exits with code 1. Stderr contains `"Error"`.
#[test]
fn test_cli_query_invalid_jsonpath_reports_error() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("query")
        .arg("NOT A VALID PATH")
        .write_stdin(r#"{"a": 1}"#)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

/// **Test Case**: Malformed JSON Document Reports Parse Error on Stderr
///
/// ### Description
/// Verifies that when the input document cannot be parsed as valid JSON, the
/// command prints diagnostics to stderr and exits with a non-zero exit code.
///
/// ### Test Procedure
/// 1. Pipe the string `"NOT VALID JSON"` on stdin.
/// 2. Run `jsonette query $.foo` with no file argument.
///
/// ### Expected Result
/// Exits with code 1. Stderr is non-empty.
#[test]
fn test_cli_query_invalid_json_reports_error() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("query")
        .arg("$.foo")
        .write_stdin("NOT VALID JSON")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

/// **Test Case**: Missing Input File Reports Error on Stderr
///
/// ### Description
/// Verifies that supplying a path to a file that does not exist causes the
/// command to report an I/O error on stderr and exit with a non-zero code.
///
/// ### Test Procedure
/// 1. Run `jsonette query $.foo /nonexistent/path/missing.json`.
///
/// ### Expected Result
/// Exits with code 1. Stderr contains `"Error"` or `"error"`.
#[test]
fn test_cli_query_missing_file_reports_error() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("query")
        .arg("$.foo")
        .arg("/nonexistent/path/missing.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error").or(predicate::str::contains("error")));
}

// ──────────────────────────── explore subcommand ──────────────────────────────

/// **Test Case**: Explore Root Object Prints Sorted Keys
///
/// ### Description
/// Verifies that exploring a JSON object outputs its keys sorted alphabetically.
///
/// ### Test Procedure
/// 1. Write `{"b": 2, "a": 1, "c": 3}` to a file.
/// 2. Run `jsonette explore '$' <file>`.
///
/// ### Expected Result
/// Exits with code 0. Stdout contains `a\nb\nc\n`.
#[test]
fn test_cli_explore_object_keys_sorted() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"b": 2, "a": 1, "c": 3}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("explore")
        .arg("$")
        .arg(json_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("a\nb\nc"));
}

/// **Test Case**: Explore Array Prints Length
///
/// ### Description
/// Verifies that exploring a JSON array outputs its length.
///
/// ### Test Procedure
/// 1. Write `[1, 2, 3]` to a file.
/// 2. Run `jsonette explore '$' <file>`.
///
/// ### Expected Result
/// Exits with code 0. Stdout contains `Length: 3 elements`.
#[test]
fn test_cli_explore_array_length() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"[1, 2, 3]"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("explore")
        .arg("$")
        .arg(json_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Length: 3 elements"));
}

/// **Test Case**: Explore With Single File Argument Defaults to Root Path
///
/// ### Description
/// Verifies the heuristic that if only one positional argument is provided and it's
/// a file, the CLI defaults the path to `$` rather than waiting on stdin.
///
/// ### Test Procedure
/// 1. Write `{"a": 1}` to a file.
/// 2. Run `jsonette explore <file>`.
///
/// ### Expected Result
/// Exits with code 0. Stdout contains the key `a`.
#[test]
fn test_cli_explore_single_file_argument() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"a": 1}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("explore")
        .arg(json_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("a\n"));
}

/// **Test Case**: Explore Object Filters Keys with Regex
///
/// ### Description
/// Verifies that exploring with `--regex` filters the returned keys.
///
/// ### Test Procedure
/// 1. Write `{"foo": 1, "bar": 2, "baz": 3}` to a file.
/// 2. Run `jsonette explore --regex '^ba' '$' <file>`.
///
/// ### Expected Result
/// Exits with code 0. Stdout contains `bar` and `baz` but not `foo`.
#[test]
fn test_cli_explore_object_regex_filter() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"foo": 1, "bar": 2, "baz": 3}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("explore")
        .arg("--regex")
        .arg("^ba")
        .arg("$")
        .arg(json_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("bar\nbaz"))
        .stdout(predicate::str::contains("foo").not());
}

/// **Test Case**: Explore With Invalid Regex Reports Error
///
/// ### Description
/// Verifies that an invalid regular expression string passed to `--regex` exits with an error.
#[test]
fn test_cli_explore_invalid_regex_error() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"a": 1}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("explore")
        .arg("--regex")
        .arg("[invalid")
        .arg("$")
        .arg(json_file.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid regex"));
}

/// **Test Case**: Explore With Invalid JSONPath Reports Error
#[test]
fn test_cli_explore_invalid_jsonpath_error() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"a": 1}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("explore")
        .arg("NOT_A_PATH")
        .arg(json_file.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

/// **Test Case**: Explore Where No Nodes Match
#[test]
fn test_cli_explore_no_match() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"a": 1}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("explore")
        .arg("$.b")
        .arg(json_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("No nodes matched"));
}

/// **Test Case**: Explore Matching a Primitive Node
#[test]
fn test_cli_explore_primitive_node() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"a": 42}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("explore")
        .arg("$.a")
        .arg(json_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Matched a number node"));
}

/// **Test Case**: Explore With Invalid JSON Reports Error
#[test]
fn test_cli_explore_invalid_json_reports_error() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("explore")
        .arg("$")
        .write_stdin("NOT VALID JSON")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

/// **Test Case**: Explore On Missing File Reports Error
#[test]
fn test_cli_explore_missing_file_reports_error() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("explore")
        .arg("$")
        .arg("/nonexistent/path/missing.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error").or(predicate::str::contains("error")));
}

/// **Test Case**: Explore Limits Output
///
/// ### Description
/// Verifies that exploring with `--limit` truncates the output and shows a "... more" message.
///
/// ### Test Procedure
/// 1. Write `{"a": 1, "b": 2, "c": 3}` to a file.
/// 2. Run `jsonette explore -n 1 '$' <file>`.
///
/// ### Expected Result
/// Exits with code 0. Stdout contains `a\n` and `... and 2 more keys`.
#[test]
fn test_cli_explore_object_limit() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"a": 1, "b": 2, "c": 3}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("explore")
        .arg("-n")
        .arg("1")
        .arg("$")
        .arg(json_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("a\n... and 2 more keys"));
}
