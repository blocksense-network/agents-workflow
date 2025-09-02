# Compiling an FSKit-Based File System Extension (Rust \+ Swift)

**Prerequisites:** Make sure you have Xcode (with macOS 15.4+ SDK) installed and Command Line Tools enabled, and a Rust toolchain (via rustup) installed. The steps below use Terminal commands so you can avoid using Xcode’s GUI.

## 1\. Set Up a New FSKit Extension Project

- **Initialize a host app and FSKit extension:** Create a new macOS App project (e.g. **MyFSApp**) using Xcode or a tool like Xcodegen. In the project, add a **File System Extension** target (e.g. **MyFSExtension**). This extension will contain your Swift wrapper.

- **Configure FSKit capabilities:** Enable the **“FSKit Module”** capability for the extension (under Signing & Capabilities). This adds the entitlement indicating your extension provides an FSKit filesystem[\[1\]](https://developer.apple.com/forums/tags/fskit#:~:text=extension%20is%20enabled%20in%20System,signing%20certificate).

- **Info.plist keys:** In the extension’s **Info.plist**, add FSKit-specific keys:

- FSName and FSShortName – the human-readable and short name of your filesystem (e.g. **MyFS**)[\[1\]](https://developer.apple.com/forums/tags/fskit#:~:text=extension%20is%20enabled%20in%20System,signing%20certificate). The short name is used for mounting (e.g. via the mount command).

- FSSupportsBlockResources – set to **YES** if your filesystem will mount block devices (most FS extensions will, since they act on disk images or devices)[\[1\]](https://developer.apple.com/forums/tags/fskit#:~:text=extension%20is%20enabled%20in%20System,signing%20certificate).

- **Signing:** Use a development team if available. Xcode can automatically manage a provisioning profile for the app and extension. (Without a paid Developer ID, you can use a local “Apple Development” certificate or temporarily disable code signing for testing.)

## 2\. Write the Rust Core Library

- **Create a Rust library crate:** In your project directory, initialize a new Rust library:

- cargo new \--lib myfs

- This creates a myfs crate for your filesystem logic. In Cargo.toml, set the library type to a static library:

- \[lib\]  
  name \= "myfs"  
  crate-type \= \["staticlib"\]

- **Implement filesystem logic in Rust:** Write your Rust code to handle the filesystem’s operations. Expose C-compatible functions for the Swift code to call. For example, you might define functions like:

- \#\[no_mangle\]  
  pub extern "C" fn myfs_init() { /\* ... \*/ }  
  \#\[no_mangle\]  
  pub extern "C" fn myfs_read(path: \*const c_char, buffer: \*mut u8, len: usize) \-\> i32 { /\* ... \*/ }

- Use \#\[no_mangle\] and an extern "C" ABI so that the symbols are accessible from Swift/ObjC. Keep the API surface minimal – the Swift extension will invoke these for filesystem operations.

- **Generate a C header for the Rust library:** You need a C header file to expose the Rust functions to Swift. You can create this manually or use **cbindgen** to generate it. Using cbindgen is convenient[\[2\]](https://medium.com/@kennethyoel/a-swiftly-oxidizing-tutorial-44b86e8d84f5#:~:text=For%20building%20the%20header%20file,cbindgen%20can%20be%20located%20here):

- cargo install \--force cbindgen \# install/update cbindgen  
  cbindgen \--lang c \--output myfs.h

- This produces a myfs.h with declarations for your extern "C" functions. (Double-check the header and adjust if needed.)

## 3\. Build the Rust Library for macOS

- **Add target architectures:** For a universal binary supporting both Apple Silicon (arm64) and Intel (x86_64) Macs, add the targets and build for each:

- rustup target add aarch64-apple-darwin x86_64-apple-darwin\[3\]  
  cargo build \--release \--target=aarch64-apple-darwin  
  cargo build \--release \--target=x86_64-apple-darwin

- **Create a universal static library:** Use the lipo tool to combine the two builds into one fat library. For example:

- lipo \-create \-output libmyfs.a \\  
   target/aarch64-apple-darwin/release/libmyfs.a \\  
   target/x86_64-apple-darwin/release/libmyfs.a\[4\]

- This produces libmyfs.a containing both architectures (arm64 \+ x86_64).\* (If you only need to run on your current Mac architecture for testing, you can skip this and use the single-arch library from target/\<arch\>/release/.)

- **Copy the library to your project:** Take the resulting libmyfs.a (or the single-arch .a if not lipo’d) and place it in a known location (e.g. copy into your Xcode project folder or a libs directory).

## 4\. Bridge the Rust Library with the Swift Extension

- **Include the Rust header in Swift:** Add the generated myfs.h to your Xcode project (e.g. by copying it into the extension folder). Create a Bridging Header for the Swift extension (if Xcode didn’t create one by default). For example, add a file **MyFSExtension-Bridging-Header.h** and in Build Settings \> **Objective-C Bridging Header** set its path. In that bridging header, import the Rust header:

- // MyFSExtension-Bridging-Header.h  
  \#include "myfs.h"

- This tells Swift to read the C header so it can use the functions – Swift will automatically bridge them for use in your code[\[5\]](https://medium.com/@kennethyoel/a-swiftly-oxidizing-tutorial-44b86e8d84f5#:~:text=For%20us%20to%20use%20our,we%20can%20use%20right%20away). (Ensure the header is in the extension target’s **Header Search Paths** if needed.)

- **Link the Rust library:** Edit the extension target’s build settings to link against libmyfs.a. The simplest approach is to add the .a file to the **“Link Binary With Libraries”** phase of the extension target (you can do this by editing the Xcode project or via xcodebuild settings). This ensures the Rust code is bundled into the extension.

- **Implement the Swift wrapper:** In your Swift extension target, subclass the appropriate FSKit base class and protocol, then call into Rust within those methods. For example, create a class MyFileSystem: FSUnaryFileSystem that conforms to FSUnaryFileSystemOperations. Implement required methods like probeResource(...), mount(volume:...), enumerateDirectory(...), etc., as specified by FSKit. Inside each implementation, delegate to your Rust library for the heavy lifting. For instance, in a read or directory-listing method, you might call a Rust function (imported via the bridging header) to get file data or directory contents. Apple’s FSKit expects your extension to provide these hooks[\[6\]](https://github.com/KhaosT/FSKitSample#:~:text=,of%20your%20custom%20filesystem%20implementation) – your Swift code can remain thin by passing requests to Rust and then converting the results to FSKit’s expected types.

_👉 Note:_ FSKit's framework provides classes like FSUnaryFileSystem, FSVolume, and protocols for operations. Ensure your Swift code returns the proper FS objects and errors as expected by FSKit. The Rust code can manage actual filesystem data structures, while Swift just translates between FSKit API and Rust FFI.\[6\](<https://github.com/KhaosT/FSKitSample#:~:text=,of%20your%20custom%20filesystem%20implementation>n>n>n>n>n>n>n>n>n>)

## 5\. Compile the Extension via Command Line

With the project configured, build everything using Xcode’s CLI:  
\- **Build with xcodebuild:** Use the xcodebuild tool to compile the app and extension without opening Xcode’s UI. For example, in Terminal, navigate to your project directory and run:

xcodebuild \-scheme MyFSApp \-configuration Debug build

(If you have a workspace or a different scheme name, include \-workspace or adjust the scheme name accordingly.) This will compile the host app **and** the extension. You should see the build progress and a **Build Succeeded** message.  
\- **Signing considerations:** By default, Xcode will try to code-sign the build. If you set up a Development Team in the project, xcodebuild will auto-sign with a development certificate (you may need to include \-allowProvisioningUpdates on the command to let Xcode create a profile). If you don't have any code signing set up, you can instruct Xcode to build without signing by adding CODE_SIGNING_ALLOWED=NO to the xcodebuild command.\*(Unsigned binaries can still be tested locally, but macOS will prompt for approval — more on this below.)\*

After build, the products will be in Xcode’s build output directory (e.g. ./build/Build/Products/Debug/MyFSApp.app). You can also find the .appex (extension bundle) inside the app bundle under MyFSApp.app/Contents/PlugIns/MyFSExtension.appex.

## 6\. Run the Host App and Enable the Extension

- **Launch the app to register the extension:** Run the built app (e.g. via open ./build/Build/Products/Debug/MyFSApp.app in Terminal, or by double-clicking it in Finder). The app itself might not have a UI, but launching it tells macOS to register the embedded File System Extension with the system (via PlugInKit).

- **Approve/enable the extension:** Open **System Settings \> General \> Login Items & Extensions \> File System Extensions**. You should see your **MyFSExtension** listed. Enable it (toggle it on) to allow it to run[\[7\]](https://github.com/KhaosT/FSKitSample#:~:text=Once%20you%20build%20and%20run,is%20a%20block%20device). If the extension is not appearing, or if it’s disabled due to signing, check **System Settings \> Privacy & Security** – you might see a message about “System software from an unidentified developer was blocked”. If so, click “Allow” for your app/extension and follow prompts (you may need to reboot if prompted, though FSKit extensions typically just require permission and enabling).

- **Development mode:** On macOS 15+, if you encounter any warnings about requiring a “Developer Mode” for system extensions, you may need to enable Developer Mode in Privacy & Security (this is usually for DriverKit, and **FSKit** being user-space shouldn’t need a reboot). Generally, if the extension is correctly signed or allowed, it will show up for enabling. Once you toggle it on, the system will launch your extension process when needed.

## 7\. Mount and Test Your File System

With the extension running (enabled), you can now mount a volume that uses your filesystem driver:  
\- **Create a mount point:** Decide on a directory to serve as the mount point, e.g. mkdir \~/TestMount (or /tmp/TestVol as in Apple’s example).  
\- **Use the mount command:** Invoke the mount tool with the \-F option to specify a File System Extension. For example:

sudo mkdir /tmp/TestVol
sudo mount \-F \-t MyFS disk18 /tmp/TestVol

Here, "MyFS" is the **FSShortName** you set earlier, and disk18 is the device to mount[\[7\]](https://github.com/KhaosT/FSKitSample#:~:text=Once%20you%20build%20and%20run,is%20a%20block%20device)[\[1\]](https://developer.apple.com/forums/tags/fskit#:~:text=extension%20is%20enabled%20in%20System,signing%20certificate). Replace disk18 with the actual device identifier or disk image you want to mount. The \-F flag tells macOS to use a File System extension rather than a built-in kernel FS. When you run this command, macOS will locate your extension (since it’s enabled) and spawn the extension to handle the mount operation.  
\- **Provide a resource to mount:** FSKit modules typically expect a block device. If you don’t have a physical device handy, you can simulate one: create a dummy disk image and attach it as a raw device. For example:

mkfile \-n 100m dummy.img \# create a 100 MB file (Mac only; use \`dd\` if mkfile isn’t available)  
hdiutil attach \-imagekey diskimage-class=CRawDiskImage \-nomount dummy.img

The hdiutil command will output a device like /dev/diskN. You can then use that device in the mount command (e.g., sudo mount \-F \-t MyFS diskN /tmp/TestVol). This approach simulates a block device backed by your file, which your FS extension can then interpret[\[8\]](https://github.com/KhaosT/FSKitSample#:~:text=To%20create%20a%20dummy%20block,you%20can%20do%20the%20following).  
\- **Verify the filesystem:** If the mount succeeds, no error will be reported. You can check mount to see the mounted volume, or list the mount point (ls /tmp/TestVol). Your Rust logic (via Swift) should populate the filesystem data. For example, if your Rust code on enumerateDirectory returns some dummy files, you should see them. Logging from your extension (e.g. print or os_log in Swift, or stderr prints in Rust) can be viewed via the Console app or log stream.  
\- **Troubleshooting:** If the mount command fails with an error, check the Console logs for messages from your extension or system (common issues are missing entitlements, incorrect FSShortName, or permission issues). A “Permission denied” or inability to invoke the extension usually points to code signing or entitlement problems – ensure the FSKit entitlement is present and that the extension is enabled[\[1\]](https://developer.apple.com/forums/tags/fskit#:~:text=extension%20is%20enabled%20in%20System,signing%20certificate).

- **Unmount when done:** Once finished testing, unmount the volume:

- sudo umount /tmp/TestVol

- This will shut down the extension’s instance for that mount[\[9\]](https://github.com/KhaosT/FSKitSample#:~:text=And%20unmount%20them%20with). You can then disable or uninstall the extension if desired (to uninstall, remove the app or use pluginkit \-r to deregister, but during development simply leaving it disabled is fine).

**Sources:** The steps above are based on Apple’s FSKit documentation and examples. FSKit (introduced in macOS 15.4 “Sequoia”) allows user-space filesystem modules[\[10\]](https://github.com/KhaosT/FSKitSample#:~:text=FSKit%20is%20the%20new%20framework,how%20to%20use%20the%20framework). We’ve followed an approach of writing the core logic in Rust with a thin Swift wrapper, which is supported by Swift’s ability to call C libraries[\[5\]](https://medium.com/@kennethyoel/a-swiftly-oxidizing-tutorial-44b86e8d84f5#:~:text=For%20us%20to%20use%20our,we%20can%20use%20right%20away)[\[2\]](https://medium.com/@kennethyoel/a-swiftly-oxidizing-tutorial-44b86e8d84f5#:~:text=For%20building%20the%20header%20file,cbindgen%20can%20be%20located%20here). The official sample project by KhaosT demonstrates a pure Swift FSKit extension, including the need to enable it in System Settings and mount via the mount \-F command[\[7\]](https://github.com/KhaosT/FSKitSample#:~:text=Once%20you%20build%20and%20run,is%20a%20block%20device). We also referenced a Rust-in-Swift integration tutorial for building and linking the Rust library (using cbindgen and lipo for universal binaries)[\[3\]](https://medium.com/@kennethyoel/a-swiftly-oxidizing-tutorial-44b86e8d84f5#:~:text=rustup%20target%20add%20aarch64)[\[4\]](https://medium.com/@kennethyoel/a-swiftly-oxidizing-tutorial-44b86e8d84f5#:~:text=%40%24%28RM%29%20,macabi%2Frelease%2Flibmunchausen.a). By following this guide, you should be able to compile your FSKit-based extension entirely from the command line and get it running on macOS 15.4+.

---

[\[1\]](https://developer.apple.com/forums/tags/fskit#:~:text=extension%20is%20enabled%20in%20System,signing%20certificate) FSKit | Apple Developer Forums

[https://developer.apple.com/forums/tags/fskit](https://developer.apple.com/forums/tags/fskit)

[\[2\]](https://medium.com/@kennethyoel/a-swiftly-oxidizing-tutorial-44b86e8d84f5#:~:text=For%20building%20the%20header%20file,cbindgen%20can%20be%20located%20here) [\[3\]](https://medium.com/@kennethyoel/a-swiftly-oxidizing-tutorial-44b86e8d84f5#:~:text=rustup%20target%20add%20aarch64) [\[4\]](https://medium.com/@kennethyoel/a-swiftly-oxidizing-tutorial-44b86e8d84f5#:~:text=%40%24%28RM%29%20,macabi%2Frelease%2Flibmunchausen.a) [\[5\]](https://medium.com/@kennethyoel/a-swiftly-oxidizing-tutorial-44b86e8d84f5#:~:text=For%20us%20to%20use%20our,we%20can%20use%20right%20away) Rust Library in Swift. Packaging native libraries written in… | by Kenneth Yo'el | Medium

[https://medium.com/@kennethyoel/a-swiftly-oxidizing-tutorial-44b86e8d84f5](https://medium.com/@kennethyoel/a-swiftly-oxidizing-tutorial-44b86e8d84f5)

[\[6\]](https://github.com/KhaosT/FSKitSample#:~:text=,of%20your%20custom%20filesystem%20implementation) [\[7\]](https://github.com/KhaosT/FSKitSample#:~:text=Once%20you%20build%20and%20run,is%20a%20block%20device) [\[8\]](https://github.com/KhaosT/FSKitSample#:~:text=To%20create%20a%20dummy%20block,you%20can%20do%20the%20following) [\[9\]](https://github.com/KhaosT/FSKitSample#:~:text=And%20unmount%20them%20with) [\[10\]](https://github.com/KhaosT/FSKitSample#:~:text=FSKit%20is%20the%20new%20framework,how%20to%20use%20the%20framework) GitHub \- KhaosT/FSKitSample: FSKit example setup

<https://github.com/KhaosT/FSKitSample>
