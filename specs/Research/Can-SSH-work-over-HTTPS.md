Yes—SSH can traverse an **HTTPS proxy**, but SSH itself doesn’t “speak HTTPS.” You have a few practical ways to make it work:

# 1) HTTP/HTTPS CONNECT tunnel (most common)

Use the proxy’s **CONNECT** method to create a raw TCP tunnel to your SSH server. From SSH’s point of view, it’s a normal TCP connection.

- **No proxy auth (HTTP proxy on :8080):**

```bash
ssh -o 'ProxyCommand=nc -X connect -x proxy.company.com:8080 %h %p' user@ssh.example.com
```

(Requires the OpenBSD flavor of `nc` that supports `-X connect -x`.)

- **Proxy requires auth or TLS (“HTTPS proxy” on :443):**  
   Use a helper that understands CONNECT + (optional) TLS and auth, e.g. **proxytunnel** or **corkscrew**.

```bash
# HTTPS proxy (with or without Basic auth)
ssh -o 'ProxyCommand=proxytunnel -p proxy.company.com:443 -d %h:%p' user@ssh.example.com
# If needed, add auth:  -P user:pass
```

```bash
# HTTP proxy with Basic auth
ssh -o 'ProxyCommand=corkscrew proxy.company.com 8080 %h %p ~/.proxy-auth' user@ssh.example.com
```

> Notes
>
> - Many corporate proxies only allow CONNECT to ports like **443**. If so, have your SSH server listen on 443 (or use a port-forwarder on the server) and CONNECT to that.
> - Some proxies do TLS interception and require a real TLS handshake (ALPN/SNI). In that case, raw SSH on :443 won’t pass—use the TLS-wrapped options below.

# 2) Wrap SSH in TLS so it “looks like HTTPS”

If the proxy (or middleboxes) insist on seeing TLS, run a TLS wrapper on both ends and tunnel SSH through it.

**Server (stunnel)**

```ini
[ssh]
accept = 0.0.0.0:443
cert   = /etc/ssl/certs/fullchain.pem
key    = /etc/ssl/private/privkey.pem
connect = 127.0.0.1:22
```

**Client (stunnel)**

```ini
[ssh]
client = yes
accept = 127.0.0.1:2222
connect = your.domain.tld:443
verifyChain = yes
CAfile = /path/to/ca.pem
```

Then:

```bash
ssh -p 2222 user@127.0.0.1
```

This satisfies “HTTPS-like” inspection because a real TLS session is established first.

# 3) SSH over WebSocket/HTTP2 (when CONNECT is restricted)

Use a tool that tunnels TCP over **WSS** (WebSocket over TLS), e.g. **chisel**, **websocat**, etc. This often passes strict proxies/firewalls that permit only browser-like traffic.

- Run a WSS server behind the proxy and a client locally, then point SSH at the local forwarded port.

---

## Common pitfalls & tips

- **Proxy policies:** Some proxies restrict CONNECT targets; use port 443 on the SSH server if necessary.
- **Auth schemes:** Basic is easy; NTLM/Kerberos may require a local helper like **cntlm** to convert to Basic for tools like corkscrew/proxytunnel.
- **OpenSSH options:** `ProxyCommand` is the knob for all of the above. (`ProxyJump` is for SSH jump hosts, not HTTP proxies.)
- **Security:** Still validate TLS (CAfile/verifyChain) when using TLS-wrappers; don’t send proxy creds in shell history.

If you share your OS and the proxy’s auth type (none/Basic/NTLM) I can give a copy-pasteable config tailored to your setup.
