import SwiftUI

// Wrapper to provide a stable UUID for SwiftUI's OutlineGroup tracking
struct SymbolItem: Identifiable {
    let id = UUID() // Generated once upon creation
    let key: String
    let valueStr: String?
    let children: [SymbolItem]?
    
    init(node: FfiSymbolNode) {
        self.key = node.key
        self.valueStr = node.valueStr
        if let children = node.children, !children.isEmpty {
            self.children = children.map { SymbolItem(node: $0) }
        } else {
            self.children = nil
        }
    }
}

struct SymbolsPanelView: View {
    @Binding var document: JsonDocument
    
    var rootItems: [SymbolItem] {
        if let root = getAstSymbols(input: document.content), let children = root.children {
            return children.map { SymbolItem(node: $0) }
        }
        return []
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("Document Structure")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.secondary)
                Spacer()
            }
            .padding(.horizontal)
            .padding(.vertical, 8)
            .background(Color(NSColor.windowBackgroundColor))
            
            Divider()
            
            if rootItems.isEmpty {
                VStack {
                    Spacer()
                    Text("No JSON structure available")
                        .foregroundColor(.secondary)
                    Spacer()
                }
            } else {
                List(rootItems, children: \.children) { item in
                    HStack(spacing: 4) {
                        Text(item.key)
                            .fontWeight(.medium)
                            .foregroundColor(.primary)
                        
                        if let val = item.valueStr {
                            Text(": \(val)")
                                .foregroundColor(.secondary)
                        }
                    }
                    .font(.system(.body, design: .monospaced))
                    .padding(.vertical, 2)
                }
                .listStyle(.sidebar)
            }
        }
    }
}
