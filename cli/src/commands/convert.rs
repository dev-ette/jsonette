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

use crate::args::ConvertArgs;
use crate::utils::read_input;
use std::str::FromStr;

/// Executes the `jsonette convert` subcommand end-to-end.
///
/// # Arguments
///
/// * `args` - Parsed convert subcommand arguments.
///
/// # Returns
///
/// Nothing.
pub fn handle_convert(args: ConvertArgs) {
    let (input, _label) = match read_input(&args.file) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            std::process::exit(1);
        }
    };

    let from_fmt = match jsonette::converter::DataFormat::from_str(&args.from) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let to_fmt = match jsonette::converter::DataFormat::from_str(&args.to) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    match jsonette::converter::convert(&input, from_fmt, to_fmt) {
        Ok(output) => {
            println!("{}", output);
        }
        Err(e) => {
            eprintln!("Conversion error: {}", e);
            std::process::exit(1);
        }
    }
}
