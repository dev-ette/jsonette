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

//! Handles the `generate` subcommand for dummy JSON data creation.

use crate::args::GenerateArgs;
use jsonette_core::generator::{GeneratorOptions, generate_from_schema};
use jsonette_core::parser::parse;
use std::fs;
use std::io::{self, Write};

/// Starts the generation process.
///
/// # Arguments
///
/// * `args` - The GenerateArgs containing constraints and file output.
///
/// # Returns
///
/// Nothing.
pub fn handle_generate(args: GenerateArgs) {
    // 1. Read schema
    let schema_text = if let Some(schema_path) = &args.schema {
        match fs::read_to_string(schema_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading schema file: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Default built-in schema
        r#"{
            "id": { "@type": "uuid" },
            "index": { "@type": "integer", "@start": 0, "@step": 1 },
            "score": { "@type": "float", "@min": 0, "@max": 100 },
            "name": { "@type": "string", "@pool": ["Alice", "Bob", "Charlie", "David"] },
            "tags": {
                "@type": "array",
                "@count": 3,
                "@item": { "@type": "string", "@pool": ["urgent", "low", "bug", "feature"] }
            }
        }"#
        .to_string()
    };

    // 2. Parse schema
    let schema_node = match parse(&schema_text) {
        Ok(node) => node,
        Err(diags) => {
            eprintln!("Error parsing schema JSON:");
            for diag in diags {
                eprintln!("- {}", diag.message);
            }
            std::process::exit(1);
        }
    };

    // 3. Generate data
    let gen_opts = GeneratorOptions {
        target_size_bytes: args.size,
        target_count: args.count,
    };
    let generated_node = match generate_from_schema(&schema_node, &gen_opts) {
        Ok(node) => node,
        Err(diagnostics) => {
            eprintln!("Schema evaluation errors:");
            for diag in diagnostics {
                eprintln!("- {}", diag.message);
            }
            std::process::exit(1);
        }
    };

    // 4. Format
    let formatted = if args.minify {
        jsonette_core::minify(&generated_node)
    } else {
        jsonette_core::format(&generated_node)
    };

    // 5. Output
    if let Some(output_path) = args.output {
        if let Err(e) = fs::write(&output_path, formatted) {
            eprintln!("Error writing output file: {}", e);
            std::process::exit(1);
        }
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        if let Err(e) = handle.write_all(formatted.as_bytes()) {
            eprintln!("Error writing to stdout: {}", e);
            std::process::exit(1);
        }
    }
}
