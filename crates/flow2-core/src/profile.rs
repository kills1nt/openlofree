use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::backlight::{self, Backlight};
use crate::client::ViaClient;
use crate::error::{Error, Result};
use crate::keymap::{self, Keymap};
use crate::layout::ModelDef;
use crate::transport::Transport;

pub const SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub schema: u32,
    pub name: String,
    /// Model registry id, for example "flow2-mac-84".
    pub model: String,
    pub rows: u8,
    pub cols: u8,
    /// layers[l][row * cols + col]
    pub layers: Vec<Vec<u16>>,
    pub backlight: Backlight,
}

impl Profile {
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| Error::Profile(e.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self> {
        let p: Profile = serde_json::from_str(json).map_err(|e| Error::Profile(e.to_string()))?;
        if p.schema != SCHEMA {
            return Err(Error::Profile(format!("unsupported schema {}", p.schema)));
        }
        let want = p.rows as usize * p.cols as usize;
        if p.layers.is_empty() || p.layers.iter().any(|l| l.len() != want) {
            return Err(Error::Profile(format!("every layer must have {want} keys")));
        }
        Ok(p)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| Error::Profile(e.to_string()))?;
        }
        fs::write(path, self.to_json()?).map_err(|e| Error::Profile(e.to_string()))
    }

    pub fn load(path: &Path) -> Result<Self> {
        Self::from_json(&fs::read_to_string(path).map_err(|e| Error::Profile(e.to_string()))?)
    }
}

/// Reads the keyboard's current keymap and backlight into a profile.
pub fn capture<T: Transport>(
    client: &mut ViaClient<T>,
    model_id: &str,
    def: &ModelDef,
    name: &str,
) -> Result<Profile> {
    let km = keymap::read(client, def.rows, def.cols)?;
    Ok(Profile {
        schema: SCHEMA,
        name: name.to_string(),
        model: model_id.to_string(),
        rows: def.rows,
        cols: def.cols,
        layers: km.layers,
        backlight: backlight::read(client)?,
    })
}

/// Writes a profile to the keyboard. Returns how many keys changed.
/// Refuses a profile whose matrix size differs from the keyboard's.
pub fn apply<T: Transport>(
    client: &mut ViaClient<T>,
    def: &ModelDef,
    p: &Profile,
) -> Result<usize> {
    if (p.rows, p.cols) != (def.rows, def.cols) {
        return Err(Error::Profile(format!(
            "profile matrix is {}x{}, keyboard is {}x{}",
            p.rows, p.cols, def.rows, def.cols
        )));
    }
    let changed = keymap::apply(
        client,
        &Keymap {
            rows: p.rows,
            cols: p.cols,
            layers: p.layers.clone(),
        },
    )?;
    backlight::apply(client, p.backlight)?;
    Ok(changed)
}

pub fn factory_backup_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("openlofree").join("factory-backup.json"))
}

/// Saves the keyboard's current state once, before the first write. Never overwrites.
/// Returns true if a backup was created now.
pub fn ensure_factory_backup<T: Transport>(
    client: &mut ViaClient<T>,
    model_id: &str,
    def: &ModelDef,
    path: &Path,
) -> Result<bool> {
    if path.exists() {
        return Ok(false);
    }
    capture(client, model_id, def, "Factory backup")?.save(path)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backlight::Mode;
    use crate::transport::mock::MockDevice;

    fn def() -> ModelDef {
        ModelDef {
            name: "T".into(),
            vendor_id: 0x388D,
            product_id: 1,
            rows: 2,
            cols: 3,
            keys: vec![],
        }
    }

    fn client() -> ViaClient<MockDevice> {
        ViaClient::new(MockDevice::new(2, 3, 2))
    }

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("openlofree-test-{}-{name}", std::process::id()));
        let _ = fs::remove_file(&p);
        p
    }

    #[test]
    fn json_round_trip() {
        let p = capture(&mut client(), "test", &def(), "Mine").unwrap();
        assert_eq!(Profile::from_json(&p.to_json().unwrap()).unwrap(), p);
    }

    #[test]
    fn rejects_wrong_schema_and_shape() {
        let mut p = capture(&mut client(), "test", &def(), "Mine").unwrap();
        p.schema = 99;
        assert!(matches!(
            Profile::from_json(&p.to_json().unwrap()),
            Err(Error::Profile(_))
        ));
        p.schema = SCHEMA;
        p.layers[0].pop();
        assert!(matches!(
            Profile::from_json(&p.to_json().unwrap()),
            Err(Error::Profile(_))
        ));
        assert!(matches!(Profile::from_json("{}"), Err(Error::Profile(_))));
    }

    #[test]
    fn apply_restores_a_captured_profile() {
        let mut c = client();
        let original = capture(&mut c, "test", &def(), "Original").unwrap();
        c.set_keycode(0, 0, 0, 0x0004).unwrap();
        backlight::apply(
            &mut c,
            Backlight {
                mode: Mode::Breathing,
                brightness: 40,
            },
        )
        .unwrap();
        assert_eq!(apply(&mut c, &def(), &original).unwrap(), 1);
        assert_eq!(
            capture(&mut c, "test", &def(), "Original").unwrap(),
            original
        );
    }

    #[test]
    fn apply_refuses_a_profile_for_a_different_matrix() {
        let mut c = client();
        let mut p = capture(&mut c, "test", &def(), "Other").unwrap();
        p.cols = 4;
        assert!(matches!(apply(&mut c, &def(), &p), Err(Error::Profile(_))));
    }

    #[test]
    fn factory_backup_is_created_once_and_never_overwritten() {
        let path = tmp("factory.json");
        let mut c = client();
        assert!(ensure_factory_backup(&mut c, "test", &def(), &path).unwrap());
        c.set_keycode(0, 0, 0, 0x0004).unwrap();
        assert!(!ensure_factory_backup(&mut c, "test", &def(), &path).unwrap());
        assert_eq!(Profile::load(&path).unwrap().layers[0][0], 0);
        let _ = fs::remove_file(path);
    }
}
