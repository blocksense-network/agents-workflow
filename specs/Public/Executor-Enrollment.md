# AW Server Executor Enrollment

## 0) Terminology / Glossary

- **Client** — the user running the `aw task` command (and optionally opening an SSH session through the access point).
- **Access point server** — the central service running `aw agent access-point` (address supplied to clients/executors via the `remote-server` option).
- **Executors** — machines executing `aw agent enroll` (also pointed at the same `remote-server`).

---

## 1) Goals & Requirements

We’re building a Nomad‑like system with a **single public access point** that everything dials into. We need:

1. **Executor connectivity (egress‑only):** Each executor connects _outbound_ to the access point, stays connected, and exposes internal capabilities (SSH, task execution).
2. **Client can SSH into workers via the access point:** External users (the "client" running `aw task` or an ops shell) reach **executors' SSH** _through_ the access point, ideally over **HTTPS :443** (HTTP CONNECT or WebSocket), so firewalls are simple.
3. **Server‑initiated commands (RPC):** The access point can issue RPC‑like commands to executors.
4. **Sophisticated placement:** Access point is scheduling‑aware (executor resources/labels/affinity); tasks are directed to specific executors.
5. **Strong auth & encryption:** Mutual authentication for executors↔access point; authenticated, authorized clients; defense‑in‑depth.
6. **Rust‑first implementation**, with a browser terminal option.

Out of scope here: observability/metrics. (We’ll add later.)

---

## 2) High‑Level Architecture

### 2.1 Components

- **aw‑serve (access point)**
  - Listens on **HTTPS :443**.
  - Terminates user auth (OIDC/JWT and/or mTLS) and exposes two ingress paths:
    - **HTTP CONNECT tunnel**: native SSH clients can tunnel to a specific executor's SSH.
    - **WebTransport endpoint (with WebSocket fallback)**: browser terminal to an executor.

  - Maintains a **long‑lived secure transport** to each executor.
  - Runs the **scheduler/placement** and **RPC control plane**.

- **aw‑agent (executor)**
  - Performs **workload attestation** and obtains a short‑lived **SPIFFE X.509 SVID**.
  - Establishes a persistent **secure transport** to aw‑serve (egress only).
  - Hosts an internal **SSH server** (system `sshd`) and executes tasks.

- **aw CLI (client)**
  - For tasks: calls `aw agent access-point`’s [REST Service](REST%20Service.md) to submit work.
  - For SSH: either uses **HTTP CONNECT** via standard `ssh` `ProxyCommand`, or uses the **web UI** with a terminal emulator.

### 2.2 Transports at a glance

- **Executor ↔ Access point:** QUIC (UDP/443) via `quinn`, with **mTLS** using SPIFFE SVIDs. Multiplex **many independent streams** (control/RPC/tunnels) over one connection.
- **Client ↔ Access point:** HTTPS (TCP/443). Two modes:
  - **HTTP CONNECT** (H1/H2) → byte‑for‑byte SSH tunnel to an executor's local `sshd`.
  - **WebTransport/WebSocket** → server‑side SSH client to the executor; browser gets a PTY stream.

---

## 3) Identity & Authentication (SPIFFE/SPIRE + OIDC)

### 3.1 Identities

- **aw‑agent (executor) identity:** `spiffe://<org>/aw/agent/<node-id>`
- **aw‑serve identity:** `spiffe://<org>/aw/serve`
- **Human clients (aw CLI / web):** OIDC (JWT) or enterprise mTLS. We _can_ also support SPIFFE for internal service users.

### 3.2 SPIRE deployment (managed via NixOS modules)

- **spire‑server** runs in our control environment; issues SVIDs under `spiffe://<org>/...`.
- **spire‑agent** runs on every executor and on the access point host; exposes the **Workload API** over a Unix socket.
- **Node attestors:** pick per environment (e.g., `x509pop`, cloud IID, or k8s).
- **Workload attestors:** match the aw‑agent and aw‑serve binaries/UIDs or cgroups.

### 3.3 mTLS policy

- **Executor→Server:** aw‑agent presents `spiffe://<org>/aw/agent/*`; aw‑serve verifies and authorizes. The **SPIFFE ID becomes the executor's canonical ID**.
- **Server→Executor:** aw‑serve presents `spiffe://<org>/aw/serve`; aw‑agent validates to prevent MITM.

### 3.5 Pluggable Identity Providers (SPIFFE‑first)

While SPIFFE via the SPIRE Workload API is the production default, the agent supports a pluggable identity provider interface so operators can bring their own PKI without code changes. SPIFFE remains first‑class and easiest to use; alternatives must still enforce peer verification.

Provider interface (conceptual):

```rust
struct TlsMaterials { client: rustls::ClientConfig, server_id_policy: PeerPolicy }

#[async_trait]
trait IdentityProvider {
  async fn load(&self) -> TlsMaterials;                // initial creds
  fn watch(&self) -> Pin<Box<dyn Stream<Item = TlsMaterials> + Send>>; // rotations
}

enum PeerPolicy {
  SpiffeId { expected: String },
  SanAllowlist { dns: Vec<String>, uri: Vec<String>, spki_pins: Vec<String> },
}
```

Built‑in providers:

- SPIFFE (default): Reads X.509 SVID + trust bundle from the SPIRE Workload API UDS, auto‑rotates, and enforces `PeerPolicy::SpiffeId` against the access point.
- Files: Reads PEM keypair and CA bundle from disk, watches for changes (inotify/kqueue) and hot‑reconnects; enforces `SanAllowlist` with DNS/URI and optional SPKI pins.
- Vault: Fetches short‑lived certs from Vault PKI (AppRole/JWT/etc.), renews on TTL, and enforces `SanAllowlist`.
- Exec: Runs an external command that prints PEM materials; refreshes on a fixed interval; enforces `SanAllowlist`.

Runtime behavior:

- On start, construct the provider from CLI/config, call `load()` to build `quinn/rustls` client, and connect.
- Subscribe to `watch()`; on update, gracefully reconnect to apply new TLS materials.
- Always enforce peer policy (SPIFFE ID in default mode; SAN allowlist in others).

### 3.4 Token‑based user auth

- **CLI/Web users** authenticate to aw‑serve using **OIDC JWT** (Auth0/Keycloak/…):
  - Bearer token in HTTP `Authorization` for REST/RPC.
  - For HTTP CONNECT, token in `Proxy-Authorization: Bearer …`.
  - For WebSocket, token in `Authorization` header during upgrade.

- **AuthZ**: RBAC mapping `user → allowed executors/projects` and `verbs` (ssh, exec, read‑logs).

---

## 4) Transport Layer (Executor ↔ Access Point)

### 4.1 QUIC session

- The agent dials `quic://<access-point>:443` using **`quinn`**. The `quinn` endpoint is configured with **`rustls`** tied to SPIFFE‑delivered certs/roots.
- One QUIC connection per executor; **streams** for:
  1. **Control** (hello/heartbeat, resource reports, liveness)
  2. **RPC** (bidirectional request/response, server‑initiated)
  3. **TCP proxy streams** (for SSH and any future port proxy)

### 4.2 Stream contracts

- **Control:**
  - `Hello{ executor_id, version, resources, labels }`
  - Periodic `Heartbeat{ running, load, free_slots }`
  - `Goodbye{ reason }`

- **OpenTcp:**
  - Server → Agent: `OpenTcp{ dst: 127.0.0.1:22, reason: "ssh" }`
  - Agent connects to its local `sshd`, replies `OpenTcpOk{}` or `OpenTcpErr{ code }`, then both sides **pump bytes** until EOF.

- **RPC:** custom framed messages (see §6).

### 4.3 Fallback plan (future)

If UDP/QUIC is blocked, provide an alternate TCP transport (e.g., TLS + HTTP/2 with a stream multiplexer). Same control/RPC semantics over a different carrier.

---

## 5) Edge Ingress on :443 (Client ↔ Access Point)

### 5.1 HTTP CONNECT tunnel for native SSH

- **Flow:**
  1. Client issues `CONNECT w-123.internal:22` to `https://ap.example.com:443` with `Proxy-Authorization: Bearer <token>`.
  2. aw‑serve authenticates+authorizes, resolves `w-123` → the live QUIC connection.
  3. aw‑serve opens a **QUIC bidirectional stream** to the executor and sends `OpenTcp{127.0.0.1:22}`.
  4. After `200 Connection Established`, aw‑serve **blindly pipes bytes** between client and executor's `sshd`.

- **Client config examples** (no custom binaries required):
  - _OpenBSD `nc`_

    ```
    Host w-*
      HostName %h
      Port 22
      ProxyCommand nc -x ap.example.com:443 -X connect %h %p
      HostKeyAlias %h
    ```

  - _socat_

    ```
    Host w-*
      HostName %h
      Port 22
      ProxyCommand socat - "PROXY:ap.example.com:%h:%p,proxyport=443"
      HostKeyAlias %h
    ```

- **Server implementation:** an `axum`/`hyper` handler for CONNECT that:
  - validates Bearer token and executor ACLs;
  - opens a QUIC stream to the executor;
  - `tokio::io::copy_bidirectional(client_tcp, quic_stream)`.

- **Host keys:** Clients verify the **executors'** `sshd` host key. We set `HostKeyAlias %h` so known_hosts entries are per‑executor (stable).

### 5.2 Browser terminal over WebSocket

- **Why:** Browsers can’t open raw TCP, so we terminate SSH at the access point and expose a PTY stream to the browser.
- **Flow:**
  - Browser connects `wss://ap.example.com/ssh/w-123` (JWT in header/cookie).
  - aw‑serve creates an SSH **client session** to the executor (via QUIC `OpenTcp` to local `sshd`), requests a PTY + shell, then bridges WS ⇄ SSH channel.
  - Frontend uses **xterm.js** for display and keyboard handling.

- **Trade‑off:** SSH is **not end‑to‑end** to the executor from the browser (it terminates at aw‑serve). For human convenience this is standard practice.

### 5.3 Browser terminal with WebTransport

- Possible evolution using HTTP/3/WebTransport for better stream control. Still server‑terminated SSH unless we ship a browser‑side SSH implementation (out of scope).

---

## 6) Control Plane & RPC

### 6.1 Framework choice

Use quic-rpc as the primary RPC framework over our existing quinn connection. It maps cleanly to our transport model: one executor maintains a single QUIC connection to the access point; each RPC is a separate QUIC stream. This gives us natural isolation and backpressure per call, avoids head‑of‑line blocking, and keeps cancellation semantics simple (drop the stream = cancel the call).

### 6.2 QUIC ↔ RPC mapping

- **Connection:** one QUIC connection per executor (mutually authenticated with SPIFFE SVIDs).
- **Streams:**
  - **Control:** long‑lived unidirectional stream (hello/heartbeat/config).
  - **RPC:** **one bi‑stream per request** created by the access point when it invokes a method on an executor (e.g., `StartTask`).
  - **Tunnels:** separate bi‑streams for OpenTcp/SSH sessions.
- **Flow control:** QUIC stream‑level flow control provides natural backpressure. If an executor is saturated, new RPC streams will queue at the access point until the peer credits more window.
- **Cancellation:** callers drop the stream handle; the callee observes EOF and aborts work if safe.

### 6.3 Service definition pattern

Define message types as serde‑serializable structures. With `quic-rpc` we implement a service interface whose methods are invoked over newly opened QUIC streams.

**Example (conceptual):**

```rust
// Messages (serde or prost-compatible)
#[derive(Serialize, Deserialize)]
pub struct StartTaskReq { pub task_id: String, /* command, env, mounts, resources, labels */ }
#[derive(Serialize, Deserialize)]
pub struct StartTaskResp { pub pid: u32, pub accepted: bool }
```

On the **access point**, when the scheduler places a task on runner `R`, it opens a new RPC stream to `R` and sends `StartTaskReq`. The **runner** handles the request, starts the container/process, and writes `StartTaskResp`, then closes the stream.

TODO: This should evolve into a concrete spec with the actual messages types that we are going to use.

### 6.4 Namespacing & lanes

Keep **control** traffic on its own stream (or a tiny control service) and invoke **task RPCs** on short‑lived, per‑call streams. If we need priority, we can maintain separate logical “lanes” (e.g., `control`, `task`, `bulk`) by opening them from distinct tasks/services and limiting concurrency per lane at the caller.

### 6.5 Idempotency, retries, and deadlines

- **Idempotency keys:** include `rpc_id` and `task_id` so an executor can de‑duplicate repeats after network loss.
- **Retries:** access point may retry non‑started operations; executors should implement at‑least‑once semantics guarded by idempotency.
- **Deadlines:** include optional `deadline_ms` in request metadata; the callee should abort on expiry and close the stream.

### 6.6 Example RPC surface (server → executor)

- `StartTask { task_id, image/command, env, mounts, resources } -> Started{ pid }`
  TODO: The fields here are placeholders. Complete the spec by examining the needs of the task spawner in a more holistic fashion.

- `StopTask  { task_id } -> Stopped{ status }`
  TODO: This should be made consistent with the session commands available in the CLI.
  The UIs (TUI and WebUI) should offer the same controls as visible short-cuts/buttons.

  TODO: It's unclear whether the messages below are needed

- `FsRead/FsWrite` (bounded, carefully authorized)
- `Ping -> Pong { ts }`

### 6.7 Scheduling hook

- Executor's `Hello/Heartbeat` carries resources + labels (`region=eu`, `gpu=true`, …).
  TODO: Detail the resource reporting.
  The hello mesasge should include OS, disk space, memory, access to special hardware (e.g. GPU).
  The heartbeat message should should cover free system resources (CPU, memory, etc), so the scheduler can make intelligent decisions on where to launch new coding sessions.

- Access point selects an executor and invokes `StartTask` over that executor's RPC stream.

---

## 7) Security model

- **mTLS** between aw‑agent and aw‑serve via SPIFFE SVIDs and trust bundles.
- **Least‑privilege OpenTcp:** Agent only accepts **allow‑listed destinations** (default: `127.0.0.1:22`).
- **Per‑request AuthZ:** aw‑serve enforces `user → executor → verb` for CONNECT, WebSocket, and RPC.
- **Key management:**
  - Executors use OS `sshd` host keys as usual (rotate per your policy).
  - aw‑serve’s SSH client for web terminal uses short‑lived ephemeral keys or a managed keyring.

- **Audit:** Access point logs who opened which tunnel to which executor (and why). (Details to be filled in phase 2.)

---

## 8) Concrete tech choices (Rust & Web)

**Runner↔Server (transport):**

- `quinn` (QUIC) for multiplexed, reliable streams over UDP/443.
- `rustls` for TLS; certs/roots sourced from SPIFFE Workload API via the `spiffe` crate.

**Edge HTTP server:**

- `axum` (on `hyper`) for HTTPS endpoints, including a custom **CONNECT** handler and **WebSocket** upgrades.

**SSH pieces:**

- **Native clients:** no special client lib required; they use HTTP CONNECT.
- **Server‑side SSH (for browser):** `russh` client to the executor; pty channel bridged to WebSocket.
- **Terminal in browser:** `xterm.js`.

**Message formats:** prost (Protobuf) or serde+CBOR for control/RPC; small and explicit.

---

## 9) NixOS modules (SPIFFE + services)

The following sketches show how we’ll model this declaratively. (Names are illustrative.)

### 9.1 Access point host

```nix
{ config, pkgs, lib, ... }:
let
  domain = "ap.example.com";
in {
  services.spire-server.enable = true;            # or run spire-server elsewhere
  services.spire-agent.enable = true;             # so aw-serve can fetch its SVID

  # Expose the Workload API socket to aw-serve
  systemd.services.aw-serve = {
    wantedBy = [ "multi-user.target" ];
    after    = [ "network-online.target" "spire-agent.service" ];
    serviceConfig = {
      Environment = [
        "SPIFFE_ENDPOINT_SOCKET=/run/spire/sockets/agent.sock"
        "AW_LISTEN_ADDR=0.0.0.0:443"
      ];
      ExecStart = "${pkgs.aw}/bin/aw agent access-point";    # our binary
      DynamicUser = true;
      AmbientCapabilities = [ "CAP_NET_BIND_SERVICE" ];
    };
  };
}
```

### 9.2 Executor hosts

```nix
{ config, pkgs, lib, ... }:
{
  services.openssh.enable = true;                 # executor's sshd (on 127.0.0.1:22 allowed)
  services.spire-agent.enable = true;             # fetch SVID for aw-agent

  systemd.services.aw-agent = {
    wantedBy = [ "multi-user.target" ];
    after    = [ "network-online.target" "spire-agent.service" "sshd.service" ];
    serviceConfig = {
      Environment = [
        "SPIFFE_ENDPOINT_SOCKET=/run/spire/sockets/agent.sock"
        "AW_REMOTE_SERVER=https://ap.example.com"  # used by enroll
      ];
      ExecStart = "${pkgs.aw}/bin/aw agent enroll --remote-server ${config.environment.variables.AW_REMOTE_SERVER} --identity spiffe --spiffe-socket /run/spire/sockets/agent.sock --expected-server-id spiffe://example.org/aw/serve";
      DynamicUser = true;
    };
  };
}
```

> In SPIRE, define registration entries so that processes matching `aw-agent` and `aw-serve` receive the intended SPIFFE IDs. Choose appropriate node/workload attestors (e.g., `x509pop`, cloud IID, k8s) for your environment.

---

## 10) Protocol sketches

### 10.1 Control

```text
stream: CONTROL (uni)
Client (agent) → Server
  Hello{executor_id, version, resources, labels}
  ⟲ Heartbeat{running, load, free_slots}
Server → Client
  Ack{now, config_hash}
```

### 10.2 OpenTcp (SSH)

```text
stream: TUNNEL (bi)
Server → Agent: OpenTcp{dst = 127.0.0.1:22}
Agent  → Server: OpenTcpOk{}
Then: raw TCP byte‑pump until EOF.
```

### 10.3 RPC (example)

```protobuf
service Runner {
  rpc StartTask(StartTaskReq) returns (StartTaskResp);
  rpc StopTask(StopTaskReq)   returns (StopTaskResp);
  rpc Exec(ExecReq)           returns (ExecResp);
}
```

(Tied to a QUIC bi‑stream; either `tarpc` custom transport or length‑prefixed protobuf frames.)

---

## 11) Client ergonomics

### 11.1 SSH via CONNECT (no custom binaries)

`~/.ssh/config`:

```sshconfig
Host w-*
  HostName %h
  Port 22
  ProxyCommand nc -x ap.example.com:443 -X connect %h %p
  HostKeyAlias %h
```

Usage: `ssh ubuntu@w-123`.

### 11.2 Browser terminal

- Open `https://ap.example.com/ui/ssh/w-123` → JWT auth → xterm.js connects to `wss://…/ssh/w-123`.

### 11.3 Task submission

- `aw task run --remote-server https://ap.example.com --executor w-123 ./job.yaml`

---

## 12) Edge cases & notes

- **Client IP visibility on executor:** executor's `sshd` sees `127.0.0.1` because the agent dials localhost. Log real client IP at the access point.
- **Rate limiting & DoS:** apply per‑user/rate policies at CONNECT/WebSocket handlers.
- **Large file copy over RPC:** prefer SSH `scp`/`sftp` through the tunnel, or dedicated artifact storage.
- **Firewalling on executors:** `sshd` can be bound to `127.0.0.1` only; agent is the only path in.

---

## 13) Milestones

1. Minimal: agent QUIC connect; hello/heartbeat; CONNECT handler; SSH tunnel.
2. RPC v0: `StartTask`/`StopTask`; placement by label.
3. Web terminal (russh client + xterm.js).
4. Policy engine & hardened allow‑lists; production SPIRE rollout.

---

## 14) Appendix: Crates & libs we’ll use

- **QUIC:** `quinn`
- **TLS:** `rustls`
- **SPIFFE Workload API (Rust):** `spiffe` crate for fetching SVIDs/bundles.
- **HTTP/HTTPS server:** `axum` on `hyper`
- **WebSocket:** `axum::extract::ws` (or `axum-tungstenite`)
- **SSH (server‑side client for web):** `russh`
- **Framing/IDL:** `prost` (Protobuf) or `serde`+`cbor`
- **Async runtime:** `tokio`

That’s the blueprint. Next step: pick the framing (tarpc vs protobuf), then stub the QUIC control loop and a CONNECT handler, and wire a trivial `StartTask` RPC.
