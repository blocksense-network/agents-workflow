//
//  AgentFsItem.swift
//  AgentFSKitExtension
//
//  Created by AgentFS on 2025-01-22.
//

@preconcurrency import Foundation
import FSKit

@available(macOS 15.4, *)
final class AgentFsItem: FSItem {

    private static let idCounter: OSAllocatedUnfairLock<UInt64> = {
        let lock = OSAllocatedUnfairLock<UInt64>(initialState: FSItem.Identifier.rootDirectory.rawValue + 1)
        return lock
    }()

    private static func getNextID() -> UInt64 {
        return idCounter.withLock { value in
            let current = value
            // Skip the root directory ID if we somehow reach it (wraparound protection)
            if current == FSItem.Identifier.rootDirectory.rawValue {
                value += 1
            }
            let result = current
            value += 1
            return result
        }
    }

    var name: FSFileName
    let id: UInt64

    // Path relative to volume root (e.g., "/foo/bar")
    var path: String

    var attributes = FSItem.Attributes()
    var xattrs: [FSFileName: Data] = [:]

    // Note: data and userData are kept for compatibility but should be moved to proper storage
    var data: Data?
    var userData: Any?

    init(name: FSFileName) {
        self.name = name
        self.id = AgentFsItem.getNextID()
        self.path = "/" // Default to root - must be set by caller using volume path builder

        // Initialize attributes after self is set up
        attributes.fileID = FSItem.Identifier(rawValue: id) ?? .invalid
        attributes.size = 0
        attributes.allocSize = 0
        attributes.flags = 0

        var timespec = timespec()
        timespec_get(&timespec, TIME_UTC)

        attributes.addedTime = timespec
        attributes.birthTime = timespec
        attributes.changeTime = timespec
        attributes.modifyTime = timespec
        attributes.accessTime = timespec
    }

    // Synchronous constructor with fixed ID
    init(name: FSFileName, id: UInt64) {
        self.name = name
        self.id = id
        self.path = "/" // Default to root - should be set by caller

        // Initialize attributes after self is set up
        attributes.fileID = FSItem.Identifier(rawValue: id) ?? .invalid
        attributes.size = 0
        attributes.allocSize = 0
        attributes.flags = 0

        var timespec = timespec()
        timespec_get(&timespec, TIME_UTC)

        attributes.addedTime = timespec
        attributes.birthTime = timespec
        attributes.changeTime = timespec
        attributes.modifyTime = timespec
        attributes.accessTime = timespec
    }

    // Create root item synchronously (special case)
    static func createRoot() -> AgentFsItem {
        let root = AgentFsItem(name: FSFileName(string: "/"), id: FSItem.Identifier.rootDirectory.rawValue)
        root.path = "/"
        root.attributes.parentID = FSItem.Identifier.parentOfRoot
        root.attributes.fileID = FSItem.Identifier.rootDirectory
        root.attributes.uid = 0
        root.attributes.gid = 0
        root.attributes.linkCount = 1
        root.attributes.type = FSItem.ItemType.directory
        root.attributes.mode = UInt32(S_IFDIR | 0o755)
        root.attributes.allocSize = 4096
        root.attributes.size = 4096
        return root
    }

}
