//
//  main.swift
//  AgentsWorkflow
//
//  Main entry point for the AgentsWorkflow macOS application.
//

import Cocoa

let delegate = AppDelegate()
NSApplication.shared.delegate = delegate

_ = NSApplicationMain(CommandLine.argc, CommandLine.unsafeArgv)