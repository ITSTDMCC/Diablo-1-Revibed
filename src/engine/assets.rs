//! `Source/engine/assets.cpp` / `assets.hpp`: finding and opening game files across the
//! loaded MPQ archives, the pref-path override folder and the `assets` folder.

use std::io::Read;
use std::path::PathBuf;

use crate::ctx::Ctx;
use crate::mpq::{MpqArchive, MpqError};
use crate::platform::log;

/// The archives `init.cpp` loads, in the original's globals (`diabdat_mpq`, ...).
#[derive(Default)]
pub struct Archives {
    pub spawn_mpq: Option<MpqArchive>,
    pub diabdat_mpq: Option<MpqArchive>,
    pub hellfire_mpq: Option<MpqArchive>,
    pub hfmonk_mpq: Option<MpqArchive>,
    pub hfbard_mpq: Option<MpqArchive>,
    pub hfbarb_mpq: Option<MpqArchive>,
    pub hfmusic_mpq: Option<MpqArchive>,
    pub hfvoice_mpq: Option<MpqArchive>,
    pub devilutionx_mpq: Option<MpqArchive>,
    pub lang_mpq: Option<MpqArchive>,
    pub font_mpq: Option<MpqArchive>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveId {
    Font,
    Lang,
    DevilutionX,
    HfVoice,
    HfMusic,
    HfBarb,
    HfBard,
    HfMonk,
    Hellfire,
    Spawn,
    Diabdat,
}

impl Archives {
    pub fn get_mut(&mut self, id: ArchiveId) -> Option<&mut MpqArchive> {
        match id {
            ArchiveId::Font => self.font_mpq.as_mut(),
            ArchiveId::Lang => self.lang_mpq.as_mut(),
            ArchiveId::DevilutionX => self.devilutionx_mpq.as_mut(),
            ArchiveId::HfVoice => self.hfvoice_mpq.as_mut(),
            ArchiveId::HfMusic => self.hfmusic_mpq.as_mut(),
            ArchiveId::HfBarb => self.hfbarb_mpq.as_mut(),
            ArchiveId::HfBard => self.hfbard_mpq.as_mut(),
            ArchiveId::HfMonk => self.hfmonk_mpq.as_mut(),
            ArchiveId::Hellfire => self.hellfire_mpq.as_mut(),
            ArchiveId::Spawn => self.spawn_mpq.as_mut(),
            ArchiveId::Diabdat => self.diabdat_mpq.as_mut(),
        }
    }

    fn get(&self, id: ArchiveId) -> Option<&MpqArchive> {
        match id {
            ArchiveId::Font => self.font_mpq.as_ref(),
            ArchiveId::Lang => self.lang_mpq.as_ref(),
            ArchiveId::DevilutionX => self.devilutionx_mpq.as_ref(),
            ArchiveId::HfVoice => self.hfvoice_mpq.as_ref(),
            ArchiveId::HfMusic => self.hfmusic_mpq.as_ref(),
            ArchiveId::HfBarb => self.hfbarb_mpq.as_ref(),
            ArchiveId::HfBard => self.hfbard_mpq.as_ref(),
            ArchiveId::HfMonk => self.hfmonk_mpq.as_ref(),
            ArchiveId::Hellfire => self.hellfire_mpq.as_ref(),
            ArchiveId::Spawn => self.spawn_mpq.as_ref(),
            ArchiveId::Diabdat => self.diabdat_mpq.as_ref(),
        }
    }

    /// Original: `FindMpqFile` (engine/assets.cpp): search order font, lang, devilutionx,
    /// then (Hellfire only) hfvoice, hfmusic, hfbarb, hfbard, hfmonk, hellfire, then spawn, diabdat.
    // @port engine/assets.cpp|devilution::FindMpqFile(const char *filename, MpqArchive **archive, uint32_t *fileNumber) sha=edd9d35eefc9
    fn find_mpq_file(&self, filename: &str, is_hellfire: bool) -> Option<(ArchiveId, u32)> {
        let hash = MpqArchive::calculate_file_hash(filename);
        let at = |id: ArchiveId| self.get(id).and_then(|a| a.get_file_number(hash)).map(|n| (id, n));
        at(ArchiveId::Font)
            .or_else(|| at(ArchiveId::Lang))
            .or_else(|| at(ArchiveId::DevilutionX))
            .or_else(|| {
                if !is_hellfire {
                    return None;
                }
                at(ArchiveId::HfVoice)
                    .or_else(|| at(ArchiveId::HfMusic))
                    .or_else(|| at(ArchiveId::HfBarb))
                    .or_else(|| at(ArchiveId::HfBard))
                    .or_else(|| at(ArchiveId::HfMonk))
                    .or_else(|| at(ArchiveId::Hellfire))
            })
            .or_else(|| at(ArchiveId::Spawn))
            .or_else(|| at(ArchiveId::Diabdat))
    }
}

/// `AssetRef`: an MPQ file, or a plain file that overrides/extends the archives.
#[derive(Debug, Clone)]
pub enum AssetRef {
    None,
    Mpq { archive: ArchiveId, file_number: u32, filename: String },
    File(PathBuf),
}

impl AssetRef {
    pub fn ok(&self) -> bool {
        !matches!(self, AssetRef::None)
    }
}

/// `AssetHandle`: an opened asset. Files are read whole when opened (the original streams MPQ
/// files through SDL_RWops block by block; the bytes delivered are the same).
pub struct AssetHandle {
    data: Vec<u8>,
    pos: usize,
    error: Option<String>,
}

impl AssetHandle {
    pub fn ok(&self) -> bool {
        self.error.is_none()
    }

    pub fn error(&self) -> &str {
        self.error.as_deref().unwrap_or("")
    }

    // @port engine/assets.hpp|devilution::AssetRef::size() sha=a738d2da1952
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// `read`: exactly `buf.len()` bytes or false.
    // @port engine/assets.hpp|devilution::AssetHandle::read(void *buffer, size_t len) sha=bbc679e4b5f1
    pub fn read(&mut self, buf: &mut [u8]) -> bool {
        if self.pos + buf.len() > self.data.len() {
            return false;
        }
        buf.copy_from_slice(&self.data[self.pos..self.pos + buf.len()]);
        self.pos += buf.len();
        true
    }

    // @port engine/assets.hpp|devilution::AssetHandle::seek(long pos) sha=337306143280
    pub fn seek(&mut self, pos: usize) -> bool {
        if pos > self.data.len() {
            return false;
        }
        self.pos = pos;
        true
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }
}

fn file_exists(p: &PathBuf) -> bool {
    p.is_file()
}

/// Original: `devilution::FindAsset` (engine/assets.cpp).
// @port engine/assets.cpp|devilution::FindAsset(const char *filename) sha=49047728462d
pub fn find_asset(ctx: &mut Ctx, filename: &str) -> AssetRef {
    let relative_path = filename.to_string();
    if relative_path.starts_with('/') {
        let p = PathBuf::from(&relative_path);
        if file_exists(&p) {
            return AssetRef::File(p);
        }
    }
    // Files in the `PrefPath()` directory can override MPQ contents.
    let pref = PathBuf::from(format!("{}{}", ctx.paths.pref_path(), relative_path));
    if file_exists(&pref) {
        log::verbose!("Loaded MPQ file override: {}", pref.display());
        return AssetRef::File(pref);
    }
    // Look for the file in all the MPQ archives:
    if let Some((archive, file_number)) = ctx.init.archives.find_mpq_file(filename, ctx.init.gb_is_hellfire) {
        return AssetRef::Mpq { archive, file_number, filename: filename.to_string() };
    }
    // Load from the `/assets` directory next to the binary.
    let assets = PathBuf::from(format!("{}{}", ctx.paths.assets_path(), relative_path));
    if file_exists(&assets) {
        return AssetRef::File(assets);
    }
    AssetRef::None
}

/// Size of a found asset (`AssetRef::size`).
pub fn asset_size(ctx: &mut Ctx, r: &AssetRef) -> usize {
    match r {
        AssetRef::None => 0,
        AssetRef::Mpq { archive, file_number, .. } => {
            ctx.init.archives.get_mut(*archive).and_then(|a| a.unpacked_file_size(*file_number).ok()).unwrap_or(0)
        }
        AssetRef::File(p) => std::fs::metadata(p).map(|m| m.len() as usize).unwrap_or(0),
    }
}

/// Original: `devilution::OpenAsset(AssetRef &&ref, bool threadsafe)` (engine/assets.cpp).
// @port engine/assets.cpp|devilution::OpenAsset(AssetRef &&ref, bool threadsafe) sha=2dd9c916e9f9
pub fn open_asset_ref(ctx: &mut Ctx, r: AssetRef) -> AssetHandle {
    let result: Result<Vec<u8>, String> = match r {
        AssetRef::None => Err("File not found".to_string()),
        AssetRef::Mpq { archive, filename, .. } => match ctx.init.archives.get_mut(archive) {
            Some(a) => a.read_file(&filename).map_err(|e: MpqError| e.to_string()),
            None => Err("archive closed".to_string()),
        },
        AssetRef::File(p) => {
            let mut v = Vec::new();
            std::fs::File::open(&p).and_then(|mut f| f.read_to_end(&mut v)).map(|_| v).map_err(|e| e.to_string())
        }
    };
    match result {
        Ok(data) => AssetHandle { data, pos: 0, error: None },
        Err(e) => AssetHandle { data: Vec::new(), pos: 0, error: Some(e) },
    }
}

/// Original: `devilution::OpenAsset(const char *filename, bool threadsafe)` (engine/assets.cpp).
// @port engine/assets.cpp|devilution::OpenAsset(const char *filename, bool threadsafe) sha=294a1e986bb7
pub fn open_asset(ctx: &mut Ctx, filename: &str) -> AssetHandle {
    let r = find_asset(ctx, filename);
    if !r.ok() {
        return AssetHandle { data: Vec::new(), pos: 0, error: Some("File not found".to_string()) };
    }
    open_asset_ref(ctx, r)
}

/// Original: `devilution::FailedToOpenFileError` (engine/assets.hpp).
// @port engine/assets.hpp|devilution::FailedToOpenFileError(const char *path, const char *error) sha=8cf012357ecb
pub fn failed_to_open_file_error(ctx: &mut Ctx, path: &str, error: &str) -> ! {
    crate::appfat::app_fatal(ctx, &format!("Failed to open file:\n{path}\n\n{error}"))
}

/// Original: `devilution::ValidateHandle` (engine/assets.hpp).
// @port engine/assets.hpp|devilution::ValidateHandle(const char *path, const AssetHandle &handle) sha=8307fd448e58
pub fn validate_handle(ctx: &mut Ctx, path: &str, handle: &AssetHandle) -> bool {
    if handle.ok() {
        return true;
    }
    if !ctx.diablo.headless_mode {
        let e = handle.error().to_string();
        failed_to_open_file_error(ctx, path, &e);
    }
    false
}
