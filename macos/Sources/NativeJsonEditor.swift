import SwiftUI
import AppKit

struct NativeJsonEditor: NSViewRepresentable {
    @Binding var document: JsonDocument
    
    func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }

    func makeNSView(context: Context) -> NSScrollView {
        let scrollView = NSScrollView()
        scrollView.hasVerticalScroller = true
        scrollView.hasHorizontalScroller = true
        scrollView.autohidesScrollers = true
        
        let textView = CustomTextView()
        textView.customDelegate = context.coordinator
        textView.delegate = context.coordinator
        textView.isRichText = false
        textView.isAutomaticQuoteSubstitutionEnabled = false
        textView.isAutomaticDashSubstitutionEnabled = false
        textView.isAutomaticSpellingCorrectionEnabled = false
        textView.isContinuousSpellCheckingEnabled = false
        textView.font = NSFont.monospacedSystemFont(ofSize: 13, weight: .regular)
        textView.autoresizingMask = [.width, .height]
        textView.textContainer?.containerSize = NSSize(width: CGFloat.greatestFiniteMagnitude, height: CGFloat.greatestFiniteMagnitude)
        textView.textContainer?.widthTracksTextView = true
        textView.isHorizontallyResizable = false
        textView.isVerticallyResizable = true
        
        scrollView.documentView = textView
        
        return scrollView
    }

    func updateNSView(_ nsView: NSScrollView, context: Context) {
        guard let textView = nsView.documentView as? CustomTextView else { return }
        
        // Preserve selection and scroll position
        let selectedRanges = textView.selectedRanges
        let scrollRect = nsView.contentView.bounds
        
        if textView.string != document.content {
            // Apply very basic highlighting for JSON
            let attrStr = highlightJSON(document.content)
            textView.textStorage?.setAttributedString(attrStr)
            
            // Restore selection if bounds make sense
            if let firstRange = selectedRanges.first as? NSRange, firstRange.location + firstRange.length <= document.content.utf16.count {
                textView.selectedRanges = selectedRanges
            }
            
            nsView.contentView.scroll(to: scrollRect.origin)
        }
    }
    
    class CustomTextView: NSTextView {
        weak var customDelegate: Coordinator?
        
        override func paste(_ sender: Any?) {
            super.paste(sender)
            customDelegate?.handlePaste(in: self)
        }
    }
    
    private func highlightJSON(_ input: String) -> NSAttributedString {
        let attrString = NSMutableAttributedString(string: input)
        let fullRange = NSRange(location: 0, length: input.utf16.count)
        
        // Base styling
        let baseFont = NSFont.monospacedSystemFont(ofSize: 13, weight: .regular)
        let textColor = NSColor.textColor
        attrString.addAttribute(.font, value: baseFont, range: fullRange)
        attrString.addAttribute(.foregroundColor, value: textColor, range: fullRange)
        
        do {
            // 1. All Strings (Assume Values first) -> Green
            let stringRegex = try NSRegularExpression(pattern: "\"(?:\\\\\"|[^\"])*\"", options: [])
            stringRegex.enumerateMatches(in: input, range: fullRange) { match, _, _ in
                if let matchRange = match?.range {
                    attrString.addAttribute(.foregroundColor, value: NSColor.systemGreen, range: matchRange)
                }
            }
            
            // 2. Keys (Strings followed by a colon) -> Teal
            let keyRegex = try NSRegularExpression(pattern: "(\"(?:\\\\\"|[^\"])*\")\\s*:", options: [])
            keyRegex.enumerateMatches(in: input, range: fullRange) { match, _, _ in
                if let keyRange = match?.range(at: 1) {
                    attrString.addAttribute(.foregroundColor, value: NSColor.systemTeal, range: keyRange)
                    // Optional: Make keys bold
                    // attrString.addAttribute(.font, value: NSFont.monospacedSystemFont(ofSize: 13, weight: .bold), range: keyRange)
                }
            }
            
            // 3. Numbers -> Orange
            let numRegex = try NSRegularExpression(pattern: "\\b-?[0-9]+(?:\\.[0-9]+)?(?:[eE][+-]?[0-9]+)?\\b", options: [])
            numRegex.enumerateMatches(in: input, range: fullRange) { match, _, _ in
                if let matchRange = match?.range {
                    attrString.addAttribute(.foregroundColor, value: NSColor.systemOrange, range: matchRange)
                }
            }
            
            // 4. Booleans / Null -> Pink
            let boolRegex = try NSRegularExpression(pattern: "\\b(true|false|null)\\b", options: [])
            boolRegex.enumerateMatches(in: input, range: fullRange) { match, _, _ in
                if let matchRange = match?.range {
                    attrString.addAttribute(.foregroundColor, value: NSColor.systemPink, range: matchRange)
                    attrString.addAttribute(.font, value: NSFont.monospacedSystemFont(ofSize: 13, weight: .bold), range: matchRange)
                }
            }
            
            // 5. Rainbow Brackets
            let bracketColors: [NSColor] = [
                NSColor.systemYellow,
                NSColor.systemPurple,
                NSColor.systemMint
            ]
            
            var depth = 0
            var inString = false
            var escapeNext = false
            
            // Fast UTF16 iteration to match NSRange lengths
            let utf16 = input.utf16
            var index = 0
            
            for scalar in utf16 {
                let char = UnicodeScalar(scalar)
                
                if escapeNext {
                    escapeNext = false
                } else if char == "\\" {
                    escapeNext = true
                } else if char == "\"" {
                    inString.toggle()
                } else if !inString {
                    if char == "{" || char == "[" {
                        let color = bracketColors[depth % 3]
                        attrString.addAttribute(.foregroundColor, value: color, range: NSRange(location: index, length: 1))
                        depth += 1
                    } else if char == "}" || char == "]" {
                        depth = max(0, depth - 1)
                        let color = bracketColors[depth % 3]
                        attrString.addAttribute(.foregroundColor, value: color, range: NSRange(location: index, length: 1))
                    }
                }
                index += 1
            }
            
        } catch {}
        
        return attrString
    }

    class Coordinator: NSObject, NSTextViewDelegate {
        var parent: NativeJsonEditor

        init(_ parent: NativeJsonEditor) {
            self.parent = parent
        }

        func textDidChange(_ notification: Notification) {
            guard let textView = notification.object as? NSTextView else { return }
            self.parent.document.updateContent(textView.string)
        }
        
        func handlePaste(in textView: NSTextView) {
            if UserDefaults.standard.bool(forKey: "formatOnPaste") {
                let formatted = formatJson(input: textView.string)
                if formatted != textView.string {
                    textView.string = formatted
                    self.parent.document.updateContent(formatted)
                }
            }
        }
    }
}
