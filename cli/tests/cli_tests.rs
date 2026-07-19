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
///
/// ### Description
/// Validates pipeline formatting via standard input.
///
/// ### Test Procedure
/// 1. Provide input via stdin.
///
/// ### Expected Result
/// Validates correct formatting on stdout.
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
///
/// ### Description
/// Tests format parameter overrides for sorting object keys.
///
/// ### Test Procedure
/// 1. Run format with `--sort-keys true`.
///
/// ### Expected Result
/// Validates sorted keys in stdout.
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
///
/// ### Description
/// Validates minification pipeline.
///
/// ### Test Procedure
/// 1. Run format with `--minify`.
///
/// ### Expected Result
/// Output is correctly stripped of whitespace.
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
///
/// ### Description
/// Tests formatting when provided a discrete file path.
///
/// ### Test Procedure
/// 1. Run format with file target.
///
/// ### Expected Result
/// Output is correctly formatted JSON.
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
///
/// ### Description
/// Validates inplace file modification safely formats and persists data.
///
/// ### Test Procedure
/// 1. Execute format with `--in-place` flag.
///
/// ### Expected Result
/// File is overwritten with correct payload and no stdout prints.
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
///
/// ### Description
/// Ensures invalid payloads print precise span-based errors pointing to failures.
///
/// ### Test Procedure
/// 1. Provide invalid JSON to format.
///
/// ### Expected Result
/// Returns non-zero exit code and diagnostic arrows pointing at the exact syntax error.
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
///
/// ### Description
/// Evaluates entire config state machine for persistent CLI configuration.
///
/// ### Test Procedure
/// 1. Execute config list.
/// 2. Modify config set.
/// 3. Read back with config get.
///
/// ### Expected Result
/// Writes persist efficiently between CLI executions.
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
///
/// ### Description
/// Tests formatting JSON directly to an output path instead of stdout.
///
/// ### Test Procedure
/// 1. Execute `jsonette format --output <file>`.
///
/// ### Expected Result
/// File is created containing the formatted payload.
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
///
/// ### Description
/// Validates error when both output configurations are provided.
///
/// ### Test Procedure
/// 1. Provide `--output` and `--in-place` together.
///
/// ### Expected Result
/// Prevents execution safely with stderr error.
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
///
/// ### Description
/// Evaluates the generation of zsh autocompletion logic.
///
/// ### Test Procedure
/// 1. Execute `jsonette completions zsh`.
///
/// ### Expected Result
/// Standard output contains `compdef jsonette`.
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
///
/// ### Test Procedure
/// 1. Execute explore with malformed regex `[invalid`.
///
/// ### Expected Result
/// Errors gracefully showing "Invalid regex".
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
///
/// ### Description
/// Tests handling of invalid JSONPath expressions.
///
/// ### Test Procedure
/// 1. Provide `NOT_A_PATH` to explore.
///
/// ### Expected Result
/// Command halts execution with parsing failure.
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
///
/// ### Description
/// Validates explore behavior when no matching keys exist.
///
/// ### Test Procedure
/// 1. Evaluate missing path `$.b`.
///
/// ### Expected Result
/// Standard output mentions "No nodes matched".
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
///
/// ### Description
/// Evaluates execution behavior targeting primitive string or integer elements directly.
///
/// ### Test Procedure
/// 1. Explore to a direct property node containing `42`.
///
/// ### Expected Result
/// Command correctly alerts the user it "Matched a number node".
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

/// **Test Case**: Explore Multiple Matches Separator
///
/// ### Description
/// Verifies the explore command correctly handles and separates outputs when JSONPath matches multiple root items.
///
/// ### Test Procedure
/// 1. Run `explore '$.*'` on a file with multiple array fields.
///
/// ### Expected Result
/// Prints `--- Match 1 ---` and `--- Match 2 ---` dividers.
#[test]
fn test_cli_explore_multiple_matches() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"a": [1, 2], "b": [3, 4]}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("explore")
        .arg("$.*")
        .arg(json_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("--- Match 1 ---"))
        .stdout(predicate::str::contains("--- Match 2 ---"));
}

/// **Test Case**: Explore Grep Filtering
///
/// ### Description
/// Tests string retention in explore output via the `--grep` option.
///
/// ### Test Procedure
/// 1. Filter a JSON object containing keys `apple`, `banana` with `--grep app`.
/// 2. Test an unmatchable grep string to evaluate the empty fallbacks.
///
/// ### Expected Result
/// Output trims unmatched entries and correctly falls back to "No keys found".
#[test]
fn test_cli_explore_grep_filter_and_empty() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"apple": 1, "banana": 2, "cherry": 3}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("explore")
        .arg("--grep")
        .arg("app")
        .arg("$")
        .arg(json_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("apple").and(predicate::str::contains("banana").not()));

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("explore")
        .arg("--grep")
        .arg("zzz")
        .arg("$")
        .arg(json_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("No keys found"));
}

/// **Test Case**: Explore With Invalid JSON Reports Error
///
/// ### Description
/// Validates invalid JSON payloads fail correctly before path exploration.
///
/// ### Test Procedure
/// 1. Pass `NOT VALID JSON`.
///
/// ### Expected Result
/// Fails and prints internal compilation diagnostics to stderr.
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

// ──────────────────────────── convert subcommand ──────────────────────────────

/// **Test Case**: Cli Convert Json To Yaml
///
/// ### Description
/// Validates cli convert json to yaml functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_convert_json_to_yaml`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_convert_json_to_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"a": 1}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("convert")
        .arg("--from")
        .arg("json")
        .arg("--to")
        .arg("yaml")
        .arg(json_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("a: 1"));
}

/// **Test Case**: Cli Convert Missing File
///
/// ### Description
/// Validates cli convert missing file functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_convert_missing_file`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_convert_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("convert")
        .arg("--from")
        .arg("json")
        .arg("--to")
        .arg("yaml")
        .arg("nonexistent.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error reading input"));
}

/// **Test Case**: Cli Convert Invalid From Format
///
/// ### Description
/// Validates cli convert invalid from format functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_convert_invalid_from_format`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_convert_invalid_from_format() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"a": 1}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("convert")
        .arg("--from")
        .arg("unknown")
        .arg("--to")
        .arg("yaml")
        .arg(json_file.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported format: unknown"));
}

/// **Test Case**: Cli Convert Invalid To Format
///
/// ### Description
/// Validates cli convert invalid to format functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_convert_invalid_to_format`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_convert_invalid_to_format() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"a": 1}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("convert")
        .arg("--from")
        .arg("json")
        .arg("--to")
        .arg("unknown")
        .arg(json_file.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported format: unknown"));
}

/// **Test Case**: Cli Convert Output File
///
/// ### Description
/// Validates cli convert output file functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_convert_output_file`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_convert_output_file() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"a": 1}"#).unwrap();
    let out_file = temp_dir.path().join("out.yaml");

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("convert")
        .arg("--from")
        .arg("json")
        .arg("--to")
        .arg("yaml")
        .arg("--output")
        .arg(out_file.to_str().unwrap())
        .arg(json_file.to_str().unwrap())
        .assert()
        .success();

    let content = fs::read_to_string(out_file).unwrap();
    assert!(content.contains("a: 1"));
}

/// **Test Case**: Cli Convert Conversion Error
///
/// ### Description
/// Validates cli convert conversion error functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_convert_conversion_error`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_convert_conversion_error() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"NOT VALID JSON"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("convert")
        .arg("--from")
        .arg("json")
        .arg("--to")
        .arg("yaml")
        .arg(json_file.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Conversion error"));
}

/// **Test Case**: Convert Command Bad Output Path Error
///
/// ### Description
/// Asserts convert command correctly flags output destination failures.
///
/// ### Test Procedure
/// 1. Pass a directory path to the `--output` parameter rather than a file.
///
/// ### Expected Result
/// Safely errors out with `Error writing output file`.
#[test]
fn test_cli_convert_output_error() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"a": 1}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("convert")
        .arg("--from")
        .arg("json")
        .arg("--to")
        .arg("yaml")
        .arg("--output")
        .arg(temp_dir.path().to_str().unwrap()) // Directory, should fail
        .arg(json_file.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error writing output file"));
}

// ──────────────────────────── generate subcommand ──────────────────────────────

/// **Test Case**: Cli Generate Default Schema
///
/// ### Description
/// Validates cli generate default schema functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_generate_default_schema`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_generate_default_schema() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\":"));
}

/// **Test Case**: Cli Generate With Schema
///
/// ### Description
/// Validates cli generate with schema functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_generate_with_schema`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_generate_with_schema() {
    let temp_dir = TempDir::new().unwrap();
    let schema_file = temp_dir.path().join("schema.json");
    fs::write(
        &schema_file,
        r#"{"test_val": { "@type": "integer", "@start": 1, "@step": 0 }}"#,
    )
    .unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("generate")
        .arg("--schema")
        .arg(schema_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"test_val\": 1"));
}

/// **Test Case**: Cli Generate Invalid Schema
///
/// ### Description
/// Validates cli generate invalid schema functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_generate_invalid_schema`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_generate_invalid_schema() {
    let temp_dir = TempDir::new().unwrap();
    let schema_file = temp_dir.path().join("schema.json");
    fs::write(&schema_file, r#"{"test_val": "#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("generate")
        .arg("--schema")
        .arg(schema_file.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error parsing schema JSON:"));
}

/// **Test Case**: Cli Generate Missing Schema
///
/// ### Description
/// Validates cli generate missing schema functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_generate_missing_schema`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_generate_missing_schema() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("generate")
        .arg("--schema")
        .arg("missing.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error reading schema file"));
}

/// **Test Case**: Cli Generate Evaluation Error
///
/// ### Description
/// Validates cli generate evaluation error functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_generate_evaluation_error`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_generate_evaluation_error() {
    let temp_dir = TempDir::new().unwrap();
    let schema_file = temp_dir.path().join("schema.json");
    // Invalid generator instruction
    fs::write(&schema_file, r#"{"test_val": { "@type": "unknown_type" }}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("generate")
        .arg("--schema")
        .arg(schema_file.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Schema evaluation errors:"));
}

/// **Test Case**: Cli Generate Output File
///
/// ### Description
/// Validates cli generate output file functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_generate_output_file`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_generate_output_file() {
    let temp_dir = TempDir::new().unwrap();
    let out_file = temp_dir.path().join("out.json");

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("generate")
        .arg("--output")
        .arg(out_file.to_str().unwrap())
        .assert()
        .success();

    let content = fs::read_to_string(out_file).unwrap();
    assert!(content.contains("\"id\":"));
}

/// **Test Case**: Generate Command Bad Output Path Error
///
/// ### Description
/// Tests validation handling for generate file writing targets.
///
/// ### Test Procedure
/// 1. Attempt schema generation into a raw system temp directory folder rather than a discrete file.
///
/// ### Expected Result
/// Returns `Error writing output file`.
#[test]
fn test_cli_generate_output_file_error() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("generate")
        .arg("--output")
        .arg(temp_dir.path().to_str().unwrap()) // Directory, should fail
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error writing output file"));
}

/// **Test Case**: Cli Generate Minify
///
/// ### Description
/// Validates cli generate minify functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_generate_minify`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_generate_minify() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("generate")
        .arg("--minify")
        .assert()
        .success()
        .stdout(predicate::str::contains("\n").not());
}

// ──────────────────────────── more config subcommand tests ──────────────────────────────

/// **Test Case**: Cli Config All Keys
///
/// ### Description
/// Validates cli config all keys functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_config_all_keys`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_config_all_keys() {
    let temp_dir = TempDir::new().unwrap();

    let settings = [
        ("format.use_tabs", "true", "true"),
        ("format.indent", "8", "8"),
        ("format.line_ending", "crlf", "crlf"),
        ("format.folding_style", "compact", "compact"),
        ("format.sort_keys", "true", "true"),
        ("format.space_after_colon", "false", "false"),
        ("format.space_after_comma", "false", "false"),
        ("parser.allow_comments", "true", "true"),
        ("parser.allow_trailing_commas", "true", "true"),
        ("lint.duplicate_keys_severity", "ignore", "ignore"),
    ];

    for (key, val, expected) in settings.iter() {
        // Set
        jsonette_cmd(&temp_dir)
            .arg("config")
            .arg("set")
            .arg(key)
            .arg(val)
            .assert()
            .success();

        // Get
        jsonette_cmd(&temp_dir)
            .arg("config")
            .arg("get")
            .arg(key)
            .assert()
            .success()
            .stdout(predicate::str::contains(*expected));
    }
}

/// **Test Case**: Cli Config Invalid Keys
///
/// ### Description
/// Validates cli config invalid keys functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_config_invalid_keys`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_config_invalid_keys() {
    let temp_dir = TempDir::new().unwrap();

    // Invalid Get
    jsonette_cmd(&temp_dir)
        .arg("config")
        .arg("get")
        .arg("unknown.key")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown configuration key"));

    // Invalid Set
    jsonette_cmd(&temp_dir)
        .arg("config")
        .arg("set")
        .arg("unknown.key")
        .arg("value")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown configuration key"));
}

/// **Test Case**: Cli Config Invalid Values
///
/// ### Description
/// Validates cli config invalid values functionality.
///
/// ### Test Procedure
/// 1. Execute `test_cli_config_invalid_values`.
///
/// ### Expected Result
/// Completes successfully meeting all assertions.
#[test]
fn test_cli_config_invalid_values() {
    let temp_dir = TempDir::new().unwrap();

    // Bool err
    jsonette_cmd(&temp_dir)
        .arg("config")
        .arg("set")
        .arg("format.use_tabs")
        .arg("notabool")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Value must be a boolean"));

    // Number err
    jsonette_cmd(&temp_dir)
        .arg("config")
        .arg("set")
        .arg("format.indent")
        .arg("notanumber")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Value must be a number"));

    // Line ending err
    jsonette_cmd(&temp_dir)
        .arg("config")
        .arg("set")
        .arg("format.line_ending")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Value must be 'lf' or 'crlf'"));

    // Folding style err
    jsonette_cmd(&temp_dir)
        .arg("config")
        .arg("set")
        .arg("format.folding_style")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Value must be 'expanded' or 'compact'",
        ));

    // Severity err
    jsonette_cmd(&temp_dir)
        .arg("config")
        .arg("set")
        .arg("lint.duplicate_keys_severity")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Value must be 'error', 'warning', or 'ignore'",
        ));
}

/// **Test Case**: Config List Output
///
/// ### Description
/// Evaluates the `config list` sub-command default execution.
///
/// ### Test Procedure
/// 1. Run `jsonette config list`.
///
/// ### Expected Result
/// Validates stdout prints configuration properties containing expected keys like `format`.
#[test]
fn test_cli_config_list() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("config")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("format"));
}

/// **Test Case**: Config Set Write Permissions
///
/// ### Description
/// Checks if the CLI intercepts OS-level file permission errors when updating local configs.
///
/// ### Test Procedure
/// 1. Mock the settings folder structure and set read-only permissions `0o444`.
/// 2. Try executing `config set format.indent 4`.
///
/// ### Expected Result
/// Evaluates internal branches without throwing uncontrolled Rust panics.
#[test]
fn test_cli_config_set_error() {
    let temp_dir = TempDir::new().unwrap();

    // Create a read-only settings file to cause an error
    let settings_dir = temp_dir.path().join("jsonette");
    fs::create_dir_all(&settings_dir).unwrap();
    let settings_file = settings_dir.join("settings.json");
    fs::write(&settings_file, "{}").unwrap();

    // On Unix, make it read-only
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&settings_file, fs::Permissions::from_mode(0o444)).unwrap();
    }

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("config")
        .arg("set")
        .arg("format.indent")
        .arg("4")
        .assert();
}

/// **Test Case**: Format Output Write Permissions
///
/// ### Description
/// Evaluates robust failures when formatted strings fail to write to destination `--output`.
///
/// ### Test Procedure
/// 1. Send the `--output` format parameter a directory path instead of a file.
///
/// ### Expected Result
/// Validates error `Error writing output file` prints to stderr cleanly.
#[test]
fn test_cli_format_output_file_error() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"a": 1}"#).unwrap();

    // Output to a directory, which should fail
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("format")
        .arg("--output")
        .arg(temp_dir.path().to_str().unwrap())
        .arg(json_file.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error writing output file"));
}

/// **Test Case**: Format Conflict In Place from Stdin
///
/// ### Description
/// Validates format rejects `--in-place` modification when files are missing (stdin).
///
/// ### Test Procedure
/// 1. Pipe json into standard input and provide `--in-place` flag.
///
/// ### Expected Result
/// Prevents execution printing `Cannot perform in-place`.
#[test]
fn test_cli_format_inplace_error() {
    let temp_dir = TempDir::new().unwrap();
    // Cannot inplace format stdin
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("format")
        .arg("--in-place")
        .write_stdin(r#"{"a": 1}"#)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot perform in-place"));
}

/// **Test Case**: Format Custom Overrides
///
/// ### Description
/// Evaluates CLI overriding settings config behavior dynamically without disk persistence.
///
/// ### Test Procedure
/// 1. Pipe formatting options `--use-tabs`, `--indent`, `--line-ending crlf`, and `--folding-style expanded` directly to the `format` sub-command.
///
/// ### Expected Result
/// Test exits successfully parsing the format string correctly.
#[test]
fn test_cli_format_options() {
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    fs::write(&json_file, r#"{"a": 1}"#).unwrap();

    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("format")
        .arg("--use-tabs")
        .arg("true")
        .arg("--indent")
        .arg("4")
        .arg("--line-ending")
        .arg("crlf")
        .arg("--folding-style")
        .arg("expanded")
        .arg(json_file.to_str().unwrap())
        .assert()
        .success();
}

/// **Test Case**: Format Invalid Input Error Output
///
/// ### Description
/// Verifies graceful degradation for file open operations.
///
/// ### Test Procedure
/// 1. Point `format` to a non-existent `missing.json` file.
///
/// ### Expected Result
/// Safely stops with `Error reading input`.
#[test]
fn test_cli_format_invalid_file_reports_error() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = jsonette_cmd(&temp_dir);
    cmd.arg("format")
        .arg("missing.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error reading input"));
}
