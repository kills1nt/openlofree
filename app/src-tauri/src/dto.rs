//! Shapes that cross the Tauri boundary. Field names are camelCase for the TypeScript side.
use flow2_core::backlight::Backlight;
use flow2_core::layout::KeyDef;
use flow2_core::Error;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectInfo {
    pub model_id: String,
    pub label: String,
    pub verified: bool,
    pub demo: bool,
    pub mac: bool,
    pub product: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub protocol: u16,
    pub layers: u8,
    pub rows: u8,
    pub cols: u8,
    pub keys: Vec<KeyDef>,
}

#[derive(Debug, Serialize)]
pub struct StateDto {
    pub layers: Vec<Vec<u16>>,
    pub backlight: Backlight,
}

#[derive(Debug, Serialize)]
pub struct BackupInfo {
    pub path: Option<String>,
    pub exists: bool,
}

/// Errors as `{ kind, message }` so the UI can pick a hint from the kind and show the message.
#[derive(Debug, Serialize)]
pub struct AppError {
    pub kind: &'static str,
    pub message: String,
}

impl AppError {
    pub fn new(kind: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn no_session() -> Self {
        Self::new("no_session", "No keyboard is connected.")
    }
}

impl From<Error> for AppError {
    fn from(e: Error) -> Self {
        let kind = match &e {
            Error::NotFound => "not_found",
            Error::Io(_) => "io",
            Error::Timeout => "timeout",
            Error::BadReply(_) => "bad_reply",
            Error::VerifyFailed { .. } => "verify_failed",
            Error::Layout(_) => "layout",
            Error::Profile(_) => "profile",
        };
        Self::new(kind, e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_carry_a_kind_the_ui_can_branch_on() {
        assert_eq!(AppError::from(Error::NotFound).kind, "not_found");
        assert_eq!(AppError::from(Error::Timeout).kind, "timeout");
        let v = Error::VerifyFailed {
            layer: 0,
            row: 1,
            col: 2,
            wrote: 4,
            read: 0,
        };
        let e = AppError::from(v);
        assert_eq!(e.kind, "verify_failed");
        assert!(e.message.contains("layer 0"));
    }

    #[test]
    fn connect_info_uses_camel_case() {
        let info = ConnectInfo {
            model_id: "m".into(),
            label: "L".into(),
            verified: true,
            demo: false,
            mac: true,
            product: "p".into(),
            vendor_id: 1,
            product_id: 2,
            protocol: 12,
            layers: 6,
            rows: 6,
            cols: 15,
            keys: vec![],
        };
        let v = serde_json::to_value(info).unwrap();
        assert!(v.get("modelId").is_some() && v.get("productId").is_some());
    }
}
