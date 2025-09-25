
@preconcurrency import Foundation
@preconcurrency import FSKit
import os

@_silgen_name("agentfs_bridge_statfs")
func agentfs_bridge_statfs(_ core: UnsafeMutableRawPointer?, _ buffer: UnsafeMutablePointer<CChar>?, _ buffer_size: size_t) -> Int32

@_silgen_name("agentfs_bridge_stat")
func agentfs_bridge_stat(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ buffer: UnsafeMutablePointer<CChar>?, _ buffer_size: size_t) -> Int32

@_silgen_name("af_stats")
func af_stats(_ fs: UInt64, _ out_stats: UnsafeMutablePointer<UInt8>?, _ stats_size: size_t) -> Int32

@_silgen_name("agentfs_bridge_mkdir")
func agentfs_bridge_mkdir(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ mode: UInt32) -> Int32

@_silgen_name("agentfs_bridge_readdir")
func agentfs_bridge_readdir(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ buffer: UnsafeMutablePointer<CChar>?, _ buffer_size: size_t) -> Int32

@_silgen_name("agentfs_bridge_open")
func agentfs_bridge_open(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ options: UnsafePointer<CChar>?, _ handle: UnsafeMutablePointer<UInt64>?) -> Int32

@_silgen_name("agentfs_bridge_read")
func agentfs_bridge_read(_ core: UnsafeMutableRawPointer?, _ handle: UInt64, _ offset: UInt64, _ buffer: UnsafeMutableRawPointer?, _ length: UInt32, _ bytes_read: UnsafeMutablePointer<UInt32>?) -> Int32

@_silgen_name("agentfs_bridge_write")
func agentfs_bridge_write(_ core: UnsafeMutableRawPointer?, _ handle: UInt64, _ offset: UInt64, _ buffer: UnsafeRawPointer?, _ length: UInt32, _ bytes_written: UnsafeMutablePointer<UInt32>?) -> Int32

@_silgen_name("agentfs_bridge_close")
func agentfs_bridge_close(_ core: UnsafeMutableRawPointer?, _ handle: UInt64) -> Int32

@_silgen_name("agentfs_bridge_getattr")
func agentfs_bridge_getattr(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ buffer: UnsafeMutablePointer<CChar>?, _ buffer_size: size_t) -> Int32


@_silgen_name("agentfs_bridge_symlink")
func agentfs_bridge_symlink(_ core: UnsafeMutableRawPointer?, _ target: UnsafePointer<CChar>?, _ linkpath: UnsafePointer<CChar>?) -> Int32

@_silgen_name("agentfs_bridge_readlink")
func agentfs_bridge_readlink(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ buffer: UnsafeMutablePointer<CChar>?, _ buffer_size: size_t) -> Int32

@_silgen_name("agentfs_bridge_rename")
func agentfs_bridge_rename(_ core: UnsafeMutableRawPointer?, _ oldpath: UnsafePointer<CChar>?, _ newpath: UnsafePointer<CChar>?) -> Int32

@_silgen_name("agentfs_bridge_rmdir")
func agentfs_bridge_rmdir(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?) -> Int32

@_silgen_name("agentfs_bridge_unlink")
func agentfs_bridge_unlink(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?) -> Int32

@available(macOS 15.4, *)
final class AgentFsVolume: FSVolume {

    /// Convert AfResult error codes to FSKit errors
    /// Returns nil for success (AfOk = 0), or an Error for actual errors
    private func afResultToFSKitError(_ result: Int32) -> Error? {
        switch result {
        case 0: // AfOk - success, no error
            return nil
        case 2: // AfErrNotFound -> ENOENT
            return fs_errorForPOSIXError(POSIXError.ENOENT.rawValue)
        case 17: // AfErrExists -> EEXIST
            return fs_errorForPOSIXError(POSIXError.EEXIST.rawValue)
        case 13: // AfErrAcces -> EACCES
            return fs_errorForPOSIXError(POSIXError.EACCES.rawValue)
        case 28: // AfErrNospc -> ENOSPC
            return fs_errorForPOSIXError(POSIXError.ENOSPC.rawValue)
        case 22: // AfErrInval -> EINVAL
            return fs_errorForPOSIXError(POSIXError.EINVAL.rawValue)
        case 16: // AfErrBusy -> EBUSY
            return fs_errorForPOSIXError(POSIXError.EBUSY.rawValue)
        case 95: // AfErrUnsupported -> ENOTSUP
            return fs_errorForPOSIXError(POSIXError.ENOTSUP.rawValue)
        default:
            return fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }
    }

    private let resource: FSResource
    private let coreHandle: UnsafeMutableRawPointer?
    private let coreHandleLock = OSAllocatedUnfairLock<Void>()

    /// Generate unique item IDs using the shared generator
    private static func generateItemID() -> UInt64 {
        return AgentFsItem.generateUniqueItemID()
    }

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

@available(macOS 15.4, *)
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

@available(macOS 15.4, *)
extension AgentFsVolume: FSVolume.Operations {

    var supportedVolumeCapabilities: FSVolume.SupportedCapabilities {
        logger.debug("supportedVolumeCapabilities")

        let capabilities = FSVolume.SupportedCapabilities()

        // Hard links are not implemented in the current AgentFS core
        // TODO: Implement hard link support in AgentFS core when needed
        capabilities.supportsHardLinks = false

        // Symbolic links are fully supported by AgentFS core
        capabilities.supportsSymbolicLinks = true

        // AgentFS is currently an in-memory filesystem, so object IDs are not
        // persistent across filesystem restarts/mounts. This is appropriate for
        // the current use case of temporary, session-based filesystem views.
        capabilities.supportsPersistentObjectIDs = false

        // AgentFS implements volume statistics reporting (total/free blocks, files, etc.)
        // so doesNotSupportVolumeSizes must be false (meaning it DOES support volume sizes)
        capabilities.doesNotSupportVolumeSizes = false

        // AgentFS supports hidden files (files/directories starting with '.')
        capabilities.supportsHiddenFiles = true

        // AgentFS uses 64-bit object IDs for filesystem items, providing
        // sufficient namespace for all practical use cases
        capabilities.supports64BitObjectIDs = true

        // caseFormat defaults to case sensitive (.caseSensitive) for Unix-style filesystems,
        // which is appropriate for AgentFS as a POSIX-like filesystem

        return capabilities
    }

    var volumeStatistics: FSStatFSResult {
        logger.debug("volumeStatistics")

        let result = FSStatFSResult(fileSystemTypeName: "AgentFS")

        // Get actual statistics from AgentFS core
        let fsId = coreHandleLock.withLock { () -> UInt64? in
            coreHandle?.load(as: UInt64.self)
        }

        guard let fsId = fsId else {
            logger.warning("volumeStatistics: no core handle available, using defaults")
            // Fallback to reasonable defaults
            result.blockSize = 4096
            result.ioSize = 4096
            result.totalBlocks = 1000000  // 4GB with 4K blocks
            result.availableBlocks = result.totalBlocks
            result.freeBlocks = result.totalBlocks
            result.totalFiles = 100000
            result.freeFiles = 100000
            return result
        }

        var statsBuffer = [UInt8](repeating: 0, count: 28) // 28 bytes for FsStats
        let statsResult = af_stats(fsId, &statsBuffer, statsBuffer.count)

        if statsResult == 0 {
            // Parse FsStats from buffer: branches(u32) + snapshots(u32) + open_handles(u32) + bytes_in_memory(u64) + bytes_spilled(u64)
            var branches: UInt32 = 0
            var snapshots: UInt32 = 0
            var openHandles: UInt32 = 0
            var bytesInMemory: UInt64 = 0
            var bytesSpilled: UInt64 = 0

            statsBuffer.withUnsafeBytes { bufferPtr in
                branches = bufferPtr.load(fromByteOffset: 0, as: UInt32.self)
                snapshots = bufferPtr.load(fromByteOffset: 4, as: UInt32.self)
                openHandles = bufferPtr.load(fromByteOffset: 8, as: UInt32.self)
                bytesInMemory = bufferPtr.load(fromByteOffset: 12, as: UInt64.self)
                bytesSpilled = bufferPtr.load(fromByteOffset: 20, as: UInt64.self)

                logger.debug("AgentFS stats: branches=\(branches), snapshots=\(snapshots), open_handles=\(openHandles), memory=\(bytesInMemory), spilled=\(bytesSpilled)")
            }

            // Convert AgentFS statistics to FSKit format
            result.blockSize = 4096
            result.ioSize = 4096

            // Estimate total space based on memory usage and configuration
            // For AgentFS, we consider total space as memory limit + some spill space
            let memoryLimit: UInt64 = 1024 * 1024 * 1024  // 1GB default, should come from config
            let totalBytes = max(memoryLimit, bytesInMemory + bytesSpilled + 100 * 1024 * 1024) // At least 100MB
            result.totalBlocks = totalBytes / 4096
            result.availableBlocks = (totalBytes - bytesInMemory - bytesSpilled) / 4096
            result.freeBlocks = result.availableBlocks

            // File count based on open handles and estimated capacity
            result.totalFiles = UInt64(max(10000, Int(openHandles) * 10))
            result.freeFiles = result.totalFiles - UInt64(min(Int(result.totalFiles), Int(openHandles)))
        } else {
            logger.warning("Failed to get AgentFS stats, using defaults: error \(statsResult)")
            // Fallback to reasonable defaults
            result.blockSize = 4096
            result.ioSize = 4096
            result.totalBlocks = 1000000  // 4GB with 4K blocks
            result.availableBlocks = result.totalBlocks
            result.freeBlocks = result.totalBlocks
            result.totalFiles = 100000
            result.freeFiles = 100000
        }

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
    }

    func unmount() async {
        logger.debug("unmount")
    }

    func synchronize(flags: FSSyncFlags) async throws {
        logger.debug("synchronize")
    }

    func attributes(
        _ desiredAttributes: FSItem.GetAttributesRequest,
        of item: FSItem
    ) async throws -> FSItem.Attributes {
        guard let agentItem = item as? AgentFsItem else {
            logger.debug("getItemAttributes2: \(item), \(desiredAttributes)")
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        logger.debug("getItemAttributes1: \(agentItem.name), \(desiredAttributes)")

        // For the root item, return cached attributes (root is immutable)
        if agentItem.attributes.fileID == FSItem.Identifier.rootDirectory {
            return agentItem.attributes
        }

        // Get fresh attributes from Rust core for all other items
        let handle = coreHandle
        let path = agentItem.path
        let (result, buffer) = coreHandleLock.withLock { () -> (Int32, [CChar]) in
            var buffer = [CChar](repeating: 0, count: 4096)
            let result = path.withCString { path_cstr in
                agentfs_bridge_stat(handle, path_cstr, &buffer, buffer.count)
            }
            return (result, buffer)
        }

        if result != 0 {
            logger.debug("attributes: failed to stat path \(agentItem.path), error: \(result)")
            if let error = afResultToFSKitError(result) {
                throw error
            } else {
                throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
            }
        }

        // Create a fresh attributes object and populate it from the buffer
        var freshAttributes = FSItem.Attributes()
        freshAttributes.fileID = agentItem.attributes.fileID
        freshAttributes.parentID = agentItem.attributes.parentID

        // Parse attributes from buffer (size: u64, type: u8)
        if buffer.count >= 9 {
            let size = buffer.withUnsafeBytes { ptr in
                ptr.load(fromByteOffset: 0, as: UInt64.self)
            }
            let fileTypeByte = buffer[8]

            freshAttributes.size = size
            freshAttributes.allocSize = size

            // Map file type byte to FSItem.ItemType
            switch fileTypeByte {
            case 0: // regular file
                freshAttributes.type = .file
            case 1: // directory
                freshAttributes.type = .directory
            case 2: // symlink
                freshAttributes.type = .symlink
            default:
                freshAttributes.type = .file // default fallback
            }
        } else {
            // Fallback if buffer is too small
            freshAttributes.type = .file
            freshAttributes.size = 0
            freshAttributes.allocSize = 0
        }

        // Preserve mode and other metadata that might be cached
        freshAttributes.mode = agentItem.attributes.mode
        freshAttributes.uid = agentItem.attributes.uid
        freshAttributes.gid = agentItem.attributes.gid
        freshAttributes.linkCount = agentItem.attributes.linkCount
        freshAttributes.flags = agentItem.attributes.flags

        // Update timestamps if not set (timespec with tv_sec=0 and tv_nsec=0 indicates unset)
        if freshAttributes.addedTime.tv_sec == 0 && freshAttributes.addedTime.tv_nsec == 0 {
            freshAttributes.addedTime = agentItem.attributes.addedTime
        }
        if freshAttributes.birthTime.tv_sec == 0 && freshAttributes.birthTime.tv_nsec == 0 {
            freshAttributes.birthTime = agentItem.attributes.birthTime
        }
        if freshAttributes.changeTime.tv_sec == 0 && freshAttributes.changeTime.tv_nsec == 0 {
            freshAttributes.changeTime = agentItem.attributes.changeTime
        }
        if freshAttributes.modifyTime.tv_sec == 0 && freshAttributes.modifyTime.tv_nsec == 0 {
            freshAttributes.modifyTime = agentItem.attributes.modifyTime
        }
        if freshAttributes.accessTime.tv_sec == 0 && freshAttributes.accessTime.tv_nsec == 0 {
            freshAttributes.accessTime = agentItem.attributes.accessTime
        }

        logger.debug("attributes: fresh attributes for \(agentItem.path) - size: \(freshAttributes.size), type: \(freshAttributes.type.rawValue)")
        return freshAttributes
    }

    func setAttributes(
        _ newAttributes: FSItem.SetAttributesRequest,
        on item: FSItem
    ) async throws -> FSItem.Attributes {
        logger.debug("setItemAttributes: \(item), \(newAttributes)")
        if let item = item as? AgentFsItem {
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
        logger.debug("lookupItem: \(String(describing: name.string)), \(directory)")

        guard let dirItem = directory as? AgentFsItem else {
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        // Construct the full path for the lookup
        let fullPath = constructPath(for: name, in: dirItem)

        // Call Rust core to get item attributes
        let (result, buffer) = coreHandleLock.withLock { () -> (Int32, [CChar]) in
            var buffer = [CChar](repeating: 0, count: 4096)
            let result = fullPath.withCString { path_cstr in
                agentfs_bridge_stat(coreHandle, path_cstr, &buffer, buffer.count)
            }
            return (result, buffer)
        }

        if result != 0 {
            logger.debug("lookupItem: failed to stat path \(fullPath), error: \(result)")
            if let error = afResultToFSKitError(result) {
                throw error
            } else {
                throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
            }
        }

        // Parse the attributes from the buffer
        let item = AgentFsItem(name: name)
        item.path = fullPath

        // Parse attributes from buffer (size: u64, type: u8)
        if buffer.count >= 9 {
            let size = buffer.withUnsafeBytes { ptr in
                ptr.load(fromByteOffset: 0, as: UInt64.self)
            }
            let fileTypeByte = buffer[8]

            item.attributes.size = size
            item.attributes.allocSize = size

            // Map file type byte to FSItem.ItemType
            switch fileTypeByte {
            case 0: // regular file
                item.attributes.type = .file
            case 1: // directory
                item.attributes.type = .directory
            case 2: // symlink
                item.attributes.type = .symlink
            default:
                item.attributes.type = .file // default fallback
            }
        } else {
            // Fallback if buffer is too small
            item.attributes.type = .file
            item.attributes.size = 0
            item.attributes.allocSize = 0
        }

        // Set the parent ID to link it to the directory
        item.attributes.parentID = dirItem.attributes.fileID

        logger.debug("lookupItem: found item \(name.string ?? "unnamed")")
        return (item, name)
    }

    func reclaimItem(_ item: FSItem) async throws {
        logger.debug("reclaimItem: \(item)")

        guard let agentItem = item as? AgentFsItem else {
            logger.warning("reclaimItem: item is not an AgentFsItem")
            return
        }

        // If this item has an open file handle, close it in the Rust core
        if let handleValue = agentItem.userData as? UInt64 {
            logger.debug("reclaimItem: closing open handle \(handleValue)")
            let result = coreHandleLock.withLock { () -> Int32 in
                agentfs_bridge_close(coreHandle, handleValue)
            }
            if result != 0 {
                logger.warning("reclaimItem: failed to close handle \(handleValue), error: \(result)")
            }
        }

        // Clear any cached data and references to help with memory management
        agentItem.data = nil
        agentItem.userData = nil

        // Note: We don't remove from children here as that's handled by the volume
        // This method is called when FsKit determines the item is no longer needed
        // and can be reclaimed for memory management purposes

        logger.debug("reclaimItem: reclaimed item \(agentItem.name.string ?? "unnamed")")
    }

    func readSymbolicLink(
        _ item: FSItem
    ) async throws -> FSFileName {
        logger.debug("readSymbolicLink: \(item)")

        guard let agentItem = item as? AgentFsItem else {
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        // Find the parent directory to construct the path
        // For now, assume we can reconstruct the path
        // TODO: Store path information in FSItem or implement proper path tracking
        let linkPath = "/" + (agentItem.name.string ?? "")  // Simplified

        let (result, buffer) = coreHandleLock.withLock { () -> (Int32, [CChar]) in
            var buffer = [CChar](repeating: 0, count: 4096)
            let result = linkPath.withCString { path_cstr in
                agentfs_bridge_readlink(coreHandle, path_cstr, &buffer, buffer.count)
            }
            return (result, buffer)
        }

        if result != 0 {
            if let error = afResultToFSKitError(result) {
                throw error
            } else {
                throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
            }
        }

        let targetPath = String(cString: buffer)
        return FSFileName(string: targetPath)
    }

    func createItem(
        named name: FSFileName,
        type: FSItem.ItemType,
        inDirectory directory: FSItem,
        attributes newAttributes: FSItem.SetAttributesRequest
    ) async throws -> (FSItem, FSFileName) {
        logger.debug("createItem: \(String(describing: name.string)) - \(newAttributes.mode)")

        guard let directory = directory as? AgentFsItem else {
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        let item = await AgentFsItem(name: name)
        item.path = constructPath(for: name, in: directory)
        mergeAttributes(item.attributes, request: newAttributes)
        item.attributes.parentID = directory.attributes.fileID
        item.attributes.type = type

        // Create the item in the filesystem based on its type
        if type == .directory {
            // Create directory in the Rust backend
            let dirPath = constructPath(for: name, in: directory)
            let mode = UInt32(newAttributes.mode)
            let result = coreHandleLock.withLock { () -> Int32 in
                dirPath.withCString { path_cstr in
                    agentfs_bridge_mkdir(coreHandle, path_cstr, mode)
                }
            }
            if result != 0 {
                if let error = afResultToFSKitError(result) {
                    throw error
                } else {
                    throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
                }
            }
        } else if type == .file {
            // For files, creation happens during open() with create/truncate flags.
            // This is the standard pattern for most filesystems where createItem
            // sets up metadata and open() actually instantiates the file.
            // The file will be created when opened for writing or when explicitly created.
            logger.debug("createItem: file creation deferred to open() operation")
        } else if type == .symlink {
            // Symlinks are handled by createSymbolicLink, not here
            logger.warning("createItem called with symlink type - should use createSymbolicLink instead")
        }

        // No need to add to in-memory children since we use path-based operations

        return (item, name)
    }

    func createSymbolicLink(
        named name: FSFileName,
        inDirectory directory: FSItem,
        attributes newAttributes: FSItem.SetAttributesRequest,
        linkContents contents: FSFileName
    ) async throws -> (FSItem, FSFileName) {
        logger.debug("createSymbolicLink: \(name)")

        guard let directory = directory as? AgentFsItem else {
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        // Construct full path for the link
        let linkPath = constructPath(for: name, in: directory)
        let targetPath = contents.string ?? ""

        let result = coreHandleLock.withLock { () -> Int32 in
            linkPath.withCString { link_cstr in
                targetPath.withCString { target_cstr in
                    agentfs_bridge_symlink(coreHandle, target_cstr, link_cstr)
                }
            }
        }

        if result != 0 {
            if let error = afResultToFSKitError(result) {
                throw error
            } else {
                throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
            }
        }

        // Create FSItem for the new symlink
        let item = AgentFsItem(name: name)
        item.path = linkPath
        mergeAttributes(item.attributes, request: newAttributes)
        item.attributes.parentID = directory.attributes.fileID
        item.attributes.type = .symlink
        // No need to add to in-memory children since we use path-based operations

        return (item, name)
    }

    func createLink(
        to item: FSItem,
        named name: FSFileName,
        inDirectory directory: FSItem
    ) async throws -> FSFileName {
        logger.debug("createLink: \(name)")
        // Hard links are not implemented in the current Rust core
        // TODO: Implement hard link support in Rust core
        throw fs_errorForPOSIXError(POSIXError.ENOTSUP.rawValue)
    }

    func removeItem(
        _ item: FSItem,
        named name: FSFileName,
        fromDirectory directory: FSItem
    ) async throws {
        logger.debug("remove: \(name)")

        guard let agentItem = item as? AgentFsItem, let directory = directory as? AgentFsItem else {
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        // Construct full path for the item to remove
        let itemPath = constructPath(for: name, in: directory)

        let itemType = agentItem.attributes.type
        let result: Int32 = coreHandleLock.withLock { () -> Int32 in
            if itemType == .directory {
                itemPath.withCString { path_cstr in
                    agentfs_bridge_rmdir(coreHandle, path_cstr)
                }
            } else {
                itemPath.withCString { path_cstr in
                    agentfs_bridge_unlink(coreHandle, path_cstr)
                }
            }
        }

        if result != 0 {
            if let error = afResultToFSKitError(result) {
                throw error
            } else {
                throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
            }
        }

        // No need to update in-memory state since we use path-based operations
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

        guard let agentItem = item as? AgentFsItem,
              let sourceDir = sourceDirectory as? AgentFsItem,
              let destDir = destinationDirectory as? AgentFsItem else {
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        let sourcePath = constructPath(for: sourceName, in: sourceDir)
        let destPath = constructPath(for: destinationName, in: destDir)

        let result = coreHandleLock.withLock { () -> Int32 in
            sourcePath.withCString { src_cstr in
                destPath.withCString { dst_cstr in
                    agentfs_bridge_rename(coreHandle, src_cstr, dst_cstr)
                }
            }
        }

        if result != 0 {
            if let error = afResultToFSKitError(result) {
                throw error
            } else {
                throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
            }
        }

        // Update item name for consistency (though path-based operations don't rely on this)
        agentItem.name = destinationName

        return destinationName
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
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        // Construct directory path
        let dirPath = directory.name.string == "/" ? "/" : "/\(directory.name.string ?? "")"

        // Call Rust core to read directory
        let (result, buffer) = coreHandleLock.withLock { () -> (Int32, [CChar]) in
            var buffer = [CChar](repeating: 0, count: 8192) // Larger buffer for directory listing
            let result = dirPath.withCString { path_cstr in
                agentfs_bridge_readdir(coreHandle, path_cstr, &buffer, buffer.count)
            }
            return (result, buffer)
        }

        if result != 0 {
            logger.error("enumerateDirectory: failed to read directory \(dirPath)")
            if let error = afResultToFSKitError(result) {
                throw error
            } else {
                throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
            }
        }

        // Parse directory entries from buffer
        // Assume buffer contains null-terminated UTF-8 strings for filenames
        var entries: [String] = []
        var offset = 0

        while offset < buffer.count {
            // Find null terminator
            var endOffset = offset
            while endOffset < buffer.count && buffer[endOffset] != 0 {
                endOffset += 1
            }

            if endOffset > offset {
                // Extract filename from null-terminated C string
                buffer.withUnsafeBufferPointer { bufferPtr in
                    let filenamePtr = bufferPtr.baseAddress!.advanced(by: offset)
                    if let filename = String(cString: filenamePtr, encoding: .utf8) {
                        entries.append(filename)
                    }
                }
            }

            // Move to next entry (skip null terminator)
            offset = endOffset + 1

            // Stop if we hit a double null (end of list) or buffer end
            if offset >= buffer.count || buffer[offset] == 0 {
                break
            }
        }

        logger.debug("enumerateDirectory: found \(entries.count) entries in \(dirPath)")

        // Handle cookie-based enumeration
        var currentCookie = cookie.rawValue
        var nextCookieValue = currentCookie

        // Always include "." and ".." first
        if currentCookie == 0 {
            let _ = packer.packEntry(
                name: FSFileName(string: "."),
                itemType: .directory,
                itemID: directory.attributes.fileID,
                nextCookie: FSDirectoryCookie(1),
                attributes: attributes != nil ? directory.attributes : nil
            )
            nextCookieValue = 1
        }

        if currentCookie <= 1 {
            let parentId = directory.attributes.parentID
            let _ = packer.packEntry(
                name: FSFileName(string: ".."),
                itemType: .directory,
                itemID: parentId,
                nextCookie: FSDirectoryCookie(2),
                attributes: nil // Don't provide attributes for ..
            )
            nextCookieValue = 2
        }


        // Add actual directory entries starting from cookie position
        let startIndex = Int(currentCookie) - 2 // Adjust for . and ..
        if startIndex >= 0 && startIndex < entries.count {
            for i in startIndex..<entries.count {
                let entryName = entries[i]
                nextCookieValue += 1

                // Skip . and .. as they're handled above
                if entryName == "." || entryName == ".." {
                    continue
                }

                // Determine entry type by trying to stat it (simplified)
                var entryType = FSItem.ItemType.file
                let entryPath = constructPath(for: FSFileName(string: entryName), in: directory)
                let (statResult, statBuffer) = coreHandleLock.withLock { () -> (Int32, [CChar]) in
                    var statBuffer = [CChar](repeating: 0, count: 4096)
                    let statResult = entryPath.withCString { path_cstr in
                        agentfs_bridge_stat(coreHandle, path_cstr, &statBuffer, statBuffer.count)
                    }
                    return (statResult, statBuffer)
                }

                if statResult == 0 && statBuffer.count >= 9 {
                    let fileTypeByte = statBuffer[8]
                    switch fileTypeByte {
                    case 0: entryType = .file
                    case 1: entryType = .directory
                    case 2: entryType = .symlink
                    default: entryType = .file
                    }
                }

                // Create a temporary item for this entry to get attributes if requested
                var entryAttributes: FSItem.Attributes? = nil
                if attributes != nil {
                    let tempItem = AgentFsItem(name: FSFileName(string: entryName))
                    tempItem.path = entryPath
                    tempItem.attributes.parentID = directory.attributes.fileID
                    tempItem.attributes.type = entryType
                    entryAttributes = tempItem.attributes
                }

                let packResult = packer.packEntry(
                    name: FSFileName(string: entryName),
                    itemType: entryType,
                    itemID: FSItem.Identifier(rawValue: AgentFsVolume.generateItemID()) ?? .invalid, // Generate unique ID for this item
                    nextCookie: FSDirectoryCookie(nextCookieValue),
                    attributes: entryAttributes
                )

                // Stop if packer indicates no more space
                if !packResult {
                    break
                }
            }
        }

        logger.debug("enumerateDirectory: completed for \(dirPath) with \(entries.count) entries")
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

        if request.isValid(FSItem.Attribute.accessTime) {
            let timespec = timespec()
            request.accessTime = timespec
            existing.accessTime = timespec
        }

        if request.isValid(FSItem.Attribute.changeTime) {
            let timespec = timespec()
            request.changeTime = timespec
            existing.changeTime = timespec
        }

        if request.isValid(FSItem.Attribute.modifyTime) {
            let timespec = timespec()
            request.modifyTime = timespec
            existing.modifyTime = timespec
        }

        if request.isValid(FSItem.Attribute.addedTime) {
            let timespec = timespec()
            request.addedTime = timespec
            existing.addedTime = timespec
        }

        if request.isValid(FSItem.Attribute.birthTime) {
            let timespec = timespec()
            request.birthTime = timespec
            existing.birthTime = timespec
        }

        if request.isValid(FSItem.Attribute.backupTime) {
            let timespec = timespec()
            request.backupTime = timespec
            existing.backupTime = timespec
        }
    }
}

@available(macOS 15.4, *)
extension AgentFsVolume: FSVolume.OpenCloseOperations {

    func openItem(_ item: FSItem, modes: FSVolume.OpenModes) async throws {
        guard let agentItem = item as? AgentFsItem else {
            logger.debug("open: unknown item type")
            return
        }

        logger.debug("open: \(String(describing: agentItem.name.string ?? "unknown")), modes: \(String(describing: modes))")

        // Only open handles for regular files
        guard agentItem.attributes.type == .file else {
            return
        }

        // If already has a handle, don't open again
        if agentItem.userData != nil {
            logger.debug("open: item already has handle")
            return
        }

        // Open file handle using Rust FFI
        let itemPath = agentItem.path
        let optionsJson = "{}" // Default options - could be extended for read/write modes

        let (result, handle) = coreHandleLock.withLock { () -> (Int32, UInt64) in
            var handle: UInt64 = 0
            let result = optionsJson.withCString { options_cstr in
                itemPath.withCString { path_cstr in
                    agentfs_bridge_open(coreHandle, path_cstr, options_cstr, &handle)
                }
            }
            return (result, handle)
        }

        if result != 0 {
            logger.error("open: failed to open handle for \(itemPath), error: \(result)")
            if let error = afResultToFSKitError(result) {
                throw error
            } else {
                throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
            }
        }

        // Store the handle in userData
        agentItem.userData = handle
        logger.debug("open: opened handle \(handle) for \(itemPath)")
    }

    func closeItem(_ item: FSItem, modes: FSVolume.OpenModes) async throws {
        guard let agentItem = item as? AgentFsItem else {
            logger.debug("close: unknown item type")
            return
        }

        logger.debug("close: \(String(describing: agentItem.name.string ?? "unknown")), modes: \(String(describing: modes))")

        // Only close handles for regular files
        guard agentItem.attributes.type == .file else {
            return
        }

        // Get and clear the handle
        guard let handle = agentItem.userData as? UInt64 else {
            logger.debug("close: no handle to close")
            return
        }

        // Close file handle using Rust FFI
        let result = coreHandleLock.withLock { () -> Int32 in
            agentfs_bridge_close(coreHandle, handle)
        }

        if result != 0 {
            logger.warning("close: failed to close handle \(handle), error: \(result)")
            // Don't throw here as the item should still be considered closed
        }

        // Clear the handle
        agentItem.userData = nil
        logger.debug("close: closed handle \(handle)")
    }
}

@available(macOS 15.4, *)
extension AgentFsVolume: FSVolume.ReadWriteOperations {

    // Async/throws version
    func read(from item: FSItem, at offset: off_t, length: Int, into buffer: FSMutableFileDataBuffer) async throws -> Int {
        guard let agentItem = item as? AgentFsItem else {
            logger.debug("Read operation: unknown item type, offset: \(offset), length: \(length)")
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        logger.debug("Read operation: \(agentItem.name), offset: \(offset), length: \(length)")

        // Handle regular file reads
        guard let handle = agentItem.userData as? UInt64 else {
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        // Prepare buffer for reading
        var bytesRead: UInt32 = 0
        let (result, readData) = coreHandleLock.withLock { () -> (Int32, Data) in
            var readData = Data(count: length)
            var bytesRead: UInt32 = 0
            let result = readData.withUnsafeMutableBytes { bufferPtr in
                agentfs_bridge_read(coreHandle, handle, UInt64(offset), bufferPtr.baseAddress, UInt32(length), &bytesRead)
            }
            return (result, readData)
        }

        if result != 0 {
            if let error = afResultToFSKitError(result) {
                throw error
            } else {
                throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
            }
        }

        // Copy data to the FSKit buffer using the correct method
        let actualBytesRead = Int(bytesRead)
        if actualBytesRead > 0 {
            let dataToCopy = readData.prefix(actualBytesRead)
            dataToCopy.withUnsafeBytes { srcPtr in
                buffer.withUnsafeMutableBytes { dstPtr in
                    memcpy(dstPtr.baseAddress, srcPtr.baseAddress, actualBytesRead)
                }
            }
        }

        return actualBytesRead
    }

    // Async/throws version for write
    func write(contents data: Data, to item: FSItem, at offset: off_t) async throws -> Int {
        guard let agentItem = item as? AgentFsItem else {
            logger.debug("Write operation: unknown item type, offset: \(offset), length: \(data.count)")
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        logger.debug("Write operation: \(agentItem.name), offset: \(offset), length: \(data.count)")

        // Handle regular file writes
        guard let handle = agentItem.userData as? UInt64 else {
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        let (result, bytesWritten) = coreHandleLock.withLock { () -> (Int32, UInt32) in
            var bytesWritten: UInt32 = 0
            let result = data.withUnsafeBytes { bufferPtr in
                agentfs_bridge_write(coreHandle, handle, UInt64(offset), bufferPtr.baseAddress, UInt32(data.count), &bytesWritten)
            }
            return (result, bytesWritten)
        }

        if result != 0 {
            if let error = afResultToFSKitError(result) {
                throw error
            } else {
                throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
            }
        }

        return Int(bytesWritten)
    }
}

@available(macOS 15.4, *)
extension AgentFsVolume: FSVolume.XattrOperations {

    func xattr(named name: FSFileName, of item: FSItem) async throws -> Data {
        logger.debug("xattr: \(item) - \(name.string ?? "NA")")

        if let item = item as? AgentFsItem {
            return item.xattrs[name] ?? Data()
        } else {
            return Data()
        }
    }

    func setXattr(named name: FSFileName, to value: Data?, on item: FSItem, policy: FSVolume.SetXattrPolicy) async throws {
        logger.debug("setXattrOf: \(item)")

        if let item = item as? AgentFsItem {
            item.xattrs[name] = value
        }
    }

    func xattrs(of item: FSItem) async throws -> [FSFileName] {
        logger.debug("listXattrs: \(item)")

        if let item = item as? AgentFsItem {
            return Array(item.xattrs.keys)
        } else {
            return []
        }
    }
}

    // MARK: - Helper Methods

    private func constructPath(for name: FSFileName, in directory: AgentFsItem) -> String {
        // Build path relative to volume root using the directory's path
        let nameStr = name.string ?? ""

        // Use the directory's stored path to build the full path
        if directory.path == "/" {
            return "/" + nameStr
        } else {
            return directory.path + "/" + nameStr
        }
    }
