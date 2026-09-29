//! Real transport over hidapi. Not unit-tested: it needs hardware, see docs/hardware-checklist.md.
use hidapi::{HidApi, HidDevice};

use crate::error::{Error, Result};
use crate::transport::Transport;
use crate::via::{Report, REPORT_SIZE};

pub const VENDOR_ID: u16 = 0x388D;
const USAGE_PAGE: u16 = 0xFF60;
const USAGE: u16 = 0x61;
const TIMEOUT_MS: i32 = 1000;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub product: String,
}

/// Field order matters: `dev` must drop before `_api`.
pub struct HidTransport {
    dev: HidDevice,
    _api: HidApi,
}

fn io(e: impl std::fmt::Display) -> Error {
    Error::Io(e.to_string())
}

/// Opens the first VIA raw HID interface of a Lofree keyboard (VID 0x388D).
pub fn find() -> Result<(HidTransport, DeviceInfo)> {
    let api = HidApi::new().map_err(io)?;
    let info = api
        .device_list()
        .find(|d| d.vendor_id() == VENDOR_ID && d.usage_page() == USAGE_PAGE && d.usage() == USAGE)
        .ok_or(Error::NotFound)?;
    let dev = info.open_device(&api).map_err(io)?;
    let device = DeviceInfo {
        vendor_id: info.vendor_id(),
        product_id: info.product_id(),
        product: info.product_string().unwrap_or_default().to_string(),
    };
    Ok((HidTransport { dev, _api: api }, device))
}

impl Transport for HidTransport {
    fn exchange(&mut self, out: &Report) -> Result<Report> {
        let mut buf = [0u8; REPORT_SIZE + 1]; // byte 0 is the report id, VIA uses none
        buf[1..].copy_from_slice(out);
        self.dev.write(&buf).map_err(io)?;
        let mut reply = [0u8; REPORT_SIZE];
        match self.dev.read_timeout(&mut reply, TIMEOUT_MS).map_err(io)? {
            0 => Err(Error::Timeout),
            _ => Ok(reply),
        }
    }
}
