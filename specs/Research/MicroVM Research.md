## How are MicroVMs launched?

MicroVMs are launched by a userspace Virtual Machine Monitor (VMM) on top of
KVM. The exact workflow depends on the VMM, but all follow the same
high‑level pattern: start the VMM process, point it at a kernel image and a
root filesystem, attach minimal paravirtual devices (virtio
block/net/console/vsock), configure networking, then boot the guest.

- Firecracker
  - Control plane: a REST API over a Unix socket configures the microVM
    pre‑boot (kernel, rootfs, network, logging) and triggers `InstanceStart`. In
    production it is typically wrapped by the `jailer` for process isolation. The
    official guide shows the complete sequence with `curl` calls to `/boot-source`,
    `/drives/<id>`, `/network-interfaces/<id>`, then `/actions` for
    `InstanceStart`. See: [Firecracker Getting Started (API and example workflow)](https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md).
  - Launching options: run Firecracker directly and drive its API; or use
    higher‑level tooling: - Weaveworks Ignite provides a Docker‑like UX, launching Firecracker
    microVMs from OCI images (`ignite run …`) and optionally from a kernel OCI
    image (`--kernel-image`, expects `/boot/vmlinux`). See: [Ignite README](https://github.com/weaveworks/ignite) and [Igniteusage](https://ignite.readthedocs.io/en/stable/usage/).

- Cloud Hypervisor
  - Direct CLI: pass kernel, cmdline, disks, memory, CPUs, and networking on
    one command (supports direct kernel boot or firmware boot via Rust Hypervisor
    Firmware/edk2). See: [Cloud Hypervisor Quick Start](https://www.cloudhypervisor.org/docs/prologue/quick-start/).- REST API: start the daemon with `--api-socket`, then invoke endpoints suchas `PUT /api/v1/vm.create` and `PUT /api/v1/vm.boot` as defined in the OpenAPIspec. See: [OpenAPIspec](https://github.com/cloud-hypervisor/cloud-hypervisor/blob/main/vmm/src/api/openapi/cloud-hypervisor.yaml).

- QEMU “microvm” machine type
  - Minimal QEMU machine modeled after Firecracker: no PCI/ACPI, up to eight
    `virtio-mmio` devices. Boot is done via host‑provided kernel and optional
    initrd (no current firmware can boot from virtio‑mmio), so you pass `-kernel
vmlinux -append …` and attach virtio‑mmio block/net. See: [QEMU microvm docs](https://www.qemu.org/docs/master/system/i386/microvm.html).

- Kata Containers (VM‑based container sandboxing)
  - From a container runtime (containerd/CRI‑O), the Kata runtime launches
    each pod in a lightweight VM using a supported VMM backend (QEMU, Cloud
    Hypervisor, or Firecracker). Kubernetes selects Kata via `RuntimeClass`. Guest
    images include the `kata-agent`. The osbuilder tooling builds the guest
    rootfs/image used by the VMM. See: [Kata osbuilder rootfs-builder](https://github.com/kata-containers/kata-containers/tree/main/tools/osbuilder/rootfs-builder).

- Examples in production
  - AWS Lambda and AWS Fargate launch workloads in Firecracker microVMs
    (milliseconds startup; hardware‑virtualization isolation).
    See: [AWS announcement](https://aws.amazon.com/blogs/aws/firecracker-lightweight-virtualization-for-serverless-computing/)
    and [Firecracker site](https://firecracker-microvm.github.io/).- Fly.io “Machines” run in Firecracker microVMs; their docs explicitlystate apps run in Firecracker microVMs. See: [Fly.ioarchitecture](https://fly.io/docs/reference/architecture/).

Notes and nuances

- Networking is commonly TAP‑based with NAT on the host (see Firecracker
  guide). Vsock is often used for host↔guest control channels (Cloud Hypervisor
  `--vsock …`).
- Consoles differ: Firecracker typically uses `ttyS0`; Cloud Hypervisor often
  uses `virtio-console`/`hvc0` (or `--serial tty` + `console=ttyS0`); QEMU
  microvm can use `ttyS0` or `hvc0` depending on devices.

## How are images for them generated?

At minimum you need a kernel image appropriate for the VMM plus a root
filesystem image (or initrd) with an `init` and the software you want to run.

- Firecracker
  - Kernel: x86‑64 requires an uncompressed ELF `vmlinux`; aarch64 uses PE
    (`Image`). Supported/validated guest kernels include LTS 5.10 and 6.1 (see
    kernel support policy). See: [Kernel policy](https://github.com/firecracker-microvm/firecracker/blob/main/docs/kernel-policy.md).
  - Rootfs: typically an ext4 image. You can create one by formatting a sparse
    file and populating it (e.g., via `debootstrap`, `buildroot`, or by expanding a
    squashfs into ext4). The Firecracker docs and community guides show both manual
    and scripted approaches.
    See: [Getting Started](https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md) and [Custom rootfs guide](https://jonathanwoollett-light.github.io/firecracker/book/book/rootfs-and-kernel-setup.html).

- Cloud Hypervisor
  - Direct kernel boot: prefers a `vmlinux` with PVH support (or a bzImage) and
    a raw disk image for the root filesystem.
  - Firmware boot: boot a standard cloud image (e.g., Ubuntu cloud image) using
    Rust Hypervisor Firmware or edk2 `CLOUDHV` firmware; seed first boot with a
    small cloud‑init ISO.
    See: [Quick Start](https://www.cloudhypervisor.org/docs/prologue/quick-start/).

- QEMU “microvm” machine type
  - Boot requires a host‑provided kernel (e.g., `-kernel vmlinux -append
"root=/dev/vda …"`) and a raw disk image; attach devices as `virtio-mmio`.
    QEMU’s docs include complete examples.
    See: [QEMU microvm](https://www.qemu.org/docs/master/system/i386/microvm.html).

- Kata Containers guest image
  - Use osbuilder to build a minimal rootfs with the `kata-agent` and either
    package it as a disk image or use an initrd. Multiple base distributions are
    supported; extra packages may be added via `EXTRA_PKGS`.
    See: [Kata osbuilder](https://github.com/kata-containers/kata-containers/tree/main/tools/osbuilder/rootfs-builder).

- Ignite (OCI → microVM)
  - Ignite can derive a VM rootfs from an OCI/Docker image, then boot it with
    Firecracker; it can also take a kernel from an OCI image at `/boot/vmlinux`.
    This enables a “container‑like” packaging path for microVM images.
    See: [Ignite README](https://github.com/weaveworks/ignite), [Igniteusage](https://ignite.readthedocs.io/en/stable/usage/).

Design note — one shared image generation pipeline for Lima VMs, MicroVMs,
and DevContainers

We can define our software set once and emit multiple formats:

- Use Nix to describe the packages and system config, then build:
  - DevContainer layers: build an OCI image from the same Nix inputs (e.g.,
    `dockerTools.buildImage` or `nix2container`), aligning with our [DevContainer Layered Images](../Public/Nix%20Devcontainer/Devcontainer%20Design.md#Layered%20Images).
  - Lima VM images: generate raw/QCOW2 NixOS disk images with
    `nixos-generators` using the same config; Lima can boot them via QEMU on macOS.
    [nixos‑generators]
  - MicroVM images: reuse the same NixOS config to produce raw disk images for
    Cloud Hypervisor/QEMU microvm or a Firecracker‑ready rootfs+kernel. The
    `microvm.nix` project lets you target Firecracker, Cloud Hypervisor, QEMU
    microvm, crosvm, and kvmtool from the same Nix flake, and can build read‑only
    root images (squashfs/erofs) with host `/nix/store` sharing if desired.
    [microvm.nix intro] [microvm options]

References

- [nixos‑generators (generate raw/qcow2/… images from one config)](https://github.com/nix-community/nixos-generators)
- [microvm.nix (define and run NixOS‑based MicroVMs across multiple hypervisors)](https://microvm-nix.github.io/microvm.nix/)

This approach satisfies the “single source of truth” requirement: the same
package list and base config produce (a) layered OCI images for DevContainers,
(b) Lima VM disk images, and (c) MicroVM images for Firecracker/Cloud
Hypervisor/QEMU microvm.

Ideally, we would have a shared image generation process allow the same list of
software packages to be specified for our [Lima VM Images](../Public/Lima%20VM%20Images.md) and our [DevContainer Layered Images](../Public/Nix%20Devcontainer/Devcontainer%20Design.md#Layered%20Images).

## Are there up-to-date benchmarks showing how the start-up times of various

MicroVMs compare?

Short answer: There are credible public benchmarks, but truly
apples‑to‑apples, regularly updated comparisons are rare. The best
available sources show that Firecracker and Cloud Hypervisor both achieve
sub‑second Linux cold boots under minimal configurations, often hundreds of
milliseconds; QEMU’s `microvm` machine type narrows the gap substantially
versus “full” QEMU, and unikernels can boot in the single‑digit
millisecond range. Snapshot/restore (“microVM resume”) is dramatically
faster than cold boot for all VMMs.

Highlights from primary sources and recent reports

- Firecracker (cold boot): The project advertises initiating userspace in “as
  little as 125 ms.” Independent reports commonly land in the ~150–800 ms
  range depending on kernel/userspace and device setup (e.g., ~800 ms in a demo
  repo issue; Alpine Linux ~330 ms in Unikraft’s evaluation). Sources:
  [Firecracker site](https://firecracker-microvm.github.io/), [AWSblog](https://aws.amazon.com/blogs/aws/firecracker-lightweight-virtualization-for-serverless-computing/), [community issue](https://github.com/firecracker-microvm/firecracker-demo/issues/44),[Unikraft performance](https://unikraft.org/docs/concepts/performance).
- Cloud Hypervisor (cold boot): Boot‑time tracking shows “VMM start →
  userspace” around 200 ms in repeated runs on release builds, with kernel
  entry near ~30 ms. See:
  [cloud-hypervisor#1728](https://github.com/cloud-hypervisor/cloud-hypervisor/issues/1728).
- QEMU “microvm” (context): Upstream QEMU documents the minimalist
  `microvm` machine type (no PCI/ACPI; virtio‑mmio). Unikernel studies report
  total boot around ~10 ms on QEMU microVM for a hello‑world guest. Links:
  [QEMU microvm docs](https://www.qemu.org/docs/master/system/i386/microvm.html),[Unikraft performance](https://unikraft.org/docs/concepts/performance).
- Unikernels (best case): Unikraft reports booting off‑the‑shelf apps in a
  few milliseconds on Firecracker and QEMU (guest‑only ~µs to 1 ms; total
  VMM+guest single‑digit ms).
  Links: [Unikraft performance](https://unikraft.org/docs/concepts/performance), [EuroSys’21abstract](https://arxiv.org/abs/2104.12721).

What to watch for when comparing numbers

- Definition of “boot”: Some measure “VMM start → kernel entry”,
  others “VMM start → first userspace process”, others
  “SSH‑reachable.” Device count (net/blk/console/vsock) and firmware vs
  direct‑kernel boot materially change results. References:
  [cloud-hypervisor#1728](https://github.com/cloud-hypervisor/cloud-hypervisor/issues/1728), [QEMU microvm docs](https://www.qemu.org/docs/master/system/i386/microvm.html).- Guest OS: BusyBox/Buildroot or stripped NixOS boots much faster than Ubuntucloud images; unikernels are faster still but not directly comparable to ageneral‑purpose distro. Reference: [Unikraftperformance](https://unikraft.org/docs/concepts/performance).
- Snapshots vs cold boot: Resume from a snapshot is dramatically faster.
  Example: Fly.io Machines suspend/resume uses Firecracker snapshots to resume in
  “hundreds of milliseconds” vs multi‑second cold starts; Firecracker
  snapshot docs cover performance considerations and caveats.
  Links: [Fly.io suspend/resume](https://fly.io/docs/reference/suspend-resume/), [Firecrackersnapshotsupport](https://github.com/firecracker-microvm/firecracker/blob/main/docs/snapshotting/snapshot-support.md).

Bottom line

- For minimal Linux guests, both Firecracker and Cloud Hypervisor can achieve
  sub‑second cold boots on modern hardware; precise numbers depend heavily on
  kernel config, devices, and what “boot complete” means. QEMU’s `microvm`
  narrows the gap substantially versus traditional QEMU, and unikernels
  demonstrate the lower bound. For production “time‑to‑first‑request,”
  snapshot/restore often matters more than raw cold‑boot speed.
