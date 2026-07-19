import Foundation
import AppKit

class DocumentManager: ObservableObject {
    @Published var content: String = ""
    @Published var diagnostics: [FfiDiagnostic] = []
    
    func openFile() {
        let panel = NSOpenPanel()
        panel.allowedFileTypes = ["json"]
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        
        if panel.runModal() == .OK, let url = panel.url {
            DispatchQueue.global(qos: .userInitiated).async {
                do {
                    let text = try String(contentsOf: url, encoding: .utf8)
                    let diags = checkSyntax(input: text)
                    DispatchQueue.main.async {
                        self.content = text
                        self.diagnostics = diags
                    }
                } catch {
                    DispatchQueue.main.async {
                        self.diagnostics = [FfiDiagnostic(start: 0, end: 0, message: "Failed to read file: \(error.localizedDescription)")]
                        self.content = ""
                    }
                }
            }
        }
    }
}
