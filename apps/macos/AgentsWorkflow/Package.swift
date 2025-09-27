// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "AgentsWorkflow",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(
            name: "AgentsWorkflow",
            targets: ["AgentsWorkflow"]
        )
    ],
    dependencies: [],
    targets: [
        .executableTarget(
            name: "AgentsWorkflow",
            dependencies: [],
            path: "AgentsWorkflow",
            exclude: ["Info.plist"],
            resources: [
                .copy("../../../adapters/macos/xcode/AgentFSKitExtension/AgentFSKitExtension.appex")
            ]
        )
    ]
)
