use crate::client::ViaClient;
use crate::error::{Error, Result};
use crate::transport::Transport;
use crate::via::BUFFER_CHUNK;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keymap {
    pub rows: u8,
    pub cols: u8,
    /// layers[l][row * cols + col]
    pub layers: Vec<Vec<u16>>,
}

/// Reads every layer. Uses the block read (about 40 requests for a full keyboard) and falls back to one
/// request per key when the firmware does not handle it or its buffer disagrees with a direct read.
pub fn read<T: Transport>(client: &mut ViaClient<T>, rows: u8, cols: u8) -> Result<Keymap> {
    let count = client.layer_count()?;
    match read_buffered(client, rows, cols, count) {
        Err(Error::BadReply(_)) => read_per_key(client, rows, cols, count),
        other => other,
    }
}

fn read_buffered<T: Transport>(
    client: &mut ViaClient<T>,
    rows: u8,
    cols: u8,
    count: u8,
) -> Result<Keymap> {
    let per_layer = rows as usize * cols as usize;
    let total = count as usize * per_layer * 2;
    let mut bytes = Vec::with_capacity(total);
    while bytes.len() < total {
        let size = (total - bytes.len()).min(BUFFER_CHUNK);
        bytes.extend(client.get_buffer(bytes.len() as u16, size as u8)?);
    }
    let (pairs, _) = bytes.as_chunks::<2>();
    let codes: Vec<u16> = pairs.iter().map(|b| u16::from_be_bytes(*b)).collect();
    // Trust the buffer only if it agrees with an ordinary single-key read.
    if let Some(&first) = codes.first() {
        if client.get_keycode(0, 0, 0)? != first {
            return Err(Error::BadReply(
                "keymap buffer disagrees with a direct read".into(),
            ));
        }
    }
    let layers = codes
        .chunks(per_layer.max(1))
        .map(<[u16]>::to_vec)
        .collect();
    Ok(Keymap { rows, cols, layers })
}

fn read_per_key<T: Transport>(
    client: &mut ViaClient<T>,
    rows: u8,
    cols: u8,
    count: u8,
) -> Result<Keymap> {
    let mut layers = Vec::with_capacity(count as usize);
    for layer in 0..count {
        let mut keys = Vec::with_capacity(rows as usize * cols as usize);
        for row in 0..rows {
            for col in 0..cols {
                keys.push(client.get_keycode(layer, row, col)?);
            }
        }
        layers.push(keys);
    }
    Ok(Keymap { rows, cols, layers })
}

/// Refuses a position outside the matrix or the layer count. Some firmware builds compute an EEPROM
/// address from these values without a bounds check, so out-of-range writes must never be sent.
pub fn check_position(rows: u8, cols: u8, layers: u8, layer: u8, row: u8, col: u8) -> Result<()> {
    if layer < layers && row < rows && col < cols {
        Ok(())
    } else {
        Err(Error::Profile(format!(
            "layer {layer} row {row} col {col} is outside {layers} layers of {rows}x{cols}"
        )))
    }
}

/// Writes one key, then reads it back. A mismatch is an error, never silent.
pub fn set_key_verified<T: Transport>(
    client: &mut ViaClient<T>,
    layer: u8,
    row: u8,
    col: u8,
    code: u16,
) -> Result<()> {
    client.set_keycode(layer, row, col, code)?;
    let read = client.get_keycode(layer, row, col)?;
    if read == code {
        Ok(())
    } else {
        Err(Error::VerifyFailed {
            layer,
            row,
            col,
            wrote: code,
            read,
        })
    }
}

/// Writes only the keys that differ from the device. Returns how many were written.
pub fn apply<T: Transport>(client: &mut ViaClient<T>, target: &Keymap) -> Result<usize> {
    let want = target.rows as usize * target.cols as usize;
    if target.layers.iter().any(|l| l.len() != want) {
        return Err(Error::Profile(format!("every layer must have {want} keys")));
    }
    let current = read(client, target.rows, target.cols)?;
    if current.layers.len() != target.layers.len() {
        return Err(Error::Profile(format!(
            "profile has {} layers, keyboard has {}",
            target.layers.len(),
            current.layers.len()
        )));
    }
    let mut written = 0;
    for (l, (have, want)) in current.layers.iter().zip(&target.layers).enumerate() {
        for (i, (&h, &w)) in have.iter().zip(want).enumerate() {
            if h != w {
                let row = (i / target.cols as usize) as u8;
                let col = (i % target.cols as usize) as u8;
                set_key_verified(client, l as u8, row, col, w)?;
                written += 1;
            }
        }
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::MockDevice;

    fn client() -> ViaClient<MockDevice> {
        ViaClient::new(MockDevice::new(2, 3, 2))
    }

    #[test]
    fn reads_all_layers() {
        let mut dev = MockDevice::new(2, 3, 2);
        dev.keymap[1][4] = 0x0029;
        let km = read(&mut ViaClient::new(dev), 2, 3).unwrap();
        assert_eq!(km.layers.len(), 2);
        assert_eq!(km.layers[1][4], 0x0029);
    }

    #[test]
    fn verified_write_succeeds() {
        let mut c = client();
        set_key_verified(&mut c, 0, 1, 2, 0x0004).unwrap();
        assert_eq!(c.get_keycode(0, 1, 2).unwrap(), 0x0004);
    }

    #[test]
    fn verified_write_reports_unapplied_write() {
        let mut dev = MockDevice::new(2, 3, 2);
        dev.ignore_key_writes = true;
        let err = set_key_verified(&mut ViaClient::new(dev), 0, 1, 2, 0x0004).unwrap_err();
        assert!(matches!(
            err,
            Error::VerifyFailed {
                wrote: 0x0004,
                read: 0,
                ..
            }
        ));
    }

    #[test]
    fn apply_writes_only_differences() {
        let mut c = client();
        let mut target = read(&mut c, 2, 3).unwrap();
        target.layers[0][0] = 0x0004;
        target.layers[1][5] = 0x5221;
        assert_eq!(apply(&mut c, &target).unwrap(), 2);
        assert_eq!(read(&mut c, 2, 3).unwrap(), target);
        assert_eq!(apply(&mut c, &target).unwrap(), 0);
    }

    #[test]
    fn apply_rejects_layer_count_mismatch() {
        let mut c = client();
        let target = Keymap {
            rows: 2,
            cols: 3,
            layers: vec![vec![0; 6]],
        };
        assert!(matches!(apply(&mut c, &target), Err(Error::Profile(_))));
    }

    /// Unplug simulation: works for `budget` exchanges, then every call fails.
    struct DiesAfter(MockDevice, usize);
    impl Transport for DiesAfter {
        fn exchange(&mut self, out: &crate::via::Report) -> Result<crate::via::Report> {
            if self.1 == 0 {
                return Err(Error::Io("device gone".into()));
            }
            self.1 -= 1;
            self.0.exchange(out)
        }
    }

    #[test]
    fn apply_interrupted_by_unplug_errors_and_can_be_finished_later() {
        let mut target = read(&mut client(), 2, 3).unwrap();
        target.layers[0][0] = 0x0004;
        target.layers[1][5] = 0x0005;

        // 1 layer-count + 1 buffer chunk + 1 cross-check read + first write/readback pair, then the cable "comes out".
        let mut dying = ViaClient::new(DiesAfter(MockDevice::new(2, 3, 2), 5));
        assert!(matches!(apply(&mut dying, &target), Err(Error::Io(_))));
        let half_written = dying.into_transport().0;
        assert_eq!(half_written.keymap[0][0], 0x0004);
        assert_eq!(half_written.keymap[1][5], 0);

        // Plugged back in: applying again finishes the job and only writes what is missing.
        let mut c = ViaClient::new(half_written);
        assert_eq!(apply(&mut c, &target).unwrap(), 1);
        assert_eq!(read(&mut c, 2, 3).unwrap(), target);
    }

    #[test]
    fn check_position_rejects_out_of_range_targets() {
        assert!(check_position(6, 15, 4, 3, 5, 14).is_ok());
        assert!(matches!(
            check_position(6, 15, 4, 4, 0, 0),
            Err(Error::Profile(_))
        ));
        assert!(matches!(
            check_position(6, 15, 4, 0, 6, 0),
            Err(Error::Profile(_))
        ));
        assert!(matches!(
            check_position(6, 15, 4, 0, 0, 15),
            Err(Error::Profile(_))
        ));
    }

    #[test]
    fn apply_rejects_wrongly_shaped_layers_before_any_write() {
        let mut c = client();
        let target = Keymap {
            rows: 2,
            cols: 3,
            layers: vec![vec![0; 6], vec![9; 5]],
        };
        assert!(matches!(apply(&mut c, &target), Err(Error::Profile(_))));
        assert_eq!(read(&mut c, 2, 3).unwrap().layers[0][0], 0);
    }

    /// Counts exchanges so a test can prove the buffered read is not one request per key.
    struct Counting(MockDevice, usize);
    impl Transport for Counting {
        fn exchange(&mut self, out: &crate::via::Report) -> Result<crate::via::Report> {
            self.1 += 1;
            self.0.exchange(out)
        }
    }

    #[test]
    fn read_uses_few_exchanges_for_a_full_keyboard() {
        let mut dev = MockDevice::new(6, 15, 6);
        dev.keymap[5][89] = 0xABCD;
        dev.keymap[0][0] = 0x0029;
        let mut c = ViaClient::new(Counting(dev, 0));
        let km = read(&mut c, 6, 15).unwrap();
        assert_eq!(
            (km.layers.len(), km.layers[5][89], km.layers[0][0]),
            (6, 0xABCD, 0x0029)
        );
        let used = c.into_transport().1;
        assert!(
            used <= 60,
            "used {used} exchanges, per-key reading needs 541"
        );
    }

    /// Firmware that answers 0xFF (unhandled) to the buffer command.
    struct NoBuffer(MockDevice);
    impl Transport for NoBuffer {
        fn exchange(&mut self, out: &crate::via::Report) -> Result<crate::via::Report> {
            if out[0] == crate::via::CMD_KEYMAP_GET_BUFFER {
                let mut r = *out;
                r[0] = 0xFF;
                return Ok(r);
            }
            self.0.exchange(out)
        }
    }

    #[test]
    fn read_falls_back_to_per_key_when_the_buffer_command_is_unhandled() {
        let mut dev = MockDevice::new(2, 3, 2);
        dev.keymap[1][4] = 0x0029;
        let km = read(&mut ViaClient::new(NoBuffer(dev)), 2, 3).unwrap();
        assert_eq!(km.layers[1][4], 0x0029);
    }

    /// Buffer that disagrees with the per-key answer must not be trusted.
    struct LyingBuffer(MockDevice);
    impl Transport for LyingBuffer {
        fn exchange(&mut self, out: &crate::via::Report) -> Result<crate::via::Report> {
            let mut r = self.0.exchange(out)?;
            if out[0] == crate::via::CMD_KEYMAP_GET_BUFFER && out[1] == 0 && out[2] == 0 {
                r[4] ^= 0xFF;
            }
            Ok(r)
        }
    }

    #[test]
    fn read_distrusts_a_buffer_that_disagrees_with_a_direct_read() {
        let mut dev = MockDevice::new(2, 3, 2);
        dev.keymap[0][0] = 0x0029;
        let km = read(&mut ViaClient::new(LyingBuffer(dev)), 2, 3).unwrap();
        assert_eq!(km.layers[0][0], 0x0029);
    }
}
