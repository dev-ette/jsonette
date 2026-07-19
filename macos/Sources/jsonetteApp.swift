import SwiftUI

@main
struct jsonetteApp: App {
    @StateObject private var documentManager = DocumentManager()

    var body: some Scene {
        WindowGroup {
            ContentView(documentManager: documentManager)
        }
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("Open...") {
                    documentManager.openFile()
                }
                .keyboardShortcut("o", modifiers: [.command])
            }
        }
    }
}
