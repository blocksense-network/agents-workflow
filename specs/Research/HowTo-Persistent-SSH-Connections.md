## How can I avoid the SSH handshake when executing multiple SSH commands during a fleet agent coding session?

Yes—use OpenSSH’s **connection multiplexing**. It lets many “exec” sessions (channels) share one persistent TCP/SSH connection, all from the CLI.

# Option A: one-off master started explicitly

```bash
# 1) Pick a control socket path (short is best to avoid path-length limits)
mkdir -p ~/.ssh/cm

# 2) Start a persistent master in the background
ssh -M -N -f \
  -o ControlPath=~/.ssh/cm/%C \
  -o ControlPersist=10m \
  user@host

# 3) Run as many commands as you like over that same connection
ssh -S ~/.ssh/cm/%C user@host 'uname -a'; echo "exit=$?"
ssh -S ~/.ssh/cm/%C user@host 'sh -c "exit 42"'; echo "exit=$?"   # prints 42
ssh -S ~/.ssh/cm/%C user@host 'sleep 3; echo done' &              # runs in parallel
ssh -S ~/.ssh/cm/%C user@host 'date' &

# 4) (Optional) Check/stop the master
ssh -S ~/.ssh/cm/%C -O check user@host    # prints “Master running”
ssh -S ~/.ssh/cm/%C -O exit  user@host    # cleanly close the master
```

- Each `ssh ... 'cmd'` opens a **new exec channel** on the persistent connection.
- The **remote command’s exit status** becomes the exit status of that ssh invocation (`$?`), as usual. (`255` means the ssh client itself failed to connect/run.)
- You can run multiple commands **concurrently** (background with `&`); the master multiplexes them under the hood.

# Option B: make it automatic in `~/.ssh/config`

```sshconfig
Host mybox
  HostName host
  User user
  ControlMaster auto
  ControlPersist 10m
  ControlPath ~/.ssh/cm/%C
```

Now just:

```bash
ssh mybox 'cmd1'
ssh mybox 'cmd2'
```

The first call creates the master automatically; later calls reuse it until `ControlPersist` expires.

# Tips & caveats

- **Short ControlPath**: use `%C` (hash of host/user/port) to avoid UNIX socket path-length issues.
- **Server limit**: concurrent sessions per connection are capped by the server’s `MaxSessions` (often \~10). If you hit it, you’ll see failures opening new channels.
- **Keep-alives**: if the master should live through idle periods or flaky links, consider `-o ServerAliveInterval=30 -o ServerAliveCountMax=3`.
- **Auth prompts**: for scripts, add `-o BatchMode=yes` so ssh fails instead of prompting for a password.

That’s all you need for a persistent connection with many exec channels—fully from the command line, and each command still returns its own exit code.
