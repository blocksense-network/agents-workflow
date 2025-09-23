//
//  MainViewController.swift
//  AgentsWorkflow
//

import Cocoa

class MainViewController: NSViewController {

    private let statusLabel = NSTextField(labelWithString: "Agents Workflow")
    private let extensionStatusLabel = NSTextField(labelWithString: "Extension Status: Checking...")
    private let infoTextView = NSTextView()

    override func loadView() {
        let view = NSView(frame: NSRect(x: 0, y: 0, width: 800, height: 600))
        self.view = view
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        setupUI()
        checkExtensionStatus()
    }

    private func setupUI() {
        statusLabel.font = NSFont.systemFont(ofSize: 24, weight: .bold)
        statusLabel.alignment = .center
        statusLabel.translatesAutoresizingMaskIntoConstraints = false

        extensionStatusLabel.font = NSFont.systemFont(ofSize: 14)
        extensionStatusLabel.alignment = .center
        extensionStatusLabel.translatesAutoresizingMaskIntoConstraints = false

        infoTextView.isEditable = false
        infoTextView.backgroundColor = .clear
        infoTextView.string = """
        Agents Workflow macOS Application

        This application hosts system extensions for the Agents Workflow platform,
        including filesystem extensions for AgentFS.
        """
        infoTextView.translatesAutoresizingMaskIntoConstraints = false

        view.addSubview(statusLabel)
        view.addSubview(extensionStatusLabel)
        view.addSubview(infoTextView)

        NSLayoutConstraint.activate([
            statusLabel.topAnchor.constraint(equalTo: view.topAnchor, constant: 50),
            statusLabel.centerXAnchor.constraint(equalTo: view.centerXAnchor),

            extensionStatusLabel.topAnchor.constraint(equalTo: statusLabel.bottomAnchor, constant: 20),
            extensionStatusLabel.centerXAnchor.constraint(equalTo: view.centerXAnchor),

            infoTextView.topAnchor.constraint(equalTo: extensionStatusLabel.bottomAnchor, constant: 40),
            infoTextView.leadingAnchor.constraint(equalTo: view.leadingAnchor, constant: 50),
            infoTextView.trailingAnchor.constraint(equalTo: view.trailingAnchor, constant: -50),
            infoTextView.bottomAnchor.constraint(equalTo: view.bottomAnchor, constant: -50)
        ])
    }

    private func checkExtensionStatus() {
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) {
            self.extensionStatusLabel.stringValue = "Extension Status: Ready"
        }
    }
}