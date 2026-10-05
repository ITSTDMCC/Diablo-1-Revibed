//! `Source/pfile.cpp`: hero and save game files (MPQ archives in the preferences folder).

use crate::codec::{codec_decode, codec_encode, codec_get_encoded_len};
use crate::ctx::Ctx;
use crate::enums::*;
use crate::mpq::MpqArchive;
use crate::mpq_writer::SaveWriter;
use crate::pack::{pack_player, unpack_player, PlayerPack, PLAYER_PACK_SIZE};
use crate::player::{MaxBeltItems, PlayerNameLength};
use crate::tables::playerdat::PlayersData;
use crate::utils::language::tr;

pub const MAX_CHARACTERS: usize = 99;

const PASSWORD_SPAWN_SINGLE: &str = "adslhfb1";
const PASSWORD_SPAWN_MULTI: &str = "lshbkfg1";
const PASSWORD_SINGLE: &str = "xrgyrkj1";
const PASSWORD_MULTI: &str = "szqnlsk1";

/// `_uiheroinfo` (DiabloUI/diabloui.h)
#[derive(Clone, Copy, Debug, Default)]
pub struct UiHeroInfo {
    pub saveNumber: u32,
    pub name: [u8; 16],
    pub level: u8,
    pub heroclass: HeroClass,
    pub herorank: u8,
    pub strength: u16,
    pub magic: u16,
    pub dexterity: u16,
    pub vitality: u16,
    pub hassaved: bool,
    pub spawned: bool,
}

impl UiHeroInfo {
    pub fn name_str(&self) -> &str {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(16);
        std::str::from_utf8(&self.name[..end]).unwrap_or("")
    }

    /// `CopyUtf8(heroinfo->name, s, sizeof(heroinfo->name))`
    pub fn set_name(&mut self, s: &str) {
        let t = crate::utils::utf8::copy_utf8(s, 16);
        self.name = [0; 16];
        self.name[..t.len()].copy_from_slice(t.as_bytes());
    }
}

/// `_uidefaultstats` (DiabloUI/diabloui.h)
#[derive(Clone, Copy, Debug, Default)]
pub struct UiDefaultStats {
    pub strength: u16,
    pub magic: u16,
    pub dexterity: u16,
    pub vitality: u16,
}

/// Globals of pfile.cpp.
pub struct PfileState {
    pub gbValidSaveFile: bool,
    /// `hero_names`
    hero_names: Vec<[u8; PlayerNameLength]>,
    /// `prevTick` of `pfile_update`
    prev_tick: u32,
}

impl Default for PfileState {
    fn default() -> Self {
        PfileState { gbValidSaveFile: false, hero_names: vec![[0; PlayerNameLength]; MAX_CHARACTERS], prev_tick: 0 }
    }
}

/// `gbValidSaveFile`
pub fn gb_valid_save_file(ctx: &Ctx) -> bool {
    ctx.pfile.gbValidSaveFile
}

/// Original: `GetSavePath` (pfile.cpp).
// @port pfile.cpp|devilution::GetSavePath(uint32_t saveNum, string_view savePrefix = {}) sha=1b839c40731a
fn get_save_path(ctx: &mut Ctx, save_num: u32, save_prefix: &str) -> String {
    let kind = if ctx.init.gb_is_spawn {
        if ctx.init.gb_is_multiplayer {
            "share_"
        } else {
            "spawn_"
        }
    } else if ctx.init.gb_is_multiplayer {
        "multi_"
    } else {
        "single_"
    };
    let ext = if ctx.init.gb_is_hellfire { ".hsv" } else { ".sv" };
    format!("{}{}{}{}{}", ctx.paths.pref_path(), save_prefix, kind, save_num, ext)
}

/// Original: `GetStashSavePath` (pfile.cpp).
// @port pfile.cpp|devilution::GetStashSavePath() sha=2e6b78ccc3a0
fn get_stash_save_path(ctx: &mut Ctx) -> String {
    let base = if ctx.init.gb_is_spawn { "stash_spawn" } else { "stash" };
    let ext = if ctx.init.gb_is_hellfire { ".hsv" } else { ".sv" };
    format!("{}{}{}", ctx.paths.pref_path(), base, ext)
}

/// Original: `GetSaveNames` (pfile.cpp).
// @port pfile.cpp|devilution::GetSaveNames(uint8_t index, string_view prefix, char *out) sha=40a8533d251b
fn get_save_names(number_of_levels: u8, mut index: u8, prefix: &str) -> Option<String> {
    let suf;
    if index < number_of_levels {
        suf = 'l';
    } else if (index as u32) < number_of_levels as u32 * 2 {
        index -= number_of_levels;
        suf = 's';
    } else {
        return None;
    }
    Some(format!("{prefix}{suf}{index:02}"))
}

/// Original: `GetPermSaveNames` (pfile.cpp).
// @port pfile.cpp|devilution::GetPermSaveNames(uint8_t dwIndex, char *szPerm) sha=9bb44cfb73d9
fn get_perm_save_names(number_of_levels: u8, index: u8) -> Option<String> {
    get_save_names(number_of_levels, index, "perm")
}

/// Original: `GetTempSaveNames` (pfile.cpp).
// @port pfile.cpp|devilution::GetTempSaveNames(uint8_t dwIndex, char *szTemp) sha=fd07208121d1
fn get_temp_save_names(number_of_levels: u8, index: u8) -> Option<String> {
    get_save_names(number_of_levels, index, "temp")
}

/// Original: `RenameTempToPerm` (pfile.cpp).
// @port pfile.cpp|devilution::RenameTempToPerm(SaveWriter &saveWriter) sha=154901db435a
fn rename_temp_to_perm(ctx: &Ctx, save_writer: &mut SaveWriter) {
    let n = ctx.loadsave.giNumberOfLevels;
    let mut index: u8 = 0;
    while let Some(sz_temp) = get_temp_save_names(n, index) {
        let sz_perm = get_perm_save_names(n, index).expect("perm name");
        index += 1;
        if save_writer.has_file(&sz_temp) {
            if save_writer.has_file(&sz_perm) {
                save_writer.remove_hash_entry(&sz_perm);
            }
            save_writer.rename_file(&sz_temp, &sz_perm);
        }
    }
    assert!(get_perm_save_names(n, index).is_none());
}

/// Original: `ReadHero` (pfile.cpp).
// @port pfile.cpp|devilution::ReadHero(SaveReader &archive, PlayerPack *pPack) sha=71459cdd33e2
fn read_hero(ctx: &Ctx, archive: &mut MpqArchive) -> Option<PlayerPack> {
    let buf = read_archive(ctx, archive, "hero")?;
    if buf.len() == PLAYER_PACK_SIZE {
        return Some(PlayerPack::from_bytes(&buf));
    }
    None
}

/// Original: `EncodeHero` (pfile.cpp).
// @port pfile.cpp|devilution::EncodeHero(SaveWriter &saveWriter, const PlayerPack *pack) sha=2d08690a8649
fn encode_hero(ctx: &Ctx, save_writer: &mut SaveWriter, pack: &PlayerPack) {
    let mut packed = pack.to_bytes();
    debug_assert_eq!(codec_get_encoded_len(PLAYER_PACK_SIZE), codec_get_encoded_len(packed.len()));
    let len = packed.len();
    codec_encode(&mut packed, len, pfile_get_password(ctx));
    save_writer.write_file("hero", &packed);
}

/// Original: `GetSaveWriter` (pfile.cpp).
// @port pfile.cpp|devilution::GetSaveWriter(uint32_t saveNum) sha=93bc2614babc
fn get_save_writer(ctx: &mut Ctx, save_num: u32) -> SaveWriter {
    let path = get_save_path(ctx, save_num, "");
    SaveWriter::new(ctx, &path)
}

/// Original: `GetStashWriter` (pfile.cpp).
// @port pfile.cpp|devilution::GetStashWriter() sha=3d0b4366c83c
fn get_stash_writer(ctx: &mut Ctx) -> SaveWriter {
    let path = get_stash_save_path(ctx);
    SaveWriter::new(ctx, &path)
}

/// Original: `CopySaveFile` (pfile.cpp).
// @port pfile.cpp|devilution::CopySaveFile(uint32_t saveNum, std::string targetPath) sha=ab45a517f0d7
fn copy_save_file(ctx: &mut Ctx, save_num: u32, target_path: &str) {
    let save_path = get_save_path(ctx, save_num, "");
    let _ = std::fs::copy(&save_path, target_path);
}

/// Original: `Game2UiPlayer` (pfile.cpp).
// @port pfile.cpp|devilution::Game2UiPlayer(const Player &player, _uiheroinfo *heroinfo, bool bHasSaveFile) sha=8d969539e7cb
fn game2_ui_player(ctx: &Ctx, pnum: usize, heroinfo: &mut UiHeroInfo, has_save_file: bool) {
    let player = &ctx.players.Players[pnum];
    heroinfo.set_name(player._pName.as_str());
    heroinfo.level = player._pLevel as u8;
    heroinfo.heroclass = player._pClass;
    heroinfo.strength = player._pStrength as u16;
    heroinfo.magic = player._pMagic as u16;
    heroinfo.dexterity = player._pDexterity as u16;
    heroinfo.vitality = player._pVitality as u16;
    heroinfo.hassaved = has_save_file;
    heroinfo.herorank = player.pDiabloKillLevel;
    heroinfo.spawned = ctx.init.gb_is_spawn;
}

/// Original: `GetFileName` (pfile.cpp).
// @port pfile.cpp|devilution::GetFileName(uint8_t lvl, char *dst) sha=d18027a599bf
fn get_file_name(is_multiplayer: bool, number_of_levels: u8, lvl: u8) -> Option<String> {
    if is_multiplayer {
        if lvl != 0 {
            return None;
        }
        return Some("hero".to_string());
    }
    if let Some(name) = get_perm_save_names(number_of_levels, lvl) {
        return Some(name);
    }
    if lvl as u32 == number_of_levels as u32 * 2 {
        return Some("game".to_string());
    }
    if lvl as u32 == number_of_levels as u32 * 2 + 1 {
        return Some("hero".to_string());
    }
    None
}

/// Original: `ArchiveContainsGame` (pfile.cpp).
// @port pfile.cpp|devilution::ArchiveContainsGame(SaveReader &hsArchive) sha=73ca4352f02e
fn archive_contains_game(ctx: &mut Ctx, archive: &mut MpqArchive) -> bool {
    if ctx.init.gb_is_multiplayer {
        return false;
    }
    let Some(game_data) = read_archive(ctx, archive, "game") else {
        return false;
    };
    let hdr = u32::from_le_bytes([game_data[0], game_data[1], game_data[2], game_data[3]]);
    crate::loadsave::is_header_valid(ctx, hdr)
}

/// Original: `CreateSaveReader` (pfile.cpp).
// @port pfile.cpp|devilution::CreateSaveReader(std::string &&path) sha=bff0e63daba4
fn create_save_reader(path: &str) -> Option<MpqArchive> {
    MpqArchive::open(path).ok().flatten()
}

/// Original: `pfile_write_hero(SaveWriter &saveWriter, bool writeGameData)` (pfile.cpp).
// @port pfile.cpp|devilution::pfile_write_hero(SaveWriter &saveWriter, bool writeGameData) sha=fbfd6191f60c
fn pfile_write_hero_to(ctx: &mut Ctx, save_writer: &mut SaveWriter, write_game_data: bool) {
    if write_game_data {
        crate::loadsave::save_game_data(ctx, save_writer);
        rename_temp_to_perm(ctx, save_writer);
    }
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let pkplr = pack_player(ctx, &ctx.players.Players[me]);
    encode_hero(ctx, save_writer, &pkplr);
    if !ctx.init.gb_vanilla {
        let player = ctx.players.Players[me].clone();
        crate::loadsave::save_hotkeys(ctx, save_writer, &player);
        crate::loadsave::save_hero_items(ctx, save_writer, &player);
    }
}

/// Original: `RemoveAllInvalidItems` (pfile.cpp).
// @port pfile.cpp|devilution::RemoveAllInvalidItems(Player &player) sha=2269415f2f32
fn remove_all_invalid_items(ctx: &mut Ctx, pnum: usize) {
    let mut player = std::mem::take(&mut ctx.players.Players[pnum]);
    for item in player.InvBody.iter_mut() {
        crate::loadsave::remove_invalid_item(ctx, item);
    }
    for i in 0..player._pNumInv as usize {
        crate::loadsave::remove_invalid_item(ctx, &mut player.InvList[i]);
    }
    for i in 0..MaxBeltItems {
        crate::loadsave::remove_invalid_item(ctx, &mut player.SpdList[i]);
    }
    ctx.players.Players[pnum] = player;
    crate::loadsave::remove_empty_inventory(ctx, pnum);
}

/// Original: `devilution::OpenSaveArchive` (pfile.cpp).
// @port pfile.cpp|devilution::OpenSaveArchive(uint32_t saveNum) sha=6a8759dc5a56
pub fn open_save_archive(ctx: &mut Ctx, save_num: u32) -> Option<MpqArchive> {
    let path = get_save_path(ctx, save_num, "");
    create_save_reader(&path)
}

/// Original: `devilution::OpenStashArchive` (pfile.cpp).
// @port pfile.cpp|devilution::OpenStashArchive() sha=5d477a88325f
pub fn open_stash_archive(ctx: &mut Ctx) -> Option<MpqArchive> {
    let path = get_stash_save_path(ctx);
    create_save_reader(&path)
}

/// Original: `devilution::ReadArchive` (pfile.cpp). Returns the decoded record.
// @port pfile.cpp|devilution::ReadArchive(SaveReader &archive, const char *pszName, size_t *pdwLen) sha=c8d54c79c26f
pub fn read_archive(ctx: &Ctx, archive: &mut MpqArchive, name: &str) -> Option<Vec<u8>> {
    let mut result = archive.read_file(name).ok()?;
    let decoded_length = codec_decode(&mut result, pfile_get_password(ctx));
    if decoded_length == 0 {
        return None;
    }
    result.truncate(decoded_length);
    Some(result)
}

/// Original: `devilution::pfile_get_password` (pfile.cpp).
// @port pfile.cpp|devilution::pfile_get_password() sha=d153ae028f9d
pub fn pfile_get_password(ctx: &Ctx) -> &'static str {
    if ctx.init.gb_is_spawn {
        return if ctx.init.gb_is_multiplayer { PASSWORD_SPAWN_MULTI } else { PASSWORD_SPAWN_SINGLE };
    }
    if ctx.init.gb_is_multiplayer {
        PASSWORD_MULTI
    } else {
        PASSWORD_SINGLE
    }
}

/// Original: `devilution::pfile_write_hero(bool writeGameData)` (pfile.cpp).
// @port pfile.cpp|devilution::pfile_write_hero(bool writeGameData) sha=7bf11894f82a
pub fn pfile_write_hero(ctx: &mut Ctx, write_game_data: bool) {
    let mut save_writer = get_save_writer(ctx, ctx.menu.g_save_number);
    pfile_write_hero_to(ctx, &mut save_writer, write_game_data);
}

/// Original: `devilution::pfile_write_hero_demo` (pfile.cpp).
// @port pfile.cpp|devilution::pfile_write_hero_demo(int demo) sha=c71e95c3aa4d
pub fn pfile_write_hero_demo(ctx: &mut Ctx, demo: i32) {
    let save_path = get_save_path(ctx, ctx.menu.g_save_number, &format!("demo_{demo}_reference_"));
    copy_save_file(ctx, ctx.menu.g_save_number, &save_path);
    let mut save_writer = SaveWriter::new(ctx, &save_path);
    pfile_write_hero_to(ctx, &mut save_writer, true);
}

crate::pending_fn!(pub fn pfile_compare_hero_demo(ctx: &mut Ctx, demo: i32, log_details: bool) -> (u8, String), "pfile.cpp|devilution::pfile_compare_hero_demo(int demo, bool logDetails)");

/// Original: `devilution::sfile_write_stash` (pfile.cpp).
// @port pfile.cpp|devilution::sfile_write_stash() sha=3e5f104c91c5
pub fn sfile_write_stash(ctx: &mut Ctx) {
    if !ctx.stash.Stash.dirty {
        return;
    }
    let mut stash_writer = get_stash_writer(ctx);
    crate::loadsave::save_stash(ctx, &mut stash_writer);
    ctx.stash.Stash.dirty = false;
}

/// Original: `devilution::pfile_ui_set_hero_infos` (pfile.cpp).
// @port pfile.cpp|devilution::pfile_ui_set_hero_infos(bool (*uiAddHeroInfo)(_uiheroinfo *)) sha=b1443af44518
pub fn pfile_ui_set_hero_infos(ctx: &mut Ctx, ui_add_hero_info: &mut dyn FnMut(&mut Ctx, &UiHeroInfo) -> bool) -> bool {
    for name in ctx.pfile.hero_names.iter_mut() {
        *name = [0; PlayerNameLength];
    }
    for i in 0..MAX_CHARACTERS as u32 {
        let Some(mut archive) = open_save_archive(ctx, i) else { continue };
        let Some(mut pkplr) = read_hero(ctx, &mut archive) else { continue };
        let mut uihero = UiHeroInfo { saveNumber: i, ..Default::default() };
        // strcpy(hero_names[i], pkplr.pName)
        let end = pkplr.pName.iter().position(|&b| b == 0).unwrap_or(PlayerNameLength);
        ctx.pfile.hero_names[i as usize] = [0; PlayerNameLength];
        ctx.pfile.hero_names[i as usize][..end].copy_from_slice(&pkplr.pName[..end]);
        let has_save_game = archive_contains_game(ctx, &mut archive);
        if has_save_game {
            pkplr.bIsHellfire = ctx.loadsave.gbIsHellfireSaveGame as i8;
        }
        drop(archive);

        unpack_player(ctx, &pkplr, 0);
        crate::loadsave::load_hero_items(ctx, 0);
        remove_all_invalid_items(ctx, 0);
        crate::items::calc_plr_inv(ctx, 0, false);

        game2_ui_player(ctx, 0, &mut uihero, has_save_game);
        ui_add_hero_info(ctx, &uihero);
    }
    true
}

/// Original: `devilution::pfile_ui_set_class_stats` (pfile.cpp).
// @port pfile.cpp|devilution::pfile_ui_set_class_stats(unsigned int playerClass, _uidefaultstats *classStats) sha=6ade5d28f43c
pub fn pfile_ui_set_class_stats(player_class: u32, class_stats: &mut UiDefaultStats) {
    let d = &PlayersData[player_class as usize];
    class_stats.strength = d.baseStr as u16;
    class_stats.magic = d.baseMag as u16;
    class_stats.dexterity = d.baseDex as u16;
    class_stats.vitality = d.baseVit as u16;
}

/// Original: `devilution::pfile_ui_get_first_unused_save_num` (pfile.cpp).
// @port pfile.cpp|devilution::pfile_ui_get_first_unused_save_num() sha=0b831c11309e
pub fn pfile_ui_get_first_unused_save_num(ctx: &Ctx) -> u32 {
    let mut save_num = 0;
    while (save_num as usize) < MAX_CHARACTERS {
        if ctx.pfile.hero_names[save_num as usize][0] == 0 {
            break;
        }
        save_num += 1;
    }
    save_num
}

/// Original: `devilution::pfile_ui_save_create` (pfile.cpp).
// @port pfile.cpp|devilution::pfile_ui_save_create(_uiheroinfo *heroinfo) sha=85796958aff4
pub fn pfile_ui_save_create(ctx: &mut Ctx, heroinfo: &mut UiHeroInfo) -> bool {
    let save_num = heroinfo.saveNumber;
    if save_num as usize >= MAX_CHARACTERS {
        return false;
    }
    heroinfo.saveNumber = save_num;

    ctx.loadsave.giNumberOfLevels = if ctx.init.gb_is_hellfire { 25 } else { 17 };

    let mut save_writer = get_save_writer(ctx, save_num);
    let (mp, n) = (ctx.init.gb_is_multiplayer, ctx.loadsave.giNumberOfLevels);
    save_writer.remove_hash_entries(&|lvl| get_file_name(mp, n, lvl));
    let t = crate::utils::utf8::copy_utf8(heroinfo.name_str(), PlayerNameLength);
    ctx.pfile.hero_names[save_num as usize] = [0; PlayerNameLength];
    ctx.pfile.hero_names[save_num as usize][..t.len()].copy_from_slice(t.as_bytes());

    crate::player::create_player(ctx, 0, heroinfo.heroclass);
    ctx.players.Players[0]._pName.set(heroinfo.name_str());
    let pkplr = pack_player(ctx, &ctx.players.Players[0]);
    encode_hero(ctx, &mut save_writer, &pkplr);
    game2_ui_player(ctx, 0, heroinfo, false);
    if !ctx.init.gb_vanilla {
        let player = ctx.players.Players[0].clone();
        crate::loadsave::save_hotkeys(ctx, &mut save_writer, &player);
        crate::loadsave::save_hero_items(ctx, &mut save_writer, &player);
    }
    true
}

/// Original: `devilution::pfile_delete_save` (pfile.cpp).
// @port pfile.cpp|devilution::pfile_delete_save(_uiheroinfo *heroInfo) sha=7ad30d2b8ef8
pub fn pfile_delete_save(ctx: &mut Ctx, hero_info: &UiHeroInfo) -> bool {
    let save_num = hero_info.saveNumber;
    if (save_num as usize) < MAX_CHARACTERS {
        ctx.pfile.hero_names[save_num as usize][0] = 0;
        let path = get_save_path(ctx, save_num, "");
        let _ = std::fs::remove_file(path);
    }
    true
}

/// Original: `devilution::pfile_read_player_from_save` (pfile.cpp).
// @port pfile.cpp|devilution::pfile_read_player_from_save(uint32_t saveNum, Player &player) sha=cdce93e78969
pub fn pfile_read_player_from_save(ctx: &mut Ctx, save_num: u32, pnum: usize) {
    let pkplr = {
        let Some(mut archive) = open_save_archive(ctx, save_num) else {
            crate::appfat::app_fatal(ctx, &tr("Unable to open archive"));
        };
        let Some(mut pkplr) = read_hero(ctx, &mut archive) else {
            crate::appfat::app_fatal(ctx, &tr("Unable to load character"));
        };
        ctx.pfile.gbValidSaveFile = archive_contains_game(ctx, &mut archive);
        if ctx.pfile.gbValidSaveFile {
            pkplr.bIsHellfire = ctx.loadsave.gbIsHellfireSaveGame as i8;
        }
        pkplr
    };
    unpack_player(ctx, &pkplr, pnum);
    crate::loadsave::load_hero_items(ctx, pnum);
    remove_all_invalid_items(ctx, pnum);
    crate::items::calc_plr_inv(ctx, pnum, false);
}

/// Original: `devilution::pfile_save_level` (pfile.cpp).
// @port pfile.cpp|devilution::pfile_save_level() sha=7c8cba9a5153
pub fn pfile_save_level(ctx: &mut Ctx) {
    let mut save_writer = get_save_writer(ctx, ctx.menu.g_save_number);
    crate::loadsave::save_level(ctx, &mut save_writer);
}

/// Original: `devilution::pfile_convert_levels` (pfile.cpp).
// @port pfile.cpp|devilution::pfile_convert_levels() sha=5564644eb855
pub fn pfile_convert_levels(ctx: &mut Ctx) {
    let mut save_writer = get_save_writer(ctx, ctx.menu.g_save_number);
    crate::loadsave::convert_levels(ctx, &mut save_writer);
}

/// Original: `devilution::pfile_remove_temp_files` (pfile.cpp).
// @port pfile.cpp|devilution::pfile_remove_temp_files() sha=52d293a4cb2e
pub fn pfile_remove_temp_files(ctx: &mut Ctx) {
    if ctx.init.gb_is_multiplayer {
        return;
    }
    let mut save_writer = get_save_writer(ctx, ctx.menu.g_save_number);
    let n = ctx.loadsave.giNumberOfLevels;
    save_writer.remove_hash_entries(&|i| get_temp_save_names(n, i));
}

/// Original: `devilution::pfile_update` (pfile.cpp).
// @port pfile.cpp|devilution::pfile_update(bool forceSave) sha=53f67e493a3c
pub fn pfile_update(ctx: &mut Ctx, force_save: bool) {
    if !ctx.init.gb_is_multiplayer {
        return;
    }
    let tick = ctx.platform.ticks();
    if !force_save && tick.wrapping_sub(ctx.pfile.prev_tick) <= 60000 {
        return;
    }
    ctx.pfile.prev_tick = tick;
    pfile_write_hero(ctx, false);
    sfile_write_stash(ctx);
}
