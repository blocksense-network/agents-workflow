//
//  AppDelegate.swift
//  AgentsWorkflow
//

import Cocoa

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
        print("Embedded system extensions registered")
    }
}