# mv7mute

A minimal CLI to toggle mute on the **Shure MV7**, designed to be bound to a global hotkey. Works on Windows, macOS, and Linux — anywhere `hidapi` can access the device.

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

## Installation

### GitHub Releases

Download the latest archive, installer, or script from the repository's GitHub Releases page.

- Windows: `.zip` and PowerShell installer
- macOS: `.tar.gz` archive and shell installer
- Linux: `.tar.gz` archive and shell installer

### Build from source

```powershell
cargo build --release
```

Binary: `target\release\mv7mute.exe`

## Release Process

- Pull requests and pushes run `cargo test` and `cargo clippy --all-targets --all-features` in GitHub Actions.
- Pushes to `main` run release-plz, which maintains a release PR and creates the version tag when that PR is merged.
- Pushed version tags trigger cargo-dist's release pipeline: a plan step validates the tag, platform jobs build the archives and installers, and the final announce step creates the GitHub Release with cargo-dist-generated notes.

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
├── README.md
└── src/
    ├── main.rs   # CLI arg parsing (clap)
    └── mv7.rs    # MV7 HID abstraction
```

## Dependencies

- [`hidapi`](https://crates.io/crates/hidapi) `2` — cross-platform HID access
- [`clap`](https://crates.io/crates/clap) `4` — CLI argument parsing

## Platform Notes

| Issue                       | Mitigation                                                      |
| --------------------------- | --------------------------------------------------------------- |
| Wake-from-sleep resets mute | Re-query state on every invocation — never assume               |
| HID exclusive access        | `hidapi` on Windows uses shared access; coexists with MOTIV app |
| Kernel driver detach        | Not needed on Windows — native HID driver works directly        |
| DSP boot overhead           | ~500 ms per invocation; acceptable for a hotkey                 |
| Linux HID permissions       | Add a udev rule granting access to the MV7 HID interface        |
| macOS HID permissions       | No extra setup needed; `hidapi` uses IOHIDManager               |

## Roadmap

**System tray app** — a lightweight tray icon that shows the current mute state and toggles on click, without needing a separate hotkey manager. The CLI stays as the backend.

**GUI with hotkey management** — a settings window to configure the global hotkey, view device status, and control lock behaviour, all in one place.

**Full DSP control** — expose the remaining MV7 settings: input gain, compressor (off / light / medium / heavy), limiter, high-pass filter, and presence filter.

## License

MIT — see [LICENSE](LICENSE)

## Credits

- [matteodelabre/mv7config](https://github.com/matteodelabre/mv7config) — MV7 HID protocol reference
- [DominicBoettger/omarchy-mic](https://github.com/DominicBoettger/omarchy-mic) — HID init sequence reference
