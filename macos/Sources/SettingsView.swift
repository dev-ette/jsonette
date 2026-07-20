import SwiftUI

struct SettingsView: View {
    // Engine Settings
    @AppStorage("formatOnSave") private var formatOnSave = true
    @AppStorage("formatOnPaste") private var formatOnPaste = false
    @AppStorage("useTabs") private var useTabs = false
    @AppStorage("indentWidth") private var indentWidth = 2
    @AppStorage("strictParsing") private var strictParsing = true
    
    // Editor Settings
    @AppStorage("theme") private var theme = "System"
    @AppStorage("showLineNumbers") private var showLineNumbers = true
    @AppStorage("showMinimap") private var showMinimap = false
    
    var body: some View {
        TabView {
            // MARK: - Editor Tab
            Form {
                Section(header: Text("Appearance")) {
                    Picker("Theme", selection: $theme) {
                        Text("System").tag("System")
                        Text("Light").tag("Light")
                        Text("Dark").tag("Dark")
                    }
                    .pickerStyle(.radioGroup)
                }
                
                Section(header: Text("Features")) {
                    Toggle("Show Line Numbers", isOn: $showLineNumbers)
                    Toggle("Show Minimap", isOn: $showMinimap)
                }
            }
            .padding()
            .tabItem {
                Label("Editor", systemImage: "macwindow")
            }
            
            // MARK: - Engine Tab
            Form {
                Section(header: Text("Formatting Engine")) {
                    Toggle("Format on Save", isOn: $formatOnSave)
                    Toggle("Format on Paste", isOn: $formatOnPaste)
                }
                
                Section(header: Text("Indentation")) {
                    Toggle("Use Tabs", isOn: $useTabs)
                    Stepper("Indent Width: \(indentWidth) spaces", value: $indentWidth, in: 1...8)
                        .disabled(useTabs)
                }
                
                Section(header: Text("Validation")) {
                    Toggle("Strict JSON Parsing", isOn: $strictParsing)
                        .help("If enabled, rejects trailing commas and single quotes.")
                }
            }
            .padding()
            .tabItem {
                Label("Engine", systemImage: "gearshape.2")
            }
            
            // MARK: - Keybindings Tab
            Form {
                VStack(alignment: .leading, spacing: 10) {
                    Text("Keyboard Shortcuts").font(.headline)
                    Text("• Format: Cmd + Shift + F")
                    Text("• Minify: Cmd + Shift + M")
                    Text("• Command Bar: Cmd + K")
                    Text("• Go to Line: Cmd + L")
                    Spacer()
                    Text("Custom keybinding configuration coming soon.")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
            .padding()
            .tabItem {
                Label("Keybindings", systemImage: "keyboard")
            }
        }
        .frame(width: 450, height: 350)
    }
}
