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

//! Binary helper to execute `uniffi-bindgen` CLI commands within the workspace.
//!
//! Invokes the core UniFFI binding generator to compile Swift FFI headers
//! and sources from the compiled Rust library files.

/// Entry point that delegates directly to UniFFI's command-line runner.
fn main() {
    uniffi::uniffi_bindgen_main();
}
