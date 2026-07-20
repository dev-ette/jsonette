# 9. Visionary macOS UI/UX Blueprint

Date: 2026-07-19

## Status

Accepted

## Context

The initial macOS application served as a basic harness to test the separation between the `jsonette-core` Rust engine and the SwiftUI layer. It utilized standard Split Views (HSplitView) and fundamental layouts that lacked the polish, flexibility, and productivity required by professional developers dealing with large JSON structures. 

To compete as a premier native macOS application, we needed to abandon the utilitarian layout and design a highly fluid, multi-modal "JSON Exploration Environment" that deeply integrates engine capabilities without cluttering the interface.

## Decision

We have decided to radically redesign the macOS application architecture based on a new "Visionary UX Blueprint."

Key structural decisions include:
1.  **Zen Shell & Navigation:** Hiding the window title bar (`.windowStyle(.hiddenTitleBar)`) and utilizing a 3-mode view toggle (Code, Tree, Split) instead of static sidebars.
2.  **Floating Overlays over Side Panels:** Moving the Query/JSONPath interface to a floating `Cmd+K` Command Bar overlay, reducing persistent visual clutter.
3.  **Strict State/Settings Separation:** Grouping Engine behaviors (e.g., Strict Parsing, Default Indent) separately from Editor GUI settings (e.g., Theme, Font size).
4.  **Interactive Elements:** Introducing dynamic floating breadcrumbs and asynchronous Tree/Code highlighting synchronization.

The detailed UI/UX specifications and interactions for every feature (Generation, Format on Save, Autocomplete) are documented in the `docs/design/ui-ux-blueprint.md` file.

## Consequences

*   **Positive:** The app will feel significantly more professional, faster, and tailored to power users. The aesthetic aligns with modern macOS HIG and premium developer tools.
*   **Positive:** Engine features like Autocomplete and Formatter are now explicitly designed into the interaction flow (e.g., drop-downs, animated flashes).
*   **Negative:** Increased complexity in SwiftUI. Managing floating overlays (`ZStack`, popovers) and bi-directional scrolling synchronization between a Tree view and an Editor view is substantially more difficult than using standard static `NavigationSplitView` columns.
