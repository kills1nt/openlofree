use crate::error::{Error, Result};
use crate::transport::Transport;
use crate::via::{self, Report};

pub struct ViaClient<T: Transport> {
    transport: T,
}

impl<T: Transport> ViaClient<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn into_transport(self) -> T {
        self.transport
    }

    /// Sends a raw report, retrying once on I/O errors or timeouts.
    pub fn raw(&mut self, report: &Report) -> Result<Report> {
        match self.transport.exchange(report) {
            Err(Error::Io(_) | Error::Timeout) => self.transport.exchange(report),
            other => other,
        }
    }

    pub fn protocol_version(&mut self) -> Result<u16> {
        via::parse_protocol_version(&self.raw(&via::protocol_version())?)
    }

    pub fn layer_count(&mut self) -> Result<u8> {
        via::parse_layer_count(&self.raw(&via::layer_count())?)
    }

    pub fn get_keycode(&mut self, layer: u8, row: u8, col: u8) -> Result<u16> {
        via::parse_keycode(&self.raw(&via::get_keycode(layer, row, col))?)
    }

    pub fn set_keycode(&mut self, layer: u8, row: u8, col: u8, code: u16) -> Result<()> {
        self.raw(&via::set_keycode(layer, row, col, code))
            .map(|_| ())
    }

    pub fn backlight_get(&mut self, value: u8) -> Result<u8> {
        via::parse_backlight_value(&self.raw(&via::backlight_get(value))?)
    }

    pub fn backlight_set(&mut self, value: u8, data: u8) -> Result<()> {
        self.raw(&via::backlight_set(value, data)).map(|_| ())
    }

    pub fn backlight_save(&mut self) -> Result<()> {
        self.raw(&via::backlight_save()).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::MockDevice;

    #[test]
    fn talks_to_mock_device() {
        let mut c = ViaClient::new(MockDevice::new(6, 15, 4));
        assert_eq!(c.protocol_version().unwrap(), 12);
        assert_eq!(c.layer_count().unwrap(), 4);
        c.set_keycode(1, 2, 3, 0x0004).unwrap();
        assert_eq!(c.get_keycode(1, 2, 3).unwrap(), 0x0004);
        assert_eq!(c.get_keycode(0, 2, 3).unwrap(), 0);
    }

    struct FailsOnce(MockDevice, bool);
    impl Transport for FailsOnce {
        fn exchange(&mut self, out: &Report) -> Result<Report> {
            if !self.1 {
                self.1 = true;
                return Err(Error::Timeout);
            }
            self.0.exchange(out)
        }
    }

    #[test]
    fn retries_once_after_timeout() {
        let mut c = ViaClient::new(FailsOnce(MockDevice::new(6, 15, 4), false));
        assert_eq!(c.protocol_version().unwrap(), 12);
    }

    struct AlwaysFails;
    impl Transport for AlwaysFails {
        fn exchange(&mut self, _: &Report) -> Result<Report> {
            Err(Error::Timeout)
        }
    }

    #[test]
    fn gives_up_after_second_failure() {
        let mut c = ViaClient::new(AlwaysFails);
        assert!(matches!(c.protocol_version(), Err(Error::Timeout)));
    }
}
