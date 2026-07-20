import SwiftUI
import UniformTypeIdentifiers

struct JsonDocument: FileDocument {
    var content: String
    var diagnostics: [FfiDiagnostic]

    init(content: String = "{}") {
        self.content = content
        self.diagnostics = checkSyntax(input: content)
    }

    static var readableContentTypes: [UTType] { [.json] }

    init(configuration: ReadConfiguration) throws {
        guard let data = configuration.file.regularFileContents,
              let string = String(data: data, encoding: .utf8)
        else {
            throw CocoaError(.fileReadCorruptFile)
        }
        self.content = string
        self.diagnostics = checkSyntax(input: string)
    }
    
    /// Generates a FileWrapper for saving the document.
    /// - Parameter configuration: The write configuration.
    /// - Returns: A `FileWrapper` containing the UTF-8 encoded string data.
    func fileWrapper(configuration: WriteConfiguration) throws -> FileWrapper {
        var stringToSave = content
        
        // Handle Format on Save if enabled in settings
        let formatOnSave = UserDefaults.standard.bool(forKey: "formatOnSave")
        if formatOnSave {
            stringToSave = formatJson(input: stringToSave)
        }
        
        guard let data = stringToSave.data(using: .utf8) else {
            throw CocoaError(.fileWriteInapplicableStringEncoding)
        }
        return .init(regularFileWithContents: data)
    }
    
    mutating func updateContent(_ newContent: String) {
        content = newContent
        diagnostics = checkSyntax(input: newContent)
    }
}
