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

//! JSON formatting and minification command execution.
//!
//! Loads file or standard input, parses and formats JSON data, and handles
//! dynamic formatting configuration overrides.

use crate::args::FormatArgs;
use crate::utils::{print_diagnostics, read_input};
use std::fs;

/// Executes the formatting and minification command.
///
/// # Arguments
///
/// * `args` - Parse command formatting options and input targets.
pub fn handle_format(args: FormatArgs) {
    if args.in_place && args.file.is_none() {
        eprintln!("Error: Cannot perform in-place formatting when reading from standard input.");
        std::process::exit(1);
    }

    let (input, label) = match read_input(&args.file) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            std::process::exit(1);
        }
    };

    // Load default/current configuration and apply any overrides
    let mut settings = jsonette::get_settings();
    if let Some(sort_keys) = args.sort_keys {
        settings.format.sort_keys = sort_keys;
    }
    if let Some(use_tabs) = args.use_tabs {
        settings.format.use_tabs = use_tabs;
    }
    if let Some(indent) = args.indent {
        settings.format.indent = indent;
    }
    if let Some(line_ending) = &args.line_ending {
        settings.format.line_ending = match line_ending.as_str() {
            "lf" => jsonette::LineEnding::LF,
            "crlf" => jsonette::LineEnding::CRLF,
            _ => unreachable!(),
        };
    }
    if let Some(folding_style) = &args.folding_style {
        settings.format.folding_style = match folding_style.as_str() {
            "expanded" => jsonette::FoldingStyle::Expanded,
            "compact" => jsonette::FoldingStyle::Compact,
            _ => unreachable!(),
        };
    }

    // Set settings in-memory only (do not write to disk during single command format run)
    jsonette::set_in_memory_settings(settings);

    // Parse the JSON
    let node = match jsonette::parse(&input) {
        Ok(node) => node,
        Err(diags) => {
            print_diagnostics(&input, &diags, &label);
            std::process::exit(1);
        }
    };

    // Format or minify
    let output = if args.minify {
        jsonette::minify(&node)
    } else {
        jsonette::format(&node)
    };

    // Output result
    if args.in_place {
        let path = args.file.as_ref().unwrap();
        let mut final_output = output;
        if !args.minify && !final_output.ends_with('\n') {
            final_output.push('\n');
        }
        if let Err(e) = fs::write(path, &final_output) {
            eprintln!("Error writing formatted file: {}", e);
            std::process::exit(1);
        }
    } else {
        print!("{}", output);
        if !args.minify && !output.ends_with('\n') {
            println!();
        }
    }
}
