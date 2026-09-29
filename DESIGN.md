# openlofree design direction

Read by antislop as the direction file. Applies to the desktop app in `app/`.

## Design Read

Reading this as: desktop configuration tool for keyboard owners, in a calm, tactile, warm-paper style, dial ENERGY 2 / RHYTHM 2 / MOTION 2.

## Identity

The keyboard is the product, so the keyboard is the hero of every screen that has one. Keys are objects with depth: a lit top face and a darker edge. Pressing lowers the cap, selecting raises it and lights it. That keycap is the identity motif, repeated in buttons and toggles so the whole app feels made of the same parts.

Not the owner's Hrateka identity. New palette and type for this project.

## Palette

Warm paper base, ink text, one amber accent. Two themes with a working toggle (default follows the system). Reason: a desk tool used day and night, not a developer terminal, so no fixed dark theme.

| token | light | dark | note |
|---|---|---|---|
| bg | #F3EEE4 | #15120E | page |
| surface | #FBF8F2 | #1E1A15 | panels |
| ink | #1F1B16 | #F3EEE4 | text, 14.8:1 and 16.2:1 on bg |
| muted | #5C5347 | #A89D8C | secondary text, 6.5:1 and 7.0:1 on bg |
| line | #D8CFBF | #38312A | decorative dividers only |
| control-edge | #857A67 | #857A69 | input and button borders, 3.7:1 and 4.4:1 on bg |
| key | #FFFDF8 | #2A241D | keycap top |
| key-edge | #CFC3AD | #0F0D0A | keycap side |
| accent | #E9A23B | #F2B24D | selected key, active layer, primary action |
| on-accent | #1F1B16 | #15120E | text on accent, 7.9:1 and 10.0:1 |
| focus | #1F1B16 | #F2B24D | focus ring, 14.8:1 and 10.0:1 on bg |
| danger | #B3391F | #FF8A6B | errors, 5.2:1 and 8.1:1 on bg |
| ok | #2A7048 | #7CC79A | confirmations, 5.2:1 and 9.3:1 on bg |

Accent is a fill, never a thin line on the paper (1.9:1 there). A selected key is accent fill plus a 2px ink border. Accent appears on at most: the selected key, the active layer tab, the primary button. That is the one deliberate accent.

## Type

Onest variable, bundled with the app (no network). Reason: friendly geometric sans with a real Cyrillic set for the later Ukrainian UI, clear at the small sizes of key legends. Not Inter, Geist or Space Grotesk. Legends and hex codes use `font-variant-numeric: tabular-nums`. No monospace headings, no wide-tracked uppercase labels.

## Shape and depth

- Keys 8px radius, panels 14px, buttons 10px, inputs 8px. Varied on purpose: the key is the friendliest shape, panels are calmer.
- Shadow means depth of a key and nothing else. Panels are flat with a 1px line.
- No glassmorphism, no gradient backgrounds, no grids or dot patterns, no capsule badges, no stat-card dashboards, no sparkle icons.

## Layout

No sidebar plus top bar shell. A slim top bar carries the wordmark as text, four tabs (Keys, Lighting, Profiles, Device) and a device chip. The stage below changes composition per screen: Keys is keyboard above a picker, Lighting is keyboard beside its controls, Profiles is a ruled list, Device is a definition list with the battery as one large number.

## Motion (every entry has a purpose)

- Key press: a key on the drawn keyboard lowers while the same physical key is held (window key events). Purpose: confirms which drawn key is which.
- Layer switch: legends slide 6px and cross-fade, staggered by column, 220ms. Purpose: shows what changed between layers.
- Selection: chosen key rises 2px and takes the accent fill, 120ms.
- Backlight preview: the drawn keys glow at the chosen brightness. Breathing loops only on the Lighting screen while breathing is selected. Purpose: previews the effect.
- Apply: button shows writing, then a check, then rests. Purpose: writing to the keyboard takes seconds.
- Only `transform` and `opacity`. `prefers-reduced-motion` turns all of it off, breathing becomes a static half-brightness.

## States

Every data view has empty, loading and error states that name the cause and the next action. Example: "No keyboard found. Connect it with USB-C, Bluetooth cannot be configured." with Try again and Try demo.

## Accessibility

Full Tab order, roving arrow-key movement inside the keyboard, visible focus ring using the focus token, dialogs are native `<dialog>` and close with Escape, live region announces apply results, contrast values above.

## Placeholders

The app icon is a generated placeholder keycap. Replace it when the owner supplies a logo.

## One-line reasons (R-31)

- Warm paper and ink: keys look like objects on a desk, and it separates the project from the owner's other work.
- One amber accent: the backlight is white, amber marks what you are editing and nothing else.
- Onest: Cyrillic ready, friendly, legible small.
- Keyboard as hero: the product is the keyboard, a dashboard would hide it.
- Varied radii: the key is the friendliest shape, panels stay calm.
- Motion tied to keyboard events: every animation answers a question the user has.
