//
//  AgentFsUnary.swift
//  AgentFSKitExtension
//
//  Created by AgentFS on 2025-01-22.
//

import Foundation
import FSKit
import os

// Functions from AgentFSBridge.c are linked directly
@_silgen_name("agentfs_bridge_core_create")
func agentfs_bridge_core_create() -> UnsafeMutableRawPointer?

@_silgen_name("agentfs_bridge_core_destroy")
func agentfs_bridge_core_destroy(_ core: UnsafeMutableRawPointer?)

@available(macOS 15.4, *)
final class AgentFsUnary: FSUnaryFileSystem, FSUnaryFileSystemOperations {

    private let logger = Logger(subsystem: "com.agentfs.AgentFSKitExtension", category: "AgentFsUnary")

    // AgentFS core handle from Rust
    private var coreHandle: UnsafeMutableRawPointer?

    func probeResource(
        resource: FSResource,
        replyHandler: @escaping (FSProbeResult?, (any Error)?) -> Void
    ) {
        logger.debug("probeResource: \(resource, privacy: .public)")

        // For now, accept any block device resource
        replyHandler(
            FSProbeResult.usable(
                name: "AgentFS",
                containerID: FSContainerIdentifier(uuid: Constants.containerIdentifier)
            ),
            nil
        )
    }

    func loadResource(
        resource: FSResource,
        options: FSTaskOptions,
        replyHandler: @escaping (FSVolume?, (any Error)?) -> Void
    ) {
        containerStatus = .ready
        logger.debug("loadResource: \(resource, privacy: .public)")

        // Initialize AgentFS core instance
        self.coreHandle = agentfs_bridge_core_create()

        if self.coreHandle == nil {
            logger.error("Failed to create AgentFS core instance")
            replyHandler(nil, NSError(domain: "AgentFS", code: -1, userInfo: [NSLocalizedDescriptionKey: "Failed to initialize filesystem core"]))
            return
        }

        logger.info("AgentFS core initialized successfully")

        // Create volume (synchronous for now)
        let volume = AgentFsVolume(resource: resource, coreHandle: coreHandle)
        replyHandler(volume, nil)
    }

    func unloadResource(
        resource: FSResource,
        options: FSTaskOptions,
        replyHandler reply: @escaping ((any Error)?) -> Void
    ) {
        logger.debug("unloadResource: \(resource, privacy: .public)")

        // Cleanup AgentFS core instance
        if let handle = coreHandle {
            agentfs_bridge_core_destroy(handle)
            coreHandle = nil
            logger.info("AgentFS core cleaned up")
        }

        reply(nil)
    }

    func didFinishLoading() {
        logger.debug("didFinishLoading")
    }
}
