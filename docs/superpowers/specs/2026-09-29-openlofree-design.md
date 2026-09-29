# openlofree: design

Date: 2026-09-29
Status: draft, awaiting owner review
Repo: `kills1nt/openlofree` (MIT, created only after this spec is approved)

## 1. Purpose

A free, open-source desktop app for Windows, Linux and macOS that configures the Lofree Flow 2 and Flow 2 Mac keyboards: key remapping across layers, profiles, backlight control, plus everything `linder3hs/lofree-flow-2` offers (battery, backlight, CLI, tray/menu bar). The owner has the 84-key Flow 2 Mac. Lofree ships only a Windows VIA configurator and no native app for macOS or Linux.

## 2. Scope

In scope (v1):
- Flow 2 and Flow 2 Mac, 68 / 84 / 100 keys. Same QMK+VIA firmware family, differences live in the layout JSON (matrix, geometry, PID) and modifier legends.
- Keymap editing per layer, macros are out of v1 (see 8).
- Profiles: save, apply, duplicate, delete, export/import as JSON.
- Backlight: mode (off, steady, breathing) and brightness. The keyboard has a white backlight only, there is no RGB.
- Device status: connection type, battery, model, VIA protocol version.
- Tray/menu bar with battery and profile switch.
- `flow2ctl` CLI with the same core.

Out of scope:
- Flow Lite (different, non-VIA vendor protocol) and the original Flow (no configuration protocol found in research, only OS-side remap is possible).
- Firmware flashing, DFU, bootloader jumps. openlofree never writes firmware.
- Automatic per-application profile switching (later issue).
- Bundling Lofree's JSON files. We ship our own geometry files in the same schema and let users import Lofree's JSON.

## 3. Facts and assumptions

Verified from the owner's `oe927-84-key.json`: name `Flow2@Lofree`, VID `0x388d`, PID `0x0028`, matrix 6 rows x 15 cols, layout in KLE-style rows.

From the research report (`lofree-flow2-research.md`, medium confidence): firmware is QMK with VIA, raw HID usage page `0xFF60`, usage `0x61`, 32-byte reports. The vendor VIA channel is not reachable over Bluetooth (two community sources, cause unknown). Behavior over the 2.4G dongle is unverified. PID differs per model and variant, so discovery matches by VID plus usage page and reads the PID from the definition file.

Assumptions to verify in step 1 of the plan:
- VIA command IDs (get protocol version, keycode get/set, layer count, buffer read/write, custom value get/set/save) match the public VIA protocol, checked against the VIA protocol source and the `linder3hs/lofree-flow-2` code.
- The backlight channel and value IDs, which `linder3hs` found by probing, not from a JSON menu.
- Raw keycodes `0x7793-0x7795` (BT 1-3) and `0x7785` (2.4G) survive a write and read back unchanged.
- The 68 and 100 models behave like the 84. Only the 84 is tested on hardware. The UI marks other models as "unverified".

## 4. Architecture

```
openlofree/
  crates/
    flow2-core/     library, all device logic, no UI
    flow2-cli/      flow2ctl binary, thin over core
  app/              Tauri 2 shell + Svelte/TypeScript UI, thin invoke commands
  layouts/          our own geometry files, one per model/variant
  docs/
```

`flow2-core` modules:
- `transport`: trait `Transport` (open, write report, read report). Implementations: `HidApiTransport`, `MockDevice`.
- `via`: VIA raw HID codec and commands. Pure functions over byte buffers, unit-testable.
- `device`: discovery by VID `0x388D` and usage page `0xFF60`/`0x61`, model lookup from the registry.
- `models`: registry of Flow 2 / Flow 2 Mac 68, 84, 100 (matrix, PID, geometry file, verified flag). Adding a model means adding one entry and one layout file.
- `layout`: parses geometry (KLE-style rows plus matrix positions "row,col").
- `keycodes`: keycode table, including raw values VIA drops, with display legends per variant (Mac legends: Option, Cmd).
- `battery`: trait with three implementations. macOS: `system_profiler`. Windows and Linux: chosen in step 1 (Windows Bluetooth battery property, Linux UPower/BlueZ), unverified today.
- `profiles`: a profile is one JSON file in the OS config dir: model id, keymap for all layers, backlight settings, schema version.

Data flow: UI -> Tauri `invoke` -> `flow2-core` -> `Transport` -> keyboard. Edits stay in memory until the user presses Apply. Device hotplug is polled.

## 5. Platform behavior

- USB: full configuration.
- Bluetooth: battery and status only. The UI says plainly that configuration needs USB.
- 2.4G dongle: treated as unverified until tested.
- macOS: needs Input Monitoring permission. Composite HID devices are opened non-exclusively so typing keeps working. The app explains the permission on first run.
- Linux: needs a udev rule for the hidraw node. The rule ships in the `.deb` and is documented for the AppImage.
- Windows: no extra permissions expected.

## 6. UI

Screens: Keys, Lighting, Profiles, Device, plus tray menu.
- Keys: visual keyboard from the geometry file, layer tabs, click a key to choose a keycode, changes preview instantly, Apply writes to the device.
- Lighting: mode and brightness with a live preview on the drawn keyboard.
- Profiles: list, save current, apply, duplicate, export, import.
- Device: connection type, battery, model, protocol version.
- Every screen has designed empty, loading and error states that name the cause and the next action ("No keyboard found. Connect it with USB-C, Bluetooth cannot be configured").
- Keyboard accessible: full Tab order, visible focus, Escape closes dialogs.
- UI language: English in v1, strings externalized so Ukrainian can be added.

### Design direction

Design Read: desktop tool for keyboard owners, calm and tactile, ENERGY 2 / RHYTHM 2 / MOTION 2. The palette and type are new for this project and do not reuse the owner's Hrateka identity.

- Theme: warm light and dark with a working toggle. Reason: not a developer terminal, used day and night.
- Palette: warm neutral base, ink text, one warm amber accent. Reason: the keyboard's backlight is white, so the accent is reserved for the selected key, active layer and Apply. No text is set on the accent (contrast), only fills.
- Type: not Inter or Geist. Chosen at implementation time with a written reason and Cyrillic support.
- Identity motif: the keycap as an object. Keys have depth, pressing lowers the cap, selecting raises a backlight-like glow.
- Not used: gradient backgrounds, glassmorphism, grids, sparkle icons, capsule badges, stat-card dashboards.
- Motion, each with a purpose: key press feedback, layer switch relabels keys, backlight preview, Apply progress and confirmation. Only `transform` and `opacity` (WebKitGTK on Linux), no infinite loops, everything off under `prefers-reduced-motion`.

antislop applies during the work (owner's choice). The Delivery Gate runs before each UI milestone is called done.

## 7. Safety, testing, release

Write safety:
- Only keymap and backlight are written, through VIA. No firmware, no DFU.
- First write auto-saves a "Factory backup" profile.
- Every write is read back and compared, a mismatch is shown as an error.
- Raw BT/2.4G keycodes are written raw and verified by read-back.

Testing:
- Unit tests on recorded byte fixtures for the VIA codec, geometry parser, profile serialization, keycode table. Own fixtures only.
- `MockDevice` backs core tests and lets the UI run without hardware.
- Hardware smoke checklist plus `flow2ctl probe`, run manually on the 84.

CI and release (GitHub Actions):
- Matrix windows/macos/ubuntu: `cargo fmt --check`, `clippy`, `cargo test`, `svelte-check`, Tauri build.
- Tag builds `.msi`, `.dmg`, `.AppImage`, `.deb`. Unsigned at first, the README documents the SmartScreen and Gatekeeper warnings.

License and credits: MIT. README credits `linder3hs/lofree-flow-2` as a protocol reference. Code is written independently.

## 8. Risks and open items

- Backlight protocol comes from another project's probing, must be re-verified on the 84 before the UI trusts it.
- Battery on Windows and Linux is not researched yet.
- Macros are left out of v1: VIA macro storage exists, but Lofree's firmware behavior with it is unverified. Revisit after keymap and lighting work on hardware.
- 68 and 100 layouts are unverified without hardware.
- Public GitHub visibility and repo settings are decided by the owner at creation time.

## 9. First plan steps (implementation order)

1. Read `linder3hs/lofree-flow-2` source and VIA protocol source, record confirmed command IDs and the backlight channel in `docs/protocol.md`.
2. `flow2-core`: `Transport`, `MockDevice`, `via` codec, tests.
3. `layout`, `models`, `keycodes` for the 84.
4. `flow2ctl probe`, first hardware read on the owner's keyboard.
5. Keymap read/write, backup, read-back verification.
6. Backlight read/write.
7. Battery per platform.
8. Tauri shell and UI screens, then profiles and tray.
9. CI and release pipeline.
10. Add 68 and 100 entries.
