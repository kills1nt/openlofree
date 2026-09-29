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
