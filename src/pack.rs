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

crate::pending_fn!(pub fn unpack_net_player(ctx: &mut Ctx, packed: &[u8], pnum: usize) -> bool, "pack.cpp|devilution::UnPackNetPlayer(const PlayerNetPack &packed, Player &player)");
