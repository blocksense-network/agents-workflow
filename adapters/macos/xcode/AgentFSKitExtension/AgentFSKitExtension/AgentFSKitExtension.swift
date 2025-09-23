//
//  AgentFSKitExtension.swift
//  AgentFSKitExtension
//
//  Created by AgentFS on 2025-01-22.
//

import Foundation
import FSKit

@available(macOS 15.4, *)
@main
struct AgentFSKitExtension : UnaryFileSystemExtension {

    var fileSystem : FSUnaryFileSystem & FSUnaryFileSystemOperations {
        AgentFsUnary()
    }
}
