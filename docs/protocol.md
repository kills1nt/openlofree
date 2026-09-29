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
- The first bring-up attempts got no reply to `0x01` (3 s, several tries). Cause: the keyboard was asleep. After waking it, `flow2ctl probe` answers: VIA protocol 12, 6 layers.
- `keys --layer 0` reads 84 keys and the legends match the physical Mac layout (Esc top left, Space at 5,6, Opt and Cmd, `MO(1)` at 5,0). Layer 1 has 26 keys set.
- Some top-row keys read as raw codes without a legend (`0x00A8` to `0x00BE`, `0x7803`, `0x7804`, `0x7E0B`, `0x7E0D`, `0x7E0F`). They are media and vendor keys, legends to add later.

## Assumed from the public VIA protocol, not yet confirmed on this keyboard

- [x] `0x04` get keycode returns the code in bytes 4 and 5 (read confirmed on hardware). `0x05` set not yet run on hardware.
- [x] `0x11` returns the layer count in byte 1 (6 on this unit).
- [x] `0x12` get keymap buffer works: 28 bytes per request, layer then row then column, two bytes per key, big endian. A full read of 6 layers takes about 1.3 s instead of over 8 s with one request per key. The reader cross-checks the buffer against a direct read and falls back to per-key reads if they disagree.
- [ ] Layer keycode ranges `TO` `0x5200`, `MO` `0x5220`, `TG` `0x5260`.
- [ ] Wireless keys `0x7793` to `0x7795` and `0x7785` survive a write and read back.

## Known limits

- With two Lofree devices attached, `find()` takes the first VIA interface.
- Windows and Linux battery is not implemented.

## Findings from bring-up

- 2026-09-29: probe timed out while the keyboard was asleep, then worked after waking it (see above). `flow2ctl scan` and the write checks in docs/hardware-checklist.md are still to run.
