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

- 2026-09-29, core plan Task 12, Windows 11, 84-key Flow 2 Mac, USB: `flow2ctl probe` finds the device and model, then times out (write ok, no reply within 1 s, two tries). A standalone test with 33-byte and 65-byte writes and a 3 s read also got no reply.
- No other program holding the keyboard was found (no Lofree, VIA, Vial or vendor tool processes running).
- Not yet tried, needs the owner: step 1 of the ladder (usevia.app in Chrome with layouts/flow2-mac-84.json, needs a WebHID permission click), the switch position and Fn combinations, and the reference project on a Mac.
- `flow2ctl scan` was not run because it needs a keyboard that answers.
