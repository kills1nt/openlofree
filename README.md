# openlofree

Open-source configuration tool for Lofree Flow 2 keyboards. It talks to the keyboard's built-in VIA firmware over USB, so there is no driver and no custom firmware.

Status: early. A desktop app (Tauri, Windows, Linux, macOS), a command line tool and a Rust library.

## Desktop app

Keys (click a key, pick what it types, apply), Lighting (live preview, saved on Apply), Profiles (save, apply, duplicate, export, import), Device (model, protocol, battery, factory backup), light and dark themes, a tray menu with one-click profiles. A Demo mode runs without a keyboard and never writes to a real one.

```bash
cd app
npm install
npm run tauri dev            # development
npm run tauri build -- --no-bundle   # release build, output in target/release
```

## What works

- Read the keymap of every layer, with key legends (Windows and Mac variants).
- Write single keys. Every write is read back and compared, a mismatch is an error.
- Backlight: off, steady, breathing, brightness. Saved to the keyboard.
- Profiles: save the keyboard's state to a JSON file and apply it back. Only the keys that differ are written.
- A factory backup of the keyboard's state is saved once, before the first write, and never overwritten.
- Battery over Bluetooth on macOS.

Tested on hardware: reading (`probe`, `keys`, and the app opening and reading the keymap) on the 84-key Flow 2 Mac over USB on Windows 11. **Not yet tested on hardware: any write** (`set-key`, `light`, `brightness`, `apply`), Linux, macOS, the 68 and 100 key models.

## Limits

- Configuration works over USB only. Bluetooth gives battery only.
- Only the Flow 2 Mac 84 is registered. Flow Lite and the original Flow use other protocols and are not supported.
- The tool writes the keymap and backlight only. It never writes firmware and never enters the bootloader.
- A sleeping keyboard does not answer. Press a key first.
- Windows and Linux battery are not implemented.

## Use

```bash
cargo install --path crates/flow2-cli   # or download flow2ctl from Releases (Windows)

flow2ctl probe                  # find the keyboard, print model, VIA protocol, layers
flow2ctl keys --layer 0         # print a layer
flow2ctl set-key 0 0 0 0x0029   # layer row col hexcode, verified by read-back
flow2ctl light on               # off | on | breathing
flow2ctl brightness 60          # 0 to 100
flow2ctl backup mine.json       # save the current state
flow2ctl apply mine.json        # write a profile back
flow2ctl battery                # macOS, Bluetooth
```

- macOS: allow Input Monitoring for your terminal (System Settings, Privacy and Security).
- Linux: building needs `libudev-dev`. Access to the hidraw device needs a udev rule, which is not shipped yet.

## Layout

```
crates/flow2-core   device logic: VIA codec, keymap, backlight, profiles, mock device
crates/flow2-cli    flow2ctl
app/                Tauri app (Svelte UI in app/src, Rust layer in app/src-tauri)
layouts/            key geometry per model
docs/               design spec, implementation plan, protocol notes, hardware checklist
```

## Credits

The backlight channel and battery approach come from [linder3hs/lofree-flow-2](https://github.com/linder3hs/lofree-flow-2) (macOS menu bar app, MIT). The code here is written independently.

## License

MIT
