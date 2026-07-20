import SwiftUI

@main
struct jsonetteApp: App {
    @AppStorage("theme") private var theme = "System"

    var body: some Scene {
        DocumentGroup(newDocument: JsonDocument()) { file in
            ContentView(document: file.$document)
                .preferredColorScheme(theme == "Light" ? .light : (theme == "Dark" ? .dark : nil))
        }
        .windowStyle(.hiddenTitleBar)
        .commands {
            CommandGroup(replacing: .pasteboard) {
                Button("Cut") { NSApp.sendAction(#selector(NSText.cut(_:)), to: nil, from: nil) }
                    .keyboardShortcut("x", modifiers: .command)
                Button("Copy") { NSApp.sendAction(#selector(NSText.copy(_:)), to: nil, from: nil) }
                    .keyboardShortcut("c", modifiers: .command)
                Button("Paste") { NSApp.sendAction(#selector(NSText.paste(_:)), to: nil, from: nil) }
                    .keyboardShortcut("v", modifiers: .command)
                Button("Cut Complete Line") { NSApp.sendAction(NSSelectorFromString("cutCompleteLine:"), to: nil, from: nil) }
                    .keyboardShortcut("x", modifiers: [.command, .shift])
                Button("Copy Complete Line") { NSApp.sendAction(NSSelectorFromString("copyCompleteLine:"), to: nil, from: nil) }
                    .keyboardShortcut("c", modifiers: [.command, .shift])
            }
            
            CommandMenu("Edit Actions") {
                Button("Move Line Up") { NSApp.sendAction(NSSelectorFromString("moveLineUp:"), to: nil, from: nil) }
                    .keyboardShortcut(.upArrow, modifiers: [.option])
                Button("Move Line Down") { NSApp.sendAction(NSSelectorFromString("moveLineDown:"), to: nil, from: nil) }
                    .keyboardShortcut(.downArrow, modifiers: [.option])
                Button("Duplicate Line Up") { NSApp.sendAction(NSSelectorFromString("duplicateLineUp:"), to: nil, from: nil) }
                    .keyboardShortcut(.upArrow, modifiers: [.option, .shift])
                Button("Duplicate Line Down") { NSApp.sendAction(NSSelectorFromString("duplicateLineDown:"), to: nil, from: nil) }
                    .keyboardShortcut(.downArrow, modifiers: [.option, .shift])
            }
            
            CommandMenu("View Actions") {
                Button("Collapse Node") { NSApp.sendAction(NSSelectorFromString("collapseNode:"), to: nil, from: nil) }
                    .keyboardShortcut("[", modifiers: [.command, .option])
                Button("Elapse Node") { NSApp.sendAction(NSSelectorFromString("elapseNode:"), to: nil, from: nil) }
                    .keyboardShortcut("]", modifiers: [.command, .option])
                Button("Collapse All") { NSApp.sendAction(NSSelectorFromString("collapseAll:"), to: nil, from: nil) }
                    .keyboardShortcut("[", modifiers: [.command, .option, .shift])
                Button("Elapse All") { NSApp.sendAction(NSSelectorFromString("elapseAll:"), to: nil, from: nil) }
                    .keyboardShortcut("]", modifiers: [.command, .option, .shift])
            }
        }
        
        #if os(macOS)
        Settings {
            SettingsView()
        }
        #endif
    }
}
