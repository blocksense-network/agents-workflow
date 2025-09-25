
import Foundation
import FSKit
import os

@_silgen_name("agentfs_bridge_statfs")
func agentfs_bridge_statfs(_ core: UnsafeMutableRawPointer?, _ buffer: UnsafeMutablePointer<CChar>?, _ buffer_size: size_t) -> Int32

@_silgen_name("agentfs_bridge_getattr")
func agentfs_bridge_getattr(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ buffer: UnsafeMutablePointer<CChar>?, _ buffer_size: size_t) -> Int32

@_silgen_name("af_stats")
func af_stats(_ fs: UInt64, _ out_stats: UnsafeMutablePointer<UInt8>?, _ stats_size: size_t) -> Int32

@_silgen_name("agentfs_bridge_mkdir")
func agentfs_bridge_mkdir(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ mode: UInt32) -> Int32

@_silgen_name("agentfs_bridge_readdir")
func agentfs_bridge_readdir(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ buffer: UnsafeMutablePointer<CChar>?, _ buffer_size: size_t, _ out_len: UnsafeMutablePointer<size_t>?) -> Int32

@_silgen_name("agentfs_bridge_open")
func agentfs_bridge_open(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ options: UnsafePointer<CChar>?, _ handle: UnsafeMutablePointer<UInt64>?) -> Int32

@_silgen_name("agentfs_bridge_read")
func agentfs_bridge_read(_ core: UnsafeMutableRawPointer?, _ handle: UInt64, _ offset: UInt64, _ buffer: UnsafeMutableRawPointer?, _ length: UInt32, _ bytes_read: UnsafeMutablePointer<UInt32>?) -> Int32

@_silgen_name("agentfs_bridge_write")
func agentfs_bridge_write(_ core: UnsafeMutableRawPointer?, _ handle: UInt64, _ offset: UInt64, _ buffer: UnsafeRawPointer?, _ length: UInt32, _ bytes_written: UnsafeMutablePointer<UInt32>?) -> Int32

@_silgen_name("agentfs_bridge_close")
func agentfs_bridge_close(_ core: UnsafeMutableRawPointer?, _ handle: UInt64) -> Int32

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

@_silgen_name("agentfs_bridge_set_times")
func agentfs_bridge_set_times(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ atime: Int64, _ mtime: Int64, _ ctime: Int64, _ birthtime: Int64) -> Int32

@_silgen_name("agentfs_bridge_set_mode")
func agentfs_bridge_set_mode(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ mode: UInt32) -> Int32

@_silgen_name("agentfs_bridge_xattr_get")
func agentfs_bridge_xattr_get(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ name: UnsafePointer<CChar>?, _ buffer: UnsafeMutableRawPointer?, _ buffer_size: size_t, _ out_len: UnsafeMutablePointer<size_t>?) -> Int32

@_silgen_name("agentfs_bridge_xattr_set")
func agentfs_bridge_xattr_set(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ name: UnsafePointer<CChar>?, _ value: UnsafeRawPointer?, _ value_len: size_t) -> Int32

@_silgen_name("agentfs_bridge_xattr_list")
func agentfs_bridge_xattr_list(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ buffer: UnsafeMutableRawPointer?, _ buffer_size: size_t, _ out_len: UnsafeMutablePointer<size_t>?) -> Int32

@_silgen_name("agentfs_bridge_resolve_id")
func agentfs_bridge_resolve_id(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ node_id: UnsafeMutablePointer<UInt64>?, _ parent_id: UnsafeMutablePointer<UInt64>?) -> Int32

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
    private let coreQueue = DispatchQueue(label: "com.agentfs.AgentFSKitExtension.core")

    /// Generate persistent item IDs for directory entries
    private static let itemIdCounter: OSAllocatedUnfairLock<UInt64> = {
        let lock = OSAllocatedUnfairLock<UInt64>(initialState: FSItem.Identifier.rootDirectory.rawValue + 1000) // Start high to avoid conflicts
        return lock
    }()

    private static func generatePersistentItemID() -> UInt64 {
        return itemIdCounter.withLock { value in
            let result = value
            value += 1
            return result
        }
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

        return capabilities
    }

    var volumeStatistics: FSStatFSResult {
        logger.debug("volumeStatistics")

        let result = FSStatFSResult(fileSystemTypeName: "AgentFS")

        // Get actual statistics from AgentFS core
        guard let fsId = coreHandle?.load(as: UInt64.self) else {
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
            logger.warning("Failed to get AgentFS stats, using conservative defaults: error \(statsResult)")
            // Conservative defaults: unknown sizes => report minimal non-zero units
            result.blockSize = 4096
            result.ioSize = 4096
            result.totalBlocks = 0
            result.availableBlocks = 0
            result.freeBlocks = 0
            result.totalFiles = 0
            result.freeFiles = 0
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

    private func fetchAttributesFor(_ agentItem: AgentFsItem) throws -> FSItem.Attributes {
        var buffer = [CChar](repeating: 0, count: 64)
        let ok = coreQueue.sync { () -> Bool in
            return agentItem.path.withCString { agentfs_bridge_getattr(coreHandle, $0, &buffer, buffer.count) } == 0
        }
        guard ok else { throw fs_errorForPOSIXError(POSIXError.EIO.rawValue) }

        let size = buffer.withUnsafeBytes { $0.load(fromByteOffset: 0, as: UInt64.self) }
        let fileTypeByte = buffer.withUnsafeBytes { $0.load(fromByteOffset: 8, as: UInt8.self) }
        let mode = buffer.withUnsafeBytes { $0.load(fromByteOffset: 9, as: UInt32.self) }
        let atime = buffer.withUnsafeBytes { $0.load(fromByteOffset: 13, as: Int64.self) }
        let mtime = buffer.withUnsafeBytes { $0.load(fromByteOffset: 21, as: Int64.self) }
        let ctime = buffer.withUnsafeBytes { $0.load(fromByteOffset: 29, as: Int64.self) }
        let birthtime = buffer.withUnsafeBytes { $0.load(fromByteOffset: 37, as: Int64.self) }

        let attrs = FSItem.Attributes()
        switch fileTypeByte {
        case 0: attrs.type = .file
        case 1: attrs.type = .directory
        case 2: attrs.type = .symlink
        default: attrs.type = .file
        }
        attrs.size = size
        attrs.allocSize = size
        attrs.mode = mode
        attrs.parentID = agentItem.attributes.parentID
        attrs.accessTime = timespec(tv_sec: Int(atime), tv_nsec: 0)
        attrs.modifyTime = timespec(tv_sec: Int(mtime), tv_nsec: 0)
        attrs.changeTime = timespec(tv_sec: Int(ctime), tv_nsec: 0)
        attrs.birthTime = timespec(tv_sec: Int(birthtime), tv_nsec: 0)
        return attrs
    }

    func attributes(
        _ desiredAttributes: FSItem.GetAttributesRequest,
        of item: FSItem
    ) async throws -> FSItem.Attributes {
        guard let agentItem = item as? AgentFsItem else {
            throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
        }

        // Root: return cached
        if agentItem.attributes.fileID == FSItem.Identifier.rootDirectory {
            return agentItem.attributes
        }

        return try fetchAttributesFor(agentItem)
    }

    func setAttributes(
        _ newAttributes: FSItem.SetAttributesRequest,
        on item: FSItem
    ) async throws -> FSItem.Attributes {
        guard let agentItem = item as? AgentFsItem else { throw fs_errorForPOSIXError(POSIXError.EIO.rawValue) }

        // Support mode and times; others unsupported for now
        if newAttributes.isValid(.mode) {
            let mode = newAttributes.mode
            let path = agentItem.path
        let rc = coreQueue.sync { return path.withCString { agentfs_bridge_set_mode(coreHandle, $0, mode) } }
            if rc != 0, let err = afResultToFSKitError(rc) { throw err }
        }

        let atime = Int64(newAttributes.accessTime.tv_sec)
        let mtime = Int64(newAttributes.modifyTime.tv_sec)
        let ctime = Int64(newAttributes.changeTime.tv_sec)
        let birthtime = Int64(newAttributes.birthTime.tv_sec)

        var needTimes = false
        if newAttributes.isValid(.accessTime) { needTimes = true }
        if newAttributes.isValid(.modifyTime) { needTimes = true }
        if newAttributes.isValid(.changeTime) { needTimes = true }
        if newAttributes.isValid(.birthTime) { needTimes = true }
        if needTimes {
            let rc = coreQueue.sync { return agentItem.path.withCString { agentfs_bridge_set_times(coreHandle, $0, atime, mtime, ctime, birthtime) } }
            if rc != 0, let err = afResultToFSKitError(rc) { throw err }
        }

        // Return fresh attributes
        return try fetchAttributesFor(agentItem)
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

        // Resolve stable IDs for the item and its parent
        var nodeId: UInt64 = 0
        var parentId: UInt64 = 0
        _ = fullPath.withCString { p in agentfs_bridge_resolve_id(coreHandle, p, &nodeId, &parentId) }

        // Call Rust core to get item attributes (48-byte struct)
        var buffer = [CChar](repeating: 0, count: 64)
        let result = coreQueue.sync { () -> Int32 in
            return fullPath.withCString { path_cstr in
                agentfs_bridge_getattr(coreHandle, path_cstr, &buffer, buffer.count)
            }
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
        let item = AgentFsItem(name: name, id: nodeId)
        item.path = fullPath
        item.attributes.fileID = FSItem.Identifier(rawValue: nodeId) ?? .invalid
        if parentId != 0, let pid = FSItem.Identifier(rawValue: parentId) {
            item.attributes.parentID = pid
        }

        // Parse attributes from buffer: size(8) + type(1) + mode(4) + times(4x i64)
        if buffer.count >= 48 {
            let size = buffer.withUnsafeBytes { ptr in
                ptr.load(fromByteOffset: 0, as: UInt64.self)
            }
            let fileTypeByte = buffer.withUnsafeBytes { $0.load(fromByteOffset: 8, as: UInt8.self) }
            let mode = buffer.withUnsafeBytes { $0.load(fromByteOffset: 9, as: UInt32.self) }
            let atime = buffer.withUnsafeBytes { $0.load(fromByteOffset: 13, as: Int64.self) }
            let mtime = buffer.withUnsafeBytes { $0.load(fromByteOffset: 21, as: Int64.self) }
            let ctime = buffer.withUnsafeBytes { $0.load(fromByteOffset: 29, as: Int64.self) }
            let birthtime = buffer.withUnsafeBytes { $0.load(fromByteOffset: 37, as: Int64.self) }

            item.attributes.size = size
            item.attributes.allocSize = size
            item.attributes.mode = mode
            item.attributes.accessTime = timespec(tv_sec: Int(atime), tv_nsec: 0)
            item.attributes.modifyTime = timespec(tv_sec: Int(mtime), tv_nsec: 0)
            item.attributes.changeTime = timespec(tv_sec: Int(ctime), tv_nsec: 0)
            item.attributes.birthTime = timespec(tv_sec: Int(birthtime), tv_nsec: 0)

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
        if let handle = agentItem.userData as? UInt64 {
            logger.debug("reclaimItem: closing open handle \(handle)")
            let result = agentfs_bridge_close(coreHandle, handle)
            if result != 0 {
                logger.warning("reclaimItem: failed to close handle \(handle), error: \(result)")
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

        // Use stored path for correct symlink location
        let linkPath = agentItem.path

        var buffer = [CChar](repeating: 0, count: 4096)
        let result = coreQueue.sync { () -> Int32 in
            return linkPath.withCString { path_cstr in
                agentfs_bridge_readlink(coreHandle, path_cstr, &buffer, buffer.count)
            }
        }

        if result != 0 {
            if let error = afResultToFSKitError(result) {
                throw error
            } else {
                throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
            }
        }

        // Safely decode C buffer as UTF-8 up to first NUL
        let targetPath: String = {
            let bytes = Data(bytes: buffer, count: strnlen(buffer, buffer.count))
            return String(decoding: bytes, as: UTF8.self)
        }()
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

        let item = AgentFsItem(name: name)
        item.path = constructPath(for: name, in: directory)
        mergeAttributes(item.attributes, request: newAttributes)
        item.attributes.parentID = directory.attributes.fileID
        item.attributes.type = type

        // If creating a directory, call the Rust backend
        if type == .directory {
            let dirPath = constructPath(for: name, in: directory)
            let result = dirPath.withCString { path_cstr in
                agentfs_bridge_mkdir(coreHandle, path_cstr, UInt32(newAttributes.mode))
            }
            if result != 0 {
                if let error = afResultToFSKitError(result) {
                    throw error
                } else {
                    throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
                }
            }
        }
        // For files, they are created via open() with create flag, so no Rust call needed here
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

        let result = linkPath.withCString { link_cstr in
            targetPath.withCString { target_cstr in
                agentfs_bridge_symlink(coreHandle, target_cstr, link_cstr)
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

        let result: Int32
        if agentItem.attributes.type == .directory {
            result = itemPath.withCString { path_cstr in
                agentfs_bridge_rmdir(coreHandle, path_cstr)
            }
        } else {
            result = itemPath.withCString { path_cstr in
                agentfs_bridge_unlink(coreHandle, path_cstr)
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

        let result = coreQueue.sync { () -> Int32 in
            return sourcePath.withCString { src_cstr in
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

        // Update item name and path to keep subsequent path-based ops correct
        agentItem.name = destinationName
        agentItem.path = destPath

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

        // Construct directory path from stored path
        let dirPath = directory.path

        // Call Rust core to read directory
        var buffer = [CChar](repeating: 0, count: 16384) // Larger buffer for directory listing
        var outLen: size_t = 0
        let result = coreQueue.sync { () -> Int32 in
            return dirPath.withCString { path_cstr in
                agentfs_bridge_readdir(coreHandle, path_cstr, &buffer, buffer.count, &outLen)
            }
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

        while offset < outLen {
            // Find null terminator
            var endOffset = offset
            while endOffset < outLen && buffer[endOffset] != 0 {
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
            if offset >= outLen || buffer[offset] == 0 {
                break
            }
        }

        logger.debug("enumerateDirectory: found \(entries.count) entries in \(dirPath)")

        // Handle cookie-based enumeration
        let currentCookie = cookie.rawValue
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
                var statBuffer = [CChar](repeating: 0, count: 64)
                let statResult = coreQueue.sync { () -> Int32 in
                    return entryPath.withCString { path_cstr in
                        agentfs_bridge_getattr(coreHandle, path_cstr, &statBuffer, statBuffer.count)
                    }
                }

                if statResult == 0 && statBuffer.count >= 48 {
                    let fileTypeByte = statBuffer.withUnsafeBytes { $0.load(fromByteOffset: 8, as: UInt8.self) }
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
                    // Fill from getattr for accuracy
                    var abuf = [CChar](repeating: 0, count: 64)
                    let ok = coreQueue.sync { () -> Bool in
                        return entryPath.withCString { agentfs_bridge_getattr(coreHandle, $0, &abuf, abuf.count) } == 0
                    }
                    if ok {
                        let size = abuf.withUnsafeBytes { $0.load(fromByteOffset: 0, as: UInt64.self) }
                        let mode = abuf.withUnsafeBytes { $0.load(fromByteOffset: 9, as: UInt32.self) }
                        let atime = abuf.withUnsafeBytes { $0.load(fromByteOffset: 13, as: Int64.self) }
                        let mtime = abuf.withUnsafeBytes { $0.load(fromByteOffset: 21, as: Int64.self) }
                        let ctime = abuf.withUnsafeBytes { $0.load(fromByteOffset: 29, as: Int64.self) }
                        let birthtime = abuf.withUnsafeBytes { $0.load(fromByteOffset: 37, as: Int64.self) }
                        let attrs = FSItem.Attributes()
                        attrs.type = entryType
                        attrs.size = size
                        attrs.allocSize = size
                        attrs.mode = mode
                        attrs.parentID = directory.attributes.fileID
                        attrs.accessTime = timespec(tv_sec: Int(atime), tv_nsec: 0)
                        attrs.modifyTime = timespec(tv_sec: Int(mtime), tv_nsec: 0)
                        attrs.changeTime = timespec(tv_sec: Int(ctime), tv_nsec: 0)
                        attrs.birthTime = timespec(tv_sec: Int(birthtime), tv_nsec: 0)
                        entryAttributes = attrs
                    }
                }

                // Resolve stable IDs for this entry
                var nodeId: UInt64 = 0
                var parentId: UInt64 = 0
                _ = entryPath.withCString { p in agentfs_bridge_resolve_id(coreHandle, p, &nodeId, &parentId) }

                let packResult = packer.packEntry(
                    name: FSFileName(string: entryName),
                    itemType: entryType,
                    itemID: FSItem.Identifier(rawValue: nodeId) ?? .invalid,
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
        // Compute a simple verifier from directory path and entry count to detect changes between calls
        var hasher = Hasher()
        hasher.combine(dirPath)
        hasher.combine(entries.count)
        let v = hasher.finalize()
        return FSDirectoryVerifier(UInt64(bitPattern: Int64(v)))
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
            let ts = timespec(tv_sec: Int(request.accessTime.tv_sec), tv_nsec: Int(request.accessTime.tv_nsec))
            existing.accessTime = ts
        }

        if request.isValid(FSItem.Attribute.changeTime) {
            let ts = timespec(tv_sec: Int(request.changeTime.tv_sec), tv_nsec: Int(request.changeTime.tv_nsec))
            existing.changeTime = ts
        }

        if request.isValid(FSItem.Attribute.modifyTime) {
            let ts = timespec(tv_sec: Int(request.modifyTime.tv_sec), tv_nsec: Int(request.modifyTime.tv_nsec))
            existing.modifyTime = ts
        }

        if request.isValid(FSItem.Attribute.addedTime) {
            let ts = timespec(tv_sec: Int(request.addedTime.tv_sec), tv_nsec: Int(request.addedTime.tv_nsec))
            existing.addedTime = ts
        }

        if request.isValid(FSItem.Attribute.birthTime) {
            let ts = timespec(tv_sec: Int(request.birthTime.tv_sec), tv_nsec: Int(request.birthTime.tv_nsec))
            existing.birthTime = ts
        }

        if request.isValid(FSItem.Attribute.backupTime) {
            let ts = timespec(tv_sec: Int(request.backupTime.tv_sec), tv_nsec: Int(request.backupTime.tv_nsec))
            existing.backupTime = ts
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

        logger.debug("open: \(String(describing: agentItem.name.string ?? "unknown")), modes: \(String(describing: modes)))")

        // Only open handles for regular files
        guard agentItem.attributes.type == .file else {
            return
        }

        // If already has a handle, don't open again
        if agentItem.userData != nil {
            logger.debug("open: item already has handle")
            return
        }

        // Map FSVolume.OpenModes to options JSON for FFI
        let itemPath = agentItem.path
        var handle: UInt64 = 0
        let wantsRead = modes.contains(.read)
        let wantsWrite = modes.contains(.write)
        // FSKit exposes create/truncate intent via GetAttributesRequest during createItem;
        // OpenModes typically covers read/write only. Synthesize conservative defaults here.
        let wantsCreate = false
        let wantsTruncate = false
        let optionsJson = "{\"read\":\(wantsRead),\"write\":\(wantsWrite),\"create\":\(wantsCreate),\"truncate\":\(wantsTruncate)}"

        let result = coreQueue.sync { () -> Int32 in
            return optionsJson.withCString { options_cstr in
                itemPath.withCString { path_cstr in
                    agentfs_bridge_open(coreHandle, path_cstr, options_cstr, &handle)
                }
            }
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

        logger.debug("close: \(String(describing: agentItem.name.string ?? "unknown")), modes: \(String(describing: modes)))")

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
        let result = coreQueue.sync { agentfs_bridge_close(coreHandle, handle) }

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
        var readData = Data(count: length)
        var bytesRead: UInt32 = 0

        let result = coreQueue.sync { () -> Int32 in
            return readData.withUnsafeMutableBytes { bufferPtr in
                agentfs_bridge_read(coreHandle, handle, UInt64(offset), bufferPtr.baseAddress, UInt32(length), &bytesRead)
            }
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
            _ = dataToCopy.withUnsafeBytes { srcPtr in
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

        var bytesWritten: UInt32 = 0
        let result = coreQueue.sync { () -> Int32 in
            return data.withUnsafeBytes { bufferPtr in
                agentfs_bridge_write(coreHandle, handle, UInt64(offset), bufferPtr.baseAddress, UInt32(data.count), &bytesWritten)
            }
        }

        if result != 0 {
            if let error = afResultToFSKitError(result) {
                throw error
            } else {
                throw fs_errorForPOSIXError(POSIXError.EIO.rawValue)
            }
        }

        let written = Int(bytesWritten)
        // Refresh attributes after write so FSKit sees updated size/times promptly
        do {
            let _ = try fetchAttributesFor(agentItem)
        } catch {
            // ignore best-effort refresh errors
        }
        return written
    }
}

@available(macOS 15.4, *)
extension AgentFsVolume: FSVolume.XattrOperations {

    func xattr(named name: FSFileName, of item: FSItem) async throws -> Data {
        logger.debug("xattr: \(item) - \(name.string ?? "NA")")

        guard let agentItem = item as? AgentFsItem, let key = name.string else {
            throw fs_errorForPOSIXError(POSIXError.EINVAL.rawValue)
        }
        var buffer = [UInt8](repeating: 0, count: 4096)
        var outLen: size_t = 0
        let rc = coreQueue.sync { () -> Int32 in
            return agentItem.path.withCString { p in
                return key.withCString { n in
                    return buffer.withUnsafeMutableBytes { bufPtr in
                        agentfs_bridge_xattr_get(coreHandle, p, n, bufPtr.baseAddress, bufPtr.count, &outLen)
                    }
                }
            }
        }
        if rc != 0, let err = afResultToFSKitError(rc) { throw err }
        return Data(buffer.prefix(Int(outLen)))
    }

    func setXattr(named name: FSFileName, to value: Data?, on item: FSItem, policy: FSVolume.SetXattrPolicy) async throws {
        logger.debug("setXattrOf: \(item)")
        guard let agentItem = item as? AgentFsItem, let key = name.string else {
            throw fs_errorForPOSIXError(POSIXError.EINVAL.rawValue)
        }
        let rc: Int32 = coreQueue.sync { () -> Int32 in
            return agentItem.path.withCString { p in
                return key.withCString { n in
                    if let value = value {
                        return value.withUnsafeBytes { bufPtr in
                            agentfs_bridge_xattr_set(coreHandle, p, n, bufPtr.baseAddress, bufPtr.count)
                        }
                    } else {
                        return agentfs_bridge_xattr_set(coreHandle, p, n, nil, 0)
                    }
                }
            }
        }
        if rc != 0, let err = afResultToFSKitError(rc) { throw err }
    }

    func xattrs(of item: FSItem) async throws -> [FSFileName] {
        logger.debug("listXattrs: \(item)")
        guard let agentItem = item as? AgentFsItem else { throw fs_errorForPOSIXError(POSIXError.EINVAL.rawValue) }
        var buffer = [UInt8](repeating: 0, count: 4096)
        var outLen: size_t = 0
        let rc = coreQueue.sync { () -> Int32 in
            return agentItem.path.withCString { p in
                return buffer.withUnsafeMutableBytes { bufPtr in
                    agentfs_bridge_xattr_list(coreHandle, p, bufPtr.baseAddress, bufPtr.count, &outLen)
                }
            }
        }
        if rc != 0, let err = afResultToFSKitError(rc) { throw err }
        // Parse NUL-delimited names
        var names: [FSFileName] = []
        var start = 0
        let total = Int(outLen)
        while start < total {
            var end = start
            while end < total && buffer[end] != 0 { end += 1 }
            if end > start {
                let s = String(bytes: buffer[start..<end], encoding: .utf8) ?? ""
                names.append(FSFileName(string: s))
            }
            start = end + 1
        }
        return names
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
