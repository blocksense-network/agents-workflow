
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

@_silgen_name("agentfs_bridge_snapshot_create")
func agentfs_bridge_snapshot_create(_ core: UnsafeMutableRawPointer?, _ name: UnsafePointer<CChar>?, _ snapshot_id: UnsafeMutablePointer<CChar>?, _ snapshot_id_size: size_t) -> Int32

@_silgen_name("agentfs_bridge_branch_create")
func agentfs_bridge_branch_create(_ core: UnsafeMutableRawPointer?, _ snapshot_id: UnsafePointer<CChar>?, _ branch_name: UnsafePointer<CChar>?, _ branch_id: UnsafeMutablePointer<CChar>?, _ branch_id_size: size_t) -> Int32

@_silgen_name("agentfs_bridge_bind_process")
func agentfs_bridge_bind_process(_ core: UnsafeMutableRawPointer?, _ branch_id: UnsafePointer<CChar>?) -> Int32

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


// @available(macOS 15.4, *)
// extension AgentFsVolume: FSVolume.Operations {
    // Operations extension - to be implemented
// }
