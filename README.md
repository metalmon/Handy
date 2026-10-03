# Handy

[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?style=for-the-badge&logo=discord&logoColor=white)](https://discord.com/invite/WVBeWsNXK4)

**A free, open source, and extensible speech-to-text application that works completely offline.**

Handy is a cross-platform desktop application that provides simple, privacy-focused speech transcription. Press a shortcut, speak, and have your words appear in any text field. This happens on your own computer without sending any information to the cloud.

## Why Handy?

Handy was created to fill the gap for a truly open source, extensible speech-to-text tool. As stated on [handy.computer](https://handy.computer):

- **Free**: Accessibility tooling belongs in everyone's hands, not behind a paywall
- **Open Source**: Together we can build further. Extend Handy for yourself and contribute to something bigger
- **Private**: Your voice stays on your computer. Get transcriptions without sending audio to the cloud
- **Simple**: One tool, one job. Transcribe what you say and put it into a text box

## How It Works

1. **Press** a configurable keyboard shortcut: hold it to record and release to stop, or tap it to toggle recording on and off (Hold-only and Toggle-only modes are also available)
2. **Speak** your words while the shortcut is active
3. **Release** and Handy processes your speech with the bundled GigaAM v3 model
4. **Get** your transcribed text pasted directly into whatever app you're using

The process is entirely local:

- Silence is filtered using VAD (Voice Activity Detection) with Silero
- Transcription runs on a **GigaAM v3** model that is bundled inside the app —
  nothing is downloaded at runtime and the app works fully offline
- Works on Windows, macOS, and Linux. This fork builds installers for Windows
  and Linux (x86-64) only; see [Installation](#installation)

## Quick Start

### Installation

This fork builds **Windows** and **Linux** x86-64 installers: an NSIS `.exe` and
an MSI for Windows, a `.deb` and an `.rpm` for Linux. macOS is not built by CI
here — build it from source instead, see [BUILD.md](BUILD.md).

> **No release has been published from this fork's pipeline yet.** Until the first
> one exists, [build Handy from source](BUILD.md).

1. Download the installer for your platform from the
   [releases page](https://github.com/metalmon/Handy/releases)
   - **Windows**: The NSIS installer and the MSI install the same application,
     but they label it differently — NSIS shows **ХЭНДИ**, the MSI shows ASCII
     **Handy**. Both put `handy.exe` in a `Handy` folder under `Program Files`.
     Neither is code-signed, so Windows warns about the download; see
     [BUILD.md](BUILD.md#release-artifacts-are-unsigned-and-windows-warns-about-them)
     for what to click to get past it.
   - **Linux**: Install the downloaded package with your distribution's package
     manager so the runtime dependencies are resolved. Run this from the folder
     the package was downloaded into:

     ```bash
     # Astra Linux, Debian, Ubuntu
     sudo apt install ./*.deb

     # РЕДОС, Fedora, RHEL
     sudo dnf install ./*.rpm
     ```

     Do not use `dpkg -i` or `rpm -i` unless those dependencies are already
     installed; if you already used them, run `sudo apt --fix-broken install`.

2. Install the application
3. Launch Handy and grant necessary system permissions (microphone, accessibility)
4. Configure your preferred keyboard shortcuts in Settings
5. Start transcribing!

> **The Homebrew cask and the winget package install upstream builds**, not this
> fork's: `brew install --cask handy` and `winget install cjpais.Handy` fetch
> releases from `cjpais/Handy`. Neither is maintained by the Handy developers.

### Astra Linux SE 1.8 and РЕДОС 8

These are the distributions this fork's packages are built for:

| Distribution       | Package | Base                  |
| ------------------ | ------- | --------------------- |
| Astra Linux SE 1.8 | `.deb`  | Debian 12, glibc 2.36 |
| РЕДОС 8            | `.rpm`  | glibc 2.36            |

The packages themselves are built on `ubuntu-22.04` (glibc 2.35). CI fails the
build if the `handy` executable ever requires a newer glibc than the build
host's, which is what keeps the artifacts loadable on both targets.

Everything Handy needs is inside the package, so installation needs no network
access: the executable goes to `/usr/bin/handy`, its private runtime libraries
and the models (GigaAM v3 and Silero VAD, under
`/usr/lib/Handy/resources/models/`) go to `/usr/lib/Handy/`, and a desktop entry
named **ХЭНДИ** is added to your application menu.

On РЕДОС the RPM dependencies must be resolvable — the package requires
`libgtk-layer-shell.so.0()(64bit)` and `libopenblas.so.0()(64bit)`. Install the
packages that provide those two libraries first if the installer cannot resolve
them (`gtk-layer-shell` and `openblas` on RPM-based distributions). See
[Linux Notes](#linux-notes) for the runtime-library table.

> **Untested on this fork:** nobody has installed these packages on Astra Linux
> SE 1.8 or РЕДОС 8 by hand yet. CI builds and audits them on `ubuntu-22.04`
> only, which bounds their behaviour on the build host and not on the target
> distributions. Please report what breaks.

### Development Setup

For detailed build instructions including platform-specific requirements, see [BUILD.md](BUILD.md).

## Sponsors

<div align="center">
  We're grateful for the support of our sponsors who help make Handy possible:
  <br><br>
  <a href="https://wordcab.com">
    <img src="sponsor-images/wordcab.png" alt="Wordcab" width="120" height="120">
  </a>
  &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
  <a href="https://github.com/epicenter-so/epicenter">
    <img src="sponsor-images/epicenter.png" alt="Epicenter" width="120" height="120">
  </a>
  &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
  <a href="https://boltai.com?utm_source=handy">
    <img src="sponsor-images/boltai.jpg" alt="Bolt AI" width="120" height="120">
  </a>
  &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
  <a href="https://cantydigital.com.au/">
    <img src="sponsor-images/cantydigital.png" alt="Canty Digital" width="120" height="120">
  </a>
</div>

## Debug Mode

Handy includes an advanced debug mode for development and troubleshooting. Access it by pressing:

- **macOS**: `Cmd+Shift+D`
- **Windows/Linux**: `Ctrl+Shift+D`

## CLI Parameters

Handy supports command-line flags for controlling a running instance and customizing startup behavior. These work on all platforms (macOS, Windows, Linux). Largely this is a beta feature.

**Remote control flags** (sent to an already-running instance via the single-instance plugin):

```bash
handy --toggle-transcription    # Toggle recording on/off
handy --toggle-post-process     # Toggle recording with post-processing on/off
handy --cancel                  # Cancel the current operation
```

**Startup flags:**

```bash
handy --start-hidden            # Start without showing the main window
handy --no-tray                 # Start without the system tray icon
handy --debug                   # Enable debug mode with verbose logging
handy --help                    # Show all available flags
```

Flags can be combined for autostart scenarios:

```bash
handy --start-hidden --no-tray
```

> **macOS tip:** When Handy is installed as an app bundle, invoke the binary directly:
>
> ```bash
> /Applications/Handy.app/Contents/MacOS/Handy --toggle-transcription
> ```

## Known Issues & Current Limitations

This fork is a distribution of [upstream Handy](https://github.com/cjpais/Handy): the speech model is bundled for offline use, the in-app updater is gone, and the packages are unsigned. Report problems with this build in this repository's [issue tracker](https://github.com/metalmon/Handy/issues). We believe in transparency about the current state:

### Bluetooth Headset Microphones (macOS)

Using a Bluetooth headset microphone on macOS may temporarily reduce playback quality or volume while recording because Bluetooth switches to bidirectional audio. Keep your headphones as the output device and select your Mac's built-in or an external microphone in Handy to avoid this.

### fn and Globe Key Shortcuts (macOS)

Shortcuts that include the `fn` (Globe) key **only work on Apple keyboards** — your Mac's built-in keyboard or an Apple external keyboard. They will never trigger on a third-party keyboard, even while it is connected to the same Mac.

This is a hardware limitation rather than a Handy bug. `fn` is not part of the standard USB HID keyboard specification: Apple reports it through a vendor-specific usage that macOS honors only from Apple devices, while third-party keyboards handle their `Fn` key entirely in firmware and send nothing to the computer. There is no event for Handy to listen for.

If you switch between a MacBook keyboard and an external one, pick a shortcut built from standard modifiers (`ctrl`, `option`, `shift`, `command`) or a regular key instead.

### Linux Notes

**Text Input Tools:**

For reliable text input on Linux, install the appropriate tool for your display server:

| Display Server | Recommended Tool | Install Command                                    |
| -------------- | ---------------- | -------------------------------------------------- |
| X11            | `xdotool`        | `sudo apt install xdotool`                         |
| Wayland        | `wtype`          | `sudo apt install wtype`                           |
| Both           | `dotool`         | `sudo apt install dotool` (requires `input` group) |

- **X11**: Install `xdotool` for both direct typing and clipboard paste shortcuts
- **Ubuntu 26.04**: Has Wayland display server by default. `wtype` does not work, you need to install `ydotool` and configure systemd as described [here](https://github.com/cjpais/Handy/pull/557#issuecomment-3781249267).
- **Wayland**: Install `wtype` (preferred) or `dotool` for text input to work correctly
- **dotool setup**: Requires adding your user to the `input` group: `sudo usermod -aG input $USER` (then log out and back in)

Without these tools, Handy falls back to enigo which may have limited compatibility, especially on Wayland.

**Wayland Support (Linux):**

- Limited support for Wayland display server
- Requires [`wtype`](https://github.com/atx/wtype) or [`dotool`](https://sr.ht/~geb/dotool/) for text input to work correctly (see [Linux Notes](#linux-notes) below for installation)

**Other Notes:**

- **Runtime library dependency (`libgtk-layer-shell.so.0`)**:
  - Handy links `gtk-layer-shell` on Linux. If startup fails with `error while loading shared libraries: libgtk-layer-shell.so.0`, install the runtime package for your distro:

    | Distro        | Package to install    | Example command                        |
    | ------------- | --------------------- | -------------------------------------- |
    | Ubuntu/Debian | `libgtk-layer-shell0` | `sudo apt install libgtk-layer-shell0` |
    | Fedora/RHEL   | `gtk-layer-shell`     | `sudo dnf install gtk-layer-shell`     |
    | Arch Linux    | `gtk-layer-shell`     | `sudo pacman -S gtk-layer-shell`       |

  - For building from source on Ubuntu/Debian, you may also need `libgtk-layer-shell-dev`.

- The recording overlay is disabled by default on Linux (`Overlay Position: None`) because certain compositors treat it as the active window. When the overlay is visible it can steal focus, which prevents Handy from pasting back into the application that triggered transcription. If you enable the overlay anyway, be aware that clipboard-based pasting might fail or end up in the wrong window.
- If you are having trouble with the app, running with the environment variable `WEBKIT_DISABLE_DMABUF_RENDERER=1` may help
- If Handy fails to start reliably on Linux, see [Troubleshooting → Linux Startup Crashes or Instability](#linux-startup-crashes-or-instability).
- **Global keyboard shortcuts (Wayland):** On Wayland, system-level shortcuts must be configured through your desktop environment or window manager. Use the [CLI flags](#cli-parameters) as the command for your custom shortcut.

  **GNOME:**
  1. Open **Settings > Keyboard > Keyboard Shortcuts > Custom Shortcuts**
  2. Click the **+** button to add a new shortcut
  3. Set the **Name** to `Toggle Handy Transcription`
  4. Set the **Command** to `handy --toggle-transcription`
  5. Click **Set Shortcut** and press your desired key combination (e.g., `Super+O`)

  **KDE Plasma:**
  1. Open **System Settings > Shortcuts > Custom Shortcuts**
  2. Click **Edit > New > Global Shortcut > Command/URL**
  3. Name it `Toggle Handy Transcription`
  4. In the **Trigger** tab, set your desired key combination
  5. In the **Action** tab, set the command to `handy --toggle-transcription`

  **Sway / i3:**

  Add to your config file (`~/.config/sway/config` or `~/.config/i3/config`):

  ```ini
  bindsym $mod+o exec handy --toggle-transcription
  ```

  **Hyprland:**

  Add to your config file (`~/.config/hypr/hyprland.conf`):

  ```ini
  bind = $mainMod, O, exec, handy --toggle-transcription
  ```

- You can also trigger Handy externally via Unix signals or the CLI flags, which lets Wayland window managers or other hotkey daemons keep ownership of keybindings:

  | Action                                    | Trigger                                                  |
  | ----------------------------------------- | -------------------------------------------------------- |
  | Toggle transcription                      | `pkill -USR2 -n handy` or `handy --toggle-transcription` |
  | Toggle transcription with post-processing | `handy --toggle-post-process`                            |

  Example Sway config:

  ```ini
  bindsym $mod+o exec pkill -USR2 -n handy
  bindsym $mod+p exec handy --toggle-post-process
  ```

  `pkill` here simply delivers the signal—it does not terminate the process.

  > **Behavior change:** older releases also accepted `SIGUSR1` for toggling transcription with post-processing. WebKitGTK — the webview engine embedded in Handy on Linux — uses SIGUSR1 internally to coordinate JavaScript garbage collection, so listening for it caused phantom recordings and interrupted dictations every few minutes ([#1660](https://github.com/cjpais/Handy/issues/1660)). Handy no longer listens for SIGUSR1 on Linux; the post-processing toggle is still available via `handy --toggle-post-process`. **Remove any `pkill -USR1` bindings**: the signal is now delivered straight to WebKit's internal handler and can crash the app.

**Overlay & Pasting Issues (Linux):**

- The recording overlay window can interfere with pasting transcribed text into target applications on Linux (X11)
- **Solution:** Open **Settings > Advanced** and set **"Overlay Position"** to **"None"** to disable the overlay
- Enable **"Audio Feedback"** (also in Advanced) if you still want audible confirmation of recording state
- Users who upgrade from older versions or import settings from other platforms may need to manually apply this change

## Release Artifacts Are Unsigned

This fork has no code-signing certificate, so the Windows installers carry no
Authenticode signature and no publisher name, and Windows warns about the
download before you can install it. See
[BUILD.md](BUILD.md#release-artifacts-are-unsigned-and-windows-warns-about-them)
for what that warning looks like and how to get past it.

There is also no in-app updater: new versions are installed by hand. Nothing is
published next to the downloads — no `.sig`, no `.minisig`, no checksum list —
so there is nothing in-band to verify a download against. Establish provenance
out of band if that matters to you.

## Troubleshooting

### Previous Clipboard Content Is Pasted Instead of the Transcription

If the transcription is correct in **History** but Handy inserts text you copied earlier, see [issue #502](https://github.com/cjpais/Handy/issues/502). With the standard clipboard paste method, Handy restores your previous clipboard after a fixed delay. Under load, the receiving application may read the clipboard only after that restoration.

1. Open Handy's settings window and press `Cmd+Shift+D` (macOS) or `Ctrl+Shift+D` (Windows/Linux) to reveal **Debug**.
2. On **macOS and Windows**, try **Reliable Paste (Beta)** in Debug with a clipboard paste method selected. It uses clipboard read notifications to delay restoration instead of relying on the standard fixed delay. Test it in the application where the problem occurs; it is still experimental.
3. If Reliable Paste is disabled or unavailable, increase **Paste Delay (After)** in Debug and test again. This controls the wait before restoring your previous clipboard. **Paste Delay (Before)** controls the wait before sending the paste keystroke and addresses a different part of the operation. These delay settings apply to the standard paste path, not Reliable Paste.

If the problem persists, add your Handy version, operating system, receiving application, paste method, Reliable Paste setting, and before/after delays to the existing issue. Redact private dictated text before sharing logs.

### Custom Model Copy (Advanced)

The GigaAM v3 model ships inside the app, so there is normally nothing to
install. If you want to run a modified or community build of the same model,
drop it into the app-data `models` directory and Handy will prefer it over the
bundled copy.

#### Step 1: Find Your App Data Directory

1. Open Handy settings
2. Navigate to the **About** section
3. Copy the "App Data Directory" path shown there, or use the shortcuts:
   - **macOS**: `Cmd+Shift+D` to open debug menu
   - **Windows/Linux**: `Ctrl+Shift+D` to open debug menu

The typical paths are:

- **macOS**: `~/Library/Application Support/com.pais.handy/`
- **Windows**: `C:\Users\{username}\AppData\Roaming\com.pais.handy\`
- **Linux**: `~/.config/com.pais.handy/`

#### Step 2: Place the Files

Create a `models/giga-am-v3-int8` directory inside it and put both model files
there:

```
{app_data_dir}/models/giga-am-v3-int8/
├── model.int8.onnx
└── vocab.txt
```

**Important Notes:**

- The directory name must be exactly `giga-am-v3-int8`, and both files must be
  present — a partial copy is ignored
- After placing the files, restart Handy
- Handy reads the copy in place; it never modifies or deletes it

#### Step 3: Verify Installation

1. Restart Handy
2. Open Settings → Models and confirm the bundled **GigaAM v3** entry is listed
   and marked **Active**

### Linux Startup Crashes or Instability

If Handy fails to start reliably on Linux — for example, it crashes shortly after launch, never shows its window, or reports a Wayland protocol error — try the steps below in order.

**1. Install (or reinstall) `gtk-layer-shell`**

Handy uses `gtk-layer-shell` for its recording overlay and links against it at runtime. A missing or broken installation is the most common cause of startup failures and can manifest as a crash or a hang well before any window is shown. Make sure the runtime package is installed for your distro:

| Distro        | Package to install    | Example command                        |
| ------------- | --------------------- | -------------------------------------- |
| Ubuntu/Debian | `libgtk-layer-shell0` | `sudo apt install libgtk-layer-shell0` |
| Fedora/RHEL   | `gtk-layer-shell`     | `sudo dnf install gtk-layer-shell`     |
| Arch Linux    | `gtk-layer-shell`     | `sudo pacman -S gtk-layer-shell`       |

If it is already installed and you still see startup problems, try reinstalling it (e.g. `sudo pacman -S gtk-layer-shell` again) in case the library files were corrupted by a partial upgrade.

**2. Disable the GTK layer shell overlay (`HANDY_NO_GTK_LAYER_SHELL`)**

If installing the library does not help, you can skip `gtk-layer-shell` initialization entirely as a workaround. On some compositors (notably KDE Plasma under Wayland) it has been reported to interact poorly with the recording overlay. With this variable set, the overlay falls back to a regular always-on-top window:

```bash
HANDY_NO_GTK_LAYER_SHELL=1 handy
```

**3. Disable WebKit DMA-BUF renderer (`WEBKIT_DISABLE_DMABUF_RENDERER`)**

On some GPU/driver combinations the WebKitGTK DMA-BUF renderer can cause the window to fail to render or to crash. Try:

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 handy
```

**Making a workaround permanent**

Once you've found a flag that helps, export it from your shell profile (`~/.bashrc`, `~/.zshenv`, …) or from the desktop autostart entry that launches Handy. If you launch Handy from a `.desktop` file, you can prefix the `Exec=` line, e.g.:

```ini
Exec=env HANDY_NO_GTK_LAYER_SHELL=1 handy
```

If a workaround helps you, please [open an issue](https://github.com/metalmon/Handy/issues) describing your distro, desktop environment, and session type — that information helps us narrow down the underlying bug.

### Empty recording overlay on Hyprland / Omarchy

If the recording overlay is empty or bordered, fully quit Handy and launch a
[native installation](BUILD.md#linux-install-from-source) with Wayland enabled:

```bash
env -u HANDY_NO_GTK_LAYER_SHELL GDK_BACKEND=wayland handy
```

If this works, apply `GDK_BACKEND=wayland` only to Handy's launcher.

### Vulkan Overlays and Capture Tools on Windows (`HANDY_KEEP_VULKAN_IMPLICIT_LAYERS`)

On Windows, Handy asks the Vulkan loader to skip implicit layers to avoid crashes caused by overlay and capture hooks ([#2049](https://github.com/cjpais/Handy/issues/2049)). GPU acceleration remains enabled; this does not change system-wide settings.

To opt out for GPU selection or debugging tools, fully quit Handy (including the tray icon), then run both commands in the same PowerShell window:

```powershell
$env:HANDY_KEEP_VULKAN_IMPLICIT_LAYERS = "1"
& "$env:ProgramFiles\Handy\handy.exe"
```

Adjust the executable path if needed. This override only applies to apps launched from that PowerShell session, not the Start menu. Handy also preserves any existing `VK_LOADER_LAYERS_DISABLE` value.

### How to Contribute

1. **Check existing issues** at [github.com/metalmon/Handy/issues](https://github.com/metalmon/Handy/issues)
2. **Fork this repository** and create a feature branch
3. **Test thoroughly** on your target platform
4. **Submit a pull request** with clear description of changes
5. **Join the discussion** - reach out at [contact@handy.computer](mailto:contact@handy.computer)

The goal is to create both a useful tool and a foundation for others to build upon—a well-patterned, simple codebase that serves the community.

## Related Projects

- **[Handy CLI](https://github.com/cjpais/handy-cli)** - The original Python command-line version
- **[handy.computer](https://handy.computer)** - Project website with demos and documentation

## License

MIT License - see [LICENSE](LICENSE) file for details.

Handy is open-source software, but the Handy name, logo, icon, and brand assets are not open-source. Unofficial forks, rewrites, and redistributions must use their own branding and must not imply endorsement or affiliation.

## Acknowledgments

- **GigaAM v3** by Salute Developers for the speech recognition model
- **ggml** for an amazing cross-platform tensor library
- **Silero** for great lightweight VAD
- **Tauri** team for the excellent Rust-based app framework
- **Community contributors** helping make Handy better
