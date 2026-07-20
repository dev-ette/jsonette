import SwiftUI

struct JsonEditorView: View {
    @Binding var document: JsonDocument

    var body: some View {
        NativeJsonEditor(document: $document)
            .background(Color(NSColor.textBackgroundColor))
            // Send updates back to document so diagnostics run
            .onChange(of: document.content) { _, newValue in
                document.updateContent(newValue)
            }
    }
}
