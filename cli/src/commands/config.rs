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

//! Global configuration management command execution.
//!
//! Exposes subcommands to list, get, and set global settings values from/to the
//! shared user configuration file on disk.

use crate::args::{ConfigArgs, ConfigCommands};
use jsonette::{AppSettings, FoldingStyle, LineEnding, Severity};

/// Executes the global configuration settings command.
///
/// # Arguments
///
/// * `args` - Parse config subcommands and arguments.
///
/// # Returns
///
/// Nothing.
pub fn handle_config(args: ConfigArgs) {
    let mut settings = jsonette::get_settings();
    match args.command {
        ConfigCommands::List => match serde_json::to_string_pretty(&settings) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("Error serializing configuration: {}", e);
                std::process::exit(1);
            }
        },
        ConfigCommands::Get { key } => match get_setting_value(&settings, &key) {
            Ok(value) => println!("{}", value),
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(1);
            }
        },
        ConfigCommands::Set { key, value } => {
            match set_setting_value(&mut settings, &key, &value) {
                Ok(()) => {
                    if let Err(e) = jsonette::update_settings(settings) {
                        eprintln!("Error saving configuration: {}", e);
                        std::process::exit(1);
                    }
                    println!("Configuration updated: {} = {}", key, value);
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
            }
        }
    }
}

/// Helper function to retrieve a specific configuration setting value as a String.
///
/// # Arguments
///
/// * `settings` - The reference to the global `AppSettings` structure.
/// * `key` - The setting key path string.
///
/// # Returns
///
/// * `Ok(String)` - The retrieved settings value.
/// * `Err(String)` - An error indicating the configuration key is unknown.
fn get_setting_value(settings: &AppSettings, key: &str) -> Result<String, String> {
    match key {
        "format.use_tabs" => Ok(settings.format.use_tabs.to_string()),
        "format.indent" => Ok(settings.format.indent.to_string()),
        "format.line_ending" => Ok(format!("{:?}", settings.format.line_ending).to_lowercase()),
        "format.folding_style" => Ok(format!("{:?}", settings.format.folding_style).to_lowercase()),
        "format.sort_keys" => Ok(settings.format.sort_keys.to_string()),
        "format.space_after_colon" => Ok(settings.format.space_after_colon.to_string()),
        "format.space_after_comma" => Ok(settings.format.space_after_comma.to_string()),
        "parser.allow_comments" => Ok(settings.parser.allow_comments.to_string()),
        "parser.allow_trailing_commas" => Ok(settings.parser.allow_trailing_commas.to_string()),
        "lint.duplicate_keys_severity" => {
            Ok(format!("{:?}", settings.lint.duplicate_keys_severity).to_lowercase())
        }
        _ => Err(format!("Unknown configuration key: '{}'", key)),
    }
}

/// Helper function to parse and update a specific configuration setting value.
///
/// # Arguments
///
/// * `settings` - A mutable reference to the `AppSettings` structure.
/// * `key` - The setting key path string.
/// * `value` - The new value string to apply.
///
/// # Returns
///
/// * `Ok(())` - Setting successfully applied.
/// * `Err(String)` - Error detail if setting key is unknown or value format is invalid.
fn set_setting_value(settings: &mut AppSettings, key: &str, value: &str) -> Result<(), String> {
    match key {
        "format.use_tabs" => {
            settings.format.use_tabs = value
                .parse()
                .map_err(|_| "Value must be a boolean (true/false)")?;
        }
        "format.indent" => {
            settings.format.indent = value
                .parse()
                .map_err(|_| "Value must be a number (0-255)")?;
        }
        "format.line_ending" => {
            settings.format.line_ending = match value.to_lowercase().as_str() {
                "lf" => LineEnding::LF,
                "crlf" => LineEnding::CRLF,
                _ => return Err("Value must be 'lf' or 'crlf'".to_string()),
            };
        }
        "format.folding_style" => {
            settings.format.folding_style = match value.to_lowercase().as_str() {
                "expanded" => FoldingStyle::Expanded,
                "compact" => FoldingStyle::Compact,
                _ => return Err("Value must be 'expanded' or 'compact'".to_string()),
            };
        }
        "format.sort_keys" => {
            settings.format.sort_keys = value
                .parse()
                .map_err(|_| "Value must be a boolean (true/false)")?;
        }
        "format.space_after_colon" => {
            settings.format.space_after_colon = value
                .parse()
                .map_err(|_| "Value must be a boolean (true/false)")?;
        }
        "format.space_after_comma" => {
            settings.format.space_after_comma = value
                .parse()
                .map_err(|_| "Value must be a boolean (true/false)")?;
        }
        "parser.allow_comments" => {
            settings.parser.allow_comments = value
                .parse()
                .map_err(|_| "Value must be a boolean (true/false)")?;
        }
        "parser.allow_trailing_commas" => {
            settings.parser.allow_trailing_commas = value
                .parse()
                .map_err(|_| "Value must be a boolean (true/false)")?;
        }
        "lint.duplicate_keys_severity" => {
            settings.lint.duplicate_keys_severity = match value.to_lowercase().as_str() {
                "error" => Severity::Error,
                "warning" => Severity::Warning,
                "ignore" => Severity::Ignore,
                _ => return Err("Value must be 'error', 'warning', or 'ignore'".to_string()),
            };
        }
        _ => return Err(format!("Unknown configuration key: '{}'", key)),
    }
    Ok(())
}
