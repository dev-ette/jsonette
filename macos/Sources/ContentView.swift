import SwiftUI
import AppKit
import UniformTypeIdentifiers

enum ViewMode: String, CaseIterable {
    case code = "Code"
    case split = "Split"
    case tree = "Tree"
}

struct ContentView: View {
    @Binding var document: JsonDocument
    
    // Core Vision States
    @State private var viewMode: ViewMode = .code
    @State private var isCommandBarOpen: Bool = false
    @State private var commandQuery: String = ""
    @State private var commandCompletions: [FfiCompletionItem] = []
    @State private var queryResult: FfiQueryResult? = nil
    
    @State private var showIssuesPopover: Bool = false
    @State private var showGeneratorSheet: Bool = false
    
    // Engine Settings tracking
    @AppStorage("formatOnSave") private var formatOnSave = true
    
    @State private var breadcrumbPath: [String] = ["root"]
    
    var body: some View {
        ZStack {
            // Main Content Area
            mainContent
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            
            // Floating Command Bar (Cmd+K style)
            if isCommandBarOpen {
                commandBarOverlay
                    .transition(.opacity.combined(with: .scale(scale: 0.95, anchor: .top)))
                    .zIndex(100)
            }
            
            // Floating Breadcrumbs
            VStack {
                Spacer()
                breadcrumbsOverlay
            }
            .padding(.bottom, 12)
            .zIndex(50)
        }
        .toolbar {
            ToolbarItem(placement: .principal) {
                Picker("View Mode", selection: $viewMode) {
                    ForEach(ViewMode.allCases, id: \.self) { mode in
                        Text(mode.rawValue).tag(mode)
                    }
                }
                .pickerStyle(.segmented)
                .frame(width: 200)
            }
            
            ToolbarItemGroup(placement: .primaryAction) {
                Button(action: { isCommandBarOpen.toggle() }) {
                    Label("Query / Command", systemImage: "magnifyingglass")
                }
                .keyboardShortcut("k", modifiers: .command)
                .help("Open Command Bar")
                
                Button(action: { showGeneratorSheet.toggle() }) {
                    Label("Generate", systemImage: "wand.and.stars")
                }
                .help("Data Generator Canvas")
                
                Menu {
                    Button("Format") { document.updateContent(formatJson(input: document.content)) }
                    Button("Minify") { document.updateContent(minifyJson(input: document.content)) }
                    Divider()
                    Button("Export as YAML") { exportData(format: "yaml") }
                    Button("Export as TOML") { exportData(format: "toml") }
                    Button("Export as XML") { exportData(format: "xml") }
                } label: {
                    Label("Actions", systemImage: "ellipsis.circle")
                }
                
                Button(action: { showIssuesPopover.toggle() }) {
                    Label("Issues", systemImage: document.diagnostics.isEmpty ? "checkmark.circle" : "exclamationmark.triangle.fill")
                        .foregroundColor(document.diagnostics.isEmpty ? .secondary : .yellow)
                }
                .popover(isPresented: $showIssuesPopover, arrowEdge: .bottom) {
                    issuesPopoverContent
                }
            }
        }
        .sheet(isPresented: $showGeneratorSheet) {
            GeneratorCanvasView(document: $document)
        }
        .frame(minWidth: 800, minHeight: 600)
        .background(Color(NSColor.windowBackgroundColor))
    }
    
    @ViewBuilder
    private var mainContent: some View {
        HStack(spacing: 0) {
            if viewMode == .code || viewMode == .split {
                // Code Editor
                JsonEditorView(document: $document)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
            
            if viewMode == .split {
                Divider()
            }
            
            if viewMode == .tree || viewMode == .split {
                // Interactive Tree View
                SymbolsPanelView(document: $document)
                    .frame(maxWidth: viewMode == .split ? 350 : .infinity, maxHeight: .infinity)
                    .background(Material.thick)
            }
        }
    }
    
    // MARK: - Overlays & Popovers
    
    private var commandBarOverlay: some View {
        VStack(spacing: 0) {
            HStack {
                Image(systemName: "magnifyingglass").foregroundColor(.secondary)
                TextField("Type JSONPath query ($.*)", text: $commandQuery)
                    .textFieldStyle(.plain)
                    .font(.system(size: 16, weight: .medium, design: .monospaced))
                    .onChange(of: commandQuery) { newValue in
                        updateCompletions(query: newValue)
                        queryResult = nil // Reset result on new typing
                    }
                    .onSubmit {
                        executeQuery()
                    }
                
                Button(action: { isCommandBarOpen = false }) {
                    Image(systemName: "xmark.circle.fill").foregroundColor(.secondary)
                }
                .buttonStyle(.plain)
            }
            .padding()
            
            if let result = queryResult {
                Divider()
                VStack(alignment: .leading) {
                    HStack {
                        Text(result.success ? "Result" : "Error")
                            .font(.caption)
                            .foregroundColor(result.success ? .green : .red)
                        Spacer()
                        if result.success {
                            Button("Copy") {
                                NSPasteboard.general.clearContents()
                                NSPasteboard.general.setString(result.output, forType: .string)
                            }
                            .controlSize(.mini)
                        }
                    }
                    ScrollView {
                        Text(result.output)
                            .font(.system(.body, design: .monospaced))
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .frame(maxHeight: 250)
                }
                .padding()
            } else if !commandCompletions.isEmpty {
                Divider()
                List(commandCompletions, id: \.self) { item in
                    Button(action: {
                        commandQuery = item.path
                        updateCompletions(query: commandQuery)
                    }) {
                        HStack {
                            Text(item.key).fontWeight(.bold)
                            Text(item.path).foregroundColor(.secondary)
                        }
                        .font(.system(.body, design: .monospaced))
                    }
                    .buttonStyle(.plain)
                }
                .frame(maxHeight: 200)
            }
        }
        .frame(width: 500)
        .background(Material.ultraThin)
        .cornerRadius(12)
        .shadow(color: Color.black.opacity(0.2), radius: 20, x: 0, y: 10)
        .padding(.top, 40)
    }
    
    private func updateCompletions(query: String) {
        if query.starts(with: "$") {
            commandCompletions = getCompletions(input: document.content, pathPrefix: query)
        } else {
            commandCompletions = []
        }
    }
    
    private func executeQuery() {
        if commandQuery.isEmpty { return }
        queryResult = queryJson(input: document.content, path: commandQuery)
    }
    
    private func exportData(format: String) {
        let result = convertJson(input: document.content, targetFormat: format)
        if result.success {
            let panel = NSSavePanel()
            panel.allowedContentTypes = [UTType(filenameExtension: format) ?? .plainText]
            panel.nameFieldStringValue = "export.\(format)"
            if panel.runModal() == .OK, let url = panel.url {
                try? result.output.write(to: url, atomically: true, encoding: .utf8)
            }
        }
    }
    
    private var breadcrumbsOverlay: some View {
        HStack(spacing: 4) {
            ForEach(Array(breadcrumbPath.enumerated()), id: \.offset) { index, component in
                Text(component)
                    .font(.system(size: 11, weight: .medium, design: .monospaced))
                    .foregroundColor(index == breadcrumbPath.count - 1 ? .primary : .secondary)
                
                if index < breadcrumbPath.count - 1 {
                    Image(systemName: "chevron.right")
                        .font(.system(size: 9, weight: .bold))
                        .foregroundColor(.secondary.opacity(0.5))
                }
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(Material.ultraThin)
        .clipShape(Capsule())
        .shadow(color: Color.black.opacity(0.1), radius: 5, x: 0, y: 2)
    }
    
    private var issuesPopoverContent: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("Diagnostics")
                .font(.headline)
                .padding()
            
            Divider()
            
            if document.diagnostics.isEmpty {
                Text("No issues found. JSON is valid.")
                    .foregroundColor(.secondary)
                    .padding()
            } else {
                List(document.diagnostics, id: \.start) { diag in
                    HStack(alignment: .top) {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundColor(.red)
                        VStack(alignment: .leading) {
                            Text(diag.message)
                                .font(.system(.subheadline, design: .monospaced))
                            Text("Bytes: \(diag.start)-\(diag.end)")
                                .font(.caption2)
                                .foregroundColor(.secondary)
                        }
                    }
                    .padding(.vertical, 4)
                }
            }
        }
        .frame(width: 300, height: 250)
    }
}
