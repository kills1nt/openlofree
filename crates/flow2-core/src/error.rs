#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no Flow 2 found. Connect it with USB-C: Bluetooth cannot be configured")]
    NotFound,
    #[error("device I/O failed: {0}")]
    Io(String),
    #[error("no response from the keyboard")]
    Timeout,
    #[error("unexpected reply: {0}")]
    BadReply(String),
    #[error("write not confirmed at layer {layer} row {row} col {col}: wrote {wrote:#06x}, read back {read:#06x}")]
    VerifyFailed {
        layer: u8,
        row: u8,
        col: u8,
        wrote: u16,
        read: u16,
    },
    #[error("invalid layout: {0}")]
    Layout(String),
    #[error("invalid profile: {0}")]
    Profile(String),
}

pub type Result<T> = std::result::Result<T, Error>;
