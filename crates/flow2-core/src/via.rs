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
pub const CMD_KEYMAP_GET_BUFFER: u8 = 0x12;

/// Most bytes one buffer reply carries (32-byte report minus 4 header bytes).
pub const BUFFER_CHUNK: usize = 28;

/// Backlight channel and value ids as used by the reference project linder3hs/lofree-flow-2.
pub const CHANNEL_BACKLIGHT: u8 = 1;
pub const VALUE_BRIGHTNESS: u8 = 1;
pub const VALUE_EFFECT: u8 = 2;

fn pack(payload: &[u8]) -> Report {
    let mut r = [0u8; REPORT_SIZE];
    r[..payload.len()].copy_from_slice(payload);
    r
}

fn expect_cmd(r: &Report, cmd: u8) -> Result<()> {
    if r[0] == cmd {
        Ok(())
    } else {
        Err(Error::BadReply(format!(
            "expected command {cmd:#04x}, got {:#04x}",
            r[0]
        )))
    }
}

/// A reply to a query must echo the request bytes it answers. This catches a late reply from an earlier
/// request being read as the answer to the current one.
fn expect_echo(req: &Report, r: &Report, n: usize) -> Result<()> {
    if r[..n] == req[..n] {
        Ok(())
    } else {
        Err(Error::BadReply(format!(
            "reply {:02x?} does not answer request {:02x?}",
            &r[..n],
            &req[..n]
        )))
    }
}

/// A reply to a write only needs to echo the command byte (0xFF means the firmware did not handle it).
pub fn expect_ack(req: &Report, r: &Report) -> Result<()> {
    expect_echo(req, r, 1)
}

pub fn protocol_version() -> Report {
    pack(&[CMD_GET_PROTOCOL_VERSION])
}

pub fn parse_protocol_version(r: &Report) -> Result<u16> {
    expect_cmd(r, CMD_GET_PROTOCOL_VERSION)?;
    Ok(u16::from_be_bytes([r[1], r[2]]))
}

pub fn layer_count() -> Report {
    pack(&[CMD_KEYMAP_LAYER_COUNT])
}

pub fn parse_layer_count(r: &Report) -> Result<u8> {
    expect_cmd(r, CMD_KEYMAP_LAYER_COUNT)?;
    Ok(r[1])
}

pub fn get_keycode(layer: u8, row: u8, col: u8) -> Report {
    pack(&[CMD_KEYMAP_GET_KEYCODE, layer, row, col])
}

pub fn parse_keycode(req: &Report, r: &Report) -> Result<u16> {
    expect_echo(req, r, 4)?;
    Ok(u16::from_be_bytes([r[4], r[5]]))
}

pub fn set_keycode(layer: u8, row: u8, col: u8, code: u16) -> Report {
    let [hi, lo] = code.to_be_bytes();
    pack(&[CMD_KEYMAP_SET_KEYCODE, layer, row, col, hi, lo])
}

/// Reads `size` bytes of the keymap buffer from `offset`. Layout: layer, then row, then column, two
/// bytes per key, big endian.
pub fn get_buffer(offset: u16, size: u8) -> Report {
    let [hi, lo] = offset.to_be_bytes();
    pack(&[CMD_KEYMAP_GET_BUFFER, hi, lo, size])
}

pub fn parse_buffer<'a>(req: &Report, r: &'a Report) -> Result<&'a [u8]> {
    expect_echo(req, r, 4)?;
    Ok(&r[4..4 + req[3] as usize])
}

pub fn backlight_get(value: u8) -> Report {
    pack(&[CMD_CUSTOM_GET_VALUE, CHANNEL_BACKLIGHT, value])
}

pub fn parse_backlight_value(req: &Report, r: &Report) -> Result<u8> {
    expect_echo(req, r, 3)?;
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

        let req = get_keycode(0, 0, 0);
        let mut r = req;
        r[4] = 0x00;
        r[5] = 0x29;
        assert_eq!(parse_keycode(&req, &r).unwrap(), 0x0029);

        let req = backlight_get(VALUE_BRIGHTNESS);
        let mut r = req;
        r[3] = 77;
        assert_eq!(parse_backlight_value(&req, &r).unwrap(), 77);
    }

    #[test]
    fn rejects_reply_that_echoes_a_different_request() {
        // A late reply for key (0,0,0) must not be accepted as the answer for key (0,0,1).
        let mut stale = get_keycode(0, 0, 0);
        stale[5] = 0x29;
        assert!(matches!(
            parse_keycode(&get_keycode(0, 0, 1), &stale),
            Err(Error::BadReply(_))
        ));
        // Same for backlight: an effect reply is not a brightness reply.
        assert!(matches!(
            parse_backlight_value(
                &backlight_get(VALUE_BRIGHTNESS),
                &backlight_get(VALUE_EFFECT)
            ),
            Err(Error::BadReply(_))
        ));
    }

    #[test]
    fn write_ack_must_echo_the_command() {
        let req = backlight_set(VALUE_BRIGHTNESS, 10);
        assert!(expect_ack(&req, &req).is_ok());
        let mut unhandled = req;
        unhandled[0] = 0xFF;
        assert!(matches!(
            expect_ack(&req, &unhandled),
            Err(Error::BadReply(_))
        ));
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

    #[test]
    fn buffer_request_layout_and_reply() {
        let req = get_buffer(0x0102, 28);
        assert_eq!(&req[..4], &[0x12, 0x01, 0x02, 28]);
        let mut r = req;
        r[4..8].copy_from_slice(&[9, 8, 7, 6]);
        assert_eq!(&parse_buffer(&req, &r).unwrap()[..4], &[9, 8, 7, 6]);
        let mut other = r;
        other[2] = 0x03; // a reply for a different offset
        assert!(matches!(
            parse_buffer(&req, &other),
            Err(Error::BadReply(_))
        ));
    }
}
