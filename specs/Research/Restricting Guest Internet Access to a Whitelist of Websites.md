Controlling a guest’s network so it can **only access a specific whitelist of
websites** is crucial for security. The goal is to allow the guest to reach
only approved domains (and ports) and **nothing else**. This must be done under
the constraint of **no root privileges on the host**. Below we discuss the
challenges and multiple approaches – from user-space networking tricks to
proxies and DNS controls – to enforce a whitelist-only policy for guest
internet access.

## Challenges in Whitelisting Network Access (Without Root)

Enforcing an outbound whitelist at the network level is non-trivial without
root. Traditional firewalls (iptables/nftables) require root and work at the IP
level, not domain name. This means **domain-based rules** are hard – you’d
have to constantly resolve domains to IPs and update rules. As one engineer
noted, _“most firewalls work with IP addresses but not DNS domain names,”_
so maintaining an up-to-date list of IPs for allowed domains is
cumbersome[\[1\]](https://fruty.medium.com/how-to-restrict-outbound-traffic-on-a-docker-infrastructure-7effc45e313d#:~:text=,refresh%20your%20firewall%20configuration%20periodically). Without root, we cannot simply install kernel firewall
rules globally.

Another challenge is that **DNS itself can be exploited**. Even if direct
connections are blocked, a malicious guest might use DNS queries to communicate
out (encoding data in domain lookups). DNS is often left open because it seems
harmless, but attackers can use techniques like _DNS tunneling_ to exfiltrate
data or receive commands via DNS
queries[\[2\]](https://www.paloaltonetworks.com/cyberpedia/what-is-dns-tunneling#:~:text=Essentially%2C%20DNS%20tunneling%20uses%20the,system%20and%20their%20command%20server). Therefore, a robust solution should consider restricting DNS
or monitoring it, even if DNS needs to function for allowed sites.

In summary, we need a strategy that (a) works without full root privileges, (b)
enforces an _allowlist_ of external destinations (by domain/IP and port), and
ideally (c) prevents misuse of DNS as a side-channel. Below are approaches to
achieve this.

## Unprivileged User-Space Networking (SLIRP)

One method is to leverage **user-space networking** so that the guest’s
internet access is mediated by a program running in user mode. For example,
QEMU’s _“user networking”_ (SLIRP) provides a virtual NAT network
entirely in
user-space[\[3\]](https://wiki.qemu.org/Documentation/Networking#:~:text=User%20Networking%20). This allows a VM/guest to reach the internet without any tap
device or root privileges on the host. Rootless container engines use a similar
mechanism (e.g. slirp4netns) to give containers outbound connectivity when
running as unprivileged
users[\[4\]](https://docs.podman.io/en/latest/markdown/podman-network.1.html)[\[5\]](https://docs.podman.io/en/latest/markdown/podman-network.1.html).

By default, SLIRP will NAT the guest’s traffic to the outside world as if it
were a normal host behind NAT. That means without further controls, the guest
could connect to any address/port. However, QEMU offers a _“restrict”_
option for user networking. Using \-netdev user,restrict=yes **completely
isolates the guest from the host and external network** by
default[\[6\]](https://wiki.qemu.org/Documentation/Networking#:~:text=You%20can%20isolate%20the%20guest,options). In this mode, the guest still has a network
interface (and can talk to other virtual devices or services QEMU provides),
but _any attempt to reach the wider internet is blocked_ – preventing malware
from “phoning
home”[\[6\]](https://wiki.qemu.org/Documentation/Networking#:~:text=You%20can%20isolate%20the%20guest,options). You can then **selectively open holes** in
this restriction using controlled forwards. For example, QEMU allows specifying
hostfwd or guestfwd rules to forward guest traffic to specific destinations. In
practice, this could mean forwarding only certain host ports or addresses that
correspond to whitelisted sites. Essentially, SLIRP with restrict=on flips the
model to _“deny by default”_ and you explicitly **allow only what you
trust**.

**Pros:** This approach is fully unprivileged and self-contained. It doesn’t
require host firewall rules – the filtering is handled in user-space. It’s
suitable if you are using QEMU or a similar VM wrapper and can take advantage
of its built-in options. For containers, slirp4netns doesn’t (currently) have
a built-in allowlist feature, but you could potentially achieve a similar
effect by modifying the slirp code or wrapping it with logic to drop disallowed
connections. The key is that since the network stack is in userland, **you can
augment it with checks**. (For example, one could patch or use an LD_PRELOAD on
the connect() calls that slirp makes to the host, to filter outgoing
connections against an IP whitelist.)

**Cons:** Vanilla slirp by itself doesn’t know about domain names – rules
would apply at IP/port level. Managing a list of allowed IPs (especially if
domains have many IPs or change) can be complex. Also, while QEMU’s restrict
option blocks all outbound traffic (which is secure), configuring fine-grained
exceptions might require complex forwarding rules or custom code. There’s
also a performance cost to user-space networking (slirp is slower than a direct
interface).

## Isolated Network Namespace with Internal Firewall

Another technique is to use Linux network namespaces to our advantage. Even as
an unprivileged user, on many systems you can create a new user namespace and a
new network namespace (unshare \-U \-n) – this gives you a sandboxed network
environment where **you are effectively root _inside that namespace_** (with
CAP_NET_ADMIN in the namespace), though not on the real host. In that isolated
netns, you can set up a virtual interface and apply firewall rules (iptables or
nftables) that **only affect the guest’s traffic**. This is how rootless
container engines can apply certain networking rules without touching the
host’s global
firewall[\[7\]](https://docs.podman.io/en/latest/markdown/podman-network.1.html)
.

For example, with rootless Podman, one can enter the container’s network
namespace and use iptables to block or allow specific IP ranges; those iptables
rules will only impact that container’s networking, not the
host[\[7\]](https://docs.podman.io/en/latest/markdown/podman-network.1.html).We can use this capability to implement an allowlist: \- **Create a new networknamespace** for the guest. \- Run a user-space NAT (like slirp4netns or even asimple TUN/TAP \+ userland forwarder) to connect that namespace to the
internet. The guest will have an interface (say eth0 or tap0) in this
namespace. \- Inside the namespace, configure iptables to **DROP all outbound
traffic by default**, and then **ACCEPT outbound to the whitelisted
IPs/ports**. For instance, you might allow TCP to IP X on port 443, IP Y on
port 443, etc., for each allowed website’s resolved IP addresses. All other
destinations would hit the DROP rule. This way, even though the NAT process
might try to open a socket to an unallowed IP, the packet never leaves the
namespace (iptables will block it). \- The guest should also be prevented from
altering these rules – but since the guest’s processes are unprivileged
(not root in the namespace), and your orchestrator set the rules as the
namespace’s root, the guest can’t change the firewall.

This approach was demonstrated by practitioners: one solution for rootless
Podman used a **netns hook to inject iptables rules** at container startup,
ensuring only specific IPs were reachable by that
container[\[8\]](https://docs.podman.io/en/latest/markdown/podman-network.1.html). It leverages the fact that we can be root _within_ the container’s ownnamespace to enforce policy, while the host remains unprivileged.
**Pros:** This gives very strong enforcement at a low level – even if the
guest tries to create raw sockets or use non-HTTP protocols, the kernel (in the
namespace) will enforce the rules. It’s like giving the guest its own
miniature firewall. And importantly, it still doesn’t require root on the
host (only the ability to use user namespaces, which is generally allowed on
modern Linux with unprivileged user namespaces enabled).

**Cons:** Setting this up is more involved. You need to have the iptables/nft
tool available and coordinate the networking. Also, maintaining the IP
whitelist (if domains change IPs) means you’d have to update rules as needed.
There’s a slight privilege escalation in that you _do_ run iptables (which is
a privileged operation) but only in a contained scope – one must ensure the
guest can’t escape that scope. This is generally safe if user namespaces are
properly configured.

_Note:_ New Linux features like **cgroup eBPF filters** can also enforce
outbound IP rules per process or cgroup. For example, systemd’s
IPAddressAllow= setting uses cgroup BPF under the hood to allow or deny
specific IP ranges for a
service[\[9\]](https://www.freedesktop.org/software/systemd/man/249/systemd.resource-control.html#:~:text=following%20rules%20are%20applied%20in,turn)[\[10\]](h
ttps://www.freedesktop.org/software/systemd/man/249/systemd.resource-control.htm
l#:~:text=In%20order%20to%20implement%20an,relevant%20services%2C%20and%20only%2
0them). Using such BPF mechanisms, one could in theory attach a filter to the
guest process that only permits certain addresses. However, loading such eBPF
programs typically requires privileges (CAP_BPF or CAP_SYS_ADMIN). If you have
a helper daemon or systemd in play, this could be leveraged – but in pure
unprivileged scenarios, it’s not trivial to use. Thus, the netns \+ iptables
method is more straightforward for most cases.

## Application-Level Proxy and Domain-Based Filtering

When the goal is to restrict _web_ access to certain domains, an
**application-layer proxy** can be an elegant solution. Instead of trying to
handle every packet at the kernel level, you allow the guest to send requests
but funnel them through a proxy that enforces the whitelist by domain name.

One approach is to run an HTTP/HTTPS proxy (like **Squid**) on the host (or a
sidecar container). The guest would be configured to use this proxy for all web
requests. The proxy can be set up with an ACL that _only allows certain
domains_ and denies all others. For example, Squid can whitelist domains or
even only specific URLs. If the guest tries to bypass the proxy, you would
combine this with a firewall rule that blocks direct HTTP/HTTPS traffic (so the
only way out is via the proxy). In a fully unprivileged setting, you might not
have a host firewall to enforce using the proxy, but you could achieve it by
network namespace routing tricks (e.g., only route the proxy’s IP, and no
default route for others).

Another clever variant (used in a Docker scenario) is to deploy **per-domain
proxies using Nginx**. In that design, the container’s internal network had
no direct internet, but for each allowed domain (say example.com), there was an
Nginx instance joined to both the internal network and the external network.
The Nginx was configured in TCP stream mode to forward traffic to example.com
on port 443\. The container’s DNS was set so that example.com resolved to the
Nginx’s internal
IP[\[11\]](https://fruty.medium.com/how-to-restrict-outbound-traffic-on-a-docker-infrastructure-7effc45e313d#:~:text=An%20important%20point%20is%20that,and%20know%20the%20destination%20domain). Thus, when the container tried to reach
example.com, it actually hit the Nginx proxy internally, which then relayed the
bytes to the real example.com. Any other domain had no such proxy and would
fail to resolve or connect. As the author of this method noted, _“inside
docker containers, only traffic to the allowed domains will be
possible.”_[\[12\]](https://fruty.medium.com/how-to-restrict-outbound-traffic-on-a-docker-infrastructure-7effc45e313d#:~:text=With%20this%20setup%2C%20no%20outbound,allowed%20domains%20will%20be%20possible) This ensures **domain-level
whitelisting** even for HTTPS (since Nginx in stream mode doesn’t terminate
SSL, it just pipes it through, so the guest sees a real certificate from the
real site, avoiding cert issues).

**Pros:** Domain-based control is much easier to manage when you have a proxy,
because you don’t have to manually chase IP addresses. If a site’s IP
changes, DNS and the proxy take care of it as long as the domain is allowed.
Also, proxies can log and audit what the guest is accessing. This approach can
be done without root on the host (you can run a proxy as user). In fact, the
Nginx-per-domain trick was done entirely with user-level tools in Docker
Compose[\[13\]](https://fruty.medium.com/how-to-restrict-outbound-traffic-on-a-docker-infrastructure-7effc45e313d#:~:text=After%20a%20lot%20of%20searching%2C,The%20core%20ideas)[\[14\]](https://fruty.medium.com/how-to-restrict-outbound-traf
fic-on-a-docker-infrastructure-7effc45e313d#:~:text=can%E2%80%99t%20really%20pro
xy%20outbound%20SSL,and%20know%20the%20destination%20domain).

**Cons:** The guest could theoretically try to use non-HTTP protocols or IP
addresses directly to circumvent the proxy. So you either need to also block
all other traffic (which brings us back to a firewall or namespace
restriction), or ensure the guest environment is such that only web traffic is
expected. Also, maintaining one proxy per domain (in the Nginx method) can
become unwieldy if the list is long. A single proxy with a domain ACL is
easier, but then you must enforce its usage. In scenarios where you truly
cannot apply any host firewall, the guest might simply ignore the proxy
settings – so this approach works best when the environment is controlled
enough that you can assume or enforce proxy usage (for example, the guest code
is known to use HTTP only and you configure its environment accordingly).

In short, proxies are powerful for **whitelisting by domain**. They can be used
in conjunction with the other methods: e.g., run the guest in a netns that only
allows traffic to the proxy’s IP/port – thus the guest has to go through
the proxy, which then enforces domain rules. This combined strategy covers both
IP-level and domain-level filtering.

## DNS and Port Considerations

Even if the guest is limited to certain external sites, **DNS resolution** must
be handled carefully. The guest will need to resolve the allowed domain names
to connect. If you leave the guest with a general-purpose DNS (like 8.8.8.8 or
the host’s resolver), it could query arbitrary domains. Those queries
themselves could leak data. For example, malware could perform lookups like
\<secret data\>.attacker.com and, even if it can’t connect out by TCP, the
DNS query reaching an external DNS server might carry the secret in the
hostname. As Palo Alto’s security researchers note, _DNS tunneling_ is a
known method to covertly send data by encoding it in DNS
queries/responses[\[2\]](https://www.paloaltonetworks.com/cyberpedia/what-is-dns-tunneling#:~:text=Essentially%2C%20DNS%20tunneling%20uses%20the,system%20and%20their%20command%20server). Thus, completely **unrestricted DNS is a potential
communication channel**.

To mitigate this, you have a few options: \- **Whitelist DNS Queries:** Use a
DNS server that only answers for your allowed domains. You can run a local DNS
(e.g., dnsmasq) for the guest. Populate it with the A records for the
whitelisted sites (or forward those queries to real DNS) but make it return
NXDOMAIN or refuse queries for anything else. This way, even DNS queries are
effectively on a whitelist. \- **Default DNS but monitor/block:**
Alternatively, allow the guest to use normal DNS but combine it with the
IP-level restrictions. Even if the guest resolves an disallowed site, it cannot
connect to it. This doesn’t stop data exfil via the query itself, but it
limits what the guest can actually reach. Depending on your threat model, you
might accept this risk if you believe DNS tunneling is unlikely or you have
other detection in place. \- **No external DNS at all:** In a very locked-down
case, you could avoid giving the guest any real DNS server. Instead, supply an
/etc/hosts file inside the guest with the required domain-\>IP mappings for the
whitelisted sites. That way, the guest never needs to query DNS for those
sites, and any other DNS queries will just fail. This is simple but only
feasible if the set of sites is small and mostly static in IP.

Also, consider **port whitelisting** as part of the policy. If the allowed use
case is web browsing or API calls, likely only TCP port 80 (HTTP) or 443
(HTTPS) should be open. You should restrict or block all other outgoing port
numbers. For instance, even if example.com is allowed, you probably don’t
want the guest to connect to example.com:22 (SSH) or some high port, in case a
malware tries to use an alternate service on the same host. This is typically
handled in the firewall rules (e.g., allow only port 443 to the IP of
example.com, not any port) or via proxies (an HTTP proxy won’t proxy non-HTTP
protocols). Linux’s new Landlock security module is interesting here – it
allows a process (even unprivileged) to restrict itself to only making TCP
connections on certain port
numbers[\[15\]](https://docs.kernel.org/userspace-api/landlock.html#:~:text=For%20network%20access,a%20specific%20action%3A%20HTTPS%20connections). For
example, you could lock a process to _only_ allow
LANDLOCK_ACCESS_NET_CONNECT_TCP on port
443[\[15\]](https://docs.kernel.org/userspace-api/landlock.html#:~:text=For%20network%20access,a%20specific%20action%3A%20HTTPS%20connections). That would
enforce that even if it somehow tried to connect to a disallowed port, the
kernel would block it. However, Landlock (as of ABI v4) doesn’t filter by
destination IP, only by port, so it’s a coarse tool (you’d still need
something to restrict which IPs it can reach). It’s a new avenue to be aware
of for unprivileged network restricting.

## Summary and Recommendations

To **restrict a guest’s internet access to a specific whitelist** of sites
without root privileges, consider a combination of the methods above for
defense-in-depth. Key approaches include:

- **User-Space NAT with Allowlist Controls:** Use an unprivileged networking
  stack like QEMU’s SLIRP or slirp4netns to provide NAT connectivity, and
  enable restrictive mode. For instance, QEMU’s restrict=yes cuts off all
  outbound traffic by default (no phoning home) and you then explicitly
  forward/allow only the approved sites or
  ports[\[6\]](https://wiki.qemu.org/Documentation/Networking#:~:text=You%20can%20isolate%20the%20guest,options). This approach requires no root and is a good
  starting point for VM sandboxing.

- **Network Namespace \+ Internal Firewall:** Create a separate network
  namespace for the guest (via user namespaces so it’s rootless on the host).
  Inside that namespace, apply iptables/nft rules that **whitelist specific IP
  addresses and ports** and drop everything else. This effectively gives the
  guest its own firewall that you
  control[\[7\]](https://docs.podman.io/en/latest/markdown/podman-network.1.html).The guest will only be able to send packets to the allowed destinations. Thisrequires some setup (bringing up the interface, NAT or forwarding for internetaccess, etc.), but it’s powerful. The host remains untouched by these rules.

- **Application-Layer Proxy (Domain Whitelisting):** If applicable, funnel the
  guest’s traffic through a proxy server that permits only certain domains. For
  web access, a proxy can enforce an ACL by domain name, which is much simpler
  than maintaining IP lists. You might run a local Squid proxy or use the Nginx
  TCP proxy technique to pass-through to allowed
  sites[\[14\]](https://fruty.medium.com/how-to-restrict-outbound-traffic-on-a-docker-infrastructure-7effc45e313d#:~:text=can%E2%80%99t%20really%20proxy%20outbound%20SSL,and%20know%20the%20destination%20domain). Ensure the guest cannot
  bypass the proxy (e.g., by blocking direct internet routes, so only the proxy
  is reachable from the guest). This method is especially useful for HTTPS, since
  the proxy (in pass-through mode) doesn’t need to decrypt traffic – it just
  controls the destination.

- **DNS and Ports Restrictions:** Don’t forget to restrict _where_ the guest
  can resolve names and which ports it can use. If possible, provide a filtered
  DNS service that only resolves whitelisted domains, to close the DNS tunneling
  loophole[\[2\]](https://www.paloaltonetworks.com/cyberpedia/what-is-dns-tunneling#:~:text=Essentially%2C%20DNS%20tunneling%20uses%20the,system%20and%20their%20command%20server). Limit outgoing connections to the standard ports (80/443 or
  others explicitly needed) – all other ports should be blocked to prevent
  abuse. This can be done via firewall rules (e.g., only allow tcp/443 to allowed
  IPs) or via mechanisms like Landlock if
  available[\[15\]](https://docs.kernel.org/userspace-api/landlock.html#:~:text=For%20network%20access,a%20specific%20action%3A%20HTTPS%20connections).

In practice, a **combined solution** often works best. For example, you might
run the guest in a locked-down network namespace with slirp (for rootless NAT)
and iptables rules allowing only specific IPs:ports. At the same time, you
could run a DNS server that only resolves approved hosts for that namespace,
and maybe even force HTTP traffic through a proxy for logging. These layers
together greatly reduce the chance of the guest reaching anything outside the
whitelist or leaking data out-of-band.

If a fully unprivileged solution proves too difficult (for instance, maybe your
scenario doesn’t allow user namespaces or you can’t easily implement slirp
filtering), you might employ a minimal privilege on the host: e.g., a helper
service with CAP_NET_ADMIN that sets up iptables rules on the host for the
guest’s traffic, or using systemd’s built-in IP allowlisting for the
service. But in most cases, the approaches outlined can be achieved without
giving the main guest process any root rights on the host.

By carefully **whitelisting at multiple layers** (network layer and
DNS/application layer), you can confidently allow the guest to access needed
sites (with proper name resolution), while **ensuring it cannot contact or
attack anything else**. This dramatically limits the potential for abuse –
even if the guest is running untrusted code, it can only talk to the small set
of external endpoints you’ve approved, on the ports you’ve allowed.

---

[\[1\]](https://fruty.medium.com/how-to-restrict-outbound-traffic-on-a-docker-infrastructure-7effc45e313d#:~:text=,refresh%20your%20firewall%20configuration%20periodically)
[\[11\]](https://fruty.medium.com/how-to-restrict-outbound-traffic-on-a-docker-infrastructure-7effc45e313d#:~:text=An%20important%20point%20is%20that,and%20know%20the%20destination%20domain)
[\[12\]](https://fruty.medium.com/how-to-restrict-outbound-traffic-on-a-docker-infrastructure-7effc45e313d#:~:text=With%20this%20setup%2C%20no%20outbound,allowed%20domains%20will%20be%20possible)
[\[13\]](https://fruty.medium.com/how-to-restrict-outbound-traffic-on-a-docker-infrastructure-7effc45e313d#:~:text=After%20a%20lot%20of%20searching%2C,The%20core%20ideas)
[\[14\]](https://fruty.medium.com/how-to-restrict-outbound-traffic-on-a-docker-infrastructure-7effc45e313d#:~:text=can%E2%80%99t%20really%20proxy%20outbound%20SSL,and%20know%20the%20destination%20domain) How to Restrict Outbound Traffic on
a Docker Infrastructure | by françois Ruty | Medium

[https://fruty.medium.com/how-to-restrict-outbound-traffic-on-a-docker-infrastructure-7effc45e313d](https://fruty.medium.com/how-to-restrict-outbound-traffic-on-a-docker-infrastructure-7effc45e313d)

[\[2\]](https://www.paloaltonetworks.com/cyberpedia/what-is-dns-tunneling#:~:text=Essentially%2C%20DNS%20tunneling%20uses%20the,system%20and%20their%20command%20server) What Is DNS Tunneling? \[+ Examples & Protection Tips\] \- Palo Alto
Networks

[https://www.paloaltonetworks.com/cyberpedia/what-is-dns-tunneling](https://www.paloaltonetworks.com/cyberpedia/what-is-dns-tunneling)

[\[3\]](https://wiki.qemu.org/Documentation/Networking#:~:text=User%20Networking%20)
[\[6\]](https://wiki.qemu.org/Documentation/Networking#:~:text=You%20can%20isolate%20the%20guest,options) Documentation/Networking \- QEMU

[https://wiki.qemu.org/Documentation/Networking](https://wiki.qemu.org/Documentation/Networking)

[\[4\]](https://docs.podman.io/en/latest/markdown/podman-network.1.html)
[\[5\]](https://docs.podman.io/en/latest/markdown/podman.1.html#rootless-mode)
[\[7\]](https://docs.podman.io/en/latest/markdown/podman-network.1.html)
[\[8\]](https://docs.podman.io/en/latest/markdown/podman-network.1.html) Podmannetworking (rootless mode)

[https://docs.podman.io/en/latest/markdown/podman-network.1.html](https://docs.podman.io/en/latest/markdown/podman-network.1.html)

[\[9\]](https://www.freedesktop.org/software/systemd/man/249/systemd.resource-control.html#:~:text=following%20rules%20are%20applied%20in,turn)
[\[10\]](https://www.freedesktop.org/software/systemd/man/249/systemd.resource-control.html#:~:text=In%20order%20to%20implement%20an,relevant%20services%2C%20and%20only%20them) systemd.resource-control

[https://www.freedesktop.org/software/systemd/man/249/systemd.resource-control.html](https://www.freedesktop.org/software/systemd/man/249/systemd.resource-control.html)

[\[15\]](https://docs.kernel.org/userspace-api/landlock.html#:~:text=For%20network%20access,a%20specific%20action%3A%20HTTPS%20connections) Landlock:
unprivileged access control — The Linux Kernel documentation

[https://docs.kernel.org/userspace-api/landlock.html](https://docs.kernel.org/userspace-api/landlock.html)
