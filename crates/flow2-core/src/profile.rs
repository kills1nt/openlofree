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
        // Write beside the target and rename, so a crash never leaves a truncated profile behind.
        let tmp = path.with_extension("json.tmp");
        let io = |e: std::io::Error| Error::Profile(e.to_string());
        fs::write(&tmp, self.to_json()?).map_err(io)?;
        fs::rename(&tmp, path).map_err(io)
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

/// Summary of a stored profile for lists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProfileInfo {
    pub name: String,
    pub model: String,
    pub layers: usize,
}

/// Profiles as one JSON file each in a directory. The name is kept inside the file, the file name is
/// only `profile-<slug>.json`, so odd names cannot escape the directory or hit Windows device names.
pub struct ProfileStore {
    dir: PathBuf,
}

const MAX_NAME_CHARS: usize = 60;

fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.trim().chars().flat_map(char::to_lowercase) {
        if c.is_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

impl ProfileStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn default_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("openlofree").join("profiles"))
    }

    fn path_for(&self, name: &str) -> Result<PathBuf> {
        let trimmed = name.trim();
        if trimmed.is_empty() || trimmed.chars().count() > MAX_NAME_CHARS {
            return Err(Error::Profile(format!(
                "name must be 1 to {MAX_NAME_CHARS} characters"
            )));
        }
        let slug = slug(trimmed);
        if slug.is_empty() {
            return Err(Error::Profile(
                "name needs at least one letter or digit".into(),
            ));
        }
        Ok(self.dir.join(format!("profile-{slug}.json")))
    }

    pub fn list(&self) -> Result<Vec<ProfileInfo>> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Ok(Vec::new());
        };
        let mut out: Vec<ProfileInfo> = entries
            .flatten()
            .filter(|e| {
                let n = e.file_name();
                let n = n.to_string_lossy();
                n.starts_with("profile-") && n.ends_with(".json")
            })
            .filter_map(|e| Profile::load(&e.path()).ok())
            .map(|p| ProfileInfo {
                layers: p.layers.len(),
                name: p.name,
                model: p.model,
            })
            .collect();
        out.sort_by_key(|p| p.name.to_lowercase());
        Ok(out)
    }

    pub fn load(&self, name: &str) -> Result<Profile> {
        let p = Profile::load(&self.path_for(name)?)
            .map_err(|_| Error::Profile(format!("no profile named {name:?}")))?;
        if p.name == name {
            Ok(p)
        } else {
            Err(Error::Profile(format!("no profile named {name:?}")))
        }
    }

    /// Saves under the profile's name, replacing a profile with exactly that name.
    /// Refuses when a differently named profile already owns the file name.
    pub fn save(&self, p: &Profile) -> Result<()> {
        let path = self.path_for(&p.name)?;
        if let Ok(existing) = Profile::load(&path) {
            if existing.name != p.name {
                return Err(Error::Profile(format!(
                    "the name is too close to the existing profile {:?}",
                    existing.name
                )));
            }
        }
        p.save(&path)
    }

    pub fn delete(&self, name: &str) -> Result<()> {
        self.load(name)?;
        fs::remove_file(self.path_for(name)?).map_err(|e| Error::Profile(e.to_string()))
    }

    pub fn duplicate(&self, name: &str, new_name: &str) -> Result<()> {
        let mut p = self.load(name)?;
        if self.path_for(new_name)?.exists() {
            return Err(Error::Profile(format!(
                "a profile named {new_name:?} already exists"
            )));
        }
        p.name = new_name.trim().to_string();
        self.save(&p)
    }
}

/// One factory backup per model, so a second keyboard never inherits the first one's.
pub fn factory_backup_path(model_id: &str) -> Option<PathBuf> {
    dirs::config_dir().map(|d| {
        d.join("openlofree")
            .join(format!("factory-backup-{model_id}.json"))
    })
}

/// Saves the keyboard's current state once, before the first write. Never overwrites a readable backup.
/// Returns true if a backup was created now.
pub fn ensure_factory_backup<T: Transport>(
    client: &mut ViaClient<T>,
    model_id: &str,
    def: &ModelDef,
    path: &Path,
) -> Result<bool> {
    if path.exists() && Profile::load(path).is_ok() {
        return Ok(false);
    }
    let backup = capture(client, model_id, def, "Factory backup")?;
    Profile::from_json(&backup.to_json()?)?; // never save a backup that could not be loaded again
    backup.save(path)?;
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

    fn store(name: &str) -> ProfileStore {
        let dir =
            std::env::temp_dir().join(format!("openlofree-store-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        ProfileStore::new(dir)
    }

    fn sample(name: &str) -> Profile {
        capture(&mut client(), "test", &def(), name).unwrap()
    }

    #[test]
    fn store_saves_lists_and_loads() {
        let s = store("basic");
        assert!(s.list().unwrap().is_empty());
        s.save(&sample("Gaming")).unwrap();
        s.save(&sample("Work")).unwrap();
        let names: Vec<_> = s.list().unwrap().into_iter().map(|p| p.name).collect();
        assert_eq!(names, ["Gaming", "Work"]);
        assert_eq!(s.load("Work").unwrap().name, "Work");
        assert!(s.load("Nope").is_err());
    }

    #[test]
    fn store_save_overwrites_the_same_name() {
        let s = store("overwrite");
        let mut p = sample("Work");
        s.save(&p).unwrap();
        p.layers[0][0] = 0x0004;
        s.save(&p).unwrap();
        assert_eq!(s.list().unwrap().len(), 1);
        assert_eq!(s.load("Work").unwrap().layers[0][0], 0x0004);
    }

    #[test]
    fn store_delete_and_duplicate() {
        let s = store("dupdel");
        s.save(&sample("Work")).unwrap();
        s.duplicate("Work", "Work copy").unwrap();
        assert_eq!(s.load("Work copy").unwrap().name, "Work copy");
        assert!(s.duplicate("Work", "Work copy").is_err());
        s.delete("Work").unwrap();
        assert!(s.load("Work").is_err());
        assert!(s.delete("Work").is_err());
        assert_eq!(s.list().unwrap().len(), 1);
    }

    #[test]
    fn store_names_that_share_a_file_name_do_not_clobber_each_other() {
        let s = store("collide");
        s.save(&sample("My Profile")).unwrap();
        assert!(matches!(
            s.save(&sample("my-profile")),
            Err(Error::Profile(_))
        ));
        assert_eq!(s.load("My Profile").unwrap().name, "My Profile");
    }

    #[test]
    fn store_keeps_hostile_names_inside_its_directory() {
        let s = store("hostile");
        s.save(&sample("../evil")).unwrap();
        let outside = s.dir.parent().unwrap().join("evil.json");
        assert!(!outside.exists());
        assert_eq!(s.load("../evil").unwrap().name, "../evil");
        // Windows reserved device names must not become file names.
        s.save(&sample("con")).unwrap();
        assert_eq!(s.load("con").unwrap().name, "con");
    }

    #[test]
    fn store_rejects_empty_and_overlong_names() {
        let s = store("names");
        assert!(matches!(s.save(&sample("   ")), Err(Error::Profile(_))));
        assert!(matches!(s.save(&sample("///")), Err(Error::Profile(_))));
        assert!(matches!(
            s.save(&sample(&"x".repeat(61))),
            Err(Error::Profile(_))
        ));
    }

    #[test]
    fn store_list_skips_broken_files() {
        let s = store("broken");
        s.save(&sample("Good")).unwrap();
        fs::write(s.dir.join("profile-bad.json"), "{ not json").unwrap();
        let names: Vec<_> = s.list().unwrap().into_iter().map(|p| p.name).collect();
        assert_eq!(names, ["Good"]);
    }

    #[test]
    fn factory_backup_path_is_per_model() {
        let a = factory_backup_path("m1").unwrap();
        let b = factory_backup_path("m2").unwrap();
        assert_ne!(a, b);
        assert!(a.file_name().unwrap().to_string_lossy().contains("m1"));
    }

    #[test]
    fn save_writes_a_temp_file_then_renames_it() {
        let path = tmp("atomic.json");
        sample("A").save(&path).unwrap();
        assert!(Profile::load(&path).is_ok());
        assert!(!path.with_extension("json.tmp").exists());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn unreadable_factory_backup_is_replaced() {
        let path = tmp("broken-backup.json");
        fs::write(&path, "{ broken").unwrap();
        assert!(ensure_factory_backup(&mut client(), "test", &def(), &path).unwrap());
        assert!(Profile::load(&path).is_ok());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn backup_of_a_keyboard_reporting_no_layers_is_refused() {
        let path = tmp("no-layers.json");
        let mut c = ViaClient::new(MockDevice::new(2, 3, 0));
        assert!(matches!(
            ensure_factory_backup(&mut c, "test", &def(), &path),
            Err(Error::Profile(_))
        ));
        assert!(!path.exists());
    }
}
