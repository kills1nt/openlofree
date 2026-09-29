//! One keyboard session at a time, shared by every command and the tray.
use std::sync::Mutex;

use flow2_core::client::ViaClient;
use flow2_core::keycodes::Variant;
use flow2_core::layout::ModelDef;
use flow2_core::profile::{self, Profile, ProfileStore};
use flow2_core::transport::Transport;
use flow2_core::{hid, models, Error};

use crate::demo;
use crate::dto::{AppError, ConnectInfo};

pub type Client = ViaClient<Box<dyn Transport + Send>>;

pub struct Session {
    pub client: Client,
    pub model_id: String,
    pub label: String,
    pub verified: bool,
    pub variant: Variant,
    pub def: ModelDef,
    pub layers: u8,
    pub demo: bool,
    /// The factory backup was checked or created in this session.
    pub backed_up: bool,
    pub product: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub protocol: u16,
}

impl Session {
    /// Call before any write to a real keyboard. Creates the factory backup once, and refuses to
    /// write when there is nowhere to keep it. Demo sessions never touch hardware or disk.
    pub fn guard_write(&mut self) -> Result<(), Error> {
        if self.demo || self.backed_up {
            return Ok(());
        }
        let path = profile::factory_backup_path(&self.model_id).ok_or_else(|| {
            Error::Profile("no config directory for the factory backup, refusing to write".into())
        })?;
        profile::ensure_factory_backup(&mut self.client, &self.model_id, &self.def, &path)?;
        self.backed_up = true;
        Ok(())
    }

    pub fn info(&self) -> ConnectInfo {
        ConnectInfo {
            model_id: self.model_id.clone(),
            label: self.label.clone(),
            verified: self.verified,
            demo: self.demo,
            mac: self.variant == Variant::Mac,
            product: self.product.clone(),
            vendor_id: self.vendor_id,
            product_id: self.product_id,
            protocol: self.protocol,
            layers: self.layers,
            rows: self.def.rows,
            cols: self.def.cols,
            keys: self.def.keys.clone(),
        }
    }
}

/// Opens the demo keyboard or the first real one.
pub fn open_session(demo_mode: bool) -> Result<Session, Error> {
    let (client, model, def, product, vendor_id, product_id) = if demo_mode {
        let model = &models::MODELS[0];
        let def = model.definition()?;
        let dev = demo::demo_device(&def);
        let client: Client = ViaClient::new(Box::new(dev));
        let pid = def.product_id;
        (client, model, def, "Demo keyboard (no hardware)".to_string(), hid::VENDOR_ID, pid)
    } else {
        let (transport, info) = hid::find()?;
        let (model, def) = models::by_product_id(info.product_id).ok_or_else(|| {
            Error::BadReply(format!(
                "unsupported model, product id {:#06x} ({}). Please open an issue at github.com/kills1nt/openlofree",
                info.product_id, info.product
            ))
        })?;
        let client: Client = ViaClient::new(Box::new(transport));
        (client, model, def, info.product, info.vendor_id, info.product_id)
    };
    let mut client = client;
    let protocol = client.protocol_version()?;
    let layers = client.layer_count()?;
    Ok(Session {
        client,
        model_id: model.id.to_string(),
        label: model.label.to_string(),
        verified: model.verified,
        variant: model.variant,
        def,
        layers,
        demo: demo_mode,
        backed_up: false,
        product,
        vendor_id,
        product_id,
        protocol,
    })
}

pub struct AppState {
    session: Mutex<Option<Session>>,
    pub store: ProfileStore,
    battery: Mutex<Option<u8>>,
}

impl AppState {
    pub fn new() -> Self {
        let dir = ProfileStore::default_dir()
            .unwrap_or_else(|| std::env::temp_dir().join("openlofree-profiles"));
        Self { session: Mutex::new(None), store: ProfileStore::new(dir), battery: Mutex::new(None) }
    }

    /// Runs `f` on the current session. Errors from the keyboard become `AppError`s.
    pub fn with<T>(&self, f: impl FnOnce(&mut Session) -> Result<T, Error>) -> Result<T, AppError> {
        let mut guard = self
            .session
            .lock()
            .map_err(|_| AppError::new("internal", "session lock poisoned"))?;
        let session = guard.as_mut().ok_or_else(AppError::no_session)?;
        f(session).map_err(AppError::from)
    }

    pub fn connect(&self, demo_mode: bool) -> Result<ConnectInfo, AppError> {
        let session = open_session(demo_mode)?;
        let info = session.info();
        *self
            .session
            .lock()
            .map_err(|_| AppError::new("internal", "session lock poisoned"))? = Some(session);
        Ok(info)
    }

    pub fn disconnect(&self) {
        if let Ok(mut g) = self.session.lock() {
            *g = None;
        }
    }

    pub fn battery(&self) -> Option<u8> {
        self.battery.lock().ok().and_then(|b| *b)
    }

    pub fn set_battery(&self, value: Option<u8>) {
        if let Ok(mut b) = self.battery.lock() {
            *b = value;
        }
    }

    /// Loads a stored profile and writes it to the connected keyboard. Returns keys changed.
    pub fn apply_profile(&self, name: &str) -> Result<usize, AppError> {
        let p = self.store.load(name)?;
        self.with(|s| {
            if p.model != s.model_id {
                return Err(Error::Profile(format!(
                    "profile is for {}, keyboard is {}",
                    p.model, s.model_id
                )));
            }
            s.guard_write()?;
            profile::apply(&mut s.client, &s.def, &p)
        })
    }

    /// Builds a profile from the UI's current data for the connected model.
    pub fn build_profile(
        &self,
        name: &str,
        layers: Vec<Vec<u16>>,
        backlight: flow2_core::backlight::Backlight,
    ) -> Result<Profile, AppError> {
        let p = self.with(|s| {
            Ok(Profile {
                schema: profile::SCHEMA,
                name: name.trim().to_string(),
                model: s.model_id.clone(),
                rows: s.def.rows,
                cols: s.def.cols,
                layers,
                backlight,
            })
        })?;
        Profile::from_json(&p.to_json()?)?; // same shape check a loaded file gets
        Ok(p)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flow2_core::backlight::{Backlight, Mode};

    fn connected() -> AppState {
        let s = AppState::new();
        s.connect(true).unwrap();
        s
    }

    #[test]
    fn commands_need_a_session() {
        let s = AppState::new();
        assert_eq!(s.with(|_| Ok(())).unwrap_err().kind, "no_session");
    }

    #[test]
    fn demo_session_reports_the_84_mac_model() {
        let info = AppState::new().connect(true).unwrap();
        assert_eq!((info.model_id.as_str(), info.layers, info.rows, info.cols), ("flow2-mac-84", 6, 6, 15));
        assert!(info.demo && info.mac && info.verified);
        assert_eq!(info.keys.len(), 84);
    }

    #[test]
    fn demo_writes_do_not_need_a_factory_backup() {
        let s = connected();
        s.with(|s| {
            s.guard_write()?;
            flow2_core::keymap::set_key_verified(&mut s.client, 0, 0, 0, 0x0004)
        })
        .unwrap();
    }

    #[test]
    fn disconnect_drops_the_session() {
        let s = connected();
        s.disconnect();
        assert_eq!(s.with(|_| Ok(())).unwrap_err().kind, "no_session");
    }

    #[test]
    fn build_profile_rejects_a_wrong_shaped_keymap() {
        let s = connected();
        let bl = Backlight { mode: Mode::Steady, brightness: 200 };
        assert!(s.build_profile("Mine", vec![vec![0; 3]], bl).is_err());
        let good = s.with(|s| flow2_core::keymap::read(&mut s.client, 6, 15)).unwrap();
        assert!(s.build_profile("Mine", good.layers, bl).is_ok());
    }
}
