
import Foundation
import FSKit
import os

@_silgen_name("agentfs_bridge_statfs")
func agentfs_bridge_statfs(_ core: UnsafeMutableRawPointer?, _ buffer: UnsafeMutablePointer<CChar>?, _ buffer_size: size_t) -> Int32

@_silgen_name("agentfs_bridge_stat")
func agentfs_bridge_stat(_ core: UnsafeMutableRawPointer?, _ path: UnsafePointer<CChar>?, _ buffer: UnsafeMutablePointer<CChar>?, _ buffer_size: size_t) -> Int32

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

@_silgen_name("af_control_request")
func af_control_request(_ fs: UInt64, _ request_data: UnsafePointer<UInt8>?, _ request_len: usize, _ response_data: UnsafeMutablePointer<UInt8>?, _ response_max_len: usize, _ response_actual_len: UnsafeMutablePointer<usize>?) -> Int32

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

// @available(macOS 15.4, *)
// extension AgentFsVolume: FSVolume.PathConfOperations {

//    var maximumLinkCount: Int {
//        return -1
//    }
//
//    var maximumNameLength: Int {
//        return -1
//    }
//
//    var restrictsOwnershipChanges: Bool {
//        return false
//    }
//
//    var truncatesLongNames: Bool {
//        return false
//    }
//
//    var maximumXattrSize: Int {
//        return Int.max
//    }


@available(macOS 15.4, *)
extension AgentFsVolume: FSVolume.Operations {

    func write(to item: FSItem, offset: UInt64, length: UInt32, data: Data) async throws -> Data {
        logger.debug("Write operation: \(item.name), offset: \(offset), length: \(length)")

        // Check if this is a control file write
        if item.name.rawValue == "snapshot" || item.name.rawValue == "branch" || item.name.rawValue == "bind" {
            // This is a control plane operation - forward raw SSZ bytes to Rust
            try await processControlCommand(item.name.rawValue, data: data)
            return Data() // Control operations don't return data
        }

        // Handle regular file writes
        guard let handle = item.userData as? UInt64 else {
            throw NSError(domain: "AgentFS", code: -1, userInfo: [NSLocalizedDescriptionKey: "Invalid file handle"])
        }

        var bytesWritten: UInt32 = 0
        let result = data.withUnsafeBytes { bufferPtr in
            agentfs_bridge_write(coreHandle, handle, offset, bufferPtr.baseAddress, length, &bytesWritten)
        }

        if result != 0 {
            throw NSError(domain: "AgentFS", code: Int(result), userInfo: [NSLocalizedDescriptionKey: "Write failed"])
        }

        return Data() // No additional data to return
    }

    private func processControlCommand(_ commandType: String, data: Data) async throws {
        logger.debug("Processing control command: \(commandType) with \(data.count) bytes")

        guard let coreHandle = coreHandle else {
            logger.error("No core handle available for control command")
            throw NSError(domain: "AgentFS", code: -1, userInfo: [NSLocalizedDescriptionKey: "No core handle available"])
        }

        // The coreHandle is the filesystem ID (AfFs/u64) stored as UnsafeMutableRawPointer
        let fsId = UInt64(bitPattern: UInt(coreHandle))

        // Prepare response buffer (reasonable size for SSZ responses)
        let maxResponseSize = 4096
        var responseBuffer = [UInt8](repeating: 0, count: maxResponseSize)
        var actualResponseSize: usize = 0

        // Call the thin FFI function with raw SSZ bytes
        let result = data.withUnsafeBytes { requestPtr in
            responseBuffer.withUnsafeMutableBytes { responsePtr in
                af_control_request(
                    fsId,
                    requestPtr.baseAddress?.assumingMemoryBound(to: UInt8.self),
                    data.count,
                    responsePtr.baseAddress?.assumingMemoryBound(to: UInt8.self),
                    maxResponseSize,
                    &actualResponseSize
                )
            }
        }

        if result != 0 {
            logger.error("Control request failed with code: \(result)")
            throw NSError(domain: "AgentFS", code: Int(result), userInfo: [NSLocalizedDescriptionKey: "Control request failed"])
        }

        logger.debug("Control request succeeded, response size: \(actualResponseSize)")
    }
}
