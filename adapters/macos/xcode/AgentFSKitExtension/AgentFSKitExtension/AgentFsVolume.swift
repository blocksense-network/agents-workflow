//
//  AgentFsVolume.swift
//  AgentFSKitExtension
//
//  Created by AgentFS on 2025-01-22.
//

import Foundation
import FSKit
import os

@available(macOS 15.4, *)
final class AgentFsVolume: FSVolume {

    private let resource: FSResource
    private let coreHandle: UnsafeMutableRawPointer?

    private let logger = Logger(subsystem: "com.agentfs.AgentFSKitExtension", category: "AgentFsVolume")

    private let root: AgentFsItem

    init(resource: FSResource, coreHandle: UnsafeMutableRawPointer?) {
        self.resource = resource
        self.coreHandle = coreHandle

        // Create root item with fixed attributes
        self.root = AgentFsItem.createRoot()

        super.init(
            volumeID: FSVolume.Identifier(uuid: Constants.volumeIdentifier),
            volumeName: FSFileName(string: "AgentFS")
        )
    }

}

// MARK: - FSVolume.PathConfOperations
extension AgentFsVolume: FSVolume.PathConfOperations {

    var maximumLinkCount: Int {
        return -1
    }

    var maximumNameLength: Int {
        return -1
    }

    var restrictsOwnershipChanges: Bool {
        return false
    }

    var truncatesLongNames: Bool {
        return false
    }

    var maximumXattrSize: Int {
        return Int.max
    }

    var maximumFileSize: UInt64 {
        return UInt64.max
    }
}

// MARK: - FSVolume.Operations
@available(macOS 15.4, *)
extension AgentFsVolume: FSVolume.Operations {

    var supportedVolumeCapabilities: FSVolume.SupportedCapabilities {
        logger.debug("supportedVolumeCapabilities")

        let capabilities = FSVolume.SupportedCapabilities()
        capabilities.supportsHardLinks = true
        capabilities.supportsSymbolicLinks = true
        capabilities.supportsPersistentObjectIDs = true
        capabilities.doesNotSupportVolumeSizes = true
        capabilities.supportsHiddenFiles = true
        capabilities.supports64BitObjectIDs = true
        capabilities.caseFormat = .insensitiveCasePreserving // AgentFS will be case sensitive
        return capabilities
    }

    var volumeStatistics: FSStatFSResult {
        logger.debug("volumeStatistics")

        let result = FSStatFSResult(fileSystemTypeName: "AgentFS")

        result.blockSize = 4096
        result.ioSize = 4096
        result.totalBlocks = 1024 * 1024 // 4GB
        result.availableBlocks = 1024 * 1024
        result.freeBlocks = 1024 * 1024
        result.totalFiles = 1024 * 1024
        result.freeFiles = 1024 * 1024

        return result
    }

    func activate(options: FSTaskOptions) async throws -> FSItem {
        logger.debug("activate")
        return root
    }

    func deactivate(options: FSDeactivateOptions = []) async throws {
        logger.debug("deactivate")
    }

    func mount(options: FSTaskOptions) async throws {
        logger.debug("mount")
        // TODO: Initialize volume state
    }

    func unmount() async {
        logger.debug("unmount")
        // TODO: Cleanup volume state
    }

    func synchronize(flags: FSSyncFlags) async throws {
        logger.debug("synchronize")
        // TODO: Implement synchronization
    }

    func attributes(
        _ desiredAttributes: FSItem.GetAttributesRequest,
        of item: FSItem
    ) async throws -> FSItem.Attributes {
        guard let item = item as? AgentFsItem else {
            logger.debug("getItemAttributes: invalid item type")
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        logger.debug("getItemAttributes: \(item.name), \(desiredAttributes)")

        // For now, return cached attributes
        // TODO: In the future, call agentfs_getattr via FFI bridge
        return item.attributes
    }

    func setAttributes(
        _ newAttributes: FSItem.SetAttributesRequest,
        on item: FSItem
    ) async throws -> FSItem.Attributes {
        logger.debug("setItemAttributes: \(item), \(newAttributes)")
        if let item = item as? AgentFsItem {
            // TODO: Implement attribute setting (M15)
            mergeAttributes(item.attributes, request: newAttributes)
            return item.attributes
        } else {
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }
    }

    func lookupItem(
        named name: FSFileName,
        inDirectory directory: FSItem
    ) async throws -> (FSItem, FSFileName) {
        logger.debug("lookupName: \(String(describing: name.string)), \(directory)")

        guard let directory = directory as? AgentFsItem else {
            throw fs_errorForPOSIXError(POSIXError.ENOENT.rawValue)
        }

        // Handle special cases
        if let nameStr = name.string {
            if nameStr == "." {
                return (directory, name)
            } else if nameStr == ".." {
                // For now, everything's parent is root
                return (root, name)
            }
        }

        // TODO: Implement full path resolution via bridge (M15)
        // For now, only allow access to root directory contents
        if directory === root {
            // Check if it's a known item in the root
            // This is a placeholder - real implementation would call bridge
            if let nameStr = name.string {
                if nameStr == "test" {
                    let testItem = await AgentFsItem(name: FSFileName(string: "test"))
                    testItem.attributes.parentID = root.attributes.fileID
                    testItem.attributes.fileID = FSItem.Identifier(rawValue: await AgentFsItem.getNextID()) ?? .invalid
                    testItem.attributes.type = FSItem.ItemType.file
                    testItem.attributes.mode = UInt32(S_IFREG | 0o644)
                    testItem.attributes.size = 0
                    testItem.attributes.allocSize = 0
                    return (testItem, name)
                } else if nameStr == ".agentfs" {
                    // Control directory for CLI operations (M16)
                    let controlDir = await AgentFsItem(name: FSFileName(string: ".agentfs"))
                    controlDir.attributes.parentID = root.attributes.fileID
                    controlDir.attributes.fileID = FSItem.Identifier(rawValue: await AgentFsItem.getNextID()) ?? .invalid
                    controlDir.attributes.type = FSItem.ItemType.directory
                    controlDir.attributes.mode = UInt32(S_IFDIR | 0o755)
                    controlDir.attributes.size = 4096
                    controlDir.attributes.allocSize = 4096
                    return (controlDir, name)
                }
            }
        }

        // Handle lookup within .agentfs control directory
        if let directoryName = directory.name.string, directoryName == ".agentfs" {
            if let nameStr = name.string {
                // Control files for CLI operations
                if nameStr == "snapshot" || nameStr == "branch" || nameStr == "bind" {
                    let controlFile = await AgentFsItem(name: name)
                    controlFile.attributes.parentID = directory.attributes.fileID
                    controlFile.attributes.fileID = FSItem.Identifier(rawValue: await AgentFsItem.getNextID()) ?? .invalid
                    controlFile.attributes.type = FSItem.ItemType.file
                    controlFile.attributes.mode = UInt32(S_IFREG | 0o644)
                    controlFile.attributes.size = 0
                    controlFile.attributes.allocSize = 0
                    return (controlFile, name)
                }
            }
        }

        throw fs_errorForPOSIXError(POSIXError.ENOENT.rawValue)
    }

    func reclaimItem(_ item: FSItem) async throws {
        logger.debug("reclaimItem: \(item)")
        // TODO: Implement reclamation
    }

    func readSymbolicLink(
        _ item: FSItem
    ) async throws -> FSFileName {
        logger.debug("readSymbolicLink: \(item)")
        // TODO: Implement symlink reading
        throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
    }

    func createItem(
        named name: FSFileName,
        type: FSItem.ItemType,
        inDirectory directory: FSItem,
        attributes newAttributes: FSItem.SetAttributesRequest
    ) async throws -> (FSItem, FSFileName) {
        logger.debug("createItem: \(String(describing: name.string)) - \(newAttributes.mode)")

        guard let directory = directory as? AgentFsItem,
              let nameStr = name.string,
              let coreHandle = coreHandle else {
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        // Build full path
        let dirPath = directory.name.string ?? "/"
        let fullPath = dirPath == "/" ? "/\(nameStr)" : "\(dirPath)/\(nameStr)"

        // TODO: Call agentfs_create via FFI bridge
        // For now, create a stub item
        let item = await AgentFsItem(name: name)
        mergeAttributes(item.attributes, request: newAttributes)
        item.attributes.parentID = directory.attributes.fileID
        item.attributes.fileID = FSItem.Identifier(rawValue: await AgentFsItem.getNextID()) ?? .invalid
        item.attributes.type = type

        // Set basic attributes
        if type == .directory {
            item.attributes.mode = UInt32(S_IFDIR | 0o755)
            item.attributes.size = 4096
            item.attributes.allocSize = 4096
        } else {
            item.attributes.mode = UInt32(S_IFREG | 0o644)
            item.attributes.size = 0
            item.attributes.allocSize = 0
        }

        directory.addItem(item)

        return (item, name)
    }

    func createSymbolicLink(
        named name: FSFileName,
        inDirectory directory: FSItem,
        attributes newAttributes: FSItem.SetAttributesRequest,
        linkContents contents: FSFileName
    ) async throws -> (FSItem, FSFileName) {
        logger.debug("createSymbolicLink: \(name)")
        // TODO: Implement symlink creation
        throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
    }

    func createLink(
        to item: FSItem,
        named name: FSFileName,
        inDirectory directory: FSItem
    ) async throws -> FSFileName {
        logger.debug("createLink: \(name)")
        // TODO: Implement hard link creation
        throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
    }

    func removeItem(
        _ item: FSItem,
        named name: FSFileName,
        fromDirectory directory: FSItem
    ) async throws {
        logger.debug("remove: \(name)")
        // TODO: Implement removal
        throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
    }

    func renameItem(
        _ item: FSItem,
        inDirectory sourceDirectory: FSItem,
        named sourceName: FSFileName,
        to destinationName: FSFileName,
        inDirectory destinationDirectory: FSItem,
        overItem: FSItem?
    ) async throws -> FSFileName {
        logger.debug("rename: \(item)")
        // TODO: Implement renaming
        throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
    }

    func enumerateDirectory(
        _ directory: FSItem,
        startingAt cookie: FSDirectoryCookie,
        verifier: FSDirectoryVerifier,
        attributes: FSItem.GetAttributesRequest?,
        packer: FSDirectoryEntryPacker
    ) async throws -> FSDirectoryVerifier {
        logger.debug("enumerateDirectory: \(directory)")

        guard let directory = directory as? AgentFsItem else {
            throw fs_errorForPOSIXError(POSIXError.ENOENT.rawValue)
        }

        logger.debug("- enumerateDirectory - \(directory.name)")

        // For now, provide a minimal directory listing
        if directory === root {
            // Add "." entry
            _ = packer.packEntry(
                name: FSFileName(string: "."),
                itemType: .directory,
                itemID: directory.attributes.fileID,
                nextCookie: FSDirectoryCookie(1),
                attributes: attributes != nil ? directory.attributes : nil
            )

            // Add ".." entry (also points to root for simplicity)
            _ = packer.packEntry(
                name: FSFileName(string: ".."),
                itemType: .directory,
                itemID: directory.attributes.fileID,
                nextCookie: FSDirectoryCookie(2),
                attributes: attributes != nil ? directory.attributes : nil
            )

            // Add .agentfs control directory (M16)
            let controlDirID = await AgentFsItem.getNextID()
            let controlAttrs = FSItem.Attributes()
            controlAttrs.fileID = FSItem.Identifier(rawValue: controlDirID) ?? .invalid
            controlAttrs.parentID = directory.attributes.fileID
            controlAttrs.type = .directory
            controlAttrs.mode = UInt32(S_IFDIR | 0o755)
            controlAttrs.size = 4096
            controlAttrs.allocSize = 4096

            _ = packer.packEntry(
                name: FSFileName(string: ".agentfs"),
                itemType: .directory,
                itemID: FSItem.Identifier(rawValue: controlDirID) ?? .invalid,
                nextCookie: FSDirectoryCookie(3),
                attributes: attributes != nil ? controlAttrs : nil
            )

            // Add a test file
            let testItemID = await AgentFsItem.getNextID()
            let testAttrs = FSItem.Attributes()
            testAttrs.fileID = FSItem.Identifier(rawValue: testItemID) ?? .invalid
            testAttrs.parentID = directory.attributes.fileID
            testAttrs.type = .file
            testAttrs.mode = UInt32(S_IFREG | 0o644)
            testAttrs.size = 0
            testAttrs.allocSize = 0

            _ = packer.packEntry(
                name: FSFileName(string: "test"),
                itemType: .file,
                itemID: FSItem.Identifier(rawValue: testItemID) ?? .invalid,
                nextCookie: FSDirectoryCookie(4),
                attributes: attributes != nil ? testAttrs : nil
            )
        } else if let directoryName = directory.name.string, directoryName == ".agentfs" {
            // List control files in .agentfs directory
            // Add "." entry
            _ = packer.packEntry(
                name: FSFileName(string: "."),
                itemType: .directory,
                itemID: directory.attributes.fileID,
                nextCookie: FSDirectoryCookie(1),
                attributes: attributes != nil ? directory.attributes : nil
            )

            // Add control files
            let controlFiles = ["snapshot", "branch", "bind"]
            for (index, fileName) in controlFiles.enumerated() {
                let fileID = await AgentFsItem.getNextID()
                let fileAttrs = FSItem.Attributes()
                fileAttrs.fileID = FSItem.Identifier(rawValue: fileID) ?? .invalid
                fileAttrs.parentID = directory.attributes.fileID
                fileAttrs.type = .file
                fileAttrs.mode = UInt32(S_IFREG | 0o644)
                fileAttrs.size = 0
                fileAttrs.allocSize = 0

                _ = packer.packEntry(
                    name: FSFileName(string: fileName),
                    itemType: .file,
                    itemID: FSItem.Identifier(rawValue: fileID) ?? .invalid,
                    nextCookie: FSDirectoryCookie(UInt64(index + 2)),
                    attributes: attributes != nil ? fileAttrs : nil
                )
            }
        }

        return FSDirectoryVerifier(0)
    }

    private func mergeAttributes(_ existing: FSItem.Attributes, request: FSItem.SetAttributesRequest) {
        if request.isValid(FSItem.Attribute.uid) {
            existing.uid = request.uid
        }

        if request.isValid(FSItem.Attribute.gid) {
            existing.gid = request.gid
        }

        if request.isValid(FSItem.Attribute.type) {
            existing.type = request.type
        }

        if request.isValid(FSItem.Attribute.mode) {
            existing.mode = request.mode
        }

        if request.isValid(FSItem.Attribute.linkCount) {
            existing.linkCount = request.linkCount
        }

        if request.isValid(FSItem.Attribute.flags) {
            existing.flags = request.flags
        }

        if request.isValid(FSItem.Attribute.size) {
            existing.size = request.size
        }

        if request.isValid(FSItem.Attribute.allocSize) {
            existing.allocSize = request.allocSize
        }

        if request.isValid(FSItem.Attribute.fileID) {
            existing.fileID = request.fileID
        }

        if request.isValid(FSItem.Attribute.parentID) {
            existing.parentID = request.parentID
        }

        // Handle timestamps
        let now = timespec()
        if request.isValid(FSItem.Attribute.accessTime) {
            existing.accessTime = now
        }

        if request.isValid(FSItem.Attribute.changeTime) {
            existing.changeTime = now
        }

        if request.isValid(FSItem.Attribute.modifyTime) {
            existing.modifyTime = now
        }

        if request.isValid(FSItem.Attribute.addedTime) {
            existing.addedTime = now
        }

        if request.isValid(FSItem.Attribute.birthTime) {
            existing.birthTime = now
        }

        if request.isValid(FSItem.Attribute.backupTime) {
            existing.backupTime = now
        }
    }
}

// MARK: - FSVolume.ReadWriteOperations
@available(macOS 15.4, *)
extension AgentFsVolume: FSVolume.ReadWriteOperations {

    func read(from item: FSItem, at offset: off_t, length: Int, into buffer: FSMutableFileDataBuffer) async throws -> Int {
        logger.debug("read: \(item) offset: \(offset), length: \(length)")

        guard let item = item as? AgentFsItem,
              let itemName = item.name.string,
              let coreHandle = coreHandle else {
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        // Check if this is a control file operation
        if isControlFile(itemName) {
            // Control files don't have readable content
            return 0
        }

        // TODO: Call agentfs_read via FFI bridge
        // For now, return data from item's data buffer if available
        if let data = item.data, offset < data.count {
            let availableData = data.suffix(from: Int(offset))
            let bytesToRead = min(length, availableData.count)

            availableData.withUnsafeBytes { srcPtr in
                buffer.withUnsafeMutableBytes { dstPtr in
                    memcpy(dstPtr.baseAddress, srcPtr.baseAddress, bytesToRead)
                }
            }

            return bytesToRead
        }

        return 0
    }

    func write(contents: Data, to item: FSItem, at offset: off_t) async throws -> Int {
        logger.debug("write: \(item) - offset: \(offset), size: \(contents.count)")

        guard let item = item as? AgentFsItem,
              let itemName = item.name.string,
              let coreHandle = coreHandle else {
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        // Check if this is a control file operation (M16)
        if isControlFile(itemName) {
            return try await handleControlFileWrite(item, itemName: itemName, contents: contents, offset: offset)
        }

        // TODO: Call agentfs_write via FFI bridge
        // For now, store data in item's data buffer
        if offset == 0 {
            // Overwrite
            item.data = contents
        } else {
            // Append/extend - simple implementation for now
            if item.data == nil {
                item.data = Data(count: Int(offset))
            }
            if let existingData = item.data, Int(offset) > existingData.count {
                // Extend with zeros
                item.data = existingData + Data(count: Int(offset) - existingData.count)
            }
            if var data = item.data {
                data.replaceSubrange(Int(offset)..<Int(offset) + contents.count, with: contents)
                item.data = data
            }
        }

        item.attributes.size = UInt64(item.data?.count ?? 0)
        item.attributes.allocSize = item.attributes.size

        return contents.count
    }

    private func isControlFile(_ name: String) -> Bool {
        // Check if this item is in the .agentfs directory
        return name == "snapshot" || name == "branch" || name == "bind"
    }

    private func handleControlFileWrite(_ item: AgentFsItem, itemName: String, contents: Data, offset: off_t) async throws -> Int {
        logger.debug("Control file write: \(itemName)")

        // For control files, we expect JSON commands
        // Accumulate the written data and process complete commands
        if offset == 0 {
            // Start new command
            item.data = contents
        } else {
            // Append to existing command
            if item.data == nil {
                item.data = Data(count: Int(offset))
            }
            if let existingData = item.data, Int(offset) > existingData.count {
                item.data = existingData + Data(count: Int(offset) - existingData.count)
            }
            if var data = item.data {
                data.replaceSubrange(Int(offset)..<Int(offset) + contents.count, with: contents)
                item.data = data
            }
        }

        // Check if we have a complete JSON command (look for newline or EOF)
        if let data = item.data,
           let jsonString = String(data: data, encoding: .utf8),
           jsonString.contains("\n") || jsonString.contains("\0") {

            // Process the command
            try await processControlCommand(itemName, jsonString: jsonString.trimmingCharacters(in: .whitespacesAndNewlines))

            // Clear the command buffer after processing
            item.data = Data()
        }

        return contents.count
    }

    private func processControlCommand(_ commandType: String, jsonString: String) async throws {
        logger.debug("Processing control command: \(commandType) - \(jsonString)")

        // TODO: Parse JSON command and execute via bridge (M16)
        // For now, just log the command
        switch commandType {
        case "snapshot":
            logger.info("Snapshot command received: \(jsonString)")
            // TODO: Parse snapshot request and call bridge
        case "branch":
            logger.info("Branch command received: \(jsonString)")
            // TODO: Parse branch request and call bridge
        case "bind":
            logger.info("Bind command received: \(jsonString)")
            // TODO: Parse bind request and call bridge
        default:
            logger.warning("Unknown control command: \(commandType)")
        }
    }
}
