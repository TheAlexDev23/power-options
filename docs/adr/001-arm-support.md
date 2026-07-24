---
id: adrs-adr001
title: "ADR001: Multi-Architecture Support (x86_64 + aarch64)"
# prettier-ignore
description: Architecture Decision Record (ADR) for adding ARM (aarch64) support to Power Options distribution
---

## Context

Power Options is a pure Rust project with zero architecture-specific code — no `#[cfg(target_arch)]`, no arch-dependent dependencies, no inline assembly. The codebase compiles cleanly on any CPU architecture supported by Rust's tier 1 and tier 2 targets.

Despite this, the project's distribution infrastructure exclusively targets x86_64:

- **CI pipelines** (`deploy_release.yaml`, `deploy_commit.yaml`) run only on `ubuntu-24.04` (x86_64) runners.
- **AUR PKGBUILD generators** hardcode `arch=('x86_64')` in all 8 generators (daemon, gtk, webview, tray × stable + git variants).
- **DistroPack uploads** provide only x86_64 binaries.
- **OBS Debian control file** correctly uses `Architecture: any`, but CI never builds for non-x86 targets.

ARM-based devices (e.g., Raspberry Pi, Apple Silicon via Asahi Linux, Snapdragon X Elite laptops, server-grade Ampere Altra) are increasingly common Linux platforms. Users on these systems are forced to build from source manually, which works but defeats the purpose of the packaged distribution.

A user on an aarch64 system attempting `apt install power-options-gtk` receives:

```
power-options-gtk:amd64 : Depends: libadwaita-1-0:amd64 but it is not installable
```

This is because only an amd64 (x86_64) binary exists in the DistroPack repository — APT tries to satisfy it via multiarch on a system with no x86_64 userspace.

## Decision

We will add aarch64 as a supported architecture across all distribution channels:

1. **CI/CD**: Convert the release build job to a strategy matrix running on both `ubuntu-24.04` and `ubuntu-24.04-arm` GitHub-hosted runners. The commit check job will also run on both architectures.

2. **DistroPack**: Upload architecture-specific binaries from each matrix job. We will use distinct package IDs per architecture to keep package metadata clean.

3. **AUR PKGBUILDs**: Update all 8 PKGBUILD generators to set `arch=('x86_64' 'aarch64')`, allowing Arch Linux ARM (asahi-linux, Arch Linux ARM) users to install from the AUR.

4. **OBS / Debian packaging**: No changes needed — `Architecture: any` in `debian.control` and the absence of arch-specific requirements in the RPM `.spec` already permit multi-arch builds. The OBS build service will handle cross-architecture builds when configured.

5. **Install scripts**: No changes needed — they invoke `cargo build --release` natively on whatever architecture the user is on.

We intentionally do **not** add cross-compilation tooling (e.g., `cross`, `cargo-zigbuild`, QEMU binfmt) at this time. Using native ARM GitHub runners is simpler, faster, and avoids the complications of cross-compiling GTK4/libadwaita/WebKit system dependencies. If native ARM runners prove unavailable or too slow in the future, cross-compilation can be revisited.

## Consequences

### Positive
- ARM Linux users can install Power Options via apt, AUR, and OBS without building from source.
- The Rust codebase needs no changes — a testament to Rust's portability.
- GitHub's ARM runners are available at no additional cost for public repositories.
- The build matrix approach is easy to understand and extend (e.g., adding riscv64 in the future).

### Negative
- CI runtime approximately doubles (two parallel builds instead of one).
- DistroPack package management becomes slightly more complex with per-arch package IDs.
- AUR PKGBUILD `arch` arrays become slightly longer (all 8 files need `('x86_64' 'aarch64')`).

### Neutral
- The check_and_format job in commit CI will also run on ARM, catching any future architecture-specific regressions.
- The webview frontend (dioxus/webkit2gtk) builds on ARM where webkit2gtk is available — no additional work needed.
- Release tags now produce two sets of binaries. We must verify DistroPack processes both correctly before the first multi-arch release.
