//! bo2mp: a Black Ops II mod folder (Plutonium's layout: `mod.json`,
//! `mod.ff`, `mod.iwd`, `*.all.sabl` / `*.all.sabs`) layered over the
//! install, read only. `IW4L_T6_MOD` names the folder (`.env` or the
//! environment; `0` / `off` turns it off); unset, `E:\mp_leetin_pub` when
//! that folder is there.
//!
//! The mod's zone (`mod.ff`) loads after Black Ops II's shared zones; its
//! archives (`*.iwd`) hold the pictures (`images/<name>.iwi`) and the
//! animations (`xanim/<name>`) its zone names but does not carry; its sound
//! banks sit beside them.

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// The variable that names the mod folder.
pub const ENV: &str = "IW4L_T6_MOD";

/// The folder used when `IW4L_T6_MOD` is not set, if it is there.
pub const DEFAULT_FOLDER: &str = r"E:\mp_leetin_pub";

/// The mod folder for a setting (`IW4L_T6_MOD`'s value, if set): the named
/// folder; none for `0`, `off`, `none` or an empty value; unset, the
/// default folder when `is_dir` says it is there.
#[must_use]
pub fn folder_from(setting: Option<&str>, is_dir: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    match setting.map(|s| s.trim().trim_matches('"')) {
        Some(s) if s.is_empty() || ["0", "off", "none", "false"].iter().any(|o| s.eq_ignore_ascii_case(o)) => None,
        Some(s) => Some(PathBuf::from(s)).filter(|p| is_dir(p)),
        None => Some(PathBuf::from(DEFAULT_FOLDER)).filter(|p| is_dir(p)),
    }
}

/// The mod folder, found once (none: no mod).
pub fn root() -> Option<&'static Path> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| {
        crate::discover::load_dotenv();
        let setting = std::env::var(ENV).ok();
        folder_from(setting.as_deref(), Path::is_dir)
    })
    .as_deref()
}

/// The mod's zone (`mod.ff`), if it has one.
pub fn zone() -> Option<PathBuf> {
    root().map(|r| r.join("mod.ff")).filter(|p| p.is_file())
}

/// The files in the mod folder (not its subfolders) whose extension is
/// `ext` (any case), sorted.
pub fn files(ext: &str) -> Vec<PathBuf> {
    let Some(root) = root() else {
        return Vec::new();
    };
    let Ok(read) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = read
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x.eq_ignore_ascii_case(ext)))
        .collect();
    out.sort();
    out
}

/// What `mod.json` says of the mod.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModInfo {
    pub name: String,
    pub author: String,
    pub version: String,
}

/// `mod.json`'s name, author and version (a flat object of strings).
#[must_use]
pub fn parse_mod_json(text: &str) -> ModInfo {
    let field = |key: &str| -> String {
        let quoted = format!("\"{key}\"");
        let Some(at) = text.find(&quoted) else {
            return String::new();
        };
        let rest = &text[at + quoted.len()..];
        let Some(colon) = rest.find(':') else {
            return String::new();
        };
        let rest = rest[colon + 1..].trim_start();
        let Some(rest) = rest.strip_prefix('"') else {
            return String::new();
        };
        rest.split('"').next().unwrap_or("").to_owned()
    };
    ModInfo { name: field("name"), author: field("author"), version: field("version") }
}

/// The mod's `mod.json` (empty fields when it has none).
pub fn info() -> Option<ModInfo> {
    let root = root()?;
    let text = std::fs::read_to_string(root.join("mod.json")).unwrap_or_default();
    Some(parse_mod_json(&text))
}

type Zip = zip::ZipArchive<std::io::BufReader<std::fs::File>>;

/// One of the mod's archives, open for reading: its entries by lower-case
/// name (`images/i_iw4_cheytac_c.iwi`, `xanim/viewmodel_cheytac_idle`).
pub struct ModArchive {
    pub path: PathBuf,
    entries: HashMap<String, usize>,
    zip: Mutex<Zip>,
}

impl ModArchive {
    pub fn open(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file)).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut entries = HashMap::new();
        for i in 0..zip.len() {
            let Ok(entry) = zip.by_index_raw(i) else {
                continue;
            };
            if entry.is_dir() {
                continue;
            }
            entries.insert(entry.name().replace('\\', "/").to_ascii_lowercase(), i);
        }
        Ok(Self { path: path.to_path_buf(), entries, zip: Mutex::new(zip) })
    }

    /// Every entry's name (lower case).
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    pub fn contains(&self, entry: &str) -> bool {
        self.entries.contains_key(&entry.replace('\\', "/").to_ascii_lowercase())
    }

    /// An entry's bytes (none when the archive has no such entry).
    pub fn read(&self, entry: &str) -> Option<Result<Vec<u8>, String>> {
        let &index = self.entries.get(&entry.replace('\\', "/").to_ascii_lowercase())?;
        let mut zip = self.zip.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        Some((|| {
            let mut file = zip.by_index(index).map_err(|e| format!("{entry} in {}: {e}", self.path.display()))?;
            let mut out = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut out).map_err(|e| format!("{entry} in {}: {e}", self.path.display()))?;
            Ok(out)
        })())
    }
}

/// The mod's archives (`*.iwd` in its folder), opened once, in name order.
pub fn archives() -> &'static [ModArchive] {
    static ARCHIVES: OnceLock<Vec<ModArchive>> = OnceLock::new();
    ARCHIVES.get_or_init(|| {
        files("iwd")
            .iter()
            .filter_map(|p| match ModArchive::open(p) {
                Ok(a) => Some(a),
                Err(e) => {
                    eprintln!("bo2mp mod archive: {e}");
                    None
                }
            })
            .collect()
    })
}

/// An entry of the mod's archives (a later archive wins a name, as the
/// game loads them).
pub fn read_entry(entry: &str) -> Option<Vec<u8>> {
    archives().iter().rev().find_map(|a| a.read(entry)).and_then(Result::ok)
}

/// Whether the mod's archives hold an entry.
pub fn has_entry(entry: &str) -> bool {
    archives().iter().any(|a| a.contains(entry))
}

/// A picture of the mod's archives (`images/<name>.iwi`), by image name.
pub fn image(name: &str) -> Option<Vec<u8>> {
    read_entry(&format!("images/{}.iwi", name.trim_start_matches(',')))
}

/// An animation file of the mod's archives (`xanim/<name>`).
pub fn xanim(name: &str) -> Option<Vec<u8>> {
    read_entry(&format!("xanim/{}", name.trim_start_matches(',')))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_folder_follows_the_setting_and_defaults_when_present() {
        let there = |_: &Path| true;
        let absent = |_: &Path| false;
        assert_eq!(folder_from(None, there), Some(PathBuf::from(DEFAULT_FOLDER)));
        assert_eq!(folder_from(None, absent), None);
        assert_eq!(folder_from(Some("D:/mods/x"), there), Some(PathBuf::from("D:/mods/x")));
        assert_eq!(folder_from(Some("\"D:/mods/x\""), there), Some(PathBuf::from("D:/mods/x")));
        assert_eq!(folder_from(Some("D:/mods/x"), absent), None);
        for off in ["0", "off", "OFF", "", "none"] {
            assert_eq!(folder_from(Some(off), there), None, "{off}");
        }
    }

    #[test]
    fn mod_json_fields() {
        let info = parse_mod_json(r#"{"name":"@leetinservers","author":"j & bbsaabaru","version":"2.0.0"}"#);
        assert_eq!(info.name, "@leetinservers");
        assert_eq!(info.author, "j & bbsaabaru");
        assert_eq!(info.version, "2.0.0");
        assert_eq!(parse_mod_json("{}"), ModInfo::default());
    }

    #[test]
    fn archive_entries_by_any_case() {
        let dir = std::env::temp_dir().join(format!("t6_mod_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("mod.iwd");
        {
            let mut w = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
            let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            w.start_file("xanim/Viewmodel_Test_Idle", opts).unwrap();
            std::io::Write::write_all(&mut w, b"\x13\x00abc").unwrap();
            w.finish().unwrap();
        }
        let a = ModArchive::open(&path).unwrap();
        assert!(a.contains("xanim/viewmodel_test_idle"));
        assert_eq!(a.read("XANIM/viewmodel_test_idle").unwrap().unwrap(), b"\x13\x00abc");
        assert!(a.read("images/nothing.iwi").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
