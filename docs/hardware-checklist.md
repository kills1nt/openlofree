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
