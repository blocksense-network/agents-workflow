## Overview

In a sandboxed execution environment, a **supervisor process** monitors file
system operations of an untrusted "guest" process (the sandboxed code). The
supervisor enforces a dynamic allow-list of file paths. Initially, the guest
can only access a minimal set of safe files. As the guest tries to open new
files, the supervisor intercepts those attempts and decides whether to allow
them. Some file accesses are **auto-approved** based on predefined rules, while
others pause the guest and require **human approval** before proceeding. This
ensures tight security while still granting the guest any necessary file access
on a case-by-case basis.

Below is a detailed step-by-step guide, with pseudo-code style explanations,
for setting up such a supervisor process. We focus on low-level file-access
interception and dynamic policy updates.

## Step-by-Step Implementation Outline

1. **Launch a Supervisor for the Session:**
   For each sandbox session, start a dedicated supervisor process. This
   supervisor will oversee one guest process. (While a single persistent
   supervisor could handle multiple sessions, using one supervisor per guest is
   simpler to implement and isolate.)

2. **Initialize Allowed File List:**
   Define an initial allow-list of file paths the guest can access from start.
   This might include essential libraries or config files required for basic
   operation (or even be empty if the guest should start with no file access). For
   example:

- allowed_paths \= {
  "/usr/lib/essential_lib.so",
  "/tmp/guest_scratch/"
  }

- This set will expand dynamically. The allow-list (policy store) can be
  in-memory for the session, since policy persistence is out of scope here.

3. **Set Up File-Access Interception Mechanism:**
   Before launching the guest, the supervisor must **hook into file system
   calls** made by that guest. There are a few ways to achieve this:

4. **Seccomp User-Space Notification:** The guest process can be started with a
   seccomp filter that sends a notification to the supervisor on each file-access
   syscall (e.g. an open() or openat() system call). Using SECCOMP_RET_USER_NOTIF,
   the kernel will pause the guest’s syscall and notify the supervisor,
   effectively delegating the decision to the
   supervisor[\[1\]](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html#:~:text=Overview%20In%20conventional%20usage%20of,implementing%20security%20policy%3B%20see%20NOTES)[\[2\]](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html#:~:text=the%20target%20is%20temporarily%20blocked,on%20the%20listening%20file%20descriptor).

5. **Fanotify with Permission Events:** The supervisor can use the Linux
   fanotify API in permission mode to listen for file open attempts by the guest.
   Fanotify will block the guest’s file access operation and wait until the
   supervisor approves or denies
   it[\[3\]](https://unix.stackexchange.com/questions/390655/can-fanotify-modify-files-before-access-by-other-applications#:~:text=When%20you%27re%20getting%20permission%20events,that%20you%20may%20have%20modified).

6. **Ptrace/System Call Interception:** The supervisor can act as a debugger on
   the guest process using ptrace to catch open system calls and suspend the guest
   at those calls.

**Low-Level Detail:** Regardless of method, the goal is that whenever the guest
attempts to open a file, the supervisor is notified _before_ the file is
actually accessed, and the guest is blocked (sleeping in kernel space) until a
decision is
made[\[2\]](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html#:~:text=the%20target%20is%20temporarily%20blocked,on%20the%20listening%20file%20descriptor)[\[3\]](https://unix.stackexchange.com/questions/390655/can-fanotify-modify-files-before-access-by-other-applications#:~:text=When%20you%27re%20getting%20permission%20events,that%20you%20may%20have%20modified). For example, with
seccomp user notifications, the guest's thread is paused by the kernel and an
event is queued for the supervisor, containing the syscall number and arguments
(including the file
path)[\[4\]](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html#:~:text=notification%20event%20is%20generated%20on,the%20listening%20file%20descriptor)[\[5\]](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html#:~:text=,in%20subsequent%20SECCOMP_IOCTL_NOTIF_ID_VALID%20and%20SECCOMP_IOCTL_NOTIF_SEND).

1. **Launch the Guest Process in Restricted Mode:**
   Start the sandboxed guest process (e.g., via fork()/exec() or container
   runtime), applying the interception mechanism from step 3\. For instance, if
   using seccomp: install the seccomp filter in the guest process before it begins
   executing untrusted code. If using fanotify or ptrace: launch the process and
   immediately attach the supervisor to monitor it. The guest should run with
   minimal privileges (possibly as a non-root user, in a chroot or mount namespace
   with limited filesystem view, etc.), to limit damage if it tries something
   unexpected. The key is that any file open will trigger our intercept.

2. **Monitor File Access Events:**
   Now the supervisor enters a loop waiting for file-access events from the
   guest. In pseudo-code:

- loop while guest_process is running:
  event \= wait_for_file_open_event() // Blocks until guest tries to open a
  file
  filepath \= event.requested_path // Get the file path the guest wants
  ...

3. For seccomp: use ioctl(SECCOMP_IOCTL_NOTIF_RECV) on the seccomp listener
   file descriptor to get the next syscall
   event[\[6\]](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html#:~:text=,NOTES%2C%20the%20file%20descriptor%20can) (this returns info including the
   syscall type and its arguments, like the path for open).

4. For fanotify: call read() on the fanotify file descriptor to receive an
   event structure when the guest attempts a file open. The event includes the
   file descriptor or inode and the operation (open, etc.), which can be converted
   to a path.

5. For ptrace: wait for a syscall-enter stop, check if it’s an open/openat
   call, then retrieve the path argument from the guest’s registers/memory.

6. **Check Against Allow-List:**
   When an event arrives, the supervisor checks if the requested file path is
   already in the allowed_paths set.

7. **If the path is in the allow-list:** Approve the access immediately. The
   supervisor instructs the kernel to continue the syscall normally.
   - For seccomp: write a response with SECCOMP_IOCTL_NOTIF_SEND indicating the
     syscall should be allowed (and optionally specify a return value if we were
     emulating, but here just allow
     it)[\[2\]](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html#:~:text=the%20target%20is%20temporarily%20blocked,on%20the%20listening%20file%20descriptor).

   - For fanotify: write an approval for the event (a fanotify response struct
     with FAN_ALLOW). The guest will then be unblocked and proceed to open the file.

   - For ptrace: simply let the syscall continue (e.g., using
     ptrace(PTRACE_SYSCALL, ...) again to resume the guest).

8. **If the path is not in the allow-list:** This triggers a policy decision
   point. The supervisor must decide whether to allow this new file access and
   possibly update the allow-list. We handle this in the next steps.

9. **Auto-Approval for Certain Paths:**
   The supervisor can automatically approve some categories of file access
   without human involvement, according to predefined rules. For example, one rule
   might be: allow read-only access to any file within a specific directory (say
   the guest’s own working directory or a temp folder), or allow creation of new
   files in /tmp/guest_scratch/. If the requested path qualifies for auto-approval:

10. **Add** the path (or a broader pattern, e.g. the directory) to
    allowed_paths. This expands the guest’s permissions dynamically.

11. **Approve** the file access event (same mechanism as above, e.g. send an
    "allow" response to the kernel).

12. Log this expansion for auditing (optional but good practice).

For example:

if is_auto_allowed(filepath):
allowed_paths.add(filepath)
allow_event(event) // resume guest, letting it access the file
continue loop

If the auto-approval covers an entire directory, the supervisor might add a
pattern or all files under that directory to the allow-list in one go.

1. **Manual Approval for Other Paths:**
   If the file path doesn’t fall under an auto-rule, the supervisor must
   involve a human decision (e.g., an administrator or the end-user) to approve or
   deny this access:

2. **Pause the guest:** The guest process is already blocked waiting for a
   decision (by the kernel/supervisor mechanism). Ensure it remains paused. (With
   ptrace or seccomp, it's automatically paused; with fanotify, it’s paused
   until
   reply[\[3\]](https://unix.stackexchange.com/questions/390655/can-fanotify-modify-files-before-access-by-other-applications#:~:text=When%20you%27re%20getting%20permission%20events,that%20you%20may%20have%20modified).)

3. **Notify a human operator:** This could be done by logging a message,
   sending an alert, or showing a prompt in a UI. Include details like which file
   is being accessed and by which session/process. For example:
   _Alert:_ Guest process PID 1234 requests access to /home/user/secret.doc.
   Approve? (y/N)

4. **Wait for response:** The supervisor process should wait (perhaps blocking
   on a socket, GUI input, or reading from a control pipe) for the human’s
   decision. The guest remains suspended during this time.

5. **Human approves:** If the answer is to allow, add this path to the
   allowed_paths set (so that future accesses to the same file won’t prompt
   again). Then send an approval to the kernel to unblock the file access. The
   guest will continue and successfully open the file.

6. **Human denies:** If the access is disallowed, instruct the kernel to **deny
   the event**. There are a few ways to deny:
   - For seccomp, respond with an error code (e.g., set resp.val to \-EPERM) so
     the guest’s open syscall returns a permission error.

   - For fanotify, send a FAN_DENY response for that event, similarly causing
     an EACCES in the guest.

   - For ptrace, one can set the guest’s registers to indicate an error
     return and then resume, or simply kill the guest process if the policy is to
     terminate on a serious violation.
     After denying, the guest’s open call will fail as if the OS refused
     permission. The supervisor can decide whether to let the guest continue running
     (despite the failed syscall) or terminate the guest if the operation was
     critical or suspicious.

The pseudo-code for this decision might look like:

else:
pause_guest(event) // ensure guest is paused (if not already)
decision \= get_human_approval(filepath)
if decision \== "approve":
allowed_paths.add(filepath)
allow_event(event) // resume guest with success
else:
deny_event(event) // e.g., make guest's syscall return an error
// (optionally terminate guest if policy dictates)

1. **Resume Monitoring:**
   After handling the event (whether auto or manual), the supervisor returns to
   waiting for the next file access from the guest. The loop continues until the
   guest process exits or is terminated.

2. **Cleanup:**
   Once the guest finishes execution, perform cleanup in the supervisor:
   - Close any monitoring file descriptors (seccomp listener, fanotify FD,
     etc.).

   - Possibly log the session’s allowed file list for auditing (to see what
     was accessed).

   - Terminate the supervisor process if it was one-per-session. (If a
     persistent supervisor handled multiple guests, it would now just free resources
     associated with that particular guest and be ready for the next one.)

Throughout this process, the allow-list is dynamically expanded to include any
new files that have been deemed safe for the guest. The separation of concerns
is clear: the guest doesn’t know about any of this supervision (it simply
experiences “permission denied” errors or successful opens), and the policy
logic – what is allowed automatically vs. what requires a person – resides
entirely in the supervisor.

## Additional Considerations

- **Performance:** Intercepting every file open has some overhead. Seccomp and
  fanotify are fairly efficient in the kernel, but the round-trip to user-space
  for a decision will slow down access, especially when waiting for human input.
  Consider caching decisions or batching requests if performance is critical.

- **Security:** The supervisor should be the only process with the power to
  approve file accesses. It must run with privileges to intercept syscalls and to
  allow/deny them. The guest should run with minimal privileges. Even the
  supervisor should be careful – for example, if using seccomp user
  notifications, the kernel documentation notes it’s not meant to _enforce_
  security
  alone[\[7\]](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html#:~:text=to%20treat%20a%20system%20call,implementing%20security%20policy%3B%20see%20NOTES), so the supervisor should be robust against malicious guest behavior (e.g.
  handle path traversal carefully, and not inadvertently allow more than
  intended).

- **Extensibility:** While we focused on file _open_ calls, a real
  implementation might also intercept other related syscalls (like mkdir, unlink,
  etc., if you want to supervise creation or deletion of files) and apply similar
  allow-list logic to them.

- **Single Supervisor for All Sessions:** In our outline we used one supervisor
  per guest for simplicity. If a single persistent supervisor process is used for
  all sessions, it would need to track allowed file sets per guest and include an
  identifier in each event (e.g., seccomp and fanotify events would include a
  process ID or handle to know which sandbox triggered it). A dispatcher would
  then route each event to the correct policy logic and user prompt. This is more
  complex but could be more resource-efficient for a large number of simultaneous
  guests.

By following these steps, you set up a supervisor that robustly monitors and
controls file system access for sandboxed processes, dynamically adjusting
permissions as needed with a mix of automatic rules and human oversight. This
design ensures the guest only accesses what it truly needs, minimizing risk to
the host system.

---

[\[1\]](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html#:~:text=Overview%20In%20conventional%20usage%20of,implementing%20security%20policy%3B%20see%20NOTES)
[\[2\]](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html#:~:text=the%20target%20is%20temporarily%20blocked,on%20the%20listening%20file%20descriptor)

[\[4\]](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html#:~:text=notification%20event%20is%20generated%20on,the%20listening%20file%20descriptor)
[\[5\]](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html#:~:text=,in%20subsequent%20SECCOMP_IOCTL_NOTIF_ID_VALID%20and%20SECCOMP_IOCTL_NOTIF_SEND)
[\[6\]](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html#:~:text=,NOTES%2C%20the%20file%20descriptor%20can)
[\[7\]](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html#:~:text=to%20treat%20a%20system%20call,implementing%20security%20policy%3B%20see%20NOTES)
seccomp_unotify(2) \- Linux manual page

[https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html](https://man7.org/linux/man-pages/man2/seccomp_unotify.2.html)

[\[3\]](https://unix.stackexchange.com/questions/390655/can-fanotify-modify-files-before-access-by-other-applications#:~:text=When%20you%27re%20getting%20permission%20events,that%20you%20may%20have%20modified) linux \- Can fanotify modify
files before access by other applications? \- Unix & Linux Stack Exchange

[https://unix.stackexchange.com/questions/390655/can-fanotify-modify-files-before-access-by-other-applications](https://unix.stackexchange.com/questions/390655/can-fanotify-modify-files-before-access-by-other-applications)
