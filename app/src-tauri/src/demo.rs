//! Demo mode: an in-memory keyboard seeded like the owner's real Flow 2 Mac 84, so the app runs
//! without hardware and can never write to a real keyboard.
use flow2_core::layout::ModelDef;
use flow2_core::transport::mock::MockDevice;

const TRNS: u16 = 0x0001;
const LAYERS: u8 = 6;

/// (row, col, keycode) of layer 0, as read from a real 84-key Flow 2 Mac.
const LAYER0: &[(u8, u8, u16)] = &[
    (0, 0, 0x29),
    (0, 1, 0x00BE),
    (0, 2, 0x00BD),
    (0, 3, 0x7E0B),
    (0, 4, 0x7E0F),
    (0, 5, 0x7803),
    (0, 6, 0x7804),
    (0, 7, 0x00AC),
    (0, 8, 0x00AE),
    (0, 9, 0x00AB),
    (0, 10, 0x00A8),
    (0, 11, 0x00AA),
    (0, 12, 0x00A9),
    (0, 13, 0x7E0D),
    (0, 14, 0x4C),
    (1, 0, 0x35),
    (1, 1, 0x1E),
    (1, 2, 0x1F),
    (1, 3, 0x20),
    (1, 4, 0x21),
    (1, 5, 0x22),
    (1, 6, 0x23),
    (1, 7, 0x24),
    (1, 8, 0x25),
    (1, 9, 0x26),
    (1, 10, 0x27),
    (1, 11, 0x2D),
    (1, 12, 0x2E),
    (1, 13, 0x2A),
    (1, 14, 0x4A),
    (2, 0, 0x2B),
    (2, 1, 0x14),
    (2, 2, 0x1A),
    (2, 3, 0x08),
    (2, 4, 0x15),
    (2, 5, 0x17),
    (2, 6, 0x1C),
    (2, 7, 0x18),
    (2, 8, 0x0C),
    (2, 9, 0x12),
    (2, 10, 0x13),
    (2, 11, 0x2F),
    (2, 12, 0x30),
    (2, 13, 0x31),
    (2, 14, 0x4D),
    (3, 0, 0x39),
    (3, 1, 0x04),
    (3, 2, 0x16),
    (3, 3, 0x07),
    (3, 4, 0x09),
    (3, 5, 0x0A),
    (3, 6, 0x0B),
    (3, 7, 0x0D),
    (3, 8, 0x0E),
    (3, 9, 0x0F),
    (3, 10, 0x33),
    (3, 11, 0x34),
    (3, 13, 0x28),
    (3, 14, 0x4B),
    (4, 0, 0xE1),
    (4, 2, 0x1D),
    (4, 3, 0x1B),
    (4, 4, 0x06),
    (4, 5, 0x19),
    (4, 6, 0x05),
    (4, 7, 0x11),
    (4, 8, 0x10),
    (4, 9, 0x36),
    (4, 10, 0x37),
    (4, 11, 0x38),
    (4, 12, 0xE5),
    (4, 13, 0x52),
    (4, 14, 0x4E),
    (5, 0, 0x5221),
    (5, 1, 0xE0),
    (5, 2, 0xE2),
    (5, 3, 0xE3),
    (5, 6, 0x2C),
    (5, 9, 0xE7),
    (5, 10, 0xE6),
    (5, 11, 0xE4),
    (5, 12, 0x50),
    (5, 13, 0x51),
    (5, 14, 0x4F),
];

/// Layer 1 in the demo: F1 to F12 on the number row and the wireless keys on Q W E R.
const LAYER1: &[(u8, u8, u16)] = &[
    (1, 1, 0x3A),
    (1, 2, 0x3B),
    (1, 3, 0x3C),
    (1, 4, 0x3D),
    (1, 5, 0x3E),
    (1, 6, 0x3F),
    (1, 7, 0x40),
    (1, 8, 0x41),
    (1, 9, 0x42),
    (1, 10, 0x43),
    (1, 11, 0x44),
    (1, 12, 0x45),
    (2, 1, 0x7793),
    (2, 2, 0x7794),
    (2, 3, 0x7795),
    (2, 4, 0x7785),
];

pub fn demo_device(def: &ModelDef) -> MockDevice {
    let mut dev = MockDevice::new(def.rows, def.cols, LAYERS);
    let cols = def.cols as usize;
    for layer in dev.keymap.iter_mut() {
        layer.fill(TRNS);
    }
    for &(r, c, code) in LAYER0 {
        dev.keymap[0][r as usize * cols + c as usize] = code;
    }
    for &(r, c, code) in LAYER1 {
        dev.keymap[1][r as usize * cols + c as usize] = code;
    }
    dev
}

/// A profile name that does not collide with `existing` (case-insensitive), for imports.
pub fn unique_name(existing: &[String], name: &str) -> String {
    let taken = |n: &str| existing.iter().any(|e| e.eq_ignore_ascii_case(n));
    if !taken(name) {
        return name.to_string();
    }
    let first = format!("{name} (imported)");
    if !taken(&first) {
        return first;
    }
    (2..)
        .map(|i| format!("{name} (imported {i})"))
        .find(|n| !taken(n))
        .unwrap_or(first)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flow2_core::models::MODELS;

    #[test]
    fn demo_keyboard_matches_the_84_layout() {
        let def = MODELS[0].definition().unwrap();
        let dev = demo_device(&def);
        assert_eq!(dev.keymap.len(), 6);
        // Every key the layout has carries a real code on layer 0, no key is left unset.
        for k in &def.keys {
            let code = dev.keymap[0][k.row as usize * def.cols as usize + k.col as usize];
            assert_ne!(code, TRNS, "key {},{}", k.row, k.col);
        }
        assert_eq!(LAYER0.len(), def.keys.len());
        assert_eq!(dev.keymap[0][5 * 15 + 6], 0x2C); // Space
    }

    #[test]
    fn unique_name_avoids_collisions() {
        let existing = vec!["Work".to_string(), "work (imported)".to_string()];
        assert_eq!(unique_name(&existing, "Gaming"), "Gaming");
        assert_eq!(unique_name(&existing, "WORK"), "WORK (imported 2)");
        assert_eq!(unique_name(&[], "Work"), "Work");
    }
}
