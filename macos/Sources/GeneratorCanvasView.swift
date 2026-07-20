import SwiftUI

struct GeneratorCanvasView: View {
    @Binding var document: JsonDocument
    @Environment(\.dismiss) var dismiss
    
    @State private var schemaInput: String = "{\n  \"id\": { \"@type\": \"uuid\" }\n}"
    @State private var targetCount: String = "10"
    @State private var targetSizeKb: String = ""
    @State private var isGenerating: Bool = false
    @State private var errorMessage: String? = nil
    
    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                VStack(alignment: .leading) {
                    Text("Data Generator Canvas")
                        .font(.title2)
                        .fontWeight(.bold)
                    Text("Generate massive JSON fixtures using a schema template.")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                Spacer()
                Button(action: { dismiss() }) {
                    Image(systemName: "xmark.circle.fill")
                        .font(.title2)
                        .foregroundColor(.secondary)
                }
                .buttonStyle(.plain)
            }
            .padding()
            .background(Color(NSColor.windowBackgroundColor))
            
            Divider()
            
            // Layout
            HSplitView {
                // Left: Schema Editor
                VStack(alignment: .leading) {
                    Text("Schema Template")
                        .font(.headline)
                        .padding(.horizontal)
                        .padding(.top, 8)
                    TextEditor(text: $schemaInput)
                        .font(.system(.body, design: .monospaced))
                        .padding()
                        .background(Color(NSColor.textBackgroundColor))
                        .cornerRadius(8)
                        .padding()
                }
                .frame(minWidth: 300)
                
                // Right: Controls & Output
                VStack(alignment: .leading, spacing: 16) {
                    Text("Configuration")
                        .font(.headline)
                    
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Target Array Count")
                        TextField("e.g. 100", text: $targetCount)
                            .textFieldStyle(.roundedBorder)
                        
                        Text("OR Target Size (KB)")
                        TextField("e.g. 1024", text: $targetSizeKb)
                            .textFieldStyle(.roundedBorder)
                    }
                    
                    if let error = errorMessage {
                        Text(error)
                            .foregroundColor(.red)
                            .font(.caption)
                    }
                    
                    Spacer()
                    
                    HStack {
                        Spacer()
                        Button(action: generate) {
                            if isGenerating {
                                ProgressView().controlSize(.small)
                            }
                            Text("Generate JSON")
                                .fontWeight(.medium)
                        }
                        .buttonStyle(.borderedProminent)
                        .controlSize(.large)
                        .disabled(isGenerating)
                    }
                }
                .padding()
                .frame(minWidth: 250, maxWidth: 350)
                .background(Color(NSColor.windowBackgroundColor).opacity(0.5))
            }
        }
        .frame(minWidth: 800, minHeight: 600)
    }
    
    private func generate() {
        isGenerating = true
        errorMessage = nil
        
        let count = UInt32(targetCount)
        let size = UInt32(targetSizeKb)
        
        // Push generation to background to keep UI responsive
        DispatchQueue.global(qos: .userInitiated).async {
            let result = generateDummyData(schemaInput: schemaInput, targetSizeKb: size, targetCount: count)
            
            DispatchQueue.main.async {
                isGenerating = false
                if result.success {
                    document.updateContent(result.output)
                    dismiss()
                } else {
                    errorMessage = result.output
                }
            }
        }
    }
}
