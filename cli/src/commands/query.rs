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

//! JSONPath query expression command execution.
//!
//! Loads file or standard input, parses JSON, and evaluates JSONPath expressions
//! against the parsed document. Results are printed as a JSON array to stdout.

use crate::args::QueryArgs;
use crate::utils::{print_diagnostics, read_input};

/// Executes the JSONPath query command.
///
/// # Arguments
///
/// * `args` - Query command arguments: the JSONPath expression and input source.
pub fn handle_query(args: QueryArgs) {
    let (input, label) = match read_input(&args.file) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            std::process::exit(1);
        }
    };

    // Parse the JSON document
    let node = match jsonette::parse(&input) {
        Ok(node) => node,
        Err(diags) => {
            print_diagnostics(&input, &diags, &label);
            std::process::exit(1);
        }
    };

    // Validate the JSONPath expression before evaluation for better error messages
    let path_diags = jsonette::diagnostics_for_path(&args.query);
    if !path_diags.is_empty() {
        for diag in &path_diags {
            eprintln!("Error: {}", diag.message);
        }
        std::process::exit(1);
    }

    // Evaluate the JSONPath expression
    match jsonette::evaluate_path(&node, &args.query) {
        Ok(matches) => {
            if matches.is_empty() {
                println!("[]");
            } else {
                let mut results = Vec::new();
                for m in matches {
                    results.push(jsonette::format(&m));
                }
                println!("[\n  {}\n]", results.join(",\n  ").replace('\n', "\n  "));
            }
        }
        Err(err) => {
            eprintln!("Error evaluating JSONPath: {}", err);
            std::process::exit(1);
        }
    }
}
