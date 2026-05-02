# mv7mute

A minimal MV7 mute toolkit with a reusable core crate, a thin CLI, and a tray app. The CLI is designed for hotkey binding, and the tray app provides a desktop tray surface built with `winit` and `tray-icon`.

## The Problem

When Windows wakes from sleep, it can reset the MV7's mute state — but only if the device is **unlocked**. The fix is to keep the device locked at rest so Windows cannot touch it.

The catch: to toggle mute you need to unlock first, change the state, then re-lock. This CLI makes the whole sequence — unlock → toggle → re-lock — atomic and fast enough to bind to a hotkey.

## Usage

```
mv7mute              # toggle mute (default — ideal for hotkey binding)
mv7mute toggle       # same as bare invocation
mv7mute on           # mute
mv7mute off          # unmute
mv7mute status       # print mute state and lock state
mv7mute lock         # lock the device
mv7mute unlock       # unlock the device
mv7mute version      # print version information
mv7mute --version    # same as the version subcommand
```

The tray app runs as a background process with a system tray icon:

```
mv7mute-tray
```

On Windows it is a `windows_subsystem = "windows"` binary — no console window opens. Install the MSI to get a Start menu entry that shows up in Windows Search, then launch it from Start or enable launch-at-startup from the tray menu.

## Installation

### GitHub Releases

Download the latest `vX.Y.Z` archive, installer, or script from the repository's GitHub Releases page.

- Windows: `.zip`, PowerShell installer, and MSI
- macOS: `.tar.gz` archive and shell installer
- Linux: `.tar.gz` archive and shell installer

### Build from source

```powershell
cargo build --workspace --release
```

Binary outputs:

- `target\release\mv7mute.exe`
- `target\release\mv7mute-tray.exe`

## Release Process

- Pull requests and pushes run `cargo test` and `cargo clippy --all-targets --all-features` in GitHub Actions.
- Pushes to `main` run release-plz. It maintains release PRs, creates the shared app tag `vX.Y.Z` for the CLI and tray binaries, and publishes the library crate with its own `mv7mute-core-vX.Y.Z` tag when needed. `release_always = true` is intentional so releases still happen when GitHub squash-merges the release PR. The `release-plz release` job requires a `RELEASE_PLZ_TOKEN` secret so the tag push can trigger the release workflow.
- Pushed `vX.Y.Z` tags trigger cargo-dist's release pipeline: a plan step validates the tag, platform jobs build the archives and installers for both app packages, and the final announce step creates the matching GitHub Release with cargo-dist-generated notes and assets.

### Hotkey binding

**PowerToys** — Run → assign a shortcut to `mv7mute.exe`

**AutoHotkey**:

```ahk
^!m:: Run mv7mute.exe
```

**Windows shortcut** — create a `.lnk` pointing to the exe, then assign a shortcut key in its properties.

## How It Works

Communication uses the MV7's USB HID text protocol over interface `3` (64-byte reports). On each invocation:

1. Open the HID device (VID `0x14ED`, PID `0x1012`, interface `3`)
2. Authenticate: send `su adm`, wait for `su=adm`
3. Boot DSP: send `bootDSP C`, wait for `dspBooted`
4. Query lock state (`lock`); if locked, send `lock off`
5. Execute the requested command
6. Re-lock if the device was originally locked (`lock on`)
7. Close the device

No driver swap (Zadig/WinUSB) is required — the native Windows HID driver works directly.

## HID Protocol Reference

| Step         | Command sent  | Expected response             |
| ------------ | ------------- | ----------------------------- |
| Authenticate | `su adm`      | `su=adm`                      |
| Boot DSP     | `bootDSP C`   | `dspBooted`                   |
| Query lock   | `lock`        | `lock=on` or `lock=off`       |
| Lock         | `lock on`     | —                             |
| Unlock       | `lock off`    | —                             |
| Query mute   | `micMute`     | `micMute=on` or `micMute=off` |
| Mute         | `micMute on`  | —                             |
| Unmute       | `micMute off` | —                             |

## Project Structure

```
mv7mute/
├── Cargo.toml
├── dist-workspace.toml
├── mv7mute-core/
│   └── src/
│       ├── lib.rs    # command orchestration and typed tray-facing state API
│       └── mv7.rs    # MV7 HID abstraction
├── mv7mute/
│   └── src/
│       └── main.rs   # thin CLI entrypoint
└── mv7mute-tray/
    └── src/
        ├── main.rs   # entry point: event loop wiring and single-instance guard
        ├── app.rs    # App, TrayState, menu handles, ApplicationHandler impl
        ├── events.rs # UserEvent, WorkerEvent, WorkerCommand
        ├── icon.rs   # procedural tray icon renderer
        └── worker.rs # background worker thread and device polling loop
```

## Dependencies

- [`hidapi`](https://crates.io/crates/hidapi) `2` — cross-platform HID access
- [`clap`](https://crates.io/crates/clap) `4` — CLI argument parsing
- [`winit`](https://crates.io/crates/winit) `0.30` — tray app event loop
- [`tray-icon`](https://crates.io/crates/tray-icon) `0.23` — system tray integration

## Platform Notes

| Issue                       | Mitigation                                                      |
| --------------------------- | --------------------------------------------------------------- |
| Wake-from-sleep resets mute | Re-query state on every invocation — never assume               |
| HID exclusive access        | `hidapi` on Windows uses shared access; coexists with MOTIV app |
| Kernel driver detach        | Not needed on Windows — native HID driver works directly        |
| DSP boot overhead           | ~500 ms per invocation; acceptable for a hotkey                 |
| Linux HID permissions       | Add a udev rule granting access to the MV7 HID interface        |
| macOS HID permissions       | No extra setup needed; `hidapi` uses IOHIDManager               |

### Tray-specific notes

- The tray app refreshes device state every 5 seconds by default.
- On Windows and macOS, left-click toggles mute and right-click opens the menu.
- On Linux, `tray-icon` depends on GTK/appindicator system libraries and does not emit tray click events, so toggle is available through the tray menu.
- Launch-at-startup integration is currently implemented only on Windows.

## Roadmap

**Tray settings** — expose refresh interval and tray behavior in the UI instead of keeping them hard-coded.

**GUI with hotkey management** — a settings window to configure the global hotkey, view device status, and control lock behaviour, all in one place.

**Full DSP control** — expose the remaining MV7 settings: input gain, compressor (off / light / medium / heavy), limiter, high-pass filter, and presence filter.

## License

MIT — see [LICENSE](LICENSE)

## Credits

- [matteodelabre/mv7config](https://github.com/matteodelabre/mv7config) — MV7 HID protocol reference
- [DominicBoettger/omarchy-mic](https://github.com/DominicBoettger/omarchy-mic) — HID init sequence reference
