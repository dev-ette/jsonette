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

// Prerequisites:
//   npm install ffi-napi ref-napi ref-struct-di
// Run with:
//   DYLD_LIBRARY_PATH=target/debug/ node bridge-spike/main.js

const ffi = require('ffi-napi');
const ref = require('ref-napi');
const Struct = require('ref-struct-di')(ref);

// Define the RustBuffer struct matching our FFI header
const RustBuffer = Struct({
    capacity: ref.types.uint64,
    len: ref.types.uint64,
    data: ref.refType(ref.types.uint8)
});

// Define the RustCallStatus struct
const RustCallStatus = Struct({
    code: ref.types.int8,
    errorBuf: RustBuffer
});

// Load the shared library
const libName = process.platform === 'darwin' ? './target/debug/libjsonette.dylib' : './target/debug/libjsonette.so';
const lib = ffi.Library(libName, {
    'uniffi_jsonette_fn_func_ping': [RustBuffer, [RustBuffer, ref.refType(RustCallStatus)]],
    'ffi_jsonette_rustbuffer_free': ['void', [RustBuffer, ref.refType(RustCallStatus)]]
});

// Helper to convert JS String to RustBuffer
function makeRustBuffer(str) {
    const buf = Buffer.from(str, 'utf-8');
    const rustBuf = new RustBuffer();
    rustBuf.len = buf.length;
    rustBuf.capacity = buf.length;
    rustBuf.data = buf;
    return rustBuf;
}

// Helper to convert RustBuffer to JS String
function readRustBuffer(rustBuf) {
    const ptr = rustBuf.data;
    const len = rustBuf.len;
    return ref.readBlock(ptr, len).toString('utf-8');
}

console.log("Calling Rust ping from Node.js FFI...");
const status = new RustCallStatus({ code: 0 });
const input = makeRustBuffer("world");
const output = lib.uniffi_jsonette_fn_func_ping(input, status.ref());

if (status.code !== 0) {
    console.error(`FFI call failed with code ${status.code}`);
    process.exit(1);
}

const result = readRustBuffer(output);
console.log(`Result: ${result}`);

const freeStatus = new RustCallStatus({ code: 0 });
lib.ffi_jsonette_rustbuffer_free(output, freeStatus.ref());

if (result === "pong: world") {
    console.log("UniFFI Node.js JS/TS bridge OK");
    process.exit(0);
} else {
    console.error("Bridge failed: unexpected result");
    process.exit(1);
}
