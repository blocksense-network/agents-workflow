//
//  AppDelegate.swift
//  AgentsWorkflow
//

import Cocoa

// Import SystemExtensions only if available (macOS 10.15+)
#if canImport(SystemExtensions)
import SystemExtensions
#endif

class AppDelegate: NSObject, NSApplicationDelegate {

    var window: NSWindow?

    func applicationDidFinishLaunching(_ aNotification: Notification) {
        createMainWindow()
        registerExtensions()
        print("AgentsWorkflow application started successfully")
    }

    func applicationWillTerminate(_ aNotification: Notification) {
        print("AgentsWorkflow application terminating")
    }

    private func createMainWindow() {
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 800, height: 600),
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )

        window.center()
        window.title = "Agents Workflow"
        window.makeKeyAndOrderFront(nil)

        let viewController = MainViewController()
        window.contentViewController = viewController

        self.window = window
    }

    private func registerExtensions() {
        print("Registering embedded system extensions...")

        // Find the extension bundle
        guard let extensionBundleURL = Bundle.main.builtInPlugInsURL?.appendingPathComponent("AgentFSKitExtension.appex") else {
            print("Error: Could not find AgentFSKitExtension bundle")
            return
        }

        print("Found extension bundle at: \(extensionBundleURL.path)")

        // System extensions are automatically registered when the app launches
        // The system will detect the embedded extension and request user approval if needed
        #if canImport(SystemExtensions)
        if #available(macOS 13.0, *) {
            print("macOS 13.0+ detected - system will handle extension registration automatically")
        } else if #available(macOS 10.15, *) {
            print("macOS 10.15-12.x detected - extension requires manual approval in System Settings")
        }
        #else
        print("SystemExtensions framework not available - extension may require manual approval")
        #endif

        print("Extension registration complete - system will handle approval workflow")
    }
}