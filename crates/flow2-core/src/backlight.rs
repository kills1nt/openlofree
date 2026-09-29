use serde::{Deserialize, Serialize};

use crate::client::ViaClient;
use crate::error::Result;
use crate::transport::Transport;
use crate::via::{VALUE_BRIGHTNESS, VALUE_EFFECT};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Off,
    Steady,
    Breathing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Backlight {
    pub mode: Mode,
    /// 0..=255, the raw VIA value. Ignored when `mode` is `Off`.
    pub brightness: u8,
}

pub fn level_from_percent(percent: u8) -> u8 {
    (percent.min(100) as u16 * 255 / 100) as u8
}

pub fn read<T: Transport>(client: &mut ViaClient<T>) -> Result<Backlight> {
    let brightness = client.backlight_get(VALUE_BRIGHTNESS)?;
    let effect = client.backlight_get(VALUE_EFFECT)?;
    let mode = match (brightness, effect) {
        (0, _) => Mode::Off,
        (_, 0) => Mode::Steady,
        _ => Mode::Breathing,
    };
    Ok(Backlight { mode, brightness })
}

/// Writes the settings and saves them to the keyboard so they survive a power cycle.
pub fn apply<T: Transport>(client: &mut ViaClient<T>, b: Backlight) -> Result<()> {
    match b.mode {
        Mode::Off => client.backlight_set(VALUE_BRIGHTNESS, 0)?,
        Mode::Steady | Mode::Breathing => {
            client.backlight_set(VALUE_EFFECT, (b.mode == Mode::Breathing) as u8)?;
            client.backlight_set(VALUE_BRIGHTNESS, b.brightness)?;
        }
    }
    client.backlight_save()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::MockDevice;

    #[test]
    fn percent_conversion_clamps() {
        assert_eq!(level_from_percent(0), 0);
        assert_eq!(level_from_percent(50), 127);
        assert_eq!(level_from_percent(100), 255);
        assert_eq!(level_from_percent(200), 255);
    }

    #[test]
    fn apply_then_read_round_trips() {
        let mut c = ViaClient::new(MockDevice::new(1, 1, 1));
        for b in [
            Backlight {
                mode: Mode::Breathing,
                brightness: 90,
            },
            Backlight {
                mode: Mode::Steady,
                brightness: 200,
            },
            Backlight {
                mode: Mode::Off,
                brightness: 0,
            },
        ] {
            apply(&mut c, b).unwrap();
            assert_eq!(read(&mut c).unwrap(), b);
        }
    }

    #[test]
    fn apply_saves_to_device() {
        let mut c = ViaClient::new(MockDevice::new(1, 1, 1));
        apply(
            &mut c,
            Backlight {
                mode: Mode::Steady,
                brightness: 10,
            },
        )
        .unwrap();
        assert!(c.into_transport().saved);
    }
}
