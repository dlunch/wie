// swift-tools-version:5.3
import PackageDescription

let package = Package(
    name: "wie-app-platform",
    platforms: [.iOS(.v13)],
    products: [.library(name: "wie-app-platform", type: .static, targets: ["wie-app-platform"])],
    dependencies: [
        .package(name: "Tauri", path: "../.tauri/tauri-api"),
        .package(url: "https://github.com/googleads/swift-package-manager-google-mobile-ads.git", from: "13.11.0"),
    ],
    targets: [
        .target(
            name: "wie-app-platform",
            dependencies: [
                .byName(name: "Tauri"),
                .product(name: "GoogleMobileAds", package: "swift-package-manager-google-mobile-ads"),
            ],
            path: "Sources"
        ),
    ]
)
