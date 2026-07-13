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

//! Command routing and dispatcher.
//!
//! Subcommands accepted by the CLI are delegated to their respective handlers
//! inside this module.

pub mod config;
pub mod convert;
pub mod explore;
pub mod format;
pub mod generate;
pub mod query;

pub use config::handle_config;
pub use explore::handle_explore;
pub use format::handle_format;
pub use generate::handle_generate;
pub use query::handle_query;
