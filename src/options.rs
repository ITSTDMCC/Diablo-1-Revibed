//! `Source/options.cpp` / `options.h`: game options and diablo.ini.
//!
//! The C++ class hierarchy (OptionEntryBase with virtual Load/Save/list methods) becomes the
//! `OptionEntry` trait. Value-changed callbacks are `OptionCallback` values run by
//! `run_option_callback` with the game context, right where the original calls them.

use std::cell::RefCell;

use crate::controls::controller_buttons::{ControllerButton, ControllerButtonCombo};
use crate::ctx::Ctx;
use crate::ini::Ini;
use crate::platform::events::keys::*;
use crate::platform::events::{BUTTON_MIDDLE, BUTTON_X1, BUTTON_X2};
use crate::platform::log;
use crate::utils::language::tr;
use crate::utils::paths::Paths;

pub const DEFAULT_WIDTH: i32 = 640;
pub const DEFAULT_HEIGHT: i32 = 480;
const DEFAULT_AUDIO_SAMPLE_RATE: i32 = 22050;
const DEFAULT_AUDIO_CHANNELS: i32 = 2;
const DEFAULT_AUDIO_BUFFER_SIZE: i32 = 2048;
const DEFAULT_AUDIO_RESAMPLING_QUALITY: i32 = 3;
/// `VOLUME_MAX` (engine/sound_defs.hpp)
pub const VOLUME_MAX: i32 = 0;
pub const KEYMAPPER_MOUSE_BUTTON_MASK: u32 = 1 << 31;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum StartUpGameMode {
    Ask = 0,
    Hellfire = 1,
    Diablo = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum StartUpIntro {
    Off = 0,
    Once = 1,
    On = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum StartUpSplash {
    None = 0,
    TitleDialog = 1,
    LogoAndTitleDialog = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ScalingQuality {
    NearestPixel,
    BilinearFiltering,
    AnisotropicFiltering,
}

/// Both resamplers are compiled into the Windows build (build_config.json).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Resampler {
    Speex = 0,
    Sdl = 1,
}
const NUM_RESAMPLERS: usize = 2;
/// `DEVILUTIONX_DEFAULT_RESAMPLER`: the first available one, Speex (CMakeLists.txt).
const DEFAULT_RESAMPLER: Resampler = Resampler::Speex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FloatingNumbers {
    Off = 0,
    Random = 1,
    Vertical = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionEntryType {
    Boolean,
    List,
    Key,
    PadButton,
}

/// `OptionEntryFlags`
pub mod flags {
    pub const NONE: u8 = 0;
    pub const INVISIBLE: u8 = 1 << 0;
    pub const CANT_CHANGE_IN_GAME: u8 = 1 << 1;
    pub const CANT_CHANGE_IN_MULTI_PLAYER: u8 = 1 << 2;
    pub const ONLY_HELLFIRE: u8 = 1 << 3;
    pub const ONLY_DIABLO: u8 = 1 << 4;
    pub const RECREATE_UI: u8 = 1 << 5;
    pub const NEED_DIABLO_MPQ: u8 = 1 << 6;
    pub const NEED_HELLFIRE_MPQ: u8 = 1 << 7;
}
use flags::*;

/// The functions the original registers with `SetValueChangedCallback`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionCallback {
    GameModeChanged,
    SharewareChanged,
    AudioChanged,
    ResizeWindow,
    SetFullscreenMode,
    ResizeWindowAndUpdateResolutionOptions,
    ReinitializeTexture,
    ReinitializeIntegerScale,
    ReinitializeRenderer,
    ShowFpsChanged,
    GrabInputChanged,
    ExperienceBarChanged,
    EnemyHealthBarChanged,
    LanguageCodeChanged,
}

// ---------------------------------------------------------------------------------------------
// diablo.ini access (GetIni, Get/SetIniValue, IniChangedChecker, SaveIni)

#[derive(Default)]
pub struct OptionsIni {
    ini: Option<Ini>,
    /// `IniChanged`
    changed: bool,
}

/// Original: `GetIniPath` (options.cpp).
// @port options.cpp|devilution::GetIniPath() sha=2054ae8ccb9d
fn get_ini_path(paths: &mut Paths) -> String {
    format!("{}diablo.ini", paths.config_path())
}

impl OptionsIni {
    /// Original: `GetIni` (options.cpp): loaded on first use.
    // @port options.cpp|devilution::GetIni() sha=d5e3be722537
    fn get(&mut self, paths: &mut Paths) -> &mut Ini {
        if self.ini.is_none() {
            let path = get_ini_path(paths);
            let text = std::fs::read(&path).map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default();
            self.ini = Some(Ini::parse(&text));
        }
        self.ini.as_mut().unwrap()
    }

    /// Original: `IniChangedChecker::GetValue` (options.cpp).
    // @port options.cpp|devilution::IniChangedChecker::GetValue() sha=5914c8d12228
    fn checker_value(ini: &Ini, section: &str, key: &str) -> Option<String> {
        ini.get_all_values(section, key).map(|values| {
            let mut ret = String::new();
            for v in values {
                ret.push_str(v);
                ret.push('\n');
            }
            ret
        })
    }

    /// Runs `set` on the ini, marking it changed if the value differs (`IniChangedChecker`
    /// constructor/destructor around the setter).
    // @port options.cpp|devilution::IniChangedChecker::IniChangedChecker(const char *sectionName, const char *keyName) sha=8462665995b5
    // @port options.cpp|devilution::IniChangedChecker::~IniChangedChecker() sha=c8235acb7083
    fn checked(&mut self, paths: &mut Paths, section: &str, key: &str, set: impl FnOnce(&mut Ini)) {
        let ini = self.get(paths);
        let old = Self::checker_value(ini, section, key);
        let mut changed = old.is_none();
        set(ini);
        if old != Self::checker_value(ini, section, key) {
            changed = true;
        }
        if changed {
            self.changed = true;
        }
    }
}

/// Everything an entry needs to read/write diablo.ini.
pub struct IniIo<'a> {
    ini: &'a mut OptionsIni,
    paths: &'a mut Paths,
}

impl IniIo<'_> {
    /// Original: `GetIniInt` (options.cpp).
    // @port options.cpp|devilution::GetIniInt(const char *keyname, const char *valuename, int defaultValue) sha=c65b380eb61c
    pub fn get_int(&mut self, section: &str, key: &str, default: i32) -> i32 {
        self.ini.get(self.paths).get_long_value(section, key, default as i64) as i32
    }

    /// Original: `GetIniBool` (options.cpp).
    // @port options.cpp|devilution::GetIniBool(const char *sectionName, const char *keyName, bool defaultValue) sha=402f9a0258df
    pub fn get_bool(&mut self, section: &str, key: &str, default: bool) -> bool {
        self.ini.get(self.paths).get_bool_value(section, key, default)
    }

    /// Original: `GetIniFloat` (options.cpp).
    // @port options.cpp|devilution::GetIniFloat(const char *sectionName, const char *keyName, float defaultValue) sha=641443f52b71
    pub fn get_float(&mut self, section: &str, key: &str, default: f32) -> f32 {
        self.ini.get(self.paths).get_double_value(section, key, default as f64) as f32
    }

    /// Original: `GetIniValue` (options.cpp): the value truncated to `string_size - 1` bytes on
    /// a UTF-8 boundary (`CopyUtf8`), or the default; returns whether the key exists.
    // @port options.cpp|devilution::GetIniValue(string_view sectionName, string_view keyName, char *string, int stringSize, const char *defaultString = "") sha=c7f5364523fe
    pub fn get_value(&mut self, section: &str, key: &str, string_size: usize, default: &str) -> (bool, String) {
        match self.ini.get(self.paths).get_value(section, key) {
            Some(v) => (true, crate::utils::utf8::copy_utf8(v, string_size)),
            None => (false, crate::utils::utf8::copy_utf8(default, string_size)),
        }
    }

    /// Original: `GetIniStringVector` (options.cpp).
    // @port options.cpp|devilution::GetIniStringVector(const char *sectionName, const char *keyName, std::vector<std::string> &stringValues) sha=49cf1a988bd0
    pub fn get_string_vector(&mut self, section: &str, key: &str, values: &mut Vec<String>) -> bool {
        match self.ini.get(self.paths).get_all_values(section, key) {
            Some(v) => {
                values.extend(v.into_iter().map(str::to_string));
                true
            }
            None => false,
        }
    }

    /// Original: `SetIniValue(const char *, const char *, int)` (options.cpp).
    // @port options.cpp|devilution::SetIniValue(const char *keyname, const char *valuename, int value) sha=da5f346fbb66
    pub fn set_int(&mut self, section: &str, key: &str, value: i32) {
        self.ini.checked(self.paths, section, key, |ini| ini.set_long_value(section, key, value as i64, true));
    }

    /// Original: `SetIniValue(const char *, const char *, bool)` (options.cpp).
    // @port options.cpp|devilution::SetIniValue(const char *keyname, const char *valuename, bool value) sha=5282e646fe51
    pub fn set_bool(&mut self, section: &str, key: &str, value: bool) {
        self.ini.checked(self.paths, section, key, |ini| ini.set_long_value(section, key, value as i64, true));
    }

    /// Original: `SetIniValue(const char *, const char *, float)` (options.cpp).
    // @port options.cpp|devilution::SetIniValue(const char *keyname, const char *valuename, float value) sha=4b28680d0db9
    pub fn set_float(&mut self, section: &str, key: &str, value: f32) {
        self.ini.checked(self.paths, section, key, |ini| ini.set_double_value(section, key, value as f64, true));
    }

    /// Original: `SetIniValue(const char *, const char *, const char *)` and the string_view
    /// overload (options.cpp).
    // @port options.cpp|devilution::SetIniValue(const char *sectionName, const char *keyName, const char *value) sha=5f84b79c7b87
    // @port options.cpp|devilution::SetIniValue(string_view sectionName, string_view keyName, string_view value) sha=cc0663d752ee
    pub fn set_str(&mut self, section: &str, key: &str, value: &str) {
        self.ini.checked(self.paths, section, key, |ini| ini.set_value(section, key, value, true));
    }

    /// Original: `SetIniValue(const char *, const char *, const std::vector<std::string> &)`.
    // @port options.cpp|devilution::SetIniValue(const char *keyname, const char *valuename, const std::vector<std::string> &stringValues) sha=6ae8a5b2d137
    pub fn set_string_vector(&mut self, section: &str, key: &str, values: &[String]) {
        self.ini.checked(self.paths, section, key, |ini| {
            let mut first_set = true;
            for v in values {
                ini.set_value(section, key, v, first_set);
                first_set = false;
            }
            if first_set {
                ini.set_value(section, key, "", true);
            }
        });
    }
}

/// Original: `SaveIni` (options.cpp).
// @port options.cpp|devilution::SaveIni() sha=dda39c0fc462
fn save_ini(ini: &mut OptionsIni, paths: &mut Paths) {
    if !ini.changed {
        return;
    }
    let dir = paths.config_path().to_string();
    let _ = std::fs::create_dir_all(&dir);
    let path = get_ini_path(paths);
    let text = ini.get(paths).save();
    if let Err(e) = std::fs::write(&path, text) {
        log::error!("Failed to write ini file to {}: {}", path, e);
    }
    ini.changed = false;
}

// ---------------------------------------------------------------------------------------------
// Entries

pub struct OptionEntryBase {
    pub key: String,
    pub flags: u8,
    name: &'static str,
    description: &'static str,
    callback: Option<OptionCallback>,
}

impl OptionEntryBase {
    /// Original: `OptionEntryBase::OptionEntryBase` (options.h).
    // @port options.h|devilution::OptionEntryBase::OptionEntryBase(string_view key, OptionEntryFlags flags, const char *name, const char *description) sha=679b51c5903d
    fn new(key: &str, flags: u8, name: &'static str, description: &'static str) -> Self {
        OptionEntryBase { key: key.to_string(), flags, name, description, callback: None }
    }

    /// Original: `OptionEntryBase::GetName` (options.cpp).
    // @port options.cpp|devilution::OptionEntryBase::GetName() sha=7e3ac1c89d01
    pub fn get_name(&self) -> String {
        tr(self.name)
    }

    /// Original: `OptionEntryBase::GetDescription` (options.cpp).
    // @port options.cpp|devilution::OptionEntryBase::GetDescription() sha=03214a301d52
    pub fn get_description(&self) -> String {
        tr(self.description)
    }

    /// Original: `OptionEntryBase::GetFlags` (options.cpp).
    // @port options.cpp|devilution::OptionEntryBase::GetFlags() sha=b34a256c6f68
    pub fn get_flags(&self) -> u8 {
        self.flags
    }

    /// Original: `OptionEntryBase::SetValueChangedCallback` (options.cpp).
    // @port options.cpp|devilution::OptionEntryBase::SetValueChangedCallback(std::function<void()> callback) sha=591e390ff071
    fn set_value_changed_callback(&mut self, cb: OptionCallback) {
        self.callback = Some(cb);
    }

    /// Original: `OptionEntryBase::NotifyValueChanged` (options.cpp): returns the callback for
    /// the caller to run with the game context.
    // @port options.cpp|devilution::OptionEntryBase::NotifyValueChanged() sha=c357ece74b58
    #[must_use]
    fn notify_value_changed(&self) -> Option<OptionCallback> {
        self.callback
    }
}

/// `OptionEntryBase`'s virtual interface (+ `OptionEntryListBase`'s list methods).
pub trait OptionEntry {
    fn base(&self) -> &OptionEntryBase;
    fn base_mut(&mut self) -> &mut OptionEntryBase;
    fn get_type(&self) -> OptionEntryType;
    fn get_value_description(&self) -> String;
    fn load_from_ini(&mut self, io: &mut IniIo, category: &str);
    fn save_to_ini(&self, io: &mut IniIo, category: &str);
    fn get_name(&self) -> String {
        self.base().get_name()
    }
    fn get_list_size(&self) -> usize {
        0
    }
    fn get_list_description(&self, _index: usize) -> String {
        String::new()
    }
    fn get_active_list_index(&self) -> usize {
        0
    }
    #[must_use]
    fn set_active_list_index(&mut self, _index: usize) -> Option<OptionCallback> {
        None
    }
    /// `static_cast<OptionEntryBoolean *>(p)->SetValue(!**p)`: only booleans implement it.
    #[must_use]
    fn toggle_boolean(&mut self) -> Option<OptionCallback> {
        None
    }
}

pub struct OptionEntryBoolean {
    pub base: OptionEntryBase,
    default_value: bool,
    value: bool,
}

impl OptionEntryBoolean {
    /// Original: `OptionEntryBoolean::OptionEntryBoolean` (options.h).
    // @port options.h|devilution::OptionEntryBoolean::OptionEntryBoolean(string_view key, OptionEntryFlags flags, const char *name, const char *description, bool defaultValue) sha=8be7291e5023
    fn new(key: &str, flags: u8, name: &'static str, description: &'static str, default: bool) -> Self {
        OptionEntryBoolean { base: OptionEntryBase::new(key, flags, name, description), default_value: default, value: default }
    }

    /// Original: `OptionEntryBoolean::operator*` (options.h).
    // @port options.h|devilution::OptionEntryBoolean::operator*() sha=37e82cf8bcad
    pub fn get(&self) -> bool {
        self.value
    }

    /// Original: `OptionEntryBoolean::SetValue` (options.cpp).
    // @port options.cpp|devilution::OptionEntryBoolean::SetValue(bool value) sha=2e646d60f29a
    #[must_use]
    pub fn set_value(&mut self, value: bool) -> Option<OptionCallback> {
        self.value = value;
        self.base.notify_value_changed()
    }
}

impl OptionEntry for OptionEntryBoolean {
    fn base(&self) -> &OptionEntryBase {
        &self.base
    }
    fn base_mut(&mut self) -> &mut OptionEntryBase {
        &mut self.base
    }
    /// Original: `OptionEntryBoolean::GetType` (options.cpp).
    // @port options.cpp|devilution::OptionEntryBoolean::GetType() sha=fc8f563e93e6
    fn get_type(&self) -> OptionEntryType {
        OptionEntryType::Boolean
    }
    /// Original: `OptionEntryBoolean::GetValueDescription` (options.cpp).
    // @port options.cpp|devilution::OptionEntryBoolean::GetValueDescription() sha=311c55c26ae5
    fn get_value_description(&self) -> String {
        if self.value { tr("ON") } else { tr("OFF") }
    }
    /// Original: `OptionEntryBoolean::LoadFromIni` (options.cpp).
    // @port options.cpp|devilution::OptionEntryBoolean::LoadFromIni(string_view category) sha=12f5067a96ce
    fn load_from_ini(&mut self, io: &mut IniIo, category: &str) {
        self.value = io.get_bool(category, &self.base.key, self.default_value);
    }
    /// Original: `OptionEntryBoolean::SaveToIni` (options.cpp).
    // @port options.cpp|devilution::OptionEntryBoolean::SaveToIni(string_view category) sha=c2b92be08f84
    fn save_to_ini(&self, io: &mut IniIo, category: &str) {
        io.set_bool(category, &self.base.key, self.value);
    }
    fn toggle_boolean(&mut self) -> Option<OptionCallback> {
        self.set_value(!self.value)
    }
}

/// `OptionEntryEnum<T>` over `OptionEntryEnumBase` (values stored as int, like the original).
pub struct OptionEntryEnum {
    pub base: OptionEntryBase,
    default_value: i32,
    value: i32,
    entry_names: Vec<&'static str>,
    entry_values: Vec<i32>,
}

impl OptionEntryEnum {
    /// Original: `OptionEntryEnum::OptionEntryEnum` / `OptionEntryEnumBase::OptionEntryEnumBase` (options.h).
    // @port options.h|devilution::OptionEntryEnum::OptionEntryEnum(string_view key, OptionEntryFlags flags, const char *name, const char *description, T defaultValue, std::initializer_list<std::pair<T, string_view>> entries) sha=202ecf51b315
    // @port options.h|devilution::OptionEntryEnumBase::OptionEntryEnumBase(string_view key, OptionEntryFlags flags, const char *name, const char *description, int defaultValue) sha=d8031dfe90d3
    fn new(key: &str, flags: u8, name: &'static str, description: &'static str, default: i32, entries: &[(i32, &'static str)]) -> Self {
        let mut e = OptionEntryEnum {
            base: OptionEntryBase::new(key, flags, name, description),
            default_value: default,
            value: default,
            entry_names: Vec::new(),
            entry_values: Vec::new(),
        };
        for &(v, n) in entries {
            e.add_entry(v, n);
        }
        e
    }

    /// Original: `OptionEntryEnumBase::AddEntry` (options.cpp).
    // @port options.cpp|devilution::OptionEntryEnumBase::AddEntry(int value, string_view name) sha=01a339133b3d
    fn add_entry(&mut self, value: i32, name: &'static str) {
        self.entry_values.push(value);
        self.entry_names.push(name);
    }

    /// Original: `OptionEntryEnum::operator*` / `OptionEntryEnumBase::GetValueInternal` (options.h).
    // @port options.h|devilution::OptionEntryEnum::operator*() sha=a00d53bb48b1
    // @port options.h|devilution::OptionEntryEnumBase::GetValueInternal() sha=230a83c40b7f
    pub fn get_raw(&self) -> i32 {
        self.value
    }

    /// Original: `OptionEntryEnum::SetValue` / `OptionEntryEnumBase::SetValueInternal`.
    // @port options.h|devilution::OptionEntryEnum::SetValue(T value) sha=f058faa88f49
    // @port options.cpp|devilution::OptionEntryEnumBase::SetValueInternal(int value) sha=28e557b596ff
    #[must_use]
    pub fn set_raw(&mut self, value: i32) -> Option<OptionCallback> {
        self.value = value;
        self.base.notify_value_changed()
    }
}

impl OptionEntry for OptionEntryEnum {
    fn base(&self) -> &OptionEntryBase {
        &self.base
    }
    fn base_mut(&mut self) -> &mut OptionEntryBase {
        &mut self.base
    }
    /// Original: `OptionEntryListBase::GetType` (options.cpp).
    // @port options.cpp|devilution::OptionEntryListBase::GetType() sha=039dba7179db
    fn get_type(&self) -> OptionEntryType {
        OptionEntryType::List
    }
    /// Original: `OptionEntryListBase::GetValueDescription` (options.cpp).
    // @port options.cpp|devilution::OptionEntryListBase::GetValueDescription() sha=cd1b3cbe8624
    fn get_value_description(&self) -> String {
        self.get_list_description(self.get_active_list_index())
    }
    /// Original: `OptionEntryEnumBase::LoadFromIni` (options.cpp).
    // @port options.cpp|devilution::OptionEntryEnumBase::LoadFromIni(string_view category) sha=197505d8ba8f
    fn load_from_ini(&mut self, io: &mut IniIo, category: &str) {
        self.value = io.get_int(category, &self.base.key, self.default_value);
    }
    /// Original: `OptionEntryEnumBase::SaveToIni` (options.cpp).
    // @port options.cpp|devilution::OptionEntryEnumBase::SaveToIni(string_view category) sha=c17a194af5bd
    fn save_to_ini(&self, io: &mut IniIo, category: &str) {
        io.set_int(category, &self.base.key, self.value);
    }
    /// Original: `OptionEntryEnumBase::GetListSize` (options.cpp).
    // @port options.cpp|devilution::OptionEntryEnumBase::GetListSize() sha=7b758a3d6db0
    fn get_list_size(&self) -> usize {
        self.entry_values.len()
    }
    /// Original: `OptionEntryEnumBase::GetListDescription` (options.cpp).
    // @port options.cpp|devilution::OptionEntryEnumBase::GetListDescription(size_t index) sha=94ed851e3adf
    fn get_list_description(&self, index: usize) -> String {
        tr(self.entry_names[index])
    }
    /// Original: `OptionEntryEnumBase::GetActiveListIndex` (options.cpp).
    // @port options.cpp|devilution::OptionEntryEnumBase::GetActiveListIndex() sha=ee61b4bf6a3d
    fn get_active_list_index(&self) -> usize {
        self.entry_values.iter().position(|&v| v == self.value).unwrap_or(0)
    }
    /// Original: `OptionEntryEnumBase::SetActiveListIndex` (options.cpp).
    // @port options.cpp|devilution::OptionEntryEnumBase::SetActiveListIndex(size_t index) sha=b1b6872697b4
    fn set_active_list_index(&mut self, index: usize) -> Option<OptionCallback> {
        self.value = self.entry_values[index];
        self.base.notify_value_changed()
    }
}

/// `OptionEntryInt<T>` over `OptionEntryIntBase`.
pub struct OptionEntryInt {
    pub base: OptionEntryBase,
    default_value: i32,
    value: i32,
    entry_names: RefCell<Vec<String>>,
    entry_values: Vec<i32>,
}

impl OptionEntryInt {
    /// Original: `OptionEntryInt::OptionEntryInt` (both overloads) / `OptionEntryIntBase::OptionEntryIntBase`.
    // @port options.h|devilution::OptionEntryInt::OptionEntryInt(string_view key, OptionEntryFlags flags, const char *name, const char *description, T defaultValue, std::initializer_list<T> entries) sha=edd648572656
    // @port options.h|devilution::OptionEntryInt::OptionEntryInt(string_view key, OptionEntryFlags flags, const char *name, const char *description, T defaultValue) sha=4122af2d1cf7
    // @port options.h|devilution::OptionEntryIntBase::OptionEntryIntBase(string_view key, OptionEntryFlags flags, const char *name, const char *description, int defaultValue) sha=e94e844f1af5
    fn new(key: &str, flags: u8, name: &'static str, description: &'static str, default: i32, entries: &[i32]) -> Self {
        let mut e = OptionEntryInt {
            base: OptionEntryBase::new(key, flags, name, description),
            default_value: default,
            value: default,
            entry_names: RefCell::new(Vec::new()),
            entry_values: Vec::new(),
        };
        if entries.is_empty() {
            e.add_entry(default);
        }
        for &v in entries {
            e.add_entry(v);
        }
        e
    }

    /// Original: `OptionEntryIntBase::AddEntry` (options.cpp).
    // @port options.cpp|devilution::OptionEntryIntBase::AddEntry(int value) sha=8803d96cee47
    fn add_entry(&mut self, value: i32) {
        self.entry_values.push(value);
    }

    /// Original: `OptionEntryInt::operator*` / `OptionEntryIntBase::GetValueInternal`.
    // @port options.h|devilution::OptionEntryInt::operator*() sha=a00d53bb48b1
    // @port options.h|devilution::OptionEntryIntBase::GetValueInternal() sha=230a83c40b7f
    pub fn get(&self) -> i32 {
        self.value
    }

    /// Original: `OptionEntryInt::SetValue` / `OptionEntryIntBase::SetValueInternal`.
    // @port options.h|devilution::OptionEntryInt::SetValue(T value) sha=f058faa88f49
    // @port options.cpp|devilution::OptionEntryIntBase::SetValueInternal(int value) sha=3b33976390e8
    #[must_use]
    pub fn set_value(&mut self, value: i32) -> Option<OptionCallback> {
        self.value = value;
        self.base.notify_value_changed()
    }
}

impl OptionEntry for OptionEntryInt {
    fn base(&self) -> &OptionEntryBase {
        &self.base
    }
    fn base_mut(&mut self) -> &mut OptionEntryBase {
        &mut self.base
    }
    fn get_type(&self) -> OptionEntryType {
        OptionEntryType::List
    }
    fn get_value_description(&self) -> String {
        self.get_list_description(self.get_active_list_index())
    }
    /// Original: `OptionEntryIntBase::LoadFromIni` (options.cpp): an ini value that is not in the
    /// list is added to it (sorted).
    // @port options.cpp|devilution::OptionEntryIntBase::LoadFromIni(string_view category) sha=dd7f0e6f521e
    fn load_from_ini(&mut self, io: &mut IniIo, category: &str) {
        self.value = io.get_int(category, &self.base.key, self.default_value);
        if !self.entry_values.contains(&self.value) {
            self.entry_values.push(self.value);
            self.entry_values.sort();
            self.entry_names.borrow_mut().clear();
        }
    }
    /// Original: `OptionEntryIntBase::SaveToIni` (options.cpp).
    // @port options.cpp|devilution::OptionEntryIntBase::SaveToIni(string_view category) sha=a3d62d06976d
    fn save_to_ini(&self, io: &mut IniIo, category: &str) {
        io.set_int(category, &self.base.key, self.value);
    }
    /// Original: `OptionEntryIntBase::GetListSize` (options.cpp).
    // @port options.cpp|devilution::OptionEntryIntBase::GetListSize() sha=bbe3f8250d60
    fn get_list_size(&self) -> usize {
        self.entry_values.len()
    }
    /// Original: `OptionEntryIntBase::GetListDescription` (options.cpp).
    // @port options.cpp|devilution::OptionEntryIntBase::GetListDescription(size_t index) sha=a1d236dc1d21
    fn get_list_description(&self, index: usize) -> String {
        let mut names = self.entry_names.borrow_mut();
        if names.is_empty() {
            for v in &self.entry_values {
                names.push(v.to_string());
            }
        }
        names[index].clone()
    }
    /// Original: `OptionEntryIntBase::GetActiveListIndex` (options.cpp).
    // @port options.cpp|devilution::OptionEntryIntBase::GetActiveListIndex() sha=b3bcaa932261
    fn get_active_list_index(&self) -> usize {
        self.entry_values.iter().position(|&v| v == self.value).unwrap_or(0)
    }
    /// Original: `OptionEntryIntBase::SetActiveListIndex` (options.cpp).
    // @port options.cpp|devilution::OptionEntryIntBase::SetActiveListIndex(size_t index) sha=e6debc734ae7
    fn set_active_list_index(&mut self, index: usize) -> Option<OptionCallback> {
        self.value = self.entry_values[index];
        self.base.notify_value_changed()
    }
}

/// `OptionEntryLanguageCode`
pub struct OptionEntryLanguageCode {
    pub base: OptionEntryBase,
    /// `szCode[6]`
    sz_code: String,
    languages: RefCell<Vec<(String, String)>>,
    /// `HaveExtraFonts()` at the time the list is built (fonts.mpq loaded).
    pub have_extra_fonts: bool,
}

impl OptionEntryLanguageCode {
    /// Original: `OptionEntryLanguageCode::OptionEntryLanguageCode` (options.cpp).
    // @port options.cpp|devilution::OptionEntryLanguageCode::OptionEntryLanguageCode() sha=fd77dfbea921
    fn new() -> Self {
        OptionEntryLanguageCode {
            base: OptionEntryBase::new(
                "Code",
                CANT_CHANGE_IN_GAME | RECREATE_UI,
                "Language",
                "Define what language to use in game.",
            ),
            sz_code: String::new(),
            languages: RefCell::new(Vec::new()),
            have_extra_fonts: false,
        }
    }

    /// Original: `OptionEntryLanguageCode::operator*` (options.h).
    // @port options.h|devilution::OptionEntryLanguageCode::operator*() sha=03b05c53f216
    pub fn code(&self) -> &str {
        &self.sz_code
    }

    /// Original: `OptionEntryLanguageCode::operator=` (options.h).
    // @port options.h|devilution::OptionEntryLanguageCode::operator=(string_view code) sha=94be639f3b0c
    pub fn set_code(&mut self, code: &str) {
        assert!(code.len() < 6);
        self.sz_code = code.to_string();
    }

    /// Original: `OptionEntryLanguageCode::CheckLanguagesAreInitialized` (options.cpp).
    // @port options.cpp|devilution::OptionEntryLanguageCode::CheckLanguagesAreInitialized() sha=d0262b8de64c
    fn check_languages_are_initialized(&self) {
        let mut languages = self.languages.borrow_mut();
        if !languages.is_empty() {
            return;
        }
        let mut add = |c: &str, n: &str| languages.push((c.to_string(), n.to_string()));
        add("bg", "Български");
        add("cs", "Čeština");
        add("da", "Dansk");
        add("de", "Deutsch");
        add("el", "Ελληνικά");
        add("en", "English");
        add("es", "Español");
        add("fr", "Français");
        add("hr", "Hrvatski");
        add("hu", "Magyar");
        add("it", "Italiano");
        if self.have_extra_fonts {
            add("ja", "日本語");
            add("ko", "한국어");
        }
        add("pl", "Polski");
        add("pt_BR", "Português do Brasil");
        add("ro", "Română");
        add("ru", "Русский");
        add("sv", "Svenska");
        add("tr", "Türkçe");
        add("uk", "Українська");
        if self.have_extra_fonts {
            add("zh_CN", "汉语");
            add("zh_TW", "漢語");
        }
        if !languages.iter().any(|(c, _)| *c == self.sz_code) {
            languages.push((self.sz_code.clone(), self.sz_code.clone()));
        }
    }
}

impl OptionEntry for OptionEntryLanguageCode {
    fn base(&self) -> &OptionEntryBase {
        &self.base
    }
    fn base_mut(&mut self) -> &mut OptionEntryBase {
        &mut self.base
    }
    fn get_type(&self) -> OptionEntryType {
        OptionEntryType::List
    }
    fn get_value_description(&self) -> String {
        self.get_list_description(self.get_active_list_index())
    }
    /// Loading needs `HasTranslation` (asset lookup), so it is done by `load_language_code`;
    /// this trait method only reads the raw ini value.
    fn load_from_ini(&mut self, io: &mut IniIo, category: &str) {
        let (_, code) = io.get_value(category, &self.base.key, 6, "");
        self.sz_code = code;
    }
    /// Original: `OptionEntryLanguageCode::SaveToIni` (options.cpp).
    // @port options.cpp|devilution::OptionEntryLanguageCode::SaveToIni(string_view category) sha=0422eb380018
    fn save_to_ini(&self, io: &mut IniIo, category: &str) {
        io.set_str(category, &self.base.key, &self.sz_code);
    }
    /// Original: `OptionEntryLanguageCode::GetListSize` (options.cpp).
    // @port options.cpp|devilution::OptionEntryLanguageCode::GetListSize() sha=be323166e709
    fn get_list_size(&self) -> usize {
        self.check_languages_are_initialized();
        self.languages.borrow().len()
    }
    /// Original: `OptionEntryLanguageCode::GetListDescription` (options.cpp).
    // @port options.cpp|devilution::OptionEntryLanguageCode::GetListDescription(size_t index) sha=9bafc39d4804
    fn get_list_description(&self, index: usize) -> String {
        self.check_languages_are_initialized();
        self.languages.borrow()[index].1.clone()
    }
    /// Original: `OptionEntryLanguageCode::GetActiveListIndex` (options.cpp).
    // @port options.cpp|devilution::OptionEntryLanguageCode::GetActiveListIndex() sha=e83a2a8680a4
    fn get_active_list_index(&self) -> usize {
        self.check_languages_are_initialized();
        self.languages.borrow().iter().position(|(c, _)| *c == self.sz_code).unwrap_or(0)
    }
    /// Original: `OptionEntryLanguageCode::SetActiveListIndex` (options.cpp).
    // @port options.cpp|devilution::OptionEntryLanguageCode::SetActiveListIndex(size_t index) sha=fdb14df1b95f
    fn set_active_list_index(&mut self, index: usize) -> Option<OptionCallback> {
        self.sz_code = crate::utils::utf8::copy_utf8(&self.languages.borrow()[index].0, 6);
        self.base.notify_value_changed()
    }
}

/// Original: `OptionEntryLanguageCode::LoadFromIni` (options.cpp).
// @port options.cpp|devilution::OptionEntryLanguageCode::LoadFromIni(string_view category) sha=eb253ce15220
fn load_language_code(ctx: &mut Ctx, category: &str) {
    let key = ctx.options.language.code.base.key.clone();
    let (found, code) = IniIo { ini: &mut ctx.options_ini, paths: &mut ctx.paths }.get_value(category, &key, 6, "");
    ctx.options.language.code.sz_code = code.clone();
    if found && crate::utils::language::has_translation(ctx, &code) {
        return;
    }
    let mut locales = crate::platform::locale::get_locales();
    for l in locales.iter_mut() {
        if l == "en_US" {
            *l = "en".to_string();
        }
    }
    // Insert non-regional codes after the last regional variation of each language.
    let mut i = locales.len();
    while i > 0 {
        i -= 1;
        if let Some(sep) = locales[i].find('_') {
            let neutral = locales[i][..sep].to_string();
            if !locales[i + 1..].contains(&neutral) {
                locales.insert(i + 1, neutral);
            }
        }
    }
    log::verbose!("Found user preferred locales: {}", locales.join(", "));
    for locale in &locales {
        log::verbose!("Trying to load translation: {}", locale);
        if crate::utils::language::has_translation(ctx, locale) {
            log::verbose!("Best match locale: {}", locale);
            ctx.options.language.code.sz_code = crate::utils::utf8::copy_utf8(locale, 6);
            return;
        }
    }
    log::verbose!("No suitable translation found");
    ctx.options.language.code.sz_code = "en".to_string();
}

/// `OptionEntryResolution`
pub struct OptionEntryResolution {
    pub base: OptionEntryBase,
    size: (i32, i32),
    resolutions: RefCell<Vec<((i32, i32), String)>>,
    /// Display information the list is built from: (display modes, desktop mode, dpi scale,
    /// upscale, fitToScreen), captured when the list is (re)built.
    pub display: RefCell<Option<ResolutionInputs>>,
}

#[derive(Clone, Debug)]
pub struct ResolutionInputs {
    pub modes: Vec<(i32, i32)>,
    pub desktop: (i32, i32),
    pub scale_factor: f32,
    pub upscale: bool,
    pub fit_to_screen: bool,
}

impl OptionEntryResolution {
    /// Original: `OptionEntryResolution::OptionEntryResolution` (options.cpp).
    // @port options.cpp|devilution::OptionEntryResolution::OptionEntryResolution() sha=53ab27ecb201
    fn new() -> Self {
        OptionEntryResolution {
            base: OptionEntryBase::new(
                "",
                CANT_CHANGE_IN_GAME | RECREATE_UI,
                "Resolution",
                "Affect the game's internal resolution and determine your view area. Note: This can differ from screen resolution, when Upscaling, Integer Scaling or Fit to Screen is used.",
            ),
            size: (0, 0),
            resolutions: RefCell::new(Vec::new()),
            display: RefCell::new(None),
        }
    }

    /// Original: `OptionEntryResolution::operator*` (options.h).
    // @port options.h|devilution::OptionEntryResolution::operator*() sha=3378f849871e
    pub fn get(&self) -> (i32, i32) {
        self.size
    }

    /// Original: `OptionEntryResolution::InvalidateList` (options.cpp).
    // @port options.cpp|devilution::OptionEntryResolution::InvalidateList() sha=f44553eeb0b3
    pub fn invalidate_list(&mut self) {
        self.resolutions.borrow_mut().clear();
    }

    /// Original: `OptionEntryResolution::CheckResolutionsAreInitialized` (options.cpp).
    // @port options.cpp|devilution::OptionEntryResolution::CheckResolutionsAreInitialized() sha=b30e0b7f45bf
    fn check_resolutions_are_initialized(&self) {
        let mut resolutions = self.resolutions.borrow_mut();
        if !resolutions.is_empty() {
            return;
        }
        let d = self.display.borrow().clone().unwrap_or_else(crate::platform::display::resolution_inputs);
        let mut sizes: Vec<(i32, i32)> = Vec::new();
        for &(mut w, mut h) in &d.modes {
            if w < h {
                std::mem::swap(&mut w, &mut h);
            }
            sizes.push(((w as f32 * d.scale_factor) as i32, (h as f32 * d.scale_factor) as i32));
        }
        let supports_any_resolution = d.upscale;
        if supports_any_resolution && sizes.len() == 1 {
            let (width, height) = sizes[0];
            for common_height in [480, 540, 720, 960, 1080, 1440, 2160] {
                if common_height > height {
                    break;
                }
                sizes.push((common_height * 4 / 3, common_height));
                if common_height * width % height == 0 {
                    sizes.push((common_height * width / height, common_height));
                }
            }
        }
        sizes.push(self.size);
        sizes.push((DEFAULT_WIDTH, DEFAULT_HEIGHT));
        if supports_any_resolution {
            sizes.push((640, 480));
        }
        if d.fit_to_screen {
            for s in sizes.iter_mut() {
                if s.1 == self.size.1 {
                    s.0 = self.size.0;
                } else {
                    s.0 = s.1 * d.desktop.0 / d.desktop.1;
                }
            }
        }
        sizes.sort_by(|x, y| if x.0 == y.0 { y.1.cmp(&x.1) } else { y.0.cmp(&x.0) });
        sizes.dedup();
        for s in sizes {
            if d.fit_to_screen {
                resolutions.push((s, format!("{}p", s.1)));
            } else {
                resolutions.push((s, format!("{}x{}", s.0, s.1)));
            }
        }
    }
}

impl OptionEntry for OptionEntryResolution {
    fn base(&self) -> &OptionEntryBase {
        &self.base
    }
    fn base_mut(&mut self) -> &mut OptionEntryBase {
        &mut self.base
    }
    fn get_type(&self) -> OptionEntryType {
        OptionEntryType::List
    }
    fn get_value_description(&self) -> String {
        self.get_list_description(self.get_active_list_index())
    }
    /// Original: `OptionEntryResolution::LoadFromIni` (options.cpp).
    // @port options.cpp|devilution::OptionEntryResolution::LoadFromIni(string_view category) sha=4cc659ebe2a2
    fn load_from_ini(&mut self, io: &mut IniIo, category: &str) {
        self.size = (io.get_int(category, "Width", DEFAULT_WIDTH), io.get_int(category, "Height", DEFAULT_HEIGHT));
    }
    /// Original: `OptionEntryResolution::SaveToIni` (options.cpp).
    // @port options.cpp|devilution::OptionEntryResolution::SaveToIni(string_view category) sha=6908fb0a8524
    fn save_to_ini(&self, io: &mut IniIo, category: &str) {
        io.set_int(category, "Width", self.size.0);
        io.set_int(category, "Height", self.size.1);
    }
    /// Original: `OptionEntryResolution::GetListSize` (options.cpp).
    // @port options.cpp|devilution::OptionEntryResolution::GetListSize() sha=8a972f277e52
    fn get_list_size(&self) -> usize {
        self.check_resolutions_are_initialized();
        self.resolutions.borrow().len()
    }
    /// Original: `OptionEntryResolution::GetListDescription` (options.cpp).
    // @port options.cpp|devilution::OptionEntryResolution::GetListDescription(size_t index) sha=48e0c35603d4
    fn get_list_description(&self, index: usize) -> String {
        self.check_resolutions_are_initialized();
        self.resolutions.borrow()[index].1.clone()
    }
    /// Original: `OptionEntryResolution::GetActiveListIndex` (options.cpp).
    // @port options.cpp|devilution::OptionEntryResolution::GetActiveListIndex() sha=16245ab1ec0e
    fn get_active_list_index(&self) -> usize {
        self.check_resolutions_are_initialized();
        self.resolutions.borrow().iter().position(|(s, _)| *s == self.size).unwrap_or(0)
    }
    /// Original: `OptionEntryResolution::SetActiveListIndex` (options.cpp).
    // @port options.cpp|devilution::OptionEntryResolution::SetActiveListIndex(size_t index) sha=91a35e80b67d
    fn set_active_list_index(&mut self, index: usize) -> Option<OptionCallback> {
        self.size = self.resolutions.borrow()[index].0;
        self.base.notify_value_changed()
    }
}

/// Original: `ResamplerToString` (options.cpp).
// @port options.cpp|devilution::ResamplerToString(Resampler resampler) sha=bcba57f5c9c7
pub fn resampler_to_string(r: Resampler) -> &'static str {
    match r {
        Resampler::Speex => "Speex",
        Resampler::Sdl => "SDL",
    }
}

/// Original: `ResamplerFromString` (options.cpp).
// @port options.cpp|devilution::ResamplerFromString(string_view resampler) sha=1bc61aafadd7
pub fn resampler_from_string(s: &str) -> Option<Resampler> {
    match s {
        "Speex" => Some(Resampler::Speex),
        "SDL" => Some(Resampler::Sdl),
        _ => None,
    }
}

/// `OptionEntryResampler`
pub struct OptionEntryResampler {
    pub base: OptionEntryBase,
    resampler: Resampler,
}

impl OptionEntryResampler {
    /// Original: `OptionEntryResampler::OptionEntryResampler` (options.cpp).
    // @port options.cpp|devilution::OptionEntryResampler::OptionEntryResampler() sha=8ddd0c8d4bb7
    fn new() -> Self {
        let recreate = if NUM_RESAMPLERS == 2 { RECREATE_UI } else { NONE };
        OptionEntryResampler {
            base: OptionEntryBase::new("Resampler", CANT_CHANGE_IN_GAME | recreate, "Resampler", "Audio resampler"),
            resampler: DEFAULT_RESAMPLER,
        }
    }

    /// Original: `OptionEntryResampler::operator*` (options.h).
    // @port options.h|devilution::OptionEntryResampler::operator*() sha=d3a085a35710
    pub fn get(&self) -> Resampler {
        self.resampler
    }
}

/// Original: `OptionEntryResampler::UpdateDependentOptions` (options.cpp): the quality option
/// is only visible for Speex.
// @port options.cpp|devilution::OptionEntryResampler::UpdateDependentOptions() sha=4c8d84b7efd4
fn update_dependent_options(audio: &mut AudioOptions) {
    if audio.resampler.resampler == Resampler::Speex {
        audio.resampling_quality.base.flags &= !INVISIBLE;
    } else {
        audio.resampling_quality.base.flags |= INVISIBLE;
    }
}

impl OptionEntry for OptionEntryResampler {
    fn base(&self) -> &OptionEntryBase {
        &self.base
    }
    fn base_mut(&mut self) -> &mut OptionEntryBase {
        &mut self.base
    }
    fn get_type(&self) -> OptionEntryType {
        OptionEntryType::List
    }
    fn get_value_description(&self) -> String {
        self.get_list_description(self.get_active_list_index())
    }
    /// Original: `OptionEntryResampler::LoadFromIni` (options.cpp). The dependent-option update
    /// is done by the caller (`AudioOptions::load`), which owns both entries.
    // @port options.cpp|devilution::OptionEntryResampler::LoadFromIni(string_view category) sha=680b86382d8d
    fn load_from_ini(&mut self, io: &mut IniIo, category: &str) {
        let (found, s) = io.get_value(category, &self.base.key, 32, "");
        self.resampler = if found { resampler_from_string(&s).unwrap_or(DEFAULT_RESAMPLER) } else { DEFAULT_RESAMPLER };
    }
    /// Original: `OptionEntryResampler::SaveToIni` (options.cpp).
    // @port options.cpp|devilution::OptionEntryResampler::SaveToIni(string_view category) sha=3c98fa3a1714
    fn save_to_ini(&self, io: &mut IniIo, category: &str) {
        io.set_str(category, &self.base.key, resampler_to_string(self.resampler));
    }
    /// Original: `OptionEntryResampler::GetListSize` (options.cpp).
    // @port options.cpp|devilution::OptionEntryResampler::GetListSize() sha=8c5b27c479eb
    fn get_list_size(&self) -> usize {
        NUM_RESAMPLERS
    }
    /// Original: `OptionEntryResampler::GetListDescription` (options.cpp).
    // @port options.cpp|devilution::OptionEntryResampler::GetListDescription(size_t index) sha=532d78d049c5
    fn get_list_description(&self, index: usize) -> String {
        resampler_to_string(if index == 0 { Resampler::Speex } else { Resampler::Sdl }).to_string()
    }
    /// Original: `OptionEntryResampler::GetActiveListIndex` (options.cpp).
    // @port options.cpp|devilution::OptionEntryResampler::GetActiveListIndex() sha=e08f57fb9e0c
    fn get_active_list_index(&self) -> usize {
        self.resampler as usize
    }
    /// Original: `OptionEntryResampler::SetActiveListIndex` (options.cpp). The dependent-option
    /// update is applied by `AudioOptions::set_resampler_index`.
    // @port options.cpp|devilution::OptionEntryResampler::SetActiveListIndex(size_t index) sha=7b8663b11383
    fn set_active_list_index(&mut self, index: usize) -> Option<OptionCallback> {
        self.resampler = if index == 0 { Resampler::Speex } else { Resampler::Sdl };
        self.base.notify_value_changed()
    }
}

/// `OptionEntryAudioDevice`. The port has no SDL device enumeration; the list holds the
/// system default device only (known difference until audio output is ported).
pub struct OptionEntryAudioDevice {
    pub base: OptionEntryBase,
    device_name: String,
}

impl OptionEntryAudioDevice {
    /// Original: `OptionEntryAudioDevice::OptionEntryAudioDevice` (options.cpp).
    // @port options.cpp|devilution::OptionEntryAudioDevice::OptionEntryAudioDevice() sha=38445e02357d
    fn new() -> Self {
        OptionEntryAudioDevice {
            base: OptionEntryBase::new("Device", CANT_CHANGE_IN_GAME, "Device", "Audio device"),
            device_name: String::new(),
        }
    }

    /// Original: `OptionEntryAudioDevice::GetDeviceName` (options.cpp).
    // @port options.cpp|devilution::OptionEntryAudioDevice::GetDeviceName(size_t index) sha=496c329874ba
    fn get_device_name(&self, index: usize) -> String {
        crate::platform::audio_device_name(index)
    }

    /// Original: `OptionEntryAudioDevice::operator*` (options.h).
    // @port options.h|devilution::OptionEntryAudioDevice::operator*() sha=623de61c5f83
    pub fn get(&self) -> String {
        for i in 0..self.get_list_size() {
            if self.get_device_name(i) == self.device_name {
                return self.device_name.clone();
            }
        }
        String::new()
    }
}

impl OptionEntry for OptionEntryAudioDevice {
    fn base(&self) -> &OptionEntryBase {
        &self.base
    }
    fn base_mut(&mut self) -> &mut OptionEntryBase {
        &mut self.base
    }
    fn get_type(&self) -> OptionEntryType {
        OptionEntryType::List
    }
    fn get_value_description(&self) -> String {
        self.get_list_description(self.get_active_list_index())
    }
    /// Original: `OptionEntryAudioDevice::LoadFromIni` (options.cpp).
    // @port options.cpp|devilution::OptionEntryAudioDevice::LoadFromIni(string_view category) sha=3666be52d6f2
    fn load_from_ini(&mut self, io: &mut IniIo, category: &str) {
        self.device_name = io.get_value(category, &self.base.key, 100, "").1;
    }
    /// Original: `OptionEntryAudioDevice::SaveToIni` (options.cpp).
    // @port options.cpp|devilution::OptionEntryAudioDevice::SaveToIni(string_view category) sha=34a9df4c8037
    fn save_to_ini(&self, io: &mut IniIo, category: &str) {
        io.set_str(category, &self.base.key, &self.device_name);
    }
    /// Original: `OptionEntryAudioDevice::GetListSize` (options.cpp).
    // @port options.cpp|devilution::OptionEntryAudioDevice::GetListSize() sha=0d12cc8e0212
    fn get_list_size(&self) -> usize {
        crate::platform::num_audio_devices() + 1
    }
    /// Original: `OptionEntryAudioDevice::GetListDescription` (options.cpp). The original
    /// shortens names wider than 500px in GameFont24; device names here are not shortened
    /// until the text renderer is ported (pending).
    // @port options.cpp|devilution::OptionEntryAudioDevice::GetListDescription(size_t index) sha=cffd189d15d9
    fn get_list_description(&self, index: usize) -> String {
        let name = self.get_device_name(index);
        if name.is_empty() { "System Default".to_string() } else { name }
    }
    /// Original: `OptionEntryAudioDevice::GetActiveListIndex` (options.cpp).
    // @port options.cpp|devilution::OptionEntryAudioDevice::GetActiveListIndex() sha=2e884b527d0e
    fn get_active_list_index(&self) -> usize {
        (0..self.get_list_size()).find(|&i| self.get_device_name(i) == self.device_name).unwrap_or(0)
    }
    /// Original: `OptionEntryAudioDevice::SetActiveListIndex` (options.cpp).
    // @port options.cpp|devilution::OptionEntryAudioDevice::SetActiveListIndex(size_t index) sha=54baa08ffcdd
    fn set_active_list_index(&mut self, index: usize) -> Option<OptionCallback> {
        self.device_name = self.get_device_name(index);
        self.base.notify_value_changed()
    }
}

// ---------------------------------------------------------------------------------------------
// Categories

pub struct OptionCategoryBase {
    key: &'static str,
    name: &'static str,
    description: &'static str,
}

impl OptionCategoryBase {
    /// Original: `OptionCategoryBase::OptionCategoryBase` (options.h).
    // @port options.h|devilution::OptionCategoryBase::OptionCategoryBase(string_view key, const char *name, const char *description) sha=127df16eebbc
    const fn new(key: &'static str, name: &'static str, description: &'static str) -> Self {
        OptionCategoryBase { key, name, description }
    }
    /// Original: `OptionCategoryBase::GetKey` (options.cpp).
    // @port options.cpp|devilution::OptionCategoryBase::GetKey() sha=6c01a91c51fd
    pub fn get_key(&self) -> &'static str {
        self.key
    }
    /// Original: `OptionCategoryBase::GetName` (options.cpp).
    // @port options.cpp|devilution::OptionCategoryBase::GetName() sha=c6b8930da6c3
    pub fn get_name(&self) -> String {
        tr(self.name)
    }
    /// Original: `OptionCategoryBase::GetDescription` (options.cpp).
    // @port options.cpp|devilution::OptionCategoryBase::GetDescription() sha=d8ee3194b589
    pub fn get_description(&self) -> String {
        tr(self.description)
    }
}

pub struct StartUpOptions {
    pub category: OptionCategoryBase,
    pub game_mode: OptionEntryEnum,
    pub shareware: OptionEntryBoolean,
    pub diablo_intro: OptionEntryEnum,
    pub hellfire_intro: OptionEntryEnum,
    pub splash: OptionEntryEnum,
}

impl StartUpOptions {
    /// Original: `StartUpOptions::StartUpOptions` (options.cpp).
    // @port options.cpp|devilution::StartUpOptions::StartUpOptions() sha=936a59eecd15
    fn new() -> Self {
        let mut o = StartUpOptions {
            category: OptionCategoryBase::new("StartUp", "Start Up", "Start Up Settings"),
            game_mode: OptionEntryEnum::new(
                "Game",
                NEED_HELLFIRE_MPQ | RECREATE_UI,
                "Game Mode",
                "Play Diablo or Hellfire.",
                StartUpGameMode::Ask as i32,
                // Ask is missing, cause we want to hide it from UI-Settings.
                &[(StartUpGameMode::Diablo as i32, "Diablo"), (StartUpGameMode::Hellfire as i32, "Hellfire")],
            ),
            shareware: OptionEntryBoolean::new(
                "Shareware",
                NEED_DIABLO_MPQ | RECREATE_UI,
                "Restrict to Shareware",
                "Makes the game compatible with the demo. Enables multiplayer with friends who don't own a full copy of Diablo.",
                false,
            ),
            diablo_intro: OptionEntryEnum::new(
                "Diablo Intro",
                ONLY_DIABLO,
                "Intro",
                "Shown Intro cinematic.",
                StartUpIntro::Once as i32,
                &[(StartUpIntro::Off as i32, "OFF"), (StartUpIntro::On as i32, "ON")],
            ),
            hellfire_intro: OptionEntryEnum::new(
                "Hellfire Intro",
                ONLY_HELLFIRE,
                "Intro",
                "Shown Intro cinematic.",
                StartUpIntro::Once as i32,
                &[(StartUpIntro::Off as i32, "OFF"), (StartUpIntro::On as i32, "ON")],
            ),
            splash: OptionEntryEnum::new(
                "Splash",
                NONE,
                "Splash",
                "Shown splash screen.",
                StartUpSplash::LogoAndTitleDialog as i32,
                &[
                    (StartUpSplash::LogoAndTitleDialog as i32, "Logo and Title Screen"),
                    (StartUpSplash::TitleDialog as i32, "Title Screen"),
                    (StartUpSplash::None as i32, "None"),
                ],
            ),
        };
        o.game_mode.base.set_value_changed_callback(OptionCallback::GameModeChanged);
        o.shareware.base.set_value_changed_callback(OptionCallback::SharewareChanged);
        o
    }

    /// Original: `StartUpOptions::GetEntries` (options.cpp).
    // @port options.cpp|devilution::StartUpOptions::GetEntries() sha=b60630d79689
    pub fn get_entries(&mut self) -> Vec<&mut dyn OptionEntry> {
        vec![&mut self.game_mode, &mut self.shareware, &mut self.diablo_intro, &mut self.hellfire_intro, &mut self.splash]
    }

    pub fn game_mode(&self) -> StartUpGameMode {
        match self.game_mode.get_raw() {
            1 => StartUpGameMode::Hellfire,
            2 => StartUpGameMode::Diablo,
            _ => StartUpGameMode::Ask,
        }
    }

    pub fn splash(&self) -> StartUpSplash {
        match self.splash.get_raw() {
            0 => StartUpSplash::None,
            1 => StartUpSplash::TitleDialog,
            _ => StartUpSplash::LogoAndTitleDialog,
        }
    }
}

pub fn intro_value(e: &OptionEntryEnum) -> StartUpIntro {
    match e.get_raw() {
        0 => StartUpIntro::Off,
        2 => StartUpIntro::On,
        _ => StartUpIntro::Once,
    }
}

pub struct LastHeroOptions {
    pub category: OptionCategoryBase,
    pub last_single_player_hero: OptionEntryInt,
    pub last_multiplayer_hero: OptionEntryInt,
    /// Hellfire only: `szItem[sizeof(ItemPack) * 2 + 1]`, the Cornerstone of the World item.
    pub sz_item: String,
}

impl LastHeroOptions {
    /// Original: `DiabloOptions::DiabloOptions` / `HellfireOptions::HellfireOptions` (options.cpp).
    // @port options.cpp|devilution::DiabloOptions::DiabloOptions() sha=b0443ec27c82
    // @port options.cpp|devilution::HellfireOptions::HellfireOptions() sha=fcad730a4685
    fn new(hellfire: bool) -> Self {
        let only = if hellfire { ONLY_HELLFIRE } else { ONLY_DIABLO };
        LastHeroOptions {
            category: if hellfire {
                OptionCategoryBase::new("Hellfire", "Hellfire", "Hellfire specific Settings")
            } else {
                OptionCategoryBase::new("Diablo", "Diablo", "Diablo specific Settings")
            },
            last_single_player_hero: OptionEntryInt::new(
                "LastSinglePlayerHero",
                INVISIBLE | only,
                "Sample Rate",
                "Remembers what singleplayer hero/save was last used.",
                0,
                &[],
            ),
            last_multiplayer_hero: OptionEntryInt::new(
                "LastMultiplayerHero",
                INVISIBLE | only,
                "Sample Rate",
                "Remembers what multiplayer hero/save was last used.",
                0,
                &[],
            ),
            sz_item: String::new(),
        }
    }

    /// Original: `DiabloOptions::GetEntries` / `HellfireOptions::GetEntries` (options.cpp).
    // @port options.cpp|devilution::DiabloOptions::GetEntries() sha=a9074318303f
    // @port options.cpp|devilution::HellfireOptions::GetEntries() sha=150e43dee7ae
    pub fn get_entries(&mut self) -> Vec<&mut dyn OptionEntry> {
        vec![&mut self.last_single_player_hero, &mut self.last_multiplayer_hero]
    }
}

pub struct AudioOptions {
    pub category: OptionCategoryBase,
    pub sound_volume: OptionEntryInt,
    pub music_volume: OptionEntryInt,
    pub walking_sound: OptionEntryBoolean,
    pub auto_equip_sound: OptionEntryBoolean,
    pub item_pickup_sound: OptionEntryBoolean,
    pub sample_rate: OptionEntryInt,
    pub channels: OptionEntryInt,
    pub buffer_size: OptionEntryInt,
    pub resampler: OptionEntryResampler,
    pub resampling_quality: OptionEntryInt,
    pub device: OptionEntryAudioDevice,
}

impl AudioOptions {
    /// Original: `AudioOptions::AudioOptions` (options.cpp).
    // @port options.cpp|devilution::AudioOptions::AudioOptions() sha=e8e9b0a9b752
    fn new() -> Self {
        let mut o = AudioOptions {
            category: OptionCategoryBase::new("Audio", "Audio", "Audio Settings"),
            sound_volume: OptionEntryInt::new("Sound Volume", INVISIBLE, "Sound Volume", "Movie and SFX volume.", VOLUME_MAX, &[]),
            music_volume: OptionEntryInt::new("Music Volume", INVISIBLE, "Music Volume", "Music Volume.", VOLUME_MAX, &[]),
            walking_sound: OptionEntryBoolean::new("Walking Sound", NONE, "Walking Sound", "Player emits sound when walking.", true),
            auto_equip_sound: OptionEntryBoolean::new(
                "Auto Equip Sound",
                NONE,
                "Auto Equip Sound",
                "Automatically equipping items on pickup emits the equipment sound.",
                false,
            ),
            item_pickup_sound: OptionEntryBoolean::new(
                "Item Pickup Sound",
                NONE,
                "Item Pickup Sound",
                "Picking up items emits the items pickup sound.",
                false,
            ),
            sample_rate: OptionEntryInt::new(
                "Sample Rate",
                CANT_CHANGE_IN_GAME,
                "Sample Rate",
                "Output sample rate (Hz).",
                DEFAULT_AUDIO_SAMPLE_RATE,
                &[22050, 44100, 48000],
            ),
            channels: OptionEntryInt::new("Channels", CANT_CHANGE_IN_GAME, "Channels", "Number of output channels.", DEFAULT_AUDIO_CHANNELS, &[1, 2]),
            buffer_size: OptionEntryInt::new(
                "Buffer Size",
                CANT_CHANGE_IN_GAME,
                "Buffer Size",
                "Buffer size (number of frames per channel).",
                DEFAULT_AUDIO_BUFFER_SIZE,
                &[1024, 2048, 5120],
            ),
            resampler: OptionEntryResampler::new(),
            resampling_quality: OptionEntryInt::new(
                "Resampling Quality",
                CANT_CHANGE_IN_GAME,
                "Resampling Quality",
                "Quality of the resampler, from 0 (lowest) to 10 (highest).",
                DEFAULT_AUDIO_RESAMPLING_QUALITY,
                &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            ),
            device: OptionEntryAudioDevice::new(),
        };
        for b in [&mut o.sample_rate.base, &mut o.channels.base, &mut o.buffer_size.base, &mut o.resampling_quality.base] {
            b.set_value_changed_callback(OptionCallback::AudioChanged);
        }
        o.resampler.base.set_value_changed_callback(OptionCallback::AudioChanged);
        o.device.base.set_value_changed_callback(OptionCallback::AudioChanged);
        o
    }

    /// Original: `AudioOptions::GetEntries` (options.cpp).
    // @port options.cpp|devilution::AudioOptions::GetEntries() sha=2f64b64db305
    pub fn get_entries(&mut self) -> Vec<&mut dyn OptionEntry> {
        vec![
            &mut self.sound_volume,
            &mut self.music_volume,
            &mut self.walking_sound,
            &mut self.auto_equip_sound,
            &mut self.item_pickup_sound,
            &mut self.sample_rate,
            &mut self.channels,
            &mut self.buffer_size,
            &mut self.resampler,
            &mut self.resampling_quality,
            &mut self.device,
        ]
    }
}

pub struct GraphicsOptions {
    pub category: OptionCategoryBase,
    pub resolution: OptionEntryResolution,
    pub fullscreen: OptionEntryBoolean,
    pub fit_to_screen: OptionEntryBoolean,
    pub upscale: OptionEntryBoolean,
    pub scale_quality: OptionEntryEnum,
    pub integer_scaling: OptionEntryBoolean,
    pub v_sync: OptionEntryBoolean,
    pub gamma_correction: OptionEntryInt,
    pub zoom: OptionEntryBoolean,
    pub color_cycling: OptionEntryBoolean,
    pub alternate_nest_art: OptionEntryBoolean,
    pub hardware_cursor: OptionEntryBoolean,
    pub hardware_cursor_for_items: OptionEntryBoolean,
    pub hardware_cursor_max_size: OptionEntryInt,
    pub limit_fps: OptionEntryBoolean,
    pub show_fps: OptionEntryBoolean,
}

/// Original: `HardwareCursorSupported` (options.cpp): SDL >= 2.0.12 (true for the shipped SDL).
// @port options.cpp|devilution::HardwareCursorSupported() sha=946f2844a611
pub fn hardware_cursor_supported() -> bool {
    true
}

/// Original: `HardwareCursorDefault` (options.cpp).
// @port options.cpp|devilution::HardwareCursorDefault() sha=9472d6393ee9
fn hardware_cursor_default() -> bool {
    hardware_cursor_supported()
}

impl GraphicsOptions {
    /// Original: `GraphicsOptions::GraphicsOptions` (options.cpp).
    // @port options.cpp|devilution::GraphicsOptions::GraphicsOptions() sha=c7dbf5739a10
    fn new() -> Self {
        let hw = if hardware_cursor_supported() { NONE } else { INVISIBLE };
        let mut o = GraphicsOptions {
            category: OptionCategoryBase::new("Graphics", "Graphics", "Graphics Settings"),
            resolution: OptionEntryResolution::new(),
            fullscreen: OptionEntryBoolean::new(
                "Fullscreen",
                CANT_CHANGE_IN_GAME | RECREATE_UI,
                "Fullscreen",
                "Display the game in windowed or fullscreen mode.",
                true,
            ),
            fit_to_screen: OptionEntryBoolean::new(
                "Fit to Screen",
                CANT_CHANGE_IN_GAME | RECREATE_UI,
                "Fit to Screen",
                "Automatically adjust the game window to your current desktop screen aspect ratio and resolution.",
                true,
            ),
            upscale: OptionEntryBoolean::new(
                "Upscale",
                CANT_CHANGE_IN_GAME | RECREATE_UI,
                "Upscale",
                "Enables image scaling from the game resolution to your monitor resolution. Prevents changing the monitor resolution and allows window resizing.",
                true,
            ),
            scale_quality: OptionEntryEnum::new(
                "Scaling Quality",
                NONE,
                "Scaling Quality",
                "Enables optional filters to the output image when upscaling.",
                ScalingQuality::AnisotropicFiltering as i32,
                &[
                    (ScalingQuality::NearestPixel as i32, "Nearest Pixel"),
                    (ScalingQuality::BilinearFiltering as i32, "Bilinear"),
                    (ScalingQuality::AnisotropicFiltering as i32, "Anisotropic"),
                ],
            ),
            integer_scaling: OptionEntryBoolean::new(
                "Integer Scaling",
                CANT_CHANGE_IN_GAME | RECREATE_UI,
                "Integer Scaling",
                "Scales the image using whole number pixel ratio.",
                false,
            ),
            v_sync: OptionEntryBoolean::new(
                "Vertical Sync",
                RECREATE_UI,
                "Vertical Sync",
                "Forces waiting for Vertical Sync. Prevents tearing effect when drawing a frame. Disabling it can help with mouse lag on some systems.",
                true,
            ),
            gamma_correction: OptionEntryInt::new("Gamma Correction", INVISIBLE, "Gamma Correction", "Gamma correction level.", 100, &[]),
            zoom: OptionEntryBoolean::new("Zoom", NONE, "Zoom", "Zoom on when enabled.", false),
            color_cycling: OptionEntryBoolean::new(
                "Color Cycling",
                NONE,
                "Color Cycling",
                "Color cycling effect used for water, lava, and acid animation.",
                true,
            ),
            alternate_nest_art: OptionEntryBoolean::new(
                "Alternate nest art",
                ONLY_HELLFIRE | CANT_CHANGE_IN_GAME,
                "Alternate nest art",
                "The game will use an alternative palette for Hellfire’s nest tileset.",
                false,
            ),
            hardware_cursor: OptionEntryBoolean::new(
                "Hardware Cursor",
                CANT_CHANGE_IN_GAME | RECREATE_UI | hw,
                "Hardware Cursor",
                "Use a hardware cursor",
                hardware_cursor_default(),
            ),
            hardware_cursor_for_items: OptionEntryBoolean::new(
                "Hardware Cursor For Items",
                CANT_CHANGE_IN_GAME | hw,
                "Hardware Cursor For Items",
                "Use a hardware cursor for items.",
                false,
            ),
            hardware_cursor_max_size: OptionEntryInt::new(
                "Hardware Cursor Maximum Size",
                CANT_CHANGE_IN_GAME | RECREATE_UI | hw,
                "Hardware Cursor Maximum Size",
                "Maximum width / height for the hardware cursor. Larger cursors fall back to software.",
                128,
                &[0, 64, 128, 256, 512],
            ),
            limit_fps: OptionEntryBoolean::new(
                "FPS Limiter",
                NONE,
                "FPS Limiter",
                "FPS is limited to avoid high CPU load. Limit considers refresh rate.",
                true,
            ),
            show_fps: OptionEntryBoolean::new("Show FPS", NONE, "Show FPS", "Displays the FPS in the upper left corner of the screen.", false),
        };
        o.resolution.base.set_value_changed_callback(OptionCallback::ResizeWindow);
        o.fullscreen.base.set_value_changed_callback(OptionCallback::SetFullscreenMode);
        o.fit_to_screen.base.set_value_changed_callback(OptionCallback::ResizeWindowAndUpdateResolutionOptions);
        o.upscale.base.set_value_changed_callback(OptionCallback::ResizeWindowAndUpdateResolutionOptions);
        o.scale_quality.base.set_value_changed_callback(OptionCallback::ReinitializeTexture);
        o.integer_scaling.base.set_value_changed_callback(OptionCallback::ReinitializeIntegerScale);
        o.v_sync.base.set_value_changed_callback(OptionCallback::ReinitializeRenderer);
        o.show_fps.base.set_value_changed_callback(OptionCallback::ShowFpsChanged);
        o
    }

    /// Original: `GraphicsOptions::GetEntries` (options.cpp).
    // @port options.cpp|devilution::GraphicsOptions::GetEntries() sha=45c0590f62ef
    pub fn get_entries(&mut self) -> Vec<&mut dyn OptionEntry> {
        vec![
            &mut self.resolution,
            &mut self.fullscreen,
            &mut self.fit_to_screen,
            &mut self.upscale,
            &mut self.scale_quality,
            &mut self.integer_scaling,
            &mut self.v_sync,
            &mut self.gamma_correction,
            &mut self.zoom,
            &mut self.limit_fps,
            &mut self.show_fps,
            &mut self.color_cycling,
            &mut self.alternate_nest_art,
            &mut self.hardware_cursor,
            &mut self.hardware_cursor_for_items,
            &mut self.hardware_cursor_max_size,
        ]
    }
}

pub struct GameplayOptions {
    pub category: OptionCategoryBase,
    pub tick_rate: OptionEntryInt,
    pub run_in_town: OptionEntryBoolean,
    pub grab_input: OptionEntryBoolean,
    pub theo_quest: OptionEntryBoolean,
    pub cow_quest: OptionEntryBoolean,
    pub friendly_fire: OptionEntryBoolean,
    pub multiplayer_full_quests: OptionEntryBoolean,
    pub test_bard: OptionEntryBoolean,
    pub test_barbarian: OptionEntryBoolean,
    pub experience_bar: OptionEntryBoolean,
    pub show_item_graphics_in_stores: OptionEntryBoolean,
    pub show_health_values: OptionEntryBoolean,
    pub show_mana_values: OptionEntryBoolean,
    pub enemy_health_bar: OptionEntryBoolean,
    pub auto_gold_pickup: OptionEntryBoolean,
    pub auto_elixir_pickup: OptionEntryBoolean,
    pub auto_oil_pickup: OptionEntryBoolean,
    pub auto_pickup_in_town: OptionEntryBoolean,
    pub adria_refills_mana: OptionEntryBoolean,
    pub auto_equip_weapons: OptionEntryBoolean,
    pub auto_equip_armor: OptionEntryBoolean,
    pub auto_equip_helms: OptionEntryBoolean,
    pub auto_equip_shields: OptionEntryBoolean,
    pub auto_equip_jewelry: OptionEntryBoolean,
    pub randomize_quests: OptionEntryBoolean,
    pub show_monster_type: OptionEntryBoolean,
    pub show_item_labels: OptionEntryBoolean,
    pub auto_refill_belt: OptionEntryBoolean,
    pub disable_crippling_shrines: OptionEntryBoolean,
    pub quick_cast: OptionEntryBoolean,
    pub num_heal_potion_pickup: OptionEntryInt,
    pub num_full_heal_potion_pickup: OptionEntryInt,
    pub num_mana_potion_pickup: OptionEntryInt,
    pub num_full_mana_potion_pickup: OptionEntryInt,
    pub num_reju_potion_pickup: OptionEntryInt,
    pub num_full_reju_potion_pickup: OptionEntryInt,
    pub enable_floating_numbers: OptionEntryEnum,
}

impl GameplayOptions {
    /// Original: `GameplayOptions::GameplayOptions` (options.cpp).
    // @port options.cpp|devilution::GameplayOptions::GameplayOptions() sha=9f5284bb343e
    fn new() -> Self {
        let b = OptionEntryBoolean::new;
        let pickup = |key: &str, name: &'static str, desc: &'static str| OptionEntryInt::new(key, NONE, name, desc, 0, &[0, 1, 2, 4, 8, 16]);
        let mut o = GameplayOptions {
            category: OptionCategoryBase::new("Game", "Gameplay", "Gameplay Settings"),
            tick_rate: OptionEntryInt::new("Speed", INVISIBLE, "Speed", "Gameplay ticks per second.", 20, &[]),
            run_in_town: b("Run in Town", CANT_CHANGE_IN_MULTI_PLAYER, "Run in Town", "Enable jogging/fast walking in town for Diablo and Hellfire. This option was introduced in the expansion.", false),
            grab_input: b("Grab Input", NONE, "Grab Input", "When enabled mouse is locked to the game window.", false),
            theo_quest: b("Theo Quest", CANT_CHANGE_IN_GAME | ONLY_HELLFIRE, "Theo Quest", "Enable Little Girl quest.", false),
            cow_quest: b("Cow Quest", CANT_CHANGE_IN_GAME | ONLY_HELLFIRE, "Cow Quest", "Enable Jersey's quest. Lester the farmer is replaced by the Complete Nut.", false),
            friendly_fire: b("Friendly Fire", CANT_CHANGE_IN_MULTI_PLAYER, "Friendly Fire", "Allow arrow/spell damage between players in multiplayer even when the friendly mode is on.", true),
            multiplayer_full_quests: b("MultiplayerFullQuests", CANT_CHANGE_IN_MULTI_PLAYER, "Full quests in Multiplayer", "Enables the full/uncut singleplayer version of quests.", false),
            test_bard: b("Test Bard", CANT_CHANGE_IN_GAME, "Test Bard", "Force the Bard character type to appear in the hero selection menu.", false),
            test_barbarian: b("Test Barbarian", CANT_CHANGE_IN_GAME, "Test Barbarian", "Force the Barbarian character type to appear in the hero selection menu.", false),
            experience_bar: b("Experience Bar", NONE, "Experience Bar", "Experience Bar is added to the UI at the bottom of the screen.", false),
            show_item_graphics_in_stores: b("Show Item Graphics in Stores", NONE, "Show Item Graphics in Stores", "Show item graphics to the left of item descriptions in store menus.", false),
            show_health_values: b("Show health values", NONE, "Show health values", "Displays current / max health value on health globe.", false),
            show_mana_values: b("Show mana values", NONE, "Show mana values", "Displays current / max mana value on mana globe.", false),
            enemy_health_bar: b("Enemy Health Bar", NONE, "Enemy Health Bar", "Enemy Health Bar is displayed at the top of the screen.", false),
            auto_gold_pickup: b("Auto Gold Pickup", NONE, "Auto Gold Pickup", "Gold is automatically collected when in close proximity to the player.", false),
            auto_elixir_pickup: b("Auto Elixir Pickup", NONE, "Auto Elixir Pickup", "Elixirs are automatically collected when in close proximity to the player.", false),
            auto_oil_pickup: b("Auto Oil Pickup", ONLY_HELLFIRE, "Auto Oil Pickup", "Oils are automatically collected when in close proximity to the player.", false),
            auto_pickup_in_town: b("Auto Pickup in Town", NONE, "Auto Pickup in Town", "Automatically pickup items in town.", false),
            adria_refills_mana: b("Adria Refills Mana", NONE, "Adria Refills Mana", "Adria will refill your mana when you visit her shop.", false),
            auto_equip_weapons: b("Auto Equip Weapons", NONE, "Auto Equip Weapons", "Weapons will be automatically equipped on pickup or purchase if enabled.", true),
            auto_equip_armor: b("Auto Equip Armor", NONE, "Auto Equip Armor", "Armor will be automatically equipped on pickup or purchase if enabled.", false),
            auto_equip_helms: b("Auto Equip Helms", NONE, "Auto Equip Helms", "Helms will be automatically equipped on pickup or purchase if enabled.", false),
            auto_equip_shields: b("Auto Equip Shields", NONE, "Auto Equip Shields", "Shields will be automatically equipped on pickup or purchase if enabled.", false),
            auto_equip_jewelry: b("Auto Equip Jewelry", NONE, "Auto Equip Jewelry", "Jewelry will be automatically equipped on pickup or purchase if enabled.", false),
            randomize_quests: b("Randomize Quests", CANT_CHANGE_IN_GAME, "Randomize Quests", "Randomly selecting available quests for new games.", true),
            show_monster_type: b("Show Monster Type", NONE, "Show Monster Type", "Hovering over a monster will display the type of monster in the description box in the UI.", false),
            show_item_labels: b("Show Item Labels", NONE, "Show Item Labels", "Show labels for items on the ground when enabled.", false),
            auto_refill_belt: b("Auto Refill Belt", NONE, "Auto Refill Belt", "Refill belt from inventory when belt item is consumed.", false),
            disable_crippling_shrines: b("Disable Crippling Shrines", NONE, "Disable Crippling Shrines", "When enabled Cauldrons, Fascinating Shrines, Goat Shrines, Ornate Shrines and Sacred Shrines are not able to be clicked on and labeled as disabled.", false),
            quick_cast: b("Quick Cast", NONE, "Quick Cast", "Spell hotkeys instantly cast the spell, rather than switching the readied spell.", false),
            num_heal_potion_pickup: pickup("Heal Potion Pickup", "Heal Potion Pickup", "Number of Healing potions to pick up automatically."),
            num_full_heal_potion_pickup: pickup("Full Heal Potion Pickup", "Full Heal Potion Pickup", "Number of Full Healing potions to pick up automatically."),
            num_mana_potion_pickup: pickup("Mana Potion Pickup", "Mana Potion Pickup", "Number of Mana potions to pick up automatically."),
            num_full_mana_potion_pickup: pickup("Full Mana Potion Pickup", "Full Mana Potion Pickup", "Number of Full Mana potions to pick up automatically."),
            num_reju_potion_pickup: pickup("Rejuvenation Potion Pickup", "Rejuvenation Potion Pickup", "Number of Rejuvenation potions to pick up automatically."),
            num_full_reju_potion_pickup: pickup("Full Rejuvenation Potion Pickup", "Full Rejuvenation Potion Pickup", "Number of Full Rejuvenation potions to pick up automatically."),
            enable_floating_numbers: OptionEntryEnum::new(
                "Enable floating numbers",
                NONE,
                "Enable floating numbers",
                "Enables floating numbers on gaining XP / dealing damage etc.",
                FloatingNumbers::Off as i32,
                &[
                    (FloatingNumbers::Off as i32, "Off"),
                    (FloatingNumbers::Random as i32, "Random Angles"),
                    (FloatingNumbers::Vertical as i32, "Vertical Only"),
                ],
            ),
        };
        o.grab_input.base.set_value_changed_callback(OptionCallback::GrabInputChanged);
        o.experience_bar.base.set_value_changed_callback(OptionCallback::ExperienceBarChanged);
        o.enemy_health_bar.base.set_value_changed_callback(OptionCallback::EnemyHealthBarChanged);
        o
    }

    /// Original: `GameplayOptions::GetEntries` (options.cpp).
    // @port options.cpp|devilution::GameplayOptions::GetEntries() sha=4d99a635dd67
    pub fn get_entries(&mut self) -> Vec<&mut dyn OptionEntry> {
        vec![
            &mut self.tick_rate,
            &mut self.friendly_fire,
            &mut self.multiplayer_full_quests,
            &mut self.randomize_quests,
            &mut self.theo_quest,
            &mut self.cow_quest,
            &mut self.run_in_town,
            &mut self.quick_cast,
            &mut self.test_bard,
            &mut self.test_barbarian,
            &mut self.experience_bar,
            &mut self.show_item_graphics_in_stores,
            &mut self.show_health_values,
            &mut self.show_mana_values,
            &mut self.enemy_health_bar,
            &mut self.show_monster_type,
            &mut self.show_item_labels,
            &mut self.enable_floating_numbers,
            &mut self.auto_refill_belt,
            &mut self.auto_equip_weapons,
            &mut self.auto_equip_armor,
            &mut self.auto_equip_helms,
            &mut self.auto_equip_shields,
            &mut self.auto_equip_jewelry,
            &mut self.auto_gold_pickup,
            &mut self.auto_elixir_pickup,
            &mut self.auto_oil_pickup,
            &mut self.num_heal_potion_pickup,
            &mut self.num_full_heal_potion_pickup,
            &mut self.num_mana_potion_pickup,
            &mut self.num_full_mana_potion_pickup,
            &mut self.num_reju_potion_pickup,
            &mut self.num_full_reju_potion_pickup,
            &mut self.auto_pickup_in_town,
            &mut self.disable_crippling_shrines,
            &mut self.adria_refills_mana,
            &mut self.grab_input,
        ]
    }
}

pub struct ControllerOptions {
    pub category: OptionCategoryBase,
    /// `szMapping[1024]`
    pub sz_mapping: String,
    pub f_deadzone: f32,
}

impl ControllerOptions {
    /// Original: `ControllerOptions::ControllerOptions` (options.cpp).
    // @port options.cpp|devilution::ControllerOptions::ControllerOptions() sha=871abc905014
    fn new() -> Self {
        ControllerOptions { category: OptionCategoryBase::new("Controller", "Controller", "Controller Settings"), sz_mapping: String::new(), f_deadzone: 0.0 }
    }
    /// Original: `ControllerOptions::GetEntries` (options.cpp).
    // @port options.cpp|devilution::ControllerOptions::GetEntries() sha=746c32097125
    pub fn get_entries(&mut self) -> Vec<&mut dyn OptionEntry> {
        vec![]
    }
}

pub struct NetworkOptions {
    pub category: OptionCategoryBase,
    pub sz_bind_address: String,
    pub sz_previous_zt_game: String,
    pub sz_previous_host: String,
    pub port: OptionEntryInt,
}

impl NetworkOptions {
    /// Original: `NetworkOptions::NetworkOptions` (options.cpp).
    // @port options.cpp|devilution::NetworkOptions::NetworkOptions() sha=640441112c55
    fn new() -> Self {
        NetworkOptions {
            category: OptionCategoryBase::new("Network", "Network", "Network Settings"),
            sz_bind_address: String::new(),
            sz_previous_zt_game: String::new(),
            sz_previous_host: String::new(),
            port: OptionEntryInt::new("Port", INVISIBLE, "Port", "What network port to use.", 6112, &[]),
        }
    }
    /// Original: `NetworkOptions::GetEntries` (options.cpp).
    // @port options.cpp|devilution::NetworkOptions::GetEntries() sha=9e96bf4912b2
    pub fn get_entries(&mut self) -> Vec<&mut dyn OptionEntry> {
        vec![&mut self.port]
    }
}

pub const QUICK_MESSAGE_OPTIONS: usize = 4;

pub struct ChatOptions {
    pub category: OptionCategoryBase,
    pub sz_hot_key_msgs: [Vec<String>; QUICK_MESSAGE_OPTIONS],
}

impl ChatOptions {
    /// Original: `ChatOptions::ChatOptions` (options.cpp).
    // @port options.cpp|devilution::ChatOptions::ChatOptions() sha=5796aaa183eb
    fn new() -> Self {
        ChatOptions { category: OptionCategoryBase::new("NetMsg", "Chat", "Chat Settings"), sz_hot_key_msgs: Default::default() }
    }
    /// Original: `ChatOptions::GetEntries` (options.cpp).
    // @port options.cpp|devilution::ChatOptions::GetEntries() sha=c7088bd9c724
    pub fn get_entries(&mut self) -> Vec<&mut dyn OptionEntry> {
        vec![]
    }
}

pub struct LanguageOptions {
    pub category: OptionCategoryBase,
    pub code: OptionEntryLanguageCode,
}

impl LanguageOptions {
    /// Original: `LanguageOptions::LanguageOptions` (options.cpp).
    // @port options.cpp|devilution::LanguageOptions::LanguageOptions() sha=e4039d0d98e8
    fn new() -> Self {
        let mut o = LanguageOptions { category: OptionCategoryBase::new("Language", "Language", "Language Settings"), code: OptionEntryLanguageCode::new() };
        o.code.base.set_value_changed_callback(OptionCallback::LanguageCodeChanged);
        o
    }
    /// Original: `LanguageOptions::GetEntries` (options.cpp).
    // @port options.cpp|devilution::LanguageOptions::GetEntries() sha=9d61f1323671
    pub fn get_entries(&mut self) -> Vec<&mut dyn OptionEntry> {
        vec![&mut self.code]
    }
}

// ---------------------------------------------------------------------------------------------
// Keymapper / Padmapper

pub type ActionFn = std::rc::Rc<dyn Fn(&mut Ctx)>;
pub type EnableFn = std::rc::Rc<dyn Fn(&Ctx) -> bool>;

/// `KeymapperOptions::Action`
pub struct KeymapperAction {
    pub base: OptionEntryBase,
    default_key: u32,
    action_pressed: Option<ActionFn>,
    action_released: Option<ActionFn>,
    enable: Option<EnableFn>,
    pub bound_key: u32,
    dynamic_index: u32,
}

impl KeymapperAction {
    /// Original: `KeymapperOptions::Action::GetName` (options.cpp): names of indexed actions
    /// are format strings with `{}`.
    // @port options.cpp|devilution::KeymapperOptions::Action::GetName() sha=e70a0b302098
    pub fn get_name(&self) -> String {
        if self.dynamic_index == 0 {
            return tr(self.base.name);
        }
        tr(self.base.name).replacen("{}", &self.dynamic_index.to_string(), 1)
    }

    /// Original: `KeymapperOptions::Action::GetType` (options.h).
    // @port options.h|devilution::KeymapperOptions::Action::GetType() sha=7ef47c4a184b
    pub fn get_type(&self) -> OptionEntryType {
        OptionEntryType::Key
    }
}

pub struct KeymapperOptions {
    pub category: OptionCategoryBase,
    pub actions: Vec<KeymapperAction>,
    key_id_to_action: std::collections::HashMap<u32, usize>,
    key_id_to_key_name: std::collections::HashMap<u32, String>,
    key_name_to_key_id: std::collections::HashMap<String, u32>,
}

impl KeymapperOptions {
    /// Original: `KeymapperOptions::KeymapperOptions` (options.cpp).
    // @port options.cpp|devilution::KeymapperOptions::KeymapperOptions() sha=80469cd646f8
    fn new() -> Self {
        let mut k = std::collections::HashMap::new();
        for c in b'A'..=b'Z' {
            k.insert(c as u32, (c as char).to_string());
        }
        for c in b'0'..=b'9' {
            k.insert(c as u32, (c as char).to_string());
        }
        for i in 0..12 {
            k.insert((SDLK_F1 + i) as u32, format!("F{}", i + 1));
        }
        k.insert(SDLK_LALT as u32, "LALT".into());
        k.insert(SDLK_RALT as u32, "RALT".into());
        k.insert(SDLK_SPACE as u32, "SPACE".into());
        k.insert(SDLK_RCTRL as u32, "RCONTROL".into());
        k.insert(SDLK_LCTRL as u32, "LCONTROL".into());
        k.insert(SDLK_PRINTSCREEN as u32, "PRINT".into());
        k.insert(SDLK_PAUSE as u32, "PAUSE".into());
        k.insert(SDLK_TAB as u32, "TAB".into());
        k.insert(BUTTON_MIDDLE as u32 | KEYMAPPER_MOUSE_BUTTON_MASK, "MMOUSE".into());
        k.insert(BUTTON_X1 as u32 | KEYMAPPER_MOUSE_BUTTON_MASK, "X1MOUSE".into());
        k.insert(BUTTON_X2 as u32 | KEYMAPPER_MOUSE_BUTTON_MASK, "X2MOUSE".into());
        let n = k.iter().map(|(id, name)| (name.clone(), *id)).collect();
        KeymapperOptions {
            category: OptionCategoryBase::new("Keymapping", "Keymapping", "Keymapping Settings"),
            actions: Vec::new(),
            key_id_to_action: Default::default(),
            key_id_to_key_name: k,
            key_name_to_key_id: n,
        }
    }

    /// Original: `KeymapperOptions::AddAction` + `Action::Action` (options.cpp). Actions are
    /// appended in order (the original pushes to the front and reverses in `CommitActions`).
    // @port options.cpp|devilution::KeymapperOptions::AddAction(string_view key, const char *name, const char *description, uint32_t defaultKey, std::function<void()> actionPressed, std::function<void()> actionReleased, std::function<bool()> enable, unsigned index) sha=4f38ac180226
    // @port options.cpp|devilution::KeymapperOptions::Action::Action(string_view key, const char *name, const char *description, uint32_t defaultKey, std::function<void()> actionPressed, std::function<void()> actionReleased, std::function<bool()> enable, unsigned index) sha=dea0710142d6
    #[allow(clippy::too_many_arguments)]
    pub fn add_action(
        &mut self,
        key: &str,
        name: &'static str,
        description: &'static str,
        default_key: u32,
        action_pressed: Option<ActionFn>,
        action_released: Option<ActionFn>,
        enable: Option<EnableFn>,
        index: u32,
    ) {
        let key = if index != 0 { key.replacen("{}", &index.to_string(), 1) } else { key.to_string() };
        self.actions.push(KeymapperAction {
            base: OptionEntryBase::new(&key, NONE, name, description),
            default_key,
            action_pressed,
            action_released,
            enable,
            bound_key: SDLK_UNKNOWN as u32,
            dynamic_index: index,
        });
    }

    /// Original: `KeymapperOptions::CommitActions` (options.cpp): the actions are already in
    /// insertion order.
    // @port options.cpp|devilution::KeymapperOptions::CommitActions() sha=aec27f70a9ec
    pub fn commit_actions(&mut self) {}

    /// Original: `KeymapperOptions::Action::SetValue` (options.cpp).
    // @port options.cpp|devilution::KeymapperOptions::Action::SetValue(int value) sha=f4129c13e316
    pub fn set_action_value(&mut self, idx: usize, value: u32) -> bool {
        if value != SDLK_UNKNOWN as u32 && !self.key_id_to_key_name.contains_key(&value) {
            return false;
        }
        let old = self.actions[idx].bound_key;
        if old != SDLK_UNKNOWN as u32 {
            self.key_id_to_action.remove(&old);
            self.actions[idx].bound_key = SDLK_UNKNOWN as u32;
        }
        if value != SDLK_UNKNOWN as u32 {
            if let Some(&other) = self.key_id_to_action.get(&value) {
                log::info!("Keymapper: key '{}' is already bound to action '{}', overwriting", value, self.actions[other].base.name);
                self.actions[other].bound_key = SDLK_UNKNOWN as u32;
            }
            self.key_id_to_action.insert(value, idx);
            self.actions[idx].bound_key = value;
        }
        true
    }

    /// Original: `KeymapperOptions::Action::LoadFromIni` (options.cpp).
    // @port options.cpp|devilution::KeymapperOptions::Action::LoadFromIni(string_view category) sha=44d0f2865e1d
    fn load_action(&mut self, io: &mut IniIo, idx: usize, category: &str) {
        let key = self.actions[idx].base.key.clone();
        let (found, read_key) = io.get_value(category, &key, 64, "");
        if !found {
            let d = self.actions[idx].default_key;
            self.set_action_value(idx, d);
            return;
        }
        if read_key.is_empty() {
            self.set_action_value(idx, SDLK_UNKNOWN as u32);
            return;
        }
        match self.key_name_to_key_id.get(&read_key).copied() {
            Some(id) => {
                self.set_action_value(idx, id);
            }
            None => {
                log::info!("Keymapper: unknown key '{}'", read_key);
                let d = self.actions[idx].default_key;
                self.set_action_value(idx, d);
            }
        }
    }

    /// Original: `KeymapperOptions::Action::SaveToIni` (options.cpp). As in the original, an
    /// unbound action writes "" and then falls through to the (failing) name lookup.
    // @port options.cpp|devilution::KeymapperOptions::Action::SaveToIni(string_view category) sha=5ba0d141ccab
    fn save_action(&self, io: &mut IniIo, idx: usize, category: &str) {
        let a = &self.actions[idx];
        if a.bound_key == SDLK_UNKNOWN as u32 {
            io.set_str(category, &a.base.key, "");
        }
        match self.key_id_to_key_name.get(&a.bound_key) {
            Some(name) => io.set_str(category, &a.base.key, name),
            None => log::verbose!("Keymapper: no name found for key '{}'", a.base.key),
        }
    }

    /// Original: `KeymapperOptions::Action::GetValueDescription` (options.cpp).
    // @port options.cpp|devilution::KeymapperOptions::Action::GetValueDescription() sha=84a771b8745d
    pub fn action_value_description(&self, idx: usize) -> String {
        let a = &self.actions[idx];
        if a.bound_key == SDLK_UNKNOWN as u32 {
            return String::new();
        }
        self.key_id_to_key_name.get(&a.bound_key).cloned().unwrap_or_default()
    }

    /// Original: `KeymapperOptions::IsTextEntryKey` (options.cpp).
    // @port options.cpp|devilution::KeymapperOptions::IsTextEntryKey(SDL_Keycode vkey) sha=82ed1e9143a7
    pub fn is_text_entry_key(vkey: i32) -> bool {
        [SDLK_ESCAPE, SDLK_RETURN, SDLK_KP_ENTER, SDLK_BACKSPACE, SDLK_DOWN, SDLK_UP].contains(&vkey)
            || (SDLK_SPACE..=b'z' as i32).contains(&vkey)
    }

    /// Original: `KeymapperOptions::IsNumberEntryKey` (options.cpp).
    // @port options.cpp|devilution::KeymapperOptions::IsNumberEntryKey(SDL_Keycode vkey) sha=a22f3cfd3a6a
    pub fn is_number_entry_key(vkey: i32) -> bool {
        (b'0' as i32..=b'9' as i32).contains(&vkey) || vkey == SDLK_BACKSPACE
    }

    /// Original: `KeymapperOptions::KeyNameForAction` (options.cpp).
    // @port options.cpp|devilution::KeymapperOptions::KeyNameForAction(string_view actionName) sha=dc3bf52639c9
    pub fn key_name_for_action(&self, action_name: &str) -> String {
        for (i, a) in self.actions.iter().enumerate() {
            if a.base.key == action_name && a.bound_key != SDLK_UNKNOWN as u32 {
                return self.action_value_description(i);
            }
        }
        String::new()
    }

    /// Original: `KeymapperOptions::KeyForAction` (options.cpp).
    // @port options.cpp|devilution::KeymapperOptions::KeyForAction(string_view actionName) sha=a53538cf3ce5
    pub fn key_for_action(&self, action_name: &str) -> u32 {
        for a in &self.actions {
            if a.base.key == action_name && a.bound_key != SDLK_UNKNOWN as u32 {
                return a.bound_key;
            }
        }
        SDLK_UNKNOWN as u32
    }
}

/// Original: `KeymapperOptions::KeyPressed` (options.cpp).
// @port options.cpp|devilution::KeymapperOptions::KeyPressed(uint32_t key) sha=2bd2ba2ec8da
pub fn keymapper_key_pressed(ctx: &mut Ctx, key: u32) {
    let mut key = key;
    if (b'a' as u32..=b'z' as u32).contains(&key) {
        key -= (b'a' - b'A') as u32;
    }
    let Some(&idx) = ctx.options.keymapper.key_id_to_action.get(&key) else { return };
    let a = &ctx.options.keymapper.actions[idx];
    let (pressed, enable) = (a.action_pressed.clone(), a.enable.clone());
    let Some(pressed) = pressed else { return };
    if enable.is_some_and(|e| !e(ctx)) || ctx.control.talkflag {
        return;
    }
    pressed(ctx);
}

/// Original: `KeymapperOptions::KeyReleased` (options.cpp).
// @port options.cpp|devilution::KeymapperOptions::KeyReleased(SDL_Keycode key) sha=b867e81bd71a
pub fn keymapper_key_released(ctx: &mut Ctx, key: i32) {
    let mut key = key;
    if (b'a' as i32..=b'z' as i32).contains(&key) {
        key -= (b'a' - b'A') as i32;
    }
    let Some(&idx) = ctx.options.keymapper.key_id_to_action.get(&(key as u32)) else { return };
    let a = &ctx.options.keymapper.actions[idx];
    let (released, enable) = (a.action_released.clone(), a.enable.clone());
    let Some(released) = released else { return };
    if enable.is_some_and(|e| !e(ctx))
        || (ctx.control.talkflag && KeymapperOptions::is_text_entry_key(key))
        || (ctx.control.drop_gold_flag && KeymapperOptions::is_number_entry_key(key))
    {
        return;
    }
    released(ctx);
}

/// `PadmapperOptions::Action`
pub struct PadmapperAction {
    pub base: OptionEntryBase,
    default_input: ControllerButtonCombo,
    action_pressed: Option<ActionFn>,
    action_released: Option<ActionFn>,
    enable: Option<EnableFn>,
    pub bound_input: ControllerButtonCombo,
    dynamic_index: u32,
}

impl PadmapperAction {
    /// Original: `PadmapperOptions::Action::GetName` (options.cpp).
    // @port options.cpp|devilution::PadmapperOptions::Action::GetName() sha=b90bf0a515b0
    pub fn get_name(&self) -> String {
        if self.dynamic_index == 0 {
            return tr(self.base.name);
        }
        tr(self.base.name).replacen("{}", &self.dynamic_index.to_string(), 1)
    }

    /// Original: `PadmapperOptions::Action::GetType` (options.h).
    // @port options.h|devilution::PadmapperOptions::Action::GetType() sha=7c0ca054a5e6
    pub fn get_type(&self) -> OptionEntryType {
        OptionEntryType::PadButton
    }

    /// Original: `PadmapperOptions::Action::SetValue` (options.cpp). The cached descriptions
    /// (`UpdateValueDescription`) are computed on demand by `value_description`.
    // @port options.cpp|devilution::PadmapperOptions::Action::SetValue(ControllerButtonCombo value) sha=735c2ab02a78
    pub fn set_value(&mut self, value: ControllerButtonCombo) -> bool {
        if self.bound_input.button != ControllerButton::None {
            self.bound_input = ControllerButtonCombo::default();
        }
        if value.button != ControllerButton::None {
            self.bound_input = value;
        }
        true
    }

    /// Original: `PadmapperOptions::Action::Shorten` (options.cpp): the first 3 characters.
    // @port options.cpp|devilution::PadmapperOptions::Action::Shorten(string_view buttonName) sha=511a8b70b1af
    fn shorten(button_name: &str) -> &str {
        match button_name.char_indices().nth(3) {
            Some((i, _)) => &button_name[..i],
            None => button_name,
        }
    }

    /// Original: `PadmapperOptions::Action::GetValueDescription(bool)` +
    /// `UpdateValueDescription` (options.cpp).
    // @port options.cpp|devilution::PadmapperOptions::Action::GetValueDescription(bool useShortName) sha=86f4de102dd8
    // @port options.cpp|devilution::PadmapperOptions::Action::GetValueDescription() sha=2cecae2006b9
    // @port options.cpp|devilution::PadmapperOptions::Action::UpdateValueDescription() sha=b90545079749
    pub fn value_description(&self, ctx: &Ctx, use_short_name: bool) -> String {
        if self.bound_input.button == ControllerButton::None {
            return String::new();
        }
        let button_name = crate::controls::game_controls::to_string(ctx, self.bound_input.button);
        if self.bound_input.modifier == ControllerButton::None {
            return if use_short_name { Self::shorten(&button_name).to_string() } else { button_name };
        }
        let modifier_name = crate::controls::game_controls::to_string(ctx, self.bound_input.modifier);
        if use_short_name {
            format!("{}+{}", Self::shorten(&modifier_name), Self::shorten(&button_name))
        } else {
            format!("{modifier_name}+{button_name}")
        }
    }
}

pub struct PadmapperOptions {
    pub category: OptionCategoryBase,
    pub actions: Vec<PadmapperAction>,
    button_to_release_action: [Option<usize>; ControllerButton::COUNT],
    button_to_button_name: [&'static str; ControllerButton::COUNT],
    committed: bool,
}

impl PadmapperOptions {
    /// Original: `PadmapperOptions::PadmapperOptions` (options.cpp).
    // @port options.cpp|devilution::PadmapperOptions::PadmapperOptions() sha=1bcc8ff5981f
    fn new() -> Self {
        PadmapperOptions {
            category: OptionCategoryBase::new("Padmapping", "Padmapping", "Padmapping Settings"),
            actions: Vec::new(),
            button_to_release_action: [None; ControllerButton::COUNT],
            button_to_button_name: [
                "", "", "LT", "RT", "A", "B", "X", "Y", "LS", "RS", "LB", "RB", "Start", "Select", "Up", "Down", "Left", "Right",
            ],
            committed: false,
        }
    }

    fn button_by_name(&self, name: &str) -> Option<ControllerButton> {
        // buttonNameToButton: the empty names of NONE/IGNORE both map "" (the last wins: IGNORE)
        let mut found = None;
        for (i, n) in self.button_to_button_name.iter().enumerate() {
            if *n == name {
                found = Some(ControllerButton::from_index(i));
            }
        }
        found
    }

    /// Original: `PadmapperOptions::AddAction` + `Action::Action` (options.cpp).
    // @port options.cpp|devilution::PadmapperOptions::AddAction(string_view key, const char *name, const char *description, ControllerButtonCombo defaultInput, std::function<void()> actionPressed, std::function<void()> actionReleased, std::function<bool()> enable, unsigned index) sha=2c5e608e4c38
    // @port options.cpp|devilution::PadmapperOptions::Action::Action(string_view key, const char *name, const char *description, ControllerButtonCombo defaultInput, std::function<void()> actionPressed, std::function<void()> actionReleased, std::function<bool()> enable, unsigned index) sha=53259d9e5531
    #[allow(clippy::too_many_arguments)]
    pub fn add_action(
        &mut self,
        key: &str,
        name: &'static str,
        description: &'static str,
        default_input: ControllerButtonCombo,
        action_pressed: Option<ActionFn>,
        action_released: Option<ActionFn>,
        enable: Option<EnableFn>,
        index: u32,
    ) {
        if self.committed {
            return;
        }
        let key = if index != 0 { key.replacen("{}", &index.to_string(), 1) } else { key.to_string() };
        self.actions.push(PadmapperAction {
            base: OptionEntryBase::new(&key, NONE, name, description),
            default_input,
            action_pressed,
            action_released,
            enable,
            bound_input: ControllerButtonCombo::default(),
            dynamic_index: index,
        });
    }

    /// Original: `PadmapperOptions::CommitActions` (options.cpp).
    // @port options.cpp|devilution::PadmapperOptions::CommitActions() sha=744d78b705fe
    pub fn commit_actions(&mut self) {
        if self.committed {
            return;
        }
        self.committed = true;
    }

    /// Original: `PadmapperOptions::Action::LoadFromIni` (options.cpp).
    // @port options.cpp|devilution::PadmapperOptions::Action::LoadFromIni(string_view category) sha=9ee1e578c6d7
    fn load_action(&mut self, io: &mut IniIo, idx: usize, category: &str) {
        let key = self.actions[idx].base.key.clone();
        let (found, result) = io.get_value(category, &key, 64, "");
        if !found {
            let d = self.actions[idx].default_input;
            self.actions[idx].set_value(d);
            return;
        }
        let parts: Vec<&str> = result.split('+').collect();
        // SplitByChar yields one empty part for an empty string
        let (mod_name, button_name) = if parts.len() >= 2 { (parts[0], parts[1]) } else { ("", parts[0]) };
        let mut input = ControllerButtonCombo::default();
        if !mod_name.is_empty() {
            match self.button_by_name(mod_name) {
                Some(b) => input.modifier = b,
                None => {
                    log::info!("Padmapper: unknown button '{}'", mod_name);
                    let d = self.actions[idx].default_input;
                    self.actions[idx].set_value(d);
                    return;
                }
            }
        }
        match self.button_by_name(button_name) {
            Some(b) => input.button = b,
            None => {
                log::info!("Padmapper: unknown button '{}'", button_name);
                let d = self.actions[idx].default_input;
                self.actions[idx].set_value(d);
                return;
            }
        }
        self.actions[idx].set_value(input);
    }

    /// Original: `PadmapperOptions::Action::SaveToIni` (options.cpp).
    // @port options.cpp|devilution::PadmapperOptions::Action::SaveToIni(string_view category) sha=8ee4a74c8d3b
    fn save_action(&self, io: &mut IniIo, idx: usize, category: &str) {
        let a = &self.actions[idx];
        if a.bound_input.button == ControllerButton::None {
            io.set_str(category, &a.base.key, "");
            return;
        }
        let mut input_name = self.button_to_button_name[a.bound_input.button as usize].to_string();
        if input_name.is_empty() {
            log::verbose!("Padmapper: no name found for key '{}'", a.base.key);
            return;
        }
        if a.bound_input.modifier != ControllerButton::None {
            let modifier_name = self.button_to_button_name[a.bound_input.modifier as usize];
            if modifier_name.is_empty() {
                log::verbose!("Padmapper: no name found for key '{}'", a.base.key);
                return;
            }
            input_name = format!("{modifier_name}+{input_name}");
        }
        io.set_str(category, &a.base.key, &input_name);
    }

    /// Original: `PadmapperOptions::IsActive` (options.cpp).
    // @port options.cpp|devilution::PadmapperOptions::IsActive(string_view actionName) sha=be7c371705cc
    pub fn is_active(&self, action_name: &str) -> bool {
        for a in &self.actions {
            if a.base.key != action_name {
                continue;
            }
            let release = self.button_to_release_action[a.bound_input.button as usize];
            return release.is_some_and(|r| self.actions[r].base.key == action_name);
        }
        false
    }

    /// Original: `PadmapperOptions::InputNameForAction` (options.cpp).
    // @port options.cpp|devilution::PadmapperOptions::InputNameForAction(string_view actionName, bool useShortName) sha=754b44a01dea
    pub fn input_name_for_action(&self, ctx: &Ctx, action_name: &str, use_short_name: bool) -> String {
        for a in &self.actions {
            if a.base.key == action_name && a.bound_input.button != ControllerButton::None {
                return a.value_description(ctx, use_short_name);
            }
        }
        String::new()
    }

    /// Original: `PadmapperOptions::ButtonComboForAction` (options.cpp).
    // @port options.cpp|devilution::PadmapperOptions::ButtonComboForAction(string_view actionName) sha=8f5d2dc5fa33
    pub fn button_combo_for_action(&self, action_name: &str) -> ControllerButtonCombo {
        for a in &self.actions {
            if a.base.key == action_name && a.bound_input.button != ControllerButton::None {
                return a.bound_input;
            }
        }
        ControllerButtonCombo::default()
    }
}

/// Original: `PadmapperOptions::FindAction` (options.cpp).
// @port options.cpp|devilution::PadmapperOptions::FindAction(ControllerButton button) sha=46e2c6c522c0
fn padmapper_find_action(ctx: &Ctx, button: ControllerButton) -> Option<usize> {
    let p = &ctx.options.padmapper;
    for (i, a) in p.actions.iter().enumerate() {
        let combo = a.bound_input;
        if combo.modifier == ControllerButton::None || button != combo.button {
            continue;
        }
        if !crate::controls::controller::is_controller_button_pressed(ctx, combo.modifier) {
            continue;
        }
        if a.enable.as_ref().is_some_and(|e| !e(ctx)) {
            continue;
        }
        return Some(i);
    }
    for (i, a) in p.actions.iter().enumerate() {
        let combo = a.bound_input;
        if combo.modifier != ControllerButton::None || button != combo.button {
            continue;
        }
        if a.enable.as_ref().is_some_and(|e| !e(ctx)) {
            continue;
        }
        return Some(i);
    }
    None
}

/// Original: `PadmapperOptions::CanDeferToMovementHandler` (options.cpp).
// @port options.cpp|devilution::PadmapperOptions::CanDeferToMovementHandler(const Action &action) sha=f43611f0cc84
fn padmapper_can_defer_to_movement_handler(ctx: &Ctx, idx: usize) -> bool {
    let a = &ctx.options.padmapper.actions[idx];
    if a.bound_input.modifier != ControllerButton::None {
        return false;
    }
    if ctx.control.spselflag && a.base.key.starts_with("QuickSpell") {
        return false;
    }
    matches!(
        a.bound_input.button,
        ControllerButton::ButtonDpadUp | ControllerButton::ButtonDpadDown | ControllerButton::ButtonDpadLeft | ControllerButton::ButtonDpadRight
    )
}

/// Original: `PadmapperOptions::ButtonPressed` (options.cpp).
// @port options.cpp|devilution::PadmapperOptions::ButtonPressed(ControllerButton button) sha=b1da09005efc
pub fn padmapper_button_pressed(ctx: &mut Ctx, button: ControllerButton) {
    let Some(idx) = padmapper_find_action(ctx, button) else { return };
    if crate::controls::plrctrls::is_movement_handler_active(ctx) && padmapper_can_defer_to_movement_handler(ctx, idx) {
        return;
    }
    if let Some(f) = ctx.options.padmapper.actions[idx].action_pressed.clone() {
        f(ctx);
    }
    ctx.controls.suppressed_button = ctx.options.padmapper.actions[idx].bound_input.modifier;
    ctx.options.padmapper.button_to_release_action[button as usize] = Some(idx);
}

/// Original: `PadmapperOptions::ButtonReleased` (options.cpp).
// @port options.cpp|devilution::PadmapperOptions::ButtonReleased(ControllerButton button, bool invokeAction) sha=cdd622229c71
pub fn padmapper_button_released(ctx: &mut Ctx, button: ControllerButton, invoke_action: bool) {
    if invoke_action {
        let Some(idx) = ctx.options.padmapper.button_to_release_action[button as usize] else { return };
        let a = &ctx.options.padmapper.actions[idx];
        let (released, enable) = (a.action_released.clone(), a.enable.clone());
        if let Some(f) = released {
            if enable.is_none_or(|e| e(ctx)) {
                f(ctx);
            }
        }
    }
    ctx.options.padmapper.button_to_release_action[button as usize] = None;
}

/// Original: `PadmapperOptions::ReleaseAllActiveButtons` (options.cpp).
// @port options.cpp|devilution::PadmapperOptions::ReleaseAllActiveButtons() sha=b26d9e1ccff0
pub fn padmapper_release_all_active_buttons(ctx: &mut Ctx) {
    for i in 0..ControllerButton::COUNT {
        let Some(idx) = ctx.options.padmapper.button_to_release_action[i] else { continue };
        let button = ctx.options.padmapper.actions[idx].bound_input.button;
        padmapper_button_released(ctx, button, true);
    }
}

/// Original: `PadmapperOptions::ActionNameTriggeredByButtonEvent` (options.cpp).
// @port options.cpp|devilution::PadmapperOptions::ActionNameTriggeredByButtonEvent(ControllerButtonEvent ctrlEvent) sha=b9acd02bf1d7
pub fn padmapper_action_name_triggered_by_button_event(ctx: &Ctx, button: ControllerButton, up: bool) -> String {
    if !ctx.diablo.gb_run_game {
        return String::new();
    }
    if !up {
        return padmapper_find_action(ctx, button).map(|i| ctx.options.padmapper.actions[i].base.key.clone()).unwrap_or_default();
    }
    match ctx.options.padmapper.button_to_release_action[button as usize] {
        Some(i) => ctx.options.padmapper.actions[i].base.key.clone(),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------------------------
// Options, LoadOptions, SaveOptions

pub struct Options {
    pub start_up: StartUpOptions,
    pub diablo: LastHeroOptions,
    pub hellfire: LastHeroOptions,
    pub audio: AudioOptions,
    pub gameplay: GameplayOptions,
    pub graphics: GraphicsOptions,
    pub controller: ControllerOptions,
    pub network: NetworkOptions,
    pub chat: ChatOptions,
    pub language: LanguageOptions,
    pub keymapper: KeymapperOptions,
    pub padmapper: PadmapperOptions,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            start_up: StartUpOptions::new(),
            diablo: LastHeroOptions::new(false),
            hellfire: LastHeroOptions::new(true),
            audio: AudioOptions::new(),
            gameplay: GameplayOptions::new(),
            graphics: GraphicsOptions::new(),
            controller: ControllerOptions::new(),
            network: NetworkOptions::new(),
            chat: ChatOptions::new(),
            language: LanguageOptions::new(),
            keymapper: KeymapperOptions::new(),
            padmapper: PadmapperOptions::new(),
        }
    }
}

/// The categories in `Options::GetCategories` order, by key.
// @port options.h|devilution::Options::GetCategories() sha=22a6dd8ef141
pub const CATEGORY_ORDER: [&str; 12] =
    ["Language", "StartUp", "Graphics", "Audio", "Diablo", "Hellfire", "Game", "Controller", "Network", "NetMsg", "Keymapping", "Padmapping"];

impl Options {
    /// `GetCategories()[i]` by key.
    pub fn category(&self, key: &str) -> &OptionCategoryBase {
        match key {
            "Language" => &self.language.category,
            "StartUp" => &self.start_up.category,
            "Graphics" => &self.graphics.category,
            "Audio" => &self.audio.category,
            "Diablo" => &self.diablo.category,
            "Hellfire" => &self.hellfire.category,
            "Game" => &self.gameplay.category,
            "Controller" => &self.controller.category,
            "Network" => &self.network.category,
            "NetMsg" => &self.chat.category,
            "Keymapping" => &self.keymapper.category,
            "Padmapping" => &self.padmapper.category,
            _ => panic!("unknown option category {key}"),
        }
    }

    /// `GetCategories()[i]->GetEntries()` for the plain (non-mapper) categories.
    pub fn entries_of(&mut self, category: &str) -> Vec<&mut dyn OptionEntry> {
        match category {
            "Language" => self.language.get_entries(),
            "StartUp" => self.start_up.get_entries(),
            "Graphics" => self.graphics.get_entries(),
            "Audio" => self.audio.get_entries(),
            "Diablo" => self.diablo.get_entries(),
            "Hellfire" => self.hellfire.get_entries(),
            "Game" => self.gameplay.get_entries(),
            "Controller" => self.controller.get_entries(),
            "Network" => self.network.get_entries(),
            "NetMsg" => self.chat.get_entries(),
            _ => vec![],
        }
    }
}

/// Original: `LoadOptions` (options.cpp).
// @port options.cpp|devilution::LoadOptions() sha=7142205146c5
pub fn load_options(ctx: &mut Ctx) {
    for category in CATEGORY_ORDER {
        match category {
            "Language" => load_language_code(ctx, category),
            "Keymapping" => {
                let mut io = IniIo { ini: &mut ctx.options_ini, paths: &mut ctx.paths };
                for i in 0..ctx.options.keymapper.actions.len() {
                    ctx.options.keymapper.load_action(&mut io, i, category);
                }
            }
            "Padmapping" => {
                let mut io = IniIo { ini: &mut ctx.options_ini, paths: &mut ctx.paths };
                for i in 0..ctx.options.padmapper.actions.len() {
                    ctx.options.padmapper.load_action(&mut io, i, category);
                }
            }
            _ => {
                let mut io = IniIo { ini: &mut ctx.options_ini, paths: &mut ctx.paths };
                for e in ctx.options.entries_of(category) {
                    e.load_from_ini(&mut io, category);
                }
                if category == "Audio" {
                    update_dependent_options(&mut ctx.options.audio);
                }
            }
        }
    }
    let mut io = IniIo { ini: &mut ctx.options_ini, paths: &mut ctx.paths };
    let o = &mut ctx.options;
    o.hellfire.sz_item = io.get_value("Hellfire", "SItem", crate::pack::ITEM_PACK_SIZE * 2 + 1, "").1;
    o.network.sz_bind_address = io.get_value("Network", "Bind Address", 129, "0.0.0.0").1;
    o.network.sz_previous_zt_game = io.get_value("Network", "Previous Game ID", 129, "").1;
    o.network.sz_previous_host = io.get_value("Network", "Previous Host", 129, "").1;
    for i in 0..QUICK_MESSAGE_OPTIONS {
        io.get_string_vector("NetMsg", crate::diablo::QUICK_MESSAGES[i].key, &mut o.chat.sz_hot_key_msgs[i]);
    }
    o.controller.sz_mapping = io.get_value("Controller", "Mapping", 1024, "").1;
    o.controller.f_deadzone = io.get_float("Controller", "deadzone", 0.07);
    if crate::engine::demomode::is_running(ctx) {
        crate::engine::demomode::override_options(ctx);
    }
}

/// Original: `SaveOptions` (options.cpp).
// @port options.cpp|devilution::SaveOptions() sha=9f04e13d7538
pub fn save_options(ctx: &mut Ctx) {
    if crate::engine::demomode::is_running(ctx) {
        return;
    }
    let mut io = IniIo { ini: &mut ctx.options_ini, paths: &mut ctx.paths };
    let o = &mut ctx.options;
    for category in CATEGORY_ORDER {
        match category {
            "Keymapping" => {
                for i in 0..o.keymapper.actions.len() {
                    o.keymapper.save_action(&mut io, i, category);
                }
            }
            "Padmapping" => {
                for i in 0..o.padmapper.actions.len() {
                    o.padmapper.save_action(&mut io, i, category);
                }
            }
            _ => {
                for e in o.entries_of(category) {
                    e.save_to_ini(&mut io, category);
                }
            }
        }
    }
    io.set_str("Hellfire", "SItem", &o.hellfire.sz_item);
    io.set_str("Network", "Bind Address", &o.network.sz_bind_address);
    io.set_str("Network", "Previous Game ID", &o.network.sz_previous_zt_game);
    io.set_str("Network", "Previous Host", &o.network.sz_previous_host);
    for i in 0..QUICK_MESSAGE_OPTIONS {
        io.set_string_vector("NetMsg", crate::diablo::QUICK_MESSAGES[i].key, &o.chat.sz_hot_key_msgs[i]);
    }
    io.set_str("Controller", "Mapping", &o.controller.sz_mapping);
    io.set_float("Controller", "deadzone", o.controller.f_deadzone);
    save_ini(&mut ctx.options_ini, &mut ctx.paths);
}

/// Runs a value-changed callback (the functions in options.cpp's anonymous namespace and the
/// display functions registered in the category constructors).
pub fn run_option_callback(ctx: &mut Ctx, cb: OptionCallback) {
    match cb {
        OptionCallback::GameModeChanged => option_game_mode_changed(ctx),
        OptionCallback::SharewareChanged => option_shareware_changed(ctx),
        OptionCallback::AudioChanged => option_audio_changed(ctx),
        OptionCallback::ResizeWindow => crate::utils::display::resize_window(ctx),
        OptionCallback::SetFullscreenMode => crate::utils::display::set_fullscreen_mode(ctx),
        OptionCallback::ResizeWindowAndUpdateResolutionOptions => resize_window_and_update_resolution_options(ctx),
        OptionCallback::ReinitializeTexture => crate::utils::display::reinitialize_texture(ctx),
        OptionCallback::ReinitializeIntegerScale => crate::utils::display::reinitialize_integer_scale(ctx),
        OptionCallback::ReinitializeRenderer => crate::utils::display::reinitialize_renderer(ctx),
        OptionCallback::ShowFpsChanged => option_show_fps_changed(ctx),
        OptionCallback::GrabInputChanged => option_grab_input_changed(ctx),
        OptionCallback::ExperienceBarChanged => option_experience_bar_changed(ctx),
        OptionCallback::EnemyHealthBarChanged => option_enemy_health_bar_changed(ctx),
        OptionCallback::LanguageCodeChanged => option_language_code_changed(ctx),
    }
}

/// Original: `OptionGrabInputChanged` (options.cpp).
// @port options.cpp|devilution::OptionGrabInputChanged() sha=fc8fdd1ca9bb
fn option_grab_input_changed(ctx: &mut Ctx) {
    let grab = ctx.options.gameplay.grab_input.get();
    ctx.platform.set_window_grab(grab);
}

/// Original: `OptionExperienceBarChanged` (options.cpp).
// @port options.cpp|devilution::OptionExperienceBarChanged() sha=17c55eeba177
fn option_experience_bar_changed(ctx: &mut Ctx) {
    if !ctx.diablo.gb_run_game {
        return;
    }
    if ctx.options.gameplay.experience_bar.get() {
        crate::qol::xpbar::init_xp_bar(ctx);
    } else {
        crate::qol::xpbar::free_xp_bar(ctx);
    }
}

/// Original: `OptionEnemyHealthBarChanged` (options.cpp).
// @port options.cpp|devilution::OptionEnemyHealthBarChanged() sha=91bbe4936eab
fn option_enemy_health_bar_changed(ctx: &mut Ctx) {
    if !ctx.diablo.gb_run_game {
        return;
    }
    if ctx.options.gameplay.enemy_health_bar.get() {
        crate::qol::monhealthbar::init_monster_health_bar(ctx);
    } else {
        crate::qol::monhealthbar::free_monster_health_bar(ctx);
    }
}

/// Original: `ResizeWindowAndUpdateResolutionOptions` (options.cpp).
// @port options.cpp|devilution::ResizeWindowAndUpdateResolutionOptions() sha=e7e11fbedbde
fn resize_window_and_update_resolution_options(ctx: &mut Ctx) {
    crate::utils::display::resize_window(ctx);
    ctx.options.graphics.resolution.invalidate_list();
}

/// Original: `OptionShowFPSChanged` (options.cpp).
// @port options.cpp|devilution::OptionShowFPSChanged() sha=9a3411681b5d
fn option_show_fps_changed(ctx: &mut Ctx) {
    if ctx.options.graphics.show_fps.get() {
        crate::diablo::enable_frame_count(ctx);
    } else {
        ctx.diablo.frameflag = false;
    }
}

/// Original: `OptionLanguageCodeChanged` (options.cpp).
// @port options.cpp|devilution::OptionLanguageCodeChanged() sha=9f74953f19d8
fn option_language_code_changed(ctx: &mut Ctx) {
    crate::utils::language::language_initialize(ctx);
    crate::init::load_language_archive(ctx);
}

/// Original: `OptionGameModeChanged` (options.cpp).
// @port options.cpp|devilution::OptionGameModeChanged() sha=b62c7523f8f4
fn option_game_mode_changed(ctx: &mut Ctx) {
    ctx.init.gb_is_hellfire = ctx.options.start_up.game_mode() == StartUpGameMode::Hellfire;
    // discord_manager::UpdateMenu(true): Discord integration is off in the port (replaced).
}

/// Original: `OptionSharewareChanged` (options.cpp).
// @port options.cpp|devilution::OptionSharewareChanged() sha=82f445edd359
fn option_shareware_changed(ctx: &mut Ctx) {
    ctx.init.gb_is_spawn = ctx.options.start_up.shareware.get();
}

/// Original: `OptionAudioChanged` (options.cpp).
// @port options.cpp|devilution::OptionAudioChanged() sha=4a103a354714
fn option_audio_changed(ctx: &mut Ctx) {
    crate::effects::effects_cleanup_sfx(ctx);
    crate::engine::sound::music_stop(ctx);
    crate::engine::sound::snd_deinit(ctx);
    crate::engine::sound::snd_init(ctx);
    crate::engine::sound::music_start(ctx, crate::engine::sound::TMUSIC_INTRO);
    if ctx.diablo.gb_run_game {
        crate::effects::sound_init(ctx);
    } else {
        crate::effects::ui_sound_init(ctx);
    }
}
