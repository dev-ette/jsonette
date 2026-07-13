<!--
Copyright (c) 2026 DevEtte.

This project is dual-licensed under both the MIT License and the
Apache License, Version 2.0 (the "License"). You may not use this
file except in compliance with one of these licenses.

You may obtain a copy of the Licenses at:
- MIT: https://opensource.org
- Apache 2.0: http://apache.org

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the licenses.
-->

# UniFFI Multi-Language Bridge Reference

This document provides a guide to generating, compiling, and verifying the FFI bindings for the `jsonette` Rust engine across **Swift, Python, C/C++, Kotlin, Java, and Node.js**.

---

## 🛠️ Setup & Architecture

The `jsonette` core engine compiles to:
*   `libjsonette.a` (static library for iOS/macOS Xcode linking)
*   `libjsonette.dylib` (dynamic library for local testing on macOS)
*   `libjsonette.so` (dynamic library for Linux testing)
*   `libjsonette.rlib` (standard Rust library for workspace dependencies)

We use UniFFI's **procedural macro-only setup** (supported in UniFFI 0.28+), which avoids maintaining a separate UDL file.

---

## 🚀 Step-by-Step Verification

First, ensure the Rust library is compiled:
```bash
cargo build
```

---

### 1. Swift (macOS / iOS)

Generate the Swift bindings, C header, and Clang modulemap:
```bash
cargo run --bin uniffi-bindgen -- generate \
  --library target/debug/libjsonette.dylib \
  --language swift \
  --out-dir bridge-spike/

mv bridge-spike/jsonetteFFI.modulemap bridge-spike/module.modulemap
```

Compile and run the smoke test:
```bash
swiftc -I bridge-spike \
       -L target/debug \
       -ljsonette \
       bridge-spike/jsonette.swift \
       bridge-spike/main.swift \
       -o bridge-spike/main

DYLD_LIBRARY_PATH=target/debug/ ./bridge-spike/main
```

---

### 2. Python (Linux / macOS)

Generate the Python bindings:
```bash
cargo run --bin uniffi-bindgen -- generate \
  --library target/debug/libjsonette.dylib \
  --language python \
  --out-dir bridge-spike/
```

Run the Python smoke test:
```bash
DYLD_LIBRARY_PATH=target/debug/ python3 bridge-spike/main.py
```

---

### 3. C / C++

Compile the C smoke test directly linking against the generated C header and Rust shared library:
```bash
clang -I bridge-spike/ \
      -L target/debug/ \
      -ljsonette \
      bridge-spike/main.c \
      -o bridge-spike/main_c

DYLD_LIBRARY_PATH=target/debug/ ./bridge-spike/main_c
```

---

### 4. Kotlin & Java (JVM)

Generate the Kotlin bindings:
```bash
cargo run --bin uniffi-bindgen -- generate \
  --library target/debug/libjsonette.dylib \
  --language kotlin \
  --out-dir bridge-spike/
```

This generates `bridge-spike/jsonette.kt`. 

To compile and run Kotlin/Java, we need the `jna-5.14.0.jar` (Java Native Access) dependency:
```bash
# Download JNA dependency jar
curl -Lo bridge-spike/jna.jar https://repo1.maven.org/maven2/net/java/dev/jna/jna/5.14.0/jna-5.14.0.jar

# Compile Kotlin bindings and test script
kotlinc -classpath bridge-spike/jna.jar \
        bridge-spike/jsonette.kt \
        bridge-spike/Main.kt \
        -d bridge-spike/MainKt.jar

# Run Kotlin test
java -classpath bridge-spike/jna.jar:bridge-spike/MainKt.jar \
     -Djna.library.path=target/debug/ \
     bridge.MainKt

# Compile Java test script referencing the Kotlin bindings jar
javac -classpath bridge-spike/jna.jar:bridge-spike/MainKt.jar \
      bridge-spike/Main.java \
      -d bridge-spike/

# Run Java test
java -classpath bridge-spike/jna.jar:bridge-spike/MainKt.jar:bridge-spike/ \
     -Djna.library.path=target/debug/ \
     bridge.Main
```

---

### 5. Node.js (JS / TS)

Node.js interacts with the FFI bindings using `ffi-napi`:
```bash
# Install Node.js FFI dependencies
npm install ffi-napi ref-napi ref-struct-di

# Run JS FFI script
DYLD_LIBRARY_PATH=target/debug/ node bridge-spike/main.js
```
