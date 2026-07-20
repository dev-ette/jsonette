# Jsonette: Visionary UI/UX Blueprint
*The Ultimate Native macOS JSON Environment*

## 1. The Core Philosophy
Jsonette is not just a text editor that happens to load JSON; it is a **data exploration environment**. The UI must be invisible when reading, and instantly accessible when editing.

---

## 2. Global Application Architecture
The application revolves around a **Workspace Window** utilizing native macOS tabbing.

### 2.1 The "Zen" Shell
*   **Window Chrome:** Full-size content view with a transparent title bar (`.windowStyle(.hiddenTitleBar)`).
*   **Toolbar:** Merged seamlessly into the top of the editor. Contains highly curated actions: View Mode Toggles, Export, and a prominent unified Search/Command Bar.

### 2.2 View Modes (The Triad)
1.  **Code Mode:** A pure, blazing-fast text editor with advanced syntax highlighting, line numbers, and a minimap.
2.  **Tree Mode:** A fully interactive, virtualized visual representation of the JSON with disclosure triangles for expansion/collapsing.
3.  **Split Mode:** Synchronized side-by-side view.

---

## 3. Core Editor Behaviors & Synchronization (Q6, Q7, Q8)

### 3.1 Editor Fundamentals
*   **Gutter:** The left edge must display line numbers, code-folding carets, and diagnostic issue markers. 
*   **Navigation:** `Cmd + L` brings up the Command Bar focused on "Go to Line...".
*   **Search & Replace:** Triggered via `Cmd+F` / `Cmd+Option+F`. Features a native inline search bar spanning the top of the editor. Unique features: Toggles to "Search in Keys Only", "Search in Values Only", and Regex support. (Q14)

### 3.2 Tree & Code Synchronization
*   **Tree Interaction:** Tree nodes MUST be expandable and collapsible via native disclosure triangles (`OutlineGroup` / `DisclosureGroup`). (Q8)
*   **Selection Tracking:** In Split mode, selection is bi-directionally synchronized. Clicking a node in the tree highlights the corresponding line range in the code editor, scrolling it into view. Clicking a line in the code editor highlights and expands the corresponding node path in the tree. (Q7)
*   **Inline Editing:** Double-clicking a primitive value (string, number, boolean) in the Tree Mode transforms it into a text field for inline mutation.

---

## 4. Rendering Engine Capabilities in the GUI

### 4.1 JSONPath Query Evaluator & Autocomplete (Q3, Q4)
*   **Invocation:** `Cmd + K` opens the floating Command Bar.
*   **Autocomplete:** As the user types `$.`, the GUI queries the Engine's `getAutocomplete(path)` function. A macOS popover drops down from the Command Bar showing available keys/symbols, allowing the user to navigate with up/down arrows and hit `Tab` to complete.
*   **Results Display:** While typing, a transient preview appears under the Command Bar. Pressing `Enter` executes the full query and opens the results in a **New Tab** or a **Bottom Panel**. The results panel features a distinct "Copy as JSON" button in its header.

### 4.2 Data Generation (Q1)
*   **Interaction:** Handled via a dedicated "Generate Canvas" macOS Sheet (Modal).
*   **Visuals:** A node-based schema builder on the left (defining types like UUID, Int ranges, random strings). A live-updating JSON preview on the right. Clicking "Insert" either replaces the current document or opens a new Tab.

### 4.3 Formatting Reactions & Triggers (Q10, Q11)
*   **Reaction:** When the engine formats the document, the GUI must **preserve the cursor position and scroll state**. The text update should trigger a subtle, brief flash (e.g., a 0.2s highlight) to indicate the layout changed, rather than a jarring snap.
*   **Triggers:** 
    *   Manual: `Cmd + Shift + F`
    *   Automatic: Configurable in Settings. "Format on Save" (Default: ON) and "Format on Paste" (Default: OFF). Formatting on every keystroke is disabled to prevent cursor jumping.

### 4.4 Conversions & Exporting (Q9)
*   Exporting to XML, YAML, or TOML is handled via `File -> Export As -> [Format]`, which triggers the standard `NSSavePanel`. 
*   Alternatively, a "Live Translation" split mode can be activated where the left pane is JSON and the right pane displays the active Engine conversion to YAML.

---

## 5. File Management & macOS Integration (Q5, Q12, Q13)

### 5.1 Saving State & "Dirty" Indicators
*   **Auto-save:** Jsonette leverages standard `NSDocument` behavior. macOS automatically saves document states periodically.
*   **Dirty State:** Unsaved modifications are indicated by a dark dot inside the Red Close button (Traffic Light) and an "Edited" suffix in the title bar dropdown, perfectly aligning with Apple HIG. (Q12, Q13)

### 5.2 Theming
*   **System Integration:** The app defaults to "System" theme, switching between Light and Dark mode dynamically based on macOS Control Center settings. (Q5)
*   **Overrides:** Users can force Light or Dark mode, and select specific syntax coloring palettes, via Settings.

---

## 6. Settings & Metadata (Q2, Q15, Q16)

### 6.1 Settings Architecture
The Preferences window (`Cmd + ,`) is separated into distinct tabs to cleanly separate GUI logic from Engine logic:
*   **Tab 1: Editor (GUI-Only):** Theme (System/Light/Dark), Font Family, Font Size, Minimap Toggle, Show Line Numbers, Word Wrap.
*   **Tab 2: Engine (Core):** Default Indentation (Spaces vs Tabs, Size), Strict Parsing Toggle, Format on Save, Quote Styles.
*   **Tab 3: Keybindings:** Customizable shortcuts.

### 6.2 The "About" Window
The standard macOS `Jsonette -> About Jsonette` dialog must display:
*   Large App Icon
*   App Name & App Version (e.g., `Version 1.0.0 (Build 42)`)
*   **Engine Version:** Distinctly displayed underneath (e.g., `Powered by jsonette-core v0.4.1`)
*   Copyright Notice
*   Links to Documentation / GitHub repository.
