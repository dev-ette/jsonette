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
//! Reads a JSON document from a file path or standard input, parses it into
//! the engine AST, validates the supplied JSONPath expression, evaluates it,
//! and prints each matched node as formatted JSON inside a top-level array
//! on standard output.

use crate::args::QueryArgs;
use crate::utils::{print_diagnostics, read_input};

/// Executes the `jsonette query` subcommand end-to-end.
///
/// The pipeline is:
/// 1. Read the input source (file or stdin) using [`read_input`].
/// 2. Parse the JSON document via the core engine parser.
/// 3. Validate the JSONPath expression using [`jsonette_core::diagnostics_for_path`]
///    before evaluation to surface actionable error messages on stderr.
/// 4. Evaluate the JSONPath expression via [`jsonette_core::evaluate_path`].
/// 5. Print matched nodes as a JSON array to stdout, one formatted node per
///    array element. Prints `[]` when no nodes match, consistent with the
///    RFC 9535 empty-nodelist convention.
///
/// Any error in steps 1–4 prints a message to stderr and exits with code 1.
///
/// # Arguments
///
/// * `args` - Parsed query subcommand arguments containing the JSONPath
///   expression string in `args.query` and the optional input file path in `args.file`.
///
/// # Returns
///
/// Nothing.
pub fn handle_query(args: QueryArgs) {
    let (input, label) = match read_input(&args.file) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            std::process::exit(1);
        }
    };

    // Parse the JSON document into the engine AST.
    let node = match jsonette_core::parse(&input) {
        Ok(node) => node,
        Err(diags) => {
            print_diagnostics(&input, &diags, &label);
            std::process::exit(1);
        }
    };

    // Validate the JSONPath expression syntax before evaluation so that
    // invalid paths produce clear error messages rather than an empty result.
    let path_diags = jsonette_core::diagnostics_for_path(&args.query);
    if !path_diags.is_empty() {
        for diag in &path_diags {
            eprintln!("Error: {}", diag.message);
        }
        std::process::exit(1);
    }

    // Evaluate the JSONPath expression and print the result.
    match jsonette_core::evaluate_path(&node, &args.query) {
        Ok(matches) => {
            if matches.is_empty() {
                println!("[]");
            } else {
                let formatted: Vec<String> = matches.iter().map(jsonette_core::format).collect();
                println!("[\n  {}\n]", formatted.join(",\n  ").replace('\n', "\n  "));
            }
        }
        Err(err) => {
            eprintln!("Error evaluating JSONPath: {}", err);
            std::process::exit(1);
        }
    }
}
