//! `Source/loadsave.cpp`: save games. The record layouts are those of the original (and of the
//! vanilla game they stay compatible with): field order, widths, padding and endianness.

use std::collections::HashMap;

use crate::codec::{codec_encode, codec_get_encoded_len};
use crate::ctx::Ctx;
use crate::engine::geometry::{Direction, Displacement, Point};
use crate::enums::*;
use crate::items::{Item, MAXITEMS};
use crate::levels::gendung::{get_level_type, DungeonType, DMAXX, DMAXY, MAXDUNX, MAXDUNY};
use crate::lighting::{Light, MAP_EXP_OLD, MAP_EXP_SELF, NO_LIGHT};
use crate::missiles::Missile;
use crate::monster::{Monster, MaxMonsters};
use crate::mpq_writer::{MaxMpqPathSize, SaveWriter};
use crate::objects::Object;
use crate::player::{InventoryGridCells, MaxBeltItems, Player, PlayerNameLength};
use crate::portal::MAXPORTAL;
use crate::spells::NUM_HOTKEYS as NumHotkeys;
use crate::tables::playerdat::PlayersData;
use crate::utils::cstr::CStr;
use crate::utils::language::tr;

const MaxMissilesForSaveGame: usize = 125;
const NUM_INVLOC_USIZE: usize = NUM_INVLOC as usize;
/// `ItemAnimWidth` (items.h)
const ItemAnimWidth: i32 = 96;

/// Globals of loadsave.cpp.
pub struct LoadsaveState {
    pub gbIsHellfireSaveGame: bool,
    pub giNumberOfLevels: u8,
    giNumberQuests: u8,
    giNumberOfSmithPremiumItems: u8,
    gbSkipSync: bool,
}

impl Default for LoadsaveState {
    fn default() -> Self {
        LoadsaveState { gbIsHellfireSaveGame: false, giNumberOfLevels: 0, giNumberQuests: 0, giNumberOfSmithPremiumItems: 0, gbSkipSync: false }
    }
}

/// `LoadHelper`: reads a decoded save record.
pub struct LoadHelper {
    buffer: Option<Vec<u8>>,
    cur: usize,
    size: usize,
}

macro_rules! next_fn {
    ($name:ident, $be:ident, $t:ty) => {
        pub fn $name(&mut self) -> $t {
            let n = std::mem::size_of::<$t>();
            if !self.is_valid(n) {
                return 0 as $t;
            }
            let b = self.buffer.as_ref().unwrap();
            let v = <$t>::from_le_bytes(b[self.cur..self.cur + n].try_into().unwrap());
            self.cur += n;
            v
        }
        pub fn $be(&mut self) -> $t {
            let n = std::mem::size_of::<$t>();
            if !self.is_valid(n) {
                return 0 as $t;
            }
            let b = self.buffer.as_ref().unwrap();
            let v = <$t>::from_be_bytes(b[self.cur..self.cur + n].try_into().unwrap());
            self.cur += n;
            v
        }
    };
}

impl LoadHelper {
    /// Original: `LoadHelper::LoadHelper(std::optional<SaveReader> archive, const char *szFileName)`.
    // @port loadsave.cpp|devilution::LoadHelper::LoadHelper(std::optional<SaveReader> archive, const char *szFileName) sha=3a27d7566871
    pub fn new(ctx: &Ctx, archive: Option<crate::mpq::MpqArchive>, file_name: &str) -> LoadHelper {
        let buffer = archive.and_then(|mut a| crate::pfile::read_archive(ctx, &mut a, file_name));
        let size = buffer.as_ref().map_or(0, |b| b.len());
        LoadHelper { buffer, cur: 0, size }
    }

    pub fn is_valid(&self, size: usize) -> bool {
        self.buffer.is_some() && self.size >= self.cur + size
    }

    pub fn skip(&mut self, size: usize) {
        self.cur += size;
    }

    pub fn next_bytes(&mut self, out: &mut [u8]) {
        if !self.is_valid(out.len()) {
            return;
        }
        let b = self.buffer.as_ref().unwrap();
        out.copy_from_slice(&b[self.cur..self.cur + out.len()]);
        self.cur += out.len();
    }

    next_fn!(next_u8, next_u8_be, u8);
    next_fn!(next_i8, next_i8_be, i8);
    next_fn!(next_u16, next_u16_be, u16);
    next_fn!(next_i16, next_i16_be, i16);
    next_fn!(next_u32, next_u32_be, u32);
    next_fn!(next_i32, next_i32_be, i32);
    next_fn!(next_u64, next_u64_be, u64);

    /// `NextLENarrow<int32_t, int8_t>(modifier)`
    pub fn next_i32_narrow_i8(&mut self, modifier: i32) -> i8 {
        let value = self.next_i32().wrapping_add(modifier);
        value.clamp(i8::MIN as i32, i8::MAX as i32) as i8
    }

    /// `NextLENarrow<int32_t, int16_t>()`
    pub fn next_i32_narrow_i16(&mut self) -> i16 {
        self.next_i32().clamp(i16::MIN as i32, i16::MAX as i32) as i16
    }

    /// `NextLENarrow<uint32_t, uint8_t>()`
    pub fn next_u32_narrow_u8(&mut self) -> u8 {
        self.next_u32().clamp(u8::MIN as u32, u8::MAX as u32) as u8
    }

    pub fn next_bool8(&mut self) -> bool {
        self.next_u8() != 0
    }

    pub fn next_bool32(&mut self) -> bool {
        self.next_u32() != 0
    }
}

/// `SaveHelper`: builds a record; `finish` encodes and writes it (the original's destructor).
pub struct SaveHelper {
    file_name: String,
    buffer: Vec<u8>,
    capacity: usize,
}

impl SaveHelper {
    // @port loadsave.cpp|devilution::SaveHelper::SaveHelper(SaveWriter &mpqWriter, const char *szFileName, size_t bufferLen) sha=aa0b3ce77c73
    pub fn new(file_name: &str, buffer_len: usize) -> SaveHelper {
        SaveHelper { file_name: file_name.to_string(), buffer: Vec::with_capacity(codec_get_encoded_len(buffer_len)), capacity: buffer_len }
    }

    pub fn is_valid(&self, len: usize) -> bool {
        self.capacity >= self.buffer.len() + len
    }

    pub fn skip(&mut self, len: usize) {
        self.buffer.resize(self.buffer.len() + len, 0);
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        if !self.is_valid(bytes.len()) {
            return;
        }
        self.buffer.extend_from_slice(bytes);
    }

    pub fn u8(&mut self, v: u8) {
        self.write_bytes(&[v]);
    }
    pub fn i8(&mut self, v: i8) {
        self.write_bytes(&[v as u8]);
    }
    pub fn u16(&mut self, v: u16) {
        self.write_bytes(&v.to_le_bytes());
    }
    pub fn i16(&mut self, v: i16) {
        self.write_bytes(&v.to_le_bytes());
    }
    pub fn u32(&mut self, v: u32) {
        self.write_bytes(&v.to_le_bytes());
    }
    pub fn i32(&mut self, v: i32) {
        self.write_bytes(&v.to_le_bytes());
    }
    pub fn u64(&mut self, v: u64) {
        self.write_bytes(&v.to_le_bytes());
    }
    pub fn u32_be(&mut self, v: u32) {
        self.write_bytes(&v.to_be_bytes());
    }
    pub fn i32_be(&mut self, v: i32) {
        self.write_bytes(&v.to_be_bytes());
    }

    /// Original: `SaveHelper::~SaveHelper()`.
    // @port loadsave.cpp|devilution::SaveHelper::~SaveHelper() sha=c7fd1c3f2340
    pub fn finish(mut self, ctx: &Ctx, writer: &mut SaveWriter) {
        let cur = self.buffer.len();
        let password = crate::pfile::pfile_get_password(ctx);
        codec_encode(&mut self.buffer, cur, password);
        writer.write_file(&self.file_name, &self.buffer);
    }
}

/// `MonsterConversionData`
#[derive(Clone, Copy, Default)]
struct MonsterConversionData {
    monster_level: i8,
    experience: u16,
    to_hit_special: u8,
}

/// `LevelConversionData`
struct LevelConversionData {
    monster_conversion_data: Vec<MonsterConversionData>,
}

impl Default for LevelConversionData {
    fn default() -> Self {
        LevelConversionData { monster_conversion_data: vec![MonsterConversionData::default(); MaxMonsters] }
    }
}

fn name_bytes<const N: usize>(file: &mut LoadHelper) -> CStr<N> {
    let mut b = [0u8; N];
    file.next_bytes(&mut b);
    CStr::from_raw(&b)
}

/// Original: `LoadItemData` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadItemData(LoadHelper &file, Item &item) sha=c8a5a1e307d6
fn load_item_data(ctx: &mut Ctx, file: &mut LoadHelper, item: &mut Item) {
    item._iSeed = file.next_u32();
    item._iCreateInfo = file.next_u16();
    file.skip(2);
    item._itype = ItemType::from_raw(file.next_u32() as i8);
    item.position.x = file.next_i32();
    item.position.y = file.next_i32();
    item._iAnimFlag = file.next_bool32();
    file.skip(4);
    item.AnimInfo = Default::default();
    item.AnimInfo.numberOfFrames = file.next_i32_narrow_i8(0);
    item.AnimInfo.currentFrame = file.next_i32_narrow_i8(-1);
    file.skip(8);
    file.skip(4);
    item._iSelFlag = file.next_u8();
    file.skip(3);
    item._iPostDraw = file.next_bool32();
    item._iIdentified = file.next_bool32();
    item._iMagical = file.next_i8() as item_quality;
    item._iName = name_bytes::<64>(file);
    item._iIName = name_bytes::<64>(file);
    item._iLoc = file.next_i8() as item_equip_type;
    item._iClass = file.next_u8() as item_class;
    file.skip(1);
    item._iCurs = file.next_i32() as u8;
    item._ivalue = file.next_i32();
    item._iIvalue = file.next_i32();
    item._iMinDam = file.next_i32() as u8;
    item._iMaxDam = file.next_i32() as u8;
    item._iAC = file.next_i32() as i16;
    item._iFlags = ItemSpecialEffect(file.next_u32());
    item._iMiscId = file.next_i32() as item_misc_id;
    item._iSpell = SpellID::from_raw(file.next_i32() as i8);
    item._iCharges = file.next_i32();
    item._iMaxCharges = file.next_i32();
    item._iDurability = file.next_i32();
    item._iMaxDur = file.next_i32();
    item._iPLDam = file.next_i32() as i16;
    item._iPLToHit = file.next_i32() as i16;
    item._iPLAC = file.next_i32() as i16;
    item._iPLStr = file.next_i32() as i16;
    item._iPLMag = file.next_i32() as i16;
    item._iPLDex = file.next_i32() as i16;
    item._iPLVit = file.next_i32() as i16;
    item._iPLFR = file.next_i32() as i16;
    item._iPLLR = file.next_i32() as i16;
    item._iPLMR = file.next_i32() as i16;
    item._iPLMana = file.next_i32() as i16;
    item._iPLHP = file.next_i32() as i16;
    item._iPLDamMod = file.next_i32() as i16;
    item._iPLGetHit = file.next_i32() as i16;
    item._iPLLight = file.next_i32() as i16;
    item._iSplLvlAdd = file.next_i8();
    item._iRequest = file.next_bool8();
    file.skip(2);
    item._iUid = file.next_i32();
    item._iFMinDam = file.next_i32() as i16;
    item._iFMaxDam = file.next_i32() as i16;
    item._iLMinDam = file.next_i32() as i16;
    item._iLMaxDam = file.next_i32() as i16;
    item._iPLEnAc = file.next_i32() as i16;
    item._iPrePower = file.next_i8() as item_effect_type;
    item._iSufPower = file.next_i8() as item_effect_type;
    file.skip(2);
    item._iVAdd1 = file.next_i32();
    item._iVMult1 = file.next_i32();
    item._iVAdd2 = file.next_i32();
    item._iVMult2 = file.next_i32();
    item._iMinStr = file.next_i8();
    item._iMinMag = file.next_u8();
    item._iMinDex = file.next_i8();
    file.skip(1);
    item._iStatFlag = file.next_bool32();
    item.IDidx = file.next_i32() as _item_indexes;
    if ctx.init.gb_is_spawn {
        item.IDidx = remap_item_idx_from_spawn(item.IDidx);
    }
    if !ctx.loadsave.gbIsHellfireSaveGame {
        item.IDidx = remap_item_idx_from_diablo(item.IDidx);
    }
    item.dwBuff = file.next_u32();
    if ctx.loadsave.gbIsHellfireSaveGame {
        item._iDamAcFlags = ItemSpecialEffectHf(file.next_u32() as u8);
    } else {
        item._iDamAcFlags = ItemSpecialEffectHf::None;
    }
    let iname = item._iIName.as_str().to_string();
    crate::items::update_hellfire_flag(ctx, item, &iname);
}

/// Original: `LoadAndValidateItemData` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadAndValidateItemData(LoadHelper &file, Item &item) sha=893edaa56c2a
fn load_and_validate_item_data(ctx: &mut Ctx, file: &mut LoadHelper, item: &mut Item) {
    load_item_data(ctx, file, item);
    remove_invalid_item(ctx, item);
}

/// Original: `LoadPlayer` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadPlayer(LoadHelper &file, Player &player) sha=602e56bf5af0
fn load_player(ctx: &mut Ctx, file: &mut LoadHelper, pnum: usize) {
    let mut player = std::mem::take(&mut ctx.players.Players[pnum]);
    player._pmode = file.next_i32() as PLR_MODE;
    for step in player.walkpath.iter_mut() {
        *step = file.next_i8();
    }
    player.plractive = file.next_bool8();
    file.skip(2);
    player.destAction = file.next_i32() as action_id;
    player.destParam1 = file.next_i32();
    player.destParam2 = file.next_i32();
    player.destParam3 = file.next_i32();
    player.destParam4 = file.next_i32();
    player.set_level(file.next_u32() as u8);
    player.position.tile.x = file.next_i32();
    player.position.tile.y = file.next_i32();
    player.position.future.x = file.next_i32();
    player.position.future.y = file.next_i32();
    file.skip(4 * 2);
    player.position.last.x = file.next_i32();
    player.position.last.y = file.next_i32();
    player.position.old.x = file.next_i32();
    player.position.old.y = file.next_i32();
    file.skip(4 * 4);
    player._pdir = Direction::from_u8(file.next_i32() as u8);
    file.skip(4);
    player._pgfxnum = file.next_u32_narrow_u8();
    file.skip(4);
    player.AnimInfo = Default::default();
    player.AnimInfo.ticksPerFrame = file.next_i32_narrow_i8(1);
    player.AnimInfo.tickCounterOfCurrentFrame = file.next_i32_narrow_i8(0);
    player.AnimInfo.numberOfFrames = file.next_i32_narrow_i8(0);
    player.AnimInfo.currentFrame = file.next_i32_narrow_i8(-1);
    file.skip(4 * 3);
    player.lightId = file.next_i32();
    file.skip(4);

    player.queuedSpell.spellId = SpellID::from_raw(file.next_i32() as i8);
    player.queuedSpell.spellType = SpellType::from_raw(file.next_u8());
    let mut spell_from = file.next_i8();
    if !crate::spells::is_valid_spell_from(spell_from as i32) {
        spell_from = 0;
    }
    player.spellFrom = spell_from;
    player.queuedSpell.spellFrom = spell_from;
    file.skip(2);
    player.inventorySpell = SpellID::from_raw(file.next_i32() as i8);
    file.skip(1);
    file.skip(3);
    player._pRSpell = SpellID::from_raw(file.next_i32() as i8);
    player._pRSplType = SpellType::from_raw(file.next_u8());
    file.skip(3);
    player._pSBkSpell = SpellID::from_raw(file.next_i32() as i8);
    file.skip(1);
    for spell_level in player._pSplLvl.iter_mut() {
        *spell_level = file.next_u8();
    }
    file.skip(7);
    player._pMemSpells = file.next_u64();
    player._pAblSpells = file.next_u64();
    player._pScrlSpells = file.next_u64();
    player._pSpellFlags = SpellFlag(file.next_u8());
    file.skip(3);

    for i in 0..4 {
        player._pSplHotKey[i] = SpellID::from_raw(file.next_i32() as i8);
    }
    for i in 0..4 {
        player._pSplTHotKey[i] = SpellType::from_raw(file.next_u8());
    }

    file.skip(4);
    player._pBlockFlag = file.next_bool8();
    player._pInvincible = file.next_bool8();
    player._pLightRad = file.next_i8();
    player._pLvlChanging = file.next_bool8();

    player._pName = name_bytes::<PlayerNameLength>(file);
    player._pClass = HeroClass::from_raw(file.next_i8() as u8);
    file.skip(3);
    player._pStrength = file.next_i32();
    player._pBaseStr = file.next_i32();
    player._pMagic = file.next_i32();
    player._pBaseMag = file.next_i32();
    player._pDexterity = file.next_i32();
    player._pBaseDex = file.next_i32();
    player._pVitality = file.next_i32();
    player._pBaseVit = file.next_i32();
    player._pStatPts = file.next_i32();
    player._pDamageMod = file.next_i32();
    player._pBaseToBlk = file.next_i32();
    if player._pBaseToBlk == 0 {
        player._pBaseToBlk = PlayersData[player._pClass as usize].blockBonus as i32;
    }
    player._pHPBase = file.next_i32();
    player._pMaxHPBase = file.next_i32();
    player._pHitPoints = file.next_i32();
    player._pMaxHP = file.next_i32();
    file.skip(4);
    player._pManaBase = file.next_i32();
    player._pMaxManaBase = file.next_i32();
    player._pMana = file.next_i32();
    player._pMaxMana = file.next_i32();
    file.skip(4);
    player._pLevel = file.next_i8();
    player._pMaxLvl = file.next_i8();
    file.skip(2);
    player._pExperience = file.next_u32();
    file.skip(4);
    player._pNextExper = file.next_u32();
    player._pArmorClass = file.next_i8();
    player._pMagResist = file.next_i8();
    player._pFireResist = file.next_i8();
    player._pLghtResist = file.next_i8();
    player._pGold = file.next_i32();
    player._pInfraFlag = file.next_bool32();

    let mut temp_position_x = file.next_i32();
    let mut temp_position_y = file.next_i32();
    if player._pmode == PM_WALK_NORTHWARDS {
        temp_position_x += player.position.tile.x;
        temp_position_y += player.position.tile.y;
    }
    // `WorldTileCoord` is uint8_t
    player.position.temp.x = temp_position_x as u8 as i32;
    player.position.temp.y = temp_position_y as u8 as i32;

    player.tempDirection = Direction::from_u8(file.next_i32() as u8);
    player.queuedSpell.spellLevel = file.next_i32();
    file.skip(4);
    file.skip(4 * 2);
    file.skip(4);

    let levels = ctx.loadsave.giNumberOfLevels as usize;
    for i in 0..levels {
        player._pLvlVisited[i] = file.next_bool8();
    }
    for i in 0..levels {
        player._pSLvlVisited[i] = file.next_bool8();
    }

    file.skip(2);
    file.skip(4);
    file.skip(4 * 8);
    player._pNFrames = file.next_i32_narrow_i8(0);
    file.skip(4);
    file.skip(4 * 8);
    player._pWFrames = file.next_i32_narrow_i8(0);
    file.skip(4);
    file.skip(4 * 8);
    player._pAFrames = file.next_i32_narrow_i8(0);
    file.skip(4);
    player._pAFNum = file.next_i32_narrow_i8(0);
    file.skip(4 * 8);
    file.skip(4 * 8);
    file.skip(4 * 8);
    player._pSFrames = file.next_i32_narrow_i8(0);
    file.skip(4);
    player._pSFNum = file.next_i32_narrow_i8(0);
    file.skip(4 * 8);
    player._pHFrames = file.next_i32_narrow_i8(0);
    file.skip(4);
    file.skip(4 * 8);
    player._pDFrames = file.next_i32_narrow_i8(0);
    file.skip(4);
    file.skip(4 * 8);
    player._pBFrames = file.next_i32_narrow_i8(0);
    file.skip(4);

    for item in player.InvBody.iter_mut() {
        load_and_validate_item_data(ctx, file, item);
    }
    for item in player.InvList.iter_mut() {
        load_and_validate_item_data(ctx, file, item);
    }
    player._pNumInv = file.next_i32();
    for cell in player.InvGrid.iter_mut() {
        *cell = file.next_i8();
    }
    for item in player.SpdList.iter_mut() {
        load_and_validate_item_data(ctx, file, item);
    }
    load_and_validate_item_data(ctx, file, &mut player.HoldItem);

    player._pIMinDam = file.next_i32();
    player._pIMaxDam = file.next_i32();
    player._pIAC = file.next_i32();
    player._pIBonusDam = file.next_i32();
    player._pIBonusToHit = file.next_i32();
    player._pIBonusAC = file.next_i32();
    player._pIBonusDamMod = file.next_i32();
    file.skip(4);

    player._pISpells = file.next_u64();
    player._pIFlags = ItemSpecialEffect(file.next_i32() as u32);
    player._pIGetHit = file.next_i32();
    player._pISplLvlAdd = file.next_i8();
    file.skip(1);
    file.skip(2);
    file.skip(4);
    player._pIEnAc = file.next_i32();
    player._pIFMinDam = file.next_i32();
    player._pIFMaxDam = file.next_i32();
    player._pILMinDam = file.next_i32();
    player._pILMaxDam = file.next_i32();
    player._pOilType = file.next_i32() as item_misc_id;
    player.pTownWarps = file.next_u8();
    player.pDungMsgs = file.next_u8();
    player.pLvlLoad = file.next_u8();

    if ctx.loadsave.gbIsHellfireSaveGame {
        player.pDungMsgs2 = file.next_u8();
    } else {
        player.pDungMsgs2 = 0;
        file.skip(1);
    }
    player.pManaShield = file.next_bool8();
    if ctx.loadsave.gbIsHellfireSaveGame {
        player.pOriginalCathedral = file.next_bool8();
    } else {
        file.skip(1);
        player.pOriginalCathedral = true;
    }
    file.skip(2);
    player.wReflections = file.next_u16();
    file.skip(14);

    player.pDiabloKillLevel = file.next_u32() as u8;
    ctx.multi.sgGameInitInfo.nDifficulty = file.next_u32() as _difficulty;
    player.pDamAcFlags = ItemSpecialEffectHf(file.next_u32() as u8);
    file.skip(20);
    ctx.players.Players[pnum] = player;
    crate::items::calc_plr_item_vals(ctx, pnum, false);

    let player = &mut ctx.players.Players[pnum];
    player.executedSpell = player.queuedSpell;

    if ctx.gendung.setlevel {
        player.set_set_level(ctx.gendung.setlvlnum);
    } else {
        player.set_level(ctx.gendung.currlevel);
    }
}

/// Original: `LoadMonster` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadMonster(LoadHelper *file, Monster &monster, MonsterConversionData *monsterConversionData = nullptr) sha=d39e9ba999fa
fn load_monster(ctx: &mut Ctx, file: &mut LoadHelper, m: usize, mut conversion: Option<&mut MonsterConversionData>) {
    let monster = &mut ctx.monster.Monsters[m];
    monster.levelType = file.next_i32() as u8;
    monster.mode = MonsterMode::from_raw(file.next_i32() as u8);
    monster.goal = MonsterGoal::from_raw(file.next_u8());
    file.skip(3);
    monster.goalVar1 = file.next_i32_narrow_i16();
    monster.goalVar2 = file.next_i32_narrow_i8(0);
    monster.goalVar3 = file.next_i32_narrow_i8(0);
    file.skip(4);
    monster.pathCount = file.next_u8();
    file.skip(3);
    monster.position.tile.x = file.next_i32();
    monster.position.tile.y = file.next_i32();
    monster.position.future.x = file.next_i32();
    monster.position.future.y = file.next_i32();
    monster.position.old.x = file.next_i32();
    monster.position.old.y = file.next_i32();
    file.skip(4 * 4);
    monster.direction = Direction::from_u8(file.next_i32() as u8);
    monster.enemy = file.next_i32() as u8;
    monster.enemyPosition.x = file.next_u8() as i32;
    monster.enemyPosition.y = file.next_u8() as i32;
    file.skip(2);

    file.skip(4);
    monster.animInfo = Default::default();
    monster.animInfo.ticksPerFrame = file.next_i32_narrow_i8(0);
    monster.animInfo.tickCounterOfCurrentFrame = file.next_i32_narrow_i8(1).wrapping_sub(1);
    monster.animInfo.numberOfFrames = file.next_i32_narrow_i8(0);
    monster.animInfo.currentFrame = file.next_i32_narrow_i8(-1);
    file.skip(4);
    monster.isInvalid = file.next_bool32();
    monster.var1 = file.next_i32_narrow_i16();
    monster.var2 = file.next_i32_narrow_i16();
    monster.var3 = file.next_i32_narrow_i8(0);
    // NextLENarrow<int32_t, WorldTileCoord>
    monster.position.temp.x = file.next_i32().clamp(0, 255);
    monster.position.temp.y = file.next_i32().clamp(0, 255);
    file.skip(4 * 2);
    file.skip(4);
    monster.maxHitPoints = file.next_i32();
    monster.hitPoints = file.next_i32();

    monster.ai = MonsterAIID::from_raw(file.next_u8() as i8);
    monster.intelligence = file.next_u8();
    file.skip(2);
    monster.flags = file.next_u32();
    monster.activeForTicks = file.next_u8();
    file.skip(3);
    file.skip(4);
    monster.position.last.x = file.next_i32();
    monster.position.last.y = file.next_i32();
    monster.rndItemSeed = file.next_u32();
    monster.aiSeed = file.next_u32();
    file.skip(4);

    monster.uniqueType = UniqueMonsterType::from_raw(file.next_u8().wrapping_sub(1));
    monster.uniqTrans = file.next_u8();
    monster.corpseId = file.next_i8();

    monster.whoHit = file.next_i8();
    match conversion.as_deref_mut() {
        Some(c) => c.monster_level = file.next_i8(),
        None => file.skip(1),
    }
    file.skip(1);
    match conversion.as_deref_mut() {
        Some(c) => c.experience = file.next_u16(),
        None => file.skip(2),
    }
    let is_minion = ctx.monster.Monsters[m].is_player_minion();
    let monster = &mut ctx.monster.Monsters[m];
    if is_minion {
        monster.toHit = file.next_u8() as u16;
    } else {
        file.skip(1);
    }
    monster.minDamage = file.next_u8();
    monster.maxDamage = file.next_u8();
    match conversion.as_deref_mut() {
        Some(c) => c.to_hit_special = file.next_u8(),
        None => file.skip(1),
    }
    monster.minDamageSpecial = file.next_u8();
    monster.maxDamageSpecial = file.next_u8();
    monster.armorClass = file.next_u8();
    file.skip(1);
    monster.resistance = file.next_u16();
    file.skip(2);

    monster.talkMsg = file.next_i32() as _speech_id;
    if monster.talkMsg == TEXT_KING1 {
        monster.talkMsg = TEXT_NONE;
    }
    monster.leader = file.next_u8();
    if monster.leader == 0 {
        monster.leader = Monster::NoLeader;
    }
    monster.leaderRelation = LeaderRelation::from_raw(file.next_u8());
    monster.packSize = file.next_u8();
    monster.lightId = file.next_i8();
    if monster.lightId == 0 {
        monster.lightId = NO_LIGHT as i8;
    }
    if monster.mode == MonsterMode::Petrified {
        monster.animInfo.isPetrified = true;
    }
}

/// Original: `SyncPackSize` (loadsave.cpp).
// @port loadsave.cpp|devilution::SyncPackSize(Monster &leader) sha=1c9e06910bf3
fn sync_pack_size(ctx: &mut Ctx, leader: usize) {
    let l = &ctx.monster.Monsters[leader];
    if !l.is_unique() {
        return;
    }
    if l.ai != MonsterAIID::Scavenger {
        return;
    }
    let mut pack_size = 0u8;
    for i in 0..ctx.monster.ActiveMonsterCount {
        let minion = &ctx.monster.Monsters[ctx.monster.ActiveMonsters[i] as usize];
        if minion.leaderRelation == LeaderRelation::Leashed && minion.leader as usize == leader {
            pack_size = pack_size.wrapping_add(1);
        }
    }
    ctx.monster.Monsters[leader].packSize = pack_size;
}

/// Original: `LoadMissile` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadMissile(LoadHelper *file) sha=ac92e5007b07
fn load_missile(ctx: &mut Ctx, file: &mut LoadHelper) {
    let mut missile = Missile::default();
    missile._mitype = MissileID::from_raw(file.next_i32() as i8);
    missile.position.tile.x = file.next_i32();
    missile.position.tile.y = file.next_i32();
    missile.position.offset.delta_x = file.next_i32();
    missile.position.offset.delta_y = file.next_i32();
    missile.position.velocity.delta_x = file.next_i32();
    missile.position.velocity.delta_y = file.next_i32();
    missile.position.start.x = file.next_i32();
    missile.position.start.y = file.next_i32();
    missile.position.traveled.delta_x = file.next_i32();
    missile.position.traveled.delta_y = file.next_i32();
    missile._mimfnum = file.next_i32();
    missile._mispllvl = file.next_i32();
    missile._miDelFlag = file.next_bool32();
    missile._miAnimType = MissileGraphicID::from_raw(file.next_u8());
    file.skip(3);
    missile._miAnimFlags = MissileGraphicsFlags(file.next_i32() as u8);
    file.skip(4);
    missile._miAnimDelay = file.next_i32();
    missile._miAnimLen = file.next_i32();
    missile._miAnimWidth = file.next_i32() as u16;
    missile._miAnimWidth2 = file.next_i32() as i16;
    missile._miAnimCnt = file.next_i32();
    missile._miAnimAdd = file.next_i32();
    missile._miAnimFrame = file.next_i32();
    missile._miDrawFlag = file.next_bool32();
    missile._miLightFlag = file.next_bool32();
    missile._miPreFlag = file.next_bool32();
    missile._miUniqTrans = file.next_u32();
    missile._mirange = file.next_i32();
    missile._misource = file.next_i32();
    missile._micaster = file.next_i32() as mienemy_type;
    missile._midam = file.next_i32();
    missile._miHitFlag = file.next_bool32();
    missile._midist = file.next_i32();
    missile._mlid = file.next_i32();
    missile._mirnd = file.next_i32();
    missile.var1 = file.next_i32();
    missile.var2 = file.next_i32();
    missile.var3 = file.next_i32();
    missile.var4 = file.next_i32();
    missile.var5 = file.next_i32();
    missile.var6 = file.next_i32();
    missile.var7 = file.next_i32();
    missile.limitReached = file.next_bool32();
    missile.lastCollisionTargetHash = 0;
    ctx.missiles.Missiles.push(missile);
}

/// Original: `ConvertFromHellfireObject` (loadsave.cpp).
// @port loadsave.cpp|devilution::ConvertFromHellfireObject(_object_id type) sha=05252f4cfc1d
fn convert_from_hellfire_object(ctx: &Ctx, ty: _object_id) -> _object_id {
    if ctx.gendung.leveltype == DungeonType::Nest {
        match ty {
            OBJ_BARREL => return OBJ_POD,
            OBJ_BARRELEX => return OBJ_PODEX,
            _ => {}
        }
    }
    if ctx.gendung.leveltype == DungeonType::Crypt {
        match ty {
            OBJ_BARREL => return OBJ_URN,
            OBJ_BARRELEX => return OBJ_URNEX,
            OBJ_STORYBOOK => return OBJ_L5BOOKS,
            OBJ_STORYCANDLE => return OBJ_L5CANDLE,
            OBJ_L1LDOOR => return OBJ_L5LDOOR,
            OBJ_L1RDOOR => return OBJ_L5RDOOR,
            OBJ_LEVER => return OBJ_L5LEVER,
            OBJ_SARC => return OBJ_L5SARC,
            _ => {}
        }
    }
    ty
}

/// Original: `LoadObject` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadObject(LoadHelper &file, Object &object) sha=228e9bcb6af1
fn load_object(ctx: &mut Ctx, file: &mut LoadHelper, oi: usize) {
    let otype = convert_from_hellfire_object(ctx, file.next_i32() as _object_id);
    let object: &mut Object = &mut ctx.objects.Objects[oi];
    object._otype = otype;
    object.position.x = file.next_i32();
    object.position.y = file.next_i32();
    object.applyLighting = file.next_bool32();
    object._oAnimFlag = file.next_bool32() as u32;
    file.skip(4);
    object._oAnimDelay = file.next_i32();
    object._oAnimCnt = file.next_i32();
    object._oAnimLen = file.next_u32();
    object._oAnimFrame = file.next_u32();
    object._oAnimWidth = file.next_i32() as u16;
    file.skip(4);
    object._oDelFlag = file.next_bool32();
    object._oBreak = file.next_i8();
    file.skip(3);
    object._oSolidFlag = file.next_bool32();
    object._oMissFlag = file.next_bool32();
    object._oSelFlag = file.next_i8() as u8;
    file.skip(3);
    object._oPreFlag = file.next_bool32();
    object._oTrapFlag = file.next_bool32();
    object._oDoorFlag = file.next_bool32();
    object._olid = file.next_i32();
    object._oRndSeed = file.next_u32();
    object._oVar1 = file.next_i32();
    object._oVar2 = file.next_i32();
    object._oVar3 = file.next_i32();
    object._oVar4 = file.next_i32();
    object._oVar5 = file.next_i32();
    object._oVar6 = file.next_u32();
    object.bookMessage = file.next_i32() as _speech_id;
    object._oVar8 = file.next_i32();
}

/// Original: `LoadItem` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadItem(LoadHelper &file, Item &item) sha=7c998ee5e760
fn load_item(ctx: &mut Ctx, file: &mut LoadHelper, item: &mut Item) {
    load_and_validate_item_data(ctx, file, item);
    crate::items::get_item_frm(ctx, item);
}

/// Original: `LoadPremium` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadPremium(LoadHelper &file, int i) sha=6c254455177a
fn load_premium(ctx: &mut Ctx, file: &mut LoadHelper, i: usize) {
    let mut item = std::mem::take(&mut ctx.stores.premiumitems[i]);
    load_and_validate_item_data(ctx, file, &mut item);
    ctx.stores.premiumitems[i] = item;
}

/// Original: `LoadQuest` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadQuest(LoadHelper *file, int i) sha=4becfa911ec5
fn load_quest(ctx: &mut Ctx, file: &mut LoadHelper, i: usize) {
    let hellfire_save = ctx.loadsave.gbIsHellfireSaveGame;
    let quest = &mut ctx.quests.Quests[i];
    quest._qlevel = file.next_u8();
    file.skip(1);
    quest._qactive = file.next_u8() as quest_state;
    quest._qlvltype = DungeonType::from_i8(file.next_u8() as i8);
    quest.position.x = file.next_i32();
    quest.position.y = file.next_i32();
    quest._qslvl = file.next_u8() as _setlevels;
    quest._qidx = file.next_u8() as quest_id;
    if hellfire_save {
        file.skip(2);
        quest._qmsg = file.next_i32() as _speech_id;
    } else {
        quest._qmsg = file.next_u8() as _speech_id;
    }
    quest._qvar1 = file.next_u8();
    quest._qvar2 = file.next_u8();
    file.skip(2);
    if !hellfire_save {
        file.skip(1);
    }
    quest._qlog = file.next_bool32();

    ctx.quests.ReturnLvlPosition.x = file.next_i32_be();
    ctx.quests.ReturnLvlPosition.y = file.next_i32_be();
    ctx.quests.ReturnLevel = file.next_i32_be();
    ctx.quests.ReturnLevelType = DungeonType::from_i8(file.next_i32_be() as i8);
    file.skip(4);
}

/// Original: `LoadLighting` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadLighting(LoadHelper *file, Light *pLight) sha=0b93b0439dcf
fn load_lighting(file: &mut LoadHelper, light: &mut Light) {
    light.position.tile.x = file.next_i32();
    light.position.tile.y = file.next_i32();
    light.radius = file.next_i32() as u8;
    file.skip(4);
    light.isInvalid = file.next_bool32();
    light.hasChanged = file.next_bool32();
    file.skip(4);
    light.position.old.x = file.next_i32();
    light.position.old.y = file.next_i32();
    light.oldRadius = file.next_i32() as u8;
    light.position.offset.delta_x = file.next_i32();
    light.position.offset.delta_y = file.next_i32();
    file.skip(4);
}

/// Original: `LoadPortal` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadPortal(LoadHelper *file, int i) sha=a5a691c6cc26
fn load_portal(ctx: &mut Ctx, file: &mut LoadHelper, i: usize) {
    let portal = &mut ctx.portal.Portals[i];
    portal.open = file.next_bool32();
    portal.position.x = file.next_i32();
    portal.position.y = file.next_i32();
    portal.level = file.next_i32();
    portal.ltype = DungeonType::from_i8(file.next_i32() as i8);
    portal.setlvl = file.next_bool32();
    if !portal.setlvl {
        portal.ltype = get_level_type(portal.level);
    }
}

/// Original: `GetLevelNames` (loadsave.cpp).
// @port loadsave.cpp|devilution::GetLevelNames(string_view prefix, char *out) sha=4e2ba42c5b45
fn get_level_names(ctx: &Ctx, prefix: &str) -> String {
    let (suf, num) = if ctx.gendung.setlevel { ('s', ctx.gendung.setlvlnum as u8) } else { ('l', ctx.gendung.currlevel) };
    let s = format!("{prefix}{suf}{num:02}");
    debug_assert!(s.len() < MaxMpqPathSize);
    s
}

/// Original: `GetTempLevelNames` (loadsave.cpp).
// @port loadsave.cpp|devilution::GetTempLevelNames(char *szTemp) sha=aa7a1810d6e2
fn get_temp_level_names(ctx: &Ctx) -> String {
    get_level_names(ctx, "temp")
}

/// Original: `GetPermLevelNames` (loadsave.cpp).
// @port loadsave.cpp|devilution::GetPermLevelNames(char *szPerm) sha=6050fdd67796
fn get_perm_level_names(ctx: &Ctx) -> String {
    get_level_names(ctx, "perm")
}

/// Original: `LevelFileExists` (loadsave.cpp).
// @port loadsave.cpp|devilution::LevelFileExists(SaveWriter &archive) sha=548eaf60b0ec
fn level_file_exists(ctx: &Ctx, archive: &SaveWriter) -> bool {
    if archive.has_file(&get_temp_level_names(ctx)) {
        return true;
    }
    archive.has_file(&get_perm_level_names(ctx))
}

/// Original: `IsShopPriceValid` (loadsave.cpp).
// @port loadsave.cpp|devilution::IsShopPriceValid(const Item &item) sha=594e44ee7c25
fn is_shop_price_valid(ctx: &Ctx, item: &Item) -> bool {
    let hf = ctx.init.gb_is_hellfire;
    let boy_price_limit = 90000;
    if !hf && (item._iCreateInfo & CF_BOY as u16) != 0 && item._iIvalue > boy_price_limit {
        return false;
    }
    let premium_price_limit = 140000;
    if !hf && (item._iCreateInfo & CF_SMITHPREMIUM as u16) != 0 && item._iIvalue > premium_price_limit {
        return false;
    }
    let smith_or_witch = (CF_SMITH | CF_WITCH) as u16;
    let smith_and_witch_price_limit = if hf { 200000 } else { 140000 };
    if (item._iCreateInfo & smith_or_witch) != 0 && item._iIvalue > smith_and_witch_price_limit {
        return false;
    }
    true
}

/// Original: `LoadMatchingItems` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadMatchingItems(LoadHelper &file, const Player &player, const int n, Item *pItem) sha=9bf58f9a6cb1
fn load_matching_items(ctx: &mut Ctx, file: &mut LoadHelper, player: &Player, items: &mut [Item]) {
    let mut hero_item = Item::default();
    for unpacked_item in items.iter_mut() {
        load_item_data(ctx, file, &mut hero_item);
        if unpacked_item.is_empty() || hero_item.is_empty() {
            continue;
        }
        if unpacked_item._iSeed != hero_item._iSeed {
            continue;
        }
        if hero_item.IDidx == IDI_EAR {
            continue;
        }
        if ctx.init.gb_is_multiplayer {
            if (hero_item.dwBuff & CF_HELLFIRE as u32) != (unpacked_item.dwBuff & CF_HELLFIRE as u32) {
                *unpacked_item = Item::default();
                crate::items::recreate_item(
                    ctx,
                    player,
                    unpacked_item,
                    hero_item.IDidx,
                    hero_item._iCreateInfo,
                    hero_item._iSeed,
                    hero_item._ivalue,
                    (hero_item.dwBuff & CF_HELLFIRE as u32) != 0,
                );
                unpacked_item._iIdentified = hero_item._iIdentified;
                unpacked_item._iMaxDur = hero_item._iMaxDur;
                unpacked_item._iDurability = crate::inv::clamp_durability(unpacked_item, hero_item._iDurability);
                unpacked_item._iMaxCharges = hero_item._iMaxCharges.clamp(0, unpacked_item._iMaxCharges);
                unpacked_item._iCharges = hero_item._iCharges.clamp(0, unpacked_item._iMaxCharges);
            }
            if !is_shop_price_valid(ctx, unpacked_item) {
                unpacked_item.clear();
                continue;
            }
            if ctx.init.gb_is_hellfire {
                unpacked_item._iPLToHit = crate::inv::clamp_to_hit(unpacked_item, hero_item._iPLToHit);
                unpacked_item._iMaxDam = crate::inv::clamp_max_dam(unpacked_item, hero_item._iMaxDam);
            }
        } else {
            *unpacked_item = hero_item.clone();
        }
    }
}

/// Original: `LoadDroppedItems` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadDroppedItems(LoadHelper &file, size_t savedItemCount) sha=85d29cddf187
fn load_dropped_items(ctx: &mut Ctx, file: &mut LoadHelper, saved_item_count: usize) {
    file.skip(MAXITEMS * 2);
    for (i, a) in ctx.items.ActiveItems.iter_mut().enumerate() {
        *a = i as u8;
    }
    ctx.items.ActiveItemCount = 0;
    for col in ctx.items.dItem.iter_mut() {
        col.fill(0);
    }
    for _ in 0..saved_item_count {
        let idx = ctx.items.ActiveItemCount as usize;
        let mut item = std::mem::take(&mut ctx.items.Items[idx]);
        load_item(ctx, file, &mut item);
        let empty = item.is_empty();
        let pos = item.position;
        ctx.items.Items[idx] = item;
        if !empty {
            ctx.items.ActiveItemCount += 1;
            ctx.items.dItem[pos.x as usize][pos.y as usize] = ctx.items.ActiveItemCount as i8;
        }
    }
}

/// Original: `getHellfireLevelType` (loadsave.cpp).
// @port loadsave.cpp|devilution::getHellfireLevelType(int type) sha=ca24f97b4f59
fn get_hellfire_level_type(ty: i32) -> i32 {
    if ty == DungeonType::Crypt as i32 {
        return DungeonType::Cathedral as i32;
    }
    if ty == DungeonType::Nest as i32 {
        return DungeonType::Caves as i32;
    }
    ty
}

/// Original: `SaveItem` (loadsave.cpp).
// @port loadsave.cpp|devilution::SaveItem(SaveHelper &file, const Item &item) sha=a3c48becba9e
fn save_item(ctx: &Ctx, file: &mut SaveHelper, item: &Item) {
    let mut idx = item.IDidx;
    if !ctx.init.gb_is_hellfire {
        idx = remap_item_idx_to_diablo(idx);
    }
    if ctx.init.gb_is_spawn {
        idx = remap_item_idx_to_spawn(idx);
    }
    let mut i_type = item._itype;
    if idx == -1 {
        idx = IDI_GOLD;
        i_type = ItemType::None;
    }

    file.u32(item._iSeed);
    file.i16(item._iCreateInfo as i16);
    file.skip(2);
    file.i32(i_type as i8 as i32);
    file.i32(item.position.x);
    file.i32(item.position.y);
    file.u32(item._iAnimFlag as u32);
    file.skip(4);
    file.i32(item.AnimInfo.numberOfFrames as i32);
    file.i32(item.AnimInfo.currentFrame as i32 + 1);
    file.i32(ItemAnimWidth);
    file.i32(crate::engine::calculate_width2(ItemAnimWidth));
    file.skip(4);
    file.u8(item._iSelFlag);
    file.skip(3);
    file.u32(item._iPostDraw as u32);
    file.u32(item._iIdentified as u32);
    file.i8(item._iMagical as i8);
    file.write_bytes(item._iName.bytes());
    file.write_bytes(item._iIName.bytes());
    file.i8(item._iLoc as i8);
    file.u8(item._iClass as u8);
    file.skip(1);
    file.i32(item._iCurs as i32);
    file.i32(item._ivalue);
    file.i32(item._iIvalue);
    file.i32(item._iMinDam as i32);
    file.i32(item._iMaxDam as i32);
    file.i32(item._iAC as i32);
    file.u32(item._iFlags.0);
    file.i32(item._iMiscId as i32);
    file.i32(item._iSpell as i8 as i32);
    file.i32(item._iCharges);
    file.i32(item._iMaxCharges);
    file.i32(item._iDurability);
    file.i32(item._iMaxDur);
    file.i32(item._iPLDam as i32);
    file.i32(item._iPLToHit as i32);
    file.i32(item._iPLAC as i32);
    file.i32(item._iPLStr as i32);
    file.i32(item._iPLMag as i32);
    file.i32(item._iPLDex as i32);
    file.i32(item._iPLVit as i32);
    file.i32(item._iPLFR as i32);
    file.i32(item._iPLLR as i32);
    file.i32(item._iPLMR as i32);
    file.i32(item._iPLMana as i32);
    file.i32(item._iPLHP as i32);
    file.i32(item._iPLDamMod as i32);
    file.i32(item._iPLGetHit as i32);
    file.i32(item._iPLLight as i32);
    file.i8(item._iSplLvlAdd);
    file.i8(item._iRequest as i8);
    file.skip(2);
    file.i32(item._iUid);
    file.i32(item._iFMinDam as i32);
    file.i32(item._iFMaxDam as i32);
    file.i32(item._iLMinDam as i32);
    file.i32(item._iLMaxDam as i32);
    file.i32(item._iPLEnAc as i32);
    file.i8(item._iPrePower as i8);
    file.i8(item._iSufPower as i8);
    file.skip(2);
    file.i32(item._iVAdd1);
    file.i32(item._iVMult1);
    file.i32(item._iVAdd2);
    file.i32(item._iVMult2);
    file.i8(item._iMinStr);
    file.u8(item._iMinMag);
    file.i8(item._iMinDex);
    file.skip(1);
    file.u32(item._iStatFlag as u32);
    file.i32(idx as i32);
    file.u32(item.dwBuff);
    if ctx.init.gb_is_hellfire {
        file.u32(item._iDamAcFlags.0 as u32);
    }
}

/// Original: `SavePlayer` (loadsave.cpp).
// @port loadsave.cpp|devilution::SavePlayer(SaveHelper &file, const Player &player) sha=2c348f0d2471
fn save_player(ctx: &Ctx, file: &mut SaveHelper, pnum: usize) {
    let player = &ctx.players.Players[pnum];
    let progress = ctx.nthread.ProgressToNextGameTick;
    file.i32(player._pmode as i32);
    for &step in player.walkpath.iter() {
        file.i8(step);
    }
    file.u8(player.plractive as u8);
    file.skip(2);
    file.i32(player.destAction as i32);
    file.i32(player.destParam1);
    file.i32(player.destParam2);
    file.i32(player.destParam3);
    file.i32(player.destParam4);
    file.u32(player.plrlevel as u32);
    file.i32(player.position.tile.x);
    file.i32(player.position.tile.y);
    file.i32(player.position.future.x);
    file.i32(player.position.future.y);

    let target = player.get_target_position();
    file.i32(target.x);
    file.i32(target.y);

    file.i32(player.position.last.x);
    file.i32(player.position.last.y);
    file.i32(player.position.old.x);
    file.i32(player.position.old.y);
    let mut offset = Displacement::default();
    let mut offset2 = Displacement::default();
    let mut velocity = Displacement::default();
    if player.is_walking() {
        offset = player.position.calculate_walking_offset(player._pdir, &player.AnimInfo, progress);
        offset2 = player.position.calculate_walking_offset_shifted8(player._pdir, &player.AnimInfo, progress);
        velocity = player.position.get_walking_velocity_shifted8(player._pdir, &player.AnimInfo);
    }
    // `DisplacementOf<int16_t>`
    file.i32(offset.delta_x as i16 as i32);
    file.i32(offset.delta_y as i16 as i32);
    file.i32(velocity.delta_x as i16 as i32);
    file.i32(velocity.delta_y as i16 as i32);
    file.i32(player._pdir as i32);
    file.skip(4);
    file.u32(player._pgfxnum as u32);
    file.skip(4);
    file.i32((player.AnimInfo.ticksPerFrame as i32 - 1).max(0));
    file.i32(player.AnimInfo.tickCounterOfCurrentFrame as i32);
    file.i32(player.AnimInfo.numberOfFrames as i32);
    file.i32(player.AnimInfo.currentFrame as i32 + 1);
    let anim_width = crate::player::get_sprite_width(ctx, pnum) as i32;
    file.i32(anim_width);
    file.i32(crate::engine::calculate_width2(anim_width));
    file.skip(4);
    file.i32(player.lightId);
    file.i32(1);

    file.i32(player.queuedSpell.spellId as i8 as i32);
    file.i8(player.queuedSpell.spellType as i8);
    file.i8(player.queuedSpell.spellFrom);
    file.skip(2);
    file.i32(player.inventorySpell as i8 as i32);
    file.skip(1);
    file.skip(3);
    file.i32(player._pRSpell as i8 as i32);
    file.i8(player._pRSplType as u8 as i8);
    file.skip(3);
    file.i32(player._pSBkSpell as i8 as i32);
    file.skip(1);

    for &spell_level in player._pSplLvl.iter() {
        file.u8(spell_level);
    }
    file.skip(7);
    file.u64(player._pMemSpells);
    file.u64(player._pAblSpells);
    file.u64(player._pScrlSpells);
    file.u8(player._pSpellFlags.0);
    file.skip(3);

    for i in 0..4 {
        file.i32(player._pSplHotKey[i] as i8 as i32);
    }
    for i in 0..4 {
        file.u8(player._pSplTHotKey[i] as u8);
    }

    file.i32(player.uses_ranged_weapon() as i32);
    file.u8(player._pBlockFlag as u8);
    file.u8(player._pInvincible as u8);
    file.i8(player._pLightRad);
    file.u8(player._pLvlChanging as u8);

    file.write_bytes(player._pName.bytes());
    file.i8(player._pClass as i8);
    file.skip(3);
    file.i32(player._pStrength);
    file.i32(player._pBaseStr);
    file.i32(player._pMagic);
    file.i32(player._pBaseMag);
    file.i32(player._pDexterity);
    file.i32(player._pBaseDex);
    file.i32(player._pVitality);
    file.i32(player._pBaseVit);
    file.i32(player._pStatPts);
    file.i32(player._pDamageMod);

    file.i32(player._pBaseToBlk);
    file.i32(player._pHPBase);
    file.i32(player._pMaxHPBase);
    file.i32(player._pHitPoints);
    file.i32(player._pMaxHP);
    file.skip(4);
    file.i32(player._pManaBase);
    file.i32(player._pMaxManaBase);
    file.i32(player._pMana);
    file.i32(player._pMaxMana);
    file.skip(4);
    file.i8(player._pLevel);
    file.i8(player._pMaxLvl);
    file.skip(2);
    file.u32(player._pExperience);
    file.skip(4);
    file.u32(player._pNextExper);
    file.i8(player._pArmorClass);
    file.i8(player._pMagResist);
    file.i8(player._pFireResist);
    file.i8(player._pLghtResist);
    file.i32(player._pGold);
    file.u32(player._pInfraFlag as u32);

    let mut temp_position_x = player.position.temp.x;
    let mut temp_position_y = player.position.temp.y;
    if player._pmode == PM_WALK_NORTHWARDS {
        temp_position_x -= player.position.tile.x;
        temp_position_y -= player.position.tile.y;
    }
    file.i32(temp_position_x);
    file.i32(temp_position_y);

    file.i32(player.tempDirection as i32);
    file.i32(player.queuedSpell.spellLevel);
    file.skip(4);
    file.i32(offset2.delta_x as i16 as i32);
    file.i32(offset2.delta_y as i16 as i32);
    file.skip(4);
    let levels = ctx.loadsave.giNumberOfLevels as usize;
    for i in 0..levels {
        file.u8(player._pLvlVisited[i] as u8);
    }
    for i in 0..levels {
        file.u8(player._pSLvlVisited[i] as u8);
    }

    file.skip(2);

    file.skip(4);
    file.skip(4 * 8);
    file.i32(player._pNFrames as i32);
    file.skip(4);
    file.skip(4 * 8);
    file.i32(player._pWFrames as i32);
    file.skip(4);
    file.skip(4 * 8);
    file.i32(player._pAFrames as i32);
    file.skip(4);
    file.i32(player._pAFNum as i32);
    file.skip(4 * 8);
    file.skip(4 * 8);
    file.skip(4 * 8);
    file.i32(player._pSFrames as i32);
    file.skip(4);
    file.i32(player._pSFNum as i32);
    file.skip(4 * 8);
    file.i32(player._pHFrames as i32);
    file.skip(4);
    file.skip(4 * 8);
    file.i32(player._pDFrames as i32);
    file.skip(4);
    file.skip(4 * 8);
    file.i32(player._pBFrames as i32);
    file.skip(4);

    for item in player.InvBody.iter() {
        save_item(ctx, file, item);
    }
    for item in player.InvList.iter() {
        save_item(ctx, file, item);
    }
    file.i32(player._pNumInv);
    for &cell in player.InvGrid.iter() {
        file.i8(cell);
    }
    for item in player.SpdList.iter() {
        save_item(ctx, file, item);
    }
    save_item(ctx, file, &player.HoldItem);

    file.i32(player._pIMinDam);
    file.i32(player._pIMaxDam);
    file.i32(player._pIAC);
    file.i32(player._pIBonusDam);
    file.i32(player._pIBonusToHit);
    file.i32(player._pIBonusAC);
    file.i32(player._pIBonusDamMod);
    file.skip(4);

    file.u64(player._pISpells);
    file.i32(player._pIFlags.0 as i32);
    file.i32(player._pIGetHit);

    file.i8(player._pISplLvlAdd);
    file.skip(1);
    file.skip(2);
    file.skip(4);
    file.i32(player._pIEnAc);
    file.i32(player._pIFMinDam);
    file.i32(player._pIFMaxDam);
    file.i32(player._pILMinDam);
    file.i32(player._pILMaxDam);
    file.i32(player._pOilType as i32);
    file.u8(player.pTownWarps);
    file.u8(player.pDungMsgs);
    file.u8(player.pLvlLoad);
    if ctx.init.gb_is_hellfire {
        file.u8(player.pDungMsgs2);
    } else {
        file.u8(0);
    }
    file.u8(player.pManaShield as u8);
    file.u8(player.pOriginalCathedral as u8);
    file.skip(2);
    file.u16(player.wReflections);
    file.skip(14);

    file.u32(player.pDiabloKillLevel as u32);
    file.u32(ctx.multi.sgGameInitInfo.nDifficulty as u32);
    file.u32(player.pDamAcFlags.0 as u32);
    file.skip(20);
}

/// Original: `SaveMonster` (loadsave.cpp).
// @port loadsave.cpp|devilution::SaveMonster(SaveHelper *file, Monster &monster, MonsterConversionData *monsterConversionData = nullptr) sha=2e440e3190ff
fn save_monster(ctx: &Ctx, file: &mut SaveHelper, m: usize, conversion: Option<&MonsterConversionData>) {
    let monster = &ctx.monster.Monsters[m];
    let progress = ctx.nthread.ProgressToNextGameTick;
    let difficulty = ctx.multi.sgGameInitInfo.nDifficulty;
    file.i32(monster.levelType as i32);
    file.i32(monster.mode as i32);
    file.u8(monster.goal as u8);
    file.skip(3);
    file.i32(monster.goalVar1 as i32);
    file.i32(monster.goalVar2 as i32);
    file.i32(monster.goalVar3 as i32);
    file.skip(4);
    file.u8(monster.pathCount);
    file.skip(3);
    file.i32(monster.position.tile.x);
    file.i32(monster.position.tile.y);
    file.i32(monster.position.future.x);
    file.i32(monster.position.future.y);
    file.i32(monster.position.old.x);
    file.i32(monster.position.old.y);
    let mut offset = Displacement::default();
    let mut offset2 = Displacement::default();
    let mut velocity = Displacement::default();
    if crate::monster::is_walking(ctx, m) {
        offset = monster.position.calculate_walking_offset(monster.direction, &monster.animInfo, progress);
        offset2 = monster.position.calculate_walking_offset_shifted4(monster.direction, &monster.animInfo, progress);
        velocity = monster.position.get_walking_velocity_shifted4(monster.direction, &monster.animInfo);
    }
    file.i32(offset.delta_x as i16 as i32);
    file.i32(offset.delta_y as i16 as i32);
    file.i32(velocity.delta_x as i16 as i32);
    file.i32(velocity.delta_y as i16 as i32);
    file.i32(monster.direction as i32);
    file.i32(monster.enemy as i32);
    file.u8(monster.enemyPosition.x as u8);
    file.u8(monster.enemyPosition.y as u8);
    file.skip(2);

    file.skip(4);
    file.i32(monster.animInfo.ticksPerFrame as i32);
    file.i32(monster.animInfo.tickCounterOfCurrentFrame as i32);
    file.i32(monster.animInfo.numberOfFrames as i32);
    file.i32(monster.animInfo.currentFrame as i32 + 1);
    file.skip(4);
    file.u32(monster.isInvalid as u32);
    file.i32(monster.var1 as i32);
    file.i32(monster.var2 as i32);
    file.i32(monster.var3 as i32);
    file.i32(monster.position.temp.x);
    file.i32(monster.position.temp.y);
    file.i32(offset2.delta_x as i16 as i32);
    file.i32(offset2.delta_y as i16 as i32);
    file.skip(4);
    file.i32(monster.maxHitPoints);
    file.i32(monster.hitPoints);

    file.u8(monster.ai as i8 as u8);
    file.u8(monster.intelligence);
    file.skip(2);
    file.u32(monster.flags);
    file.u8(monster.activeForTicks);
    file.skip(3);
    file.skip(4);
    file.i32(monster.position.last.x);
    file.i32(monster.position.last.y);
    file.u32(monster.rndItemSeed);
    file.u32(monster.aiSeed);
    file.skip(4);

    file.u8(monster.uniqueType.0.wrapping_add(1));
    file.u8(monster.uniqTrans);
    file.i8(monster.corpseId);

    file.i8(monster.whoHit);
    match conversion {
        Some(c) => file.i8(c.monster_level),
        None => file.i8(crate::monster::monster_level(ctx, m, difficulty) as i8),
    }
    file.skip(1);
    match conversion {
        Some(c) => file.u16(c.experience),
        None => file.u16(crate::monster::monster_exp(ctx, m, difficulty).min(u16::MAX as u32) as u16),
    }

    file.u8(monster.toHit.min(u8::MAX as u16) as u8);
    file.u8(monster.minDamage);
    file.u8(monster.maxDamage);
    match conversion {
        Some(c) => file.u8(c.to_hit_special),
        None => file.u8((crate::monster::to_hit_special(ctx, m, difficulty) as u16).min(u8::MAX as u16) as u8),
    }
    file.u8(monster.minDamageSpecial);
    file.u8(monster.maxDamageSpecial);
    file.u8(monster.armorClass);
    file.skip(1);
    file.u16(monster.resistance);
    file.skip(2);

    file.i32(if monster.talkMsg == TEXT_NONE { 0 } else { monster.talkMsg as i32 });
    file.u8(if monster.leader == Monster::NoLeader { 0 } else { monster.leader });
    file.u8(monster.leaderRelation as u8);
    file.u8(monster.packSize);
    if monster.lightId as i32 == NO_LIGHT {
        file.i8(0);
    } else {
        file.i8(monster.lightId);
    }
}

/// Original: `SaveMissile` (loadsave.cpp).
// @port loadsave.cpp|devilution::SaveMissile(SaveHelper *file, const Missile &missile) sha=8dd26af21e22
fn save_missile(file: &mut SaveHelper, missile: &Missile) {
    file.i32(missile._mitype as i8 as i32);
    file.i32(missile.position.tile.x);
    file.i32(missile.position.tile.y);
    file.i32(missile.position.offset.delta_x);
    file.i32(missile.position.offset.delta_y);
    file.i32(missile.position.velocity.delta_x);
    file.i32(missile.position.velocity.delta_y);
    file.i32(missile.position.start.x);
    file.i32(missile.position.start.y);
    file.i32(missile.position.traveled.delta_x);
    file.i32(missile.position.traveled.delta_y);
    file.i32(missile._mimfnum);
    file.i32(missile._mispllvl);
    file.u32(missile._miDelFlag as u32);
    file.u8(missile._miAnimType as u8);
    file.skip(3);
    file.i32(missile._miAnimFlags.0 as i32);
    file.skip(4);
    file.i32(missile._miAnimDelay);
    file.i32(missile._miAnimLen);
    file.i32(missile._miAnimWidth as i32);
    file.i32(missile._miAnimWidth2 as i32);
    file.i32(missile._miAnimCnt);
    file.i32(missile._miAnimAdd);
    file.i32(missile._miAnimFrame);
    file.u32(missile._miDrawFlag as u32);
    file.u32(missile._miLightFlag as u32);
    file.u32(missile._miPreFlag as u32);
    file.u32(missile._miUniqTrans);
    file.i32(missile._mirange);
    file.i32(missile._misource);
    file.i32(missile._micaster as i32);
    file.i32(missile._midam);
    file.u32(missile._miHitFlag as u32);
    file.i32(missile._midist);
    file.i32(missile._mlid);
    file.i32(missile._mirnd);
    file.i32(missile.var1);
    file.i32(missile.var2);
    file.i32(missile.var3);
    file.i32(missile.var4);
    file.i32(missile.var5);
    file.i32(missile.var6);
    file.i32(missile.var7);
    file.u32(missile.limitReached as u32);
}

/// Original: `ConvertToHellfireObject` (loadsave.cpp).
// @port loadsave.cpp|devilution::ConvertToHellfireObject(_object_id type) sha=95a776a7af30
fn convert_to_hellfire_object(ctx: &Ctx, ty: _object_id) -> _object_id {
    if ctx.gendung.leveltype == DungeonType::Nest {
        match ty {
            OBJ_POD => return OBJ_BARREL,
            OBJ_PODEX => return OBJ_BARRELEX,
            _ => {}
        }
    }
    if ctx.gendung.leveltype == DungeonType::Crypt {
        match ty {
            OBJ_URN => return OBJ_BARREL,
            OBJ_URNEX => return OBJ_BARRELEX,
            OBJ_L5BOOKS => return OBJ_STORYBOOK,
            OBJ_L5CANDLE => return OBJ_STORYCANDLE,
            OBJ_L5LDOOR => return OBJ_L1LDOOR,
            OBJ_L5RDOOR => return OBJ_L1RDOOR,
            OBJ_L5LEVER => return OBJ_LEVER,
            OBJ_L5SARC => return OBJ_SARC,
            _ => {}
        }
    }
    ty
}

/// Original: `SaveObject` (loadsave.cpp).
// @port loadsave.cpp|devilution::SaveObject(SaveHelper &file, const Object &object) sha=fb67b42b11ac
fn save_object(ctx: &Ctx, file: &mut SaveHelper, object: &Object) {
    file.i32(convert_to_hellfire_object(ctx, object._otype) as i32);
    file.i32(object.position.x);
    file.i32(object.position.y);
    file.u32(object.applyLighting as u32);
    file.u32((object._oAnimFlag != 0) as u32);
    file.skip(4);
    file.i32(object._oAnimDelay);
    file.i32(object._oAnimCnt);
    file.u32(object._oAnimLen);
    file.u32(object._oAnimFrame);
    file.i32(object._oAnimWidth as i32);
    file.i32(crate::engine::calculate_width2(object._oAnimWidth as i32));
    file.u32(object._oDelFlag as u32);
    file.i8(object._oBreak);
    file.skip(3);
    file.u32(object._oSolidFlag as u32);
    file.u32(object._oMissFlag as u32);

    file.i8(object._oSelFlag as i8);
    file.skip(3);
    file.u32(object._oPreFlag as u32);
    file.u32(object._oTrapFlag as u32);
    file.u32(object._oDoorFlag as u32);
    file.i32(object._olid);
    file.u32(object._oRndSeed);

    let mut var1 = object._oVar1;
    if matches!(
        object._otype,
        OBJ_L1LIGHT
            | OBJ_SKFIRE
            | OBJ_CANDLE1
            | OBJ_CANDLE2
            | OBJ_BOOKCANDLE
            | OBJ_STORYCANDLE
            | OBJ_L5CANDLE
            | OBJ_TORCHL
            | OBJ_TORCHR
            | OBJ_TORCHL2
            | OBJ_TORCHR2
            | OBJ_BCROSS
            | OBJ_TBCROSS
    ) && var1 != -1
    {
        var1 = 0;
    }
    file.i32(var1);
    file.i32(object._oVar2);
    file.i32(object._oVar3);
    file.i32(object._oVar4);
    file.i32(object._oVar5);
    file.u32(object._oVar6);
    file.i32(object.bookMessage as i32);
    file.i32(object._oVar8);
}

/// Original: `SaveQuest` (loadsave.cpp).
// @port loadsave.cpp|devilution::SaveQuest(SaveHelper *file, int i) sha=020677923d52
fn save_quest(ctx: &Ctx, file: &mut SaveHelper, i: usize) {
    let quest = &ctx.quests.Quests[i];
    file.u8(quest._qlevel);
    file.u8(quest._qidx as u8);
    file.u8(quest._qactive as u8);
    file.u8(quest._qlvltype as i8 as u8);
    file.i32(quest.position.x);
    file.i32(quest.position.y);
    file.u8(quest._qslvl as u8);
    file.u8(quest._qidx as u8);
    if ctx.init.gb_is_hellfire {
        file.skip(2);
        file.i32(quest._qmsg as i32);
    } else {
        file.u8(quest._qmsg as u8);
    }
    file.u8(quest._qvar1);
    file.u8(quest._qvar2);
    file.skip(2);
    if !ctx.init.gb_is_hellfire {
        file.skip(1);
    }
    file.u32(quest._qlog as u32);

    file.i32_be(ctx.quests.ReturnLvlPosition.x);
    file.i32_be(ctx.quests.ReturnLvlPosition.y);
    file.i32_be(ctx.quests.ReturnLevel);
    file.i32_be(ctx.quests.ReturnLevelType as i32);
    file.skip(4);
}

/// Original: `SaveLighting` (loadsave.cpp).
// @port loadsave.cpp|devilution::SaveLighting(SaveHelper *file, Light *pLight, bool vision = false) sha=be017759c17b
fn save_lighting(file: &mut SaveHelper, light: &Light, vision: bool) {
    file.i32(light.position.tile.x);
    file.i32(light.position.tile.y);
    file.i32(light.radius as i32);
    file.i32(vision as i32);
    file.u32(light.isInvalid as u32);
    file.u32(light.hasChanged as u32);
    file.skip(4);
    file.i32(light.position.old.x);
    file.i32(light.position.old.y);
    file.i32(light.oldRadius as i32);
    file.i32(light.position.offset.delta_x);
    file.i32(light.position.offset.delta_y);
    file.u32(vision as u32);
}

/// Original: `SavePortal` (loadsave.cpp).
// @port loadsave.cpp|devilution::SavePortal(SaveHelper *file, int i) sha=7db964189695
fn save_portal(ctx: &Ctx, file: &mut SaveHelper, i: usize) {
    let portal = &ctx.portal.Portals[i];
    file.u32(portal.open as u32);
    file.i32(portal.position.x);
    file.i32(portal.position.y);
    file.i32(portal.level);
    file.i32(if portal.setlvl { portal.ltype as i32 } else { get_hellfire_level_type(portal.ltype as i32) });
    file.u32(portal.setlvl as u32);
}

/// Original: `SaveDroppedItems` (loadsave.cpp).
// @port loadsave.cpp|devilution::SaveDroppedItems(SaveHelper &file) sha=56778d568a5e
fn save_dropped_items(ctx: &Ctx, file: &mut SaveHelper) -> HashMap<u8, u8> {
    let count = ctx.items.ActiveItemCount as usize;
    for i in 0..MAXITEMS {
        file.u8(i as u8);
    }
    for i in 0..MAXITEMS {
        file.u8(((i + count) % MAXITEMS) as u8);
    }
    let mut item_indexes = HashMap::new();
    item_indexes.insert(0u8, 0u8);
    for i in 0..count {
        let a = ctx.items.ActiveItems[i];
        item_indexes.insert(a + 1, i as u8 + 1);
        save_item(ctx, file, &ctx.items.Items[a as usize]);
    }
    item_indexes
}

/// Original: `SaveDroppedItemLocations` (loadsave.cpp).
// @port loadsave.cpp|devilution::SaveDroppedItemLocations(SaveHelper &file, const std::unordered_map<uint8_t, uint8_t> &itemIndexes) sha=0a4c2341d9fb
fn save_dropped_item_locations(ctx: &Ctx, file: &mut SaveHelper, item_indexes: &HashMap<u8, u8>) {
    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            // `std::unordered_map::at` throws for a missing key
            file.u8(*item_indexes.get(&(ctx.items.dItem[i][j] as u8)).expect("dItem index not saved"));
        }
    }
}

const VersionAdditionalMissiles: u32 = 0;

/// Original: `SaveAdditionalMissiles` (loadsave.cpp).
// @port loadsave.cpp|devilution::SaveAdditionalMissiles(SaveWriter &saveWriter) sha=362391e845d3
fn save_additional_missiles(ctx: &Ctx, save_writer: &mut SaveWriter) {
    const BYTES_WRITTEN_BY_SAVE_MISSILE: usize = 180;
    let n = ctx.missiles.Missiles.len();
    let missile_count_additional = if n > MaxMissilesForSaveGame { n - MaxMissilesForSaveGame } else { 0 };
    let mut file = SaveHelper::new("additionalMissiles", 4 + 4 + missile_count_additional * BYTES_WRITTEN_BY_SAVE_MISSILE);
    file.u32(VersionAdditionalMissiles);
    file.u32(missile_count_additional as u32);
    if missile_count_additional > 0 {
        for missile in ctx.missiles.Missiles.iter().skip(MaxMissilesForSaveGame) {
            save_missile(&mut file, missile);
        }
    }
    file.finish(ctx, save_writer);
}

/// Original: `LoadAdditionalMissiles` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadAdditionalMissiles() sha=7780e6f606c5
fn load_additional_missiles(ctx: &mut Ctx) {
    let archive = crate::pfile::open_save_archive(ctx, ctx.menu.g_save_number);
    let mut file = LoadHelper::new(ctx, archive, "additionalMissiles");
    if !file.is_valid(1) {
        return;
    }
    let loaded_version = file.next_u32();
    if loaded_version > VersionAdditionalMissiles {
        return;
    }
    let missile_count_additional = file.next_u32();
    for _ in 0..missile_count_additional {
        load_missile(ctx, &mut file);
    }
}

/// Original: `SaveLevel(SaveWriter &saveWriter, LevelConversionData *levelConversionData)`.
// @port loadsave.cpp|devilution::SaveLevel(SaveWriter &saveWriter, LevelConversionData *levelConversionData) sha=7c2f23bf83ad
fn save_level_with(ctx: &mut Ctx, save_writer: &mut SaveWriter, level_conversion_data: Option<&LevelConversionData>) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let (tile, rad) = (ctx.players.Players[me].position.tile, ctx.players.Players[me]._pLightRad);
    crate::lighting::do_un_vision(ctx, tile, rad as u8);

    if ctx.gendung.leveltype == DungeonType::Town {
        ctx.diablo.glSeedTbl[0] = ctx.rng.advance_rnd_seed() as u32;
    }

    let sz_name = get_temp_level_names(ctx);
    let mut file = SaveHelper::new(&sz_name, 256 * 1024);
    let town = ctx.gendung.leveltype == DungeonType::Town;

    if !town {
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                file.i8(ctx.gendung.dCorpse[i][j]);
            }
        }
    }

    file.i32_be(ctx.monster.ActiveMonsterCount as i32);
    file.i32_be(ctx.items.ActiveItemCount as i32);
    file.i32_be(ctx.objects.ActiveObjectCount);

    if !town {
        for &monster_id in ctx.monster.ActiveMonsters.iter() {
            file.i32_be(monster_id);
        }
        for i in 0..ctx.monster.ActiveMonsterCount {
            let m = ctx.monster.ActiveMonsters[i] as usize;
            let conversion = level_conversion_data.map(|l| &l.monster_conversion_data[m]);
            save_monster(ctx, &mut file, m, conversion);
        }
        for &object_id in ctx.objects.ActiveObjects.iter() {
            file.i8(object_id as i8);
        }
        for &object_id in ctx.objects.AvailableObjects.iter() {
            file.i8(object_id as i8);
        }
        for i in 0..ctx.objects.ActiveObjectCount as usize {
            let oi = ctx.objects.ActiveObjects[i] as usize;
            save_object(ctx, &mut file, &ctx.objects.Objects[oi]);
        }
    }

    let item_indexes = save_dropped_items(ctx, &mut file);

    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            file.u8((ctx.gendung.dFlags[i][j].0 & DungeonFlag::SavedFlags.0) as u8);
        }
    }
    save_dropped_item_locations(ctx, &mut file, &item_indexes);

    if !town {
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                file.i32_be(ctx.gendung.dMonster[i][j] as i32);
            }
        }
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                file.i8(ctx.gendung.dObject[i][j]);
            }
        }
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                file.u8(ctx.gendung.dLight[i][j]);
            }
        }
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                file.u8(ctx.gendung.dPreLight[i][j]);
            }
        }
        for j in 0..DMAXY {
            for i in 0..DMAXX {
                file.u8(ctx.automap.AutomapView[i][j]);
            }
        }
    }

    file.finish(ctx, save_writer);

    let (setlevel, currlevel, setlvlnum) = (ctx.gendung.setlevel, ctx.gendung.currlevel, ctx.gendung.setlvlnum);
    let player = &mut ctx.players.Players[me];
    if !setlevel {
        player._pLvlVisited[currlevel as usize] = true;
    } else {
        player._pSLvlVisited[setlvlnum as usize] = true;
    }
}

/// Original: `LoadLevel(LevelConversionData *levelConversionData)`.
// @port loadsave.cpp|devilution::LoadLevel(LevelConversionData *levelConversionData) sha=3c029635e588
fn load_level_with(ctx: &mut Ctx, mut level_conversion_data: Option<&mut LevelConversionData>) {
    let archive = crate::pfile::open_save_archive(ctx, ctx.menu.g_save_number);
    let mut sz_name = get_temp_level_names(ctx);
    if archive.as_ref().map_or(true, |a| !a.has_file(&sz_name)) {
        sz_name = get_perm_level_names(ctx);
    }
    let mut file = LoadHelper::new(ctx, archive, &sz_name);
    if !file.is_valid(1) {
        crate::appfat::app_fatal(ctx, &tr("Unable to open save file archive"));
    }
    let town = ctx.gendung.leveltype == DungeonType::Town;

    if !town {
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                ctx.gendung.dCorpse[i][j] = file.next_i8();
            }
        }
        crate::dead::move_lights_to_corpses(ctx);
    }

    ctx.monster.ActiveMonsterCount = file.next_i32_be() as usize;
    let saved_item_count = file.next_u32_be();
    ctx.objects.ActiveObjectCount = file.next_i32_be();

    if !town {
        for i in 0..MaxMonsters {
            ctx.monster.ActiveMonsters[i] = file.next_i32_be();
        }
        for i in 0..ctx.monster.ActiveMonsterCount {
            let m = ctx.monster.ActiveMonsters[i] as usize;
            let conversion = level_conversion_data.as_deref_mut().map(|l| &mut l.monster_conversion_data[m]);
            load_monster(ctx, &mut file, m, conversion);
            let monster = &ctx.monster.Monsters[m];
            if monster.is_unique() && monster.lightId as i32 != NO_LIGHT {
                let l = monster.lightId as usize;
                ctx.lighting.Lights[l].isInvalid = false;
            }
        }
        if !ctx.loadsave.gbSkipSync {
            for i in 0..ctx.monster.ActiveMonsterCount {
                let m = ctx.monster.ActiveMonsters[i] as usize;
                crate::monster::sync_monster_anim(ctx, m);
            }
        }
        for i in 0..ctx.objects.ActiveObjects.len() {
            ctx.objects.ActiveObjects[i] = file.next_i8() as i32;
        }
        for i in 0..ctx.objects.AvailableObjects.len() {
            ctx.objects.AvailableObjects[i] = file.next_i8() as i32;
        }
        for i in 0..ctx.objects.ActiveObjectCount as usize {
            let oi = ctx.objects.ActiveObjects[i] as usize;
            load_object(ctx, &mut file, oi);
        }
        if !ctx.loadsave.gbSkipSync {
            for i in 0..ctx.objects.ActiveObjectCount as usize {
                let oi = ctx.objects.ActiveObjects[i] as usize;
                crate::objects::sync_object_anim(ctx, oi);
            }
        }
    }

    load_dropped_items(ctx, &mut file, saved_item_count as usize);

    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            ctx.gendung.dFlags[i][j] = DungeonFlag(file.next_u8() & DungeonFlag::LoadedFlags.0);
        }
    }

    file.skip(MAXDUNX * MAXDUNY);

    if !town {
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                ctx.gendung.dMonster[i][j] = file.next_i32_be() as i16;
            }
        }
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                ctx.gendung.dObject[i][j] = file.next_i8();
            }
        }
        file.skip(MAXDUNY * MAXDUNX);
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                ctx.gendung.dPreLight[i][j] = file.next_u8();
            }
        }
        for j in 0..DMAXY {
            for i in 0..DMAXX {
                let automap_view = file.next_u8();
                ctx.automap.AutomapView[i][j] = if automap_view == MAP_EXP_OLD { MAP_EXP_SELF } else { automap_view };
            }
        }
        *ctx.gendung.dLight = *ctx.gendung.dPreLight;
        let my = ctx.players.MyPlayerId;
        let (light_id, tile) = (ctx.players.Players[my].lightId, ctx.players.Players[my].position.tile);
        crate::lighting::change_light_xy(ctx, light_id, tile);
    } else {
        for col in ctx.gendung.dLight.iter_mut() {
            col.fill(0);
        }
    }

    if !ctx.loadsave.gbSkipSync {
        crate::automap::automap_zoom_reset(ctx);
        crate::quests::resync_quests(ctx);
        crate::missiles::redo_missile_flags(ctx);
        ctx.lighting.UpdateLighting = true;
    }

    for pnum in 0..ctx.players.Players.len() {
        if ctx.players.Players[pnum].plractive && crate::player::is_on_active_level(ctx, pnum) {
            let l = ctx.players.Players[pnum].lightId as usize;
            ctx.lighting.Lights[l].hasChanged = true;
        }
    }
}

const DiabloItemSaveSize: usize = 368;
const HellfireItemSaveSize: usize = 372;

/// Original: `devilution::ConvertLevels` (loadsave.cpp).
// @port loadsave.cpp|devilution::ConvertLevels(SaveWriter &saveWriter) sha=1d17b36f53d1
pub fn convert_levels(ctx: &mut Ctx, save_writer: &mut SaveWriter) {
    let tmp_setlevel = ctx.gendung.setlevel;
    let tmp_setlvlnum = ctx.gendung.setlvlnum;
    let tmp_currlevel = ctx.gendung.currlevel;
    let tmp_leveltype = ctx.gendung.leveltype;

    ctx.loadsave.gbSkipSync = true;

    ctx.gendung.setlevel = false;
    for i in 0..ctx.loadsave.giNumberOfLevels as i32 {
        ctx.gendung.currlevel = i as u8;
        if !level_file_exists(ctx, save_writer) {
            continue;
        }
        ctx.gendung.leveltype = get_level_type(ctx.gendung.currlevel as i32);
        let mut level_conversion_data = LevelConversionData::default();
        load_level_with(ctx, Some(&mut level_conversion_data));
        save_level_with(ctx, save_writer, Some(&level_conversion_data));
    }

    ctx.gendung.setlevel = true;
    for q in 0..ctx.quests.Quests.len() {
        let quest = ctx.quests.Quests[q];
        if quest._qactive == QUEST_NOTAVAIL {
            continue;
        }
        ctx.gendung.leveltype = quest._qlvltype;
        if ctx.gendung.leveltype == DungeonType::None {
            continue;
        }
        ctx.gendung.setlvlnum = quest._qslvl;
        if !level_file_exists(ctx, save_writer) {
            continue;
        }
        let mut level_conversion_data = LevelConversionData::default();
        load_level_with(ctx, Some(&mut level_conversion_data));
        save_level_with(ctx, save_writer, Some(&level_conversion_data));
    }

    ctx.loadsave.gbSkipSync = false;

    ctx.gendung.setlevel = tmp_setlevel;
    ctx.gendung.setlvlnum = tmp_setlvlnum;
    ctx.gendung.currlevel = tmp_currlevel;
    ctx.gendung.leveltype = tmp_leveltype;
}

/// Original: `devilution::RemoveInvalidItem` (loadsave.cpp).
// @port loadsave.cpp|devilution::RemoveInvalidItem(Item &item) sha=66f5c5414b0a
pub fn remove_invalid_item(ctx: &Ctx, item: &mut Item) {
    let mut is_invalid = !crate::items::is_item_available(ctx, item.IDidx as i32) || !crate::items::is_unique_available(ctx, item._iUid);
    if !ctx.init.gb_is_hellfire {
        is_invalid = is_invalid || (item._itype == ItemType::Staff && crate::spells::get_spell_staff_level(ctx, item._iSpell) == -1);
        is_invalid = is_invalid || (item._iMiscId == IMISC_BOOK && crate::spells::get_spell_book_level(ctx, item._iSpell) == -1);
        is_invalid = is_invalid || item._iDamAcFlags != ItemSpecialEffectHf::None;
        is_invalid = is_invalid || item._iPrePower as i32 > IPL_LASTDIABLO as i32;
        is_invalid = is_invalid || item._iSufPower as i32 > IPL_LASTDIABLO as i32;
    }
    if is_invalid {
        item.clear();
    }
}

/// Original: `devilution::RemapItemIdxFromDiablo` (loadsave.cpp).
// @port loadsave.cpp|devilution::RemapItemIdxFromDiablo(_item_indexes i) sha=ca61ae3c3622
pub fn remap_item_idx_from_diablo(i: _item_indexes) -> _item_indexes {
    let mut i = i as i32;
    if i == IDI_SORCERER as i32 {
        return IDI_SORCERER_DIABLO;
    }
    if i >= 156 {
        i += 5; // Hellfire exclusive items
    }
    if i >= 88 {
        i += 1; // Scroll of Search
    }
    if i >= 83 {
        i += 4; // Oils
    }
    i as _item_indexes
}

/// Original: `devilution::RemapItemIdxToDiablo` (loadsave.cpp).
// @port loadsave.cpp|devilution::RemapItemIdxToDiablo(_item_indexes i) sha=2c7b3e61041e
pub fn remap_item_idx_to_diablo(i: _item_indexes) -> _item_indexes {
    let mut i = i as i32;
    if i == IDI_SORCERER_DIABLO as i32 {
        return IDI_SORCERER;
    }
    if (83..=86).contains(&i) || i == 92 || i >= 161 {
        return -1; // Hellfire exclusive items
    }
    if i >= 93 {
        i -= 1; // Scroll of Search
    }
    if i >= 87 {
        i -= 4; // Oils
    }
    i as _item_indexes
}

/// Original: `devilution::RemapItemIdxFromSpawn` (loadsave.cpp).
// @port loadsave.cpp|devilution::RemapItemIdxFromSpawn(_item_indexes i) sha=d52799cf14a5
pub fn remap_item_idx_from_spawn(i: _item_indexes) -> _item_indexes {
    let mut i = i as i32;
    if i >= 62 {
        i += 9; // Medium and heavy armors
    }
    if i >= 96 {
        i += 1; // Scroll of Stone Curse
    }
    if i >= 98 {
        i += 1; // Scroll of Guardian
    }
    if i >= 99 {
        i += 1; // Scroll of ...
    }
    if i >= 101 {
        i += 1; // Scroll of Golem
    }
    if i >= 102 {
        i += 1; // Scroll of None
    }
    if i >= 104 {
        i += 1; // Scroll of Apocalypse
    }
    i as _item_indexes
}

/// Original: `devilution::RemapItemIdxToSpawn` (loadsave.cpp).
// @port loadsave.cpp|devilution::RemapItemIdxToSpawn(_item_indexes i) sha=b5f9bda16a70
pub fn remap_item_idx_to_spawn(i: _item_indexes) -> _item_indexes {
    let mut i = i as i32;
    if i >= 104 {
        i -= 1; // Scroll of Apocalypse
    }
    if i >= 102 {
        i -= 1; // Scroll of None
    }
    if i >= 101 {
        i -= 1; // Scroll of Golem
    }
    if i >= 99 {
        i -= 1; // Scroll of ...
    }
    if i >= 98 {
        i -= 1; // Scroll of Guardian
    }
    if i >= 96 {
        i -= 1; // Scroll of Stone Curse
    }
    if i >= 71 {
        i -= 9; // Medium and heavy armors
    }
    i as _item_indexes
}

/// Original: `devilution::IsHeaderValid` (loadsave.cpp).
// @port loadsave.cpp|devilution::IsHeaderValid(uint32_t magicNumber) sha=a7faae101975
pub fn is_header_valid(ctx: &mut Ctx, magic_number: u32) -> bool {
    let le = |s: &[u8; 4]| u32::from_le_bytes(*s);
    ctx.loadsave.gbIsHellfireSaveGame = false;
    if magic_number == le(b"SHAR") {
        return true;
    }
    if magic_number == le(b"SHLF") {
        ctx.loadsave.gbIsHellfireSaveGame = true;
        return true;
    }
    if !ctx.init.gb_is_spawn && magic_number == le(b"RETL") {
        return true;
    }
    if !ctx.init.gb_is_spawn && magic_number == le(b"HELF") {
        ctx.loadsave.gbIsHellfireSaveGame = true;
        return true;
    }
    false
}

/// Original: `HotkeysSize` (loadsave.cpp).
// @port loadsave.cpp|devilution::HotkeysSize(size_t nHotkeys = NumHotkeys) sha=4bfb2ca80b2a
fn hotkeys_size(n_hotkeys: usize) -> usize {
    1 + (n_hotkeys * 4) + n_hotkeys + 4 + 1
}

/// Original: `devilution::LoadHotkeys` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadHotkeys() sha=3882f2e133c6
pub fn load_hotkeys(ctx: &mut Ctx) {
    let archive = crate::pfile::open_save_archive(ctx, ctx.menu.g_save_number);
    let mut file = LoadHelper::new(ctx, archive, "hotkeys");
    if !file.is_valid(1) {
        return;
    }
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let my_player = &mut ctx.players.Players[me];
    let mut n_hotkeys = 4usize;
    my_player._pSplHotKey.fill(SpellID::Invalid);
    my_player._pSplTHotKey.fill(SpellType::Invalid);
    if file.is_valid(hotkeys_size(n_hotkeys)) {
        n_hotkeys = file.next_u8() as usize;
    }
    for i in 0..n_hotkeys {
        if i < NumHotkeys {
            my_player._pSplHotKey[i] = SpellID::from_raw(file.next_i32() as i8);
        } else {
            file.skip(4);
        }
    }
    for i in 0..n_hotkeys {
        if i < NumHotkeys {
            my_player._pSplTHotKey[i] = SpellType::from_raw(file.next_u8());
        } else {
            file.skip(1);
        }
    }
    my_player._pRSpell = SpellID::from_raw(file.next_i32() as i8);
    my_player._pRSplType = SpellType::from_raw(file.next_u8());
}

/// Original: `devilution::SaveHotkeys` (loadsave.cpp).
// @port loadsave.cpp|devilution::SaveHotkeys(SaveWriter &saveWriter, const Player &player) sha=59aadb334017
pub fn save_hotkeys(ctx: &Ctx, save_writer: &mut SaveWriter, player: &Player) {
    let mut file = SaveHelper::new("hotkeys", hotkeys_size(NumHotkeys));
    file.u8(NumHotkeys as u8);
    for &spell_id in player._pSplHotKey.iter() {
        file.i32(spell_id as i8 as i32);
    }
    for &spell_type in player._pSplTHotKey.iter() {
        file.u8(spell_type as i8 as u8);
    }
    file.i32(player._pRSpell as i8 as i32);
    file.u8(player._pRSplType as i8 as u8);
    file.finish(ctx, save_writer);
}

/// Original: `devilution::LoadHeroItems` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadHeroItems(Player &player) sha=45e36be0884c
pub fn load_hero_items(ctx: &mut Ctx, pnum: usize) {
    let archive = crate::pfile::open_save_archive(ctx, ctx.menu.g_save_number);
    let mut file = LoadHelper::new(ctx, archive, "heroitems");
    if !file.is_valid(1) {
        return;
    }
    ctx.loadsave.gbIsHellfireSaveGame = file.next_bool8();
    let mut player = std::mem::take(&mut ctx.players.Players[pnum]);
    let snapshot = player.clone();
    load_matching_items(ctx, &mut file, &snapshot, &mut player.InvBody);
    load_matching_items(ctx, &mut file, &snapshot, &mut player.InvList[..InventoryGridCells]);
    load_matching_items(ctx, &mut file, &snapshot, &mut player.SpdList);
    ctx.players.Players[pnum] = player;
    ctx.loadsave.gbIsHellfireSaveGame = ctx.init.gb_is_hellfire;
}

const StashVersion: u8 = 0;

/// Original: `devilution::LoadStash` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadStash() sha=2e70c7567bf9
pub fn load_stash(ctx: &mut Ctx) {
    let filename = if !ctx.init.gb_is_multiplayer { "spstashitems" } else { "mpstashitems" };
    ctx.stash.Stash = Default::default();
    let archive = crate::pfile::open_stash_archive(ctx);
    let mut file = LoadHelper::new(ctx, archive, filename);
    if !file.is_valid(1) {
        return;
    }
    let version = file.next_u8();
    if version > StashVersion {
        return;
    }
    ctx.stash.Stash.gold = file.next_u32() as i32;
    let pages = file.next_u32();
    for _ in 0..pages {
        let page = file.next_u32();
        let grid = ctx.stash.Stash.stashGrids.entry(page).or_default();
        for row in grid.iter_mut() {
            for cell in row.iter_mut() {
                *cell = file.next_u16();
            }
        }
    }
    let item_count = file.next_u32() as usize;
    ctx.stash.Stash.stashList.resize(item_count, Item::default());
    for i in 0..item_count {
        let mut item = std::mem::take(&mut ctx.stash.Stash.stashList[i]);
        load_and_validate_item_data(ctx, &mut file, &mut item);
        ctx.stash.Stash.stashList[i] = item;
    }
    let page = file.next_u32();
    crate::qol::stash::set_page(ctx, page);
}

/// Original: `devilution::RemoveEmptyInventory` (loadsave.cpp).
// @port loadsave.cpp|devilution::RemoveEmptyInventory(Player &player) sha=c554af3b8fa6
pub fn remove_empty_inventory(ctx: &mut Ctx, pnum: usize) {
    for i in (1..=InventoryGridCells).rev() {
        let idx = ctx.players.Players[pnum].InvGrid[i - 1];
        if idx > 0 && ctx.players.Players[pnum].InvList[idx as usize - 1].is_empty() {
            crate::player::remove_inv_item(ctx, pnum, idx as i32 - 1, true);
        }
    }
}

/// Original: `devilution::LoadGame` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadGame(bool firstflag) sha=68cf491323ae
pub fn load_game(ctx: &mut Ctx, firstflag: bool) {
    crate::diablo::free_game_mem(ctx);

    let archive = crate::pfile::open_save_archive(ctx, ctx.menu.g_save_number);
    let mut file = LoadHelper::new(ctx, archive, "game");
    if !file.is_valid(1) {
        crate::appfat::app_fatal(ctx, &tr("Unable to open save file archive"));
    }
    let magic = file.next_u32();
    if !is_header_valid(ctx, magic) {
        crate::appfat::app_fatal(ctx, &tr("Invalid save file"));
    }

    if ctx.loadsave.gbIsHellfireSaveGame {
        ctx.loadsave.giNumberOfLevels = 25;
        ctx.loadsave.giNumberQuests = 24;
        ctx.loadsave.giNumberOfSmithPremiumItems = 15;
    } else {
        ctx.loadsave.giNumberOfLevels = 17;
        ctx.loadsave.giNumberQuests = 16;
        ctx.loadsave.giNumberOfSmithPremiumItems = 6;
    }

    crate::pfile::pfile_remove_temp_files(ctx);

    ctx.gendung.setlevel = file.next_bool8();
    ctx.gendung.setlvlnum = file.next_u32_be() as _setlevels;
    ctx.gendung.currlevel = file.next_u32_be() as u8;
    ctx.gendung.leveltype = DungeonType::from_i8(file.next_u32_be() as i8);
    if !ctx.gendung.setlevel {
        ctx.gendung.leveltype = get_level_type(ctx.gendung.currlevel as i32);
    }
    let view_x = file.next_i32_be();
    let view_y = file.next_i32_be();
    ctx.inv.invflag = file.next_bool8();
    ctx.control.chrflag = file.next_bool8();
    let tmp_nummonsters = file.next_i32_be();
    let saved_item_count = file.next_u32_be();
    let tmp_nummissiles = file.next_i32_be();
    let tmp_nobjects = file.next_i32_be();

    if !ctx.init.gb_is_hellfire && matches!(ctx.gendung.leveltype, DungeonType::Nest | DungeonType::Crypt) {
        crate::appfat::app_fatal(ctx, &tr("Player is on a Hellfire only level"));
    }

    for i in 0..ctx.loadsave.giNumberOfLevels as usize {
        ctx.diablo.glSeedTbl[i] = file.next_u32_be();
        file.skip(4);
    }

    let me = ctx.players.MyPlayer.expect("MyPlayer");
    load_player(ctx, &mut file, me);

    if ctx.multi.sgGameInitInfo.nDifficulty > DIFF_HELL {
        ctx.multi.sgGameInitInfo.nDifficulty = DIFF_NORMAL;
    }

    for i in 0..ctx.loadsave.giNumberQuests as usize {
        load_quest(ctx, &mut file, i);
    }
    for i in 0..MAXPORTAL {
        load_portal(ctx, &mut file, i);
    }

    if ctx.loadsave.gbIsHellfireSaveGame != ctx.init.gb_is_hellfire {
        crate::pfile::pfile_convert_levels(ctx);
        remove_empty_inventory(ctx, me);
    }

    crate::diablo::load_game_level(ctx, firstflag, ENTRY_LOAD);
    crate::player::set_plr_anims(ctx, me);
    crate::player::sync_plr_anim(ctx, me);

    ctx.gendung.ViewPosition = Point::new(view_x, view_y);
    ctx.monster.ActiveMonsterCount = tmp_nummonsters as usize;
    ctx.objects.ActiveObjectCount = tmp_nobjects;

    for k in 0..ctx.monster.MonsterKillCounts.len() {
        ctx.monster.MonsterKillCounts[k] = file.next_i32_be();
    }
    file.skip(4 * (MaxMonsters - NUM_MTYPES as usize));
    if ctx.gendung.leveltype != DungeonType::Town {
        for i in 0..MaxMonsters {
            ctx.monster.ActiveMonsters[i] = file.next_i32_be();
        }
        for i in 0..ctx.monster.ActiveMonsterCount {
            let m = ctx.monster.ActiveMonsters[i] as usize;
            load_monster(ctx, &mut file, m, None);
        }
        for i in 0..ctx.monster.ActiveMonsterCount {
            let m = ctx.monster.ActiveMonsters[i] as usize;
            sync_pack_size(ctx, m);
        }
        file.skip(MaxMissilesForSaveGame);
        file.skip(MaxMissilesForSaveGame);
        for _ in 0..tmp_nummissiles {
            load_missile(ctx, &mut file);
        }
        for i in 0..ctx.monster.ActiveMonsterCount {
            let m = ctx.monster.ActiveMonsters[i] as usize;
            crate::monster::sync_monster_anim(ctx, m);
        }
        for i in 0..ctx.objects.ActiveObjects.len() {
            ctx.objects.ActiveObjects[i] = file.next_i8() as i32;
        }
        for i in 0..ctx.objects.AvailableObjects.len() {
            ctx.objects.AvailableObjects[i] = file.next_i8() as i32;
        }
        for i in 0..ctx.objects.ActiveObjectCount as usize {
            let oi = ctx.objects.ActiveObjects[i] as usize;
            load_object(ctx, &mut file, oi);
        }
        for i in 0..ctx.objects.ActiveObjectCount as usize {
            let oi = ctx.objects.ActiveObjects[i] as usize;
            crate::objects::sync_object_anim(ctx, oi);
        }

        ctx.lighting.ActiveLightCount = file.next_i32_be();
        for i in 0..ctx.lighting.ActiveLights.len() {
            ctx.lighting.ActiveLights[i] = file.next_u8();
        }
        for i in 0..ctx.lighting.ActiveLightCount as usize {
            let l = ctx.lighting.ActiveLights[i] as usize;
            load_lighting(&mut file, &mut ctx.lighting.Lights[l]);
        }

        file.skip(4);
        let vision_count = file.next_i32_be();
        for i in 0..vision_count as usize {
            load_lighting(&mut file, &mut ctx.lighting.VisionList[i]);
            ctx.lighting.VisionActive[i] = true;
        }
    }

    load_dropped_items(ctx, &mut file, saved_item_count as usize);

    load_additional_missiles(ctx);

    for i in 0..ctx.items.UniqueItemFlags.len() {
        ctx.items.UniqueItemFlags[i] = file.next_bool8();
    }

    file.skip(MAXDUNY * MAXDUNX);
    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            ctx.gendung.dFlags[i][j] = DungeonFlag(file.next_u8() & DungeonFlag::LoadedFlags.0);
        }
    }
    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            ctx.gendung.dPlayer[i][j] = file.next_i8();
        }
    }

    file.skip(MAXDUNX * MAXDUNY);

    if ctx.gendung.leveltype != DungeonType::Town {
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                ctx.gendung.dMonster[i][j] = file.next_i32_be() as i16;
            }
        }
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                ctx.gendung.dCorpse[i][j] = file.next_i8();
            }
        }
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                ctx.gendung.dObject[i][j] = file.next_i8();
            }
        }
        file.skip(MAXDUNY * MAXDUNX);
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                ctx.gendung.dPreLight[i][j] = file.next_u8();
            }
        }
        for j in 0..DMAXY {
            for i in 0..DMAXX {
                let automap_view = file.next_u8();
                ctx.automap.AutomapView[i][j] = if automap_view == MAP_EXP_OLD { MAP_EXP_SELF } else { automap_view };
            }
        }
        file.skip(MAXDUNX * MAXDUNY);

        *ctx.gendung.dLight = *ctx.gendung.dPreLight;
        let (light_id, tile) = (ctx.players.Players[me].lightId, ctx.players.Players[me].position.tile);
        crate::lighting::change_light_xy(ctx, light_id, tile);
    } else {
        for col in ctx.gendung.dLight.iter_mut() {
            col.fill(0);
        }
    }

    ctx.stores.numpremium = file.next_i32_be();
    ctx.stores.premiumlevel = file.next_i32_be();

    for i in 0..ctx.loadsave.giNumberOfSmithPremiumItems as usize {
        load_premium(ctx, &mut file, i);
    }
    if ctx.init.gb_is_hellfire && !ctx.loadsave.gbIsHellfireSaveGame {
        let player = ctx.players.Players[me].clone();
        crate::items::spawn_premium(ctx, &player);
    }

    ctx.automap.AutomapActive = file.next_bool8();
    ctx.automap.AutoMapScale = file.next_i32_be();
    crate::automap::automap_zoom_reset(ctx);
    crate::quests::resync_quests(ctx);

    if ctx.gendung.leveltype != DungeonType::Town {
        crate::lighting::redo_player_vision(ctx);
        crate::lighting::process_vision_list(ctx);
        crate::lighting::process_light_list(ctx);
    }

    for i in 0..ctx.missiles.Missiles.len() {
        let missile = &ctx.missiles.Missiles[i];
        if missile._mitype == MissileID::ManaShield && !missile._miDelFlag {
            let src = missile._misource as usize;
            ctx.players.Players[src].pManaShield = true;
            ctx.missiles.Missiles[i]._miDelFlag = true;
        }
    }

    crate::missiles::missiles_process_charge(ctx);
    crate::missiles::redo_missile_flags(ctx);
    crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_HAND);
    ctx.diablo.gb_process_players = crate::monster::is_diablo_alive(ctx, !firstflag);

    if ctx.loadsave.gbIsHellfireSaveGame != ctx.init.gb_is_hellfire {
        save_game(ctx);
    }

    ctx.loadsave.gbIsHellfireSaveGame = ctx.init.gb_is_hellfire;
}

/// Original: `devilution::SaveHeroItems` (loadsave.cpp).
// @port loadsave.cpp|devilution::SaveHeroItems(SaveWriter &saveWriter, Player &player) sha=98500dc42dea
pub fn save_hero_items(ctx: &Ctx, save_writer: &mut SaveWriter, player: &Player) {
    let item_count = NUM_INVLOC_USIZE + InventoryGridCells + MaxBeltItems;
    let item_size = if ctx.init.gb_is_hellfire { HellfireItemSaveSize } else { DiabloItemSaveSize };
    let mut file = SaveHelper::new("heroitems", item_count * item_size + 1);
    file.u8(ctx.init.gb_is_hellfire as u8);
    for item in player.InvBody.iter() {
        save_item(ctx, &mut file, item);
    }
    for item in player.InvList.iter() {
        save_item(ctx, &mut file, item);
    }
    for item in player.SpdList.iter() {
        save_item(ctx, &mut file, item);
    }
    file.finish(ctx, save_writer);
}

/// Original: `devilution::SaveStash` (loadsave.cpp).
// @port loadsave.cpp|devilution::SaveStash(SaveWriter &stashWriter) sha=c235525eee92
pub fn save_stash(ctx: &Ctx, stash_writer: &mut SaveWriter) {
    let filename = if !ctx.init.gb_is_multiplayer { "spstashitems" } else { "mpstashitems" };
    let item_size = if ctx.init.gb_is_hellfire { HellfireItemSaveSize } else { DiabloItemSaveSize };
    let stash = &ctx.stash.Stash;
    let mut file = SaveHelper::new(
        filename,
        1 + 4 + 4 + (4 + 10 * 10 * 2) * stash.stashGrids.len() + 4 + item_size * stash.stashList.len() + 4,
    );
    file.u8(StashVersion);
    file.u32(stash.gold as u32);
    // `std::map` iterates pages in key order
    let pages_to_save: Vec<u32> =
        stash.stashGrids.iter().filter(|(_, grid)| grid.iter().any(|row| row.iter().any(|&cell| cell > 0))).map(|(&page, _)| page).collect();
    file.u32(pages_to_save.len() as u32);
    for page in pages_to_save {
        file.u32(page);
        for row in stash.stashGrids[&page].iter() {
            for &cell in row.iter() {
                file.u16(cell);
            }
        }
    }
    file.u32(stash.stashList.len() as u32);
    for item in stash.stashList.iter() {
        save_item(ctx, &mut file, item);
    }
    file.u32(stash.GetPage());
    file.finish(ctx, stash_writer);
}

/// Original: `devilution::SaveGameData` (loadsave.cpp).
// @port loadsave.cpp|devilution::SaveGameData(SaveWriter &saveWriter) sha=7a2e8d0b4494
pub fn save_game_data(ctx: &mut Ctx, save_writer: &mut SaveWriter) {
    let mut file = SaveHelper::new("game", 320 * 1024);
    let (spawn, hf) = (ctx.init.gb_is_spawn, ctx.init.gb_is_hellfire);
    let magic = match (spawn, hf) {
        (true, false) => b"SHAR",
        (true, true) => b"SHLF",
        (false, true) => b"HELF",
        (false, false) => b"RETL",
    };
    file.u32(u32::from_le_bytes(*magic));

    if hf {
        ctx.loadsave.giNumberOfLevels = 25;
        ctx.loadsave.giNumberQuests = 24;
        ctx.loadsave.giNumberOfSmithPremiumItems = 15;
    } else {
        ctx.loadsave.giNumberOfLevels = 17;
        ctx.loadsave.giNumberQuests = 16;
        ctx.loadsave.giNumberOfSmithPremiumItems = 6;
    }

    let ctx_ro: &Ctx = ctx;
    file.u8(ctx_ro.gendung.setlevel as u8);
    file.u32_be(ctx_ro.gendung.setlvlnum as u32);
    file.u32_be(ctx_ro.gendung.currlevel as u32);
    file.u32_be(get_hellfire_level_type(ctx_ro.gendung.leveltype as i32) as u32);
    file.i32_be(ctx_ro.gendung.ViewPosition.x);
    file.i32_be(ctx_ro.gendung.ViewPosition.y);
    file.u8(ctx_ro.inv.invflag as u8);
    file.u8(ctx_ro.control.chrflag as u8);
    file.i32_be(ctx_ro.monster.ActiveMonsterCount as i32);
    file.i32_be(ctx_ro.items.ActiveItemCount as i32);
    file.u32_be(ctx_ro.missiles.Missiles.len().min(MaxMissilesForSaveGame) as u32);
    file.i32_be(ctx_ro.objects.ActiveObjectCount);

    for i in 0..ctx_ro.loadsave.giNumberOfLevels as usize {
        file.u32_be(ctx_ro.diablo.glSeedTbl[i]);
        file.i32_be(get_hellfire_level_type(get_level_type(i as i32) as i32));
    }

    let me = ctx_ro.players.MyPlayer.expect("MyPlayer");
    save_player(ctx_ro, &mut file, me);

    for i in 0..ctx_ro.loadsave.giNumberQuests as usize {
        save_quest(ctx_ro, &mut file, i);
    }
    for i in 0..MAXPORTAL {
        save_portal(ctx_ro, &mut file, i);
    }
    for &monstkill in ctx_ro.monster.MonsterKillCounts.iter() {
        file.i32_be(monstkill);
    }
    file.skip(4 * (MaxMonsters - NUM_MTYPES as usize));

    if ctx_ro.gendung.leveltype != DungeonType::Town {
        for &monster_id in ctx_ro.monster.ActiveMonsters.iter() {
            file.i32_be(monster_id);
        }
        for i in 0..ctx_ro.monster.ActiveMonsterCount {
            save_monster(ctx_ro, &mut file, ctx_ro.monster.ActiveMonsters[i] as usize, None);
        }
        for active_missile in 0..MaxMissilesForSaveGame {
            file.u8(active_missile as u8);
        }
        for available_missiles in ctx_ro.missiles.Missiles.len()..MaxMissilesForSaveGame {
            file.u8(available_missiles as u8);
        }
        let saved_missiles = ctx_ro.missiles.Missiles.len().min(MaxMissilesForSaveGame);
        file.skip(saved_missiles);
        for missile in ctx_ro.missiles.Missiles.iter().take(saved_missiles) {
            save_missile(&mut file, missile);
        }
        for &object_id in ctx_ro.objects.ActiveObjects.iter() {
            file.i8(object_id as i8);
        }
        for &object_id in ctx_ro.objects.AvailableObjects.iter() {
            file.i8(object_id as i8);
        }
        for i in 0..ctx_ro.objects.ActiveObjectCount as usize {
            save_object(ctx_ro, &mut file, &ctx_ro.objects.Objects[ctx_ro.objects.ActiveObjects[i] as usize]);
        }

        file.i32_be(ctx_ro.lighting.ActiveLightCount);
        for &light_id in ctx_ro.lighting.ActiveLights.iter() {
            file.u8(light_id);
        }
        for i in 0..ctx_ro.lighting.ActiveLightCount as usize {
            save_lighting(&mut file, &ctx_ro.lighting.Lights[ctx_ro.lighting.ActiveLights[i] as usize], false);
        }

        let vision_count = ctx_ro.players.Players.len() as i32;
        file.i32_be(vision_count + 1);
        file.i32_be(vision_count);
        for pnum in 0..ctx_ro.players.Players.len() {
            save_lighting(&mut file, &ctx_ro.lighting.VisionList[pnum], true);
        }
    }

    let item_indexes = save_dropped_items(ctx_ro, &mut file);

    for &unique_item_flag in ctx_ro.items.UniqueItemFlags.iter() {
        file.u8(unique_item_flag as u8);
    }

    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            file.u8(ctx_ro.gendung.dLight[i][j]);
        }
    }
    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            file.u8((ctx_ro.gendung.dFlags[i][j].0 & DungeonFlag::SavedFlags.0) as u8);
        }
    }
    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            file.i8(ctx_ro.gendung.dPlayer[i][j]);
        }
    }

    save_dropped_item_locations(ctx_ro, &mut file, &item_indexes);

    if ctx_ro.gendung.leveltype != DungeonType::Town {
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                file.i32_be(ctx_ro.gendung.dMonster[i][j] as i32);
            }
        }
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                file.i8(ctx_ro.gendung.dCorpse[i][j]);
            }
        }
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                file.i8(ctx_ro.gendung.dObject[i][j]);
            }
        }
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                file.u8(ctx_ro.gendung.dLight[i][j]);
            }
        }
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                file.u8(ctx_ro.gendung.dPreLight[i][j]);
            }
        }
        for j in 0..DMAXY {
            for i in 0..DMAXX {
                file.u8(ctx_ro.automap.AutomapView[i][j]);
            }
        }
        for j in 0..MAXDUNY {
            for i in 0..MAXDUNX {
                file.i8(if crate::missiles::tile_contains_missile(ctx_ro, Point::new(i as i32, j as i32)) { -1 } else { 0 });
            }
        }
    }

    file.i32_be(ctx_ro.stores.numpremium);
    file.i32_be(ctx_ro.stores.premiumlevel);

    for i in 0..ctx_ro.loadsave.giNumberOfSmithPremiumItems as usize {
        save_item(ctx_ro, &mut file, &ctx_ro.stores.premiumitems[i]);
    }

    file.u8(ctx_ro.automap.AutomapActive as u8);
    file.i32_be(ctx_ro.automap.AutoMapScale);

    save_additional_missiles(ctx_ro, save_writer);
    // The original writes "game" when its SaveHelper goes out of scope, after "additionalMissiles".
    file.finish(ctx_ro, save_writer);
}

/// Original: `devilution::SaveGame` (loadsave.cpp).
// @port loadsave.cpp|devilution::SaveGame() sha=244eb72f6df5
pub fn save_game(ctx: &mut Ctx) {
    ctx.pfile.gbValidSaveFile = true;
    crate::pfile::pfile_write_hero(ctx, true);
    crate::pfile::sfile_write_stash(ctx);
}

/// Original: `devilution::SaveLevel(SaveWriter &saveWriter)` (loadsave.cpp).
// @port loadsave.cpp|devilution::SaveLevel(SaveWriter &saveWriter) sha=fe9c883d739e
pub fn save_level(ctx: &mut Ctx, save_writer: &mut SaveWriter) {
    save_level_with(ctx, save_writer, None);
}

/// Original: `devilution::LoadLevel()` (loadsave.cpp).
// @port loadsave.cpp|devilution::LoadLevel() sha=b3c5d1d2e4a8
pub fn load_level(ctx: &mut Ctx) {
    load_level_with(ctx, None);
}
