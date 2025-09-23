// swift-tools-version: 6.0
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "AgentFSKitExtension",
    platforms: [
        .macOS("15.4")
    ],
    products: [
        .executable(
            name: "AgentFSKitExtension",
            targets: ["AgentFSKitExtension"]
        )
    ],
    dependencies: [],
    targets: [
        .executableTarget(
            name: "AgentFSKitExtension",
            dependencies: [],
            path: "AgentFSKitExtension",
            exclude: [
                "AgentFSKitFFI.h",
                "AgentFSKitFFI.modulemap"
            ],
            sources: [
                "AgentFSKitExtension.swift",
                "AgentFsUnary.swift",
                "AgentFsVolume.swift",
                "AgentFsItem.swift",
                "Constants.swift",
                "AgentFSBridge.c"
            ],
            cSettings: [
                .headerSearchPath(".")
            ],
            swiftSettings: [
                .interoperabilityMode(.Cxx)
            ],
            linkerSettings: [
                .unsafeFlags(["-L", "libs"]),
                .linkedLibrary("agentfs_fskit_bridge"),
                .linkedLibrary("agentfs_fskit_sys"),
                .linkedLibrary("agentfs_ffi"),
            ]
        )
    ]
)
