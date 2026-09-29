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
        self.transport.flush();
        match self.transport.exchange(report) {
            Err(Error::Io(_) | Error::Timeout) => {
                self.transport.flush();
                self.transport.exchange(report)
            }
            other => other,
        }
    }

    /// Sends a write and checks that the firmware handled the command.
    fn write(&mut self, req: &Report) -> Result<()> {
        via::expect_ack(req, &self.raw(req)?)
    }

    fn query(&mut self, req: &Report) -> Result<Report> {
        self.raw(req)
    }

    pub fn protocol_version(&mut self) -> Result<u16> {
        via::parse_protocol_version(&self.raw(&via::protocol_version())?)
    }

    pub fn layer_count(&mut self) -> Result<u8> {
        via::parse_layer_count(&self.raw(&via::layer_count())?)
    }

    pub fn get_keycode(&mut self, layer: u8, row: u8, col: u8) -> Result<u16> {
        {
            let req = via::get_keycode(layer, row, col);
            via::parse_keycode(&req, &self.query(&req)?)
        }
    }

    pub fn set_keycode(&mut self, layer: u8, row: u8, col: u8, code: u16) -> Result<()> {
        self.write(&via::set_keycode(layer, row, col, code))
    }

    /// Reads up to 28 bytes of the keymap buffer.
    pub fn get_buffer(&mut self, offset: u16, size: u8) -> Result<Vec<u8>> {
        if size as usize > via::BUFFER_CHUNK {
            return Err(Error::BadReply(format!(
                "buffer reads carry at most {} bytes",
                via::BUFFER_CHUNK
            )));
        }
        let req = via::get_buffer(offset, size);
        Ok(via::parse_buffer(&req, &self.query(&req)?)?.to_vec())
    }

    pub fn backlight_get(&mut self, value: u8) -> Result<u8> {
        {
            let req = via::backlight_get(value);
            via::parse_backlight_value(&req, &self.query(&req)?)
        }
    }

    pub fn backlight_set(&mut self, value: u8, data: u8) -> Result<()> {
        self.write(&via::backlight_set(value, data))
    }

    pub fn backlight_save(&mut self) -> Result<()> {
        self.write(&via::backlight_save())
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

    /// Answers each request with the reply to the previous one, like a keyboard that answered late once.
    struct ShiftedByOne(MockDevice, Option<Report>);
    impl Transport for ShiftedByOne {
        fn exchange(&mut self, out: &Report) -> Result<Report> {
            let fresh = self.0.exchange(out)?;
            Ok(self.1.replace(fresh).unwrap_or(fresh))
        }
    }

    #[test]
    fn detects_reply_shifted_by_one_exchange() {
        let mut c = ViaClient::new(ShiftedByOne(MockDevice::new(2, 3, 1), None));
        c.get_keycode(0, 0, 0).unwrap();
        assert!(matches!(c.get_keycode(0, 0, 1), Err(Error::BadReply(_))));
    }

    struct Counting {
        flushes: usize,
        exchanges: usize,
    }
    impl Transport for Counting {
        fn flush(&mut self) {
            self.flushes += 1;
        }
        fn exchange(&mut self, out: &Report) -> Result<Report> {
            self.exchanges += 1;
            if self.exchanges == 1 {
                return Err(Error::Timeout);
            }
            Ok(*out)
        }
    }

    #[test]
    fn flushes_stale_input_before_every_attempt() {
        let mut c = ViaClient::new(Counting {
            flushes: 0,
            exchanges: 0,
        });
        c.protocol_version().unwrap();
        let t = c.into_transport();
        assert_eq!((t.exchanges, t.flushes), (2, 2));
    }

    struct Unhandled;
    impl Transport for Unhandled {
        fn exchange(&mut self, out: &Report) -> Result<Report> {
            let mut r = *out;
            r[0] = 0xFF;
            Ok(r)
        }
    }

    #[test]
    fn write_replies_are_checked() {
        let mut c = ViaClient::new(Unhandled);
        assert!(matches!(c.set_keycode(0, 0, 0, 4), Err(Error::BadReply(_))));
        assert!(matches!(c.backlight_set(1, 10), Err(Error::BadReply(_))));
        assert!(matches!(c.backlight_save(), Err(Error::BadReply(_))));
    }

    #[test]
    fn reads_the_keymap_buffer_in_chunks() {
        let mut dev = MockDevice::new(2, 3, 2);
        dev.keymap[1][4] = 0x1234;
        let mut c = ViaClient::new(dev);
        // layer 1, key 4 starts at byte (6 + 4) * 2 = 20
        assert_eq!(c.get_buffer(20, 2).unwrap(), vec![0x12, 0x34]);
        assert!(matches!(c.get_buffer(0, 29), Err(Error::BadReply(_))));
    }
}
