import XCTest
@testable import jsonette

final class jsonetteTests: XCTestCase {
    
    /// **Test Case**: JsonDocument initialization generates correct diagnostics for valid JSON
    ///
    /// ### Description
    /// Validates that creating a `JsonDocument` with valid JSON sets an empty diagnostics array.
    func testJsonDocumentInitializationValid() throws {
        let doc = JsonDocument(content: "{\"key\": \"value\"}")
        XCTAssertTrue(doc.diagnostics.isEmpty, "Valid JSON should produce no diagnostics")
    }
    
    /// **Test Case**: JsonDocument initialization generates correct diagnostics for invalid JSON
    ///
    /// ### Description
    /// Validates that creating a `JsonDocument` with invalid JSON parses diagnostics correctly.
    func testJsonDocumentInitializationInvalid() throws {
        let doc = JsonDocument(content: "{\"key\": ")
        XCTAssertFalse(doc.diagnostics.isEmpty, "Invalid JSON should produce diagnostics")
    }

    /// **Test Case**: JsonDocument updateContent properly recalculates diagnostics
    ///
    /// ### Description
    /// Validates that mutating `JsonDocument` content dynamically re-runs syntax checks.
    func testJsonDocumentUpdateContent() throws {
        var doc = JsonDocument(content: "{}")
        XCTAssertTrue(doc.diagnostics.isEmpty)
        
        doc.updateContent("{")
        XCTAssertFalse(doc.diagnostics.isEmpty, "Updating to invalid JSON must trigger diagnostics")
        
        doc.updateContent("[]")
        XCTAssertTrue(doc.diagnostics.isEmpty, "Updating to valid JSON must clear diagnostics")
    }
}
