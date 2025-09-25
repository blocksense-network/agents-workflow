//
//  AgentFSKitExtension.swift
//  AgentFSKitExtension
//
//  Created by AgentFS on 2025-01-22.
//

import Foundation
import FSKit

// AgentFS FFI functions
@_silgen_name("af_fs_create")
func af_fs_create(_ config_json: UnsafePointer<CChar>?, _ out_fs: UnsafeMutablePointer<UInt64>?) -> Int32

@_silgen_name("af_fs_destroy")
func af_fs_destroy(_ fs: UInt64) -> Int32

@_silgen_name("af_snapshot_create")
func af_snapshot_create(_ fs: UInt64, _ name: UnsafePointer<CChar>?, _ out_id: UnsafeMutablePointer<UInt8>?) -> Int32

@_silgen_name("af_branch_create_from_snapshot")
func af_branch_create_from_snapshot(_ fs: UInt64, _ snap: UnsafePointer<UInt8>?, _ name: UnsafePointer<CChar>?, _ out_id: UnsafeMutablePointer<UInt8>?) -> Int32

@_silgen_name("af_bind_process_to_branch")
func af_bind_process_to_branch(_ fs: UInt64, _ branch: UnsafePointer<UInt8>?) -> Int32

/// XPC protocol for AgentFS control operations
@objc protocol AgentFSControlProtocol {
    func createSnapshot(name: String?, reply: @escaping (Data?, Error?) -> Void)
    func createBranch(fromSnapshot snapshotId: Data, branchName: String, reply: @escaping (Data?, Error?) -> Void)
    func bindProcess(toBranch branchId: Data, reply: @escaping (Error?) -> Void)
    func listSnapshots(reply: @escaping ([Data]?, Error?) -> Void)
    func listBranches(reply: @escaping ([Data]?, Error?) -> Void)
}

/// XPC service implementation
@available(macOS 15.4, *)
class AgentFSControlService: NSObject, AgentFSControlProtocol, NSXPCListenerDelegate {

    private let coreHandle: UnsafeMutableRawPointer?
    private var listener: NSXPCListener?
    private var fsId: UInt64?

    init(coreHandle: UnsafeMutableRawPointer?) {
        self.coreHandle = coreHandle
        super.init()

        // Extract filesystem ID from core handle
        if let coreHandle = coreHandle {
            self.fsId = coreHandle.load(as: UInt64.self)
        }

        setupXPCListener()
    }

    private func setupXPCListener() {
        // Create XPC listener for the service
        listener = NSXPCListener(machServiceName: "com.agentfs.AgentFSKitExtension.control")
        listener?.delegate = self
        listener?.resume()
    }

    func createSnapshot(name: String?, reply: @escaping (Data?, Error?) -> Void) {
        guard let fsId = fsId else {
            reply(nil, NSError(domain: "AgentFS", code: -1, userInfo: [NSLocalizedDescriptionKey: "No filesystem available"]))
            return
        }

        var snapshotId = [UInt8](repeating: 0, count: 16) // 16 bytes for snapshot ID

        let result = name.map { name in
            name.withCString { cName in
                af_snapshot_create(fsId, cName, &snapshotId)
            }
        } ?? af_snapshot_create(fsId, nil, &snapshotId)

        if result == 0 { // AfResult::AfOk = 0
            reply(Data(snapshotId), nil)
        } else {
            reply(nil, NSError(domain: "AgentFS", code: Int(result), userInfo: [NSLocalizedDescriptionKey: "Failed to create snapshot"]))
        }
    }

    func createBranch(fromSnapshot snapshotId: Data, branchName: String, reply: @escaping (Data?, Error?) -> Void) {
        guard let fsId = fsId else {
            reply(nil, NSError(domain: "AgentFS", code: -1, userInfo: [NSLocalizedDescriptionKey: "No filesystem available"]))
            return
        }

        var branchId = [UInt8](repeating: 0, count: 16) // 16 bytes for branch ID

        let result = snapshotId.withUnsafeBytes { snapPtr in
            branchName.withCString { cName in
                af_branch_create_from_snapshot(fsId, snapPtr.baseAddress?.assumingMemoryBound(to: UInt8.self), cName, &branchId)
            }
        }

        if result == 0 { // AfResult::AfOk = 0
            reply(Data(branchId), nil)
        } else {
            reply(nil, NSError(domain: "AgentFS", code: Int(result), userInfo: [NSLocalizedDescriptionKey: "Failed to create branch"]))
        }
    }

    func bindProcess(toBranch branchId: Data, reply: @escaping (Error?) -> Void) {
        guard let fsId = fsId else {
            reply(NSError(domain: "AgentFS", code: -1, userInfo: [NSLocalizedDescriptionKey: "No filesystem available"]))
            return
        }

        let result = branchId.withUnsafeBytes { branchPtr in
            af_bind_process_to_branch(fsId, branchPtr.baseAddress?.assumingMemoryBound(to: UInt8.self))
        }

        if result == 0 { // AfResult::AfOk = 0
            reply(nil)
        } else {
            reply(NSError(domain: "AgentFS", code: Int(result), userInfo: [NSLocalizedDescriptionKey: "Failed to bind process to branch"]))
        }
    }

    func listSnapshots(reply: @escaping ([Data]?, Error?) -> Void) {
        // TODO: Implement snapshot listing in AgentFS FFI
        // For now, return empty list
        reply([], nil)
    }

    func listBranches(reply: @escaping ([Data]?, Error?) -> Void) {
        // TODO: Implement branch listing in AgentFS FFI
        // For now, return empty list
        reply([], nil)
    }

    // MARK: - NSXPCListenerDelegate

    func listener(_ listener: NSXPCListener, shouldAcceptNewConnection newConnection: NSXPCConnection) -> Bool {
        // Client authentication: Only accept connections from the same process or team
        // This prevents unauthorized processes from connecting to the XPC service

        // For security, we could check:
        // 1. Process belongs to same team ID (requires audit token access)
        // 2. Process has specific entitlements
        // 3. Process is running as the same user
        // For now, accept only same-process connections to reduce attack surface
        if newConnection.processIdentifier != getpid() {
            NSLog("AgentFSControl refusing XPC client pid=%d", newConnection.processIdentifier)
            return false
        }
        // TODO: Implement proper client authentication based on team ID or entitlements
        // Example: let clientPID = newConnection.processIdentifier

        let interface = NSXPCInterface(with: AgentFSControlProtocol.self)

        // Secure XPC interface by whitelisting allowed classes for method arguments
        // This prevents deserialization attacks by restricting what objects can be passed
        // Implementation follows: specs/Research/Securing-XPC-Interfaces-with-NSXPCInterface.md

        // createSnapshot(name: String?, reply: @escaping (Data?, Error?) -> Void)
        // Allow NSString for the optional name parameter
        let createSnapshotClasses: Set<AnyHashable> = NSSet(objects: NSString.self) as! Set<AnyHashable>
        interface.setClasses(createSnapshotClasses, for: #selector(AgentFSControlProtocol.createSnapshot(name:reply:)), argumentIndex: 0, ofReply: false)

        // createBranch(fromSnapshot snapshotId: Data, branchName: String, reply: @escaping (Data?, Error?) -> Void)
        // Allow NSData for snapshotId parameter, NSString for branchName parameter
        let snapshotIdClasses: Set<AnyHashable> = NSSet(objects: NSData.self) as! Set<AnyHashable>
        let branchNameClasses: Set<AnyHashable> = NSSet(objects: NSString.self) as! Set<AnyHashable>
        interface.setClasses(snapshotIdClasses, for: #selector(AgentFSControlProtocol.createBranch(fromSnapshot:branchName:reply:)), argumentIndex: 0, ofReply: false)
        interface.setClasses(branchNameClasses, for: #selector(AgentFSControlProtocol.createBranch(fromSnapshot:branchName:reply:)), argumentIndex: 1, ofReply: false)

        // bindProcess(toBranch branchId: Data, reply: @escaping (Error?) -> Void)
        // Allow NSData for branchId parameter
        let branchIdClasses: Set<AnyHashable> = NSSet(objects: NSData.self) as! Set<AnyHashable>
        interface.setClasses(branchIdClasses, for: #selector(AgentFSControlProtocol.bindProcess(toBranch:reply:)), argumentIndex: 0, ofReply: false)

        // listSnapshots and listBranches have no input parameters, so no class restrictions needed

        newConnection.exportedInterface = interface
        newConnection.exportedObject = self
        newConnection.resume()
        return true
    }
}

@available(macOS 15.4, *)
@main
struct AgentFSKitExtension : UnaryFileSystemExtension {

    var fileSystem : FSUnaryFileSystem & FSUnaryFileSystemOperations {
        AgentFsUnary()
    }
}
