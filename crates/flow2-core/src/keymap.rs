use crate::client::ViaClient;
use crate::error::{Error, Result};
use crate::transport::Transport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keymap {
    pub rows: u8,
    pub cols: u8,
    /// layers[l][row * cols + col]
    pub layers: Vec<Vec<u16>>,
}

pub fn read<T: Transport>(client: &mut ViaClient<T>, rows: u8, cols: u8) -> Result<Keymap> {
    let count = client.layer_count()?;
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

        // 1 layer-count + 12 reads + first write/readback pair, then the cable "comes out".
        let mut dying = ViaClient::new(DiesAfter(MockDevice::new(2, 3, 2), 15));
        assert!(matches!(apply(&mut dying, &target), Err(Error::Io(_))));
        let half_written = dying.into_transport().0;
        assert_eq!(half_written.keymap[0][0], 0x0004);
        assert_eq!(half_written.keymap[1][5], 0);

        // Plugged back in: applying again finishes the job and only writes what is missing.
        let mut c = ViaClient::new(half_written);
        assert_eq!(apply(&mut c, &target).unwrap(), 1);
        assert_eq!(read(&mut c, 2, 3).unwrap(), target);
    }
}
