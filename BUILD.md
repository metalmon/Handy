# Build Instructions

This guide covers how to set up the development environment and build Handy from source across different platforms.

## Prerequisites

### All Platforms

- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) package manager
- [Tauri Prerequisites](https://tauri.app/start/prerequisites/)

### Platform-Specific Requirements

> [!NOTE]
> This fork's CI ships **Windows** (NSIS + MSI) and **Linux** (deb + rpm, built on
> `ubuntu-22.04`) only. There is no macOS job in any workflow, so the macOS
> instructions below are for local development and are not exercised by CI.

#### macOS

- Xcode Command Line Tools
- Install with: `xcode-select --install`

##### Intel Mac (x86_64)

Prebuilt ONNX Runtime binaries are not available for Intel Macs. Install ONNX Runtime via Homebrew and link dynamically:

```bash
brew install onnxruntime
ORT_LIB_LOCATION=$(brew --prefix onnxruntime)/lib ORT_PREFER_DYNAMIC_LINK=1 bun run tauri dev
```

The same environment variables apply for production builds:

```bash
ORT_LIB_LOCATION=$(brew --prefix onnxruntime)/lib ORT_PREFER_DYNAMIC_LINK=1 bun run tauri build
```

#### Windows

- Microsoft C++ Build Tools: Visual Studio 2019/2022 with C++ development
  tools, or Visual Studio Build Tools 2019/2022
- [CMake](https://cmake.org/download/) (must be on `PATH`):

  ```powershell
  winget install Kitware.CMake
  ```

- [Vulkan SDK](https://vulkan.lunarg.com/sdk/home) from LunarG — required to
  build the Vulkan GPU backend (`vulkan-shaders-gen` needs the SDK's headers
  and `glslc`):

  ```powershell
  winget install KhronosGroup.VulkanSDK
  ```

  Open a new terminal afterward so `VULKAN_SDK` is set.

> [!NOTE]
> Windows' 260-character path limit used to break the native Vulkan build in
> most checkouts. Since `transcribe-cpp` 0.1.3 the build works around it
> automatically (it compiles through a short NTFS junction — no admin rights
> or setup needed), so a normal checkout just builds. If you still hit
> path-limit errors, see
> [Windows build fails with path-limit errors](#windows-build-fails-with-path-limit-errors-msb3491--ftk1011--msb6003)
> in Troubleshooting.

#### Linux

- Build essentials
- ALSA development libraries
- Install with:

  ```bash
  # Ubuntu/Debian
  sudo apt update
  sudo apt install build-essential clang libclang-dev libevdev-dev libasound2-dev pkg-config libssl-dev libvulkan-dev vulkan-tools glslc spirv-headers glslang-tools libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libgtk-layer-shell0 libgtk-layer-shell-dev patchelf cmake

  # Fedora/RHEL
  sudo dnf groupinstall "Development Tools"
  sudo dnf install alsa-lib-devel pkgconf openssl-devel vulkan-devel glslc \
    clang clang-devel libevdev-devel \
    spirv-headers-devel spirv-tools-devel glslang \
    gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel \
    gtk-layer-shell gtk-layer-shell-devel \
    cmake

  # Arch Linux
  sudo pacman -S base-devel clang libevdev shaderc spirv-headers glslang alsa-lib pkgconf openssl vulkan-devel \
    gtk3 webkit2gtk-4.1 libappindicator-gtk3 librsvg gtk-layer-shell \
    cmake
  ```

## Setup Instructions

### 1. Clone the Repository

```bash
git clone git@github.com:cjpais/Handy.git
cd Handy
```

### 2. Install Dependencies

```bash
bun install
```

### 3. Start Dev Server

```bash
bun tauri dev
```

### 4. Build for Production

```bash
bun run tauri build
```

This compiles a release binary and generates platform-specific bundles
(deb, rpm and AppImage on Linux; app and dmg on macOS; NSIS and MSI on Windows).

### 5. The Bundled Model

There is nothing to download by hand. `beforeBuildCommand` runs
`bun run fetch:model` first, which downloads the GigaAM v3 CTC model
(~152 MB compressed, ~215 MB unpacked), verifies its SHA-256, and unpacks it
into `src-tauri/resources/models/giga-am-v3-int8/`. That directory is git-ignored.

That is the only model fetched at build time. The Silero VAD model
(`src-tauri/resources/models/silero_vad_v4.onnx`, ~1.7 MB) is committed to the
repository and is bundled from the checkout as-is.

The script is idempotent: once `model.int8.onnx` is present it prints
`already present, skipping download` and never touches the network, so an
offline rebuild works after the first fetch. To fetch it without a full build:

```bash
bun run fetch:model
```

Because the model ships inside the bundle, a Windows installer is roughly
165 MB and a release build takes noticeably longer than it used to.

## Building Linux Packages

deb, rpm and AppImage can only be produced on a Linux host. From Windows or
macOS, use WSL2 or a Linux container with the dependencies listed under
[Linux](#linux) above.

No host packaging tool is needed beyond that list. `tauri-bundler` writes both
formats itself, in-process and in pure Rust: the deb via `flate2` + `tar` +
`ar`, the rpm via the `rpm-rs` crate. Nothing shells out to `dpkg-deb`,
`fakeroot` or `rpmbuild`. `dpkg-dev` and `rpm` are only needed to **inspect** a
finished package (`dpkg-deb -c`, `rpm -qpl`), which is what CI does once
bundling has produced it.

### What CI verifies about Linux packages

Every Linux build audits the package contents before the job can go green: the
"Audit Linux package runtime contents" step in `.github/workflows/build.yml` runs
on all four workflows that build Linux (`build-test`, `main-build`,
`pr-test-build`, `release`). For each `.deb` in `bundle/deb/` it requires, from
the `dpkg-deb -c` listing:

- `usr/bin/handy`
- `usr/lib/Handy/libtranscribe.so` (matched by SONAME) and the `libggml-cpu`
  backend module
- `usr/lib/Handy/libonnxruntime.so.1` on hosts that link ONNX Runtime
  dynamically — which includes the `ubuntu-22.04` build this fork ships
- `usr/lib/Handy/resources/models/giga-am-v3-int8/model.int8.onnx`, the matching
  `vocab.txt`, and `usr/lib/Handy/resources/models/silero_vad_v4.onnx`
- no library installed directly into `/usr/lib` (the issue #1639 guard)

It then unpacks the deb with `dpkg-deb -x` and launches the packaged binary
under `xvfb-run handy --list-devices`, so a package that installs but cannot
start fails the build. Each `.rpm` in `bundle/rpm/` gets the same listing
assertions via `rpm -qpl` — including the ONNX Runtime library, which is why the
rpm needs its own `files` entry for it — but is **not** launched.

Independently, the same step caps the executable's glibc requirement: the highest
`GLIBC_x.y` symbol version `handy` requires must not exceed `2.35`, the
`ubuntu-22.04` baseline. Both shipping targets, Astra Linux SE 1.8 and РЕДОС 8,
ship glibc 2.36, and glibc is forward-compatible only.

So the bundled model is no longer merely **expected** at
`<install>/lib/Handy/resources/models/giga-am-v3-int8/` on the strength of
reading `tauri-bundler`'s source — the path is asserted on every run. To re-check
a local build:

```bash
dpkg-deb -c src-tauri/target/release/bundle/deb/*.deb | grep -i "models/"
```

What CI still does **not** cover: no release has been published from this
pipeline yet, and neither Astra Linux SE 1.8 nor РЕДОС 8 has been installed and
tested by hand — CI builds and audits on `ubuntu-22.04` only. The glibc cap
covers the `handy` executable, not the prebuilt `libonnxruntime.so.1`. A green
run therefore bounds the packages' behaviour on the CI host, not on the target
distributions.

## Linux Install (from source)

The raw binary (`src-tauri/target/release/handy`) cannot run standalone — it needs Tauri resource files (tray icons, sounds, VAD model) to be co-located at the expected path.

**Install from the deb bundle** (works on any Linux distro):

```bash
cd /tmp
ar x /path/to/Handy/src-tauri/target/release/bundle/deb/Handy_*_amd64.deb data.tar.gz
tar xzf data.tar.gz
sudo cp usr/bin/handy /usr/bin/
sudo cp -a usr/lib/. /usr/lib/
sudo cp -r usr/share/icons/hicolor/* /usr/share/icons/hicolor/
sudo cp usr/share/applications/Handy.desktop /usr/share/applications/
```

The runtime libraries live in the app-private `/usr/lib/Handy/` (on the binary's rpath), so no `ldconfig` step is needed.

After subsequent rebuilds, copy the binary and any refreshed runtime libraries:

```bash
sudo cp src-tauri/target/release/handy /usr/bin/
sudo mkdir -p /usr/lib/Handy
sudo cp -a src-tauri/transcribe-libs/. /usr/lib/Handy/
```

Resources only need re-copying if they change upstream (new icons, sounds, models, etc.).

## Troubleshooting

### macOS Accessibility remains enabled after a local rebuild

Local builds use the ad-hoc `signingIdentity: "-"`. A rebuild can have a new macOS code
identity while the old **System Settings > Privacy & Security > Accessibility** entry
remains visibly enabled, leaving Handy on `Waiting...`.

After installing the final bundle at `/Applications/Handy.app`, quit Handy, clear only its
stale Accessibility record, then reopen it:

```bash
osascript -e 'tell application id "com.pais.handy" to quit' || true
tccutil reset Accessibility com.pais.handy
open /Applications/Handy.app
```

Grant Accessibility again when prompted. This does not reset Microphone or other TCC
services, and official releases normally do not need it.

For optional diagnosis, compare the designated requirements of the previous and rebuilt
bundles:

```bash
codesign -dr - /path/to/previous/Handy.app 2>&1
codesign -dr - /Applications/Handy.app 2>&1
```

An ad-hoc requirement contains a `cdhash`; a changed requirement confirms the rebuild is
not covered by the old grant. The reset procedure does not require this check.

See [issue #1618](https://github.com/cjpais/Handy/issues/1618) for the related onboarding
and stale-permission report.

### AppImage build fails on Arch / rolling-release distros

> [!NOTE]
> This fork's CI passes `--bundles deb,rpm` and never builds an AppImage. What
> follows applies to a local `bun run tauri build`, which uses
> `targets: "all"` from `tauri.conf.json` and so attempts all three.

`linuxdeploy` bundles its own `strip` binary which is too old to process system libraries built with newer toolchains on rolling-release distros (Arch, CachyOS, Manjaro, EndeavourOS).

The error from Tauri:

```
Bundling Handy_*_amd64.AppImage
failed to bundle project `failed to run linuxdeploy`
```

Tauri swallows the real linuxdeploy error. To see it, run linuxdeploy manually:

```bash
cd src-tauri/target/release/bundle/appimage
~/.cache/tauri/linuxdeploy-x86_64.AppImage --appimage-extract-and-run \
  --appdir Handy.AppDir --plugin gtk --output appimage
```

**Workaround:** The binary, deb, and rpm bundles all build fine — only the AppImage step fails. To skip it:

```bash
bun run tauri build -- --bundles deb
```

Then install using the deb extraction method above.

### Windows build fails with path-limit errors (`MSB3491` / `FTK1011` / `MSB6003`)

On Windows the native build can fail partway through `transcribe-cpp-sys` with
any of these (all the same root cause):

```
error MSB3491: Could not write lines to file "...VCTargetsPath.tlog\VCTargetsPath.lastbuildstate".
Path: ... exceeds the OS max path limit. The fully qualified file name must be less than 260 characters.
```

```
FileTracker : error FTK1011: could not create the new file tracking log file:
...\vulkan-shaders-gen-build\...\cmTC_xxxxx.tlog\link.write.1.tlog.
The system cannot find the path specified.
```

```
error MSB6003: The specified task executable "CL.exe" could not be run.
System.IO.DirectoryNotFoundException: Could not find a part of the path ...
```

This is **not** a code or toolchain problem — it's Windows' legacy 260-character
path limit (`MAX_PATH`), overflowed by the Vulkan shader generator's nested
CMake build tree on top of Cargo's already-deep
`target\release\build\<crate>-<hash>\out\build\...` directory.

Since `transcribe-cpp` 0.1.3 this is mitigated automatically: the native build
compiles through a short NTFS junction under `%LOCALAPPDATA%\tcs` (created
without admin rights), so a normal checkout builds with no setup. Enabling
Windows long paths does **not** reliably help here — MSBuild's native
`FileTracker` (`tracker.exe`) ignores the long-paths flag — which is why the
junction, not the registry flag, is the fix.

If you still see the errors above, junction creation was likely blocked
(filesystem or corporate policy) — the failing build's log then contains a
`transcribe-cpp-sys: could not create short build junction ...` warning — or
your checkout is deep enough to overflow even the shortened layout. Work
around either case with a short Cargo target directory:

```powershell
# Per-shell:
$env:CARGO_TARGET_DIR = "C:\h"

# Or persist it for all future terminals (note: redirects ALL your
# Rust projects' build output, not just Handy):
[Environment]::SetEnvironmentVariable('CARGO_TARGET_DIR', 'C:\h', 'User')
```

Artifacts then land in `C:\h\release\...` instead of the repo's
`src-tauri\target\`. Open a **new terminal** if you persisted the variable —
it is only picked up by freshly started processes. Then `bun run tauri dev`
and `bun run tauri build` work normally.

### Release artifacts are unsigned, and Windows warns about them

This fork has no code-signing certificate, and `src-tauri/tauri.conf.json` asks for
no signing step at all: there is no `signCommand`, no `certificateThumbprint` and
no `timestampUrl` under `bundle.windows`, so `tauri build` skips signing on every
host — developer machine and CI alike. The `sign-binaries` input in
`.github/workflows/build.yml` defaults to `false` and `release.yml` passes
`false` explicitly; the Apple certificate import steps hang off that input and
never execute.

There is no override to add. A plain `bun run tauri build` produces an installer,
and `bun run tauri build --bundles nsis` narrows it to a single one.

What that means in practice: a Handy installer carries no Authenticode signature
and no publisher name. On Windows that surfaces as a SmartScreen interstitial —
"Windows protected your PC" on the NSIS `.exe`, an unknown-publisher prompt on
the MSI. That warning is a consequence of the artifact being unsigned, not
evidence of a tampered download.

To install past it, use the **More info** → **Run anyway** link on the SmartScreen
block page.

> [!NOTE]
> No checksum or signature file is published alongside the artifacts — no
> `.sig`, no `.minisig`, no hash list — so there is nothing in-band to verify a
> download against. Establish provenance out of band if that matters to you.
