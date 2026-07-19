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

//! JSONPath exploration command execution.
//!
//! Provides a mechanism to inspect JSON structure (keys or array length)
//! rather than extracting exact values. Resolves the discoverability problem
//! for unfamiliar JSON structures.

use crate::args::ExploreArgs;
use crate::utils::{print_diagnostics, read_input};
use jsonette_core::JsonNode;
use regex::Regex;

/// Executes the `jsonette explore` subcommand end-to-end.
///
/// The pipeline is:
/// 1. Read input source.
/// 2. Parse JSON.
/// 3. Validate JSONPath.
/// 4. Evaluate JSONPath.
/// 5. For each match:
///    - If Array: print length.
///    - If Object: filter keys (grep/regex), sort alphabetically, apply limits, and print.
/// 6. Pipe output to `less` (with fallback to stdout).
///
/// # Arguments
///
/// * `args` - Parsed explore subcommand arguments.
///
/// # Returns
///
/// Nothing.
pub fn handle_explore(mut args: ExploreArgs) {
    // Heuristic: If only one positional argument is provided (parsed as `path`)
    // and it corresponds to an existing file on disk, assume the user omitted
    // the JSONPath and meant to explore the root of that file.
    if args.file.is_none() {
        let path_as_file = std::path::Path::new(&args.path);
        if path_as_file.exists() && path_as_file.is_file() {
            args.file = Some(path_as_file.to_path_buf());
            args.path = "$".to_string();
        }
    }

    let (input, label) = match read_input(&args.file) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            std::process::exit(1);
        }
    };

    let path_diags = jsonette_core::diagnostics_for_path(&args.path);
    if !path_diags.is_empty() {
        for diag in &path_diags {
            eprintln!("Error: {}", diag.message);
        }
        std::process::exit(1);
    }

    match jsonette_core::evaluate_path_on_str(&input, &args.path) {
        Ok(matches) => {
            if matches.is_empty() {
                println!("No nodes matched the path.");
                return;
            }

            let mut output = String::new();

            for (i, match_node) in matches.iter().enumerate() {
                if matches.len() > 1 {
                    output.push_str(&format!("--- Match {} ---\n", i + 1));
                }

                match match_node {
                    JsonNode::Array(elements, _) => {
                        output.push_str(&format!("Length: {} elements\n", elements.len()));
                        if !elements.is_empty() {
                            output.push_str(&format!(
                                "(indices [0] through [{}])\n",
                                elements.len() - 1
                            ));
                        }
                    }
                    JsonNode::Object(pairs, _) => {
                        let mut keys: Vec<String> = pairs.iter().map(|kv| kv.key.clone()).collect();

                        // Filter by grep
                        if let Some(grep_str) = &args.grep {
                            keys.retain(|k| k.contains(grep_str));
                        }

                        // Filter by regex
                        if let Some(regex_str) = &args.regex {
                            if let Ok(re) = Regex::new(regex_str) {
                                keys.retain(|k| re.is_match(k));
                            } else {
                                eprintln!("Error: Invalid regex '{}'", regex_str);
                                std::process::exit(1);
                            }
                        }

                        keys.sort();

                        let total_matched = keys.len();

                        // Apply limit
                        if let Some(limit) = args.limit {
                            keys.truncate(limit);
                        }

                        if keys.is_empty() {
                            output.push_str("No keys found (or all filtered out).\n");
                        } else {
                            for key in &keys {
                                output.push_str(&format!("{}\n", key));
                            }
                            if total_matched > keys.len() {
                                output.push_str(&format!(
                                    "... and {} more keys\n",
                                    total_matched - keys.len()
                                ));
                            }
                        }
                    }
                    _ => {
                        output.push_str(&format!(
                            "Matched a {} node. Use 'query' to see its value.\n",
                            match_node.node_type()
                        ));
                    }
                }
                if matches.len() > 1 && i < matches.len() - 1 {
                    output.push('\n');
                }
            }

            // Send output to pager
            page_output(&output);
        }
        Err(err) => {
            if err.starts_with("Invalid JSON") {
                if let Err(diags) = jsonette_core::parse(&input) {
                    print_diagnostics(&input, &diags, &label);
                } else {
                    eprintln!("{}", err);
                }
                std::process::exit(1);
            } else {
                eprintln!("Error evaluating JSONPath: {}", err);
                std::process::exit(1);
            }
        }
    }
}

/// Helper to pipe string output to `less`, or fallback to stdout if unavailable.
fn page_output(output: &str) {
    #[cfg(not(test))]
    {
        use std::io::{IsTerminal, Write};
        use std::process::{Command, Stdio};

        if std::io::stdout().is_terminal()
            && let Ok(mut child) = Command::new("less")
                .args(["-F", "-R", "-X"])
                .stdin(Stdio::piped())
                .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(output.as_bytes());
            }
            let _ = child.wait();
            return;
        }
    }

    print!("{}", output);
}
