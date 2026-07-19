import SwiftUI

struct ContentView: View {
    @ObservedObject var documentManager: DocumentManager

    var body: some View {
        VStack(spacing: 0) {
            if !documentManager.diagnostics.isEmpty {
                VStack(alignment: .leading) {
                    ForEach(0..<documentManager.diagnostics.count, id: \.self) { i in
                        let diag = documentManager.diagnostics[i]
                        Text("Error: \(diag.message) (bytes \(diag.start)-\(diag.end))")
                            .foregroundColor(.white)
                            .font(.system(size: 12, weight: .bold, design: .monospaced))
                    }
                }
                .padding()
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color.red)
            }
            
            if documentManager.content.isEmpty && documentManager.diagnostics.isEmpty {
                Text("Open a JSON file (Cmd+O)")
                    .foregroundColor(.secondary)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                TextEditor(text: .constant(documentManager.content))
                    .font(.system(.body, design: .monospaced))
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .frame(minWidth: 600, minHeight: 400)
    }
}
