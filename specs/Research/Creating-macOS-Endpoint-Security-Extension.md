# Creating a macOS Endpoint Security Extension for File & Network Monitoring

Developing a security **Endpoint Security** extension on macOS allows you to monitor and control file system access and network connections for targeted processes. Below is a comprehensive step-by-step guide to create such an extension (in Rust or Swift, Rust preferred) via the command line, covering development through packaging and notarization. This guide assumes macOS 11+ (Big Sur or later) and Xcode command-line tools installed.

## Prerequisites and Entitlements

1. **Apple Developer Setup:** Ensure you have a registered Apple Developer account and have Xcode’s Command Line Tools installed (xcode-select \--install). You will need to request and obtain Apple’s special Endpoint Security entitlement (com.apple.developer.endpoint-security.client) for distribution[\[1\]](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security#:~:text=Endpoint%20Security%3A)[\[2\]](https://prodisup.com/posts/2022/01/building-and-testing-an-endpoint-security-macos-system-extension-on-bitrise/#:~:text=5). Apple requires a justification for this entitlement, and approval can take weeks[\[3\]](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security#:~:text=,application%20needs%20not%20only%20the). _(During development, if you lack this entitlement, you can test on a Mac with System Integrity Protection (SIP) disabled or in reduced security mode to allow the extension to run[\[2\]](https://prodisup.com/posts/2022/01/building-and-testing-an-endpoint-security-macos-system-extension-on-bitrise/#:~:text=5).)_ Similarly, to monitor network traffic, you may need Network Extension content filter entitlements (e.g. com.apple.developer.networking.networkextension with filter rights) from Apple.

2. **Enable System Extensions in macOS:** On Apple Silicon Macs, you must allow third-party system extensions. Reboot to Recovery and use the Startup Security Utility to set _Reduced Security_ and enable “Allow user management of kernel extensions (system extensions) from identified developers”[\[4\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=While%20System%20Extensions%20enhance%20the,the%20application%20to%20function%20correctly)[\[5\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=5,to%20reboot%20your%20Mac). After reboot, if macOS shows a “System Extension Blocked” alert, go to **System Settings \> Privacy & Security** and **Allow** the extension[\[6\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=7,to%20reboot%20your%20Mac)[\[7\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=Extension%20was%20blocked,software%20was%20blocked%20from%20loading). (This is needed to run your extension on a development machine and for users to enable it.)

## Step 1: Set Up Project Structure from the Command Line

All development can be done via terminal without opening Xcode’s GUI:

- **Create a Host App Bundle:** macOS System Extensions must be contained in an application bundle[\[8\]](https://prodisup.com/posts/2022/01/building-and-testing-an-endpoint-security-macos-system-extension-on-bitrise/#:~:text=1). Create a project directory, e.g. SecurityExtensionProj, and within it set up a minimal dummy app bundle structure:

- mkdir \-p SecurityExtensionProj/MySecurityApp.app/Contents/MacOS  
  mkdir \-p SecurityExtensionProj/MySecurityApp.app/Contents/Library/SystemExtensions/MyEndpointExt.systemextension/Contents/MacOS

- Here, MySecurityApp.app is a container (it can be a trivial app that does nothing or just helps install the extension), and MyEndpointExt.systemextension is the Endpoint Security extension bundle (with the .systemextension suffix)[\[9\]](https://github.com/redcanaryco/mac-monitor#:~:text=,com.redcanary.agent.securityextension.systemextension).

- **Info.plist for App and Extension:** Using a text editor, create an Info.plist inside MySecurityApp.app/Contents/ with at least a **CFBundleIdentifier** (e.g. com.example.mysecurityapp) and a **NSPrincipalClass** (often empty for a non-UI app). In the extension bundle’s Contents, create an Info.plist with keys:

- **CFBundleIdentifier** (e.g. com.example.mysecurityapp.endpointext).

- **NSExtensionPointIdentifier** \= com.apple.system_extension.endpoint_security (tells the system this is an Endpoint Security system extension)[\[10\]](https://discussions.apple.com/thread/254455105#:~:text=Communities%20discussions.apple.com%20%20,GT8P3H7SPW%20com.mcafee.CMF).

- **NSExtensionPrincipalClass** \= an empty string ("") for a non-UI system extension (no principal class needed, the extension runs its main() like a daemon).

- **NSEndpointSecurityEarlyBoot** (Boolean, optional) \= true _if_ you need the extension to load during early boot (not usually needed for development)[\[11\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=endpoint%20security%20system%20extensions%3A).

- Optionally, **NSSystemExtensionUsageDescription** with a user-facing message explaining why the extension needs to run (displayed when the user is prompted to allow it).

- **Entitlements:** Prepare an entitlements file for the extension (e.g. endpoint-ext.entitlements) enabling the necessary privileges:

- \<?xml version="1.0" encoding="UTF-8"?\>  
  \<\!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "<https://www.apple.com/DTDs/PropertyList-1.0.dtd"\>>>>>>>>>>>>>>>>>>>>
  \<plist version="1.0"\>  
  \<dict\>  
   \<key\>com.apple.developer.endpoint-security.client\</key\>  
   \<true/\>  
   \<key\>com.apple.security.get-task-allow\</key\>  
   \<true/\>  
   \<key\>com.apple.security.cs.disable-library-validation\</key\>  
   \<true/\>  
   \<\!-- If using a Network Extension Content Filter as well: \--\>  
   \<\!-- \<key\>com.apple.developer.networking.networkextension\</key\>  
   \<array\>\<string\>content-filter-provider\</string\>\</array\> \--\>  
  \</dict\>  
  \</plist\>

- The Endpoint Security entitlement is critical – without it, es_new_client will fail with ES_NEW_CLIENT_RESULT_ERR_NOT_ENTITLED[\[12\]](https://objective-see.org/blog/blog_0x47.html#:~:text=%2F%2F%2FThe%20caller%20is%20not%20properly,entitled%20to%20connect%20ES_NEW_CLIENT_RESULT_ERR_NOT_ENTITLED)[\[13\]](https://objective-see.org/blog/blog_0x47.html#:~:text=Hopefully%20these%20are%20rather%20self,security.client%60%20entitlement). For local testing without the entitlement, disabling SIP can bypass this check (not for production)[\[2\]](https://prodisup.com/posts/2022/01/building-and-testing-an-endpoint-security-macos-system-extension-on-bitrise/#:~:text=5). If implementing a network content filter extension, you would include the appropriate Network Extension entitlement as well. Make sure to also create an entitlements file for the app container if needed (usually just enabling com.apple.security.app-sandbox \= false or other basics, plus the SystemExtension install capability).

## Step 2: Implement the Endpoint Security Extension (File Monitoring)

Now, implement the extension logic in Rust (or Swift). We will focus on Rust using the **EndpointSecurity C API** via Rust bindings for brevity. Create a new Rust cargo project for the extension binary:

- **Initialize Rust Project:** In SecurityExtensionProj, run:

- cargo init \--bin endpoint_ext

- This creates src/main.rs and a Cargo.toml. Adjust Cargo.toml to build a Mach-O suitable for an extension:

- \[package\]  
  name \= "endpoint_ext"  
  version \= "0.1.0"  
  edition \= "2021"  
  \# Ensure the output binary name matches your extension's bundle executable name  
  \[\[bin\]\]  
  name = "MyEndpointExt"
  path \= "src/main.rs"

- **Add Endpoint Security Crate:** Add a dependency for Endpoint Security bindings. For example, you can use the open-source crate **endpoint-sec** (by HarfangLab) which provides safe Rust bindings:

- cargo add endpoint-sec

- This crate wraps Apple’s EndpointSecurity.framework (available on macOS 10.15+ only)[\[14\]](https://docs.rs/endpoint-sec/latest/endpoint_sec/#:~:text=Available%20on%20macOS%20only). It provides a Client abstraction for the ES client and Rust enums/structs for events.

- **Write the Extension Code:** Edit src/main.rs to set up the ES client, subscribe to events, and handle them. Specifically, we will monitor file system events (like file open) and possibly process exec events to track the sandbox processes. We’ll also prepare to handle network events (explained later). A simplified example in Rust:

- use endpoint_sec::{Client, EventType, Event, Action, sys};  
  use std::process;

  fn main() {  
   // Set ES runtime version to current OS for compatibility  
   endpoint_sec::version::set_runtime_version().expect("Failed to set ES version");

      // Define which events to subscribe to (authorization events for file opens and process execs)
      let subscribed \= &\[
          EventType::AuthOpen,         // file open (auth) events
          EventType::AuthExecute,      // process exec auth events (to catch sandbox process launches)
          // ... (add other auth events if needed like AuthCreate, AuthUipcConnect)
      \];

      // Create ES client with an event handler closure
      let client \= Client::new(move |message| {
          // Each ES message corresponds to an event
          let proc\_info \= message.process();
          let proc\_path \= proc\_info.executable().path().to\_string\_lossy();
          let event \= message.event();

          // Filter: Only consider events from our sandboxed root process or its descendants
          // (Assume we know the sandbox root process path or identifier)
          let sandbox\_root \= "/Path/To/SandboxRootExecutable";
          let is\_sandbox\_proc \= proc\_path.starts\_with(sandbox\_root)
              || /\* (check parent lineage via proc\_info.ppid if needed) \*/ false;
          if \!is\_sandbox\_proc {
              // Ignore events not from our sandbox environment
              return;
          }

          match event {
              Event::AuthOpen(open\_event) \=\> {
                  // Dynamically approve the file open (ES auth event) by responding with ALLOW
                  message.respond(sys::es\_auth\_result\_t::ES\_AUTH\_RESULT\_ALLOW)
                         .expect("Failed to respond to AuthOpen");
                  println\!("Allowed file open: {:?}", open\_event.target().path());
              },
              Event::AuthExec(exec\_event) \=\> {
                  // A process is about to execute; if it's the root sandbox launching, we allow it
                  message.respond(sys::es\_auth\_result\_t::ES\_AUTH\_RESULT\_ALLOW)
                         .expect("Failed to respond to AuthExec");
                  println\!("Allowed exec: {:?}", exec\_event.target().executable().path());
              },
              \_ \=\> {
                  // Allow all other events by default
                  if message.action() \== Action::Auth(\_) {
                      message.respond(sys::es\_auth\_result\_t::ES\_AUTH\_RESULT\_ALLOW).ok();
                  }
              }
          }
      }, subscribed).expect("Failed to create ES client");

      // Keep the extension running to continue receiving events
      println\!("Endpoint Security extension running (PID {})", process::id());
      loop { std::thread::park(); }

  }

- In this code, we use the endpoint*sec crate’s API to subscribe to auth events (file opens and execs) and define a handler. The handler checks if the event’s process originates from our sandbox root (by path or tracked parent). If so, for each **Auth** event (which requires approval) we call message.respond(ES_AUTH_RESULT_ALLOW) to **dynamically approve** the action. For example, for file open events we send an “ALLOW” verdict[\[15\]](https://speakerdeck.com/patrickwardle/mastering-apples-endpoint-security-for-advanced-macos-malware-detection#:~:text=,which%20takes%20a%20flag), permitting the file access. This must be done before a deadline – if you don’t respond in time, macOS will assume your extension is hung and kill it[\[16\]](https://docs.rs/endpoint-sec/latest/endpoint_sec/#:~:text=Client%3A%3Asubscribe%28%29%20,avoid%20stalling%20for%20the%20user). We log approvals to stdout for testing. (In a real product, you might prompt the user or consult a policy before allowing; here we automatically allow but you \_could* call ES_AUTH_RESULT_DENY to block if needed.)

**Important:** The ES client needs full disk access to monitor file events across the system. During development, give your host app “Full Disk Access” in System Preferences, or run the extension in an environment that has it (the system extension should inherit it if properly signed and approved).

- **Filtering to Sandbox Processes:** We used a simplistic check by matching the process’s executable path against a known sandbox root path. Depending on your sandbox, you might track the root process’s PID or **audit token** and use Apple’s muting APIs to filter out other events. For example, you can call es*mute_process(client, other_proc_audit_token) for any processes not of interest[\[17\]](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security#:~:text=Say%20your%20application%20processes%20a,path%20to%20the%20executable%20file)[\[18\]](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security#:~:text=C), or conversely mute everything \_except* your target by muting known other paths. Using es_mute_path or es_mute_process_events can reduce overhead by preventing irrelevant events from reaching your handler[\[18\]](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security#:~:text=C). In our simple example, we just check inside the handler and return early for processes outside the sandbox.

- **Dynamic Approval:** The code above **dynamically grants approval** for both file opens and process executions originating from the sandbox. The call to respond(ES_AUTH_RESULT_ALLOW) is what actually informs the OS to allow the operation to proceed[\[15\]](https://speakerdeck.com/patrickwardle/mastering-apples-endpoint-security-for-advanced-macos-malware-detection#:~:text=,which%20takes%20a%20flag). You can expand this to other events (e.g., file creation, or Event::AuthUipcConnect for UNIX socket connects) similarly. Ensure **each Auth event** is answered, or the operation will be blocked by default after the deadline[\[16\]](https://docs.rs/endpoint-sec/latest/endpoint_sec/#:~:text=Client%3A%3Asubscribe%28%29%20,avoid%20stalling%20for%20the%20user).

_(If using Swift instead of Rust, you would use Apple’s EndpointSecurity C APIs via Bridging or Objective-C. The logic is the same: initialize an es_new_client, subscribe to events, and implement a handler block that calls es_respond_auth_result(client, message, ES_AUTH_RESULT_ALLOW, NULL) for auth events[\[15\]](https://speakerdeck.com/patrickwardle/mastering-apples-endpoint-security-for-advanced-macos-malware-detection#:~:text=,which%20takes%20a%20flag). Swift can use the EndpointSecurity framework directly since it's C-based. You’d also set up a RunLoop or dispatch main to keep the extension alive to process events.)_

## Step 3: Implement Network Monitoring (Internet Access)

Monitoring _internet/network access_ requires a Network Extension **Content Filter**, because the Endpoint Security API itself does not provide direct events for TCP/UDP network connections. Apple’s Network Extensions allow filtering network flows or packets at the system level[\[19\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=Network%20Extensions%20empower%20developers%20to,encompasses%20several%20extension%20types%2C%20including)[\[20\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=Filter%20Data%3A%20Facilitates%20the%20filtering,data%20at%20the%20flow%20level). We suggest implementing a content filter system extension alongside the ES extension:

- **Add a Network Extension Target:** Create another extension bundle, e.g. MyNetworkFilter.systemextension under MySecurityApp.app/Contents/Library/SystemExtensions/. Its Info.plist should have **NSExtensionPointIdentifier \= com.apple.system_extension.network_extension** and a sub-dictionary for **NSExtensionAttributes** specifying the filter type (data or packet). For example, for a content filter data provider:

- \<key\>NSExtensionPointIdentifier\</key\>  
  \<string\>com.apple.system_extension.network_extension\</string\>  
  \<key\>NSExtensionAttributes\</key\>  
  \<dict\>  
   \<key\>NETunnelProvider\</key\>  
   \<false/\>  
   \<key\>NEFilterProvider\</key\>  
   \<true/\>  
   \<key\>NEFilterProviderConfiguration\</key\>  
   \<dict\>  
   \<key\>FilterPackets\</key\>\<false/\>  
   \<key\>FilterData\</key\>\<true/\>  
   \</dict\>  
  \</dict\>

- Also include a **CFBundleIdentifier** (e.g. com.example.mysecurityapp.netfilter) and any usage description keys needed.

- **Network Filter Entitlement:** Update your extension’s entitlements (or use a separate entitlements file for the network filter extension) to include Network Extension capabilities, for instance:

- \<key\>com.apple.developer.networking.networkextension\</key\>  
  \<array\>  
   \<string\>content-filter-provider\</string\>  
  \</array\>  
  \<key\>com.apple.developer.endpoint-security.client\</key\>  
  \<true/\>

- (It’s possible to combine the file-monitor and network-filter logic in one system extension process, but typically they might be separate extensions. For simplicity, you might keep one extension process that calls both EndpointSecurity and NetworkExtension APIs, though Apple’s templates often separate them.)

- **Implement Filter Logic:** If using Rust, you can leverage the Objective-C runtime to call NetworkExtension APIs (for example via the objc2 crate or similar). In Swift, you would subclass NEFilterDataProvider or NEFilterPacketProvider in the extension code. The filter extension will receive callbacks for new network flows or packets. In those callbacks, you can inspect the process (NEFilterFlow gives you the source application), and if it’s within your sandbox, decide to allow or block. For instance, a simple filter might allow all traffic but log it:

- In a Filter Data Provider (Swift pseudo-code):

- override func handleNewFlow(\_ flow: NEFilterFlow) \-\> NEFilterNewFlowVerdict {  
   if let sourceApp \= flow.sourceAppIdentifier, isSandboxApp(sourceApp) {  
   // Dynamically approve network flow for sandbox processes  
   return .allow()  
   }  
   // For others, pass or block as needed
  return .allow()
  }

- Here isSandboxApp() would check if the flow’s source matches your sandbox app's bundle ID or path. You could also implement handleInboundData/handleOutboundData to further monitor data if needed. The goal is to **dynamically grant network access** for sandbox processes by returning an allow verdict. If you wanted to prompt the user or apply policy, you could return .needMoreRules and have a companion **Filter Control** extension or the container app supply a decision.

- **Integrate with Host App:** The host app must load/activate the network filter. This is done via NEFilterManager. For example, the app can set the filter provider configuration and call NEFilterManager.shared().saveToPreferences and then enable() the filter. Ensure you include a usage description in the app’s Info.plist (like NSUserNetworkUsageDescription) and the app has the necessary entitlement to configure content filters (e.g., com.apple.developer.networking.networkextension with content-filter keys as well).

_(Note: Implementing a Network Extension can be complex. As an alternative for testing, you might forego a full network extension and instead intercept_ _Unix domain socket_ _connections from the sandbox using Endpoint Security’s ES_EVENT_TYPE_AUTH_UIPC_CONNECT events[\[21\]](https://developer.apple.com/documentation/endpointsecurity/es_event_type_auth_uipc_connect#:~:text=ES_EVENT_TYPE_AUTH_UIPC_CONNECT,Mac%20Catalyst). However, that only covers local socket connections, not internet access. For true internet monitoring, a Network Extension is the recommended approach.)_

## Step 4: Build and Compile Everything via Terminal

With code in place, you need to compile the extension and bundle it into the app:

- **Compile the Rust Extension:** Run cargo build for release:

- cargo build \--release \--target x86_64-apple-darwin  
  cargo build \--release \--target aarch64-apple-darwin

- Build for both architectures if you want a universal binary. The output binary (e.g. MyEndpointExt) will be in target/\<arch\>/release/. Place this binary into the extension bundle:

- cp target/x86_64-apple-darwin/release/MyEndpointExt \\  
   SecurityExtensionProj/MySecurityApp.app/Contents/Library/SystemExtensions/MyEndpointExt.systemextension/Contents/MacOS/MyEndpointExt

- Do similarly for the arm64 binary and use lipo to create a universal binary if needed:

- lipo \-create \-output MyEndpointExt.universal \\  
   MyEndpointExt(x86_64) MyEndpointExt(arm64)  
  mv MyEndpointExt.universal MyEndpointExt

- _(If using Swift, you would use xcodebuild to build the app and extension targets. For example: xcodebuild \-scheme MySecurityApp \-configuration Release build from the project directory. The resulting .app in Release-_/ directory would contain the extension.)\*

- **Code Signing:** Sign the extension and app with your certificates and entitlements. You can use codesign via CLI:

- codesign \--force \--timestamp \--sign "Developer ID Application: Your Name (TeamID)" \\  
   \--entitlements endpoint-ext.entitlements \\  
   SecurityExtensionProj/MySecurityApp.app/Contents/Library/SystemExtensions/MyEndpointExt.systemextension/Contents/MacOS/MyEndpointExt  
  codesign \--force \--timestamp \--sign "Developer ID Application: Your Name (TeamID)" \\  
   SecurityExtensionProj/MySecurityApp.app

- Use your Developer ID certificate (for distribution) or Mac Development certificate (for testing locally). Signing the .app may also require an entitlements file if you have any app-specific entitlements. The extension binary **must** be signed with the Endpoint Security and/or Network Extension entitlements embedded[\[1\]](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security#:~:text=Endpoint%20Security%3A).

- **Verify codesign:** Run codesign \-dv \--entitlements :- SecurityExtensionProj/MySecurityApp.app to confirm the signatures and entitlements. Also run codesign \-vv SecurityExtensionProj/MySecurityApp.app to ensure code signature is valid.

## Step 5: Enabling and Testing the Extension Locally

With the app and extension built and signed, you can now load it on your dev machine:

- **Install the App:** Simply copy or drag MySecurityApp.app to /Applications (or any location). Since this is not from the App Store, the first time you run it, macOS will prompt to trust the developer if not already trusted.

- **Activate the System Extension:** There are two ways:

- **Programmatically:** You can run the host app (even if it does nothing visible). The app can use OSSystemExtensionRequest API to request activation of the extension. In our case, since we want CLI, you can trigger it with:

- /Applications/MySecurityApp.app/Contents/MacOS/MySecurityApp &

- If the app calls OSSystemExtensionManager.shared.submitRequest(...) for the extension (matching by bundle identifier), macOS will prompt the user to allow it.

- **Manually via Terminal:** Alternatively, use the systemextensionsctl tool:

- sudo systemextensionsctl install /Applications/MySecurityApp.app

- This will initiate the install of any system extensions in the app. You should then see a prompt to allow it (unless already allowed). After allowing in System Preferences (and possibly entering your credentials), the extension will load.

- **Confirm the Extension is Running:** Use the Terminal command:

- systemextensionsctl list

- You should see an entry like:

- \--- com.apple.system_extension.endpoint_security  
  enabled active teamID bundleID (version) name \[state\]  
  \* \* TEAMID com.example.mysecurityapp.endpointext (1.0/1) MyEndpointExt \[activated\]

- and similarly one for the network extension if installed (with com.apple.system_extension.network_extension)[\[10\]](https://discussions.apple.com/thread/254455105#:~:text=Communities%20discussions.apple.com%20%20,GT8P3H7SPW%20com.mcafee.CMF)[\[22\]](https://developer.apple.com/forums/thread/676423#:~:text=Forums%20developer.apple.com%20%20,0%2F1%29%20myApp). A status of “activated” indicates it’s running.

- **Testing File Monitoring:** Now, generate some events from your sandboxed process. Launch your sandbox root process (the one you want to monitor) and have it attempt file accesses or network connections. You should see your extension’s logs (from println\! or NSLog) in the Console.app or via log stream \--process MyEndpointExt. For example, if your sandbox process opens a file, your extension should print an “Allowed file open: /path/to/file” message, indicating it intercepted and approved the file access.

- **Testing Network Monitoring:** If you implemented the content filter and enabled it (note: content filters also often require user consent in System Preferences \> Network \> Content Filters section), try making an outbound network request (e.g., curl or from your sandbox app). The network filter extension’s logic should trigger. If you allowed all flows by default, the connection will succeed, but you can log or breakpoint in your filter extension to confirm it’s intercepting flows. The systemextensionsctl list will show the content filter as active if properly enabled. You may also check **System Settings \> Network \> Filters** to see if your filter is listed once activated.

- **Debugging Tips:** If the extension isn’t capturing events, ensure:

- The extension has “Full Disk Access” (for file events) and the content filter is enabled (for network).

- The processes you intend to monitor are correctly identified (check your filtering logic).

- Look for any errors in Console (search for your bundle ID or process name). Common issues include missing entitlements or not having approved the extension in Security settings.

- You can attach a debugger to the running extension process using LLDB if needed (lldb \-p \<pid\>).

## Step 6: Packaging and Notarization for Distribution

When your extension is working as expected, you need to prepare it for distribution:

- **Bundle the App:** Create a .dmg or installer containing MySecurityApp.app. The app bundle includes the extension inside, which will be installed on the user’s system when the app runs and requests activation. You can use **productbuild** or **hdiutil** to create an installer or disk image. For example, to create a signed installer:

- productbuild \--component /Applications/MySecurityApp.app /Applications \\  
   \--sign "Developer ID Installer: Your Name (TeamID)" \\  
   MySecurityAppInstaller.pkg

- **Notarize the App:** Apple requires notarization for distributing software with system extensions (outside the App Store) to run on macOS with default security. Use Xcode’s altool or the notarytool to upload for notarization:

- xcrun notarytool submit MySecurityAppInstaller.pkg \--keychain-profile "AC_API" \--wait

- (Set up an API key or use altool with your Apple ID credentials). Wait for notarization success, then staple the ticket:

- xcrun stapler staple MySecurityAppInstaller.pkg

- Now the installer is notarized. When users run it, they will still have to approve the system extension in System Settings, but they won’t get unidentified developer warnings.

- **Ready for Distribution:** Provide documentation to users on how to enable the system extension (as in Step 1 prerequisites, they may need to allow it in Security & Privacy, and on Apple Silicon possibly reduce security). For enterprise deployments, consider using a MDM profile to pre-approve your Team ID’s system extensions to avoid user prompts[\[4\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=While%20System%20Extensions%20enhance%20the,the%20application%20to%20function%20correctly)[\[23\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=Streamlining%20System%20Extension%20Management%20with,Trio%20MDM). Tools like an MDM can whitelist your extension for smoother installation.

By following these steps, you have a **fully functional Endpoint Security extension** that monitors file system and network events for processes in a sandboxed environment, dynamically allowing those actions at runtime. This extension runs in user space (improving safety over legacy kernel extensions) and can be distributed once properly signed and notarized[\[24\]](https://prodisup.com/posts/2022/01/building-and-testing-an-endpoint-security-macos-system-extension-on-bitrise/#:~:text=System%20extension%20style%20application,of%20packaging%20a%20system%20extension)[\[25\]](https://prodisup.com/posts/2022/01/building-and-testing-an-endpoint-security-macos-system-extension-on-bitrise/#:~:text=restricted%20entitlements%20%28ES%2C%20DriverKit%2C%20etc,Store%2C%20for%20better%20or%20worse). Always remember to test thoroughly under real-world conditions and with the proper Apple-provided entitlements to ensure your extension works on user machines with SIP enabled[\[3\]](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security#:~:text=,application%20needs%20not%20only%20the)[\[26\]](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security#:~:text=security.client,can%E2%80%99t%20be%20launched%20on%20user).

**Sources:**

- Apple Developer Documentation – _Endpoint Security and System Extensions_[\[27\]](https://developer.apple.com/videos/play/wwdc2020/10159/#:~:text=There%20are%20two%20categories%20of,advanced%20features%20available%20to%20you)[\[28\]](https://developer.apple.com/videos/play/wwdc2020/10159/#:~:text=last%20year%20in%20macOS%20Catalina,for%20more%20information)

- Apriorit – _Collecting Telemetry Data on macOS using Endpoint Security_[\[1\]](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security#:~:text=Endpoint%20Security%3A)[\[26\]](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security#:~:text=security.client,can%E2%80%99t%20be%20launched%20on%20user)

- Trio Security – _Demystifying macOS System Extensions_ (system extension types and enabling)[\[29\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=comprehensive%20set%20of%20APIs%20to,network%20activities%2C%20and%20kernel%20operations)[\[4\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=While%20System%20Extensions%20enhance%20the,the%20application%20to%20function%20correctly)

- Prodisup Blog – _Building an Endpoint Security system extension_ (notes on entitlements and packaging)[\[2\]](https://prodisup.com/posts/2022/01/building-and-testing-an-endpoint-security-macos-system-extension-on-bitrise/#:~:text=5)[\[8\]](https://prodisup.com/posts/2022/01/building-and-testing-an-endpoint-security-macos-system-extension-on-bitrise/#:~:text=1)

- Endpoint Security Rust Crate Documentation – _endpoint-sec crate usage_[\[30\]](https://docs.rs/endpoint-sec/latest/endpoint_sec/#:~:text=At%20runtime%2C%20users%20should%20call,the%20app%20is%20running%20on)[\[16\]](https://docs.rs/endpoint-sec/latest/endpoint_sec/#:~:text=Client%3A%3Asubscribe%28%29%20,avoid%20stalling%20for%20the%20user)

- Objective-See Blog – _Endpoint Security framework primer_ (AUTH vs NOTIFY events)[\[31\]](https://objective-see.org/blog/blog_0x47.html#:~:text=typedef%20enum%20,ES_EVENT_TYPE_AUTH_RENAME%20%2C%20ES_EVENT_TYPE_AUTH_SIGNAL%20%2C%20ES_EVENT_TYPE_AUTH_UNLINK)[\[32\]](https://objective-see.org/blog/blog_0x47.html#:~:text=Note%20there%20are%20two%20main,and%20%60ES_EVENT_TYPE_NOTIFY)

---

[\[1\]](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security#:~:text=Endpoint%20Security%3A) [\[3\]](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security#:~:text=,application%20needs%20not%20only%20the) [\[17\]](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security#:~:text=Say%20your%20application%20processes%20a,path%20to%20the%20executable%20file) [\[18\]](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security#:~:text=C) [\[26\]](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security#:~:text=security.client,can%E2%80%99t%20be%20launched%20on%20user) Collect Telemetry Data on macOS with Apple’s Endpoint Security \- Apriorit

[https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security](https://www.apriorit.com/dev-blog/collecting-telemetry-data-on-macos-using-endpoint-security)

[\[2\]](https://prodisup.com/posts/2022/01/building-and-testing-an-endpoint-security-macos-system-extension-on-bitrise/#:~:text=5) [\[8\]](https://prodisup.com/posts/2022/01/building-and-testing-an-endpoint-security-macos-system-extension-on-bitrise/#:~:text=1) [\[24\]](https://prodisup.com/posts/2022/01/building-and-testing-an-endpoint-security-macos-system-extension-on-bitrise/#:~:text=System%20extension%20style%20application,of%20packaging%20a%20system%20extension) [\[25\]](https://prodisup.com/posts/2022/01/building-and-testing-an-endpoint-security-macos-system-extension-on-bitrise/#:~:text=restricted%20entitlements%20%28ES%2C%20DriverKit%2C%20etc,Store%2C%20for%20better%20or%20worse) Building and testing an Endpoint Security macOS system extension on Bitrise

[https://prodisup.com/posts/2022/01/building-and-testing-an-endpoint-security-macos-system-extension-on-bitrise/](https://prodisup.com/posts/2022/01/building-and-testing-an-endpoint-security-macos-system-extension-on-bitrise/)

[\[4\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=While%20System%20Extensions%20enhance%20the,the%20application%20to%20function%20correctly) [\[5\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=5,to%20reboot%20your%20Mac) [\[6\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=7,to%20reboot%20your%20Mac) [\[7\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=Extension%20was%20blocked,software%20was%20blocked%20from%20loading) [\[11\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=endpoint%20security%20system%20extensions%3A) [\[19\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=Network%20Extensions%20empower%20developers%20to,encompasses%20several%20extension%20types%2C%20including) [\[20\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=Filter%20Data%3A%20Facilitates%20the%20filtering,data%20at%20the%20flow%20level) [\[23\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=Streamlining%20System%20Extension%20Management%20with,Trio%20MDM) [\[29\]](https://www.trio.so/blog/macos-system-extensions/#:~:text=comprehensive%20set%20of%20APIs%20to,network%20activities%2C%20and%20kernel%20operations) Demystifying macOS System Extensions: A Comprehensive Guide

[https://www.trio.so/blog/macos-system-extensions/](https://www.trio.so/blog/macos-system-extensions/)

[\[9\]](https://github.com/redcanaryco/mac-monitor#:~:text=,com.redcanary.agent.securityextension.systemextension) GitHub \- redcanaryco/mac-monitor: Red Canary Mac Monitor is an advanced, stand-alone system monitoring tool tailor-made for macOS security research. Beginning with Endpoint Security (ES), it collects and enriches system events, displaying them graphically, with an expansive feature set designed to reduce noise.

[https://github.com/redcanaryco/mac-monitor](https://github.com/redcanaryco/mac-monitor)

[\[10\]](https://discussions.apple.com/thread/254455105#:~:text=Communities%20discussions.apple.com%20%20,GT8P3H7SPW%20com.mcafee.CMF) How to disable System Extension? \- Apple Support Communities

[https://discussions.apple.com/thread/254455105](https://discussions.apple.com/thread/254455105)

[\[12\]](https://objective-see.org/blog/blog_0x47.html#:~:text=%2F%2F%2FThe%20caller%20is%20not%20properly,entitled%20to%20connect%20ES_NEW_CLIENT_RESULT_ERR_NOT_ENTITLED) [\[13\]](https://objective-see.org/blog/blog_0x47.html#:~:text=Hopefully%20these%20are%20rather%20self,security.client%60%20entitlement) [\[31\]](https://objective-see.org/blog/blog_0x47.html#:~:text=typedef%20enum%20,ES_EVENT_TYPE_AUTH_RENAME%20%2C%20ES_EVENT_TYPE_AUTH_SIGNAL%20%2C%20ES_EVENT_TYPE_AUTH_UNLINK) [\[32\]](https://objective-see.org/blog/blog_0x47.html#:~:text=Note%20there%20are%20two%20main,and%20%60ES_EVENT_TYPE_NOTIFY) Objective-See's Blog

[https://objective-see.org/blog/blog_0x47.html](https://objective-see.org/blog/blog_0x47.html)

[\[14\]](https://docs.rs/endpoint-sec/latest/endpoint_sec/#:~:text=Available%20on%20macOS%20only) [\[16\]](https://docs.rs/endpoint-sec/latest/endpoint_sec/#:~:text=Client%3A%3Asubscribe%28%29%20,avoid%20stalling%20for%20the%20user) [\[30\]](https://docs.rs/endpoint-sec/latest/endpoint_sec/#:~:text=At%20runtime%2C%20users%20should%20call,the%20app%20is%20running%20on) endpoint_sec \- Rust

[https://docs.rs/endpoint-sec/latest/endpoint_sec/](https://docs.rs/endpoint-sec/latest/endpoint_sec/)

[\[15\]](https://speakerdeck.com/patrickwardle/mastering-apples-endpoint-security-for-advanced-macos-malware-detection#:~:text=,which%20takes%20a%20flag) Mastering Apple's Endpoint Security for Advanced macOS Malware ...

[https://speakerdeck.com/patrickwardle/mastering-apples-endpoint-security-for-advanced-macos-malware-detection](https://speakerdeck.com/patrickwardle/mastering-apples-endpoint-security-for-advanced-macos-malware-detection)

[\[21\]](https://developer.apple.com/documentation/endpointsecurity/es_event_type_auth_uipc_connect#:~:text=ES_EVENT_TYPE_AUTH_UIPC_CONNECT,Mac%20Catalyst) ES_EVENT_TYPE_AUTH_UIPC...

[https://developer.apple.com/documentation/endpointsecurity/es_event_type_auth_uipc_connect](https://developer.apple.com/documentation/endpointsecurity/es_event_type_auth_uipc_connect)

[\[22\]](https://developer.apple.com/forums/thread/676423#:~:text=Forums%20developer.apple.com%20%20,0%2F1%29%20myApp) NSConnection between Endpoint secu… | Apple Developer Forums

[https://developer.apple.com/forums/thread/676423](https://developer.apple.com/forums/thread/676423)

[\[27\]](https://developer.apple.com/videos/play/wwdc2020/10159/#:~:text=There%20are%20two%20categories%20of,advanced%20features%20available%20to%20you) [\[28\]](https://developer.apple.com/videos/play/wwdc2020/10159/#:~:text=last%20year%20in%20macOS%20Catalina,for%20more%20information) Build an Endpoint Security app \- WWDC20 \- Videos \- Apple Developer

<https://developer.apple.com/videos/play/wwdc2020/10159/>
