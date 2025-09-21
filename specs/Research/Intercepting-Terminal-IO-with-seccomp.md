# Intercepting TTY Output for Audit Logging on macOS and Linux

## Overview of the Requirement

You want to **intercept writes to a TTY** (terminal) in child processes (and their descendants) such that whenever a **new line** ('\\n') is output, an **audit logging action** is performed **synchronously**. In other words, the writing thread should be **blocked** until your logging operation has completed and synced to disk. You also want to avoid the high overhead of ptrace and use a more efficient mechanism on both macOS and Linux.

Achieving this **synchronous interception** requires hooking into the low-level write operation. Below we outline approaches for macOS and Linux that satisfy these requirements, focusing on **EndpointSecurity** on macOS and **seccomp user-space notifications** on Linux, as well as alternative methods where applicable.

## macOS Solutions

### 1\. EndpointSecurity Framework (System Extension)

Apple’s **Endpoint Security (ES) framework** allows user-space system extensions to subscribe to security-related events (process executions, file access, etc.)[\[1\]](https://www.withsecure.com/en/expertise/resources/macos-endpoint-security-framework#:~:text=In%20the%20new%20and%20improved,result%20of%20Apple%20deprecating%20KEXT). You can use ES to monitor **file write events** system-wide, including writes to device files like TTYs. In particular, ES provides a ES_EVENT_TYPE_NOTIFY_WRITE event type for **notifications when a process writes to a file**[\[2\]](https://objective-see.org/blog/blog_0x48.html#:~:text=match%20at%20L366%20,%E2%80%9D). Your ES client would receive an event with details such as the file path (e.g. /dev/ttys002) and the process ID whenever data is written[\[3\]](https://objective-see.org/blog/blog_0x48.html#:~:text=match%20at%20L667%20FILE%20WRITE,null%29%20destination%20path%3A%20%2Fprivate%2Ftmp%2Ftest).

However, **EndpointSecurity has a crucial limitation**: write events are **notify-only**, _not_ authorization events. This means you **cannot block or delay the write** via ES; the event is delivered _after_ the write has occurred[\[4\]](https://stackoverflow.com/questions/75859573/intercepting-filesystem-calls-of-other-processes-on-macos-ventura#:~:text=The%20Endpoint%20Security%20framework%20you,es_respond_auth_result%20or%20es_respond_flags_result%20as%20appropriate). Apple does not provide an AUTH_WRITE event (no ES_EVENT_TYPE_AUTH_WRITE exists), so you **cannot use ES to hold the thread** until your logging is done. In other words, ES can **inform** you of each write (including the data size and file path), but it can't natively pause the writing thread on a per-write basis.

**Implication:** You _can_ use EndpointSecurity to **monitor and log** TTY output events on macOS, but **not to synchronously block the writer**. If synchronous blocking is absolutely required, ES alone won't suffice. You could still perform the audit log in your ES event handler, but the process will have continued executing in the meantime.

### 2\. Kernel Extension or Driver-Level Hook (Advanced/Not Recommended)

For truly blocking interception at the moment of each write on macOS, you would need to hook the kernel’s write path to the TTY device. Historically, one could write a **Kernel Extension (KEXT)** using the Kernel Authorization (Kauth) API or similar to intercept file operations (e.g., KAUTH_FILEOP_WRITE events). This could allow you to veto or pause a write operation. **However, KEXTs are deprecated on modern macOS** and require disabling SIP (System Integrity Protection) for third-party loading[\[5\]](https://www.withsecure.com/en/expertise/resources/macos-endpoint-security-framework#:~:text=Note%3A%20Although%20kernel%20extensions%20have,security%20of%20their%20customers%E2%80%99%20systems). Apple strongly discourages this approach for production systems.

Another low-level approach would be implementing an **I/O Kit filter** for the TTY device. This means writing a driver that attaches to the terminal device node (e.g., the pseudoterminal driver) and intercepts data as it’s written. This is extremely complex and also effectively a form of kernel extension (requiring special entitlements or SIP disabled). Given Apple’s deprecation of KEXTs, this is typically not practical.

In summary, while it’s _theoretically_ possible to achieve synchronous blocking via a custom kernel component on macOS, there is **no high-level public API** that supports this. The EndpointSecurity system extension is the Apple-endorsed method for monitoring, but it only offers post-hoc notification for writes, not pre-write interception[\[4\]](https://stackoverflow.com/questions/75859573/intercepting-filesystem-calls-of-other-processes-on-macos-ventura#:~:text=The%20Endpoint%20Security%20framework%20you,es_respond_auth_result%20or%20es_respond_flags_result%20as%20appropriate).

### 3\. **Pseudo-Terminal (PTY) Proxying** (User-Space Alternative)

If you **control how the child processes are launched**, a user-space workaround on both macOS and Linux is to run the processes attached to a **pseudo-terminal** (**pty**) that you control. For example:

- **Allocate a pty** (using openpty() or similar).

- **Launch the child process** with its STDOUT/STDERR connected to the **slave** end of the pty (which appears as a TTY to the process).

- Your monitoring program reads from the **master** end of the pty, which receives everything the child writes. You can detect '\\n' in this output stream and perform your audit logging.

- Only after logging, write the data to the real terminal (or wherever it should go) from the master end.

By doing this, you effectively place a **buffer between the child and the real TTY**. The child’s write(tty, data) calls will actually write into the pty buffer. If you **do not read** from the master immediately, the child can **block** (the kernel pty buffer has limited size), achieving back-pressure. For each newline, you can purposely halt reading (which will block the child if it writes more data) until your audit log is flushed, then resume reading/writing. This ensures the child’s thread is indirectly **blocked until logging is done**.

This PTY proxy method does not require special kernel APIs – it leverages normal terminal I/O behavior. The downside is that you must **start the processes yourself** under this pty environment (and handle their input/output), but it works on both macOS and Linux. It also has relatively **low performance overhead**, as it uses the OS’s efficient pty mechanism rather than trapping every syscall in the kernel.

## Linux Solutions

On Linux, you have more straightforward APIs to intercept syscalls synchronously without using ptrace:

### 1\. Seccomp User-Space Notifications (Seccomp Unotify)

Linux’s **seccomp** (secure computing) subsystem can be used not only to filter/deny syscalls but also to **notify a monitoring process** and **pause the target** while the monitor handles the event. This is done via **seccomp user-space notifications** (also called seccomp_unotify). It’s an excellent alternative to ptrace for this use-case, offering **lower overhead** by filtering in-kernel and only notifying user-space for targeted syscalls[\[6\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=Why%20Use%20,ptrace).

How this works for your scenario:

- You would install a seccomp filter in the child processes (and have it inherit to all descendants) that turns specific syscalls **into notifications**. In your case, you can filter the write (and writev, etc.) syscalls. The filter can be made conditional – for example, only trigger when writing to a TTY fd (you might inspect the file descriptor number or use a lightweight check in the monitor to decide).

- When a child process calls write(), the kernel **pauses the thread** and sends a message over a special file descriptor to your monitoring process/thread.

- The monitor (in user space) receives the notification, which includes details of the syscall **number and arguments** (e.g., file descriptor, buffer pointer, length)[\[7\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=rule%20,intercepted%20syscalls%20in%20user%20space)[\[8\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=go%20func%28req%20,0%2C%20Flags%3A%200). (The content of the buffer isn't copied in the message, but the monitor can **read the target’s memory** if needed, since it has the PID.)

- Your monitor can then perform the **audit logging**. For example, if the buffer contains a newline (you may need to peek at the buffer contents via /proc/\<pid\>/mem or similar), write the relevant log to disk and flush it.

- Once done, the monitor uses seccomp_notifRespond() (via libseccomp or direct ioctl) to **resume the syscall**. You have two choices:

- **Continue the original write:** Use the flag SECCOMP_USER_NOTIF_FLAG_CONTINUE, which tells the kernel “proceed with the syscall as normal now”[\[9\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=We%20define%20a%20handler%20for,and%20allowing%20it%20to%20continue). This unblocks the child’s thread and lets the write actually happen in the kernel.

- **Emulate/modify the write:** Alternatively, the monitor could itself write the data (or filtered data) to the TTY and then respond with a result indicating the bytes written. For simple logging use-case, you likely just want to continue the original write after logging.

Because the thread is truly paused by the kernel until you respond, this meets the requirement that **the intercepted program’s thread is blocked until logging is complete**. And unlike ptrace, which traps _every_ syscall with heavy context switches, seccomp notifiers let you intercept only the targeted syscalls with much less overhead[\[6\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=Why%20Use%20,ptrace).

**Notes & Implementation Details:**

- **Performance:** Seccomp with user notifications is quite efficient for moderate rates of syscalls. It’s designed for scenarios like sandbox monitors and container runtimes. There is still overhead (a context switch to the monitor for each intercepted write), but it's significantly lower than full ptrace trapping of all syscalls[\[6\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=Why%20Use%20,ptrace).

- **Filtering to TTY only:** One challenge is determining if a given write(fd, buf, len) is writing to a TTY. In the seccomp filter BPF, you might not easily know the target of the FD. One approach is to **intercept all writes** from the child, then in the monitor, check the file descriptor: you can call isatty(fd) on the **target process’s fd** or inspect /proc/\<pid\>/fd/\<N\> to see if it points to /dev/pts/X or /dev/tty. If it’s not a TTY, you can immediately allow it to continue without delay. This way, only TTY writes incur the logging delay.

- **Propagation to children:** If you set up seccomp in the initial parent (or use prctl(PR_SET_NO_NEW_PRIVS) and attach the filter before exec), the filter will **inherit across fork/exec** into all descendant processes. This covers the “transitive children” automatically, as long as they don’t explicitly remove seccomp (which they normally cannot, since seccomp filters are strictly reducing privileges).

- **Coding:** You can use **libseccomp** (in C or even Go, as shown in some references) to simplify setting up the filter and handling the notification FD. There are examples of intercepting syscalls like connect using seccomp notify[\[10\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=rule%20,intercepted%20syscalls%20in%20user%20space); your case would be similar but for write. After installing the filter (with an action SCMP_ACT_NOTIFY on SYS_write), you'll get a file descriptor from seccomp(2) which you poll for events. Reading from it gives you a seccomp_notif structure with the syscall details, and you respond with a seccomp_notif_resp to continue.

### 2\. Other Linux Options

- **LD_PRELOAD / Function Wrapping:** In user space, you could inject a library into the child processes that overrides the write() C library call (or fprintf, etc.) to intercept output. On Linux, setting the LD_PRELOAD environment variable for the children could load your interceptor. However, this is less robust (it can be bypassed by static binaries or if the program calls syscall directly) and requires managing the injection for every child. It also won’t catch writes from non-stdlib sources (e.g., if the program uses low-level syscalls). Given that you specifically need to capture all transitive children without knowing them in advance, LD_PRELOAD is fragile compared to a seccomp or kernel-level solution.

- **ptrace (for comparison):** Using ptrace to catch write syscalls (via PTRACE*SYSCALL and examining registers) \_would* work and allow pausing, but as you noted, it carries significant performance overhead. Every syscall (or at least every write syscall) causes a context switch into the tracer and back, and ptrace must single-step or continue the child each time. Seccomp user notifications achieve a similar result more efficiently by handling the filtering in-kernel and waking the monitor only when needed[\[6\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=Why%20Use%20,ptrace).

- **eBPF / KProbes:** eBPF programs could be attached to the sys*enter_write tracepoint or a kprobe on the ksys_write kernel function to observe write calls. This would let you \_monitor* writes with very low overhead. However, eBPF cannot easily **pause and wait** for user-space to do work. An eBPF program must complete quickly and cannot sleep or yield for a user-space response. You could use eBPF to log events (or send events to user-space), but not to block the syscall until some external action completes. (There is experimental work with eBPF and ring buffers for user-space communication, but you’d essentially end up re-implementing a notification system – which is exactly what seccomp provides.) For your needs, seccomp’s built-in blocking mechanism is more appropriate.

- **Linux Audit Framework:** The kernel’s audit subsystem (auditd, audit rules) can log syscalls and TTY input/output events (with TTY input logging via PAM, etc.), but it is not designed to intercept and block calls in real time. Audit will record that something happened (e.g., a write to /dev/pts/3), but you cannot intervene in the syscall’s execution or pause it. Thus, auditd doesn’t satisfy the “block until logged” requirement.

## Summary and Trade-offs

**On macOS:** You can use the EndpointSecurity API to _monitor_ all writes to TTY devices and perform logging (Apple supports this in user-land with system extensions[\[1\]](https://www.withsecure.com/en/expertise/resources/macos-endpoint-security-framework#:~:text=In%20the%20new%20and%20improved,result%20of%20Apple%20deprecating%20KEXT)). But you **cannot directly block the writer** via ES because write events are notify-only[\[4\]](https://stackoverflow.com/questions/75859573/intercepting-filesystem-calls-of-other-processes-on-macos-ventura#:~:text=The%20Endpoint%20Security%20framework%20you,es_respond_auth_result%20or%20es_respond_flags_result%20as%20appropriate). Truly pausing a thread’s output on macOS would require a non-trivial kernel-level hack (not generally advisable). If synchronous logging is critical, a more practical approach is to **architect the solution in user-space** using a **PTY proxy**, so that you control when data is read (thus indirectly blocking the child). This requires launching the process yourself, but it avoids needing unsupported kernel tricks and works under normal macOS security constraints.

**On Linux:** The **seccomp user-space notification** mechanism is a powerful fit for this problem. It allows you to designate syscalls (like write) to be trapped, _pause_ the calling thread, and notify your monitor process, which can then log and resume the syscall. This meets the strict requirement of blocking the thread until the audit log is synced, with much lower overhead than ptrace[\[6\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=Why%20Use%20,ptrace). The complexity is moderate (you’ll be writing a monitor loop to handle notifications), but it is a officially supported kernel feature (available since Linux 5.0+). Alternatively, if you prefer not to dip into seccomp, you can use the same **PTY wrapper technique** on Linux (e.g., using forkpty() or openpty \+ execv) to capture output, though that is more of an architectural workaround than a system-call interception.

In conclusion, **yes, it is possible to satisfy the stricter requirement on both macOS and Linux**, but the approach differs:

- **macOS:** Use the EndpointSecurity API for monitoring (with asynchronous notifications)[\[2\]](https://objective-see.org/blog/blog_0x48.html#:~:text=match%20at%20L366%20,%E2%80%9D), or leverage a pseudo-terminal to enforce synchronous behavior in user-space.

- **Linux:** Use seccomp user-space notifications to intercept write syscalls and block until a logging action completes[\[9\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=We%20define%20a%20handler%20for,and%20allowing%20it%20to%20continue), for a solution that is both synchronous and relatively low-overhead compared to ptrace.

Each approach comes with implementation complexity, but they are the intended modern ways to monitor and control process I/O on their respective platforms. Make sure to consider the performance impact of logging every line and the potential volume of events (especially if a program writes many small chunks) – you'll want to minimize what you intercept (filter only TTY writes, for example) to avoid bottlenecks.

**Sources:**

- Apple Developer Documentation – _Endpoint Security framework_ (monitoring and authorization of system events)[\[4\]](https://stackoverflow.com/questions/75859573/intercepting-filesystem-calls-of-other-processes-on-macos-ventura#:~:text=The%20Endpoint%20Security%20framework%20you,es_respond_auth_result%20or%20es_respond_flags_result%20as%20appropriate)[\[2\]](https://objective-see.org/blog/blog_0x48.html#:~:text=match%20at%20L366%20,%E2%80%9D)

- Objective-See Blog – _Writing a File Monitor with Endpoint Security_ (example of catching file write events on macOS)[\[3\]](https://objective-see.org/blog/blog_0x48.html#:~:text=match%20at%20L667%20FILE%20WRITE,null%29%20destination%20path%3A%20%2Fprivate%2Ftmp%2Ftest)

- Linux man pages / Medium article – _Seccomp user notifications vs ptrace_ (illustrating lower overhead and usage of seccomp notify to intercept syscalls)[\[6\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=Why%20Use%20,ptrace)[\[9\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=We%20define%20a%20handler%20for,and%20allowing%20it%20to%20continue)

---

[\[1\]](https://www.withsecure.com/en/expertise/resources/macos-endpoint-security-framework#:~:text=In%20the%20new%20and%20improved,result%20of%20Apple%20deprecating%20KEXT) [\[5\]](https://www.withsecure.com/en/expertise/resources/macos-endpoint-security-framework#:~:text=Note%3A%20Although%20kernel%20extensions%20have,security%20of%20their%20customers%E2%80%99%20systems) MacOS Endpoint Security Framework (ESF) | WithSecure™

[https://www.withsecure.com/en/expertise/resources/macos-endpoint-security-framework](https://www.withsecure.com/en/expertise/resources/macos-endpoint-security-framework)

[\[2\]](https://objective-see.org/blog/blog_0x48.html#:~:text=match%20at%20L366%20,%E2%80%9D) [\[3\]](https://objective-see.org/blog/blog_0x48.html#:~:text=match%20at%20L667%20FILE%20WRITE,null%29%20destination%20path%3A%20%2Fprivate%2Ftmp%2Ftest) Objective-See's Blog

[https://objective-see.org/blog/blog_0x48.html](https://objective-see.org/blog/blog_0x48.html)

[\[4\]](https://stackoverflow.com/questions/75859573/intercepting-filesystem-calls-of-other-processes-on-macos-ventura#:~:text=The%20Endpoint%20Security%20framework%20you,es_respond_auth_result%20or%20es_respond_flags_result%20as%20appropriate) security \- Intercepting filesystem calls of other processes on MacOS Ventura \- Stack Overflow

[https://stackoverflow.com/questions/75859573/intercepting-filesystem-calls-of-other-processes-on-macos-ventura](https://stackoverflow.com/questions/75859573/intercepting-filesystem-calls-of-other-processes-on-macos-ventura)

[\[6\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=Why%20Use%20,ptrace) [\[7\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=rule%20,intercepted%20syscalls%20in%20user%20space) [\[8\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=go%20func%28req%20,0%2C%20Flags%3A%200) [\[9\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=We%20define%20a%20handler%20for,and%20allowing%20it%20to%20continue) [\[10\]](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960#:~:text=rule%20,intercepted%20syscalls%20in%20user%20space) Using Seccomp User Notifications (seccomp_unotify) as an Alternative to Ptrace for Syscall Interception in Golang | by Robert Mindo | Medium

[https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960](https://medium.com/@mindo.robert1/using-seccomp-user-notifications-seccomp-unotify-as-an-alternative-to-ptrace-for-syscall-1c806a3e2960)
