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

//! Command-line argument structures and parser definitions.
//!
//! This module defines the hierarchy of commands, parameters, and flags
//! accepted by the application using `clap`.

use clap::{Args, Parser as ClapParser, Subcommand};
use std::path::PathBuf;

/// The top-level CLI command parser.
#[derive(ClapParser, Debug)]
#[command(
    name = "jsonette",
    version = "0.3.0",
    about = "Fast, lightweight JSON format, query, and configuration utility."
)]
pub struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available top-level subcommands for the application.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Format (pretty-print) or minify JSON.
    Format(FormatArgs),
    /// Query JSON using JSONPath.
    Query(QueryArgs),
    /// Explore a JSONPath node (list keys or array length).
    Explore(ExploreArgs),
    /// Generate dummy JSON data based on size.
    Generate(GenerateArgs),
    /// Manage global configuration settings.
    Config(ConfigArgs),
    /// Generate shell autocompletion scripts.
    Completions {
        /// The shell to generate completions for.
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

/// Arguments and options for the formatting and minification command.
#[derive(Args, Debug)]
pub struct FormatArgs {
    /// Input file path. If omitted, the tool reads from standard input.
    pub file: Option<PathBuf>,

    /// Minify the JSON output, stripping all whitespace.
    #[arg(short = 'm', long)]
    pub minify: bool,

    /// Sort object keys alphabetically. Overrides global settings.
    #[arg(short = 's', long)]
    pub sort_keys: Option<bool>,

    /// Indentation count (number of spaces or tab characters). Overrides global settings.
    #[arg(short = 'n', long)]
    pub indent: Option<u8>,

    /// Use tab characters for indentation instead of spaces. Overrides global settings.
    #[arg(long)]
    pub use_tabs: Option<bool>,

    /// Force specific line endings: 'lf' or 'crlf'. Overrides global settings.
    #[arg(long, value_parser = ["lf", "crlf"])]
    pub line_ending: Option<String>,

    /// Folding strategy for arrays and objects: 'expanded' or 'compact'. Overrides global settings.
    #[arg(long, value_parser = ["expanded", "compact"])]
    pub folding_style: Option<String>,

    /// Write the output to a specific file instead of standard output.
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Edit file in-place instead of printing to standard output.
    #[arg(short = 'i', long)]
    pub in_place: bool,
}

/// Arguments and options for the JSONPath query command.
#[derive(Args, Debug)]
pub struct QueryArgs {
    /// The JSONPath query string (e.g. `$.store.book[*].author`).
    pub query: String,

    /// Input file path. If omitted, the tool reads from standard input.
    pub file: Option<PathBuf>,
}

/// Arguments and options for the explore command.
#[derive(Args, Debug)]
pub struct ExploreArgs {
    /// The JSONPath expression to explore (e.g. `$[0]`).
    pub path: String,

    /// Input file path. If omitted, the tool reads from standard input.
    pub file: Option<PathBuf>,

    /// Optional substring match to filter keys.
    #[arg(short = 'g', long)]
    pub grep: Option<String>,

    /// Optional regex match to filter keys.
    #[arg(short = 'r', long)]
    pub regex: Option<String>,

    /// Limit the number of keys displayed.
    #[arg(short = 'n', long)]
    pub limit: Option<usize>,
}

/// Arguments and options for the generate command.
#[derive(Args, Debug)]
pub struct GenerateArgs {
    /// Target size in bytes (e.g. 1000000 for 1MB). Mutually exclusive with count.
    #[arg(short = 's', long, conflicts_with = "count")]
    pub size: Option<usize>,

    /// Target number of array items to generate. Mutually exclusive with size.
    #[arg(short = 'n', long, conflicts_with = "size")]
    pub count: Option<usize>,

    /// Optional path to a JSON generation schema file.
    #[arg(short = 'c', long)]
    pub schema: Option<PathBuf>,

    /// Write the output to a specific file instead of standard output.
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Minify the JSON output, stripping all whitespace.
    #[arg(short = 'm', long)]
    pub minify: bool,
}

/// Arguments for the global configuration subcommand.
#[derive(Args, Debug)]
pub struct ConfigArgs {
    /// The configuration action to perform.
    #[command(subcommand)]
    pub command: ConfigCommands,
}

/// Subcommands to view, query, or modify global settings.
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// List all settings in JSON format.
    List,
    /// Get the value of a specific setting key.
    Get {
        /// The config key path (e.g. `format.sort_keys`).
        key: String,
    },
    /// Update a specific setting key.
    Set {
        /// The config key path (e.g. `format.sort_keys`).
        key: String,
        /// The new value to store (e.g. `true`).
        value: String,
    },
}
