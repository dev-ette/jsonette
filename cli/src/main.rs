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

//! The main entry point for the `jsonette` command-line application.
//!
//! It parses command-line arguments and subcommands, delegating execution
//! to specific sub-modules while configuring environment preferences.

mod args;
mod commands;
mod utils;

use args::{Cli, Commands};
use clap::{CommandFactory, Parser};

/// The main entry point of the CLI application.
/// Parses the arguments and executes the requested subcommand.
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Format(args) => commands::handle_format(args),
        Commands::Query(args) => commands::handle_query(args),
        Commands::Explore(args) => commands::handle_explore(args),
        Commands::Config(args) => commands::handle_config(args),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
        }
    }
}
