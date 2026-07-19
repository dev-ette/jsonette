import XCTest
@testable import jsonette

final class jsonetteTests: XCTestCase {
    
    /// **Test Case**: DocumentManager processes valid JSON correctly
    ///
    /// ### Description
    /// Validates that calling the FFI engine with valid JSON returns no diagnostics.
    func testCheckSyntaxValid() throws {
        let manager = DocumentManager()
        let text = "{}"
        let diagnostics = checkSyntax(input: text)
        XCTAssertTrue(diagnostics.isEmpty, "Valid JSON should produce no diagnostics")
    }
    
    /// **Test Case**: DocumentManager identifies invalid JSON
    ///
    /// ### Description
    /// Validates that calling the FFI engine with invalid JSON returns diagnostics.
    func testCheckSyntaxInvalid() throws {
        let manager = DocumentManager()
        let text = "{"
        let diagnostics = checkSyntax(input: text)
        XCTAssertFalse(diagnostics.isEmpty, "Invalid JSON should produce diagnostics")
    }
}
