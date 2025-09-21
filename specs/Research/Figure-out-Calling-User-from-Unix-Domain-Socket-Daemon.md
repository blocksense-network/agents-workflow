With Unix domain sockets the server can query the connecting peer’s credentials (UID/GID—and often PID), so you can identify the user.

Here’s how on common platforms:

- **Linux (STREAM/SEQPACKET):**
  Call `getsockopt(fd, SOL_SOCKET, SO_PEERCRED, …)` on the accepted socket. It returns a `struct ucred { pid_t pid; uid_t uid; gid_t gid; }` for the peer (credentials are from the time of `connect(2)`).

- **Linux (DATAGRAM):**
  Enable per-message creds with `setsockopt(fd, SOL_SOCKET, SO_PASSCRED, &one, sizeof(one))`, then read ancillary data `SCM_CREDENTIALS` via `recvmsg(2)`; each datagram carries the sender’s `ucred`.

- **BSDs/macOS:**
  Use `getpeereid(int sock, uid_t *euid, gid_t *egid)` on the connected socket (STREAM/SEQPACKET).
  On some BSDs you can also use `getsockopt(..., SOL_LOCAL, LOCAL_PEERCRED, …)` to get a richer struct.

- **Solaris/Illumos:**
  `getpeerucred(int sock, ucred_t **uc)` gives you the peer’s creds.

### Minimal examples

**Linux (C, STREAM):**

```c
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>
#include <stdio.h>

int peer_uid(int fd, uid_t *uid, gid_t *gid, pid_t *pid) {
    struct ucred cred;
    socklen_t len = sizeof(cred);
    if (getsockopt(fd, SOL_SOCKET, SO_PEERCRED, &cred, &len) == -1) return -1;
    if (uid) *uid = cred.uid;
    if (gid) *gid = cred.gid;
    if (pid) *pid = cred.pid;
    return 0;
}
```

**BSD/macOS (C):**

```c
#include <sys/socket.h>
#include <unistd.h>

uid_t uid; gid_t gid;
if (getpeereid(conn_fd, &uid, &gid) == -1) { /* handle error */ }
```

**Linux (Python, STREAM):**

```python
import socket, struct
ucred_fmt = "3i"  # pid, uid, gid (all ints)
pid, uid, gid = struct.unpack(ucred_fmt,
    conn.getsockopt(socket.SOL_SOCKET, socket.SO_PEERCRED, struct.calcsize(ucred_fmt)))
```

### Notes & gotchas

- Credentials are tied to the peer process’s **effective** UID/GID at `connect()` (or per datagram for `SCM_CREDENTIALS`).
- Make sure you’re calling these on the **connected** socket (the result of `accept()`), not the listening socket.
- File system permissions on the socket path are useful for **access control**, but peer credential queries are how you **identify** who connected.
- Namespaced/contained environments don’t affect UIDs seen over a Unix socket within the same kernel—they’re real kernel UIDs/GIDs.

If you tell me your OS and socket type (stream vs datagram), I can tailor a tiny snippet you can drop in.
