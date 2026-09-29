# openlofree Desktop App Plan

> Executed inline in one session, at the owner's request to build the whole app now and test the UI by hand. Rulings are recorded in the final message.

**Goal:** A Tauri 2 desktop app for Windows, Linux and macOS that configures the Flow 2 Mac 84: visual keymap editing per layer, profiles, backlight, device status, tray. Built on `flow2-core`.

**Architecture:** `app/src-tauri` is a thin Tauri layer (state holding one `ViaClient<Box<dyn Transport + Send>>`, async commands, tray). `app/src` is Svelte 5 + TypeScript. The frontend talks to Rust only through `src/lib/api.ts`, which uses Tauri `invoke` when present and an in-memory mock otherwise, so the UI can be developed and checked in a plain browser. A Demo mode uses `MockDevice` so the app runs without a keyboard and never writes to a real one.

**Tech Stack:** Tauri 2, Svelte 5, Vite, TypeScript, vitest, `@fontsource-variable/onest`, `tauri-plugin-dialog`.

**Spec:** `docs/superpowers/specs/2026-09-29-openlofree-design.md`. **Direction:** `DESIGN.md`.

## Global Constraints

- Every rule in the core plan's Global Constraints still holds: keymap and backlight only, factory backup before the first write of a session, every key write read back, no firmware or DFU.
- Config over USB only, battery elsewhere. UI language English, all strings in `src/lib/strings.ts`.
- antislop applies during the work: DESIGN.md direction, dials 2/2/2, Delivery Gate before calling the UI done. No em dash in any text.
- Only `transform` and `opacity` animate. `prefers-reduced-motion` disables motion.
- Both themes must work. Text meets WCAG AA. Keyboard-only use works.
- Build from `E:\Projects\openlofree` (short path).

## Tasks

1. **Core additions (TDD).** `impl Transport for Box<T>`; `Serialize` on `KeyDef` and `ModelDef`; `backlight::preview` (writes without saving, so a slider does not wear the EEPROM); keycode labels for media (`0x00A8` to `0x00BE`, confirmed against the owner's top row) and custom keyboard codes (`0x7E00` to `0x7E3F`); `keycodes::catalog(variant, layers)` grouped for a picker and `legends(codes, variant)`; `profile::ProfileStore` (list, save, load, delete, duplicate in a directory, safe file names). Tests first for each.
2. **Scaffold.** `app/` with Vite, Svelte, TS, vitest, Tauri config, capabilities, generated placeholder icon, workspace member `app/src-tauri`. Acceptance: `cargo build -p openlofree-app` and `npm run build` succeed.
3. **Rust app layer.** State, async commands (`connect`, `disconnect`, `read_state`, `set_key`, `apply_keymap`, `preview_backlight`, `apply_backlight`, profile commands, `battery`, `catalog`, `legends`, `factory_backup_info`), factory backup guard, demo mode, tray with battery line and profile items. Pure helpers unit-tested.
4. **Frontend foundations.** Tokens and themes, strings, typed API with mock, theme store, toast/live region, dialog helper.
5. **Keyboard component.** Geometry to pixels, keycaps with depth, roving arrow-key focus, press animation from window key events, layer relabel animation. Unit tests for geometry, event-code map, draft diff.
6. **Keys screen.** Layer tabs, picker with groups and search and hex entry, pending changes bar with Apply and Discard, verified write result, error handling.
7. **Lighting screen.** Mode segmented control, brightness slider with live preview on the keyboard and on the device, Apply saves.
8. **Profiles screen.** List, save current, apply, duplicate, delete with confirm, export, import.
9. **Device screen and shell.** Connection state, battery, model, protocol, layers, factory backup path, empty/loading/error states, Try demo, theme toggle, tab navigation.
10. **Verification.** vitest, svelte-check, vite build, browser run with the mock (console clean, both themes, narrow width, keyboard-only, reduced motion), `cargo clippy` and `cargo test`, `tauri build --no-bundle`, smoke launch, Delivery Gate report, README update.
