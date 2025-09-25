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

    // XPC control service
    private var xpcService: AgentFSControlService?

    func probeResource(
        resource: FSResource,
        replyHandler: @escaping (FSProbeResult?, (any Error)?) -> Void
    ) {
        logger.debug("probeResource: \(resource, privacy: .public)")

        // AgentFS can work with any resource as backing store
        // For now, we accept any resource and treat it as usable
        // In a full implementation, we might check for existing AgentFS metadata
        // or validate the resource characteristics

        logger.debug("Probing resource: \(resource)")

        // For AgentFS, we can work with any resource type
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

        // Initialize XPC control service
        self.xpcService = AgentFSControlService(coreHandle: coreHandle)
        logger.info("XPC control service initialized")

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

        // Cleanup XPC service first
        self.xpcService = nil
        logger.info("XPC control service cleaned up")

        // Cleanup AgentFS core instance
        if let handle = coreHandle {
            agentfs_bridge_core_destroy(handle)
            coreHandle = nil
            logger.info("AgentFS core cleaned up")
        } else {
            logger.warning("unloadResource: no core handle to cleanup")
        }

        // Container status is managed automatically by FSKit
        logger.info("Container marked as unloaded")

        reply(nil)
    }

    func didFinishLoading() {
        logger.debug("didFinishLoading")
    }
}
