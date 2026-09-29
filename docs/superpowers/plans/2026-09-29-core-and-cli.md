# openlofree Core and CLI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A Rust library (`flow2-core`) and command line tool (`flow2ctl`) that read and write the keymap and backlight of a Lofree Flow 2 Mac 84 over USB, with profiles, a factory backup, read-back verification and macOS battery, ending with a first real hardware bring-up.

**Architecture:** All device logic lives in `flow2-core` behind a `Transport` trait (real `hidapi` transport plus an in-memory `MockDevice` that emulates VIA). A `ViaClient` adds a single retry. Pure modules (`via` codec, `layout` parser, `keycodes`, `profile`) sit beside device modules (`keymap`, `backlight`). `flow2ctl` is a thin clap wrapper. The Tauri UI and CI are separate plans that start after this one's bring-up.

**Tech Stack:** Rust 2021 (stable 1.98 available), `hidapi` 2, `serde` and `serde_json`, `thiserror` 2, `dirs` 6, `clap` 4 (CLI only).

**Spec:** `docs/superpowers/specs/2026-09-29-openlofree-design.md`

**Verification status of the code in this plan:** every code block below was assembled from a scratch crate where `cargo fmt`, `cargo clippy -- -D warnings` and `cargo test --workspace` (37 tests) pass on Windows. The `hidapi` transport and the CLI were also compiled and run once against the owner's keyboard (see Task 12). Nothing in it has been verified on Linux or macOS yet.

## Global Constraints

- License MIT. Repo `kills1nt/openlofree`, commits authored as `kills1nt <77559585+kills1nt@users.noreply.github.com>` (already set in the repo's local git config).
- openlofree only writes keymap and backlight through VIA. It never writes firmware, never enters DFU, never sends the bootloader-jump command.
- The first write of a run creates the factory backup once and never overwrites it.
- Every key write is read back and compared; a mismatch is an error, never silent.
- VIA raw HID: vendor id `0x388D`, usage page `0xFF60`, usage `0x61`, 32-byte reports, no report id on the wire (hidapi gets a leading `0x00`).
- Backlight is VIA custom channel 1: value 1 brightness (0 to 255), value 2 effect (0 steady, 1 breathing), save with command `0x09`.
- Configuration works over USB only. Bluetooth gives battery only.
- No em dash character in any text, code comment or doc.
- Build and test from a short path such as `E:\Projects\openlofree`: Windows fails to link or run tests when the path passes 260 characters.

## Review Focus

Input classes the spec implies but a plain happy-path test would miss, most likely first. Each has a test in the task that owns the code.

- Cable unplugged halfway through applying a profile: the call returns an error, the keyboard holds a partial keymap, and applying again finishes the job writing only what is missing. Test `apply_interrupted_by_unplug_errors_and_can_be_finished_later` in Task 4.
- A profile made for a different matrix size than the connected keyboard is refused before any write. Test `apply_refuses_a_profile_for_a_different_matrix` in Task 9.
- Firmware acknowledges a key write but does not apply it: reported as `VerifyFailed` with layer, row, column, wrote and read values. Test `verified_write_reports_unapplied_write` in Task 4.
- Firmware answers an unhandled command with `0xFF` as the first byte: reported as `BadReply`, not misread as data. Test `rejects_unhandled_command_reply` in Task 2.
- A profile file that is hand-edited, truncated, from a newer schema, or has a wrong layer length: rejected with a message, never partially applied. Test `rejects_wrong_schema_and_shape` in Task 9.

## File Structure

```
Cargo.toml                          workspace
.gitignore
layouts/flow2-mac-84.json           our own geometry file for the 84-key Mac model
crates/flow2-core/
  Cargo.toml
  src/lib.rs                        module list and re-exports
  src/error.rs                      Error and Result
  src/via.rs                        VIA report builders and reply parsers (pure)
  src/transport.rs                  Transport trait and MockDevice
  src/client.rs                     ViaClient: typed calls, one retry
  src/keymap.rs                     read, verified single write, apply differences
  src/backlight.rs                  mode and brightness read and apply
  src/layout.rs                     parse a VIA-style definition into ModelDef
  src/models.rs                     registry of supported models
  src/keycodes.rs                   display legends, Win and Mac variants
  src/profile.rs                    Profile JSON, capture, apply, factory backup
  src/battery.rs                    system_profiler parser, macOS reader
  src/hid.rs                        HidTransport over hidapi
crates/flow2-cli/
  Cargo.toml
  src/main.rs                       flow2ctl
docs/protocol.md                    confirmed protocol facts and open questions
docs/hardware-checklist.md          manual checks on a real keyboard
```

Each module has one job and depends only on earlier ones. `hid.rs` and `main.rs` have no unit tests because they need hardware; they are covered by the checklist in Task 11 and the bring-up in Task 12.

---

### Task 1: Workspace skeleton and errors

**Files:**
- Create: `Cargo.toml`, `.gitignore`, `crates/flow2-core/Cargo.toml`, `crates/flow2-core/src/lib.rs`, `crates/flow2-core/src/error.rs`

**Interfaces:**
- Produces: `flow2_core::Error` (variants `NotFound`, `Io(String)`, `Timeout`, `BadReply(String)`, `VerifyFailed { layer, row, col, wrote, read }`, `Layout(String)`, `Profile(String)`) and `flow2_core::Result<T>`.

- [ ] **Step 1: Create the workspace files**

`Cargo.toml`:
```toml
[workspace]
resolver = "2"
members = ["crates/flow2-core"]
```

`.gitignore`:
```
/target
```

`crates/flow2-core/Cargo.toml`:
```toml
[package]
name = "flow2-core"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "Device logic for Lofree Flow 2 keyboards (VIA raw HID)"

[dependencies]
dirs = "6"
hidapi = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
```

- [ ] **Step 2: Create `crates/flow2-core/src/error.rs`**

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no Flow 2 found. Connect it with USB-C: Bluetooth cannot be configured")]
    NotFound,
    #[error("device I/O failed: {0}")]
    Io(String),
    #[error("no response from the keyboard")]
    Timeout,
    #[error("unexpected reply: {0}")]
    BadReply(String),
    #[error("write not confirmed at layer {layer} row {row} col {col}: wrote {wrote:#06x}, read back {read:#06x}")]
    VerifyFailed {
        layer: u8,
        row: u8,
        col: u8,
        wrote: u16,
        read: u16,
    },
    #[error("invalid layout: {0}")]
    Layout(String),
    #[error("invalid profile: {0}")]
    Profile(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

- [ ] **Step 3: Create `crates/flow2-core/src/lib.rs`**

```rust
pub mod error;

pub use error::{Error, Result};
```

- [ ] **Step 4: Verify it builds**

Run: `cargo check`
Expected: `Finished` with no warnings.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml .gitignore crates
git commit -m "chore: cargo workspace with flow2-core and error type"
```

---

### Task 2: VIA codec

**Files:**
- Create: `crates/flow2-core/src/via.rs`
- Modify: `crates/flow2-core/src/lib.rs` (add `pub mod via;`)

**Interfaces:**
- Produces: `REPORT_SIZE: usize = 32`, `type Report = [u8; 32]`, command constants `CMD_GET_PROTOCOL_VERSION 0x01`, `CMD_KEYMAP_GET_KEYCODE 0x04`, `CMD_KEYMAP_SET_KEYCODE 0x05`, `CMD_CUSTOM_SET_VALUE 0x07`, `CMD_CUSTOM_GET_VALUE 0x08`, `CMD_CUSTOM_SAVE 0x09`, `CMD_KEYMAP_LAYER_COUNT 0x11`, `CHANNEL_BACKLIGHT 1`, `VALUE_BRIGHTNESS 1`, `VALUE_EFFECT 2`; builders `protocol_version() -> Report`, `layer_count() -> Report`, `get_keycode(layer, row, col) -> Report`, `set_keycode(layer, row, col, code: u16) -> Report`, `backlight_get(value) -> Report`, `backlight_set(value, data) -> Report`, `backlight_save() -> Report`; parsers `parse_protocol_version(&Report) -> Result<u16>`, `parse_layer_count -> Result<u8>`, `parse_keycode -> Result<u16>`, `parse_backlight_value -> Result<u8>`.

- [ ] **Step 1: Write the failing tests.** Create `via.rs` containing only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_are_32_bytes_zero_padded() {
        let r = protocol_version();
        assert_eq!(r.len(), 32);
        assert_eq!(&r[..2], &[0x01, 0x00]);
    }

    #[test]
    fn keycode_request_layout() {
        assert_eq!(&get_keycode(2, 3, 4)[..4], &[0x04, 2, 3, 4]);
        assert_eq!(
            &set_keycode(1, 2, 3, 0x7793)[..6],
            &[0x05, 1, 2, 3, 0x77, 0x93]
        );
    }

    #[test]
    fn backlight_requests_match_reference_project() {
        assert_eq!(
            &backlight_set(VALUE_BRIGHTNESS, 128)[..4],
            &[0x07, 0x01, 0x01, 128]
        );
        assert_eq!(&backlight_set(VALUE_EFFECT, 1)[..4], &[0x07, 0x01, 0x02, 1]);
        assert_eq!(&backlight_get(VALUE_EFFECT)[..3], &[0x08, 0x01, 0x02]);
        assert_eq!(&backlight_save()[..2], &[0x09, 0x01]);
    }

    #[test]
    fn parses_replies() {
        let mut r = protocol_version();
        r[1] = 0x00;
        r[2] = 0x0C;
        assert_eq!(parse_protocol_version(&r).unwrap(), 12);

        let mut r = get_keycode(0, 0, 0);
        r[4] = 0x00;
        r[5] = 0x29;
        assert_eq!(parse_keycode(&r).unwrap(), 0x0029);

        let mut r = backlight_get(VALUE_BRIGHTNESS);
        r[3] = 77;
        assert_eq!(parse_backlight_value(&r).unwrap(), 77);
    }

    #[test]
    fn rejects_unhandled_command_reply() {
        let mut r = protocol_version();
        r[0] = 0xFF; // VIA answers 0xFF for commands the firmware does not handle
        assert!(matches!(
            parse_protocol_version(&r),
            Err(Error::BadReply(_))
        ));
    }
}
```

Add `pub mod via;` to `lib.rs`.

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p flow2-core via`
Expected: FAIL, the module does not compile because the functions under test do not exist.

- [ ] **Step 3: Add the implementation above the tests module in `via.rs`**

```rust
//! VIA raw HID codec: pure functions over 32-byte reports.
use crate::error::{Error, Result};

pub const REPORT_SIZE: usize = 32;
pub type Report = [u8; REPORT_SIZE];

pub const CMD_GET_PROTOCOL_VERSION: u8 = 0x01;
pub const CMD_KEYMAP_GET_KEYCODE: u8 = 0x04;
pub const CMD_KEYMAP_SET_KEYCODE: u8 = 0x05;
pub const CMD_CUSTOM_SET_VALUE: u8 = 0x07;
pub const CMD_CUSTOM_GET_VALUE: u8 = 0x08;
pub const CMD_CUSTOM_SAVE: u8 = 0x09;
pub const CMD_KEYMAP_LAYER_COUNT: u8 = 0x11;

/// Backlight channel and value ids as used by the reference project linder3hs/lofree-flow-2.
pub const CHANNEL_BACKLIGHT: u8 = 1;
pub const VALUE_BRIGHTNESS: u8 = 1;
pub const VALUE_EFFECT: u8 = 2;

fn pack(payload: &[u8]) -> Report {
    let mut r = [0u8; REPORT_SIZE];
    r[..payload.len()].copy_from_slice(payload);
    r
}

fn expect_echo(r: &Report, cmd: u8) -> Result<()> {
    if r[0] == cmd {
        Ok(())
    } else {
        Err(Error::BadReply(format!(
            "expected command {cmd:#04x}, got {:#04x}",
            r[0]
        )))
    }
}

pub fn protocol_version() -> Report {
    pack(&[CMD_GET_PROTOCOL_VERSION])
}

pub fn parse_protocol_version(r: &Report) -> Result<u16> {
    expect_echo(r, CMD_GET_PROTOCOL_VERSION)?;
    Ok(u16::from_be_bytes([r[1], r[2]]))
}

pub fn layer_count() -> Report {
    pack(&[CMD_KEYMAP_LAYER_COUNT])
}

pub fn parse_layer_count(r: &Report) -> Result<u8> {
    expect_echo(r, CMD_KEYMAP_LAYER_COUNT)?;
    Ok(r[1])
}

pub fn get_keycode(layer: u8, row: u8, col: u8) -> Report {
    pack(&[CMD_KEYMAP_GET_KEYCODE, layer, row, col])
}

pub fn parse_keycode(r: &Report) -> Result<u16> {
    expect_echo(r, CMD_KEYMAP_GET_KEYCODE)?;
    Ok(u16::from_be_bytes([r[4], r[5]]))
}

pub fn set_keycode(layer: u8, row: u8, col: u8, code: u16) -> Report {
    let [hi, lo] = code.to_be_bytes();
    pack(&[CMD_KEYMAP_SET_KEYCODE, layer, row, col, hi, lo])
}

pub fn backlight_get(value: u8) -> Report {
    pack(&[CMD_CUSTOM_GET_VALUE, CHANNEL_BACKLIGHT, value])
}

pub fn parse_backlight_value(r: &Report) -> Result<u8> {
    expect_echo(r, CMD_CUSTOM_GET_VALUE)?;
    Ok(r[3])
}

pub fn backlight_set(value: u8, data: u8) -> Report {
    pack(&[CMD_CUSTOM_SET_VALUE, CHANNEL_BACKLIGHT, value, data])
}

pub fn backlight_save() -> Report {
    pack(&[CMD_CUSTOM_SAVE, CHANNEL_BACKLIGHT])
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p flow2-core via`
Expected: 5 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/flow2-core/src
git commit -m "feat(core): VIA raw HID report codec"
```

---

### Task 3: Transport, mock device and client

**Files:**
- Create: `crates/flow2-core/src/transport.rs`, `crates/flow2-core/src/client.rs`
- Modify: `crates/flow2-core/src/lib.rs` (add `pub mod transport;` and `pub mod client;`)

**Interfaces:**
- Consumes: everything from Task 2.
- Produces: `trait Transport { fn exchange(&mut self, out: &Report) -> Result<Report>; }`. `transport::mock::MockDevice` with public fields `protocol, rows, cols, layers, keymap: Vec<Vec<u16>>, brightness, effect, saved, ignore_key_writes` and `MockDevice::new(rows, cols, layers)`. `ViaClient<T: Transport>` with `new(T)`, `into_transport(self) -> T`, `raw(&Report) -> Result<Report>` (retries once on `Io` or `Timeout`), `protocol_version() -> Result<u16>`, `layer_count() -> Result<u8>`, `get_keycode(layer,row,col) -> Result<u16>`, `set_keycode(layer,row,col,code) -> Result<()>`, `backlight_get(value) -> Result<u8>`, `backlight_set(value,data) -> Result<()>`, `backlight_save() -> Result<()>`.

- [ ] **Step 1: Create `transport.rs`** (the trait and the mock are exercised by the client tests, so they come first)

```rust
use crate::error::Result;
use crate::via::Report;

/// One request, one reply. VIA answers every command, so a single call is enough.
pub trait Transport {
    fn exchange(&mut self, out: &Report) -> Result<Report>;
}

/// In-memory VIA device for tests and for running the UI without a keyboard.
pub mod mock {
    use super::Transport;
    use crate::error::{Error, Result};
    use crate::via::*;

    pub struct MockDevice {
        pub protocol: u16,
        pub rows: u8,
        pub cols: u8,
        pub layers: u8,
        /// layers[l][row * cols + col]
        pub keymap: Vec<Vec<u16>>,
        pub brightness: u8,
        pub effect: u8,
        pub saved: bool,
        /// Simulates a firmware that acks key writes but does not apply them.
        pub ignore_key_writes: bool,
    }

    impl MockDevice {
        pub fn new(rows: u8, cols: u8, layers: u8) -> Self {
            Self {
                protocol: 12,
                rows,
                cols,
                layers,
                keymap: vec![vec![0; rows as usize * cols as usize]; layers as usize],
                brightness: 255,
                effect: 0,
                saved: false,
                ignore_key_writes: false,
            }
        }

        fn slot(&self, layer: u8, row: u8, col: u8) -> Option<(usize, usize)> {
            if layer < self.layers && row < self.rows && col < self.cols {
                Some((
                    layer as usize,
                    row as usize * self.cols as usize + col as usize,
                ))
            } else {
                None
            }
        }
    }

    impl Transport for MockDevice {
        fn exchange(&mut self, out: &Report) -> Result<Report> {
            let mut r = *out;
            match out[0] {
                CMD_GET_PROTOCOL_VERSION => r[1..3].copy_from_slice(&self.protocol.to_be_bytes()),
                CMD_KEYMAP_LAYER_COUNT => r[1] = self.layers,
                CMD_KEYMAP_GET_KEYCODE => {
                    let code = self
                        .slot(out[1], out[2], out[3])
                        .map_or(0, |(l, i)| self.keymap[l][i]);
                    r[4..6].copy_from_slice(&code.to_be_bytes());
                }
                CMD_KEYMAP_SET_KEYCODE => {
                    if let (false, Some((l, i))) =
                        (self.ignore_key_writes, self.slot(out[1], out[2], out[3]))
                    {
                        self.keymap[l][i] = u16::from_be_bytes([out[4], out[5]]);
                    }
                }
                CMD_CUSTOM_SET_VALUE if out[1] == CHANNEL_BACKLIGHT => match out[2] {
                    VALUE_BRIGHTNESS => self.brightness = out[3],
                    VALUE_EFFECT => self.effect = out[3],
                    _ => {}
                },
                CMD_CUSTOM_GET_VALUE if out[1] == CHANNEL_BACKLIGHT => match out[2] {
                    VALUE_BRIGHTNESS => r[3] = self.brightness,
                    VALUE_EFFECT => r[3] = self.effect,
                    _ => {}
                },
                CMD_CUSTOM_SAVE => self.saved = true,
                other => {
                    return Err(Error::BadReply(format!(
                        "mock: unhandled command {other:#04x}"
                    )))
                }
            }
            Ok(r)
        }
    }
}
```

Add `pub mod transport;` to `lib.rs`.

- [ ] **Step 2: Write the failing client tests.** Create `client.rs` containing only:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::MockDevice;

    #[test]
    fn talks_to_mock_device() {
        let mut c = ViaClient::new(MockDevice::new(6, 15, 4));
        assert_eq!(c.protocol_version().unwrap(), 12);
        assert_eq!(c.layer_count().unwrap(), 4);
        c.set_keycode(1, 2, 3, 0x0004).unwrap();
        assert_eq!(c.get_keycode(1, 2, 3).unwrap(), 0x0004);
        assert_eq!(c.get_keycode(0, 2, 3).unwrap(), 0);
    }

    struct FailsOnce(MockDevice, bool);
    impl Transport for FailsOnce {
        fn exchange(&mut self, out: &Report) -> Result<Report> {
            if !self.1 {
                self.1 = true;
                return Err(Error::Timeout);
            }
            self.0.exchange(out)
        }
    }

    #[test]
    fn retries_once_after_timeout() {
        let mut c = ViaClient::new(FailsOnce(MockDevice::new(6, 15, 4), false));
        assert_eq!(c.protocol_version().unwrap(), 12);
    }

    struct AlwaysFails;
    impl Transport for AlwaysFails {
        fn exchange(&mut self, _: &Report) -> Result<Report> {
            Err(Error::Timeout)
        }
    }

    #[test]
    fn gives_up_after_second_failure() {
        let mut c = ViaClient::new(AlwaysFails);
        assert!(matches!(c.protocol_version(), Err(Error::Timeout)));
    }
}
```

Add `pub mod client;` to `lib.rs`.

- [ ] **Step 3: Run the tests to see them fail**

Run: `cargo test -p flow2-core client`
Expected: FAIL, `ViaClient` does not exist.

- [ ] **Step 4: Add the implementation above the tests module in `client.rs`**

```rust
use crate::error::{Error, Result};
use crate::transport::Transport;
use crate::via::{self, Report};

pub struct ViaClient<T: Transport> {
    transport: T,
}

impl<T: Transport> ViaClient<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn into_transport(self) -> T {
        self.transport
    }

    /// Sends a raw report, retrying once on I/O errors or timeouts.
    pub fn raw(&mut self, report: &Report) -> Result<Report> {
        match self.transport.exchange(report) {
            Err(Error::Io(_) | Error::Timeout) => self.transport.exchange(report),
            other => other,
        }
    }

    pub fn protocol_version(&mut self) -> Result<u16> {
        via::parse_protocol_version(&self.raw(&via::protocol_version())?)
    }

    pub fn layer_count(&mut self) -> Result<u8> {
        via::parse_layer_count(&self.raw(&via::layer_count())?)
    }

    pub fn get_keycode(&mut self, layer: u8, row: u8, col: u8) -> Result<u16> {
        via::parse_keycode(&self.raw(&via::get_keycode(layer, row, col))?)
    }

    pub fn set_keycode(&mut self, layer: u8, row: u8, col: u8, code: u16) -> Result<()> {
        self.raw(&via::set_keycode(layer, row, col, code))
            .map(|_| ())
    }

    pub fn backlight_get(&mut self, value: u8) -> Result<u8> {
        via::parse_backlight_value(&self.raw(&via::backlight_get(value))?)
    }

    pub fn backlight_set(&mut self, value: u8, data: u8) -> Result<()> {
        self.raw(&via::backlight_set(value, data)).map(|_| ())
    }

    pub fn backlight_save(&mut self) -> Result<()> {
        self.raw(&via::backlight_save()).map(|_| ())
    }
}
```

- [ ] **Step 5: Run the tests to see them pass**

Run: `cargo test -p flow2-core client`
Expected: 3 passed.

- [ ] **Step 6: Commit**

```bash
git add crates/flow2-core/src
git commit -m "feat(core): Transport trait, MockDevice and ViaClient"
```

---

### Task 4: Keymap read, verified write, apply

**Files:**
- Create: `crates/flow2-core/src/keymap.rs`
- Modify: `crates/flow2-core/src/lib.rs` (add `pub mod keymap;`)

**Interfaces:**
- Consumes: `ViaClient`, `Transport`, `MockDevice`, `Error`.
- Produces: `struct Keymap { rows: u8, cols: u8, layers: Vec<Vec<u16>> }` (index `row * cols + col`), `read(&mut ViaClient<T>, rows, cols) -> Result<Keymap>`, `set_key_verified(&mut ViaClient<T>, layer, row, col, code) -> Result<()>`, `apply(&mut ViaClient<T>, &Keymap) -> Result<usize>` (writes only differences, returns how many).

- [ ] **Step 1: Write the failing tests.** Create `keymap.rs` containing only:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::MockDevice;

    fn client() -> ViaClient<MockDevice> {
        ViaClient::new(MockDevice::new(2, 3, 2))
    }

    #[test]
    fn reads_all_layers() {
        let mut dev = MockDevice::new(2, 3, 2);
        dev.keymap[1][4] = 0x0029;
        let km = read(&mut ViaClient::new(dev), 2, 3).unwrap();
        assert_eq!(km.layers.len(), 2);
        assert_eq!(km.layers[1][4], 0x0029);
    }

    #[test]
    fn verified_write_succeeds() {
        let mut c = client();
        set_key_verified(&mut c, 0, 1, 2, 0x0004).unwrap();
        assert_eq!(c.get_keycode(0, 1, 2).unwrap(), 0x0004);
    }

    #[test]
    fn verified_write_reports_unapplied_write() {
        let mut dev = MockDevice::new(2, 3, 2);
        dev.ignore_key_writes = true;
        let err = set_key_verified(&mut ViaClient::new(dev), 0, 1, 2, 0x0004).unwrap_err();
        assert!(matches!(
            err,
            Error::VerifyFailed {
                wrote: 0x0004,
                read: 0,
                ..
            }
        ));
    }

    #[test]
    fn apply_writes_only_differences() {
        let mut c = client();
        let mut target = read(&mut c, 2, 3).unwrap();
        target.layers[0][0] = 0x0004;
        target.layers[1][5] = 0x5221;
        assert_eq!(apply(&mut c, &target).unwrap(), 2);
        assert_eq!(read(&mut c, 2, 3).unwrap(), target);
        assert_eq!(apply(&mut c, &target).unwrap(), 0);
    }

    #[test]
    fn apply_rejects_layer_count_mismatch() {
        let mut c = client();
        let target = Keymap {
            rows: 2,
            cols: 3,
            layers: vec![vec![0; 6]],
        };
        assert!(matches!(apply(&mut c, &target), Err(Error::Profile(_))));
    }

    /// Unplug simulation: works for `budget` exchanges, then every call fails.
    struct DiesAfter(MockDevice, usize);
    impl Transport for DiesAfter {
        fn exchange(&mut self, out: &crate::via::Report) -> Result<crate::via::Report> {
            if self.1 == 0 {
                return Err(Error::Io("device gone".into()));
            }
            self.1 -= 1;
            self.0.exchange(out)
        }
    }

    #[test]
    fn apply_interrupted_by_unplug_errors_and_can_be_finished_later() {
        let mut target = read(&mut client(), 2, 3).unwrap();
        target.layers[0][0] = 0x0004;
        target.layers[1][5] = 0x0005;

        // 1 layer-count + 12 reads + first write/readback pair, then the cable "comes out".
        let mut dying = ViaClient::new(DiesAfter(MockDevice::new(2, 3, 2), 15));
        assert!(matches!(apply(&mut dying, &target), Err(Error::Io(_))));
        let half_written = dying.into_transport().0;
        assert_eq!(half_written.keymap[0][0], 0x0004);
        assert_eq!(half_written.keymap[1][5], 0);

        // Plugged back in: applying again finishes the job and only writes what is missing.
        let mut c = ViaClient::new(half_written);
        assert_eq!(apply(&mut c, &target).unwrap(), 1);
        assert_eq!(read(&mut c, 2, 3).unwrap(), target);
    }
}
```

Add `pub mod keymap;` to `lib.rs`.

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p flow2-core keymap`
Expected: FAIL, `read`, `apply` and the others do not exist.

- [ ] **Step 3: Add the implementation above the tests module**

```rust
use crate::client::ViaClient;
use crate::error::{Error, Result};
use crate::transport::Transport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keymap {
    pub rows: u8,
    pub cols: u8,
    /// layers[l][row * cols + col]
    pub layers: Vec<Vec<u16>>,
}

pub fn read<T: Transport>(client: &mut ViaClient<T>, rows: u8, cols: u8) -> Result<Keymap> {
    let count = client.layer_count()?;
    let mut layers = Vec::with_capacity(count as usize);
    for layer in 0..count {
        let mut keys = Vec::with_capacity(rows as usize * cols as usize);
        for row in 0..rows {
            for col in 0..cols {
                keys.push(client.get_keycode(layer, row, col)?);
            }
        }
        layers.push(keys);
    }
    Ok(Keymap { rows, cols, layers })
}

/// Writes one key, then reads it back. A mismatch is an error, never silent.
pub fn set_key_verified<T: Transport>(
    client: &mut ViaClient<T>,
    layer: u8,
    row: u8,
    col: u8,
    code: u16,
) -> Result<()> {
    client.set_keycode(layer, row, col, code)?;
    let read = client.get_keycode(layer, row, col)?;
    if read == code {
        Ok(())
    } else {
        Err(Error::VerifyFailed {
            layer,
            row,
            col,
            wrote: code,
            read,
        })
    }
}

/// Writes only the keys that differ from the device. Returns how many were written.
pub fn apply<T: Transport>(client: &mut ViaClient<T>, target: &Keymap) -> Result<usize> {
    let current = read(client, target.rows, target.cols)?;
    if current.layers.len() != target.layers.len() {
        return Err(Error::Profile(format!(
            "profile has {} layers, keyboard has {}",
            target.layers.len(),
            current.layers.len()
        )));
    }
    let mut written = 0;
    for (l, (have, want)) in current.layers.iter().zip(&target.layers).enumerate() {
        for (i, (&h, &w)) in have.iter().zip(want).enumerate() {
            if h != w {
                let row = (i / target.cols as usize) as u8;
                let col = (i % target.cols as usize) as u8;
                set_key_verified(client, l as u8, row, col, w)?;
                written += 1;
            }
        }
    }
    Ok(written)
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p flow2-core keymap`
Expected: 6 passed, including `apply_interrupted_by_unplug_errors_and_can_be_finished_later` and `verified_write_reports_unapplied_write`.

- [ ] **Step 5: Commit**

```bash
git add crates/flow2-core/src
git commit -m "feat(core): keymap read, verified write and apply"
```

---

### Task 5: Backlight

**Files:**
- Create: `crates/flow2-core/src/backlight.rs`
- Modify: `crates/flow2-core/src/lib.rs` (add `pub mod backlight;`)

**Interfaces:**
- Consumes: `ViaClient`, `via::{VALUE_BRIGHTNESS, VALUE_EFFECT}`.
- Produces: `enum Mode { Off, Steady, Breathing }`, `struct Backlight { mode: Mode, brightness: u8 }` (both `Serialize` and `Deserialize`, mode serialized lowercase), `level_from_percent(u8) -> u8`, `read(&mut ViaClient<T>) -> Result<Backlight>`, `apply(&mut ViaClient<T>, Backlight) -> Result<()>` (writes then saves).

- [ ] **Step 1: Write the failing tests.** Create `backlight.rs` containing only:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::MockDevice;

    #[test]
    fn percent_conversion_clamps() {
        assert_eq!(level_from_percent(0), 0);
        assert_eq!(level_from_percent(50), 127);
        assert_eq!(level_from_percent(100), 255);
        assert_eq!(level_from_percent(200), 255);
    }

    #[test]
    fn apply_then_read_round_trips() {
        let mut c = ViaClient::new(MockDevice::new(1, 1, 1));
        for b in [
            Backlight {
                mode: Mode::Breathing,
                brightness: 90,
            },
            Backlight {
                mode: Mode::Steady,
                brightness: 200,
            },
            Backlight {
                mode: Mode::Off,
                brightness: 0,
            },
        ] {
            apply(&mut c, b).unwrap();
            assert_eq!(read(&mut c).unwrap(), b);
        }
    }

    #[test]
    fn apply_saves_to_device() {
        let mut c = ViaClient::new(MockDevice::new(1, 1, 1));
        apply(
            &mut c,
            Backlight {
                mode: Mode::Steady,
                brightness: 10,
            },
        )
        .unwrap();
        assert!(c.into_transport().saved);
    }
}
```

Add `pub mod backlight;` to `lib.rs`.

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p flow2-core backlight`
Expected: FAIL, `Backlight`, `apply` and `read` do not exist.

- [ ] **Step 3: Add the implementation above the tests module**

```rust
use serde::{Deserialize, Serialize};

use crate::client::ViaClient;
use crate::error::Result;
use crate::transport::Transport;
use crate::via::{VALUE_BRIGHTNESS, VALUE_EFFECT};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Off,
    Steady,
    Breathing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Backlight {
    pub mode: Mode,
    /// 0..=255, the raw VIA value. Ignored when `mode` is `Off`.
    pub brightness: u8,
}

pub fn level_from_percent(percent: u8) -> u8 {
    (percent.min(100) as u16 * 255 / 100) as u8
}

pub fn read<T: Transport>(client: &mut ViaClient<T>) -> Result<Backlight> {
    let brightness = client.backlight_get(VALUE_BRIGHTNESS)?;
    let effect = client.backlight_get(VALUE_EFFECT)?;
    let mode = match (brightness, effect) {
        (0, _) => Mode::Off,
        (_, 0) => Mode::Steady,
        _ => Mode::Breathing,
    };
    Ok(Backlight { mode, brightness })
}

/// Writes the settings and saves them to the keyboard so they survive a power cycle.
pub fn apply<T: Transport>(client: &mut ViaClient<T>, b: Backlight) -> Result<()> {
    match b.mode {
        Mode::Off => client.backlight_set(VALUE_BRIGHTNESS, 0)?,
        Mode::Steady | Mode::Breathing => {
            client.backlight_set(VALUE_EFFECT, (b.mode == Mode::Breathing) as u8)?;
            client.backlight_set(VALUE_BRIGHTNESS, b.brightness)?;
        }
    }
    client.backlight_save()
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p flow2-core backlight`
Expected: 3 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/flow2-core/src
git commit -m "feat(core): backlight mode and brightness"
```

---

### Task 6: Layout parser

**Files:**
- Create: `crates/flow2-core/src/layout.rs`
- Modify: `crates/flow2-core/src/lib.rs` (add `pub mod layout;`)

**Interfaces:**
- Consumes: `Error::Layout`.
- Produces: `struct KeyDef { row: u8, col: u8, x: f32, y: f32, w: f32, h: f32 }`, `struct ModelDef { name: String, vendor_id: u16, product_id: u16, rows: u8, cols: u8, keys: Vec<KeyDef> }`, `parse_definition(&str) -> Result<ModelDef>`. Geometry follows KLE rules: `x` and `y` add to the cursor, `w` and `h` apply to the next key only, a row ends with `y += 1`, and a key label may carry a `\n` decal that is ignored.

- [ ] **Step 1: Write the failing tests.** Create `layout.rs` containing only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const TINY: &str = r#"{
        "name": "Tiny", "vendorId": "0x388d", "productId": "0x00AB",
        "matrix": {"rows": 2, "cols": 3},
        "layouts": {"keymap": [
            [{"w": 1.5}, "0,0", "0,1", {"x": 0.5}, "0,2"],
            [{"y": 0.25}, "1,0\nDecal", {"w": 2}, "1,1"]
        ]}
    }"#;

    #[test]
    fn parses_ids_and_matrix() {
        let m = parse_definition(TINY).unwrap();
        assert_eq!(
            (m.name.as_str(), m.vendor_id, m.product_id, m.rows, m.cols),
            ("Tiny", 0x388D, 0x00AB, 2, 3)
        );
    }

    #[test]
    fn computes_geometry() {
        let k = parse_definition(TINY).unwrap().keys;
        assert_eq!(
            k[0],
            KeyDef {
                row: 0,
                col: 0,
                x: 0.0,
                y: 0.0,
                w: 1.5,
                h: 1.0
            }
        );
        assert_eq!(k[1].x, 1.5);
        assert_eq!(k[2].x, 3.0); // 1.5 + 1.0 + 0.5 gap
        assert_eq!(
            k[3],
            KeyDef {
                row: 1,
                col: 0,
                x: 0.0,
                y: 1.25,
                w: 1.0,
                h: 1.0
            }
        );
        assert_eq!(k[4].w, 2.0);
    }

    #[test]
    fn rejects_key_outside_matrix() {
        let bad_json = TINY.replace("\"0,2\"", "\"0,9\"");
        assert!(matches!(parse_definition(&bad_json), Err(Error::Layout(_))));
    }

    #[test]
    fn rejects_missing_fields() {
        assert!(matches!(parse_definition("{}"), Err(Error::Layout(_))));
        assert!(matches!(
            parse_definition("not json"),
            Err(Error::Layout(_))
        ));
    }
}
```

Add `pub mod layout;` to `lib.rs`.

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p flow2-core layout`
Expected: FAIL, `parse_definition` does not exist.

- [ ] **Step 3: Add the implementation above the tests module**

```rust
//! Parses a VIA-style definition: name, ids, matrix size and KLE-style key geometry.
use serde_json::Value;

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct KeyDef {
    pub row: u8,
    pub col: u8,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelDef {
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub rows: u8,
    pub cols: u8,
    pub keys: Vec<KeyDef>,
}

fn bad(msg: impl Into<String>) -> Error {
    Error::Layout(msg.into())
}

fn hex16(v: &Value, field: &str) -> Result<u16> {
    let s = v
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| bad(format!("missing {field}")))?;
    u16::from_str_radix(s.trim_start_matches("0x").trim_start_matches("0X"), 16)
        .map_err(|_| bad(format!("bad {field}: {s}")))
}

fn matrix_dim(v: &Value, field: &str) -> Result<u8> {
    v.pointer(&format!("/matrix/{field}"))
        .and_then(Value::as_u64)
        .and_then(|n| u8::try_from(n).ok())
        .ok_or_else(|| bad(format!("missing matrix.{field}")))
}

pub fn parse_definition(json: &str) -> Result<ModelDef> {
    let v: Value = serde_json::from_str(json).map_err(|e| bad(e.to_string()))?;
    let name = v
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| bad("missing name"))?
        .to_string();
    let (rows, cols) = (matrix_dim(&v, "rows")?, matrix_dim(&v, "cols")?);
    let kle = v
        .pointer("/layouts/keymap")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("missing layouts.keymap"))?;

    let mut keys = Vec::new();
    let mut y = 0f32;
    for row in kle {
        let items = row
            .as_array()
            .ok_or_else(|| bad("keymap row is not an array"))?;
        let (mut x, mut w, mut h) = (0f32, 1f32, 1f32);
        for item in items {
            match item {
                Value::Object(props) => {
                    let num = |k: &str| props.get(k).and_then(Value::as_f64).map(|n| n as f32);
                    x += num("x").unwrap_or(0.0);
                    y += num("y").unwrap_or(0.0);
                    w = num("w").unwrap_or(w);
                    h = num("h").unwrap_or(h);
                }
                Value::String(label) => {
                    // "row,col", optionally followed by "\n<decal>" which we ignore.
                    let pos = label.lines().next().unwrap_or("");
                    let (r, c) = pos
                        .split_once(',')
                        .ok_or_else(|| bad(format!("bad key label {label:?}")))?;
                    let parse = |s: &str| {
                        s.trim()
                            .parse::<u8>()
                            .map_err(|_| bad(format!("bad key label {label:?}")))
                    };
                    let (r, c) = (parse(r)?, parse(c)?);
                    if r >= rows || c >= cols {
                        return Err(bad(format!("key {r},{c} outside {rows}x{cols} matrix")));
                    }
                    keys.push(KeyDef {
                        row: r,
                        col: c,
                        x,
                        y,
                        w,
                        h,
                    });
                    x += w;
                    w = 1.0;
                    h = 1.0;
                }
                _ => return Err(bad("unexpected item in keymap row")),
            }
        }
        y += 1.0;
    }

    Ok(ModelDef {
        name,
        vendor_id: hex16(&v, "vendorId")?,
        product_id: hex16(&v, "productId")?,
        rows,
        cols,
        keys,
    })
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p flow2-core layout`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/flow2-core/src
git commit -m "feat(core): parse VIA-style layout definitions"
```

---

### Task 7: Model registry and the 84-key layout file

**Files:**
- Create: `layouts/flow2-mac-84.json`, `crates/flow2-core/src/models.rs`
- Modify: `crates/flow2-core/src/lib.rs` (add `pub mod models;`)

**Interfaces:**
- Consumes: `layout::{parse_definition, ModelDef}`.
- Produces: `struct Model { id, label, verified: bool, .. }` with `definition() -> Result<ModelDef>`, `const MODELS: &[Model]`, `by_product_id(u16) -> Option<(&'static Model, ModelDef)>`. First entry: id `flow2-mac-84`, product id `0x0028`, verified true (matches the keyboard enumerated on 2026-09-29).

The geometry file is our own, transcribed from the physical layout (six rows, each 16 units wide, 84 keys). Matrix positions are hardware facts.

- [ ] **Step 1: Create `layouts/flow2-mac-84.json`**

```json
{
  "name": "Flow2@Lofree",
  "vendorId": "0x388d",
  "productId": "0x0028",
  "matrix": { "rows": 6, "cols": 15 },
  "layouts": {
    "keymap": [
      [{"w":1.5},"0,0","0,1","0,2","0,3","0,4","0,5","0,6","0,7","0,8","0,9","0,10","0,11","0,12","0,13",{"w":1.5},"0,14"],
      ["1,0","1,1","1,2","1,3","1,4","1,5","1,6","1,7","1,8","1,9","1,10","1,11","1,12",{"w":2},"1,13","1,14"],
      [{"w":1.5},"2,0","2,1","2,2","2,3","2,4","2,5","2,6","2,7","2,8","2,9","2,10","2,11","2,12",{"w":1.5},"2,13","2,14"],
      [{"w":1.75},"3,0","3,1","3,2","3,3","3,4","3,5","3,6","3,7","3,8","3,9","3,10","3,11",{"w":2.25},"3,13","3,14"],
      [{"w":2.25},"4,0","4,2","4,3","4,4","4,5","4,6","4,7","4,8","4,9","4,10","4,11",{"w":1.75},"4,12","4,13","4,14"],
      ["5,0","5,1","5,2",{"w":1.25},"5,3",{"w":5},"5,6",{"w":1.25},"5,9",{"w":1.25},"5,10",{"w":1.25},"5,11","5,12","5,13","5,14"]
    ]
  }
}
```

- [ ] **Step 2: Write the failing tests.** Create `models.rs` containing only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registered_layout_parses() {
        for m in MODELS {
            m.definition().unwrap_or_else(|e| panic!("{}: {e}", m.id));
        }
    }

    #[test]
    fn flow2_mac_84_has_84_keys_in_16_unit_rows() {
        let def = MODELS[0].definition().unwrap();
        assert_eq!((def.rows, def.cols, def.product_id), (6, 15, 0x0028));
        assert_eq!(def.keys.len(), 84);
        for row in 0..def.rows {
            let width: f32 = def.keys.iter().filter(|k| k.row == row).map(|k| k.w).sum();
            assert_eq!(width, 16.0, "row {row}");
        }
    }

    #[test]
    fn looks_up_by_product_id() {
        assert_eq!(by_product_id(0x0028).unwrap().0.id, "flow2-mac-84");
        assert!(by_product_id(0xFFFF).is_none());
    }
}
```

Add `pub mod models;` to `lib.rs`.

- [ ] **Step 3: Run the tests to see them fail**

Run: `cargo test -p flow2-core models`
Expected: FAIL, `MODELS` and `by_product_id` do not exist.

- [ ] **Step 4: Add the implementation above the tests module**

```rust
//! Registry of supported models. Adding a model means one entry here and one file in `layouts/`.
use crate::error::Result;
use crate::layout::{parse_definition, ModelDef};

pub struct Model {
    pub id: &'static str,
    pub label: &'static str,
    /// True only for models tested on real hardware.
    pub verified: bool,
    json: &'static str,
}

impl Model {
    pub fn definition(&self) -> Result<ModelDef> {
        parse_definition(self.json)
    }
}

pub const MODELS: &[Model] = &[Model {
    id: "flow2-mac-84",
    label: "Flow 2 Mac 84",
    verified: true,
    json: include_str!("../../../layouts/flow2-mac-84.json"),
}];

pub fn by_product_id(product_id: u16) -> Option<(&'static Model, ModelDef)> {
    MODELS.iter().find_map(|m| {
        m.definition()
            .ok()
            .filter(|d| d.product_id == product_id)
            .map(|d| (m, d))
    })
}
```

- [ ] **Step 5: Run the tests to see them pass**

Run: `cargo test -p flow2-core models`
Expected: 3 passed (84 keys, every row 16 units wide, lookup by product id).

- [ ] **Step 6: Commit**

```bash
git add layouts crates/flow2-core/src
git commit -m "feat(core): model registry with the Flow 2 Mac 84 layout"
```

---

### Task 8: Keycode legends

**Files:**
- Create: `crates/flow2-core/src/keycodes.rs`
- Modify: `crates/flow2-core/src/lib.rs` (add `pub mod keycodes;`)

**Interfaces:**
- Produces: `enum Variant { Win, Mac }`, `legend(code: u16, variant: Variant) -> String`, constants `KC_NO`, `BT1`, `BT2`, `BT3`, `DONGLE_2G4`. Unknown codes render as `0x1234`, never hidden. Layer keycodes use the QMK 0.19+ ranges (`TO` `0x5200`, `MO` `0x5220`, `TG` `0x5260`); the wireless codes `0x7793` to `0x7795` and `0x7785` come from community research and are confirmed by read-back on hardware (Task 11).

- [ ] **Step 1: Write the failing tests.** Create `keycodes.rs` containing only:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use Variant::*;

    #[test]
    fn letters_digits_and_function_keys() {
        assert_eq!(legend(0x04, Win), "A");
        assert_eq!(legend(0x1D, Win), "Z");
        assert_eq!(legend(0x1E, Win), "1");
        assert_eq!(legend(0x27, Win), "0");
        assert_eq!(legend(0x3A, Win), "F1");
        assert_eq!(legend(0x45, Win), "F12");
    }

    #[test]
    fn modifier_legends_depend_on_variant() {
        assert_eq!(legend(0xE2, Win), "Alt");
        assert_eq!(legend(0xE2, Mac), "Opt");
        assert_eq!(legend(0xE3, Win), "Win");
        assert_eq!(legend(0xE7, Mac), "R Cmd");
    }

    #[test]
    fn wireless_and_layer_codes() {
        assert_eq!(legend(BT1, Mac), "BT1");
        assert_eq!(legend(DONGLE_2G4, Mac), "2.4G");
        assert_eq!(legend(0x5221, Mac), "MO(1)");
        assert_eq!(legend(0x5202, Mac), "TO(2)");
        assert_eq!(legend(0x5263, Mac), "TG(3)");
    }

    #[test]
    fn unknown_codes_show_hex() {
        assert_eq!(legend(0x1234, Mac), "0x1234");
    }
}
```

Add `pub mod keycodes;` to `lib.rs`.

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p flow2-core keycodes`
Expected: FAIL, `legend` and `Variant` do not exist.

- [ ] **Step 3: Add the implementation above the tests module**

```rust
//! Keycode legends for display. Unknown codes fall back to hex, nothing is hidden.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Variant {
    Win,
    Mac,
}

pub const KC_NO: u16 = 0x0000;

/// Layer keycodes use the QMK 0.19+ ranges (VIA protocol 12).
const QK_TO: u16 = 0x5200;
const QK_MOMENTARY: u16 = 0x5220;
const QK_TOGGLE_LAYER: u16 = 0x5260;

/// Wireless switch keys. VIA often loses these on save and load (KC_NO), we write them raw.
pub const BT1: u16 = 0x7793;
pub const BT2: u16 = 0x7794;
pub const BT3: u16 = 0x7795;
pub const DONGLE_2G4: u16 = 0x7785;

pub fn legend(code: u16, variant: Variant) -> String {
    let (alt, gui) = match variant {
        Variant::Win => ("Alt", "Win"),
        Variant::Mac => ("Opt", "Cmd"),
    };
    let s = |t: &str| t.to_string();
    match code {
        KC_NO => s("None"),
        0x0001 => s("Trns"),
        0x04..=0x1D => ((b'A' + (code - 0x04) as u8) as char).to_string(),
        0x1E..=0x26 => ((b'1' + (code - 0x1E) as u8) as char).to_string(),
        0x27 => s("0"),
        0x28 => s("Enter"),
        0x29 => s("Esc"),
        0x2A => s("Bksp"),
        0x2B => s("Tab"),
        0x2C => s("Space"),
        0x2D => s("-"),
        0x2E => s("="),
        0x2F => s("["),
        0x30 => s("]"),
        0x31 => s("\\"),
        0x33 => s(";"),
        0x34 => s("'"),
        0x35 => s("`"),
        0x36 => s(","),
        0x37 => s("."),
        0x38 => s("/"),
        0x39 => s("Caps"),
        0x3A..=0x45 => format!("F{}", code - 0x39),
        0x46 => s("PrtSc"),
        0x47 => s("ScrLk"),
        0x48 => s("Pause"),
        0x49 => s("Ins"),
        0x4A => s("Home"),
        0x4B => s("PgUp"),
        0x4C => s("Del"),
        0x4D => s("End"),
        0x4E => s("PgDn"),
        0x4F => s("Right"),
        0x50 => s("Left"),
        0x51 => s("Down"),
        0x52 => s("Up"),
        0xE0 => s("Ctrl"),
        0xE1 => s("Shift"),
        0xE2 => s(alt),
        0xE3 => s(gui),
        0xE4 => s("R Ctrl"),
        0xE5 => s("R Shift"),
        0xE6 => format!("R {alt}"),
        0xE7 => format!("R {gui}"),
        BT1 => s("BT1"),
        BT2 => s("BT2"),
        BT3 => s("BT3"),
        DONGLE_2G4 => s("2.4G"),
        c if (QK_TO..QK_TO + 0x20).contains(&c) => format!("TO({})", c - QK_TO),
        c if (QK_MOMENTARY..QK_MOMENTARY + 0x20).contains(&c) => {
            format!("MO({})", c - QK_MOMENTARY)
        }
        c if (QK_TOGGLE_LAYER..QK_TOGGLE_LAYER + 0x20).contains(&c) => {
            format!("TG({})", c - QK_TOGGLE_LAYER)
        }
        c => format!("0x{c:04X}"),
    }
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p flow2-core keycodes`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/flow2-core/src
git commit -m "feat(core): keycode legends for Windows and Mac variants"
```

---

### Task 9: Profiles and factory backup

**Files:**
- Create: `crates/flow2-core/src/profile.rs`
- Modify: `crates/flow2-core/src/lib.rs` (add `pub mod profile;`)

**Interfaces:**
- Consumes: `keymap::{read, apply, Keymap}`, `backlight::{read, apply, Backlight}`, `layout::ModelDef`, `ViaClient`.
- Produces: `SCHEMA: u32 = 1`, `struct Profile { schema, name, model, rows, cols, layers, backlight }` with `to_json`, `from_json` (checks schema and that every layer has `rows * cols` keys), `save(&Path)`, `load(&Path)`; `capture(&mut ViaClient<T>, model_id, &ModelDef, name) -> Result<Profile>`; `apply(&mut ViaClient<T>, &ModelDef, &Profile) -> Result<usize>` (refuses a different matrix size); `factory_backup_path() -> Option<PathBuf>` (`<config dir>/openlofree/factory-backup.json`); `ensure_factory_backup(&mut ViaClient<T>, model_id, &ModelDef, &Path) -> Result<bool>` (true when created now, never overwrites).

- [ ] **Step 1: Write the failing tests.** Create `profile.rs` containing only:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::backlight::Mode;
    use crate::transport::mock::MockDevice;

    fn def() -> ModelDef {
        ModelDef {
            name: "T".into(),
            vendor_id: 0x388D,
            product_id: 1,
            rows: 2,
            cols: 3,
            keys: vec![],
        }
    }

    fn client() -> ViaClient<MockDevice> {
        ViaClient::new(MockDevice::new(2, 3, 2))
    }

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("openlofree-test-{}-{name}", std::process::id()));
        let _ = fs::remove_file(&p);
        p
    }

    #[test]
    fn json_round_trip() {
        let p = capture(&mut client(), "test", &def(), "Mine").unwrap();
        assert_eq!(Profile::from_json(&p.to_json().unwrap()).unwrap(), p);
    }

    #[test]
    fn rejects_wrong_schema_and_shape() {
        let mut p = capture(&mut client(), "test", &def(), "Mine").unwrap();
        p.schema = 99;
        assert!(matches!(
            Profile::from_json(&p.to_json().unwrap()),
            Err(Error::Profile(_))
        ));
        p.schema = SCHEMA;
        p.layers[0].pop();
        assert!(matches!(
            Profile::from_json(&p.to_json().unwrap()),
            Err(Error::Profile(_))
        ));
        assert!(matches!(Profile::from_json("{}"), Err(Error::Profile(_))));
    }

    #[test]
    fn apply_restores_a_captured_profile() {
        let mut c = client();
        let original = capture(&mut c, "test", &def(), "Original").unwrap();
        c.set_keycode(0, 0, 0, 0x0004).unwrap();
        backlight::apply(
            &mut c,
            Backlight {
                mode: Mode::Breathing,
                brightness: 40,
            },
        )
        .unwrap();
        assert_eq!(apply(&mut c, &def(), &original).unwrap(), 1);
        assert_eq!(
            capture(&mut c, "test", &def(), "Original").unwrap(),
            original
        );
    }

    #[test]
    fn apply_refuses_a_profile_for_a_different_matrix() {
        let mut c = client();
        let mut p = capture(&mut c, "test", &def(), "Other").unwrap();
        p.cols = 4;
        assert!(matches!(apply(&mut c, &def(), &p), Err(Error::Profile(_))));
    }

    #[test]
    fn factory_backup_is_created_once_and_never_overwritten() {
        let path = tmp("factory.json");
        let mut c = client();
        assert!(ensure_factory_backup(&mut c, "test", &def(), &path).unwrap());
        c.set_keycode(0, 0, 0, 0x0004).unwrap();
        assert!(!ensure_factory_backup(&mut c, "test", &def(), &path).unwrap());
        assert_eq!(Profile::load(&path).unwrap().layers[0][0], 0);
        let _ = fs::remove_file(path);
    }
}
```

Add `pub mod profile;` to `lib.rs`.

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p flow2-core profile`
Expected: FAIL, `Profile`, `capture` and the others do not exist.

- [ ] **Step 3: Add the implementation above the tests module**

```rust
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::backlight::{self, Backlight};
use crate::client::ViaClient;
use crate::error::{Error, Result};
use crate::keymap::{self, Keymap};
use crate::layout::ModelDef;
use crate::transport::Transport;

pub const SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub schema: u32,
    pub name: String,
    /// Model registry id, for example "flow2-mac-84".
    pub model: String,
    pub rows: u8,
    pub cols: u8,
    /// layers[l][row * cols + col]
    pub layers: Vec<Vec<u16>>,
    pub backlight: Backlight,
}

impl Profile {
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| Error::Profile(e.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self> {
        let p: Profile = serde_json::from_str(json).map_err(|e| Error::Profile(e.to_string()))?;
        if p.schema != SCHEMA {
            return Err(Error::Profile(format!("unsupported schema {}", p.schema)));
        }
        let want = p.rows as usize * p.cols as usize;
        if p.layers.is_empty() || p.layers.iter().any(|l| l.len() != want) {
            return Err(Error::Profile(format!("every layer must have {want} keys")));
        }
        Ok(p)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| Error::Profile(e.to_string()))?;
        }
        fs::write(path, self.to_json()?).map_err(|e| Error::Profile(e.to_string()))
    }

    pub fn load(path: &Path) -> Result<Self> {
        Self::from_json(&fs::read_to_string(path).map_err(|e| Error::Profile(e.to_string()))?)
    }
}

/// Reads the keyboard's current keymap and backlight into a profile.
pub fn capture<T: Transport>(
    client: &mut ViaClient<T>,
    model_id: &str,
    def: &ModelDef,
    name: &str,
) -> Result<Profile> {
    let km = keymap::read(client, def.rows, def.cols)?;
    Ok(Profile {
        schema: SCHEMA,
        name: name.to_string(),
        model: model_id.to_string(),
        rows: def.rows,
        cols: def.cols,
        layers: km.layers,
        backlight: backlight::read(client)?,
    })
}

/// Writes a profile to the keyboard. Returns how many keys changed.
/// Refuses a profile whose matrix size differs from the keyboard's.
pub fn apply<T: Transport>(
    client: &mut ViaClient<T>,
    def: &ModelDef,
    p: &Profile,
) -> Result<usize> {
    if (p.rows, p.cols) != (def.rows, def.cols) {
        return Err(Error::Profile(format!(
            "profile matrix is {}x{}, keyboard is {}x{}",
            p.rows, p.cols, def.rows, def.cols
        )));
    }
    let changed = keymap::apply(
        client,
        &Keymap {
            rows: p.rows,
            cols: p.cols,
            layers: p.layers.clone(),
        },
    )?;
    backlight::apply(client, p.backlight)?;
    Ok(changed)
}

pub fn factory_backup_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("openlofree").join("factory-backup.json"))
}

/// Saves the keyboard's current state once, before the first write. Never overwrites.
/// Returns true if a backup was created now.
pub fn ensure_factory_backup<T: Transport>(
    client: &mut ViaClient<T>,
    model_id: &str,
    def: &ModelDef,
    path: &Path,
) -> Result<bool> {
    if path.exists() {
        return Ok(false);
    }
    capture(client, model_id, def, "Factory backup")?.save(path)?;
    Ok(true)
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p flow2-core profile`
Expected: 5 passed, including `apply_refuses_a_profile_for_a_different_matrix` and `rejects_wrong_schema_and_shape`.

- [ ] **Step 5: Commit**

```bash
git add crates/flow2-core/src
git commit -m "feat(core): profiles, apply and one-time factory backup"
```

---

### Task 10: Battery (macOS)

**Files:**
- Create: `crates/flow2-core/src/battery.rs`
- Modify: `crates/flow2-core/src/lib.rs` (add `pub mod battery;`)

**Interfaces:**
- Produces: `parse_system_profiler(json: &str, name_filter: &str) -> Option<u8>` (case-insensitive name match, keys `device_batteryLevelMain` then `device_batteryLevel`, skips a matching device that has no battery and keeps looking), `read() -> Option<u8>` (macOS runs `/usr/sbin/system_profiler SPBluetoothDataType -json`; other platforms return `None`).

Windows and Linux battery is not implemented here on purpose: it needs research on real machines and belongs to the next plan.

- [ ] **Step 1: Write the failing tests.** Create `battery.rs` containing only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{"SPBluetoothDataType":[{"device_connected":[
        {"Magic Mouse":{"device_batteryLevelMain":"55%"}},
        {"Lofree Flow2":{"device_address":"AA","device_batteryLevelMain":"87%"}}
    ]}]}"#;

    #[test]
    fn finds_matching_device_case_insensitively() {
        assert_eq!(parse_system_profiler(SAMPLE, "flow"), Some(87));
        assert_eq!(parse_system_profiler(SAMPLE, "FLOW"), Some(87));
    }

    #[test]
    fn falls_back_to_second_key() {
        let json = r#"{"SPBluetoothDataType":[{"device_connected":[{"Flow":{"device_batteryLevel":"12%"}}]}]}"#;
        assert_eq!(parse_system_profiler(json, "flow"), Some(12));
    }

    #[test]
    fn skips_matching_device_without_battery_and_continues() {
        let json = r#"{"SPBluetoothDataType":[{"device_connected":[
            {"Flow A":{"device_address":"1"}},
            {"Flow B":{"device_batteryLevelMain":"40%"}}]}]}"#;
        assert_eq!(parse_system_profiler(json, "flow"), Some(40));
    }

    #[test]
    fn returns_none_for_missing_or_invalid_data() {
        assert_eq!(parse_system_profiler(SAMPLE, "keychron"), None);
        assert_eq!(parse_system_profiler("not json", "flow"), None);
        assert_eq!(parse_system_profiler("{}", "flow"), None);
    }
}
```

Add `pub mod battery;` to `lib.rs`.

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p flow2-core battery`
Expected: FAIL, `parse_system_profiler` does not exist.

- [ ] **Step 3: Add the implementation above the tests module**

```rust
//! Battery level. Only macOS is implemented, see the plan for Windows and Linux.
use serde_json::Value;

/// Keys observed in `system_profiler SPBluetoothDataType -json` (reference project, real hardware).
const LEVEL_KEYS: [&str; 2] = ["device_batteryLevelMain", "device_batteryLevel"];

/// Finds the first connected device whose name contains `name_filter` and reports its battery percent.
pub fn parse_system_profiler(json: &str, name_filter: &str) -> Option<u8> {
    let root: Value = serde_json::from_str(json).ok()?;
    let connected = root
        .pointer("/SPBluetoothDataType/0/device_connected")?
        .as_array()?;
    let needle = name_filter.to_lowercase();
    connected.iter().find_map(|wrapper| {
        let (name, info) = wrapper.as_object()?.iter().next()?;
        if !name.to_lowercase().contains(&needle) {
            return None;
        }
        LEVEL_KEYS.iter().find_map(|k| {
            info.get(k)?
                .as_str()?
                .trim_end_matches('%')
                .trim()
                .parse::<u8>()
                .ok()
        })
    })
}

#[cfg(target_os = "macos")]
pub fn read() -> Option<u8> {
    // ponytail: no timeout, Command::output drains both pipes so it cannot deadlock. Add one if system_profiler hangs.
    let out = std::process::Command::new("/usr/sbin/system_profiler")
        .args(["SPBluetoothDataType", "-json"])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| parse_system_profiler(&String::from_utf8_lossy(&out.stdout), "flow"))
        .flatten()
}

#[cfg(not(target_os = "macos"))]
pub fn read() -> Option<u8> {
    None
}
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p flow2-core battery`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/flow2-core/src
git commit -m "feat(core): battery parser and macOS reader"
```

---

### Task 11: HID transport and hardware checklist

**Files:**
- Create: `crates/flow2-core/src/hid.rs`, `docs/hardware-checklist.md`
- Modify: `crates/flow2-core/src/lib.rs` (add `pub mod hid;`)

**Interfaces:**
- Consumes: `Transport`, `via::{Report, REPORT_SIZE}`, `Error`.
- Produces: `VENDOR_ID: u16 = 0x388D`, `struct DeviceInfo { vendor_id, product_id, product }`, `struct HidTransport` (implements `Transport`), `find() -> Result<(HidTransport, DeviceInfo)>` (first interface with usage page `0xFF60` and usage `0x61` on a `0x388D` device, `Error::NotFound` otherwise). `exchange` writes 33 bytes (leading report id `0x00`), then reads up to 1000 ms and returns `Error::Timeout` on silence.

There is no unit test: this module needs a keyboard. It is checked by compiling and by the checklist.

- [ ] **Step 1: Create `hid.rs`**

```rust
//! Real transport over hidapi. Not unit-tested: it needs hardware, see docs/hardware-checklist.md.
use hidapi::{HidApi, HidDevice};

use crate::error::{Error, Result};
use crate::transport::Transport;
use crate::via::{Report, REPORT_SIZE};

pub const VENDOR_ID: u16 = 0x388D;
const USAGE_PAGE: u16 = 0xFF60;
const USAGE: u16 = 0x61;
const TIMEOUT_MS: i32 = 1000;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub product: String,
}

/// Field order matters: `dev` must drop before `_api`.
pub struct HidTransport {
    dev: HidDevice,
    _api: HidApi,
}

fn io(e: impl std::fmt::Display) -> Error {
    Error::Io(e.to_string())
}

/// Opens the first VIA raw HID interface of a Lofree keyboard (VID 0x388D).
pub fn find() -> Result<(HidTransport, DeviceInfo)> {
    let api = HidApi::new().map_err(io)?;
    let info = api
        .device_list()
        .find(|d| d.vendor_id() == VENDOR_ID && d.usage_page() == USAGE_PAGE && d.usage() == USAGE)
        .ok_or(Error::NotFound)?;
    let dev = info.open_device(&api).map_err(io)?;
    let device = DeviceInfo {
        vendor_id: info.vendor_id(),
        product_id: info.product_id(),
        product: info.product_string().unwrap_or_default().to_string(),
    };
    Ok((HidTransport { dev, _api: api }, device))
}

impl Transport for HidTransport {
    fn exchange(&mut self, out: &Report) -> Result<Report> {
        let mut buf = [0u8; REPORT_SIZE + 1]; // byte 0 is the report id, VIA uses none
        buf[1..].copy_from_slice(out);
        self.dev.write(&buf).map_err(io)?;
        let mut reply = [0u8; REPORT_SIZE];
        match self.dev.read_timeout(&mut reply, TIMEOUT_MS).map_err(io)? {
            0 => Err(Error::Timeout),
            _ => Ok(reply),
        }
    }
}
```

Add `pub mod hid;` to `lib.rs`.

- [ ] **Step 2: Verify it builds and the whole suite still passes**

Run: `cargo clippy --all-targets -- -D warnings && cargo test`
Expected: no warnings, all tests pass.

- [ ] **Step 3: Create `docs/hardware-checklist.md`**

```markdown
# Hardware checklist

Run on a real keyboard over USB, wired-mode switch position. Note the model, OS and result of every line. Do not run anything here on a keyboard you cannot afford to reflash.

Before starting: close VIA in the browser and Lofree's configurator.

- [ ] `flow2ctl probe` prints the device line, the model id, a VIA protocol number and the layer count.
- [ ] `flow2ctl keys --layer 0` prints 84 lines for the 84-key model and the legends look right (Esc top left, Space in the middle of the bottom row).
- [ ] `flow2ctl set-key 0 0 0 0x0004` prints ok and a factory backup path on the first run only. Run again with the original code to restore the key (read it from `keys` first).
- [ ] The factory backup file exists and `flow2ctl apply <that file>` reports `0 keys changed`.
- [ ] Wireless switch keys: write `0x7793` to a spare key, read it back with `keys`, then restore. Record whether it survives.
- [ ] `flow2ctl light off`, `light on`, `light breathing`, `brightness 30`, `brightness 100` change the backlight as expected and persist after unplugging and replugging.
- [ ] `flow2ctl scan` output saved into `docs/protocol.md`.
- [ ] Unplug during `flow2ctl apply` on a two-key profile difference, replug, run apply again: the second run finishes and reports the remaining count.
- [ ] macOS only: `flow2ctl battery` over Bluetooth prints a percentage.
- [ ] Known limit to record: with two Lofree devices attached (keyboard by USB plus a dongle) `find()` takes the first VIA interface it sees.
```

- [ ] **Step 4: Commit**

```bash
git add docs/hardware-checklist.md crates/flow2-core/src
git commit -m "feat(core): hidapi transport and hardware checklist"
```

---

### Task 12: flow2ctl and hardware bring-up

**Files:**
- Create: `crates/flow2-cli/Cargo.toml`, `crates/flow2-cli/src/main.rs`, `docs/protocol.md`
- Modify: `Cargo.toml` (add the CLI member), `crates/flow2-core/src/lib.rs` (must equal the final file below)

**Interfaces:**
- Consumes: the whole of `flow2-core`.
- Produces: the `flow2ctl` binary with subcommands `probe`, `keys [--layer N]`, `set-key <layer> <row> <col> <hexcode>`, `light <off|on|breathing>`, `brightness <0-100>`, `battery`, `backup <file>`, `apply <file>`, `scan`. Every write subcommand runs the factory-backup guard first. A `Timeout` error prints a hint about wired mode and other programs holding the keyboard.

- [ ] **Step 1: Add the CLI crate**

Change the root `Cargo.toml` to:
```toml
[workspace]
resolver = "2"
members = ["crates/flow2-core", "crates/flow2-cli"]
```

`crates/flow2-cli/Cargo.toml`:
```toml
[package]
name = "flow2-cli"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "flow2ctl: command line for Lofree Flow 2 keyboards"

[[bin]]
name = "flow2ctl"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
flow2-core = { path = "../flow2-core" }
```

`crates/flow2-cli/src/main.rs`:
```rust
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use flow2_core::backlight::{self, Backlight, Mode};
use flow2_core::client::ViaClient;
use flow2_core::hid::{self, DeviceInfo, HidTransport};
use flow2_core::keycodes::{legend, Variant};
use flow2_core::layout::ModelDef;
use flow2_core::{battery, keymap, models, profile, via, Error, Result};

#[derive(Parser)]
#[command(
    name = "flow2ctl",
    about = "Configure Lofree Flow 2 keyboards over USB"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Clone, Copy, ValueEnum)]
enum LightArg {
    Off,
    On,
    Breathing,
}

#[derive(Subcommand)]
enum Cmd {
    /// Check the connection and print protocol version, layers and model.
    Probe,
    /// Print one layer as key legends.
    Keys {
        #[arg(long, default_value_t = 0)]
        layer: u8,
    },
    /// Set one key. `code` is a hex keycode such as 0x0029. The write is read back and verified.
    SetKey {
        layer: u8,
        row: u8,
        col: u8,
        #[arg(value_parser = parse_hex)]
        code: u16,
    },
    /// Backlight mode.
    Light { mode: LightArg },
    /// Backlight brightness, 0 to 100.
    Brightness {
        #[arg(value_parser = clap::value_parser!(u8).range(0..=100))]
        percent: u8,
    },
    /// Battery level (Bluetooth, macOS only for now).
    Battery,
    /// Save the keyboard's current state to a profile file.
    Backup { file: PathBuf },
    /// Write a profile file to the keyboard.
    Apply { file: PathBuf },
    /// Diagnostic: list VIA custom channels that answer with data.
    Scan,
}

fn parse_hex(s: &str) -> std::result::Result<u16, String> {
    u16::from_str_radix(s.trim_start_matches("0x"), 16).map_err(|e| e.to_string())
}

struct Session {
    client: ViaClient<HidTransport>,
    model_id: &'static str,
    def: ModelDef,
    info: DeviceInfo,
}

fn connect() -> Result<Session> {
    let (transport, info) = hid::find()?;
    let (model, def) = models::by_product_id(info.product_id).ok_or_else(|| {
        Error::BadReply(format!(
            "unsupported model, product id {:#06x} ({}). Please open an issue at github.com/kills1nt/openlofree",
            info.product_id, info.product
        ))
    })?;
    if !model.verified {
        eprintln!("note: {} is not verified on real hardware", model.label);
    }
    Ok(Session {
        client: ViaClient::new(transport),
        model_id: model.id,
        def,
        info,
    })
}

impl Session {
    /// Call before the first write of a run. Creates the factory backup once.
    fn guard_write(&mut self) -> Result<()> {
        let Some(path) = profile::factory_backup_path() else {
            return Ok(());
        };
        if profile::ensure_factory_backup(&mut self.client, self.model_id, &self.def, &path)? {
            println!("saved factory backup to {}", path.display());
        }
        Ok(())
    }
}

fn run(cli: Cli) -> Result<()> {
    if let Cmd::Battery = cli.cmd {
        match battery::read() {
            Some(pct) => println!("battery: {pct}%"),
            None => println!("battery: unknown (needs a Bluetooth connection, macOS only for now)"),
        }
        return Ok(());
    }
    let mut s = connect()?;
    match cli.cmd {
        Cmd::Battery => unreachable!(),
        Cmd::Probe => {
            println!(
                "device: {} ({:04x}:{:04x})",
                s.info.product, s.info.vendor_id, s.info.product_id
            );
            println!("model: {}", s.model_id);
            println!("via protocol: {}", s.client.protocol_version()?);
            println!("layers: {}", s.client.layer_count()?);
        }
        Cmd::Keys { layer } => {
            let km = keymap::read(&mut s.client, s.def.rows, s.def.cols)?;
            let keys = km
                .layers
                .get(layer as usize)
                .ok_or_else(|| Error::Profile(format!("no layer {layer}")))?;
            for k in &s.def.keys {
                let code = keys[k.row as usize * s.def.cols as usize + k.col as usize];
                println!(
                    "{},{}\t{:#06x}\t{}",
                    k.row,
                    k.col,
                    code,
                    legend(code, Variant::Mac)
                );
            }
        }
        Cmd::SetKey {
            layer,
            row,
            col,
            code,
        } => {
            s.guard_write()?;
            keymap::set_key_verified(&mut s.client, layer, row, col, code)?;
            println!(
                "ok: layer {layer} key {row},{col} = {code:#06x} ({})",
                legend(code, Variant::Mac)
            );
        }
        Cmd::Light { mode } => {
            s.guard_write()?;
            let current = backlight::read(&mut s.client)?;
            let brightness = if current.brightness == 0 {
                255
            } else {
                current.brightness
            };
            let mode = match mode {
                LightArg::Off => Mode::Off,
                LightArg::On => Mode::Steady,
                LightArg::Breathing => Mode::Breathing,
            };
            backlight::apply(&mut s.client, Backlight { mode, brightness })?;
            println!("ok");
        }
        Cmd::Brightness { percent } => {
            s.guard_write()?;
            let current = backlight::read(&mut s.client)?;
            let mode = if percent == 0 {
                Mode::Off
            } else if current.mode == Mode::Off {
                Mode::Steady
            } else {
                current.mode
            };
            backlight::apply(
                &mut s.client,
                Backlight {
                    mode,
                    brightness: backlight::level_from_percent(percent),
                },
            )?;
            println!("ok");
        }
        Cmd::Backup { file } => {
            profile::capture(&mut s.client, s.model_id, &s.def, "Backup")?.save(&file)?;
            println!("saved {}", file.display());
        }
        Cmd::Apply { file } => {
            let p = profile::Profile::load(&file)?;
            if p.model != s.model_id {
                return Err(Error::Profile(format!(
                    "profile is for {}, keyboard is {}",
                    p.model, s.model_id
                )));
            }
            s.guard_write()?;
            println!(
                "ok: {} keys changed",
                profile::apply(&mut s.client, &s.def, &p)?
            );
        }
        Cmd::Scan => {
            for channel in 0u8..=5 {
                for value in 1u8..=15 {
                    let mut r = via::backlight_get(value);
                    r[1] = channel;
                    if let Ok(reply) = s.client.raw(&r) {
                        if reply[0] == via::CMD_CUSTOM_GET_VALUE
                            && reply[3..].iter().any(|&b| b != 0)
                        {
                            println!("channel {channel} value {value}: {:02X?}", &reply[..8]);
                        }
                    }
                }
            }
            println!("scan done (channels 0-5, values 1-15, only answers with data are listed)");
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            if matches!(e, Error::Timeout) {
                eprintln!(
                    "The keyboard was found but did not answer. Check that its switch is in wired (USB) mode, \
                     close anything else using it (VIA in a browser, Lofree's configurator) and try another cable or port."
                );
            }
            ExitCode::FAILURE
        }
    }
}
```

Check that `crates/flow2-core/src/lib.rs` now reads exactly:
```rust
pub mod backlight;
pub mod battery;
pub mod client;
pub mod error;
pub mod hid;
pub mod keycodes;
pub mod keymap;
pub mod layout;
pub mod models;
pub mod profile;
pub mod transport;
pub mod via;

pub use error::{Error, Result};
```

- [ ] **Step 2: Verify build, lint and tests**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: all clean, 37 core tests pass.

- [ ] **Step 3: Run the read-only bring-up on the owner's keyboard**

Run: `cargo run -q -p flow2-cli -- probe`

Known result on 2026-09-29 (Windows 11, 84-key Flow 2 Mac, USB): the first two lines print (`device: Flow2@Lofree (388d:0028)`, `model: flow2-mac-84`), then `error: no response from the keyboard`. A standalone test showed the write to interface 2 succeeds (33 bytes) and nothing is read back within 3 seconds, with 33-byte and 65-byte writes alike.

- [ ] **Step 4: If probe still times out, work down this ladder and stop at the first step that gets a reply**

1. Open `https://usevia.app` in Chrome, load `layouts/flow2-mac-84.json` under Settings and Design, authorize the keyboard. If VIA connects, the firmware does answer over WebHID, so the fault is in our transport (compare what Chrome sends with a USB capture if needed).
2. Try `flow2ctl probe` again with the keyboard freshly replugged and awake (press a key first), and with VIA closed.
3. Try the keyboard's other switch position and any documented Fn combination that selects wired mode, then replug.
4. Run the reference project's `flow2ctl probe` from `linder3hs/lofree-flow-2` on a Mac with the same keyboard to see whether the firmware replies there.
5. If nothing answers anywhere, stop and report: the VIA path is not reachable on this unit and the design needs revisiting.

- [ ] **Step 5: Record what you found in `docs/protocol.md`**

Create the file with this content, then fill in the last section with the real results of Step 3 and Step 4 and the output of `flow2ctl scan`:

```markdown
# Protocol notes

## Confirmed from source (reference project linder3hs/lofree-flow-2, read 2026-09-29)

- Raw HID usage page `0xFF60`, usage `0x61`, 32-byte reports.
- Command `0x01` returns the protocol version in bytes 1 and 2, big endian. The reference project saw 12 on a Flow 2 84.
- Backlight: custom channel 1. Value 1 is brightness 0 to 255, value 2 is effect (0 steady, 1 breathing). Set is `0x07`, get is `0x08`, save is `0x09`.
- Over Bluetooth the VIA interface is not exposed. With the switch in Bluetooth mode a USB cable only charges.
- The firmware turns the backlight off after idle in Bluetooth mode and that timeout is not exposed through VIA.

## Confirmed on hardware, 2026-09-29 (84-key Flow 2 Mac, USB, Windows 11)

- Enumerates as `388d:0028`, product `Flow2@Lofree`, release `0x2004`.
- Interface 2 is `0xFF60` / `0x61`. Interfaces 0 and 1 are the keyboard and consumer collections.
- Writes to interface 2 are accepted. Reply to `0x01`: none in 3 s (see Task 12 of the core plan).

## Assumed from the public VIA protocol, not yet confirmed on this keyboard

- [ ] `0x04` get keycode returns the code in bytes 4 and 5, `0x05` sets it.
- [ ] `0x11` returns the layer count in byte 1.
- [ ] Layer keycode ranges `TO` `0x5200`, `MO` `0x5220`, `TG` `0x5260`.
- [ ] Wireless keys `0x7793` to `0x7795` and `0x7785` survive a write and read back.

## Known limits

- With two Lofree devices attached, `find()` takes the first VIA interface.
- Windows and Linux battery is not implemented.

## Findings from bring-up

Record here: the results of Task 12 Steps 3 and 4, and the output of `flow2ctl scan`.
```

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml crates docs
git commit -m "feat(cli): flow2ctl and hardware bring-up notes"
```

---

## Out of this plan

- Tauri app, screens, animations, tray: next plan, written after Task 12 shows the VIA path is reachable.
- CI and release workflows: third plan.
- Windows and Linux battery, Windows and Linux hardware runs, the 68 and 100 models, macros: after the first two plans.

## Self-review notes

- Spec coverage: architecture (Tasks 1 to 11), profiles and factory backup (9), keymap and verification (4), backlight (5), battery macOS (10), model registry and layout (6, 7), CLI (12), risks and hardware finding (12), testing without hardware via `MockDevice` (3). UI, tray, CI, Windows and Linux battery are deliberately in later plans.
- Type consistency: `Keymap`, `Backlight`, `Profile`, `ModelDef`, `ViaClient` signatures match between the tasks that define them and the tasks that use them, since all blocks are taken from one compiling crate.
