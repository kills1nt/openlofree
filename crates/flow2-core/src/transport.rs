use crate::error::Result;
use crate::via::Report;

/// One request, one reply. VIA answers every command, so a single call is enough.
pub trait Transport {
    /// Drops any input the device queued earlier, such as a reply that arrived after its request timed out.
    /// Without this a late reply is read as the answer to the next request.
    fn flush(&mut self) {}

    fn exchange(&mut self, out: &Report) -> Result<Report>;
}

impl<T: Transport + ?Sized> Transport for Box<T> {
    fn flush(&mut self) {
        (**self).flush()
    }

    fn exchange(&mut self, out: &Report) -> Result<Report> {
        (**self).exchange(out)
    }
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

#[cfg(test)]
mod tests {
    use super::mock::MockDevice;
    use super::Transport;
    use crate::client::ViaClient;

    #[test]
    fn boxed_dyn_transport_works_in_a_client() {
        let boxed: Box<dyn Transport + Send> = Box::new(MockDevice::new(2, 3, 2));
        let mut c = ViaClient::new(boxed);
        assert_eq!(c.layer_count().unwrap(), 2);
    }
}
