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
