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

    @MainActor
    private static var id: UInt64 = FSItem.Identifier.rootDirectory.rawValue + 1
    private static let idLock = NSLock()

    @MainActor
    static func getNextID() -> UInt64 {
        idLock.lock()
        defer { idLock.unlock() }
        let current = id
        id += 1
        return current
    }

    let name: FSFileName
    let id: UInt64

    var attributes = FSItem.Attributes()
    var xattrs: [FSFileName: Data] = [:]
    var data: Data?

    private(set) var children: [FSFileName: AgentFsItem] = [:]

    init(name: FSFileName) async {
        self.name = name
        self.id = await AgentFsItem.getNextID()

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
        let root = AgentFsItem(name: FSFileName(string: "/"), id: 0)
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

    func addItem(_ item: AgentFsItem) {
        children[item.name] = item
    }

    func removeItem(_ item: AgentFsItem) {
        children[item.name] = nil
    }
}
