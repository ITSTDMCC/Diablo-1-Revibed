//! `Source/pack.cpp` / `pack.h`: the packed (save file and network) forms of items and players.
//!
//! The packed structs are `#pragma pack(1)` little-endian records in the original; here they are
//! plain structs with explicit `to_bytes` / `from_bytes` that produce the same byte layout.

use crate::ctx::Ctx;
use crate::engine::geometry::Point;
use crate::enums::*;
use crate::items::{recreate_ear, recreate_item, Item};
use crate::loadsave::{remap_item_idx_from_diablo, remap_item_idx_from_spawn, remap_item_idx_to_diablo, remap_item_idx_to_spawn};
use crate::player::{InventoryGridCells, MaxBeltItems, MaxCharacterLevel, Player, PlayerNameLength, NUMLEVELS};
use crate::tables::playerdat::PlayersData;
use crate::utils::cstr::CStr;
use crate::platform::log;

/// `sizeof(ItemPack)`: the packed (1-byte aligned) struct: u32 iSeed, u16 iCreateInfo, u16 idx,
/// u8 bId, bDur, bMDur, bCh, bMCh, u16 wValue, u32 dwBuff.
pub const ITEM_PACK_SIZE: usize = 4 + 2 + 2 + 5 + 2 + 4;

/// `sizeof(PlayerPack)`
pub const PLAYER_PACK_SIZE: usize = 1266;

const NUM_INVLOC_USIZE: usize = NUM_INVLOC as usize;

/// Little-endian byte writer for the packed records.
struct Writer<'a> {
    buf: &'a mut Vec<u8>,
}

impl Writer<'_> {
    fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }
    fn i8(&mut self, v: i8) {
        self.buf.push(v as u8);
    }
    fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn i16(&mut self, v: i16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn i32(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn bytes(&mut self, v: &[u8]) {
        self.buf.extend_from_slice(v);
    }
}

/// Little-endian byte reader for the packed records (missing bytes read as zero).
pub(crate) struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(buf: &'a [u8]) -> Self {
        Reader { buf, pos: 0 }
    }
    fn take<const N: usize>(&mut self) -> [u8; N] {
        let mut out = [0u8; N];
        for (i, o) in out.iter_mut().enumerate() {
            *o = self.buf.get(self.pos + i).copied().unwrap_or(0);
        }
        self.pos += N;
        out
    }
    fn u8(&mut self) -> u8 {
        self.take::<1>()[0]
    }
    fn i8(&mut self) -> i8 {
        self.take::<1>()[0] as i8
    }
    fn u16(&mut self) -> u16 {
        u16::from_le_bytes(self.take())
    }
    fn i16(&mut self) -> i16 {
        i16::from_le_bytes(self.take())
    }
    fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take())
    }
    fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.take())
    }
    fn u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take())
    }
}

/// `ItemPack`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemPack {
    pub iSeed: u32,
    pub iCreateInfo: u16,
    pub idx: u16,
    pub bId: u8,
    pub bDur: u8,
    pub bMDur: u8,
    pub bCh: u8,
    pub bMCh: u8,
    pub wValue: u16,
    pub dwBuff: u32,
}

impl ItemPack {
    fn write(&self, w: &mut Writer) {
        w.u32(self.iSeed);
        w.u16(self.iCreateInfo);
        w.u16(self.idx);
        w.u8(self.bId);
        w.u8(self.bDur);
        w.u8(self.bMDur);
        w.u8(self.bCh);
        w.u8(self.bMCh);
        w.u16(self.wValue);
        w.u32(self.dwBuff);
    }

    fn read(r: &mut Reader) -> ItemPack {
        ItemPack {
            iSeed: r.u32(),
            iCreateInfo: r.u16(),
            idx: r.u16(),
            bId: r.u8(),
            bDur: r.u8(),
            bMDur: r.u8(),
            bCh: r.u8(),
            bMCh: r.u8(),
            wValue: r.u16(),
            dwBuff: r.u32(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(ITEM_PACK_SIZE);
        self.write(&mut Writer { buf: &mut buf });
        buf
    }

    pub fn from_bytes(raw: &[u8]) -> ItemPack {
        ItemPack::read(&mut Reader::new(raw))
    }
}

/// `PlayerPack`
#[derive(Clone, Debug)]
pub struct PlayerPack {
    pub dwLowDateTime: u32,
    pub dwHighDateTime: u32,
    pub destAction: i8,
    pub destParam1: i8,
    pub destParam2: i8,
    pub plrlevel: u8,
    pub px: u8,
    pub py: u8,
    pub targx: u8,
    pub targy: u8,
    pub pName: [u8; PlayerNameLength],
    pub pClass: u8,
    pub pBaseStr: u8,
    pub pBaseMag: u8,
    pub pBaseDex: u8,
    pub pBaseVit: u8,
    pub pLevel: i8,
    pub pStatPts: u8,
    pub pExperience: u32,
    pub pGold: i32,
    pub pHPBase: i32,
    pub pMaxHPBase: i32,
    pub pManaBase: i32,
    pub pMaxManaBase: i32,
    pub pSplLvl: [u8; 37],
    pub pMemSpells: u64,
    pub InvBody: [ItemPack; NUM_INVLOC_USIZE],
    pub InvList: [ItemPack; InventoryGridCells],
    pub InvGrid: [i8; InventoryGridCells],
    pub _pNumInv: u8,
    pub SpdList: [ItemPack; MaxBeltItems],
    pub pTownWarps: i8,
    pub pDungMsgs: i8,
    pub pLvlLoad: i8,
    pub pBattleNet: u8,
    pub pManaShield: u8,
    pub pDungMsgs2: u8,
    pub bIsHellfire: i8,
    pub reserved: u8,
    pub wReflections: u16,
    pub reserved2: [u8; 2],
    pub pSplLvl2: [u8; 10],
    pub wReserved8: i16,
    pub pDiabloKillLevel: u32,
    pub pDifficulty: u32,
    pub pDamAcFlags: u32,
    pub reserved3: [u8; 20],
}

impl Default for PlayerPack {
    fn default() -> Self {
        PlayerPack::from_bytes(&[])
    }
}

impl PlayerPack {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(PLAYER_PACK_SIZE);
        let w = &mut Writer { buf: &mut buf };
        w.u32(self.dwLowDateTime);
        w.u32(self.dwHighDateTime);
        w.i8(self.destAction);
        w.i8(self.destParam1);
        w.i8(self.destParam2);
        w.u8(self.plrlevel);
        w.u8(self.px);
        w.u8(self.py);
        w.u8(self.targx);
        w.u8(self.targy);
        w.bytes(&self.pName);
        w.u8(self.pClass);
        w.u8(self.pBaseStr);
        w.u8(self.pBaseMag);
        w.u8(self.pBaseDex);
        w.u8(self.pBaseVit);
        w.i8(self.pLevel);
        w.u8(self.pStatPts);
        w.u32(self.pExperience);
        w.i32(self.pGold);
        w.i32(self.pHPBase);
        w.i32(self.pMaxHPBase);
        w.i32(self.pManaBase);
        w.i32(self.pMaxManaBase);
        w.bytes(&self.pSplLvl);
        w.u64(self.pMemSpells);
        for it in self.InvBody.iter() {
            it.write(w);
        }
        for it in self.InvList.iter() {
            it.write(w);
        }
        for &g in self.InvGrid.iter() {
            w.i8(g);
        }
        w.u8(self._pNumInv);
        for it in self.SpdList.iter() {
            it.write(w);
        }
        w.i8(self.pTownWarps);
        w.i8(self.pDungMsgs);
        w.i8(self.pLvlLoad);
        w.u8(self.pBattleNet);
        w.u8(self.pManaShield);
        w.u8(self.pDungMsgs2);
        w.i8(self.bIsHellfire);
        w.u8(self.reserved);
        w.u16(self.wReflections);
        w.bytes(&self.reserved2);
        w.bytes(&self.pSplLvl2);
        w.i16(self.wReserved8);
        w.u32(self.pDiabloKillLevel);
        w.u32(self.pDifficulty);
        w.u32(self.pDamAcFlags);
        w.bytes(&self.reserved3);
        debug_assert_eq!(buf.len(), PLAYER_PACK_SIZE);
        buf
    }

    pub fn from_bytes(raw: &[u8]) -> PlayerPack {
        let r = &mut Reader::new(raw);
        PlayerPack {
            dwLowDateTime: r.u32(),
            dwHighDateTime: r.u32(),
            destAction: r.i8(),
            destParam1: r.i8(),
            destParam2: r.i8(),
            plrlevel: r.u8(),
            px: r.u8(),
            py: r.u8(),
            targx: r.u8(),
            targy: r.u8(),
            pName: r.take(),
            pClass: r.u8(),
            pBaseStr: r.u8(),
            pBaseMag: r.u8(),
            pBaseDex: r.u8(),
            pBaseVit: r.u8(),
            pLevel: r.i8(),
            pStatPts: r.u8(),
            pExperience: r.u32(),
            pGold: r.i32(),
            pHPBase: r.i32(),
            pMaxHPBase: r.i32(),
            pManaBase: r.i32(),
            pMaxManaBase: r.i32(),
            pSplLvl: r.take(),
            pMemSpells: r.u64(),
            InvBody: std::array::from_fn(|_| ItemPack::read(r)),
            InvList: std::array::from_fn(|_| ItemPack::read(r)),
            InvGrid: std::array::from_fn(|_| r.i8()),
            _pNumInv: r.u8(),
            SpdList: std::array::from_fn(|_| ItemPack::read(r)),
            pTownWarps: r.i8(),
            pDungMsgs: r.i8(),
            pLvlLoad: r.i8(),
            pBattleNet: r.u8(),
            pManaShield: r.u8(),
            pDungMsgs2: r.u8(),
            bIsHellfire: r.i8(),
            reserved: r.u8(),
            wReflections: r.u16(),
            reserved2: r.take(),
            pSplLvl2: r.take(),
            wReserved8: r.i16(),
            pDiabloKillLevel: r.u32(),
            pDifficulty: r.u32(),
            pDamAcFlags: r.u32(),
            reserved3: r.take(),
        }
    }
}

/// Original: `VerifyGoldSeeds` (pack.cpp).
// @port pack.cpp|devilution::VerifyGoldSeeds(Player &player) sha=dc7fca6954cf
fn verify_gold_seeds(ctx: &mut Ctx, player: &mut Player) {
    let n = player._pNumInv;
    let mut i = 0;
    while i < n {
        if player.InvList[i as usize].IDidx != IDI_GOLD {
            i += 1;
            continue;
        }
        let mut j = 0;
        while j < n {
            if i == j || player.InvList[j as usize].IDidx != IDI_GOLD || player.InvList[i as usize]._iSeed != player.InvList[j as usize]._iSeed {
                j += 1;
                continue;
            }
            player.InvList[i as usize]._iSeed = ctx.rng.advance_rnd_seed() as u32;
            // `j = -1;` followed by the loop's `j++`
            j = 0;
        }
        i += 1;
    }
}

/// `LoadBE32`
fn load_be32(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

/// Original: `devilution::PackItem` (pack.cpp).
// @port pack.cpp|devilution::PackItem(ItemPack &packedItem, const Item &item, bool isHellfire) sha=a4298e6840a3
pub fn pack_item(ctx: &Ctx, item: &Item, is_hellfire: bool) -> ItemPack {
    let mut packed_item = ItemPack::default();
    if item.is_empty() || item._iMiscId == IMISC_ARENAPOT {
        packed_item.idx = 0xFFFF;
    } else {
        let mut idx = item.IDidx;
        if !is_hellfire {
            idx = remap_item_idx_to_diablo(idx);
        }
        if ctx.init.gb_is_spawn {
            idx = remap_item_idx_to_spawn(idx);
        }
        packed_item.idx = idx as u16;
        if item.IDidx == IDI_EAR {
            let name = item._iIName.bytes();
            // `char` is signed in the original build.
            let n = |i: usize| name[i] as i8 as i32;
            packed_item.iCreateInfo = (n(1) | (n(0) << 8)) as u16;
            packed_item.iSeed = load_be32(&name[2..6]);
            packed_item.bId = name[6];
            packed_item.bDur = name[7];
            packed_item.bMDur = name[8];
            packed_item.bCh = name[9];
            packed_item.bMCh = name[10];
            packed_item.wValue = (item._ivalue | (n(11) << 8) | ((item._iCurs as i32 - ICURS_EAR_SORCERER as i32) << 6)) as u16;
            packed_item.dwBuff = load_be32(&name[12..16]);
        } else {
            packed_item.iSeed = item._iSeed;
            packed_item.iCreateInfo = item._iCreateInfo;
            packed_item.bId = ((item._iMagical as i32) << 1 | if item._iIdentified { 1 } else { 0 }) as u8;
            if item._iMaxDur > 255 {
                packed_item.bMDur = 254;
            } else {
                packed_item.bMDur = item._iMaxDur as u8;
            }
            packed_item.bDur = item._iDurability.min(packed_item.bMDur as i32) as u8;
            packed_item.bCh = item._iCharges as u8;
            packed_item.bMCh = item._iMaxCharges as u8;
            if item.IDidx == IDI_GOLD {
                packed_item.wValue = item._ivalue as u16;
            }
            packed_item.dwBuff = item.dwBuff;
        }
    }
    packed_item
}

/// Original: `devilution::PackPlayer` (pack.cpp).
// @port pack.cpp|devilution::PackPlayer(PlayerPack &packed, const Player &player) sha=84374e2543c6
pub fn pack_player(ctx: &Ctx, player: &Player) -> PlayerPack {
    let mut packed = PlayerPack::default();
    packed.destAction = player.destAction;
    packed.destParam1 = player.destParam1 as i8;
    packed.destParam2 = player.destParam2 as i8;
    packed.plrlevel = player.plrlevel;
    packed.px = player.position.tile.x as u8;
    packed.py = player.position.tile.y as u8;
    if ctx.init.gb_vanilla {
        packed.targx = player.position.tile.x as u8;
        packed.targy = player.position.tile.y as u8;
    }
    packed.pName = *player._pName.bytes();
    packed.pClass = player._pClass as u8;
    packed.pBaseStr = player._pBaseStr as u8;
    packed.pBaseMag = player._pBaseMag as u8;
    packed.pBaseDex = player._pBaseDex as u8;
    packed.pBaseVit = player._pBaseVit as u8;
    packed.pLevel = player._pLevel;
    packed.pStatPts = player._pStatPts as u8;
    packed.pExperience = player._pExperience;
    packed.pGold = player._pGold;
    packed.pHPBase = player._pHPBase;
    packed.pMaxHPBase = player._pMaxHPBase;
    packed.pManaBase = player._pManaBase;
    packed.pMaxManaBase = player._pMaxManaBase;
    packed.pMemSpells = player._pMemSpells;

    for i in 0..37 {
        packed.pSplLvl[i] = player._pSplLvl[i];
    }
    for i in 37..47 {
        packed.pSplLvl2[i - 37] = player._pSplLvl[i];
    }

    let hf = ctx.init.gb_is_hellfire;
    for i in 0..NUM_INVLOC_USIZE {
        packed.InvBody[i] = pack_item(ctx, &player.InvBody[i], hf);
    }
    packed._pNumInv = player._pNumInv as u8;
    for i in 0..packed._pNumInv as usize {
        packed.InvList[i] = pack_item(ctx, &player.InvList[i], hf);
    }
    for i in 0..InventoryGridCells {
        packed.InvGrid[i] = player.InvGrid[i];
    }
    for i in 0..MaxBeltItems {
        packed.SpdList[i] = pack_item(ctx, &player.SpdList[i], hf);
    }

    packed.wReflections = player.wReflections;
    packed.pDamAcFlags = player.pDamAcFlags.0 as u32;
    packed.pDiabloKillLevel = player.pDiabloKillLevel as u32;
    packed.bIsHellfire = if hf { 1 } else { 0 };
    packed
}

/// Original: `devilution::UnPackItem` (pack.cpp).
// @port pack.cpp|devilution::UnPackItem(const ItemPack &packedItem, const Player &player, Item &item, bool isHellfire) sha=b47b283ff62f
pub fn unpack_item(ctx: &mut Ctx, packed_item: &ItemPack, player: &Player, item: &mut Item, is_hellfire: bool) {
    if packed_item.idx == 0xFFFF {
        item.clear();
        return;
    }
    let mut idx = packed_item.idx as _item_indexes;
    if ctx.init.gb_is_spawn {
        idx = remap_item_idx_from_spawn(idx);
    }
    if !is_hellfire {
        idx = remap_item_idx_from_diablo(idx);
    }
    if !crate::items::is_item_available(ctx, idx as i32) {
        item.clear();
        return;
    }

    if idx == IDI_EAR {
        let ic = packed_item.iCreateInfo;
        let iseed = packed_item.iSeed;
        let ivalue = packed_item.wValue;
        let ibuff = packed_item.dwBuff as i32;

        let hero_name: [u8; 16] = [
            ((ic >> 8) & 0x7F) as u8,
            (ic & 0x7F) as u8,
            ((iseed >> 24) & 0x7F) as u8,
            ((iseed >> 16) & 0x7F) as u8,
            ((iseed >> 8) & 0x7F) as u8,
            (iseed & 0x7F) as u8,
            packed_item.bId & 0x7F,
            packed_item.bDur & 0x7F,
            packed_item.bMDur & 0x7F,
            packed_item.bCh & 0x7F,
            packed_item.bMCh & 0x7F,
            ((ivalue >> 8) & 0x7F) as u8,
            ((ibuff >> 24) & 0x7F) as u8,
            ((ibuff >> 16) & 0x7F) as u8,
            ((ibuff >> 8) & 0x7F) as u8,
            (ibuff & 0x7F) as u8,
        ];
        let end = hero_name.iter().position(|&b| b == 0).unwrap_or(16);
        let name = String::from_utf8_lossy(&hero_name[..end]).into_owned();
        recreate_ear(ctx, item, ic, iseed, (ivalue & 0xFF) as u8, &name);
    } else {
        *item = Item::default();
        recreate_item(ctx, player, item, idx, packed_item.iCreateInfo, packed_item.iSeed, packed_item.wValue as i32, is_hellfire);
        item._iIdentified = (packed_item.bId & 1) != 0;
        item._iMaxDur = packed_item.bMDur as i32;
        item._iDurability = crate::inv::clamp_durability(item, packed_item.bDur as i32);
        item._iMaxCharges = (packed_item.bMCh as i32).clamp(0, item._iMaxCharges);
        item._iCharges = (packed_item.bCh as i32).clamp(0, item._iMaxCharges);
    }
}

/// Original: `devilution::UnPackPlayer` (pack.cpp). `pnum` is the player slot being filled.
// @port pack.cpp|devilution::UnPackPlayer(const PlayerPack &packed, Player &player) sha=bd2abf73c5be
pub fn unpack_player(ctx: &mut Ctx, packed: &PlayerPack, pnum: usize) {
    let position = Point::new(packed.px as i32, packed.py as i32);

    {
        let player = &mut ctx.players.Players[pnum];
        *player = Player::default();
        player._pLevel = packed.pLevel.clamp(1, MaxCharacterLevel as i8);
        player._pMaxHPBase = packed.pMaxHPBase;
        player._pHPBase = packed.pHPBase;
        player._pHPBase = player._pHPBase.clamp(0, player._pMaxHPBase);
        player._pMaxHP = player._pMaxHPBase;
        player._pHitPoints = player._pHPBase;
        player.position.tile = position;
        player.position.future = position;
        player.set_level((packed.plrlevel as i8).clamp(0, NUMLEVELS as i8) as u8);
        player._pClass = HeroClass::from_raw(packed.pClass.clamp(0, 5));
    }

    crate::player::clr_plr_path(ctx, pnum);
    ctx.players.Players[pnum].destAction = ACTION_NONE;
    ctx.players.Players[pnum]._pName = CStr::from_raw(&packed.pName);
    let name = ctx.players.Players[pnum]._pName.as_str().to_string();
    ctx.players.Players[pnum]._pName.set(&name);

    crate::player::init_player(ctx, pnum, true);

    {
        let player = &mut ctx.players.Players[pnum];
        player._pBaseStr = (packed.pBaseStr as i32).min(player.get_maximum_attribute_value(CharacterAttribute::Strength) as u8 as i32);
        player._pStrength = player._pBaseStr;
        player._pBaseMag = (packed.pBaseMag as i32).min(player.get_maximum_attribute_value(CharacterAttribute::Magic) as u8 as i32);
        player._pMagic = player._pBaseMag;
        player._pBaseDex = (packed.pBaseDex as i32).min(player.get_maximum_attribute_value(CharacterAttribute::Dexterity) as u8 as i32);
        player._pDexterity = player._pBaseDex;
        player._pBaseVit = (packed.pBaseVit as i32).min(player.get_maximum_attribute_value(CharacterAttribute::Vitality) as u8 as i32);
        player._pVitality = player._pBaseVit;
        player._pStatPts = packed.pStatPts as i32;

        player._pExperience = packed.pExperience;
        player._pGold = packed.pGold;
        player._pBaseToBlk = PlayersData[player._pClass as usize].blockBonus as i32;
        if ((player._pHPBase as u32 & 0xFFFFFFC0) as i32) < 64 {
            player._pHPBase = 64;
        }

        player._pMaxManaBase = packed.pMaxManaBase;
        player._pManaBase = packed.pManaBase;
        player._pManaBase = player._pManaBase.min(player._pMaxManaBase);
        player._pMemSpells = packed.pMemSpells;

        for i in 0..37 {
            player._pSplLvl[i] = packed.pSplLvl[i];
        }
        for i in 37..47 {
            player._pSplLvl[i] = packed.pSplLvl2[i - 37];
        }
    }

    let is_hellfire = packed.bIsHellfire != 0;

    // UnPackItem reads the player (for RecreateItem) while writing its items.
    let mut player = std::mem::take(&mut ctx.players.Players[pnum]);
    for i in 0..NUM_INVLOC_USIZE {
        let mut it = std::mem::take(&mut player.InvBody[i]);
        unpack_item(ctx, &packed.InvBody[i], &player, &mut it, is_hellfire);
        player.InvBody[i] = it;
    }
    player._pNumInv = packed._pNumInv as i32;
    for i in 0..player._pNumInv as usize {
        let mut it = std::mem::take(&mut player.InvList[i]);
        unpack_item(ctx, &packed.InvList[i], &player, &mut it, is_hellfire);
        player.InvList[i] = it;
    }
    for i in 0..InventoryGridCells {
        player.InvGrid[i] = packed.InvGrid[i];
    }
    verify_gold_seeds(ctx, &mut player);
    for i in 0..MaxBeltItems {
        let mut it = std::mem::take(&mut player.SpdList[i]);
        unpack_item(ctx, &packed.SpdList[i], &player, &mut it, is_hellfire);
        player.SpdList[i] = it;
    }
    ctx.players.Players[pnum] = player;

    crate::items::calc_plr_inv(ctx, pnum, false);
    let player = &mut ctx.players.Players[pnum];
    player.wReflections = packed.wReflections;
    player.pDiabloKillLevel = packed.pDiabloKillLevel as u8;
}


/// `sizeof(PlayerNetPack)`
pub const PLAYER_NET_PACK_SIZE: usize = 1691;

/// Original: `hasMultipleFlags` (pack.cpp).
// @port pack.cpp|devilution::hasMultipleFlags(uint16_t flags) sha=f9538fb64388
fn has_multiple_flags(flags: u16) -> bool {
    (flags & flags.wrapping_sub(1)) > 0
}

/// Original: `devilution::IsCreationFlagComboValid` (pack.cpp).
// @port pack.cpp|devilution::IsCreationFlagComboValid(uint16_t iCreateInfo) sha=af2dfe10260a
pub fn is_creation_flag_combo_valid(mut i_create_info: u16) -> bool {
    i_create_info &= !(CF_LEVEL as u16);
    let is_town_item = (i_create_info & CF_TOWN as u16) != 0;
    let is_pregen_item = (i_create_info & CF_PREGEN as u16) != 0;
    let is_useful_item = (i_create_info & CF_USEFUL as u16) == CF_USEFUL as u16;
    if is_pregen_item {
        return false;
    }
    if is_useful_item && (i_create_info & !(CF_USEFUL as u16)) != 0 {
        return false;
    }
    if is_town_item && has_multiple_flags(i_create_info) {
        return false;
    }
    true
}

/// Original: `devilution::IsTownItemValid` (pack.cpp).
// @port pack.cpp|devilution::IsTownItemValid(uint16_t iCreateInfo) sha=db87a7f336bc
pub fn is_town_item_valid(i_create_info: u16) -> bool {
    let level = (i_create_info & CF_LEVEL as u16) as u8;
    let is_boy_item = (i_create_info & CF_BOY as u16) != 0;
    let max_town_item_level = 30u8;
    if is_boy_item && level as i32 <= MaxCharacterLevel {
        return true;
    }
    level <= max_town_item_level
}

/// Original: `devilution::IsUniqueMonsterItemValid` (pack.cpp).
// @port pack.cpp|devilution::IsUniqueMonsterItemValid(uint16_t iCreateInfo, uint32_t dwBuff) sha=266bc6d50862
pub fn is_unique_monster_item_valid(i_create_info: u16, _dw_buff: u32) -> bool {
    use crate::tables::monstdat::{MonstersData, UniqueMonstersData};
    let level = (i_create_info & CF_LEVEL as u16) as u8;
    for unique_monster_data in UniqueMonstersData.iter() {
        if unique_monster_data.mName.is_empty() {
            break;
        }
        let unique_monster_level = MonstersData[unique_monster_data.mtype as usize].level as u8;
        if matches!(unique_monster_data.mtype, MT_DEFILER | MT_NAKRUL | MT_HORKDMN) {
            continue;
        }
        if level == unique_monster_level {
            return true;
        }
    }
    false
}

/// Original: `devilution::IsDungeonItemValid` (pack.cpp).
// @port pack.cpp|devilution::IsDungeonItemValid(uint16_t iCreateInfo, uint32_t dwBuff) sha=acddb609322e
pub fn is_dungeon_item_valid(i_create_info: u16, dw_buff: u32) -> bool {
    use crate::tables::monstdat::MonstersData;
    let level = (i_create_info & CF_LEVEL as u16) as u8;
    let is_hellfire_item = (dw_buff & CF_HELLFIRE as u32) != 0;
    for i in 0..NUM_MTYPES as i16 {
        let monster_data = &MonstersData[i as usize];
        let mut monster_level = monster_data.level as u8;
        if i != MT_DIABLO as i16 && monster_data.availability == MonsterAvailability::Never {
            continue;
        }
        if i == MT_DIABLO as i16 && !is_hellfire_item {
            monster_level = monster_level.wrapping_sub(15);
        }
        if level == monster_level {
            return true;
        }
    }
    if is_hellfire_item {
        let mut hellfire_max_dungeon_level = 24u8;
        hellfire_max_dungeon_level -= 7;
        return level as u32 <= hellfire_max_dungeon_level as u32 * 2;
    }
    let mut diablo_max_dungeon_level = 16u8;
    diablo_max_dungeon_level -= 1;
    level as u32 <= diablo_max_dungeon_level as u32 * 2
}

/// Original: `devilution::RecreateHellfireSpellBook` (pack.cpp). `pnum` names the player for
/// the validation log message.
// @port pack.cpp|devilution::RecreateHellfireSpellBook(const Player &player, const TItem &packedItem, Item *item) sha=9f87c70c533b
pub fn recreate_hellfire_spell_book(ctx: &mut Ctx, player: &Player, packed_item: &crate::msg::NetItem, item: Option<&mut Item>, pnum: usize) -> bool {
    let mut spell_book = Item::default();
    crate::msg::recreate_item(ctx, player, packed_item, &mut spell_book);
    let mut spell_book_level = crate::spells::get_spell_book_level(ctx, spell_book._iSpell);
    spell_book_level += 1;
    if spell_book_level >= 1 && (spell_book._iCreateInfo & CF_LEVEL as u16) as i32 == spell_book_level * 2 {
        if let Some(i) = item {
            *i = spell_book;
        }
        return true;
    }
    if !is_dungeon_item_valid(spell_book._iCreateInfo, spell_book.dwBuff) {
        log::verbose!("Remote player validation failed: ValidateFields(spellBook._iCreateInfo, spellBook.dwBuff, IsDungeonItemValid)");
        let name = ctx.players.Players[pnum]._pName.as_str().to_string();
        crate::plrmsg::event_plr_msg(ctx, &format!("Player '{name}' sent invalid player data during attempt to join the game."));
        return false;
    }
    if let Some(i) = item {
        *i = spell_book;
    }
    true
}

/// Original: `devilution::PackNetItem` (pack.cpp).
// @port pack.cpp|devilution::PackNetItem(const Item &item, ItemNetPack &packedItem) sha=f3c8c618735e
fn pack_net_item(item: &Item) -> crate::msg::NetItem {
    let mut n = crate::msg::NetItem::default();
    if item.is_empty() {
        n.0[0..2].copy_from_slice(&0xFFFFu16.to_le_bytes());
        return n;
    }
    n.set_def(item.IDidx, item._iCreateInfo, item._iSeed);
    if item.IDidx != IDI_EAR {
        crate::msg::prepare_item_for_network(item, &mut n);
    } else {
        crate::msg::prepare_ear_for_network(item, &mut n);
    }
    n
}

/// Original: `devilution::PackNetPlayer` (pack.cpp). Returns the `PlayerNetPack` bytes.
// @port pack.cpp|devilution::PackNetPlayer(PlayerNetPack &packed, const Player &player) sha=db8cae60fb8f
pub fn pack_net_player(ctx: &Ctx, pnum: usize) -> Vec<u8> {
    let player = &ctx.players.Players[pnum];
    let mut buf: Vec<u8> = Vec::with_capacity(PLAYER_NET_PACK_SIZE);
    let w = &mut Writer { buf: &mut buf };
    w.u8(player.plrlevel);
    w.u8(player.position.tile.x as u8);
    w.u8(player.position.tile.y as u8);
    w.bytes(player._pName.bytes());
    w.u8(player._pClass as u8);
    w.u8(player._pBaseStr as u8);
    w.u8(player._pBaseMag as u8);
    w.u8(player._pBaseDex as u8);
    w.u8(player._pBaseVit as u8);
    w.i8(player._pLevel);
    w.u8(player._pStatPts as u8);
    w.u32(player._pExperience);
    w.i32(player._pHPBase);
    w.i32(player._pMaxHPBase);
    w.i32(player._pManaBase);
    w.i32(player._pMaxManaBase);
    w.bytes(&player._pSplLvl[..crate::items::MAX_SPELLS as usize]);
    w.u64(player._pMemSpells);
    for item in player.InvBody.iter() {
        w.bytes(&pack_net_item(item).0);
    }
    for i in 0..InventoryGridCells {
        // Only _pNumInv entries are packed; the rest of the (uninitialised) buffer is sent as is.
        if (i as i32) < player._pNumInv {
            w.bytes(&pack_net_item(&player.InvList[i]).0);
        } else {
            w.bytes(&[0u8; crate::msg::SIZE_NETITEM]);
        }
    }
    for &g in player.InvGrid.iter() {
        w.i8(g);
    }
    w.u8(player._pNumInv as u8);
    for item in player.SpdList.iter() {
        w.bytes(&pack_net_item(item).0);
    }
    w.u8(player.pManaShield as u8);
    w.u16(player.wReflections);
    w.u8(player.pDiabloKillLevel);
    w.u8(player.friendlyMode as u8);
    w.u8(player.plrIsOnSetLevel as u8);
    for v in [
        player._pStrength,
        player._pMagic,
        player._pDexterity,
        player._pVitality,
        player._pHitPoints,
        player._pMaxHP,
        player._pMana,
        player._pMaxMana,
        player._pDamageMod,
        player._pBaseToBlk,
        player._pIMinDam,
        player._pIMaxDam,
        player._pIAC,
        player._pIBonusDam,
        player._pIBonusToHit,
        player._pIBonusAC,
        player._pIBonusDamMod,
        player._pIGetHit,
        player._pIEnAc,
        player._pIFMinDam,
        player._pIFMaxDam,
        player._pILMinDam,
        player._pILMaxDam,
    ] {
        w.i32(v);
    }
    debug_assert_eq!(buf.len(), PLAYER_NET_PACK_SIZE);
    buf
}

/// Original: `EventFailedJoinAttempt` (pack.cpp).
// @port pack.cpp|devilution::EventFailedJoinAttempt(const char *playerName) sha=bf5dd4edf6d5
fn event_failed_join_attempt(ctx: &mut Ctx, player_name: &str) {
    let message = format!("Player '{player_name}' sent invalid player data during attempt to join the game.");
    crate::plrmsg::event_plr_msg(ctx, &message);
}

/// Original: `LogFailedJoinAttempt` (pack.cpp), both overloads: the `ValidateField` /
/// `ValidateFields` log line.
// @port pack.cpp|devilution::LogFailedJoinAttempt(const char *condition, const char *name, T value) sha=6404d5cdaae8
// @port pack.cpp|devilution::LogFailedJoinAttempt(const char *condition, const char *name1, T1 value1, const char *name2, T2 value2) sha=25148fba3cce
fn log_failed_join_attempt(condition: &str, fields: &[(&str, String)]) {
    let values: Vec<String> = fields.iter().map(|(n, v)| format!("{n}: {v}")).collect();
    let macro_name = if fields.len() == 1 { "ValidateField" } else { "ValidateFields" };
    log::verbose!("Remote player validation failed: {}({}, {})", macro_name, values.join(", "), condition);
}

/// `ValidateField` / `ValidateFields`: logs, tells the players and makes the caller fail.
fn validate(ctx: &mut Ctx, pnum: usize, ok: bool, condition: &str, fields: &[(&str, String)]) -> bool {
    if !ok {
        log_failed_join_attempt(condition, fields);
        let name = ctx.players.Players[pnum]._pName.as_str().to_string();
        event_failed_join_attempt(ctx, &name);
    }
    ok
}

macro_rules! validate_field {
    ($ctx:expr, $pnum:expr, $value:expr, $cond:expr) => {
        if !validate($ctx, $pnum, $cond, stringify!($cond), &[(stringify!($value), format!("{}", $value))]) {
            return false;
        }
    };
}

macro_rules! validate_fields {
    ($ctx:expr, $pnum:expr, $v1:expr, $v2:expr, $cond:expr) => {
        if !validate($ctx, $pnum, $cond, stringify!($cond), &[(stringify!($v1), format!("{}", $v1)), (stringify!($v2), format!("{}", $v2))]) {
            return false;
        }
    };
}

/// Original: `devilution::UnPackNetItem` (pack.cpp). `pnum` is the player the item belongs to.
// @port pack.cpp|devilution::UnPackNetItem(const Player &player, const ItemNetPack &packedItem, Item &item) sha=65e7fb9c3b7e
fn unpack_net_item(ctx: &mut Ctx, pnum: usize, packed_item: &crate::msg::NetItem) -> Result<Item, ()> {
    let mut item = Item::default();
    let idx = packed_item.w_indx();
    if idx < 0 || idx > IDI_LAST {
        return Ok(item);
    }
    if idx == IDI_EAR {
        crate::items::recreate_ear(ctx, &mut item, packed_item.w_ci(), packed_item.dw_seed(), packed_item.b_cursval(), &packed_item.heroname());
        return Ok(item);
    }
    let fail = || -> Result<Item, ()> { Err(()) };
    let creation_flags = packed_item.w_ci();
    let dw_buff = packed_item.dw_buff();
    let ok = |ctx: &mut Ctx, ok: bool, cond: &str, fields: &[(&str, String)]| validate(ctx, pnum, ok, cond, fields);
    if idx != IDI_GOLD && !ok(ctx, is_creation_flag_combo_valid(creation_flags), "IsCreationFlagComboValid(creationFlags)", &[("creationFlags", creation_flags.to_string())]) {
        return fail();
    }
    let both = [("creationFlags", creation_flags.to_string()), ("dwBuff", dw_buff.to_string())];
    if (creation_flags & CF_TOWN as u16) != 0 {
        if !ok(ctx, is_town_item_valid(creation_flags), "IsTownItemValid(creationFlags)", &[("creationFlags", creation_flags.to_string())]) {
            return fail();
        }
    } else if (creation_flags & CF_USEFUL as u16) == CF_UPER15 as u16 {
        if !ok(ctx, is_unique_monster_item_valid(creation_flags, dw_buff), "IsUniqueMonsterItemValid(creationFlags, dwBuff)", &both) {
            return fail();
        }
    } else if (dw_buff & CF_HELLFIRE as u32) != 0 && crate::tables::itemdat::AllItemsList[idx as usize].iMiscId == IMISC_BOOK {
        let player = ctx.players.Players[pnum].clone();
        return if recreate_hellfire_spell_book(ctx, &player, packed_item, Some(&mut item), pnum) { Ok(item) } else { Err(()) };
    } else if !ok(ctx, is_dungeon_item_valid(creation_flags, dw_buff), "IsDungeonItemValid(creationFlags, dwBuff)", &both) {
        return fail();
    }
    let player = ctx.players.Players[pnum].clone();
    crate::msg::recreate_item(ctx, &player, packed_item, &mut item);
    Ok(item)
}

/// Original: `devilution::UnPackNetPlayer` (pack.cpp): rebuilds a joining player from the
/// `PlayerNetPack` bytes and checks every value against what the game would compute.
// @port pack.cpp|devilution::UnPackNetPlayer(const PlayerNetPack &packed, Player &player) sha=30d2b8a74114
pub fn unpack_net_player(ctx: &mut Ctx, packed: &[u8], pnum: usize) -> bool {
    let r = &mut Reader::new(packed);
    let plrlevel = r.u8();
    let px = r.u8();
    let py = r.u8();
    let p_name: [u8; PlayerNameLength] = r.take();
    let p_class = r.u8();
    let p_base_str = r.u8();
    let p_base_mag = r.u8();
    let p_base_dex = r.u8();
    let p_base_vit = r.u8();
    let p_level = r.i8();
    let p_stat_pts = r.u8();
    let p_experience = r.u32();
    let p_hp_base = r.i32();
    let p_max_hp_base = r.i32();
    let p_mana_base = r.i32();
    let p_max_mana_base = r.i32();
    let p_spl_lvl: [u8; crate::items::MAX_SPELLS as usize] = r.take();
    let p_mem_spells = r.u64();
    let take_item = |r: &mut Reader| crate::msg::NetItem(r.take());
    let inv_body: Vec<crate::msg::NetItem> = (0..NUM_INVLOC_USIZE).map(|_| take_item(r)).collect();
    let inv_list: Vec<crate::msg::NetItem> = (0..InventoryGridCells).map(|_| take_item(r)).collect();
    let inv_grid: [u8; InventoryGridCells] = r.take();
    let p_num_inv = r.u8();
    let spd_list: Vec<crate::msg::NetItem> = (0..MaxBeltItems).map(|_| take_item(r)).collect();
    let p_mana_shield = r.u8();
    let w_reflections = r.u16();
    let p_diablo_kill_level = r.u8();
    let friendly_mode = r.u8();
    let is_on_set_level = r.u8();
    let mut check = [0i32; 23];
    for v in check.iter_mut() {
        *v = r.i32();
    }
    let [p_strength, p_magic, p_dexterity, p_vitality, p_hit_points, p_max_hp, p_mana, p_max_mana, p_damage_mod, p_base_to_blk, p_i_min_dam, p_i_max_dam, p_i_ac, p_i_bonus_dam, p_i_bonus_to_hit, p_i_bonus_ac, p_i_bonus_dam_mod, p_i_get_hit, p_i_en_ac, p_i_f_min_dam, p_i_f_max_dam, p_i_l_min_dam, p_i_l_max_dam] = check;

    let name = CStr::<PlayerNameLength>::from_raw(&p_name);
    let name = name.as_str().to_string();
    ctx.players.Players[pnum]._pName.set(&name);

    validate_field!(ctx, pnum, p_class, p_class <= HeroClass::LAST as u8);
    ctx.players.Players[pnum]._pClass = HeroClass::from_raw(p_class);

    let position = Point::new(px as i32, py as i32);
    validate_fields!(ctx, pnum, position.x, position.y, crate::levels::gendung::in_dungeon_bounds(position));
    validate_field!(ctx, pnum, plrlevel, (plrlevel as usize) < NUMLEVELS as usize);
    validate_field!(ctx, pnum, p_level, p_level >= 1 && p_level as i32 <= MaxCharacterLevel);

    let base_hp_max = p_max_hp_base;
    let base_hp = p_hp_base;
    let hp_max = p_max_hp;
    validate_fields!(ctx, pnum, base_hp, base_hp_max, base_hp >= base_hp_max.wrapping_sub(hp_max) && base_hp <= base_hp_max);

    let base_mana_max = p_max_mana_base;
    let base_mana = p_mana_base;
    validate_fields!(ctx, pnum, base_mana, base_mana_max, base_mana <= base_mana_max);

    let max = |ctx: &Ctx, a| ctx.players.Players[pnum].get_maximum_attribute_value(a);
    validate_fields!(ctx, pnum, p_class, p_base_str, (p_base_str as i32) <= max(ctx, CharacterAttribute::Strength));
    validate_fields!(ctx, pnum, p_class, p_base_mag, (p_base_mag as i32) <= max(ctx, CharacterAttribute::Magic));
    validate_fields!(ctx, pnum, p_class, p_base_dex, (p_base_dex as i32) <= max(ctx, CharacterAttribute::Dexterity));
    validate_fields!(ctx, pnum, p_class, p_base_vit, (p_base_vit as i32) <= max(ctx, CharacterAttribute::Vitality));

    validate_field!(ctx, pnum, p_num_inv, (p_num_inv as usize) <= InventoryGridCells);

    {
        let player = &mut ctx.players.Players[pnum];
        player._pLevel = p_level;
        player.position.tile = position;
        player.position.future = position;
        player.plrlevel = plrlevel;
        player.plrIsOnSetLevel = is_on_set_level != 0;
        player._pMaxHPBase = base_hp_max;
        player._pHPBase = base_hp;
        player._pMaxHP = base_hp_max;
        player._pHitPoints = base_hp;
    }

    crate::player::clr_plr_path(ctx, pnum);
    ctx.players.Players[pnum].destAction = ACTION_NONE;

    crate::player::init_player(ctx, pnum, true);

    {
        let player = &mut ctx.players.Players[pnum];
        player._pBaseStr = p_base_str as i32;
        player._pStrength = player._pBaseStr;
        player._pBaseMag = p_base_mag as i32;
        player._pMagic = player._pBaseMag;
        player._pBaseDex = p_base_dex as i32;
        player._pDexterity = player._pBaseDex;
        player._pBaseVit = p_base_vit as i32;
        player._pVitality = player._pBaseVit;
        player._pStatPts = p_stat_pts as i32;

        player._pExperience = p_experience;
        player._pBaseToBlk = crate::tables::playerdat::PlayersData[player._pClass as usize].blockBonus as i32;
        player._pMaxManaBase = base_mana_max;
        player._pManaBase = base_mana;
        player._pMemSpells = p_mem_spells;
        player.wReflections = w_reflections;
        player.pDiabloKillLevel = p_diablo_kill_level;
        player.pManaShield = p_mana_shield != 0;
        player.friendlyMode = friendly_mode != 0;

        for i in 0..crate::items::MAX_SPELLS as usize {
            player._pSplLvl[i] = p_spl_lvl[i];
        }
    }

    for i in 0..NUM_INVLOC_USIZE {
        let Ok(item) = unpack_net_item(ctx, pnum, &inv_body[i]) else { return false };
        ctx.players.Players[pnum].InvBody[i] = item;
        if ctx.players.Players[pnum].InvBody[i].is_empty() {
            continue;
        }
        let loc = ctx.players.Players[pnum].get_item_location(&ctx.players.Players[pnum].InvBody[i]) as i8;
        match i as inv_body_loc {
            INVLOC_HEAD => validate_field!(ctx, pnum, loc, loc == ILOC_HELM as i8),
            INVLOC_RING_LEFT | INVLOC_RING_RIGHT => validate_field!(ctx, pnum, loc, loc == ILOC_RING as i8),
            INVLOC_AMULET => validate_field!(ctx, pnum, loc, loc == ILOC_AMULET as i8),
            INVLOC_HAND_LEFT | INVLOC_HAND_RIGHT => validate_field!(ctx, pnum, loc, loc == ILOC_ONEHAND as i8 || loc == ILOC_TWOHAND as i8),
            INVLOC_CHEST => validate_field!(ctx, pnum, loc, loc == ILOC_ARMOR as i8),
            _ => {}
        }
    }

    ctx.players.Players[pnum]._pNumInv = p_num_inv as i32;
    for i in 0..p_num_inv as usize {
        let Ok(item) = unpack_net_item(ctx, pnum, &inv_list[i]) else { return false };
        ctx.players.Players[pnum].InvList[i] = item;
    }

    for i in 0..InventoryGridCells {
        ctx.players.Players[pnum].InvGrid[i] = inv_grid[i] as i8;
    }

    for i in 0..MaxBeltItems {
        let Ok(item) = unpack_net_item(ctx, pnum, &spd_list[i]) else { return false };
        ctx.players.Players[pnum].SpdList[i] = item;
        let item = &ctx.players.Players[pnum].SpdList[i];
        if item.is_empty() {
            continue;
        }
        let belt_item_size = crate::inv::get_inventory_size(item);
        let belt_item_type = item._itype as i8;
        let belt_item_usable = item.is_usable(ctx);
        let is_gold = item._itype == ItemType::Gold;
        validate_fields!(ctx, pnum, belt_item_size.width, belt_item_size.height, belt_item_size == crate::engine::geometry::Size::new(1, 1));
        validate_field!(ctx, pnum, belt_item_type, !is_gold);
        validate_field!(ctx, pnum, belt_item_usable, belt_item_usable);
    }

    crate::items::calc_plr_inv(ctx, pnum, false);
    let gold = crate::inv::calculate_gold(ctx, pnum);
    ctx.players.Players[pnum]._pGold = gold;

    let p = ctx.players.Players[pnum].clone();
    validate_fields!(ctx, pnum, p._pStrength, p_strength, p._pStrength == p_strength);
    validate_fields!(ctx, pnum, p._pMagic, p_magic, p._pMagic == p_magic);
    validate_fields!(ctx, pnum, p._pDexterity, p_dexterity, p._pDexterity == p_dexterity);
    validate_fields!(ctx, pnum, p._pVitality, p_vitality, p._pVitality == p_vitality);
    validate_fields!(ctx, pnum, p._pHitPoints, p_hit_points, p._pHitPoints == p_hit_points);
    validate_fields!(ctx, pnum, p._pMaxHP, p_max_hp, p._pMaxHP == p_max_hp);
    validate_fields!(ctx, pnum, p._pMana, p_mana, p._pMana == p_mana);
    validate_fields!(ctx, pnum, p._pMaxMana, p_max_mana, p._pMaxMana == p_max_mana);
    validate_fields!(ctx, pnum, p._pDamageMod, p_damage_mod, p._pDamageMod == p_damage_mod);
    validate_fields!(ctx, pnum, p._pBaseToBlk, p_base_to_blk, p._pBaseToBlk == p_base_to_blk);
    validate_fields!(ctx, pnum, p._pIMinDam, p_i_min_dam, p._pIMinDam == p_i_min_dam);
    validate_fields!(ctx, pnum, p._pIMaxDam, p_i_max_dam, p._pIMaxDam == p_i_max_dam);
    validate_fields!(ctx, pnum, p._pIAC, p_i_ac, p._pIAC == p_i_ac);
    validate_fields!(ctx, pnum, p._pIBonusDam, p_i_bonus_dam, p._pIBonusDam == p_i_bonus_dam);
    validate_fields!(ctx, pnum, p._pIBonusToHit, p_i_bonus_to_hit, p._pIBonusToHit == p_i_bonus_to_hit);
    validate_fields!(ctx, pnum, p._pIBonusAC, p_i_bonus_ac, p._pIBonusAC == p_i_bonus_ac);
    validate_fields!(ctx, pnum, p._pIBonusDamMod, p_i_bonus_dam_mod, p._pIBonusDamMod == p_i_bonus_dam_mod);
    validate_fields!(ctx, pnum, p._pIGetHit, p_i_get_hit, p._pIGetHit == p_i_get_hit);
    validate_fields!(ctx, pnum, p._pIEnAc, p_i_en_ac, p._pIEnAc == p_i_en_ac);
    validate_fields!(ctx, pnum, p._pIFMinDam, p_i_f_min_dam, p._pIFMinDam == p_i_f_min_dam);
    validate_fields!(ctx, pnum, p._pIFMaxDam, p_i_f_max_dam, p._pIFMaxDam == p_i_f_max_dam);
    validate_fields!(ctx, pnum, p._pILMinDam, p_i_l_min_dam, p._pILMinDam == p_i_l_min_dam);
    validate_fields!(ctx, pnum, p._pILMaxDam, p_i_l_max_dam, p._pILMaxDam == p_i_l_max_dam);
    validate_fields!(ctx, pnum, p._pMaxHPBase, p.calculate_base_life(), p._pMaxHPBase <= p.calculate_base_life());
    validate_fields!(ctx, pnum, p._pMaxManaBase, p.calculate_base_mana(), p._pMaxManaBase <= p.calculate_base_mana());

    true
}

