# macOS App Guide

The `jsonette` macOS application is a native shell built using SwiftUI and AppKit. True to the project's architecture decisions, it operates strictly as a **dumb rendering layer**. All parsing, logic, and evaluation tasks are fully delegated to the shared Rust core engine via a UniFFI bridge.

---

## 🏗️ Architecture

- **UI Framework**: SwiftUI (for primary views) and AppKit (for low-level integration).
- **Core Engine Bridge**: `UniFFI` is used to automatically generate Swift language bindings (`macos/Generated/jsonette_core.swift`) from the Rust engine FFI exports.
- **State Management**: Uses native Combine and `@Published` properties within `ObservableObject` wrappers to marshal data out of the Rust engine.

### UI/UX Design Direction
The user experience and component layout are strictly governed by our [Visionary UI/UX Blueprint](design/ui-ux-blueprint.md).
The architectural decision to transition to this fluid, multi-modal layout is recorded in [ADR 0009](architecture/decisions/0009-visionary-macos-ui-ux-blueprint.md).

---

## 🛠️ Local Development & VSCode Setup

While you can use Xcode, the project is configured for a seamless VSCode workflow.

**Prerequisites**:

1. Install [Homebrew](https://brew.sh/).
2. `brew install xcodegen swiftlint`
3. Install the VSCode extensions: **CodeLLDB**, **Swift**, and **SwiftLint** (`vknabel.vscode-swiftlint`).

**Build Steps**:

1. Run the `macOS: Generate Xcode Project` VSCode task to invoke `xcodegen` and generate `jsonette.xcodeproj`.
2. Run the `macOS: Build App` VSCode task to compile the Rust engine, run the UniFFI bridge code generation, and build the Swift app into `macos/build/Debug`.

**Debugging**:
Press `F5` in VSCode with the **"Debug macOS App"** launch configuration selected to attach LLDB to the running macOS app.

---

## 📏 Coding Standards & Formatting

### 1. SwiftLint

The macOS project strictly adheres to `SwiftLint` rules to maintain code quality.

- Run `swiftlint lint --strict` before submitting a PR. Our CI pipeline enforces this check.
- Keep components small and view structs modular.

### 2. Documentation Style

For public structs, classes, and methods, use standard Swift docstrings (`///`).

For unit tests, we mirror the rigorous documentation standards applied in the Rust engine. Every test must define its intent, procedure, and expected outcome using the following format:

```swift
/// **Test Case**: <Brief Title>
///
/// ### Description
/// <What this test validates>
///
/// ### Test Procedure
/// 1. <Step 1>
///
/// ### Expected Result
/// <What should happen>
func testSomethingImportant() throws {
    // Implementation
}
```

### 3. Engine Isolation

**Never** import parsing libraries or evaluate business logic in Swift. Always pass the raw string payload or required arguments into the generated `jsonette_core` Swift module and handle the returned structured data.

---

## 🧪 Testing

We use the native `XCTest` framework.

- Tests are located in `macos/Tests/`.
- Run tests directly in VSCode via the `macOS: Run Tests` task, or via the CLI:
  ```bash
  xcodebuild test -project jsonette.xcodeproj -scheme jsonette -destination 'platform=macOS' SYMROOT="build"
  ```

---

## 🔒 CI and Secrets

We utilize a robust GitHub Actions pipeline (`macos-ci.yml` and `release.yml`) for the macOS application to guarantee correctness and security.

### Pull Request Validation

Every PR targeting `main` runs:

- `swiftlint --strict`
- Complete Rust and Swift compilations
- Automated `XCTest` runs

### Release Pipeline & Ad-Hoc Signing

On tagged releases (`v*`), the app is built and packaged into a `.dmg`. Because the project currently operates without a paid Apple Developer Program enrollment, the application is **ad-hoc signed**.

**Important notice for users:**
Since the app is not signed with a recognized Developer ID or notarized by Apple, macOS Gatekeeper will block it upon first launch with an "App is damaged" or "Unidentified developer" warning.

To run the application, users must manually strip the quarantine attribute after copying it to the Applications folder:

```bash
xattr -cr /Applications/jsonette.app
```
