//! `Source/utils/paths.cpp`: base (data), pref (saves), config and assets directories.
//!
//! Known difference: the original's pref/config default is SDL_GetPrefPath("diasurgical",
//! "devilution") (`%APPDATA%\diasurgical\devilution\`). The port uses
//! `%APPDATA%\diablo1_rs\devilution\` so it never reads or overwrites an existing DevilutionX
//! profile or saves. `--save-dir` / `--config-dir` override it as in the original.

use std::path::Path;

pub const DIRECTORY_SEPARATOR: char = '\\';

const PREF_ORG: &str = "diablo1_rs";
const PREF_APP: &str = "devilution";

#[derive(Default, Debug, Clone)]
pub struct Paths {
    base_path: Option<String>,
    pref_path: Option<String>,
    config_path: Option<String>,
    assets_path: Option<String>,
}

/// Original: `paths::AddTrailingSlash` (utils/paths.cpp).
// @port utils/paths.cpp|devilution::paths::AddTrailingSlash(std::string &path) sha=863204feae14
fn add_trailing_slash(path: &mut String) {
    if !path.is_empty() && !path.ends_with(DIRECTORY_SEPARATOR) && !path.ends_with('/') {
        path.push(DIRECTORY_SEPARATOR);
    }
}

/// `SDL_GetBasePath`: the folder holding the executable, with a trailing separator.
fn sdl_base_path() -> String {
    let mut p = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|d| d.to_string_lossy().into_owned()))
        .unwrap_or_default();
    add_trailing_slash(&mut p);
    p
}

/// `SDL_GetPrefPath(org, app)` on Windows: `%APPDATA%\org\app\`, created if missing.
fn sdl_pref_path(org: &str, app: &str) -> String {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let mut p = Path::new(&appdata).join(org).join(app).to_string_lossy().into_owned();
    let _ = std::fs::create_dir_all(&p);
    add_trailing_slash(&mut p);
    p
}

/// `FileExistsAndIsWriteable("diablo.ini")` relative to the working directory.
fn local_ini_is_writeable() -> bool {
    std::fs::metadata("diablo.ini").map(|m| !m.permissions().readonly()).unwrap_or(false)
}

impl Paths {
    /// Original: `paths::BasePath` (utils/paths.cpp).
    // @port utils/paths.cpp|devilution::paths::BasePath() sha=00e5088b5f0d
    pub fn base_path(&mut self) -> &str {
        self.base_path.get_or_insert_with(sdl_base_path)
    }

    /// Original: `paths::PrefPath` (utils/paths.cpp).
    // @port utils/paths.cpp|devilution::paths::PrefPath() sha=969d127d118a
    pub fn pref_path(&mut self) -> &str {
        self.pref_path.get_or_insert_with(|| {
            if local_ini_is_writeable() { format!(".{DIRECTORY_SEPARATOR}") } else { sdl_pref_path(PREF_ORG, PREF_APP) }
        })
    }

    /// Original: `paths::ConfigPath` (utils/paths.cpp).
    // @port utils/paths.cpp|devilution::paths::ConfigPath() sha=8e831193cdd5
    pub fn config_path(&mut self) -> &str {
        self.config_path.get_or_insert_with(|| {
            if local_ini_is_writeable() { format!(".{DIRECTORY_SEPARATOR}") } else { sdl_pref_path(PREF_ORG, PREF_APP) }
        })
    }

    /// Original: `paths::AssetsPath` (utils/paths.cpp).
    // @port utils/paths.cpp|devilution::paths::AssetsPath() sha=9d78759a3233
    pub fn assets_path(&mut self) -> &str {
        self.assets_path.get_or_insert_with(|| format!("{}assets{DIRECTORY_SEPARATOR}", sdl_base_path()))
    }

    /// Original: `paths::SetBasePath` (utils/paths.cpp).
    // @port utils/paths.cpp|devilution::paths::SetBasePath(const std::string &path) sha=9df51e8a6c34
    pub fn set_base_path(&mut self, path: &str) {
        let mut p = path.to_string();
        add_trailing_slash(&mut p);
        self.base_path = Some(p);
    }

    /// Original: `paths::SetPrefPath` (utils/paths.cpp).
    // @port utils/paths.cpp|devilution::paths::SetPrefPath(const std::string &path) sha=aca438a4657c
    pub fn set_pref_path(&mut self, path: &str) {
        let mut p = path.to_string();
        add_trailing_slash(&mut p);
        self.pref_path = Some(p);
    }

    /// Original: `paths::SetConfigPath` (utils/paths.cpp).
    // @port utils/paths.cpp|devilution::paths::SetConfigPath(const std::string &path) sha=7f5a82700f24
    pub fn set_config_path(&mut self, path: &str) {
        let mut p = path.to_string();
        add_trailing_slash(&mut p);
        self.config_path = Some(p);
    }

    /// Original: `paths::SetAssetsPath` (utils/paths.cpp).
    // @port utils/paths.cpp|devilution::paths::SetAssetsPath(const std::string &path) sha=0883418fbb6b
    pub fn set_assets_path(&mut self, path: &str) {
        let mut p = path.to_string();
        add_trailing_slash(&mut p);
        self.assets_path = Some(p);
    }
}
