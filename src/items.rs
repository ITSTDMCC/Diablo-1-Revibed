//! `Source/items.cpp`: items on the ground, item generation (drops, uniques, affixes, vendor
//! stock), item stats and descriptions.
//!
//! Functions that modify an `Item` take it by `&mut` next to `ctx`; when the item lives inside
//! `ctx` (ground items, inventories) the caller takes it out and puts it back. Item generation
//! that reads a player's stats takes the player by reference (a copy when it is the context's own).

use crate::ctx::Ctx;
use crate::engine::animationinfo::AnimationInfo;
use crate::engine::backbuffer_state::{redraw_component, PanelDrawComponent};
use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::geometry::{Direction, Displacement, Point};
use crate::engine::render::text_render::UiFlags;
use crate::enums::*;
use crate::levels::gendung::{in_dungeon_bounds, DungeonType, MAXDUNX, MAXDUNY};
use crate::player::Player;
use crate::tables::itemdat::*;
use crate::tables::items_tables::*;
use crate::tables::spelldat::SpellsData;
use crate::utils::cstr::CStr;
use crate::utils::language::{ngettext, pgettext, tr};

pub const MAXITEMS: usize = 127;
pub const ITEMTYPES: usize = 43;
pub const GOLD_SMALL_LIMIT: i32 = 1000;
pub const GOLD_MEDIUM_LIMIT: i32 = 2500;
pub const GOLD_MAX_LIMIT: i32 = 5000;
pub const DUR_INDESTRUCTIBLE: i32 = 255;
pub const ITEM_ANIM_WIDTH: u16 = 96;
pub const MAX_SPELLS: i32 = 52;

/// `PlayerArmorGraphic`
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum PlayerArmorGraphic {
    Light = 0,
    Medium = 1 << 4,
    Heavy = 1 << 5,
}

/// `Item`
#[derive(Clone, Debug)]
pub struct Item {
    /// Randomly generated identifier
    pub _iSeed: u32,
    pub _iCreateInfo: u16,
    pub _itype: ItemType,
    pub _iAnimFlag: bool,
    pub position: Point,
    pub AnimInfo: AnimationInfo,
    pub _iDelFlag: bool,
    pub _iSelFlag: u8,
    pub _iPostDraw: bool,
    pub _iIdentified: bool,
    pub _iMagical: item_quality,
    pub _iName: CStr<64>,
    pub _iIName: CStr<64>,
    pub _iLoc: item_equip_type,
    pub _iClass: item_class,
    pub _iCurs: u8,
    pub _ivalue: i32,
    pub _iIvalue: i32,
    pub _iMinDam: u8,
    pub _iMaxDam: u8,
    pub _iAC: i16,
    pub _iFlags: ItemSpecialEffect,
    pub _iMiscId: item_misc_id,
    pub _iSpell: SpellID,
    pub IDidx: _item_indexes,
    pub _iCharges: i32,
    pub _iMaxCharges: i32,
    pub _iDurability: i32,
    pub _iMaxDur: i32,
    pub _iPLDam: i16,
    pub _iPLToHit: i16,
    pub _iPLAC: i16,
    pub _iPLStr: i16,
    pub _iPLMag: i16,
    pub _iPLDex: i16,
    pub _iPLVit: i16,
    pub _iPLFR: i16,
    pub _iPLLR: i16,
    pub _iPLMR: i16,
    pub _iPLMana: i16,
    pub _iPLHP: i16,
    pub _iPLDamMod: i16,
    pub _iPLGetHit: i16,
    pub _iPLLight: i16,
    pub _iSplLvlAdd: i8,
    pub _iRequest: bool,
    /// Unique item ID, used as an index into UniqueItemList
    pub _iUid: i32,
    pub _iFMinDam: i16,
    pub _iFMaxDam: i16,
    pub _iLMinDam: i16,
    pub _iLMaxDam: i16,
    pub _iPLEnAc: i16,
    pub _iPrePower: item_effect_type,
    pub _iSufPower: item_effect_type,
    pub _iVAdd1: i32,
    pub _iVMult1: i32,
    pub _iVAdd2: i32,
    pub _iVMult2: i32,
    pub _iMinStr: i8,
    pub _iMinMag: u8,
    pub _iMinDex: i8,
    pub _iStatFlag: bool,
    pub _iDamAcFlags: ItemSpecialEffectHf,
    pub dwBuff: u32,
}

impl Default for Item {
    fn default() -> Self {
        Item {
            _iSeed: 0,
            _iCreateInfo: 0,
            _itype: ItemType::None,
            _iAnimFlag: false,
            position: Point::new(0, 0),
            AnimInfo: AnimationInfo::default(),
            _iDelFlag: false,
            _iSelFlag: 0,
            _iPostDraw: false,
            _iIdentified: false,
            _iMagical: ITEM_QUALITY_NORMAL,
            _iName: CStr::default(),
            _iIName: CStr::default(),
            _iLoc: ILOC_NONE,
            _iClass: ICLASS_NONE,
            _iCurs: 0,
            _ivalue: 0,
            _iIvalue: 0,
            _iMinDam: 0,
            _iMaxDam: 0,
            _iAC: 0,
            _iFlags: ItemSpecialEffect::None,
            _iMiscId: IMISC_NONE,
            _iSpell: SpellID::Null,
            IDidx: IDI_NONE,
            _iCharges: 0,
            _iMaxCharges: 0,
            _iDurability: 0,
            _iMaxDur: 0,
            _iPLDam: 0,
            _iPLToHit: 0,
            _iPLAC: 0,
            _iPLStr: 0,
            _iPLMag: 0,
            _iPLDex: 0,
            _iPLVit: 0,
            _iPLFR: 0,
            _iPLLR: 0,
            _iPLMR: 0,
            _iPLMana: 0,
            _iPLHP: 0,
            _iPLDamMod: 0,
            _iPLGetHit: 0,
            _iPLLight: 0,
            _iSplLvlAdd: 0,
            _iRequest: false,
            _iUid: 0,
            _iFMinDam: 0,
            _iFMaxDam: 0,
            _iLMinDam: 0,
            _iLMaxDam: 0,
            _iPLEnAc: 0,
            _iPrePower: IPL_INVALID,
            _iSufPower: IPL_INVALID,
            _iVAdd1: 0,
            _iVMult1: 0,
            _iVAdd2: 0,
            _iVMult2: 0,
            _iMinStr: 0,
            _iMinMag: 0,
            _iMinDex: 0,
            _iStatFlag: false,
            _iDamAcFlags: ItemSpecialEffectHf::None,
            dwBuff: 0,
        }
    }
}

/// `int16_t` arithmetic: `field += v` with the C++ implicit narrowing.
#[inline]
fn add16(a: i16, b: i32) -> i16 {
    (a as i32).wrapping_add(b) as i16
}

impl Item {
    /// `pop`: clears this item and returns the old value.
    // @port items.h|devilution::Item::pop() sha=0f91f5182dd1
    pub fn pop(&mut self) -> Item {
        let temp = self.clone();
        self.clear();
        temp
    }

    /// `clear`: resets the item so `isEmpty()` returns true.
    // @port items.h|devilution::Item::clear() sha=1643fed50bad
    pub fn clear(&mut self) {
        self._itype = ItemType::None;
    }

    // @port items.h|devilution::Item::isEmpty() sha=5b1c0fa437f4
    pub fn is_empty(&self) -> bool {
        self._itype == ItemType::None
    }

    // @port items.h|devilution::Item::isEquipment() sha=f3e34a5347d8
    pub fn is_equipment(&self) -> bool {
        if self.is_empty() {
            return false;
        }
        matches!(self._iLoc, ILOC_AMULET | ILOC_ARMOR | ILOC_HELM | ILOC_ONEHAND | ILOC_RING | ILOC_TWOHAND)
    }

    // @port items.h|devilution::Item::isWeapon() sha=cffb72000084
    pub fn is_weapon(&self) -> bool {
        if self.is_empty() {
            return false;
        }
        matches!(self._itype, ItemType::Axe | ItemType::Bow | ItemType::Mace | ItemType::Staff | ItemType::Sword)
    }

    // @port items.h|devilution::Item::isArmor() sha=9002669fc2d4
    pub fn is_armor(&self) -> bool {
        if self.is_empty() {
            return false;
        }
        matches!(self._itype, ItemType::HeavyArmor | ItemType::LightArmor | ItemType::MediumArmor)
    }

    // @port items.h|devilution::Item::isHelm() sha=ac3c3abf0bf2
    pub fn is_helm(&self) -> bool {
        !self.is_empty() && self._itype == ItemType::Helm
    }

    // @port items.h|devilution::Item::isShield() sha=ab714161120e
    pub fn is_shield(&self) -> bool {
        !self.is_empty() && self._itype == ItemType::Shield
    }

    // @port items.h|devilution::Item::isJewelry() sha=439c015cd939
    pub fn is_jewelry(&self) -> bool {
        if self.is_empty() {
            return false;
        }
        matches!(self._itype, ItemType::Amulet | ItemType::Ring)
    }

    // @port items.h|devilution::Item::isScroll() sha=0ae7c6523309
    pub fn is_scroll(&self) -> bool {
        matches!(self._iMiscId, IMISC_SCROLL | IMISC_SCROLLT)
    }

    // @port items.h|devilution::Item::isScrollOf(SpellID spellId) sha=34e49400e661
    pub fn is_scroll_of(&self, spell_id: SpellID) -> bool {
        self.is_scroll() && self._iSpell == spell_id
    }

    // @port items.h|devilution::Item::isRune() sha=751b60db97cd
    pub fn is_rune(&self) -> bool {
        self._iMiscId > IMISC_RUNEFIRST && self._iMiscId < IMISC_RUNELAST
    }

    // @port items.h|devilution::Item::isRuneOf(SpellID spellId) sha=ba7a8b6b3394
    pub fn is_rune_of(&self, spell_id: SpellID) -> bool {
        if !self.is_rune() {
            return false;
        }
        match self._iMiscId {
            IMISC_RUNEF => spell_id == SpellID::RuneOfFire,
            IMISC_RUNEL => spell_id == SpellID::RuneOfLight,
            IMISC_GR_RUNEL => spell_id == SpellID::RuneOfNova,
            IMISC_GR_RUNEF => spell_id == SpellID::RuneOfImmolation,
            IMISC_RUNES => spell_id == SpellID::RuneOfStone,
            _ => false,
        }
    }

    // @port items.h|devilution::Item::keyAttributesMatch(uint32_t seed, _item_indexes itemIndex, uint16_t createInfo) sha=dbbcc003ed39
    pub fn key_attributes_match(&self, seed: u32, item_index: _item_indexes, create_info: u16) -> bool {
        self._iSeed == seed && self.IDidx == item_index && self._iCreateInfo == create_info
    }

    // @port items.h|devilution::Item::getTextColor() sha=4c84182a8518
    pub fn get_text_color(&self) -> UiFlags {
        match self._iMagical {
            ITEM_QUALITY_MAGIC => UiFlags::COLOR_BLUE,
            ITEM_QUALITY_UNIQUE => UiFlags::COLOR_WHITEGOLD,
            _ => UiFlags::COLOR_WHITE,
        }
    }

    // @port items.h|devilution::Item::getTextColorWithStatCheck() sha=0b535365d4ec
    pub fn get_text_color_with_stat_check(&self) -> UiFlags {
        if !self._iStatFlag {
            return UiFlags::COLOR_RED;
        }
        self.get_text_color()
    }

    /// Original: `Item::isUsable` (items.cpp).
    // @port items.cpp|devilution::Item::isUsable() sha=d42b87cd4c51
    pub fn is_usable(&self, ctx: &Ctx) -> bool {
        if self.IDidx == IDI_SPECELIX && ctx.quests.Quests[Q_MUSHROOM as usize]._qactive != QUEST_DONE {
            return false;
        }
        AllItemsList[self.IDidx as usize].iUsable
    }

    /// Original: `Item::setNewAnimation` (items.cpp).
    // @port items.cpp|devilution::Item::setNewAnimation(bool showAnimation) sha=5fefb71e2b28
    pub fn set_new_animation(&mut self, ctx: &Ctx, show_animation: bool) {
        let it = ItemCAnimTbl[self._iCurs as usize];
        let number_of_frames = ItemAnimLs[it as usize];
        let sprite = ctx.items.itemanims[it as usize].clone();
        if self._iCurs as item_cursor_graphic != ICURS_MAGIC_ROCK {
            self.AnimInfo.set_new_animation(sprite, number_of_frames, 1, AnimationDistributionFlags::ProcessAnimationPending, 0, number_of_frames, 0);
        } else {
            self.AnimInfo.set_new_animation_simple(sprite, number_of_frames, 1);
        }
        self._iPostDraw = false;
        self._iRequest = false;
        if show_animation {
            self._iAnimFlag = true;
            self._iSelFlag = 0;
        } else {
            self.AnimInfo.currentFrame = self.AnimInfo.numberOfFrames - 1;
            self._iAnimFlag = false;
            self._iSelFlag = 1;
        }
    }

    /// Original: `Item::updateRequiredStatsCacheForPlayer` (items.cpp).
    // @port items.cpp|devilution::Item::updateRequiredStatsCacheForPlayer(const Player &player) sha=c1adc545a098
    pub fn update_required_stats_cache_for_player(&mut self, player: &Player) {
        if self._itype == ItemType::Misc && self._iMiscId == IMISC_BOOK {
            self._iMinMag = get_spell_data(self._iSpell).minInt;
            let mut spell_level = player._pSplLvl[self._iSpell as i8 as usize] as i8;
            while spell_level != 0 {
                self._iMinMag = (self._iMinMag as i32 + 20 * self._iMinMag as i32 / 100) as u8;
                spell_level -= 1;
                if self._iMinMag as i32 + 20 * self._iMinMag as i32 / 100 > 255 {
                    self._iMinMag = 255;
                    spell_level = 0;
                }
            }
        }
        self._iStatFlag = player.can_use_item(self);
    }

    /// Original: `Item::getName` (items.cpp): the translated name to display.
    // @port items.cpp|devilution::Item::getName() sha=9a6a32b4b730
    pub fn get_name(&self, ctx: &mut Ctx) -> String {
        if self.is_empty() {
            String::new()
        } else if !self._iIdentified || self._iCreateInfo == 0 || self._iMagical == ITEM_QUALITY_NORMAL {
            get_translated_item_name(ctx, self)
        } else if self._iMagical == ITEM_QUALITY_UNIQUE {
            tr(UniqueItems[self._iUid as usize].UIName)
        } else {
            get_translated_item_name_magical(ctx, self, self.dwBuff & CF_HELLFIRE as u32 != 0, true, None)
        }
    }
}

/// `GetSpellData`
pub fn get_spell_data(spell_id: SpellID) -> &'static crate::tables::spelldat::SpellData {
    &SpellsData[spell_id as i8 as usize]
}

/// `SpellData::type()`
pub fn spell_magic_type(spell_id: SpellID) -> MagicType {
    match get_spell_data(spell_id).flags.0 & 0b11 {
        0 => MagicType::Fire,
        1 => MagicType::Lightning,
        _ => MagicType::Magic,
    }
}

/// `ItemGetRecordStruct`
#[derive(Clone, Copy, Debug, Default)]
pub struct ItemGetRecordStruct {
    pub nSeed: u32,
    pub wCI: u16,
    pub nIndex: i32,
    pub dwTimestamp: u32,
}

/// `CornerStoneStruct`
#[derive(Clone, Debug, Default)]
pub struct CornerStoneStruct {
    pub position: Point,
    pub activated: bool,
    pub item: Item,
}

impl CornerStoneStruct {
    /// Original: `CornerStoneStruct::isAvailable` (items.cpp).
    // @port items.cpp|devilution::CornerStoneStruct::isAvailable() sha=ac75cd3cda71
    pub fn is_available(ctx: &Ctx) -> bool {
        ctx.gendung.currlevel == 21 && !ctx.init.gb_is_multiplayer
    }
}

/// Globals of items.cpp.
pub struct ItemsState {
    pub Items: Vec<Item>,
    pub ActiveItems: [u8; MAXITEMS],
    pub ActiveItemCount: u8,
    pub dItem: Box<crate::levels::gendung::DunArray<i8>>,
    pub ShowUniqueItemInfoBox: bool,
    pub CornerStone: CornerStoneStruct,
    pub UniqueItemFlags: [bool; 128],
    pub MaxGold: i32,
    /// `itemanims`
    pub itemanims: Vec<Option<ClxSpriteList>>,
    curruitem: Item,
    itemrecord: [ItemGetRecordStruct; MAXITEMS],
    itemhold: [[bool; 3]; 3],
    gnNumGetRecords: i32,
    /// static `idoppely` in ItemDoppel
    idoppely: i32,
}

impl Default for ItemsState {
    fn default() -> Self {
        ItemsState {
            Items: vec![Item::default(); MAXITEMS + 1],
            ActiveItems: [0; MAXITEMS],
            ActiveItemCount: 0,
            dItem: Box::new([[0; MAXDUNY]; MAXDUNX]),
            ShowUniqueItemInfoBox: false,
            CornerStone: CornerStoneStruct::default(),
            UniqueItemFlags: [false; 128],
            MaxGold: GOLD_MAX_LIMIT,
            itemanims: (0..ITEMTYPES).map(|_| None).collect(),
            curruitem: Item::default(),
            itemrecord: [ItemGetRecordStruct::default(); MAXITEMS],
            itemhold: [[false; 3]; 3],
            gnNumGetRecords: 0,
            idoppely: 16,
        }
    }
}

/// Original: `IsPrefixValidForItemType` (items.cpp).
// @port items.cpp|devilution::IsPrefixValidForItemType(int i, AffixItemType flgs, bool hellfireItem) sha=3e1c3c3e9eeb
fn is_prefix_valid_for_item_type(i: usize, flgs: AffixItemType, hellfire_item: bool) -> bool {
    let mut item_types = ItemPrefixes[i].PLIType;
    if !hellfire_item {
        if i > 82 {
            return false;
        }
        if (12..=20).contains(&i) {
            item_types &= !AffixItemType::Staff;
        }
    }
    flgs.has_any_of(item_types)
}

/// Original: `IsSuffixValidForItemType` (items.cpp).
// @port items.cpp|devilution::IsSuffixValidForItemType(int i, AffixItemType flgs, bool hellfireItem) sha=6a1c7fe586ad
fn is_suffix_valid_for_item_type(i: usize, flgs: AffixItemType, hellfire_item: bool) -> bool {
    let mut item_types = ItemSuffixes[i].PLIType;
    if !hellfire_item {
        if i > 94 {
            return false;
        }
        if i <= 1 || (14..=15).contains(&i) || (21..=22).contains(&i) || (34..=36).contains(&i) || (41..=44).contains(&i) || (60..=63).contains(&i) {
            item_types &= !AffixItemType::Staff;
        }
    }
    flgs.has_any_of(item_types)
}

/// Original: `ItemsGetCurrlevel` (items.cpp).
// @port items.cpp|devilution::ItemsGetCurrlevel() sha=cd96a7b82af4
fn items_get_currlevel(ctx: &Ctx) -> i32 {
    let g = &ctx.gendung;
    if g.setlevel {
        let q = &ctx.quests.Quests;
        return match g.setlvlnum {
            SL_SKELKING => q[Q_SKELKING as usize]._qlevel as i32,
            SL_BONECHAMB => q[Q_SCHAMB as usize]._qlevel as i32,
            SL_POISONWATER => q[Q_PWATER as usize]._qlevel as i32,
            SL_VILEBETRAYER => q[Q_BETRAYER as usize]._qlevel as i32,
            _ => 1,
        };
    }
    if g.leveltype == DungeonType::Nest {
        return g.currlevel as i32 - 8;
    }
    if g.leveltype == DungeonType::Crypt {
        return g.currlevel as i32 - 7;
    }
    g.currlevel as i32
}

/// Original: `ItemPlace` (items.cpp).
// @port items.cpp|devilution::ItemPlace(Point position) sha=9d7e7f3ade0b
fn item_place(ctx: &Ctx, position: Point) -> bool {
    let (x, y) = (position.x as usize, position.y as usize);
    if ctx.gendung.dMonster[x][y] != 0 {
        return false;
    }
    if ctx.gendung.dPlayer[x][y] != 0 {
        return false;
    }
    if ctx.items.dItem[x][y] != 0 {
        return false;
    }
    if crate::objects::is_object_at_position(ctx, position) {
        return false;
    }
    if crate::levels::gendung::tile_contains_set_piece(ctx, position) {
        return false;
    }
    if crate::engine::path::is_tile_solid(ctx, position) {
        return false;
    }
    true
}

/// Original: `GetRandomAvailableItemPosition` (items.cpp).
// @port items.cpp|devilution::GetRandomAvailableItemPosition() sha=c935e6805dca
fn get_random_available_item_position(ctx: &mut Ctx) -> Point {
    loop {
        let x = ctx.rng.generate_rnd(80);
        let y = ctx.rng.generate_rnd(80);
        let position = Point::new(x, y) + Displacement::new(16, 16);
        if item_place(ctx, position) {
            return position;
        }
    }
}

/// Runs `f` on ground item `ii` taken out of `Items` (it is put back afterwards).
pub fn with_item<R>(ctx: &mut Ctx, ii: usize, f: impl FnOnce(&mut Ctx, &mut Item) -> R) -> R {
    let mut item = std::mem::take(&mut ctx.items.Items[ii]);
    let r = f(ctx, &mut item);
    ctx.items.Items[ii] = item;
    r
}

/// A copy of the local player, for item generation that reads its stats.
pub fn my_player_copy(ctx: &Ctx) -> Player {
    ctx.players.Players[ctx.players.my_player_id()].clone()
}

/// Original: `AddInitItems` (items.cpp).
// @port items.cpp|devilution::AddInitItems() sha=af539dbe3afc
fn add_init_items(ctx: &mut Ctx) {
    let curlv = items_get_currlevel(ctx);
    let rnd = ctx.rng.generate_rnd(3) + 3;
    for _ in 0..rnd {
        let ii = allocate_item(ctx) as usize;
        let position = get_random_available_item_position(ctx);
        ctx.items.Items[ii].position = position;
        ctx.items.dItem[position.x as usize][position.y as usize] = (ii + 1) as i8;
        let seed = ctx.rng.advance_rnd_seed() as u32;
        ctx.items.Items[ii]._iSeed = seed;
        ctx.rng.set_rnd_seed(seed);
        let idx = ctx.rng.pick_randomly_among(&[IDI_MANA, IDI_HEAL]);
        with_item(ctx, ii, |ctx, item| {
            get_item_attrs(ctx, item, idx, curlv);
            item._iCreateInfo = (curlv | CF_PREGEN) as u16;
            setup_item(ctx, item);
            item.AnimInfo.currentFrame = item.AnimInfo.numberOfFrames - 1;
            item._iAnimFlag = false;
            item._iSelFlag = 1;
        });
        crate::msg::delta_add_item(ctx, ii as i32);
    }
}

/// Original: `SpawnNote` (items.cpp).
// @port items.cpp|devilution::SpawnNote() sha=c976289b29a6
fn spawn_note(ctx: &mut Ctx) {
    let id = match ctx.gendung.currlevel {
        22 => IDI_NOTE2,
        23 => IDI_NOTE3,
        _ => IDI_NOTE1,
    };
    let position = get_random_available_item_position(ctx);
    spawn_quest_item(ctx, id, position, 0, 1, false);
}

/// Original: `CalcSelfItems` (items.cpp).
// @port items.cpp|devilution::CalcSelfItems(Player &player) sha=3a35027a6ed0
fn calc_self_items(player: &mut Player) {
    let mut sa = 0;
    let mut ma = 0;
    let mut da = 0;
    for equipment in player.InvBody.iter_mut().filter(|i| !i.is_empty()) {
        equipment._iStatFlag = true;
        if equipment._iIdentified {
            sa += equipment._iPLStr as i32;
            ma += equipment._iPLMag as i32;
            da += equipment._iPLDex as i32;
        }
    }
    loop {
        let currstr = 0.max(sa + player._pBaseStr);
        let currmag = 0.max(ma + player._pBaseMag);
        let currdex = 0.max(da + player._pBaseDex);
        let mut changeflag = false;
        for equipment in player.InvBody.iter_mut().filter(|i| !i.is_empty()) {
            if !equipment._iStatFlag {
                continue;
            }
            if currstr >= equipment._iMinStr as i32 && currmag >= equipment._iMinMag as i32 && currdex >= equipment._iMinDex as i32 {
                continue;
            }
            changeflag = true;
            equipment._iStatFlag = false;
            if equipment._iIdentified {
                sa -= equipment._iPLStr as i32;
                ma -= equipment._iPLMag as i32;
                da -= equipment._iPLDex as i32;
            }
        }
        if !changeflag {
            break;
        }
    }
}

/// Original: `GetItemSpace` (items.cpp).
// @port items.cpp|devilution::GetItemSpace(Point position, int8_t inum) sha=e389900dd799
fn get_item_space(ctx: &mut Ctx, position: Point, inum: i8) -> bool {
    let mut yy = 0usize;
    for j in position.y - 1..=position.y + 1 {
        let mut xx = 0usize;
        for i in position.x - 1..=position.x + 1 {
            ctx.items.itemhold[xx][yy] = item_space_ok(ctx, Point::new(i, j));
            xx += 1;
        }
        yy += 1;
    }
    let mut savail = false;
    for j in 0..3 {
        for i in 0..3 {
            if ctx.items.itemhold[i][j] {
                savail = true;
            }
        }
    }
    let mut rs = ctx.rng.generate_rnd(15) + 1;
    if !savail {
        return false;
    }
    let mut xx = 0i32;
    let mut yy = 0i32;
    while rs > 0 {
        if ctx.items.itemhold[xx as usize][yy as usize] {
            rs -= 1;
        }
        if rs <= 0 {
            continue;
        }
        xx += 1;
        if xx != 3 {
            continue;
        }
        xx = 0;
        yy += 1;
        if yy == 3 {
            yy = 0;
        }
    }
    xx += position.x - 1;
    yy += position.y - 1;
    ctx.items.Items[inum as usize].position = Point::new(xx, yy);
    ctx.items.dItem[xx as usize][yy as usize] = inum + 1;
    true
}

/// Original: `GetSuperItemSpace` (items.cpp).
// @port items.cpp|devilution::GetSuperItemSpace(Point position, int8_t inum) sha=5f1930545ce1
fn get_super_item_space(ctx: &mut Ctx, position: Point, inum: i8) {
    if get_item_space(ctx, position, inum) {
        return;
    }
    for k in 2..50 {
        for j in -k..=k {
            for i in -k..=k {
                let position_to_check = position + Displacement::new(i, j);
                if !item_space_ok(ctx, position_to_check) {
                    continue;
                }
                ctx.items.Items[inum as usize].position = position_to_check;
                ctx.items.dItem[position_to_check.x as usize][position_to_check.y as usize] = inum + 1;
                return;
            }
        }
    }
}

/// Original: `CalcItemValue` (items.cpp).
// @port items.cpp|devilution::CalcItemValue(Item &item) sha=8ae6e254fb8c
fn calc_item_value(item: &mut Item) {
    let mut v = item._iVMult1 + item._iVMult2;
    if v > 0 {
        v *= item._ivalue;
    }
    if v < 0 {
        v = item._ivalue / v;
    }
    v = item._iVAdd1 + item._iVAdd2 + v;
    item._iIvalue = v.max(1);
}

/// `CopyUtf8(buf + len, src, sizeof(buf) - len)`: appends to a fixed-size name.
fn append_name(dst: &mut CStr<64>, s: &str) {
    let mut v = dst.as_str().to_string();
    let room = 64 - v.len();
    v.push_str(&crate::utils::utf8::copy_utf8(s, room));
    dst.set(&v);
}

/// Original: `GetBookSpell` (items.cpp).
// @port items.cpp|devilution::GetBookSpell(Item &item, int lvl) sha=80760f7a4d0f
fn get_book_spell(ctx: &mut Ctx, item: &mut Item, mut lvl: i32) {
    if lvl == 0 {
        lvl = 1;
    }
    let max_spells = if ctx.init.gb_is_hellfire { MAX_SPELLS } else { 37 };
    let mut rv = ctx.rng.generate_rnd(max_spells) + 1;
    if ctx.init.gb_is_spawn && lvl > 5 {
        lvl = 5;
    }
    let mut s = SpellID::Firebolt as i32;
    let mut bs = SpellID::Firebolt;
    while rv > 0 {
        let s_level = crate::spells::get_spell_book_level(ctx, SpellID::from_raw(s as i8));
        if s_level != -1 && lvl >= s_level {
            rv -= 1;
            bs = SpellID::from_raw(s as i8);
        }
        s += 1;
        if !ctx.init.gb_is_multiplayer && s == SpellID::Resurrect as i32 {
            s = SpellID::Telekinesis as i32;
        }
        if !ctx.init.gb_is_multiplayer && s == SpellID::HealOther as i32 {
            s = SpellID::BloodStar as i32;
        }
        if s == max_spells {
            s = 1;
        }
    }
    let spell_name = get_spell_data(bs).sNameText;
    append_name(&mut item._iName, spell_name);
    append_name(&mut item._iIName, spell_name);
    item._iSpell = bs;
    let spell_data = get_spell_data(bs);
    item._iMinMag = spell_data.minInt;
    item._ivalue += spell_data.bookCost10 as i32 * 10;
    item._iIvalue += spell_data.bookCost10 as i32 * 10;
    item._iCurs = match spell_magic_type(bs) {
        MagicType::Fire => ICURS_BOOK_RED,
        MagicType::Lightning => ICURS_BOOK_BLUE,
        MagicType::Magic => ICURS_BOOK_GREY,
    } as u8;
}

/// Original: `RndPL` (items.cpp).
// @port items.cpp|devilution::RndPL(int param1, int param2) sha=31fb1f2514e3
fn rnd_pl(ctx: &mut Ctx, param1: i32, param2: i32) -> i32 {
    param1 + ctx.rng.generate_rnd(param2 - param1 + 1)
}

/// Original: `CalculateToHitBonus` (items.cpp).
// @port items.cpp|devilution::CalculateToHitBonus(int level) sha=d74e9aafe356
fn calculate_to_hit_bonus(ctx: &mut Ctx, level: i32) -> i32 {
    match level {
        -50 => -rnd_pl(ctx, 6, 10),
        -25 => -rnd_pl(ctx, 1, 5),
        20 => rnd_pl(ctx, 1, 5),
        36 => rnd_pl(ctx, 6, 10),
        51 => rnd_pl(ctx, 11, 15),
        66 => rnd_pl(ctx, 16, 20),
        81 => rnd_pl(ctx, 21, 30),
        96 => rnd_pl(ctx, 31, 40),
        111 => rnd_pl(ctx, 41, 50),
        126 => rnd_pl(ctx, 51, 75),
        151 => rnd_pl(ctx, 76, 100),
        _ => crate::appfat::app_fatal(ctx, "Unknown to hit bonus"),
    }
}

/// Original: `SaveItemPower` (items.cpp).
// @port items.cpp|devilution::SaveItemPower(const Player &player, Item &item, ItemPower &power) sha=22714669ee1a
fn save_item_power(ctx: &mut Ctx, player: &Player, item: &mut Item, power: &mut ItemPower) -> i32 {
    if !ctx.init.gb_is_hellfire && power.type_ == IPL_TARGAC {
        power.param1 = 1 << power.param1;
        power.param2 = 3 << power.param2;
    }
    let mut r = rnd_pl(ctx, power.param1, power.param2);
    use ItemSpecialEffect as E;
    use ItemSpecialEffectHf as H;
    match power.type_ {
        IPL_TOHIT => item._iPLToHit = add16(item._iPLToHit, r),
        IPL_TOHIT_CURSE => item._iPLToHit = add16(item._iPLToHit, -r),
        IPL_DAMP => item._iPLDam = add16(item._iPLDam, r),
        IPL_DAMP_CURSE => item._iPLDam = add16(item._iPLDam, -r),
        IPL_DOPPELGANGER | IPL_TOHIT_DAMP => {
            if power.type_ == IPL_DOPPELGANGER {
                item._iDamAcFlags |= H::Doppelganger;
            }
            r = rnd_pl(ctx, power.param1, power.param2);
            item._iPLDam = add16(item._iPLDam, r);
            let b = calculate_to_hit_bonus(ctx, power.param1);
            item._iPLToHit = add16(item._iPLToHit, b);
        }
        IPL_TOHIT_DAMP_CURSE => {
            item._iPLDam = add16(item._iPLDam, -r);
            let b = calculate_to_hit_bonus(ctx, -power.param1);
            item._iPLToHit = add16(item._iPLToHit, b);
        }
        IPL_ACP => item._iPLAC = add16(item._iPLAC, r),
        IPL_ACP_CURSE => item._iPLAC = add16(item._iPLAC, -r),
        IPL_SETAC => item._iAC = r as i16,
        IPL_AC_CURSE => item._iAC = add16(item._iAC, -r),
        IPL_FIRERES => item._iPLFR = add16(item._iPLFR, r),
        IPL_LIGHTRES => item._iPLLR = add16(item._iPLLR, r),
        IPL_MAGICRES => item._iPLMR = add16(item._iPLMR, r),
        IPL_ALLRES => {
            item._iPLFR = (item._iPLFR as i32 + r).max(0) as i16;
            item._iPLLR = (item._iPLLR as i32 + r).max(0) as i16;
            item._iPLMR = (item._iPLMR as i32 + r).max(0) as i16;
        }
        IPL_SPLLVLADD => item._iSplLvlAdd = r as i8,
        IPL_CHARGES => {
            item._iCharges *= power.param1;
            item._iMaxCharges = item._iCharges;
        }
        IPL_SPELL => {
            item._iSpell = SpellID::from_raw(power.param1 as i8);
            item._iCharges = power.param2;
            item._iMaxCharges = power.param2;
        }
        IPL_FIREDAM => {
            item._iFlags |= E::FireDamage;
            item._iFlags &= !E::LightningDamage;
            item._iFMinDam = power.param1 as i16;
            item._iFMaxDam = power.param2 as i16;
            item._iLMinDam = 0;
            item._iLMaxDam = 0;
        }
        IPL_LIGHTDAM => {
            item._iFlags |= E::LightningDamage;
            item._iFlags &= !E::FireDamage;
            item._iLMinDam = power.param1 as i16;
            item._iLMaxDam = power.param2 as i16;
            item._iFMinDam = 0;
            item._iFMaxDam = 0;
        }
        IPL_STR => item._iPLStr = add16(item._iPLStr, r),
        IPL_STR_CURSE => item._iPLStr = add16(item._iPLStr, -r),
        IPL_MAG => item._iPLMag = add16(item._iPLMag, r),
        IPL_MAG_CURSE => item._iPLMag = add16(item._iPLMag, -r),
        IPL_DEX => item._iPLDex = add16(item._iPLDex, r),
        IPL_DEX_CURSE => item._iPLDex = add16(item._iPLDex, -r),
        IPL_VIT => item._iPLVit = add16(item._iPLVit, r),
        IPL_VIT_CURSE => item._iPLVit = add16(item._iPLVit, -r),
        IPL_ATTRIBS => {
            item._iPLStr = add16(item._iPLStr, r);
            item._iPLMag = add16(item._iPLMag, r);
            item._iPLDex = add16(item._iPLDex, r);
            item._iPLVit = add16(item._iPLVit, r);
        }
        IPL_ATTRIBS_CURSE => {
            item._iPLStr = add16(item._iPLStr, -r);
            item._iPLMag = add16(item._iPLMag, -r);
            item._iPLDex = add16(item._iPLDex, -r);
            item._iPLVit = add16(item._iPLVit, -r);
        }
        IPL_GETHIT_CURSE => item._iPLGetHit = add16(item._iPLGetHit, r),
        IPL_GETHIT => item._iPLGetHit = add16(item._iPLGetHit, -r),
        IPL_LIFE => item._iPLHP = add16(item._iPLHP, r << 6),
        IPL_LIFE_CURSE => item._iPLHP = add16(item._iPLHP, -(r << 6)),
        IPL_MANA => {
            item._iPLMana = add16(item._iPLMana, r << 6);
            redraw_component(ctx, PanelDrawComponent::Mana);
        }
        IPL_MANA_CURSE => {
            item._iPLMana = add16(item._iPLMana, -(r << 6));
            redraw_component(ctx, PanelDrawComponent::Mana);
        }
        IPL_DUR => {
            let bonus = r * item._iMaxDur / 100;
            item._iMaxDur += bonus;
            item._iDurability += bonus;
        }
        IPL_CRYSTALLINE | IPL_DUR_CURSE => {
            if power.type_ == IPL_CRYSTALLINE {
                item._iPLDam = add16(item._iPLDam, 140 + r * 2);
            }
            item._iMaxDur -= r * item._iMaxDur / 100;
            item._iMaxDur = (item._iMaxDur as u8).max(1) as i32;
            item._iDurability = item._iMaxDur;
        }
        IPL_INDESTRUCTIBLE => {
            item._iDurability = DUR_INDESTRUCTIBLE;
            item._iMaxDur = DUR_INDESTRUCTIBLE;
        }
        IPL_LIGHT => item._iPLLight = add16(item._iPLLight, power.param1),
        IPL_LIGHT_CURSE => item._iPLLight = add16(item._iPLLight, -power.param1),
        IPL_MULT_ARROWS => item._iFlags |= E::MultipleArrows,
        IPL_FIRE_ARROWS => {
            item._iFlags |= E::FireArrows;
            item._iFlags &= !E::LightningArrows;
            item._iFMinDam = power.param1 as i16;
            item._iFMaxDam = power.param2 as i16;
            item._iLMinDam = 0;
            item._iLMaxDam = 0;
        }
        IPL_LIGHT_ARROWS => {
            item._iFlags |= E::LightningArrows;
            item._iFlags &= !E::FireArrows;
            item._iLMinDam = power.param1 as i16;
            item._iLMaxDam = power.param2 as i16;
            item._iFMinDam = 0;
            item._iFMaxDam = 0;
        }
        IPL_FIREBALL => {
            item._iFlags |= E::LightningArrows | E::FireArrows;
            item._iFMinDam = power.param1 as i16;
            item._iFMaxDam = power.param2 as i16;
            item._iLMinDam = 0;
            item._iLMaxDam = 0;
        }
        IPL_THORNS => item._iFlags |= E::Thorns,
        IPL_NOMANA => {
            item._iFlags |= E::NoMana;
            redraw_component(ctx, PanelDrawComponent::Mana);
        }
        IPL_ABSHALFTRAP => item._iFlags |= E::HalfTrapDamage,
        IPL_KNOCKBACK => item._iFlags |= E::Knockback,
        IPL_3XDAMVDEM => item._iFlags |= E::TripleDemonDamage,
        IPL_ALLRESZERO => item._iFlags |= E::ZeroResistance,
        IPL_STEALMANA => {
            if power.param1 == 3 {
                item._iFlags |= E::StealMana3;
            }
            if power.param1 == 5 {
                item._iFlags |= E::StealMana5;
            }
            redraw_component(ctx, PanelDrawComponent::Mana);
        }
        IPL_STEALLIFE => {
            if power.param1 == 3 {
                item._iFlags |= E::StealLife3;
            }
            if power.param1 == 5 {
                item._iFlags |= E::StealLife5;
            }
            redraw_component(ctx, PanelDrawComponent::Health);
        }
        IPL_TARGAC => {
            if ctx.init.gb_is_hellfire {
                item._iPLEnAc = power.param1 as i16;
            } else {
                item._iPLEnAc = add16(item._iPLEnAc, r);
            }
        }
        IPL_FASTATTACK => {
            match power.param1 {
                1 => item._iFlags |= E::QuickAttack,
                2 => item._iFlags |= E::FastAttack,
                3 => item._iFlags |= E::FasterAttack,
                4 => item._iFlags |= E::FastestAttack,
                _ => {}
            }
        }
        IPL_FASTRECOVER => {
            match power.param1 {
                1 => item._iFlags |= E::FastHitRecovery,
                2 => item._iFlags |= E::FasterHitRecovery,
                3 => item._iFlags |= E::FastestHitRecovery,
                _ => {}
            }
        }
        IPL_FASTBLOCK => item._iFlags |= E::FastBlock,
        IPL_DAMMOD => item._iPLDamMod = add16(item._iPLDamMod, r),
        IPL_RNDARROWVEL => item._iFlags |= E::RandomArrowVelocity,
        IPL_SETDAM => {
            item._iMinDam = power.param1 as u8;
            item._iMaxDam = power.param2 as u8;
        }
        IPL_SETDUR => {
            item._iDurability = power.param1;
            item._iMaxDur = power.param1;
        }
        IPL_ONEHAND => item._iLoc = ILOC_ONEHAND,
        IPL_DRAINLIFE => item._iFlags |= E::DrainLife,
        IPL_RNDSTEALLIFE => item._iFlags |= E::RandomStealLife,
        IPL_NOMINSTR => item._iMinStr = 0,
        IPL_INVCURS => item._iCurs = power.param1 as u8,
        IPL_ADDACLIFE => {
            item._iFlags |= E::LightningArrows | E::FireArrows;
            item._iFMinDam = power.param1 as i16;
            item._iFMaxDam = power.param2 as i16;
            item._iLMinDam = 1;
            item._iLMaxDam = 0;
        }
        IPL_ADDMANAAC => {
            item._iFlags |= E::LightningDamage | E::FireDamage;
            item._iFMinDam = power.param1 as i16;
            item._iFMaxDam = power.param2 as i16;
            item._iLMinDam = 2;
            item._iLMaxDam = 0;
        }
        IPL_FIRERES_CURSE => item._iPLFR = add16(item._iPLFR, -r),
        IPL_LIGHTRES_CURSE => item._iPLLR = add16(item._iPLLR, -r),
        IPL_MAGICRES_CURSE => item._iPLMR = add16(item._iPLMR, -r),
        IPL_DEVASTATION => item._iDamAcFlags |= H::Devastation,
        IPL_DECAY => {
            item._iDamAcFlags |= H::Decay;
            item._iPLDam = add16(item._iPLDam, r);
        }
        IPL_PERIL => item._iDamAcFlags |= H::Peril,
        IPL_JESTERS => item._iDamAcFlags |= H::Jesters,
        IPL_ACDEMON => item._iDamAcFlags |= H::ACAgainstDemons,
        IPL_ACUNDEAD => item._iDamAcFlags |= H::ACAgainstUndead,
        IPL_MANATOLIFE => {
            let portion = ((player._pMaxManaBase >> 6) * 50 / 100) << 6;
            item._iPLMana = add16(item._iPLMana, -portion);
            item._iPLHP = add16(item._iPLHP, portion);
        }
        IPL_LIFETOMANA => {
            let portion = ((player._pMaxHPBase >> 6) * 40 / 100) << 6;
            item._iPLHP = add16(item._iPLHP, -portion);
            item._iPLMana = add16(item._iPLMana, portion);
        }
        _ => {}
    }
    r
}

/// Original: `StringInPanel` (items.cpp).
// @port items.cpp|devilution::StringInPanel(const char *str) sha=ca35ec0294b4
fn string_in_panel(ctx: &mut Ctx, s: &str) -> bool {
    crate::engine::render::text_render::get_line_width(ctx, s, crate::engine::render::text_render::GameFontTables::GameFont12, 2, None) < 254
}

/// Original: `PLVal` (items.cpp).
// @port items.cpp|devilution::PLVal(int pv, int p1, int p2, int minv, int maxv) sha=c200ef1b7c9e
fn pl_val(pv: i32, p1: i32, p2: i32, minv: i32, maxv: i32) -> i32 {
    if p1 == p2 {
        return minv;
    }
    if minv == maxv {
        return minv;
    }
    minv + (maxv - minv) * (100 * (pv - p1) / (p2 - p1)) / 100
}

/// Original: `SaveItemAffix` (items.cpp).
// @port items.cpp|devilution::SaveItemAffix(const Player &player, Item &item, const PLStruct &affix) sha=e54f8d342e0d
fn save_item_affix(ctx: &mut Ctx, player: &Player, item: &mut Item, affix: &PLStruct) {
    let mut power = affix.power;
    let mut value = save_item_power(ctx, player, item, &mut power);
    value = pl_val(value, power.param1, power.param2, affix.minVal, affix.maxVal);
    if item._iVAdd1 != 0 || item._iVMult1 != 0 {
        item._iVAdd2 = value;
        item._iVMult2 = affix.multVal;
    } else {
        item._iVAdd1 = value;
        item._iVMult1 = affix.multVal;
    }
}

/// Original: `GetStaffPrefixId` (items.cpp).
// @port items.cpp|devilution::GetStaffPrefixId(int lvl, bool onlygood, bool hellfireItem) sha=96088a231de0
fn get_staff_prefix_id(ctx: &mut Ctx, lvl: i32, onlygood: bool, hellfire_item: bool) -> i32 {
    let mut preidx = -1;
    if ctx.rng.flip_coin(10) || onlygood {
        let mut l: Vec<usize> = Vec::with_capacity(256);
        let mut j = 0;
        while ItemPrefixes[j].power.type_ != IPL_INVALID {
            if !is_prefix_valid_for_item_type(j, AffixItemType::Staff, hellfire_item) || ItemPrefixes[j].PLMinLvl as i32 > lvl {
                j += 1;
                continue;
            }
            if onlygood && !ItemPrefixes[j].PLOk {
                j += 1;
                continue;
            }
            l.push(j);
            if ItemPrefixes[j].PLDouble {
                l.push(j);
            }
            j += 1;
        }
        if !l.is_empty() {
            preidx = l[ctx.rng.generate_rnd(l.len() as i32) as usize] as i32;
        }
    }
    preidx
}

/// `fmt::format("{0} of {1}")` and friends: positional substitution.
fn fmt_positional(fmt: &str, args: &[&str]) -> String {
    let mut out = fmt.to_string();
    for (i, a) in args.iter().enumerate() {
        out = out.replace(&format!("{{{i}}}"), a);
    }
    out
}

/// Original: `GenerateStaffName` (items.cpp).
// @port items.cpp|devilution::GenerateStaffName(const ItemData &baseItemData, SpellID spellId, bool translate) sha=b361268c4620
fn generate_staff_name(ctx: &mut Ctx, base: &ItemData, spell_id: SpellID, translate: bool) -> String {
    let base_name = if translate { tr(base.iName) } else { base.iName.to_string() };
    let spell = get_spell_data(spell_id).sNameText;
    let spell_name = if translate { pgettext("spell", spell) } else { spell.to_string() };
    let normal_fmt = if translate { pgettext("spell", "{0} of {1}") } else { "{0} of {1}".to_string() };
    let mut name = fmt_positional(&normal_fmt, &[&base_name, &spell_name]);
    if !string_in_panel(ctx, &name) {
        let s = base.iSName.unwrap_or("");
        let short_name = if translate { tr(s) } else { s.to_string() };
        name = fmt_positional(&normal_fmt, &[&short_name, &spell_name]);
    }
    name
}

/// Original: `GenerateStaffNameMagical` (items.cpp).
// @port items.cpp|devilution::GenerateStaffNameMagical(const ItemData &baseItemData, SpellID spellId, int preidx, bool translate, std::optional<bool> forceNameLengthCheck) sha=7a326de5bc07
fn generate_staff_name_magical(ctx: &mut Ctx, base: &ItemData, spell_id: SpellID, preidx: i32, translate: bool, force_name_length_check: Option<bool>) -> String {
    let base_name = if translate { tr(base.iName) } else { base.iName.to_string() };
    let magic_fmt = if translate { pgettext("spell", "{0} {1} of {2}") } else { "{0} {1} of {2}".to_string() };
    let spell = get_spell_data(spell_id).sNameText;
    let spell_name = if translate { pgettext("spell", spell) } else { spell.to_string() };
    let p = ItemPrefixes[preidx as usize].PLName;
    let prefix_name = if translate { tr(p) } else { p.to_string() };
    let mut identified_name = fmt_positional(&magic_fmt, &[&prefix_name, &base_name, &spell_name]);
    let check = match force_name_length_check {
        Some(v) => v,
        None => !string_in_panel(ctx, &identified_name),
    };
    if check {
        let s = base.iSName.unwrap_or("");
        let short_name = if translate { tr(s) } else { s.to_string() };
        identified_name = fmt_positional(&magic_fmt, &[&prefix_name, &short_name, &spell_name]);
    }
    identified_name
}

/// Original: `GetStaffPower` (items.cpp).
// @port items.cpp|devilution::GetStaffPower(const Player &player, Item &item, int lvl, SpellID bs, bool onlygood) sha=d071538380ca
fn get_staff_power(ctx: &mut Ctx, player: &Player, item: &mut Item, lvl: i32, _bs: SpellID, onlygood: bool) {
    let hf = ctx.init.gb_is_hellfire;
    let preidx = get_staff_prefix_id(ctx, lvl, onlygood, hf);
    if preidx != -1 {
        item._iMagical = ITEM_QUALITY_MAGIC;
        save_item_affix(ctx, player, item, &ItemPrefixes[preidx as usize]);
        item._iPrePower = ItemPrefixes[preidx as usize].power.type_;
    }
    let base = &AllItemsList[item.IDidx as usize];
    let staff_name = generate_staff_name(ctx, base, item._iSpell, false);
    item._iName.set(&staff_name);
    if preidx != -1 {
        let n = generate_staff_name_magical(ctx, base, item._iSpell, preidx, false, None);
        item._iIName.set(&n);
    } else {
        item._iIName = item._iName;
    }
    calc_item_value(item);
}

/// Original: `GenerateMagicItemName` (items.cpp).
// @port items.cpp|devilution::GenerateMagicItemName(const string_view &baseNamel, const PLStruct *pPrefix, const PLStruct *pSufix, bool translate) sha=1c35ca6cf8b3
fn generate_magic_item_name(base_name: &str, prefix: Option<&PLStruct>, suffix: Option<&PLStruct>, translate: bool) -> String {
    let t = |s: &str| if translate { tr(s) } else { s.to_string() };
    match (prefix, suffix) {
        (Some(p), Some(s)) => fmt_positional(&t("{0} {1} of {2}"), &[&t(p.PLName), base_name, &t(s.PLName)]),
        (Some(p), None) => fmt_positional(&t("{0} {1}"), &[&t(p.PLName), base_name]),
        (None, Some(s)) => fmt_positional(&t("{0} of {1}"), &[base_name, &t(s.PLName)]),
        (None, None) => base_name.to_string(),
    }
}

/// Original: `GetItemPowerPrefixAndSuffix` (items.cpp). The callbacks become the returned indices.
// @port items.cpp|devilution::GetItemPowerPrefixAndSuffix(int minlvl, int maxlvl, AffixItemType flgs, bool onlygood, bool hellfireItem, tl::function_ref<void(const PLStruct &prefix)> prefixFound, tl::function_ref<void(const PLStruct &suffix)> suffixFound) sha=db4712086e0a
#[allow(clippy::too_many_arguments)]
fn get_item_power_prefix_and_suffix(
    ctx: &mut Ctx,
    minlvl: i32,
    maxlvl: i32,
    flgs: AffixItemType,
    mut onlygood: bool,
    hellfire_item: bool,
    prefix_found: &mut dyn FnMut(&mut Ctx, usize),
    suffix_found: &mut dyn FnMut(&mut Ctx, usize),
) {
    let mut goe = GOE_ANY;
    let mut allocate_prefix = ctx.rng.flip_coin(4);
    let mut allocate_suffix = !ctx.rng.flip_coin(3);
    if !allocate_prefix && !allocate_suffix {
        if ctx.rng.flip_coin(2) {
            allocate_prefix = true;
        } else {
            allocate_suffix = true;
        }
    }
    if !onlygood && !ctx.rng.flip_coin(3) {
        onlygood = true;
    }
    if allocate_prefix {
        let mut l: Vec<usize> = Vec::with_capacity(256);
        let mut j = 0;
        while ItemPrefixes[j].power.type_ != IPL_INVALID {
            let p = &ItemPrefixes[j];
            let lv = p.PLMinLvl as i32;
            let ok = is_prefix_valid_for_item_type(j, flgs, hellfire_item)
                && !(lv < minlvl || lv > maxlvl)
                && !(onlygood && !p.PLOk)
                && !(flgs.has_any_of(AffixItemType::Staff) && p.power.type_ == IPL_CHARGES);
            if ok {
                l.push(j);
                if p.PLDouble {
                    l.push(j);
                }
            }
            j += 1;
        }
        if !l.is_empty() {
            let preidx = l[ctx.rng.generate_rnd(l.len() as i32) as usize];
            goe = ItemPrefixes[preidx].PLGOE;
            prefix_found(ctx, preidx);
        }
    }
    if allocate_suffix {
        let mut l: Vec<usize> = Vec::with_capacity(256);
        let mut j = 0;
        while ItemSuffixes[j].power.type_ != IPL_INVALID {
            let s = &ItemSuffixes[j];
            if is_suffix_valid_for_item_type(j, flgs, hellfire_item)
                && s.PLMinLvl as i32 >= minlvl
                && s.PLMinLvl as i32 <= maxlvl
                && !((goe == GOE_GOOD && s.PLGOE == GOE_EVIL) || (goe == GOE_EVIL && s.PLGOE == GOE_GOOD))
                && (!onlygood || s.PLOk)
            {
                l.push(j);
            }
            j += 1;
        }
        if !l.is_empty() {
            let sufidx = l[ctx.rng.generate_rnd(l.len() as i32) as usize];
            suffix_found(ctx, sufidx);
        }
    }
}

/// Original: `GetItemPower` (items.cpp).
// @port items.cpp|devilution::GetItemPower(const Player &player, Item &item, int minlvl, int maxlvl, AffixItemType flgs, bool onlygood) sha=d6647d5db533
fn get_item_power(ctx: &mut Ctx, player: &Player, item: &mut Item, minlvl: i32, maxlvl: i32, flgs: AffixItemType, onlygood: bool) {
    let mut p_prefix: Option<usize> = None;
    let mut p_sufix: Option<usize> = None;
    let hf = ctx.init.gb_is_hellfire;
    {
        let item_cell = std::cell::RefCell::new(&mut *item);
        get_item_power_prefix_and_suffix(
            ctx,
            minlvl,
            maxlvl,
            flgs,
            onlygood,
            hf,
            &mut |ctx, prefix| {
                let mut it = item_cell.borrow_mut();
                it._iMagical = ITEM_QUALITY_MAGIC;
                save_item_affix(ctx, player, &mut it, &ItemPrefixes[prefix]);
                it._iPrePower = ItemPrefixes[prefix].power.type_;
                p_prefix = Some(prefix);
            },
            &mut |ctx, suffix| {
                let mut it = item_cell.borrow_mut();
                it._iMagical = ITEM_QUALITY_MAGIC;
                save_item_affix(ctx, player, &mut it, &ItemSuffixes[suffix]);
                it._iSufPower = ItemSuffixes[suffix].power.type_;
                p_sufix = Some(suffix);
            },
        );
    }
    let pre = p_prefix.map(|i| &ItemPrefixes[i]);
    let suf = p_sufix.map(|i| &ItemSuffixes[i]);
    let name = item._iName.as_str().to_string();
    item._iIName.set(&generate_magic_item_name(&name, pre, suf, false));
    let iname = item._iIName.as_str().to_string();
    if !string_in_panel(ctx, &iname) {
        let s = AllItemsList[item.IDidx as usize].iSName.unwrap_or("");
        item._iIName.set(&generate_magic_item_name(s, pre, suf, false));
    }
    if pre.is_some() || suf.is_some() {
        calc_item_value(item);
    }
}

/// Original: `GetStaffSpell` (items.cpp).
// @port items.cpp|devilution::GetStaffSpell(const Player &player, Item &item, int lvl, bool onlygood) sha=adc6119d9e1f
fn get_staff_spell(ctx: &mut Ctx, player: &Player, item: &mut Item, mut lvl: i32, onlygood: bool) {
    if !ctx.init.gb_is_hellfire && ctx.rng.flip_coin(4) {
        get_item_power(ctx, player, item, lvl / 2, lvl, AffixItemType::Staff, onlygood);
        return;
    }
    let max_spells = if ctx.init.gb_is_hellfire { MAX_SPELLS } else { 37 };
    let mut l = lvl / 2;
    if l == 0 {
        l = 1;
    }
    let mut rv = ctx.rng.generate_rnd(max_spells) + 1;
    if ctx.init.gb_is_spawn && lvl > 10 {
        lvl = 10;
    }
    let mut s = SpellID::Firebolt as i32;
    let mut bs = SpellID::Null;
    while rv > 0 {
        let s_level = crate::spells::get_spell_staff_level(ctx, SpellID::from_raw(s as i8));
        if s_level != -1 && l >= s_level {
            rv -= 1;
            bs = SpellID::from_raw(s as i8);
        }
        s += 1;
        if !ctx.init.gb_is_multiplayer && s == SpellID::Resurrect as i32 {
            s = SpellID::Telekinesis as i32;
        }
        if !ctx.init.gb_is_multiplayer && s == SpellID::HealOther as i32 {
            s = SpellID::BloodStar as i32;
        }
        if s == max_spells {
            s = SpellID::Firebolt as i32;
        }
    }
    let sd = get_spell_data(bs);
    let minc = sd.sStaffMin as i32;
    let maxc = sd.sStaffMax as i32 - minc + 1;
    item._iSpell = bs;
    item._iCharges = minc + ctx.rng.generate_rnd(maxc);
    item._iMaxCharges = item._iCharges;
    item._iMinMag = sd.minInt;
    let v = item._iCharges * (sd.staffCost10 as i32 * 10) / 5;
    item._ivalue += v;
    item._iIvalue += v;
    get_staff_power(ctx, player, item, lvl, bs, onlygood);
}

/// Original: `GetOilType` (items.cpp).
// @port items.cpp|devilution::GetOilType(Item &item, int maxLvl) sha=9ccd4ae06127
fn get_oil_type(ctx: &mut Ctx, item: &mut Item, mut max_lvl: i32) {
    let mut cnt = 2;
    let mut rnd = [0i8; 32];
    rnd[0] = 5;
    rnd[1] = 6;
    if !ctx.init.gb_is_multiplayer {
        if max_lvl == 0 {
            max_lvl = 1;
        }
        cnt = 0;
        for (j, &lv) in OilLevels.iter().enumerate() {
            if lv <= max_lvl {
                rnd[cnt] = j as i8;
                cnt += 1;
            }
        }
    }
    let t = rnd[ctx.rng.generate_rnd(cnt as i32) as usize] as usize;
    item._iName.set(OilNames[t]);
    item._iIName.set(OilNames[t]);
    item._iMiscId = OilMagic[t];
    item._ivalue = OilValues[t];
    item._iIvalue = OilValues[t];
}

/// Original: `GetItemBonus` (items.cpp).
// @port items.cpp|devilution::GetItemBonus(const Player &player, Item &item, int minlvl, int maxlvl, bool onlygood, bool allowspells) sha=7a5803795072
fn get_item_bonus(ctx: &mut Ctx, player: &Player, item: &mut Item, mut minlvl: i32, maxlvl: i32, onlygood: bool, allowspells: bool) {
    if minlvl > 25 {
        minlvl = 25;
    }
    match item._itype {
        ItemType::Sword | ItemType::Axe | ItemType::Mace => get_item_power(ctx, player, item, minlvl, maxlvl, AffixItemType::Weapon, onlygood),
        ItemType::Bow => get_item_power(ctx, player, item, minlvl, maxlvl, AffixItemType::Bow, onlygood),
        ItemType::Shield => get_item_power(ctx, player, item, minlvl, maxlvl, AffixItemType::Shield, onlygood),
        ItemType::LightArmor | ItemType::Helm | ItemType::MediumArmor | ItemType::HeavyArmor => {
            get_item_power(ctx, player, item, minlvl, maxlvl, AffixItemType::Armor, onlygood)
        }
        ItemType::Staff => {
            if allowspells {
                get_staff_spell(ctx, player, item, maxlvl, onlygood);
            } else {
                get_item_power(ctx, player, item, minlvl, maxlvl, AffixItemType::Staff, onlygood);
            }
        }
        ItemType::Ring | ItemType::Amulet => get_item_power(ctx, player, item, minlvl, maxlvl, AffixItemType::Misc, onlygood),
        ItemType::None | ItemType::Misc | ItemType::Gold => {}
    }
}

/// Original: `GetItemIndexForDroppableItem` (items.cpp).
// @port items.cpp|devilution::GetItemIndexForDroppableItem(bool considerDropRate, tl::function_ref<bool(const ItemData &item)> isItemOkay) sha=3ac49943e638
fn get_item_index_for_droppable_item(ctx: &mut Ctx, consider_drop_rate: bool, is_item_okay: &dyn Fn(&Ctx, &ItemData) -> bool) -> _item_indexes {
    let mut ril: Vec<_item_indexes> = Vec::with_capacity(IDI_LAST as usize * 2);
    for i in IDI_GOLD..=IDI_LAST {
        if !is_item_available(ctx, i as i32) {
            continue;
        }
        let item = &AllItemsList[i as usize];
        if item.iRnd == IDROP_NEVER {
            continue;
        }
        if matches!(item.iSpell, SpellID::Resurrect | SpellID::HealOther) && !ctx.init.gb_is_multiplayer {
            continue;
        }
        if !is_item_okay(ctx, item) {
            continue;
        }
        ril.push(i);
        if item.iRnd == IDROP_DOUBLE && consider_drop_rate {
            ril.push(i);
        }
    }
    // An empty list indexes the zero-initialised static array in the original.
    let r = ctx.rng.generate_rnd(ril.len() as i32) as usize;
    ril.get(r).copied().unwrap_or(0)
}

/// Original: `RndUItem` (items.cpp).
// @port items.cpp|devilution::RndUItem(Monster *monster) sha=8f5408adb8e3
fn rnd_u_item(ctx: &mut Ctx, monster: Option<usize>) -> _item_indexes {
    let mut item_max_level = items_get_currlevel(ctx) * 2;
    if let Some(m) = monster {
        item_max_level = crate::monster::monster_level(ctx, m, ctx.multi.sgGameInitInfo.nDifficulty) as i32;
    }
    get_item_index_for_droppable_item(ctx, false, &|_, item| {
        if item.itype == ItemType::Misc && item.iMiscId == IMISC_BOOK {
            return true;
        }
        if item_max_level < item.iMinMLvl as i32 {
            return false;
        }
        if matches!(item.itype, ItemType::Gold | ItemType::Misc) {
            return false;
        }
        true
    })
}

/// Original: `RndAllItems` (items.cpp).
// @port items.cpp|devilution::RndAllItems() sha=1b0d003edaa8
fn rnd_all_items(ctx: &mut Ctx) -> _item_indexes {
    if ctx.rng.generate_rnd(100) > 25 {
        return IDI_GOLD;
    }
    let item_max_level = items_get_currlevel(ctx) * 2;
    get_item_index_for_droppable_item(ctx, false, &|_, item| item_max_level >= item.iMinMLvl as i32)
}

/// Original: `RndTypeItems` (items.cpp).
// @port items.cpp|devilution::RndTypeItems(ItemType itemType, int imid, int lvl) sha=5ad2c00e8f32
fn rnd_type_items(ctx: &mut Ctx, item_type: ItemType, imid: i32, lvl: i32) -> _item_indexes {
    let item_max_level = lvl * 2;
    get_item_index_for_droppable_item(ctx, false, &|_, item| {
        if item_max_level < item.iMinMLvl as i32 {
            return false;
        }
        if item.itype != item_type {
            return false;
        }
        if imid != -1 && item.iMiscId as i32 != imid {
            return false;
        }
        true
    })
}

/// Original: `CheckUnique` (items.cpp).
// @port items.cpp|devilution::CheckUnique(Item &item, int lvl, int uper, bool recreate) sha=d3f880ff4d22
fn check_unique(ctx: &mut Ctx, item: &Item, lvl: i32, uper: i32, recreate: bool) -> _unique_items {
    let mut uok = [false; 128];
    if ctx.rng.generate_rnd(100) > uper {
        return UITEM_INVALID;
    }
    let mut numu = 0;
    let mut j = 0;
    while UniqueItems[j].UIItemId != UITYPE_INVALID {
        if !is_unique_available(ctx, j as i32) {
            break;
        }
        if UniqueItems[j].UIItemId == AllItemsList[item.IDidx as usize].iItemId
            && lvl >= UniqueItems[j].UIMinLvl as i32
            && (recreate || !ctx.items.UniqueItemFlags[j] || ctx.init.gb_is_multiplayer)
        {
            uok[j] = true;
            numu += 1;
        }
        j += 1;
    }
    if numu == 0 {
        return UITEM_INVALID;
    }
    ctx.rng.discard_random_values(1);
    let mut item_data: u8 = 0;
    while numu > 0 {
        if uok[item_data as usize] {
            numu -= 1;
        }
        if numu > 0 {
            item_data = (item_data + 1) % 128;
        }
    }
    item_data as _unique_items
}

/// Original: `GetUniqueItem` (items.cpp).
// @port items.cpp|devilution::GetUniqueItem(const Player &player, Item &item, _unique_items uid) sha=860b8effacc9
fn get_unique_item(ctx: &mut Ctx, player: &Player, item: &mut Item, uid: _unique_items) {
    ctx.items.UniqueItemFlags[uid as usize] = true;
    for power in UniqueItems[uid as usize].powers {
        if power.type_ == IPL_INVALID {
            break;
        }
        let mut p = power;
        save_item_power(ctx, player, item, &mut p);
    }
    item._iIName.set(UniqueItems[uid as usize].UIName);
    item._iIvalue = UniqueItems[uid as usize].UIValue;
    if item._iMiscId == IMISC_UNIQUE {
        item._iSeed = uid as u32;
    }
    item._iUid = uid as i32;
    item._iMagical = ITEM_QUALITY_UNIQUE;
    item._iCreateInfo |= CF_UNIQUE as u16;
}

/// Original: `ItemRndDur` (items.cpp).
// @port items.cpp|devilution::ItemRndDur(Item &item) sha=1385ba2fb2ad
fn item_rnd_dur(ctx: &mut Ctx, item: &mut Item) {
    if item._iDurability > 0 && item._iDurability != DUR_INDESTRUCTIBLE {
        item._iDurability = ctx.rng.generate_rnd(item._iMaxDur / 2) + (item._iMaxDur / 4) + 1;
    }
}

/// Original: `GetItemBLevel` (items.cpp).
// @port items.cpp|devilution::GetItemBLevel(int lvl, item_misc_id miscId, bool onlygood, bool uper15) sha=4bb89bdad1f1
fn get_item_b_level(ctx: &mut Ctx, lvl: i32, misc_id: item_misc_id, onlygood: bool, uper15: bool) -> i32 {
    let mut iblvl = -1;
    if ctx.rng.generate_rnd(100) <= 10 || ctx.rng.generate_rnd(100) <= lvl || onlygood || matches!(misc_id, IMISC_STAFF | IMISC_RING | IMISC_AMULET) {
        iblvl = lvl;
    }
    if uper15 {
        iblvl = lvl + 4;
    }
    iblvl
}

/// Original: `SetupAllItems` (items.cpp).
// @port items.cpp|devilution::SetupAllItems(const Player &player, Item &item, _item_indexes idx, uint32_t iseed, int lvl, int uper, bool onlygood, bool recreate, bool pregen) sha=07a209bb018f
#[allow(clippy::too_many_arguments)]
fn setup_all_items(ctx: &mut Ctx, player: &Player, item: &mut Item, idx: _item_indexes, iseed: u32, lvl: i32, uper: i32, onlygood: bool, recreate: bool, pregen: bool) {
    item._iSeed = iseed;
    ctx.rng.set_rnd_seed(iseed);
    get_item_attrs(ctx, item, idx, lvl / 2);
    item._iCreateInfo = lvl as u16;
    if pregen {
        item._iCreateInfo |= CF_PREGEN as u16;
    }
    if onlygood {
        item._iCreateInfo |= CF_ONLYGOOD as u16;
    }
    if uper == 15 {
        item._iCreateInfo |= CF_UPER15 as u16;
    } else if uper == 1 {
        item._iCreateInfo |= CF_UPER1 as u16;
    }
    if item._iMiscId != IMISC_UNIQUE {
        let iblvl = get_item_b_level(ctx, lvl, item._iMiscId, onlygood, uper == 15);
        if iblvl != -1 {
            let uid = check_unique(ctx, item, iblvl, uper, recreate);
            if uid == UITEM_INVALID {
                get_item_bonus(ctx, player, item, iblvl / 2, iblvl, onlygood, true);
            } else {
                get_unique_item(ctx, player, item, uid);
            }
        }
        if item._iMagical != ITEM_QUALITY_UNIQUE {
            item_rnd_dur(ctx, item);
        }
    } else if item._iLoc != ILOC_UNEQUIPABLE {
        if iseed > 109 || AllItemsList[idx as usize].iItemId != UniqueItems[iseed as usize].UIItemId {
            item.clear();
            return;
        }
        get_unique_item(ctx, player, item, iseed as _unique_items); // uid is stored in iseed for uniques
    }
    setup_item(ctx, item);
}

/// Original: `SetupBaseItem` (items.cpp). `spawn` defaults to false.
// @port items.cpp|devilution::SetupBaseItem(Point position, _item_indexes idx, bool onlygood, bool sendmsg, bool delta, bool spawn = false) sha=66a6f68c57ba
fn setup_base_item(ctx: &mut Ctx, position: Point, idx: _item_indexes, onlygood: bool, sendmsg: bool, delta: bool, spawn: bool) {
    if ctx.items.ActiveItemCount as usize >= MAXITEMS {
        return;
    }
    let ii = allocate_item(ctx) as usize;
    get_super_item_space(ctx, position, ii as i8);
    let curlv = items_get_currlevel(ctx);
    let me = my_player_copy(ctx);
    let seed = ctx.rng.advance_rnd_seed() as u32;
    with_item(ctx, ii, |ctx, item| setup_all_items(ctx, &me, item, idx, seed, 2 * curlv, 1, onlygood, false, delta));
    let pos = ctx.items.Items[ii].position;
    if sendmsg {
        { let it = ctx.items.Items[ii as usize].clone(); crate::msg::net_send_cmd_p_item(ctx, false, CMD_DROPITEM, pos, &it); }
    }
    if delta {
        crate::msg::delta_add_item(ctx, ii as i32);
    }
    if spawn {
        { let it = ctx.items.Items[ii as usize].clone(); crate::msg::net_send_cmd_p_item(ctx, false, CMD_SPAWNITEM, pos, &it); }
    }
}

/// Original: `SetupAllUseful` (items.cpp).
// @port items.cpp|devilution::SetupAllUseful(Item &item, int iseed, int lvl) sha=395640714f32
fn setup_all_useful(ctx: &mut Ctx, item: &mut Item, iseed: i32, lvl: i32) {
    item._iSeed = iseed as u32;
    ctx.rng.set_rnd_seed(iseed as u32);
    let idx;
    if ctx.init.gb_is_hellfire {
        idx = match ctx.rng.generate_rnd(7) {
            0 => {
                if lvl <= 1 {
                    IDI_HEAL
                } else {
                    IDI_PORTAL
                }
            }
            1 | 2 => IDI_HEAL,
            3 => {
                if lvl <= 1 {
                    IDI_MANA
                } else {
                    IDI_PORTAL
                }
            }
            4 | 5 => IDI_MANA,
            _ => IDI_OIL,
        };
    } else {
        let mut i = ctx.rng.pick_randomly_among(&[IDI_MANA, IDI_HEAL]);
        if lvl > 1 && ctx.rng.flip_coin(3) {
            i = IDI_PORTAL;
        }
        idx = i;
    }
    get_item_attrs(ctx, item, idx, lvl);
    item._iCreateInfo = (lvl | CF_USEFUL) as u16;
    setup_item(ctx, item);
}

/// Original: `Char2int` (items.cpp).
// @port items.cpp|devilution::Char2int(uint8_t input) sha=3337a2d173b3
fn char2int(input: u8) -> u8 {
    match input {
        b'0'..=b'9' => input - b'0',
        b'A'..=b'F' => input - b'A' + 10,
        _ => 0,
    }
}

/// Original: `Hex2bin` (items.cpp).
// @port items.cpp|devilution::Hex2bin(const char *src, int bytes, uint8_t *target) sha=e3ea77849186
fn hex2bin(src: &[u8], bytes: usize, target: &mut [u8]) {
    for i in 0..bytes {
        let a = src.get(i * 2).copied().unwrap_or(0);
        let b = src.get(i * 2 + 1).copied().unwrap_or(0);
        target[i] = (char2int(a) << 4) | char2int(b);
    }
}

/// Original: `SpawnRock` (items.cpp).
// @port items.cpp|devilution::SpawnRock() sha=110c0df70278
fn spawn_rock(ctx: &mut Ctx) {
    if ctx.items.ActiveItemCount as usize >= MAXITEMS {
        return;
    }
    let Some(stand_pos) = crate::objects::find_stand_position(ctx) else { return };
    let ii = allocate_item(ctx) as usize;
    ctx.items.Items[ii].position = stand_pos;
    ctx.items.dItem[stand_pos.x as usize][stand_pos.y as usize] = (ii + 1) as i8;
    let curlv = items_get_currlevel(ctx);
    with_item(ctx, ii, |ctx, item| {
        get_item_attrs(ctx, item, IDI_ROCK, curlv);
        setup_item(ctx, item);
        item._iSelFlag = 2;
        item._iPostDraw = true;
        item.AnimInfo.currentFrame = 10;
        item._iCreateInfo |= CF_PREGEN as u16;
    });
    crate::msg::delta_add_item(ctx, ii as i32);
}

/// Original: `ItemDoppel` (items.cpp).
// @port items.cpp|devilution::ItemDoppel() sha=5aaebd08589b
fn item_doppel(ctx: &mut Ctx) {
    if !ctx.init.gb_is_multiplayer {
        return;
    }
    let idoppely = ctx.items.idoppely;
    for idoppelx in 16..96 {
        let d = ctx.items.dItem[idoppelx][idoppely as usize];
        if d != 0 {
            let i = &ctx.items.Items[(d - 1) as usize];
            if i.position.x != idoppelx as i32 || i.position.y != idoppely {
                ctx.items.dItem[idoppelx][idoppely as usize] = 0;
            }
        }
    }
    ctx.items.idoppely += 1;
    if ctx.items.idoppely == 96 {
        ctx.items.idoppely = 16;
    }
}

/// Original: `PrintItemOil` (items.cpp).
// @port items.cpp|devilution::PrintItemOil(char iDidx) sha=e331ec95b27a
fn print_item_oil(ctx: &mut Ctx, i_didx: item_misc_id) {
    let add = |ctx: &mut Ctx, s: &str| crate::control::add_panel_string(ctx, &tr(s));
    match i_didx {
        IMISC_OILACC => {
            add(ctx, "increases a weapon's");
            add(ctx, "chance to hit");
        }
        IMISC_OILMAST => {
            add(ctx, "greatly increases a");
            add(ctx, "weapon's chance to hit");
        }
        IMISC_OILSHARP => {
            add(ctx, "increases a weapon's");
            add(ctx, "damage potential");
        }
        IMISC_OILDEATH => {
            add(ctx, "greatly increases a weapon's");
            add(ctx, "damage potential - not bows");
        }
        IMISC_OILSKILL => {
            add(ctx, "reduces attributes needed");
            add(ctx, "to use armor or weapons");
        }
        IMISC_OILBSMTH => {
            add(ctx, "restores 20% of an");
            add(ctx, "item's durability");
        }
        IMISC_OILFORT => {
            add(ctx, "increases an item's");
            add(ctx, "current and max durability");
        }
        IMISC_OILPERM => add(ctx, "makes an item indestructible"),
        IMISC_OILHARD => {
            add(ctx, "increases the armor class");
            add(ctx, "of armor and shields");
        }
        IMISC_OILIMP => {
            add(ctx, "greatly increases the armor");
            add(ctx, "class of armor and shields");
        }
        IMISC_RUNEF => add(ctx, "sets fire trap"),
        IMISC_RUNEL | IMISC_GR_RUNEL => add(ctx, "sets lightning trap"),
        IMISC_GR_RUNEF => add(ctx, "sets fire trap"),
        IMISC_RUNES => add(ctx, "sets petrification trap"),
        IMISC_FULLHEAL => add(ctx, "restore all life"),
        IMISC_HEAL => add(ctx, "restore some life"),
        IMISC_MANA => add(ctx, "restore some mana"),
        IMISC_FULLMANA => add(ctx, "restore all mana"),
        IMISC_ELIXSTR => add(ctx, "increase strength"),
        IMISC_ELIXMAG => add(ctx, "increase magic"),
        IMISC_ELIXDEX => add(ctx, "increase dexterity"),
        IMISC_ELIXVIT => add(ctx, "increase vitality"),
        IMISC_REJUV => add(ctx, "restore some life and mana"),
        IMISC_FULLREJUV => add(ctx, "restore all life and mana"),
        IMISC_ARENAPOT => {
            add(ctx, "restore all life and mana");
            add(ctx, "(works only in arenas)");
        }
        _ => {}
    }
}

/// Original: `devilution::DrawUniqueInfoWindow` (items.cpp).
// @port items.cpp|devilution::DrawUniqueInfoWindow(const Surface &out) sha=43c3a340be4e
fn draw_unique_info_window(ctx: &mut Ctx, out: &crate::engine::surface::Surface) {
    let (spw, _) = crate::control::SIDE_PANEL_SIZE;
    let p = crate::control::get_panel_position(ctx, UiPanels::Inventory, Point::new(24 - spw, 327));
    let sprite = ctx.info_box.p_s_text_box_cels.as_ref().expect("pSTextBoxCels").get(0);
    crate::engine::render::clx_render::clx_draw(out, (p.x, p.y), &sprite);
    let right = crate::control::get_right_panel(ctx);
    crate::engine::draw_half_transparent_rect_to(ctx, out, right.x - spw + 27, right.y + 28, 265, 297);
}

/// Original: `printItemMiscKBM` (items.cpp).
// @port items.cpp|devilution::printItemMiscKBM(const Item &item, const bool isOil, const bool isCastOnTarget) sha=b75e75f66aa6
fn print_item_misc_kbm(ctx: &mut Ctx, item: &Item, is_oil: bool, is_cast_on_target: bool) {
    if item._iMiscId == IMISC_MAPOFDOOM {
        crate::control::add_panel_string(ctx, &tr("Right-click to view"));
    } else if is_oil {
        print_item_oil(ctx, item._iMiscId);
        crate::control::add_panel_string(ctx, &tr("Right-click to use"));
    } else if is_cast_on_target {
        crate::control::add_panel_string(ctx, &tr("Right-click to read, then\nleft-click to target"));
    } else if matches!(item._iMiscId, IMISC_BOOK | IMISC_NOTE | IMISC_SCROLL | IMISC_SCROLLT) {
        crate::control::add_panel_string(ctx, &tr("Right-click to read"));
    }
}

/// Original: `printItemMiscGenericGamepad` (items.cpp).
// @port items.cpp|devilution::printItemMiscGenericGamepad(const Item &item, const bool isOil, bool isCastOnTarget) sha=84aaed01f970
fn print_item_misc_generic_gamepad(ctx: &mut Ctx, item: &Item, is_oil: bool, is_cast_on_target: bool) {
    if item._iMiscId == IMISC_MAPOFDOOM {
        crate::control::add_panel_string(ctx, &tr("Activate to view"));
    } else if is_oil {
        print_item_oil(ctx, item._iMiscId);
        if !ctx.inv.invflag {
            crate::control::add_panel_string(ctx, &tr("Open inventory to use"));
        } else {
            crate::control::add_panel_string(ctx, &tr("Activate to use"));
        }
    } else if is_cast_on_target {
        crate::control::add_panel_string(ctx, &tr("Select from spell book, then\ncast spell to read"));
    } else if matches!(item._iMiscId, IMISC_BOOK | IMISC_NOTE | IMISC_SCROLL | IMISC_SCROLLT) {
        crate::control::add_panel_string(ctx, &tr("Activate to read"));
    }
}

/// Original: `printItemMiscGamepad` (items.cpp).
// @port items.cpp|devilution::printItemMiscGamepad(const Item &item, bool isOil, bool isCastOnTarget) sha=81fe3e7eaf0e
fn print_item_misc_gamepad(ctx: &mut Ctx, item: &Item, is_oil: bool, is_cast_on_target: bool) {
    use crate::controls::game_controls::GamepadLayout;
    let (activate_button, cast_button) = match ctx.controls.gamepad_type {
        GamepadLayout::Generic => {
            print_item_misc_generic_gamepad(ctx, item, is_oil, is_cast_on_target);
            return;
        }
        GamepadLayout::Xbox => (crate::controls::controller_buttons::XBOX_Y, crate::controls::controller_buttons::XBOX_X),
        GamepadLayout::PlayStation => (crate::controls::controller_buttons::PLAYSTATION_TRIANGLE, crate::controls::controller_buttons::PLAYSTATION_SQUARE),
        GamepadLayout::Nintendo => (crate::controls::controller_buttons::NINTENDO_X, crate::controls::controller_buttons::NINTENDO_Y),
    };
    let sub = |f: String, b: &str| f.replacen("{}", b, 1);
    if item._iMiscId == IMISC_MAPOFDOOM {
        crate::control::add_panel_string(ctx, &sub(tr("{} to view"), activate_button));
    } else if is_oil {
        print_item_oil(ctx, item._iMiscId);
        if !ctx.inv.invflag {
            crate::control::add_panel_string(ctx, &tr("Open inventory to use"));
        } else {
            crate::control::add_panel_string(ctx, &sub(tr("{} to use"), activate_button));
        }
    } else if is_cast_on_target {
        crate::control::add_panel_string(ctx, &sub(tr("Select from spell book,\nthen {} to read"), cast_button));
    } else if matches!(item._iMiscId, IMISC_BOOK | IMISC_NOTE | IMISC_SCROLL | IMISC_SCROLLT) {
        crate::control::add_panel_string(ctx, &sub(tr("{} to read"), activate_button));
    }
}

/// Original: `PrintItemMisc` (items.cpp).
// @port items.cpp|devilution::PrintItemMisc(const Item &item) sha=ea54e4d7938a
fn print_item_misc(ctx: &mut Ctx, item: &Item) {
    if item._iMiscId == IMISC_EAR {
        crate::control::add_panel_string(ctx, &pgettext("player", "Level: {:d}").replacen("{:d}", &item._ivalue.to_string(), 1));
        return;
    }
    if item._iMiscId == IMISC_AURIC {
        crate::control::add_panel_string(ctx, &tr("Doubles gold capacity"));
        return;
    }
    let m = item._iMiscId;
    let is_oil = (m >= IMISC_USEFIRST && m <= IMISC_USELAST)
        || (m > IMISC_OILFIRST && m < IMISC_OILLAST)
        || (m > IMISC_RUNEFIRST && m < IMISC_RUNELAST)
        || m == IMISC_ARENAPOT;
    let is_cast_on_target =
        (m == IMISC_SCROLLT && item._iSpell != SpellID::Flash) || (m == IMISC_SCROLL && matches!(item._iSpell, SpellID::TownPortal | SpellID::Identify));
    use crate::controls::ControlTypes;
    match ctx.controls.control_mode {
        ControlTypes::None => {}
        ControlTypes::KeyboardAndMouse => print_item_misc_kbm(ctx, item, is_oil, is_cast_on_target),
        ControlTypes::VirtualGamepad => print_item_misc_generic_gamepad(ctx, item, is_oil, is_cast_on_target),
        ControlTypes::Gamepad => print_item_misc_gamepad(ctx, item, is_oil, is_cast_on_target),
    }
}

/// Original: `PrintItemInfo` (items.cpp).
// @port items.cpp|devilution::PrintItemInfo(const Item &item) sha=03aa48fee6fc
fn print_item_info(ctx: &mut Ctx, item: &Item) {
    print_item_misc(ctx, item);
    let str_ = item._iMinStr as u8;
    let dex = item._iMinDex as u8;
    let mag = item._iMinMag;
    if str_ != 0 || mag != 0 || dex != 0 {
        let mut text = tr("Required:");
        if str_ != 0 {
            text.push_str(&tr(" {:d} Str").replacen("{:d}", &str_.to_string(), 1));
        }
        if mag != 0 {
            text.push_str(&tr(" {:d} Mag").replacen("{:d}", &mag.to_string(), 1));
        }
        if dex != 0 {
            text.push_str(&tr(" {:d} Dex").replacen("{:d}", &dex.to_string(), 1));
        }
        crate::control::add_panel_string(ctx, &text);
    }
}

/// Original: `SmithItemOk` (items.cpp).
// @port items.cpp|devilution::SmithItemOk(const Player &player, const ItemData &item) sha=9220197eb06f
fn smith_item_ok(ctx: &Ctx, _player: &Player, item: &ItemData) -> bool {
    if item.itype == ItemType::Misc || item.itype == ItemType::Gold {
        return false;
    }
    if item.itype == ItemType::Staff && (!ctx.init.gb_is_hellfire || crate::spells::is_valid_spell(ctx, item.iSpell)) {
        return false;
    }
    if item.itype == ItemType::Ring || item.itype == ItemType::Amulet {
        return false;
    }
    true
}

type VendorOk = fn(&Ctx, &Player, &ItemData) -> bool;

/// Original: `RndVendorItem` (items.cpp).
// @port items.cpp|devilution::RndVendorItem(const Player &player, int minlvl, int maxlvl) sha=c2b3eb89d0a3
fn rnd_vendor_item(ctx: &mut Ctx, ok: VendorOk, consider_drop_rate: bool, player: &Player, minlvl: i32, maxlvl: i32) -> _item_indexes {
    get_item_index_for_droppable_item(ctx, consider_drop_rate, &|ctx, item| {
        if !ok(ctx, player, item) {
            return false;
        }
        if (item.iMinMLvl as i32) < minlvl || item.iMinMLvl as i32 > maxlvl {
            return false;
        }
        true
    })
}

/// Original: `RndSmithItem` (items.cpp).
// @port items.cpp|devilution::RndSmithItem(const Player &player, int lvl) sha=954363c6410d
fn rnd_smith_item(ctx: &mut Ctx, player: &Player, lvl: i32) -> _item_indexes {
    rnd_vendor_item(ctx, smith_item_ok, true, player, 0, lvl)
}

/// Original: `SortVendor` (items.cpp): `std::sort` by `IDidx` (libstdc++ ordering for ties).
// @port items.cpp|devilution::SortVendor(Item *itemList) sha=8be6c80fd60d
fn sort_vendor(item_list: &mut [Item]) {
    let mut count = 1;
    while count < item_list.len() && !item_list[count].is_empty() {
        count += 1;
    }
    crate::utils::stdsort::sort_by(&mut item_list[..count], |a, b| a.IDidx < b.IDidx);
}

/// Original: `PremiumItemOk` (items.cpp).
// @port items.cpp|devilution::PremiumItemOk(const Player &player, const ItemData &item) sha=bfe04e634ef7
fn premium_item_ok(ctx: &Ctx, _player: &Player, item: &ItemData) -> bool {
    if item.itype == ItemType::Misc || item.itype == ItemType::Gold {
        return false;
    }
    if !ctx.init.gb_is_hellfire && item.itype == ItemType::Staff {
        return false;
    }
    if ctx.init.gb_is_multiplayer {
        if item.iMiscId == IMISC_OILOF {
            return false;
        }
        if item.itype == ItemType::Ring || item.itype == ItemType::Amulet {
            return false;
        }
    }
    true
}

/// Original: `RndPremiumItem` (items.cpp).
// @port items.cpp|devilution::RndPremiumItem(const Player &player, int minlvl, int maxlvl) sha=d4deb0e29ab0
fn rnd_premium_item(ctx: &mut Ctx, player: &Player, minlvl: i32, maxlvl: i32) -> _item_indexes {
    rnd_vendor_item(ctx, premium_item_ok, false, player, minlvl, maxlvl)
}

/// `GetMostValuableItem` for an item type filter.
fn most_valuable_player_item_value(player: &Player, pred: &dyn Fn(&Item) -> bool) -> i32 {
    match player.get_most_valuable_item(pred) {
        None => 0,
        Some(item) => item._iIvalue,
    }
}

/// Original: `SpawnOnePremium` (items.cpp).
// @port items.cpp|devilution::SpawnOnePremium(Item &premiumItem, int plvl, const Player &player) sha=c1da2b3a3433
fn spawn_one_premium(ctx: &mut Ctx, premium_item: &mut Item, mut plvl: i32, player: &Player) {
    let mut strength = player.get_maximum_attribute_value(CharacterAttribute::Strength).max(player._pStrength);
    let mut dexterity = player.get_maximum_attribute_value(CharacterAttribute::Dexterity).max(player._pDexterity);
    let mut magic = player.get_maximum_attribute_value(CharacterAttribute::Magic).max(player._pMagic);
    strength += strength / 5;
    dexterity += dexterity / 5;
    magic += magic / 5;
    plvl = plvl.clamp(1, 30);
    let max_count = 150;
    let unlimited = !ctx.init.gb_is_hellfire;
    let mut count = 0;
    while unlimited || count < max_count {
        *premium_item = Item::default();
        premium_item._iSeed = ctx.rng.advance_rnd_seed() as u32;
        ctx.rng.set_rnd_seed(premium_item._iSeed);
        let item_type = rnd_premium_item(ctx, player, plvl / 4, plvl);
        get_item_attrs(ctx, premium_item, item_type, plvl);
        let hf = ctx.init.gb_is_hellfire;
        get_item_bonus(ctx, player, premium_item, plvl / 2, plvl, true, !hf);
        if !hf {
            if premium_item._iIvalue <= 140000 {
                break;
            }
        } else {
            let mut item_value = match premium_item._itype {
                ItemType::LightArmor | ItemType::MediumArmor | ItemType::HeavyArmor => most_valuable_player_item_value(player, &|item| {
                    matches!(item._itype, ItemType::LightArmor | ItemType::MediumArmor | ItemType::HeavyArmor)
                }),
                ItemType::Shield
                | ItemType::Axe
                | ItemType::Bow
                | ItemType::Mace
                | ItemType::Sword
                | ItemType::Helm
                | ItemType::Staff
                | ItemType::Ring
                | ItemType::Amulet => {
                    let filter = premium_item._itype;
                    most_valuable_player_item_value(player, &|item| item._itype == filter)
                }
                _ => 0,
            };
            item_value = item_value * 4 / 5;
            if premium_item._iIvalue <= 200000
                && premium_item._iMinStr as i32 <= strength
                && premium_item._iMinMag as i32 <= magic
                && premium_item._iMinDex as i32 <= dexterity
                && premium_item._iIvalue >= item_value
            {
                break;
            }
        }
        count += 1;
    }
    premium_item._iCreateInfo = (plvl | CF_SMITHPREMIUM) as u16;
    premium_item._iIdentified = true;
    premium_item._iStatFlag = player.can_use_item(premium_item);
}

/// Original: `WitchItemOk` (items.cpp).
// @port items.cpp|devilution::WitchItemOk(const Player &player, const ItemData &item) sha=7d8e0fae19e6
fn witch_item_ok(ctx: &Ctx, _player: &Player, item: &ItemData) -> bool {
    if !matches!(item.itype, ItemType::Misc | ItemType::Staff) {
        return false;
    }
    if item.iMiscId == IMISC_MANA || item.iMiscId == IMISC_FULLMANA {
        return false;
    }
    if item.iSpell == SpellID::TownPortal {
        return false;
    }
    if item.iMiscId == IMISC_FULLHEAL || item.iMiscId == IMISC_HEAL {
        return false;
    }
    if item.iMiscId > IMISC_OILFIRST && item.iMiscId < IMISC_OILLAST {
        return false;
    }
    if item.iSpell == SpellID::Resurrect && !ctx.init.gb_is_multiplayer {
        return false;
    }
    if item.iSpell == SpellID::HealOther && !ctx.init.gb_is_multiplayer {
        return false;
    }
    true
}

/// Original: `RndWitchItem` (items.cpp).
// @port items.cpp|devilution::RndWitchItem(const Player &player, int lvl) sha=1e4b17b6ca0e
fn rnd_witch_item(ctx: &mut Ctx, player: &Player, lvl: i32) -> _item_indexes {
    rnd_vendor_item(ctx, witch_item_ok, false, player, 0, lvl)
}

/// Original: `RndBoyItem` (items.cpp).
// @port items.cpp|devilution::RndBoyItem(const Player &player, int lvl) sha=cbffb54be256
fn rnd_boy_item(ctx: &mut Ctx, player: &Player, lvl: i32) -> _item_indexes {
    rnd_vendor_item(ctx, premium_item_ok, false, player, 0, lvl)
}

/// Original: `HealerItemOk` (items.cpp).
// @port items.cpp|devilution::HealerItemOk(const Player &player, const ItemData &item) sha=a67b1a897d0d
fn healer_item_ok(ctx: &Ctx, player: &Player, item: &ItemData) -> bool {
    if item.itype != ItemType::Misc {
        return false;
    }
    if item.iMiscId == IMISC_SCROLL {
        return item.iSpell == SpellID::Healing;
    }
    if item.iMiscId == IMISC_SCROLLT {
        return item.iSpell == SpellID::HealOther && ctx.init.gb_is_multiplayer;
    }
    let hf = ctx.init.gb_is_hellfire;
    if !ctx.init.gb_is_multiplayer {
        match item.iMiscId {
            IMISC_ELIXSTR => return !hf || player._pBaseStr < player.get_maximum_attribute_value(CharacterAttribute::Strength),
            IMISC_ELIXMAG => return !hf || player._pBaseMag < player.get_maximum_attribute_value(CharacterAttribute::Magic),
            IMISC_ELIXDEX => return !hf || player._pBaseDex < player.get_maximum_attribute_value(CharacterAttribute::Dexterity),
            IMISC_ELIXVIT => return !hf || player._pBaseVit < player.get_maximum_attribute_value(CharacterAttribute::Vitality),
            _ => {}
        }
    }
    if item.iMiscId == IMISC_REJUV || item.iMiscId == IMISC_FULLREJUV {
        return true;
    }
    false
}

/// Original: `RndHealerItem` (items.cpp).
// @port items.cpp|devilution::RndHealerItem(const Player &player, int lvl) sha=a8d30bdc546d
fn rnd_healer_item(ctx: &mut Ctx, player: &Player, lvl: i32) -> _item_indexes {
    rnd_vendor_item(ctx, healer_item_ok, false, player, 0, lvl)
}

/// Original: `RecreateSmithItem` (items.cpp).
// @port items.cpp|devilution::RecreateSmithItem(const Player &player, Item &item, int lvl, int iseed) sha=0876a0e705d4
fn recreate_smith_item(ctx: &mut Ctx, player: &Player, item: &mut Item, lvl: i32, iseed: i32) {
    ctx.rng.set_rnd_seed(iseed as u32);
    let itype = rnd_smith_item(ctx, player, lvl);
    get_item_attrs(ctx, item, itype, lvl);
    item._iSeed = iseed as u32;
    item._iCreateInfo = (lvl | CF_SMITH) as u16;
    item._iIdentified = true;
}

/// Original: `RecreatePremiumItem` (items.cpp).
// @port items.cpp|devilution::RecreatePremiumItem(const Player &player, Item &item, int plvl, int iseed) sha=8a88bdc5081c
fn recreate_premium_item(ctx: &mut Ctx, player: &Player, item: &mut Item, plvl: i32, iseed: i32) {
    ctx.rng.set_rnd_seed(iseed as u32);
    let itype = rnd_premium_item(ctx, player, plvl / 4, plvl);
    get_item_attrs(ctx, item, itype, plvl);
    let hf = ctx.init.gb_is_hellfire;
    get_item_bonus(ctx, player, item, plvl / 2, plvl, true, !hf);
    item._iSeed = iseed as u32;
    item._iCreateInfo = (plvl | CF_SMITHPREMIUM) as u16;
    item._iIdentified = true;
}

/// Original: `RecreateBoyItem` (items.cpp).
// @port items.cpp|devilution::RecreateBoyItem(const Player &player, Item &item, int lvl, int iseed) sha=2e2442ed5f24
fn recreate_boy_item(ctx: &mut Ctx, player: &Player, item: &mut Item, lvl: i32, iseed: i32) {
    ctx.rng.set_rnd_seed(iseed as u32);
    let itype = rnd_boy_item(ctx, player, lvl);
    get_item_attrs(ctx, item, itype, lvl);
    get_item_bonus(ctx, player, item, lvl, 2 * lvl, true, true);
    item._iSeed = iseed as u32;
    item._iCreateInfo = (lvl | CF_BOY) as u16;
    item._iIdentified = true;
}

/// Original: `RecreateWitchItem` (items.cpp).
// @port items.cpp|devilution::RecreateWitchItem(const Player &player, Item &item, _item_indexes idx, int lvl, int iseed) sha=9f258e122f66
fn recreate_witch_item(ctx: &mut Ctx, player: &Player, item: &mut Item, idx: _item_indexes, lvl: i32, iseed: i32) {
    if matches!(idx, IDI_MANA | IDI_FULLMANA | IDI_PORTAL) {
        get_item_attrs(ctx, item, idx, lvl);
    } else if ctx.init.gb_is_hellfire && (114..=117).contains(&idx) {
        ctx.rng.set_rnd_seed(iseed as u32);
        ctx.rng.discard_random_values(1);
        get_item_attrs(ctx, item, idx, lvl);
    } else {
        ctx.rng.set_rnd_seed(iseed as u32);
        let itype = rnd_witch_item(ctx, player, lvl);
        get_item_attrs(ctx, item, itype, lvl);
        let mut iblvl = -1;
        if ctx.rng.generate_rnd(100) <= 5 {
            iblvl = 2 * lvl;
        }
        if iblvl == -1 && item._iMiscId == IMISC_STAFF {
            iblvl = 2 * lvl;
        }
        if iblvl != -1 {
            get_item_bonus(ctx, player, item, iblvl / 2, iblvl, true, true);
        }
    }
    item._iSeed = iseed as u32;
    item._iCreateInfo = (lvl | CF_WITCH) as u16;
    item._iIdentified = true;
}

/// Original: `RecreateHealerItem` (items.cpp).
// @port items.cpp|devilution::RecreateHealerItem(const Player &player, Item &item, _item_indexes idx, int lvl, int iseed) sha=2ea4adcdb0b0
fn recreate_healer_item(ctx: &mut Ctx, player: &Player, item: &mut Item, idx: _item_indexes, lvl: i32, iseed: i32) {
    if matches!(idx, IDI_HEAL | IDI_FULLHEAL | IDI_RESURRECT) {
        get_item_attrs(ctx, item, idx, lvl);
    } else {
        ctx.rng.set_rnd_seed(iseed as u32);
        let itype = rnd_healer_item(ctx, player, lvl);
        get_item_attrs(ctx, item, itype, lvl);
    }
    item._iSeed = iseed as u32;
    item._iCreateInfo = (lvl | CF_HEALER) as u16;
    item._iIdentified = true;
}

/// Original: `RecreateTownItem` (items.cpp).
// @port items.cpp|devilution::RecreateTownItem(const Player &player, Item &item, _item_indexes idx, uint16_t icreateinfo, int iseed) sha=2ac79125c052
fn recreate_town_item(ctx: &mut Ctx, player: &Player, item: &mut Item, idx: _item_indexes, icreateinfo: u16, iseed: i32) {
    let ci = icreateinfo as i32;
    let lvl = ci & CF_LEVEL;
    if ci & CF_SMITH != 0 {
        recreate_smith_item(ctx, player, item, lvl, iseed);
    } else if ci & CF_SMITHPREMIUM != 0 {
        recreate_premium_item(ctx, player, item, lvl, iseed);
    } else if ci & CF_BOY != 0 {
        recreate_boy_item(ctx, player, item, lvl, iseed);
    } else if ci & CF_WITCH != 0 {
        recreate_witch_item(ctx, player, item, idx, lvl, iseed);
    } else if ci & CF_HEALER != 0 {
        recreate_healer_item(ctx, player, item, idx, lvl, iseed);
    }
}

/// Original: `CreateMagicItem` (items.cpp). `spawn` defaults to false.
// @port items.cpp|devilution::CreateMagicItem(Point position, int lvl, ItemType itemType, int imid, int icurs, bool sendmsg, bool delta, bool spawn = false) sha=c8f4115aa5aa
#[allow(clippy::too_many_arguments)]
fn create_magic_item(ctx: &mut Ctx, position: Point, lvl: i32, item_type: ItemType, imid: i32, icurs: i32, sendmsg: bool, delta: bool, spawn: bool) {
    if ctx.items.ActiveItemCount as usize >= MAXITEMS {
        return;
    }
    let ii = allocate_item(ctx) as usize;
    let mut idx = rnd_type_items(ctx, item_type, imid, lvl);
    let me = my_player_copy(ctx);
    loop {
        let seed = ctx.rng.advance_rnd_seed() as u32;
        let done = with_item(ctx, ii, |ctx, item| {
            *item = Item::default();
            setup_all_items(ctx, &me, item, idx, seed, 2 * lvl, 1, true, false, delta);
            item._iCurs as i32 == icurs
        });
        if done {
            break;
        }
        idx = rnd_type_items(ctx, item_type, imid, lvl);
    }
    get_super_item_space(ctx, position, ii as i8);
    let pos = ctx.items.Items[ii].position;
    if sendmsg {
        { let it = ctx.items.Items[ii as usize].clone(); crate::msg::net_send_cmd_p_item(ctx, false, CMD_DROPITEM, pos, &it); }
    }
    if delta {
        crate::msg::delta_add_item(ctx, ii as i32);
    }
    if spawn {
        { let it = ctx.items.Items[ii as usize].clone(); crate::msg::net_send_cmd_p_item(ctx, false, CMD_SPAWNITEM, pos, &it); }
    }
}

/// Original: `NextItemRecord` (items.cpp).
// @port items.cpp|devilution::NextItemRecord(int i) sha=2ba94b24881d
fn next_item_record(ctx: &mut Ctx, i: usize) {
    let s = &mut ctx.items;
    s.gnNumGetRecords -= 1;
    if s.gnNumGetRecords == 0 {
        return;
    }
    let n = s.gnNumGetRecords as usize;
    s.itemrecord[i] = s.itemrecord[n];
}

/// Original: `RndItemForMonsterLevel` (items.cpp).
// @port items.cpp|devilution::RndItemForMonsterLevel(int8_t monsterLevel) sha=8770a0a9d43d
fn rnd_item_for_monster_level(ctx: &mut Ctx, monster_level: i8) -> _item_indexes {
    if ctx.rng.generate_rnd(100) > 40 {
        return IDI_NONE;
    }
    if ctx.rng.generate_rnd(100) > 25 {
        return IDI_GOLD;
    }
    get_item_index_for_droppable_item(ctx, true, &|_, item| item.iMinMLvl as i32 <= monster_level as i32)
}

/// Original: `GetTranslatedItemName` (items.cpp).
// @port items.cpp|devilution::GetTranslatedItemName(const Item &item) sha=3cebb4b8668d
fn get_translated_item_name(ctx: &mut Ctx, item: &Item) -> String {
    let base = &AllItemsList[item.IDidx as usize];
    if item._iCreateInfo == 0 {
        tr(base.iName)
    } else if item._iMiscId == IMISC_BOOK {
        let spell_name = pgettext("spell", get_spell_data(item._iSpell).sNameText);
        let mut name = tr(base.iName);
        name.push_str(&spell_name);
        name
    } else if item._iMiscId == IMISC_EAR {
        tr("Ear of {:s}").replacen("{:s}", item._iIName.as_str(), 1)
    } else if item._iMiscId > IMISC_OILFIRST && item._iMiscId < IMISC_OILLAST {
        for i in 0..10 {
            if OilMagic[i] != item._iMiscId {
                continue;
            }
            return tr(OilNames[i]);
        }
        crate::appfat::app_fatal(ctx, "unkown oil")
    } else if item._itype == ItemType::Staff && item._iSpell != SpellID::Null && item._iMagical != ITEM_QUALITY_UNIQUE {
        generate_staff_name(ctx, base, item._iSpell, true)
    } else {
        tr(base.iName)
    }
}

/// Original: `GetTranslatedItemNameMagical` (items.cpp).
// @port items.cpp|devilution::GetTranslatedItemNameMagical(const Item &item, bool hellfireItem, bool translate, std::optional<bool> forceNameLengthCheck) sha=7d78b46d72a6
fn get_translated_item_name_magical(ctx: &mut Ctx, item: &Item, hellfire_item: bool, translate: bool, force_name_length_check: Option<bool>) -> String {
    let mut identified_name = String::new();
    let base = &AllItemsList[item.IDidx as usize];
    let ci = item._iCreateInfo as i32;
    let lvl = ci & CF_LEVEL;
    let onlygood = ci & (CF_ONLYGOOD | CF_SMITHPREMIUM | CF_BOY | CF_WITCH) != 0;
    let current_seed = ctx.rng.lcg_engine_state();
    ctx.rng.set_rnd_seed(item._iSeed);
    let mut minlvl;
    let maxlvl;
    if ci & CF_SMITHPREMIUM != 0 {
        ctx.rng.discard_random_values(2);
        minlvl = lvl / 2;
        maxlvl = lvl;
    } else if ci & CF_BOY != 0 {
        ctx.rng.discard_random_values(2);
        minlvl = lvl;
        maxlvl = lvl * 2;
    } else if ci & CF_WITCH != 0 {
        ctx.rng.discard_random_values(2);
        let mut iblvl = -1;
        if ctx.rng.generate_rnd(100) <= 5 {
            iblvl = 2 * lvl;
        }
        if iblvl == -1 && item._iMiscId == IMISC_STAFF {
            iblvl = 2 * lvl;
        }
        minlvl = iblvl / 2;
        maxlvl = iblvl;
    } else {
        ctx.rng.discard_random_values(1);
        let iblvl = get_item_b_level(ctx, lvl, item._iMiscId, onlygood, ci & CF_UPER15 != 0);
        minlvl = iblvl / 2;
        maxlvl = iblvl;
        ctx.rng.discard_random_values(1);
    }
    if minlvl > 25 {
        minlvl = 25;
    }
    let mut affix_item_type = AffixItemType::None;
    match item._itype {
        ItemType::Sword | ItemType::Axe | ItemType::Mace => affix_item_type = AffixItemType::Weapon,
        ItemType::Bow => affix_item_type = AffixItemType::Bow,
        ItemType::Shield => affix_item_type = AffixItemType::Shield,
        ItemType::LightArmor | ItemType::Helm | ItemType::MediumArmor | ItemType::HeavyArmor => affix_item_type = AffixItemType::Armor,
        ItemType::Staff => {
            let allowspells = !hellfire_item || (ci & CF_SMITHPREMIUM) == 0;
            if !allowspells {
                affix_item_type = AffixItemType::Staff;
            } else if !hellfire_item && ctx.rng.flip_coin(4) {
                affix_item_type = AffixItemType::Staff;
            } else {
                ctx.rng.discard_random_values(2);
                let preidx = get_staff_prefix_id(ctx, maxlvl, onlygood, hellfire_item);
                if preidx == -1 || item._iSpell == SpellID::Null {
                    if force_name_length_check.is_some() {
                        identified_name.clear();
                    } else {
                        crate::platform::log::verbose!(
                            "GetTranslatedItemNameMagical failed for item '{}' with preidx '{}' and spellid '{}'",
                            item._iIName.as_str(),
                            preidx,
                            item._iSpell as i8
                        );
                        identified_name = item._iIName.as_str().to_string();
                    }
                } else {
                    identified_name = generate_staff_name_magical(ctx, base, item._iSpell, preidx, translate, force_name_length_check);
                }
            }
        }
        ItemType::Ring | ItemType::Amulet => affix_item_type = AffixItemType::Misc,
        ItemType::None | ItemType::Misc | ItemType::Gold => {}
    }
    if affix_item_type != AffixItemType::None {
        let mut p_prefix: Option<usize> = None;
        let mut p_sufix: Option<usize> = None;
        get_item_power_prefix_and_suffix(
            ctx,
            minlvl,
            maxlvl,
            affix_item_type,
            onlygood,
            hellfire_item,
            &mut |ctx, prefix| {
                p_prefix = Some(prefix);
                ctx.rng.discard_random_values(1);
                match ItemPrefixes[prefix].power.type_ {
                    IPL_DOPPELGANGER | IPL_TOHIT_DAMP => ctx.rng.discard_random_values(2),
                    IPL_TOHIT_DAMP_CURSE => ctx.rng.discard_random_values(1),
                    _ => {}
                }
            },
            &mut |_, suffix| {
                p_sufix = Some(suffix);
            },
        );
        let pre = p_prefix.map(|i| &ItemPrefixes[i]);
        let suf = p_sufix.map(|i| &ItemSuffixes[i]);
        identified_name = generate_magic_item_name(&tr(base.iName), pre, suf, translate);
        let check = match force_name_length_check {
            Some(v) => v,
            None => !string_in_panel(ctx, &identified_name),
        };
        if check {
            identified_name = generate_magic_item_name(&tr(base.iSName.unwrap_or("")), pre, suf, translate);
        }
    }
    ctx.rng.set_rnd_seed(current_seed);
    identified_name
}

/// Original: `devilution::IsItemAvailable` (items.cpp).
// @port items.cpp|devilution::IsItemAvailable(int i) sha=398375436143
pub fn is_item_available(ctx: &Ctx, i: i32) -> bool {
    if i < 0 || i > IDI_LAST as i32 {
        return false;
    }
    if ctx.init.gb_is_spawn {
        if (62..=70).contains(&i) {
            return false; // Medium and heavy armors
        }
        if matches!(i, 105 | 107 | 108 | 110 | 111 | 113) {
            return false; // Unavailable scrolls
        }
    }
    if ctx.init.gb_is_hellfire {
        return true;
    }
    (i != IDI_MAPOFDOOM as i32
        && i != IDI_LGTFORGE as i32
        && (i < IDI_OIL as i32 || i > IDI_GREYSUIT as i32)
        && !(83..=86).contains(&i)
        && i != 92
        && !(161..=165).contains(&i)
        && i != IDI_SORCERER as i32)
        || (ctx.options.gameplay.test_bard.get() && matches!(i as _item_indexes, IDI_BARDSWORD | IDI_BARDDAGGER))
}

/// Original: `devilution::GetOutlineColor` (items.cpp).
// @port items.cpp|devilution::GetOutlineColor(const Item &item, bool checkReq) sha=97ec5ea5fd4c
pub fn get_outline_color(item: &Item, check_req: bool) -> u8 {
    if check_req && !item._iStatFlag {
        return ICOL_RED;
    }
    if item._itype == ItemType::Gold {
        return ICOL_YELLOW;
    }
    if item._iMagical == ITEM_QUALITY_MAGIC {
        return ICOL_BLUE;
    }
    if item._iMagical == ITEM_QUALITY_UNIQUE {
        return ICOL_YELLOW;
    }
    ICOL_WHITE
}

/// Original: `devilution::IsUniqueAvailable` (items.cpp).
// @port items.cpp|devilution::IsUniqueAvailable(int i) sha=9b439b2736a5
pub fn is_unique_available(ctx: &Ctx, i: i32) -> bool {
    ctx.init.gb_is_hellfire || i <= 89
}

/// Original: `devilution::InitItemGFX` (items.cpp).
// @port items.cpp|devilution::InitItemGFX() sha=4c1a0ab30db4
pub fn init_item_gfx(ctx: &mut Ctx) {
    let item_types = if ctx.init.gb_is_hellfire { ITEMTYPES } else { 35 };
    for i in 0..item_types {
        let arglist = format!("items\\{}", ItemDropNames[i]);
        ctx.items.itemanims[i] = Some(crate::engine::load_sprites::load_cel(ctx, &arglist, ITEM_ANIM_WIDTH));
    }
    ctx.items.UniqueItemFlags = [false; 128];
}

/// Original: `devilution::InitItems` (items.cpp).
// @port items.cpp|devilution::InitItems() sha=38ec7b3b95ac
pub fn init_items(ctx: &mut Ctx) {
    ctx.items.ActiveItemCount = 0;
    *ctx.items.dItem = [[0; MAXDUNY]; MAXDUNX];
    for item in ctx.items.Items.iter_mut() {
        item.clear();
        item.position = Point::new(0, 0);
        item._iAnimFlag = false;
        item._iSelFlag = 0;
        item._iIdentified = false;
        item._iPostDraw = false;
    }
    for i in 0..MAXITEMS {
        ctx.items.ActiveItems[i] = i as u8;
    }
    if !ctx.gendung.setlevel {
        ctx.rng.discard_random_values(1);
        if crate::quests::is_quest_available(ctx, Q_ROCK) {
            spawn_rock(ctx);
        }
        if crate::quests::is_quest_available(ctx, Q_ANVIL) {
            let p = ctx.gendung.SetPiece.position.mega_to_world() + Displacement::new(11, 11);
            spawn_quest_item(ctx, IDI_ANVIL, p, 0, 1, false);
        }
        let cl = ctx.gendung.currlevel;
        if ctx.multi.sgGameInitInfo.bCowQuest != 0 && cl == 20 {
            spawn_quest_item(ctx, IDI_BROWNSUIT, Point::new(25, 25), 3, 1, false);
        }
        if ctx.multi.sgGameInitInfo.bCowQuest != 0 && cl == 19 {
            spawn_quest_item(ctx, IDI_GREYSUIT, Point::new(25, 25), 3, 1, false);
        }
        if ctx.init.gb_is_multiplayer {
            if crate::quests::is_quest_available(ctx, Q_MUSHROOM) {
                spawn_quest_item(ctx, IDI_FUNGALTM, Point::new(0, 0), 5, 1, false);
            }
            let veil = &ctx.quests.Quests[Q_VEIL as usize];
            if cl as i32 == veil._qlevel as i32 + 1 && veil._qactive != QUEST_NOTAVAIL {
                spawn_quest_item(ctx, IDI_GLDNELIX, Point::new(0, 0), 5, 1, false);
            }
        }
        if cl > 0 && cl < 16 {
            add_init_items(ctx);
        }
        if (21..=23).contains(&cl) {
            spawn_note(ctx);
        }
    }
    ctx.items.ShowUniqueItemInfoBox = false;
    init_item_get_records(ctx);
}

/// Original: `devilution::CalcPlrItemVals` (items.cpp).
// @port items.cpp|devilution::CalcPlrItemVals(Player &player, bool loadgfx) sha=0002c7b1127e
pub fn calc_plr_item_vals(ctx: &mut Ctx, pnum: usize, loadgfx: bool) {
    let mut mind = 0;
    let mut maxd = 0;
    let mut tac = 0;
    let mut bdam = 0;
    let mut btohit = 0;
    let mut bac = 0;
    let mut iflgs = ItemSpecialEffect::None;
    let mut p_dam_ac_flags = ItemSpecialEffectHf::None;
    let mut sadd = 0;
    let mut madd = 0;
    let mut dadd = 0;
    let mut vadd = 0;
    let mut spl: u64 = 0;
    let mut fr = 0;
    let mut lr = 0;
    let mut mr = 0;
    let mut dmod = 0;
    let mut ghit = 0;
    let mut lrad = 10;
    let mut ihp = 0;
    let mut imana = 0;
    let mut spllvladd = 0;
    let mut enac = 0;
    let mut fmin = 0;
    let mut fmax = 0;
    let mut lmin = 0;
    let mut lmax = 0;

    {
        let player = &ctx.players.Players[pnum];
        for item in player.InvBody.iter() {
            if !item.is_empty() && item._iStatFlag {
                mind += item._iMinDam as i32;
                maxd += item._iMaxDam as i32;
                tac += item._iAC as i32;
                if crate::spells::is_valid_spell(ctx, item._iSpell) {
                    spl |= crate::spells::get_spell_bitmask(item._iSpell);
                }
                if item._iMagical == ITEM_QUALITY_NORMAL || item._iIdentified {
                    bdam += item._iPLDam as i32;
                    btohit += item._iPLToHit as i32;
                    if item._iPLAC != 0 {
                        let mut tmpac = item._iAC as i32;
                        tmpac *= item._iPLAC as i32;
                        tmpac /= 100;
                        if tmpac == 0 {
                            tmpac = (item._iPLAC as i32).signum();
                        }
                        bac += tmpac;
                    }
                    iflgs |= item._iFlags;
                    p_dam_ac_flags |= item._iDamAcFlags;
                    sadd += item._iPLStr as i32;
                    madd += item._iPLMag as i32;
                    dadd += item._iPLDex as i32;
                    vadd += item._iPLVit as i32;
                    fr += item._iPLFR as i32;
                    lr += item._iPLLR as i32;
                    mr += item._iPLMR as i32;
                    dmod += item._iPLDamMod as i32;
                    ghit += item._iPLGetHit as i32;
                    lrad += item._iPLLight as i32;
                    ihp += item._iPLHP as i32;
                    imana += item._iPLMana as i32;
                    spllvladd += item._iSplLvlAdd as i32;
                    enac += item._iPLEnAc as i32;
                    fmin += item._iFMinDam as i32;
                    fmax += item._iFMaxDam as i32;
                    lmin += item._iLMinDam as i32;
                    lmax += item._iLMaxDam as i32;
                }
            }
        }
        if mind == 0 && maxd == 0 {
            mind = 1;
            maxd = 1;
            let l = &player.InvBody[INVLOC_HAND_LEFT as usize];
            let r = &player.InvBody[INVLOC_HAND_RIGHT as usize];
            if l._itype == ItemType::Shield && l._iStatFlag {
                maxd = 3;
            }
            if r._itype == ItemType::Shield && r._iStatFlag {
                maxd = 3;
            }
            if player._pClass == HeroClass::Monk {
                mind = mind.max(player._pLevel as i32 / 2);
                maxd = maxd.max(player._pLevel as i32);
            }
        }
        let lvl = player._pLevel as i32;
        if player._pSpellFlags.has_any_of(SpellFlag::RageActive) {
            sadd += 2 * lvl;
            dadd += lvl + lvl / 2;
            vadd += 2 * lvl;
        }
        if player._pSpellFlags.has_any_of(SpellFlag::RageCooldown) {
            sadd -= 2 * lvl;
            dadd -= lvl + lvl / 2;
            vadd -= 2 * lvl;
        }
    }

    {
        let player = &mut ctx.players.Players[pnum];
        player._pIMinDam = mind;
        player._pIMaxDam = maxd;
        player._pIAC = tac;
        player._pIBonusDam = bdam;
        player._pIBonusToHit = btohit;
        player._pIBonusAC = bac;
        player._pIFlags = iflgs;
        player.pDamAcFlags = p_dam_ac_flags;
        player._pIBonusDamMod = dmod;
        player._pIGetHit = ghit;
    }
    lrad = lrad.clamp(2, 15);
    if ctx.players.Players[pnum]._pLightRad as i32 != lrad {
        let light_id = ctx.players.Players[pnum].lightId;
        crate::lighting::change_light_radius(ctx, light_id, lrad as u8);
        crate::lighting::change_vision_radius(ctx, pnum as i32, lrad);
        ctx.players.Players[pnum]._pLightRad = lrad as i8;
    }
    {
        let player = &mut ctx.players.Players[pnum];
        player._pStrength = 0.max(sadd + player._pBaseStr);
        player._pMagic = 0.max(madd + player._pBaseMag);
        player._pDexterity = 0.max(dadd + player._pBaseDex);
        player._pVitality = 0.max(vadd + player._pBaseVit);
        let lvl = player._pLevel as i32;
        let lt = player.InvBody[INVLOC_HAND_LEFT as usize]._itype;
        let rt = player.InvBody[INVLOC_HAND_RIGHT as usize]._itype;
        match player._pClass {
            HeroClass::Rogue => player._pDamageMod = lvl * (player._pStrength + player._pDexterity) / 200,
            HeroClass::Monk => {
                player._pDamageMod = lvl * (player._pStrength + player._pDexterity) / 150;
                let l = &player.InvBody[INVLOC_HAND_LEFT as usize];
                let r = &player.InvBody[INVLOC_HAND_RIGHT as usize];
                if (!l.is_empty() && l._itype != ItemType::Staff) || (!r.is_empty() && r._itype != ItemType::Staff) {
                    player._pDamageMod /= 2;
                }
            }
            HeroClass::Bard => {
                if lt == ItemType::Sword || rt == ItemType::Sword {
                    player._pDamageMod = lvl * (player._pStrength + player._pDexterity) / 150;
                } else if lt == ItemType::Bow || rt == ItemType::Bow {
                    player._pDamageMod = lvl * (player._pStrength + player._pDexterity) / 250;
                } else {
                    player._pDamageMod = lvl * player._pStrength / 100;
                }
            }
            HeroClass::Barbarian => {
                if lt == ItemType::Axe || rt == ItemType::Axe {
                    player._pDamageMod = lvl * player._pStrength / 75;
                } else if lt == ItemType::Mace || rt == ItemType::Mace {
                    player._pDamageMod = lvl * player._pStrength / 75;
                } else if lt == ItemType::Bow || rt == ItemType::Bow {
                    player._pDamageMod = lvl * player._pStrength / 300;
                } else {
                    player._pDamageMod = lvl * player._pStrength / 100;
                }
                if lt == ItemType::Shield || rt == ItemType::Shield {
                    if lt == ItemType::Shield {
                        player._pIAC -= player.InvBody[INVLOC_HAND_LEFT as usize]._iAC as i32 / 2;
                    } else if rt == ItemType::Shield {
                        player._pIAC -= player.InvBody[INVLOC_HAND_RIGHT as usize]._iAC as i32 / 2;
                    }
                } else if !matches!(lt, ItemType::Staff | ItemType::Bow) && !matches!(rt, ItemType::Staff | ItemType::Bow) {
                    player._pDamageMod += lvl * player._pVitality / 100;
                }
                player._pIAC += lvl / 4;
            }
            _ => player._pDamageMod = lvl * player._pStrength / 100,
        }
        player._pISpells = spl;
    }
    crate::player::ensure_valid_readied_spell(ctx, pnum);
    let lvl;
    let class;
    {
        let player = &mut ctx.players.Players[pnum];
        player._pISplLvlAdd = spllvladd as i8;
        player._pIEnAc = enac;
        lvl = player._pLevel as i32;
        class = player._pClass;
        if class == HeroClass::Barbarian {
            mr += lvl;
            fr += lvl;
            lr += lvl;
        }
        if player._pSpellFlags.has_any_of(SpellFlag::RageCooldown) {
            mr -= lvl;
            fr -= lvl;
            lr -= lvl;
        }
        if iflgs.has_any_of(ItemSpecialEffect::ZeroResistance) {
            mr = 0;
            fr = 0;
            lr = 0;
        }
        player._pMagResist = mr.clamp(0, crate::player::MaxResistance) as i8;
        player._pFireResist = fr.clamp(0, crate::player::MaxResistance) as i8;
        player._pLghtResist = lr.clamp(0, crate::player::MaxResistance) as i8;
        let pd = &crate::tables::playerdat::PlayersData[class as usize];
        vadd = (vadd * pd.itmLife as i32) >> 6;
        ihp += vadd << 6;
        madd = (madd * pd.itmMana as i32) >> 6;
        imana += madd << 6;
        player._pMaxHP = ihp + player._pMaxHPBase;
        player._pHitPoints = (ihp + player._pHPBase).min(player._pMaxHP);
    }
    if Some(pnum) == ctx.players.MyPlayer && (ctx.players.Players[pnum]._pHitPoints >> 6) <= 0 {
        crate::player::set_player_hit_points(ctx, pnum, 0);
    }
    let gfx_num;
    {
        let player = &mut ctx.players.Players[pnum];
        player._pMaxMana = imana + player._pMaxManaBase;
        player._pMana = (imana + player._pManaBase).min(player._pMaxMana);
        player._pIFMinDam = fmin;
        player._pIFMaxDam = fmax;
        player._pILMinDam = lmin;
        player._pILMaxDam = lmax;
        player._pInfraFlag = false;
        player._pBlockFlag = false;
        let hl = INVLOC_HAND_LEFT as usize;
        let hr = INVLOC_HAND_RIGHT as usize;
        if class == HeroClass::Monk {
            if player.InvBody[hl]._itype == ItemType::Staff && player.InvBody[hl]._iStatFlag {
                player._pBlockFlag = true;
                player._pIFlags |= ItemSpecialEffect::FastBlock;
            }
            if player.InvBody[hr]._itype == ItemType::Staff && player.InvBody[hr]._iStatFlag {
                player._pBlockFlag = true;
                player._pIFlags |= ItemSpecialEffect::FastBlock;
            }
            if player.InvBody[hl].is_empty() && player.InvBody[hr].is_empty() {
                player._pBlockFlag = true;
            }
            if player.InvBody[hl]._iClass == ICLASS_WEAPON && player.get_item_location(&player.InvBody[hl]) != ILOC_TWOHAND && player.InvBody[hr].is_empty() {
                player._pBlockFlag = true;
            }
            if player.InvBody[hr]._iClass == ICLASS_WEAPON && player.get_item_location(&player.InvBody[hr]) != ILOC_TWOHAND && player.InvBody[hl].is_empty() {
                player._pBlockFlag = true;
            }
        }
        let mut weapon_item_type = ItemType::None;
        let mut holds_shield = false;
        if !player.InvBody[hl].is_empty() && player.InvBody[hl]._iClass == ICLASS_WEAPON && player.InvBody[hl]._iStatFlag {
            weapon_item_type = player.InvBody[hl]._itype;
        }
        if !player.InvBody[hr].is_empty() && player.InvBody[hr]._iClass == ICLASS_WEAPON && player.InvBody[hr]._iStatFlag {
            weapon_item_type = player.InvBody[hr]._itype;
        }
        if player.InvBody[hl]._itype == ItemType::Shield && player.InvBody[hl]._iStatFlag {
            player._pBlockFlag = true;
            holds_shield = true;
        }
        if player.InvBody[hr]._itype == ItemType::Shield && player.InvBody[hr]._iStatFlag {
            player._pBlockFlag = true;
            holds_shield = true;
        }
        let anim_weapon_id = match weapon_item_type {
            ItemType::Sword => {
                if holds_shield {
                    PlayerWeaponGraphic::SwordShield
                } else {
                    PlayerWeaponGraphic::Sword
                }
            }
            ItemType::Axe => PlayerWeaponGraphic::Axe,
            ItemType::Bow => PlayerWeaponGraphic::Bow,
            ItemType::Mace => {
                if holds_shield {
                    PlayerWeaponGraphic::MaceShield
                } else {
                    PlayerWeaponGraphic::Mace
                }
            }
            ItemType::Staff => PlayerWeaponGraphic::Staff,
            _ => {
                if holds_shield {
                    PlayerWeaponGraphic::UnarmedShield
                } else {
                    PlayerWeaponGraphic::Unarmed
                }
            }
        };
        let chest = INVLOC_CHEST as usize;
        let mut anim_armor_id = PlayerArmorGraphic::Light;
        if player.InvBody[chest]._itype == ItemType::HeavyArmor && player.InvBody[chest]._iStatFlag {
            if class == HeroClass::Monk && player.InvBody[chest]._iMagical == ITEM_QUALITY_UNIQUE {
                player._pIAC += lvl / 2;
            }
            anim_armor_id = PlayerArmorGraphic::Heavy;
        } else if player.InvBody[chest]._itype == ItemType::MediumArmor && player.InvBody[chest]._iStatFlag {
            if class == HeroClass::Monk {
                if player.InvBody[chest]._iMagical == ITEM_QUALITY_UNIQUE {
                    player._pIAC += lvl * 2;
                } else {
                    player._pIAC += lvl / 2;
                }
            }
            anim_armor_id = PlayerArmorGraphic::Medium;
        } else if class == HeroClass::Monk {
            player._pIAC += lvl * 2;
        }
        gfx_num = anim_weapon_id as u8 | anim_armor_id as u8;
    }
    if ctx.players.Players[pnum]._pgfxnum != gfx_num && loadgfx {
        ctx.players.Players[pnum]._pgfxnum = gfx_num;
        crate::player::reset_player_gfx(ctx, pnum);
        crate::player::set_plr_anims(ctx, pnum);
        ctx.players.Players[pnum].previewCelSprite = None;
        let graphic = ctx.players.Players[pnum].get_graphic();
        let (number_of_frames, ticks_per_frame) = ctx.players.Players[pnum].get_animation_frames_and_ticks_per_frame(graphic);
        crate::player::load_plr_gfx(ctx, pnum, graphic);
        let mut sprites = None;
        if !ctx.diablo.headless_mode {
            let p = &ctx.players.Players[pnum];
            sprites = p.AnimationData[graphic as usize].sprites_for_direction(p._pdir);
        }
        ctx.players.Players[pnum].AnimInfo.change_animation_data(sprites, number_of_frames, ticks_per_frame);
    } else {
        ctx.players.Players[pnum]._pgfxnum = gfx_num;
    }
    if Some(pnum) == ctx.players.MyPlayer {
        let amulet = &ctx.players.Players[pnum].InvBody[INVLOC_AMULET as usize];
        if amulet.is_empty() || amulet.IDidx != IDI_AURIC {
            let half = ctx.items.MaxGold;
            ctx.items.MaxGold = GOLD_MAX_LIMIT;
            if half != ctx.items.MaxGold {
                crate::player::strip_top_gold(ctx, pnum);
            }
        } else {
            ctx.items.MaxGold = GOLD_MAX_LIMIT * 2;
        }
    }
    redraw_component(ctx, PanelDrawComponent::Mana);
    redraw_component(ctx, PanelDrawComponent::Health);
}

/// Original: `devilution::CalcPlrInv` (items.cpp).
// @port items.cpp|devilution::CalcPlrInv(Player &player, bool loadgfx) sha=92539ac5c278
pub fn calc_plr_inv(ctx: &mut Ctx, pnum: usize, mut loadgfx: bool) {
    calc_self_items(&mut ctx.players.Players[pnum]);
    let is_me = Some(pnum) == ctx.players.MyPlayer;
    if !is_me && !crate::player::is_on_active_level(ctx, pnum) {
        loadgfx = false;
    }
    calc_plr_item_vals(ctx, pnum, loadgfx);
    if is_me {
        let snapshot = ctx.players.Players[pnum].clone();
        let player = &mut ctx.players.Players[pnum];
        let n = player._pNumInv as usize;
        for item in player.InvList[..n].iter_mut() {
            item.update_required_stats_cache_for_player(&snapshot);
        }
        for item in player.SpdList.iter_mut() {
            if !item.is_empty() {
                item.update_required_stats_cache_for_player(&snapshot);
            }
        }
        crate::player::calc_scrolls(ctx, pnum);
        crate::player::calc_plr_staff(ctx, pnum);
        if ctx.stash.IsStashOpen {
            crate::qol::stash::refresh_item_stat_flags(ctx);
        }
    }
}

/// Original: `devilution::InitializeItem` (items.cpp).
// @port items.cpp|devilution::InitializeItem(Item &item, _item_indexes itemData) sha=6c94d59f655a
pub fn initialize_item(ctx: &Ctx, item: &mut Item, item_data: _item_indexes) {
    let p = &AllItemsList[item_data as usize];
    *item = Item::default();
    item._itype = p.itype;
    item._iCurs = p.iCurs as u8;
    item._iName.set(p.iName);
    item._iIName.set(p.iName);
    item._iLoc = p.iLoc;
    item._iClass = p.iClass;
    item._iMinDam = p.iMinDam;
    item._iMaxDam = p.iMaxDam;
    item._iAC = p.iMinAC as i16;
    item._iMiscId = p.iMiscId;
    item._iSpell = p.iSpell;
    if p.iMiscId == IMISC_STAFF {
        item._iCharges = if ctx.init.gb_is_hellfire { 18 } else { 40 };
    }
    item._iMaxCharges = item._iCharges;
    item._iDurability = p.iDurability as i32;
    item._iMaxDur = p.iDurability as i32;
    item._iMinStr = p.iMinStr as i8;
    item._iMinMag = p.iMinMag;
    item._iMinDex = p.iMinDex as i8;
    item._ivalue = p.iValue as i32;
    item._iIvalue = p.iValue as i32;
    item._iPrePower = IPL_INVALID;
    item._iSufPower = IPL_INVALID;
    item._iMagical = ITEM_QUALITY_NORMAL;
    item.IDidx = item_data;
    if ctx.init.gb_is_hellfire {
        item.dwBuff |= CF_HELLFIRE as u32;
    }
}

/// Original: `devilution::GenerateNewSeed` (items.cpp).
// @port items.cpp|devilution::GenerateNewSeed(Item &item) sha=84e11645a96e
pub fn generate_new_seed(ctx: &mut Ctx, item: &mut Item) {
    item._iSeed = ctx.rng.advance_rnd_seed() as u32;
}

/// Original: `devilution::GetGoldCursor` (items.cpp).
// @port items.cpp|devilution::GetGoldCursor(int value) sha=b0c35edd4a3b
pub fn get_gold_cursor(value: i32) -> i32 {
    if value >= GOLD_MEDIUM_LIMIT {
        return ICURS_GOLD_LARGE as i32;
    }
    if value <= GOLD_SMALL_LIMIT {
        return ICURS_GOLD_SMALL as i32;
    }
    ICURS_GOLD_MEDIUM as i32
}

/// Original: `devilution::SetPlrHandGoldCurs` (items.cpp).
// @port items.cpp|devilution::SetPlrHandGoldCurs(Item &gold) sha=5039c168ee92
pub fn set_plr_hand_gold_curs(gold: &mut Item) {
    gold._iCurs = get_gold_cursor(gold._ivalue) as u8;
}

/// Original: `devilution::CreatePlrItems` (items.cpp).
// @port items.cpp|devilution::CreatePlrItems(Player &player) sha=f0854f84933f
pub fn create_plr_items(ctx: &mut Ctx, pnum: usize) {
    {
        let player = &mut ctx.players.Players[pnum];
        for item in player.InvBody.iter_mut() {
            item.clear();
        }
        player.InvGrid = [0; crate::player::InventoryGridCells];
        for item in player.InvList.iter_mut() {
            item.clear();
        }
        player._pNumInv = 0;
        for item in player.SpdList.iter_mut() {
            item.clear();
        }
    }
    let hf = ctx.init.gb_is_hellfire;
    let class = ctx.players.Players[pnum]._pClass;
    let (left, right, belt): (_item_indexes, Option<_item_indexes>, _item_indexes) = match class {
        HeroClass::Warrior => (IDI_WARRIOR, Some(IDI_WARRSHLD), IDI_HEAL),
        HeroClass::Rogue => (IDI_ROGUE, None, IDI_HEAL),
        HeroClass::Sorcerer => (if hf { IDI_SORCERER } else { IDI_SORCERER_DIABLO }, None, if hf { IDI_HEAL } else { IDI_MANA }),
        HeroClass::Monk => (IDI_SHORTSTAFF, None, IDI_HEAL),
        HeroClass::Bard => (IDI_BARDSWORD, Some(IDI_BARDDAGGER), IDI_HEAL),
        HeroClass::Barbarian => (IDI_BARBARIAN, Some(IDI_WARRSHLD), IDI_HEAL),
    };
    let mut make = |ctx: &mut Ctx, idx: _item_indexes| {
        let mut it = Item::default();
        initialize_item(ctx, &mut it, idx);
        generate_new_seed(ctx, &mut it);
        it
    };
    let it = make(ctx, left);
    ctx.players.Players[pnum].InvBody[INVLOC_HAND_LEFT as usize] = it;
    if let Some(r) = right {
        let it = make(ctx, r);
        ctx.players.Players[pnum].InvBody[INVLOC_HAND_RIGHT as usize] = it;
    }
    if class == HeroClass::Warrior {
        let club = make(ctx, IDI_WARRCLUB);
        crate::inv::auto_place_item_in_inventory_slot(ctx, pnum, 0, &club, true);
    }
    let it = make(ctx, belt);
    ctx.players.Players[pnum].SpdList[0] = it;
    let it = make(ctx, belt);
    ctx.players.Players[pnum].SpdList[1] = it;

    let n = ctx.players.Players[pnum]._pNumInv as usize;
    let mut gold = Item::default();
    make_gold_stack(ctx, &mut gold, 100);
    let player = &mut ctx.players.Players[pnum];
    player.InvList[n] = gold;
    player._pNumInv += 1;
    player.InvGrid[30] = player._pNumInv as i8;
    player._pGold = player.InvList[n]._ivalue;
    calc_plr_item_vals(ctx, pnum, false);
}

/// Original: `devilution::ItemSpaceOk` (items.cpp).
// @port items.cpp|devilution::ItemSpaceOk(Point position) sha=bdd025582e84
pub fn item_space_ok(ctx: &Ctx, position: Point) -> bool {
    if !in_dungeon_bounds(position) {
        return false;
    }
    if crate::engine::path::is_tile_solid(ctx, position) {
        return false;
    }
    let (x, y) = (position.x as usize, position.y as usize);
    if ctx.items.dItem[x][y] != 0 {
        return false;
    }
    if ctx.gendung.dMonster[x][y] != 0 {
        return false;
    }
    if ctx.gendung.dPlayer[x][y] != 0 {
        return false;
    }
    if crate::objects::is_item_blocking_object_at_position(ctx, position) {
        return false;
    }
    true
}

/// Original: `devilution::AllocateItem` (items.cpp).
// @port items.cpp|devilution::AllocateItem() sha=ae1f39369d83
pub fn allocate_item(ctx: &mut Ctx) -> i32 {
    assert!((ctx.items.ActiveItemCount as usize) < MAXITEMS);
    let inum = ctx.items.ActiveItems[ctx.items.ActiveItemCount as usize] as usize;
    ctx.items.ActiveItemCount += 1;
    ctx.items.Items[inum] = Item::default();
    inum as i32
}

/// Original: `devilution::PlaceItemInWorld` (items.cpp).
// @port items.cpp|devilution::PlaceItemInWorld(Item &&item, WorldTilePosition position) sha=2bef1dd3d1ba
pub fn place_item_in_world(ctx: &mut Ctx, item: Item, position: Point) -> u8 {
    assert!((ctx.items.ActiveItemCount as usize) < MAXITEMS);
    let ii = ctx.items.ActiveItems[ctx.items.ActiveItemCount as usize];
    ctx.items.ActiveItemCount += 1;
    ctx.items.dItem[position.x as usize][position.y as usize] = (ii + 1) as i8;
    ctx.items.Items[ii as usize] = item;
    ctx.items.Items[ii as usize].position = position;
    with_item(ctx, ii as usize, |ctx, it| respawn_item(ctx, it, true));
    if CornerStoneStruct::is_available(ctx) && position == ctx.items.CornerStone.position {
        ctx.items.CornerStone.item = ctx.items.Items[ii as usize].clone();
        crate::minitext::init_q_text_msg(ctx, TEXT_CORNSTN);
        ctx.quests.Quests[Q_CORNSTN as usize]._qactive = QUEST_DONE;
    }
    ii
}

/// Original: `devilution::GetSuperItemLoc` (items.cpp).
// @port items.cpp|devilution::GetSuperItemLoc(Point position) sha=1d27b800e6f0
pub fn get_super_item_loc(ctx: &Ctx, position: Point) -> Point {
    crate::engine::path::find_closest_valid_position(ctx, &|ctx, p| item_space_ok(ctx, p), position, 1, 50).unwrap_or(Point::new(0, 0))
}

/// Original: `devilution::GetItemAttrs` (items.cpp).
// @port items.cpp|devilution::GetItemAttrs(Item &item, _item_indexes itemData, int lvl) sha=ee4809770885
pub fn get_item_attrs(ctx: &mut Ctx, item: &mut Item, item_data: _item_indexes, lvl: i32) {
    let b = &AllItemsList[item_data as usize];
    item._itype = b.itype;
    item._iCurs = b.iCurs as u8;
    item._iName.set(b.iName);
    item._iIName.set(b.iName);
    item._iLoc = b.iLoc;
    item._iClass = b.iClass;
    item._iMinDam = b.iMinDam;
    item._iMaxDam = b.iMaxDam;
    item._iAC = (b.iMinAC as i32 + ctx.rng.generate_rnd(b.iMaxAC as i32 - b.iMinAC as i32 + 1)) as i16;
    item._iFlags = b.iFlags;
    item._iMiscId = b.iMiscId;
    item._iSpell = b.iSpell;
    item._iMagical = ITEM_QUALITY_NORMAL;
    item._ivalue = b.iValue as i32;
    item._iIvalue = b.iValue as i32;
    item._iDurability = b.iDurability as i32;
    item._iMaxDur = b.iDurability as i32;
    item._iMinStr = b.iMinStr as i8;
    item._iMinMag = b.iMinMag;
    item._iMinDex = b.iMinDex as i8;
    item.IDidx = item_data;
    if ctx.init.gb_is_hellfire {
        item.dwBuff |= CF_HELLFIRE as u32;
    }
    item._iPrePower = IPL_INVALID;
    item._iSufPower = IPL_INVALID;
    if item._iMiscId == IMISC_BOOK {
        get_book_spell(ctx, item, lvl);
    }
    if ctx.init.gb_is_hellfire && item._iMiscId == IMISC_OILOF {
        get_oil_type(ctx, item, lvl);
    }
    if item._itype != ItemType::Gold {
        return;
    }
    let itemlevel = items_get_currlevel(ctx);
    let mut rndv = match ctx.multi.sgGameInitInfo.nDifficulty {
        DIFF_NIGHTMARE => 5 * (itemlevel + 16) + ctx.rng.generate_rnd(10 * (itemlevel + 16)),
        DIFF_HELL => 5 * (itemlevel + 32) + ctx.rng.generate_rnd(10 * (itemlevel + 32)),
        _ => 5 * itemlevel + ctx.rng.generate_rnd(10 * itemlevel),
    };
    if ctx.gendung.leveltype == DungeonType::Hell {
        rndv += rndv / 8;
    }
    item._ivalue = rndv.min(GOLD_MAX_LIMIT);
    set_plr_hand_gold_curs(item);
}

/// Original: `devilution::SetupItem` (items.cpp).
// @port items.cpp|devilution::SetupItem(Item &item) sha=40287ffee5b6
pub fn setup_item(ctx: &mut Ctx, item: &mut Item) {
    let show = match ctx.players.MyPlayer {
        Some(m) => ctx.players.Players[m].pLvlLoad == 0,
        None => false,
    };
    item.set_new_animation(ctx, show);
    item._iIdentified = false;
}

/// Original: `devilution::SpawnUnique` (items.cpp). Returns the ground item index.
// @port items.cpp|devilution::SpawnUnique(_unique_items uid, Point position, std::optional<int> level , bool sendmsg , bool exactPosition) sha=ef7a8d20ae90
pub fn spawn_unique(ctx: &mut Ctx, uid: _unique_items, position: Point, level: Option<i32>, sendmsg: bool, exact_position: bool) -> Option<usize> {
    if ctx.items.ActiveItemCount as usize >= MAXITEMS {
        return None;
    }
    let ii = allocate_item(ctx) as usize;
    if exact_position && crate::inv::can_put(ctx, position) {
        ctx.items.Items[ii].position = position;
        ctx.items.dItem[position.x as usize][position.y as usize] = (ii + 1) as i8;
    } else {
        get_super_item_space(ctx, position, ii as i8);
    }
    let mut curlv = items_get_currlevel(ctx);
    let mut idx = 0usize;
    while AllItemsList[idx].iItemId != UniqueItems[uid as usize].UIItemId {
        idx += 1;
    }
    let me = my_player_copy(ctx);
    if ctx.multi.sgGameInitInfo.nDifficulty == DIFF_NORMAL {
        with_item(ctx, ii, |ctx, item| {
            get_item_attrs(ctx, item, idx as _item_indexes, curlv);
            get_unique_item(ctx, &me, item, uid);
            setup_item(ctx, item);
        });
    } else {
        if let Some(l) = level {
            curlv = l;
        }
        let unique_itype = AllItemsList[idx].itype;
        let idx2 = get_item_index_for_droppable_item(ctx, false, &|_, item| item.itype == unique_itype);
        let seed = ctx.rng.advance_rnd_seed() as u32;
        with_item(ctx, ii, |ctx, item| setup_all_items(ctx, &me, item, idx2, seed, curlv * 2, 15, true, false, false));
    }
    if sendmsg {
        let pos = ctx.items.Items[ii].position;
        { let it = ctx.items.Items[ii as usize].clone(); crate::msg::net_send_cmd_p_item(ctx, false, CMD_SPAWNITEM, pos, &it); }
    }
    Some(ii)
}

/// Original: `devilution::SpawnItem` (items.cpp). `spawn` defaults to false.
// @port items.cpp|devilution::SpawnItem(Monster &monster, Point position, bool sendmsg, bool spawn) sha=56302edbaa36
pub fn spawn_item(ctx: &mut Ctx, monster: usize, position: Point, sendmsg: bool, spawn: bool) {
    let mut onlygood = true;
    let treasure = crate::monster::monster_data(ctx, monster).treasure;
    let drops_special_treasure = treasure & T_UNIQ != 0;
    let q = &ctx.quests.Quests[Q_MUSHROOM as usize];
    let drop_brain = q._qactive == QUEST_ACTIVE && q._qvar1 as i32 == QS_MUSHGIVEN as i32;
    let is_unique = ctx.monster.Monsters[monster].is_unique();
    let idx;
    if drops_special_treasure && !crate::quests::use_multiplayer_quests(ctx) {
        let unique_item = spawn_unique(ctx, (treasure & T_MASK) as _unique_items, position, None, false, false);
        if let Some(u) = unique_item {
            if sendmsg {
                let pos = ctx.items.Items[u].position;
                { let it = ctx.items.Items[u as usize].clone(); crate::msg::net_send_cmd_p_item(ctx, false, CMD_DROPITEM, pos, &it); }
            }
        }
        return;
    } else if is_unique || drops_special_treasure {
        idx = rnd_u_item(ctx, Some(monster));
    } else if drop_brain && !ctx.init.gb_is_multiplayer {
        ctx.quests.Quests[Q_MUSHROOM as usize]._qvar1 = QS_BRAINSPAWNED as u8;
        crate::msg::net_send_cmd_quest(ctx, true, Q_MUSHROOM as usize);
        idx = IDI_BRAIN;
    } else {
        if drop_brain && ctx.init.gb_is_multiplayer && sendmsg {
            ctx.quests.Quests[Q_MUSHROOM as usize]._qvar1 = QS_BRAINSPAWNED as u8;
            crate::msg::net_send_cmd_quest(ctx, true, Q_MUSHROOM as usize);
            let pos_brain = get_super_item_loc(ctx, position);
            spawn_quest_item(ctx, IDI_BRAIN, pos_brain, 0, 0, true);
        }
        if treasure & T_NODROP != 0 {
            return;
        }
        onlygood = false;
        let lvl = crate::monster::monster_level(ctx, monster, ctx.multi.sgGameInitInfo.nDifficulty);
        idx = rnd_item_for_monster_level(ctx, lvl as i8);
    }
    if idx == IDI_NONE {
        return;
    }
    if ctx.items.ActiveItemCount as usize >= MAXITEMS {
        return;
    }
    let ii = allocate_item(ctx) as usize;
    get_super_item_space(ctx, position, ii as i8);
    let uper = if is_unique { 15 } else { 1 };
    let mut m_level = crate::monster::monster_data(ctx, monster).level;
    if !ctx.init.gb_is_hellfire && crate::monster::monster_type_id(ctx, monster) == MT_DIABLO {
        m_level -= 15;
    }
    let me = my_player_copy(ctx);
    let seed = ctx.rng.advance_rnd_seed() as u32;
    with_item(ctx, ii, |ctx, item| setup_all_items(ctx, &me, item, idx, seed, m_level as i32, uper, onlygood, false, false));
    let pos = ctx.items.Items[ii].position;
    if sendmsg {
        { let it = ctx.items.Items[ii as usize].clone(); crate::msg::net_send_cmd_p_item(ctx, false, CMD_DROPITEM, pos, &it); }
    }
    if spawn {
        { let it = ctx.items.Items[ii as usize].clone(); crate::msg::net_send_cmd_p_item(ctx, false, CMD_SPAWNITEM, pos, &it); }
    }
}

/// Original: `devilution::CreateRndItem` (items.cpp).
// @port items.cpp|devilution::CreateRndItem(Point position, bool onlygood, bool sendmsg, bool delta) sha=f595379376f1
pub fn create_rnd_item(ctx: &mut Ctx, position: Point, onlygood: bool, sendmsg: bool, delta: bool) {
    let idx = if onlygood { rnd_u_item(ctx, None) } else { rnd_all_items(ctx) };
    setup_base_item(ctx, position, idx, onlygood, sendmsg, delta, false);
}

/// Original: `devilution::CreateRndUseful` (items.cpp).
// @port items.cpp|devilution::CreateRndUseful(Point position, bool sendmsg) sha=f8fe8d5fd992
pub fn create_rnd_useful(ctx: &mut Ctx, position: Point, sendmsg: bool) {
    if ctx.items.ActiveItemCount as usize >= MAXITEMS {
        return;
    }
    let ii = allocate_item(ctx) as usize;
    get_super_item_space(ctx, position, ii as i8);
    let curlv = items_get_currlevel(ctx);
    let seed = ctx.rng.advance_rnd_seed();
    with_item(ctx, ii, |ctx, item| setup_all_useful(ctx, item, seed, curlv));
    if sendmsg {
        let pos = ctx.items.Items[ii].position;
        { let it = ctx.items.Items[ii as usize].clone(); crate::msg::net_send_cmd_p_item(ctx, false, CMD_DROPITEM, pos, &it); }
    }
}

/// Original: `devilution::CreateTypeItem` (items.cpp). `spawn` defaults to false.
// @port items.cpp|devilution::CreateTypeItem(Point position, bool onlygood, ItemType itemType, int imisc, bool sendmsg, bool delta, bool spawn) sha=44289a9e8b67
#[allow(clippy::too_many_arguments)]
pub fn create_type_item(ctx: &mut Ctx, position: Point, onlygood: bool, item_type: ItemType, imisc: i32, sendmsg: bool, delta: bool, spawn: bool) {
    let curlv = items_get_currlevel(ctx);
    let idx = if item_type != ItemType::Gold { rnd_type_items(ctx, item_type, imisc, curlv) } else { IDI_GOLD };
    setup_base_item(ctx, position, idx, onlygood, sendmsg, delta, spawn);
}

/// Original: `devilution::RecreateItem` (items.cpp).
// @port items.cpp|devilution::RecreateItem(const Player &player, Item &item, _item_indexes idx, uint16_t icreateinfo, uint32_t iseed, int ivalue, bool isHellfire) sha=15027c1fb621
#[allow(clippy::too_many_arguments)]
pub fn recreate_item(ctx: &mut Ctx, player: &Player, item: &mut Item, idx: _item_indexes, icreateinfo: u16, iseed: u32, ivalue: i32, is_hellfire: bool) {
    let tmp_is_hellfire = ctx.init.gb_is_hellfire;
    ctx.init.gb_is_hellfire = is_hellfire;
    let ci = icreateinfo as i32;
    if idx == IDI_GOLD {
        initialize_item(ctx, item, IDI_GOLD);
        item._iSeed = iseed;
        item._iCreateInfo = icreateinfo;
        item._ivalue = ivalue;
        set_plr_hand_gold_curs(item);
        ctx.init.gb_is_hellfire = tmp_is_hellfire;
        return;
    }
    if icreateinfo == 0 {
        initialize_item(ctx, item, idx);
        item._iSeed = iseed;
        ctx.init.gb_is_hellfire = tmp_is_hellfire;
        return;
    }
    if ci & CF_UNIQUE == 0 {
        if ci & CF_TOWN != 0 {
            recreate_town_item(ctx, player, item, idx, icreateinfo, iseed as i32);
            ctx.init.gb_is_hellfire = tmp_is_hellfire;
            return;
        }
        if ci & CF_USEFUL == CF_USEFUL {
            setup_all_useful(ctx, item, iseed as i32, ci & CF_LEVEL);
            ctx.init.gb_is_hellfire = tmp_is_hellfire;
            return;
        }
    }
    let level = ci & CF_LEVEL;
    let mut uper = 0;
    if ci & CF_UPER1 != 0 {
        uper = 1;
    }
    if ci & CF_UPER15 != 0 {
        uper = 15;
    }
    let onlygood = ci & CF_ONLYGOOD != 0;
    let recreate = ci & CF_UNIQUE != 0;
    let pregen = ci & CF_PREGEN != 0;
    setup_all_items(ctx, player, item, idx, iseed, level, uper, onlygood, recreate, pregen);
    ctx.init.gb_is_hellfire = tmp_is_hellfire;
}

/// Original: `devilution::RecreateEar` (items.cpp).
// @port items.cpp|devilution::RecreateEar(Item &item, uint16_t ic, uint32_t iseed, uint8_t bCursval, string_view heroName) sha=dadb9f472bea
pub fn recreate_ear(ctx: &Ctx, item: &mut Item, ic: u16, iseed: u32, b_cursval: u8, hero_name: &str) {
    initialize_item(ctx, item, IDI_EAR);
    let item_name = format!("Ear of {hero_name}");
    item._iName.set(&item_name);
    item._iIName.set(hero_name);
    item._iCurs = ((b_cursval >> 6) & 3) + ICURS_EAR_SORCERER as u8;
    item._ivalue = (b_cursval & 0x3F) as i32;
    item._iCreateInfo = ic;
    item._iSeed = iseed;
}

/// Original: `devilution::CornerstoneSave` (items.cpp).
// @port items.cpp|devilution::CornerstoneSave() sha=8033ddaf0284
pub fn cornerstone_save(ctx: &mut Ctx) {
    if !ctx.items.CornerStone.activated {
        return;
    }
    if !ctx.items.CornerStone.item.is_empty() {
        let item = ctx.items.CornerStone.item.clone();
        let id = crate::pack::pack_item(ctx, &item, item.dwBuff & CF_HELLFIRE as u32 != 0);
        let mut s = String::new();
        for b in id.to_bytes() {
            s.push_str(&format!("{b:02X}"));
        }
        ctx.options.hellfire.sz_item = s;
    } else {
        ctx.options.hellfire.sz_item.clear();
    }
}

/// Original: `devilution::CornerstoneLoad` (items.cpp).
// @port items.cpp|devilution::CornerstoneLoad(Point position) sha=d217938d7b0d
pub fn cornerstone_load(ctx: &mut Ctx, position: Point) {
    if ctx.items.CornerStone.activated || position.x == 0 || position.y == 0 {
        return;
    }
    ctx.items.CornerStone.item.clear();
    ctx.items.CornerStone.activated = true;
    let d = ctx.items.dItem[position.x as usize][position.y as usize];
    if d != 0 {
        let ii = (d - 1) as u8;
        for i in 0..ctx.items.ActiveItemCount as usize {
            if ctx.items.ActiveItems[i] == ii {
                delete_item(ctx, i as i32);
                break;
            }
        }
        ctx.items.dItem[position.x as usize][position.y as usize] = 0;
    }
    let sz = ctx.options.hellfire.sz_item.clone();
    let pack_size = crate::pack::ITEM_PACK_SIZE;
    if sz.len() < pack_size * 2 {
        return;
    }
    let mut raw = vec![0u8; pack_size];
    hex2bin(sz.as_bytes(), pack_size, &mut raw);
    let pk_s_item = crate::pack::ItemPack::from_bytes(&raw);
    let ii = allocate_item(ctx) as usize;
    ctx.items.dItem[position.x as usize][position.y as usize] = (ii + 1) as i8;
    let me = my_player_copy(ctx);
    with_item(ctx, ii, |ctx, item| {
        crate::pack::unpack_item(ctx, &pk_s_item, &me, item, pk_s_item.dwBuff & CF_HELLFIRE as u32 != 0);
        item.position = position;
        respawn_item(ctx, item, false);
    });
    ctx.items.CornerStone.item = ctx.items.Items[ii].clone();
}

/// Original: `devilution::SpawnQuestItem` (items.cpp).
// @port items.cpp|devilution::SpawnQuestItem(_item_indexes itemid, Point position, int randarea, int selflag, bool sendmsg) sha=96824ce770ac
pub fn spawn_quest_item(ctx: &mut Ctx, itemid: _item_indexes, mut position: Point, mut randarea: i32, selflag: i32, sendmsg: bool) {
    if randarea > 0 {
        let mut tries = 0;
        loop {
            tries += 1;
            if tries > 1000 && randarea > 1 {
                randarea -= 1;
            }
            position.x = ctx.rng.generate_rnd(MAXDUNX as i32);
            position.y = ctx.rng.generate_rnd(MAXDUNY as i32);
            let mut failed = false;
            'outer: for i in 0..randarea {
                for j in 0..randarea {
                    failed = !item_space_ok(ctx, position + Displacement::new(i, j));
                    if failed {
                        break 'outer;
                    }
                }
            }
            if !failed {
                break;
            }
        }
    }
    if ctx.items.ActiveItemCount as usize >= MAXITEMS {
        return;
    }
    let ii = allocate_item(ctx) as usize;
    ctx.items.Items[ii].position = position;
    ctx.items.dItem[position.x as usize][position.y as usize] = (ii + 1) as i8;
    let curlv = items_get_currlevel(ctx);
    with_item(ctx, ii, |ctx, item| {
        get_item_attrs(ctx, item, itemid, curlv);
        setup_item(ctx, item);
        item._iSeed = ctx.rng.advance_rnd_seed() as u32;
        ctx.rng.set_rnd_seed(item._iSeed);
        item._iPostDraw = true;
        if selflag != 0 {
            item._iSelFlag = selflag as u8;
            item.AnimInfo.currentFrame = item.AnimInfo.numberOfFrames - 1;
            item._iAnimFlag = false;
        }
    });
    if sendmsg {
        let pos = ctx.items.Items[ii].position;
        { let it = ctx.items.Items[ii as usize].clone(); crate::msg::net_send_cmd_p_item(ctx, true, CMD_SPAWNITEM, pos, &it); }
    } else {
        ctx.items.Items[ii]._iCreateInfo |= CF_PREGEN as u16;
        crate::msg::delta_add_item(ctx, ii as i32);
    }
}

/// Original: `devilution::SpawnRewardItem` (items.cpp).
// @port items.cpp|devilution::SpawnRewardItem(_item_indexes itemid, Point position, bool sendmsg) sha=a3d51c3dcae2
pub fn spawn_reward_item(ctx: &mut Ctx, itemid: _item_indexes, position: Point, sendmsg: bool) {
    if ctx.items.ActiveItemCount as usize >= MAXITEMS {
        return;
    }
    let ii = allocate_item(ctx) as usize;
    ctx.items.Items[ii].position = position;
    ctx.items.dItem[position.x as usize][position.y as usize] = (ii + 1) as i8;
    let curlv = items_get_currlevel(ctx);
    with_item(ctx, ii, |ctx, item| {
        get_item_attrs(ctx, item, itemid, curlv);
        item.set_new_animation(ctx, true);
        item._iSelFlag = 2;
        item._iPostDraw = true;
        item._iIdentified = true;
        generate_new_seed(ctx, item);
    });
    if sendmsg {
        let pos = ctx.items.Items[ii].position;
        { let it = ctx.items.Items[ii as usize].clone(); crate::msg::net_send_cmd_p_item(ctx, true, CMD_SPAWNITEM, pos, &it); }
    }
}

/// Original: `devilution::SpawnMapOfDoom` (items.cpp).
// @port items.cpp|devilution::SpawnMapOfDoom(Point position, bool sendmsg) sha=0d5490776209
pub fn spawn_map_of_doom(ctx: &mut Ctx, position: Point, sendmsg: bool) {
    spawn_reward_item(ctx, IDI_MAPOFDOOM, position, sendmsg);
}

/// Original: `devilution::SpawnRuneBomb` (items.cpp).
// @port items.cpp|devilution::SpawnRuneBomb(Point position, bool sendmsg) sha=5686b7993a89
pub fn spawn_rune_bomb(ctx: &mut Ctx, position: Point, sendmsg: bool) {
    spawn_reward_item(ctx, IDI_RUNEBOMB, position, sendmsg);
}

/// Original: `devilution::SpawnTheodore` (items.cpp).
// @port items.cpp|devilution::SpawnTheodore(Point position, bool sendmsg) sha=4efbf6c9db62
pub fn spawn_theodore(ctx: &mut Ctx, position: Point, sendmsg: bool) {
    spawn_reward_item(ctx, IDI_THEODORE, position, sendmsg);
}

/// Original: `devilution::RespawnItem` (items.cpp).
// @port items.cpp|devilution::RespawnItem(Item &item, bool flipFlag) sha=913ee5cf6828
pub fn respawn_item(ctx: &mut Ctx, item: &mut Item, flip_flag: bool) {
    let it = ItemCAnimTbl[item._iCurs as usize];
    item.set_new_animation(ctx, flip_flag);
    item._iRequest = false;
    let c = item._iCurs as item_cursor_graphic;
    if matches!(c, ICURS_MAGIC_ROCK | ICURS_TAVERN_SIGN | ICURS_ANVIL_OF_FURY) {
        item._iSelFlag = 1;
    } else if matches!(c, ICURS_MAP_OF_THE_STARS | ICURS_RUNE_BOMB | ICURS_THEODORE | ICURS_AURIC_AMULET) {
        item._iSelFlag = 2;
    }
    if c == ICURS_MAGIC_ROCK {
        crate::effects::play_sfx_loc(ctx, ItemDropSnds[it as usize], item.position, true);
    }
}

/// Original: `devilution::DeleteItem` (items.cpp).
// @port items.cpp|devilution::DeleteItem(int i) sha=88a384d601b0
pub fn delete_item(ctx: &mut Ctx, i: i32) {
    if ctx.items.ActiveItemCount > 0 {
        ctx.items.ActiveItemCount -= 1;
    }
    assert!(i >= 0 && (i as usize) < MAXITEMS && (ctx.items.ActiveItemCount as usize) < MAXITEMS);
    if ctx.cursor.pcursitem == ctx.items.ActiveItems[i as usize] as i8 {
        ctx.cursor.pcursitem = -1;
    }
    let n = ctx.items.ActiveItemCount as usize;
    if (i as usize) < n {
        ctx.items.ActiveItems.swap(i as usize, n);
    }
}

/// Original: `devilution::ProcessItems` (items.cpp).
// @port items.cpp|devilution::ProcessItems() sha=754807b8b429
pub fn process_items(ctx: &mut Ctx) {
    let mut i = 0;
    while i < ctx.items.ActiveItemCount as usize {
        let ii = ctx.items.ActiveItems[i] as usize;
        if !ctx.items.Items[ii]._iAnimFlag {
            i += 1;
            continue;
        }
        ctx.items.Items[ii].AnimInfo.process_animation(false);
        let item = &mut ctx.items.Items[ii];
        if item._iCurs as item_cursor_graphic == ICURS_MAGIC_ROCK {
            if item._iSelFlag == 1 && item.AnimInfo.currentFrame == 10 {
                item.AnimInfo.currentFrame = 0;
            }
            if item._iSelFlag == 2 && item.AnimInfo.currentFrame == 20 {
                item.AnimInfo.currentFrame = 10;
            }
        } else {
            if item.AnimInfo.currentFrame as i32 == (item.AnimInfo.numberOfFrames as i32 - 1) / 2 {
                let sfx = ItemDropSnds[ItemCAnimTbl[item._iCurs as usize] as usize];
                let pos = item.position;
                crate::effects::play_sfx_loc(ctx, sfx, pos, true);
            }
            let item = &mut ctx.items.Items[ii];
            if item.AnimInfo.is_last_frame() {
                item.AnimInfo.currentFrame = item.AnimInfo.numberOfFrames - 1;
                item._iAnimFlag = false;
                item._iSelFlag = 1;
            }
        }
        i += 1;
    }
    item_doppel(ctx);
}

/// Original: `devilution::FreeItemGFX` (items.cpp).
// @port items.cpp|devilution::FreeItemGFX() sha=ff3c6c606a44
pub fn free_item_gfx(ctx: &mut Ctx) {
    for itemanim in ctx.items.itemanims.iter_mut() {
        *itemanim = None;
    }
}

/// Original: `devilution::GetItemFrm` (items.cpp).
// @port items.cpp|devilution::GetItemFrm(Item &item) sha=89020c4f1bd0
pub fn get_item_frm(ctx: &Ctx, item: &mut Item) {
    let it = ItemCAnimTbl[item._iCurs as usize] as usize;
    if let Some(a) = &ctx.items.itemanims[it] {
        item.AnimInfo.sprites = Some(a.clone());
    }
}

/// Original: `devilution::GetItemStr` (items.cpp).
// @port items.cpp|devilution::GetItemStr(Item &item) sha=b8cb79f5bf97
pub fn get_item_str(ctx: &mut Ctx, item: &Item) {
    if item._itype != ItemType::Gold {
        let name = item.get_name(ctx);
        ctx.control.info_string = name;
        ctx.control.info_color = item.get_text_color();
    } else {
        let n_gold = item._ivalue;
        ctx.control.info_string = ngettext("{:s} gold piece", "{:s} gold pieces", n_gold).replacen("{:s}", &crate::utils::format_int::format_integer(n_gold), 1);
    }
}

/// `cii >= NUM_INVLOC ? &InvList[cii - NUM_INVLOC] : &InvBody[cii]`
pub fn inv_item_mut(player: &mut Player, cii: i32) -> &mut Item {
    if cii >= NUM_INVLOC as i32 {
        &mut player.InvList[(cii - NUM_INVLOC as i32) as usize]
    } else {
        &mut player.InvBody[cii as usize]
    }
}

/// Original: `devilution::CheckIdentify` (items.cpp).
// @port items.cpp|devilution::CheckIdentify(Player &player, int cii) sha=cacca1d50b40
pub fn check_identify(ctx: &mut Ctx, pnum: usize, cii: i32) {
    inv_item_mut(&mut ctx.players.Players[pnum], cii)._iIdentified = true;
    calc_plr_inv(ctx, pnum, true);
}

/// Original: `devilution::DoRepair` (items.cpp).
// @port items.cpp|devilution::DoRepair(Player &player, int cii) sha=475451427ced
pub fn do_repair(ctx: &mut Ctx, pnum: usize, cii: i32) {
    let pos = ctx.players.Players[pnum].position.tile;
    crate::effects::play_sfx_loc(ctx, crate::effects::IS_REPAIR, pos, true);
    let lvl = ctx.players.Players[pnum]._pLevel as i32;
    let mut item = std::mem::take(inv_item_mut(&mut ctx.players.Players[pnum], cii));
    repair_item(ctx, &mut item, lvl);
    *inv_item_mut(&mut ctx.players.Players[pnum], cii) = item;
    calc_plr_inv(ctx, pnum, true);
}

/// Original: `devilution::DoRecharge` (items.cpp).
// @port items.cpp|devilution::DoRecharge(Player &player, int cii) sha=4cbd62dcdcb9
pub fn do_recharge(ctx: &mut Ctx, pnum: usize, cii: i32) {
    recharge_item(ctx, pnum, cii);
    calc_plr_inv(ctx, pnum, true);
}

/// Original: `devilution::DoOil` (items.cpp).
// @port items.cpp|devilution::DoOil(Player &player, int cii) sha=d82cb939d916
pub fn do_oil(ctx: &mut Ctx, pnum: usize, cii: i32) -> bool {
    let oil = ctx.players.Players[pnum]._pOilType;
    let mut item = std::mem::take(inv_item_mut(&mut ctx.players.Players[pnum], cii));
    let ok = apply_oil_to_item(ctx, &mut item, oil);
    *inv_item_mut(&mut ctx.players.Players[pnum], cii) = item;
    if !ok {
        return false;
    }
    calc_plr_inv(ctx, pnum, true);
    true
}

/// `fmt` with `{:+d}`: a signed number with an explicit sign.
fn plus(v: i32) -> String {
    format!("{v:+}")
}

/// Original: `devilution::PrintItemPower` (items.cpp).
// @port items.cpp|devilution::PrintItemPower(char plidx, const Item &item) sha=2f070c8a6c5f
pub fn print_item_power(plidx: item_effect_type, item: &Item) -> String {
    let f1 = |fmt: &str, a: String| tr(fmt).replacen(fmt_arg(fmt), &a, 1);
    use ItemSpecialEffect as E;
    let max_res = crate::player::MaxResistance;
    match plidx {
        IPL_TOHIT | IPL_TOHIT_CURSE => f1("chance to hit: {:+d}%", plus(item._iPLToHit as i32)),
        IPL_DAMP | IPL_DAMP_CURSE => f1("{:+d}% damage", plus(item._iPLDam as i32)),
        IPL_TOHIT_DAMP | IPL_TOHIT_DAMP_CURSE | IPL_DOPPELGANGER => {
            tr("to hit: {:+d}%, {:+d}% damage").replacen("{:+d}", &plus(item._iPLToHit as i32), 1).replacen("{:+d}", &plus(item._iPLDam as i32), 1)
        }
        IPL_ACP | IPL_ACP_CURSE => f1("{:+d}% armor", plus(item._iPLAC as i32)),
        IPL_SETAC | IPL_AC_CURSE => f1("armor class: {:d}", item._iAC.to_string()),
        IPL_FIRERES | IPL_FIRERES_CURSE => {
            if (item._iPLFR as i32) < max_res {
                f1("Resist Fire: {:+d}%", plus(item._iPLFR as i32))
            } else {
                f1("Resist Fire: {:+d}% MAX", plus(max_res))
            }
        }
        IPL_LIGHTRES | IPL_LIGHTRES_CURSE => {
            if (item._iPLLR as i32) < max_res {
                f1("Resist Lightning: {:+d}%", plus(item._iPLLR as i32))
            } else {
                f1("Resist Lightning: {:+d}% MAX", plus(max_res))
            }
        }
        IPL_MAGICRES | IPL_MAGICRES_CURSE => {
            if (item._iPLMR as i32) < max_res {
                f1("Resist Magic: {:+d}%", plus(item._iPLMR as i32))
            } else {
                f1("Resist Magic: {:+d}% MAX", plus(max_res))
            }
        }
        IPL_ALLRES => {
            if (item._iPLFR as i32) < max_res {
                f1("Resist All: {:+d}%", plus(item._iPLFR as i32))
            } else {
                f1("Resist All: {:+d}% MAX", plus(max_res))
            }
        }
        IPL_SPLLVLADD => {
            let v = item._iSplLvlAdd as i32;
            if v > 0 {
                ngettext("spells are increased {:d} level", "spells are increased {:d} levels", v).replacen("{:d}", &v.to_string(), 1)
            } else if v < 0 {
                ngettext("spells are decreased {:d} level", "spells are decreased {:d} levels", -v).replacen("{:d}", &(-v).to_string(), 1)
            } else {
                tr("spell levels unchanged (?)")
            }
        }
        IPL_CHARGES => tr("Extra charges"),
        IPL_SPELL => ngettext("{:d} {:s} charge", "{:d} {:s} charges", item._iMaxCharges)
            .replacen("{:d}", &item._iMaxCharges.to_string(), 1)
            .replacen("{:s}", &pgettext("spell", get_spell_data(item._iSpell).sNameText), 1),
        IPL_FIREDAM => {
            if item._iFMinDam == item._iFMaxDam {
                f1("Fire hit damage: {:d}", item._iFMinDam.to_string())
            } else {
                tr("Fire hit damage: {:d}-{:d}").replacen("{:d}", &item._iFMinDam.to_string(), 1).replacen("{:d}", &item._iFMaxDam.to_string(), 1)
            }
        }
        IPL_LIGHTDAM => {
            if item._iLMinDam == item._iLMaxDam {
                f1("Lightning hit damage: {:d}", item._iLMinDam.to_string())
            } else {
                tr("Lightning hit damage: {:d}-{:d}").replacen("{:d}", &item._iLMinDam.to_string(), 1).replacen("{:d}", &item._iLMaxDam.to_string(), 1)
            }
        }
        IPL_STR | IPL_STR_CURSE => f1("{:+d} to strength", plus(item._iPLStr as i32)),
        IPL_MAG | IPL_MAG_CURSE => f1("{:+d} to magic", plus(item._iPLMag as i32)),
        IPL_DEX | IPL_DEX_CURSE => f1("{:+d} to dexterity", plus(item._iPLDex as i32)),
        IPL_VIT | IPL_VIT_CURSE => f1("{:+d} to vitality", plus(item._iPLVit as i32)),
        IPL_ATTRIBS | IPL_ATTRIBS_CURSE => f1("{:+d} to all attributes", plus(item._iPLStr as i32)),
        IPL_GETHIT_CURSE | IPL_GETHIT => f1("{:+d} damage from enemies", plus(item._iPLGetHit as i32)),
        IPL_LIFE | IPL_LIFE_CURSE => f1("Hit Points: {:+d}", plus(item._iPLHP as i32 >> 6)),
        IPL_MANA | IPL_MANA_CURSE => f1("Mana: {:+d}", plus(item._iPLMana as i32 >> 6)),
        IPL_DUR => tr("high durability"),
        IPL_DUR_CURSE => tr("decreased durability"),
        IPL_INDESTRUCTIBLE => tr("indestructible"),
        IPL_LIGHT => f1("+{:d}% light radius", (10 * item._iPLLight as i32).to_string()),
        IPL_LIGHT_CURSE => f1("-{:d}% light radius", (-10 * item._iPLLight as i32).to_string()),
        IPL_MULT_ARROWS => tr("multiple arrows per shot"),
        IPL_FIRE_ARROWS => {
            if item._iFMinDam == item._iFMaxDam {
                f1("fire arrows damage: {:d}", item._iFMinDam.to_string())
            } else {
                tr("fire arrows damage: {:d}-{:d}").replacen("{:d}", &item._iFMinDam.to_string(), 1).replacen("{:d}", &item._iFMaxDam.to_string(), 1)
            }
        }
        IPL_LIGHT_ARROWS => {
            if item._iLMinDam == item._iLMaxDam {
                f1("lightning arrows damage {:d}", item._iLMinDam.to_string())
            } else {
                tr("lightning arrows damage {:d}-{:d}").replacen("{:d}", &item._iLMinDam.to_string(), 1).replacen("{:d}", &item._iLMaxDam.to_string(), 1)
            }
        }
        IPL_FIREBALL => {
            if item._iFMinDam == item._iFMaxDam {
                f1("fireball damage: {:d}", item._iFMinDam.to_string())
            } else {
                tr("fireball damage: {:d}-{:d}").replacen("{:d}", &item._iFMinDam.to_string(), 1).replacen("{:d}", &item._iFMaxDam.to_string(), 1)
            }
        }
        IPL_THORNS => tr("attacker takes 1-3 damage"),
        IPL_NOMANA => tr("user loses all mana"),
        IPL_ABSHALFTRAP => tr("absorbs half of trap damage"),
        IPL_KNOCKBACK => tr("knocks target back"),
        IPL_3XDAMVDEM => tr("+200% damage vs. demons"),
        IPL_ALLRESZERO => tr("All Resistance equals 0"),
        IPL_STEALMANA => {
            if item._iFlags.has_any_of(E::StealMana3) {
                tr("hit steals 3% mana")
            } else if item._iFlags.has_any_of(E::StealMana5) {
                tr("hit steals 5% mana")
            } else {
                String::new()
            }
        }
        IPL_STEALLIFE => {
            if item._iFlags.has_any_of(E::StealLife3) {
                tr("hit steals 3% life")
            } else if item._iFlags.has_any_of(E::StealLife5) {
                tr("hit steals 5% life")
            } else {
                String::new()
            }
        }
        IPL_TARGAC => tr("penetrates target's armor"),
        IPL_FASTATTACK => {
            if item._iFlags.has_any_of(E::QuickAttack) {
                tr("quick attack")
            } else if item._iFlags.has_any_of(E::FastAttack) {
                tr("fast attack")
            } else if item._iFlags.has_any_of(E::FasterAttack) {
                tr("faster attack")
            } else if item._iFlags.has_any_of(E::FastestAttack) {
                tr("fastest attack")
            } else {
                tr("Another ability (NW)")
            }
        }
        IPL_FASTRECOVER => {
            if item._iFlags.has_any_of(E::FastHitRecovery) {
                tr("fast hit recovery")
            } else if item._iFlags.has_any_of(E::FasterHitRecovery) {
                tr("faster hit recovery")
            } else if item._iFlags.has_any_of(E::FastestHitRecovery) {
                tr("fastest hit recovery")
            } else {
                tr("Another ability (NW)")
            }
        }
        IPL_FASTBLOCK => tr("fast block"),
        IPL_DAMMOD => ngettext("adds {:d} point to damage", "adds {:d} points to damage", item._iPLDamMod as i32).replacen("{:d}", &item._iPLDamMod.to_string(), 1),
        IPL_RNDARROWVEL => tr("fires random speed arrows"),
        IPL_SETDAM => tr("unusual item damage"),
        IPL_SETDUR => tr("altered durability"),
        IPL_ONEHAND => tr("one handed sword"),
        IPL_DRAINLIFE => tr("constantly lose hit points"),
        IPL_RNDSTEALLIFE => tr("life stealing"),
        IPL_NOMINSTR => tr("no strength requirement"),
        IPL_INVCURS => " ".to_string(),
        IPL_ADDACLIFE => {
            if item._iFMinDam == item._iFMaxDam {
                f1("lightning damage: {:d}", item._iFMinDam.to_string())
            } else {
                tr("lightning damage: {:d}-{:d}").replacen("{:d}", &item._iFMinDam.to_string(), 1).replacen("{:d}", &item._iFMaxDam.to_string(), 1)
            }
        }
        IPL_ADDMANAAC => tr("charged bolts on hits"),
        IPL_DEVASTATION => tr("occasional triple damage"),
        IPL_DECAY => f1("decaying {:+d}% damage", plus(item._iPLDam as i32)),
        IPL_PERIL => tr("2x dmg to monst, 1x to you"),
        IPL_JESTERS => tr("Random 0 - 600% damage"),
        IPL_CRYSTALLINE => f1("low dur, {:+d}% damage", plus(item._iPLDam as i32)),
        IPL_ACDEMON => tr("extra AC vs demons"),
        IPL_ACUNDEAD => tr("extra AC vs undead"),
        IPL_MANATOLIFE => tr("50% Mana moved to Health"),
        IPL_LIFETOMANA => tr("40% Health moved to Mana"),
        _ => tr("Another ability (NW)"),
    }
}

/// The format placeholder used by a single-argument `PrintItemPower` string.
fn fmt_arg(fmt: &str) -> &'static str {
    if fmt.contains("{:+d}") {
        "{:+d}"
    } else {
        "{:d}"
    }
}

/// Original: `devilution::DrawUniqueInfo` (items.cpp).
// @port items.cpp|devilution::DrawUniqueInfo(const Surface &out) sha=f1ccd89d1154
pub fn draw_unique_info(ctx: &mut Ctx, out: &crate::engine::surface::Surface) {
    use crate::engine::render::text_render::draw_string;
    use crate::engine::surface::Rect;
    let right = crate::control::get_right_panel(ctx);
    let position = Point::new(right.x - crate::control::SIDE_PANEL_SIZE.0, right.y);
    if crate::control::is_left_panel_open(ctx) && crate::control::get_left_panel(ctx).contains(position) {
        return;
    }

    draw_unique_info_window(ctx, out);

    let mut rect = Rect::new(position.x + 32, position.y + 56, 257, 0);
    let uitem = &UniqueItems[ctx.items.curruitem._iUid as usize];
    draw_string(ctx, out, &tr(uitem.UIName), rect, UiFlags::ALIGN_CENTER, 1, -1);

    let divider = Rect::new(position.x + 26, position.y + 25, 267, 3);
    out.blit_from(out, divider, (divider.x, divider.y + 5 * 12 + 13));

    rect.y += (10 - uitem.UINumPL as i32) * 12;
    assert!(uitem.UINumPL as usize <= uitem.powers.len());
    let curruitem = ctx.items.curruitem.clone();
    for power in uitem.powers.iter() {
        if power.type_ == IPL_INVALID {
            break;
        }
        rect.y += 2 * 12;
        let s = print_item_power(power.type_, &curruitem);
        draw_string(ctx, out, &s, rect, UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, 1, -1);
    }
}

/// Original: `devilution::PrintItemDetails` (items.cpp).
// @port items.cpp|devilution::PrintItemDetails(const Item &item) sha=b746923962e2
pub fn print_item_details(ctx: &mut Ctx, item: &Item) {
    if ctx.diablo.headless_mode {
        return;
    }
    print_item_dur_lines(ctx, item, true);
    if item._iPrePower != -1 {
        let s = print_item_power(item._iPrePower, item);
        crate::control::add_panel_string(ctx, &s);
    }
    if item._iSufPower != -1 {
        let s = print_item_power(item._iSufPower, item);
        crate::control::add_panel_string(ctx, &s);
    }
    if item._iMagical == ITEM_QUALITY_UNIQUE {
        crate::control::add_panel_string(ctx, &tr("unique item"));
        ctx.items.ShowUniqueItemInfoBox = true;
        ctx.items.curruitem = item.clone();
    }
    print_item_info(ctx, item);
}

/// The weapon/armor/charges lines shared by `PrintItemDetails` (identified) and `PrintItemDur`.
fn print_item_dur_lines(ctx: &mut Ctx, item: &Item, details: bool) {
    let add = |ctx: &mut Ctx, s: String| crate::control::add_panel_string(ctx, &s);
    let d = |v: i32| v.to_string();
    if item._iClass == ICLASS_WEAPON {
        if item._iMinDam == item._iMaxDam {
            if item._iMaxDur == DUR_INDESTRUCTIBLE {
                add(ctx, tr("damage: {:d}  Indestructible").replacen("{:d}", &d(item._iMinDam as i32), 1));
            } else {
                add(
                    ctx,
                    tr("damage: {:d}  Dur: {:d}/{:d}")
                        .replacen("{:d}", &d(item._iMinDam as i32), 1)
                        .replacen("{:d}", &d(item._iDurability), 1)
                        .replacen("{:d}", &d(item._iMaxDur), 1),
                );
            }
        } else if item._iMaxDur == DUR_INDESTRUCTIBLE {
            add(ctx, tr("damage: {:d}-{:d}  Indestructible").replacen("{:d}", &d(item._iMinDam as i32), 1).replacen("{:d}", &d(item._iMaxDam as i32), 1));
        } else {
            add(
                ctx,
                tr("damage: {:d}-{:d}  Dur: {:d}/{:d}")
                    .replacen("{:d}", &d(item._iMinDam as i32), 1)
                    .replacen("{:d}", &d(item._iMaxDam as i32), 1)
                    .replacen("{:d}", &d(item._iDurability), 1)
                    .replacen("{:d}", &d(item._iMaxDur), 1),
            );
        }
        if !details {
            if item._iMiscId == IMISC_STAFF && item._iMaxCharges > 0 {
                add(ctx, tr("Charges: {:d}/{:d}").replacen("{:d}", &d(item._iCharges), 1).replacen("{:d}", &d(item._iMaxCharges), 1));
            }
            if item._iMagical != ITEM_QUALITY_NORMAL {
                add(ctx, tr("Not Identified"));
            }
        }
    }
    if item._iClass == ICLASS_ARMOR {
        if item._iMaxDur == DUR_INDESTRUCTIBLE {
            add(ctx, tr("armor: {:d}  Indestructible").replacen("{:d}", &d(item._iAC as i32), 1));
        } else {
            add(
                ctx,
                tr("armor: {:d}  Dur: {:d}/{:d}").replacen("{:d}", &d(item._iAC as i32), 1).replacen("{:d}", &d(item._iDurability), 1).replacen("{:d}", &d(item._iMaxDur), 1),
            );
        }
        if !details {
            if item._iMagical != ITEM_QUALITY_NORMAL {
                add(ctx, tr("Not Identified"));
            }
            if item._iMiscId == IMISC_STAFF && item._iMaxCharges > 0 {
                add(ctx, tr("Charges: {:d}/{:d}").replacen("{:d}", &d(item._iCharges), 1).replacen("{:d}", &d(item._iMaxCharges), 1));
            }
        }
    }
    if details && item._iMiscId == IMISC_STAFF && item._iMaxCharges != 0 {
        add(ctx, tr("Charges: {:d}/{:d}").replacen("{:d}", &d(item._iCharges), 1).replacen("{:d}", &d(item._iMaxCharges), 1));
    }
}

/// Original: `devilution::PrintItemDur` (items.cpp).
// @port items.cpp|devilution::PrintItemDur(const Item &item) sha=21855e273b3c
pub fn print_item_dur(ctx: &mut Ctx, item: &Item) {
    if ctx.diablo.headless_mode {
        return;
    }
    print_item_dur_lines(ctx, item, false);
    if matches!(item._itype, ItemType::Ring | ItemType::Amulet) {
        crate::control::add_panel_string(ctx, &tr("Not Identified"));
    }
    print_item_info(ctx, item);
}

/// Original: `devilution::UseItem` (items.cpp).
// @port items.cpp|devilution::UseItem(size_t pnum, item_misc_id mid, SpellID spellID, int spellFrom) sha=098c2e1c3fbd
pub fn use_item(ctx: &mut Ctx, pnum: usize, mid: item_misc_id, spell_id: SpellID, spell_from: i32) {
    let is_me = Some(pnum) == ctx.players.MyPlayer;
    let mut prepare_spell_id: Option<SpellID> = None;
    let hf = ctx.init.gb_is_hellfire;
    match mid {
        IMISC_HEAL => {
            crate::player::restore_partial_life(ctx, pnum);
            if is_me {
                redraw_component(ctx, PanelDrawComponent::Health);
            }
        }
        IMISC_FULLHEAL => {
            ctx.players.Players[pnum].restore_full_life();
            if is_me {
                redraw_component(ctx, PanelDrawComponent::Health);
            }
        }
        IMISC_MANA => {
            crate::player::restore_partial_mana(ctx, pnum);
            if is_me {
                redraw_component(ctx, PanelDrawComponent::Mana);
            }
        }
        IMISC_FULLMANA => {
            ctx.players.Players[pnum].restore_full_mana();
            if is_me {
                redraw_component(ctx, PanelDrawComponent::Mana);
            }
        }
        IMISC_ELIXSTR => crate::player::modify_plr_str(ctx, pnum, 1),
        IMISC_ELIXMAG => {
            crate::player::modify_plr_mag(ctx, pnum, 1);
            if hf {
                ctx.players.Players[pnum].restore_full_mana();
                if is_me {
                    redraw_component(ctx, PanelDrawComponent::Mana);
                }
            }
        }
        IMISC_ELIXDEX => crate::player::modify_plr_dex(ctx, pnum, 1),
        IMISC_ELIXVIT => {
            crate::player::modify_plr_vit(ctx, pnum, 1);
            if hf {
                ctx.players.Players[pnum].restore_full_life();
                if is_me {
                    redraw_component(ctx, PanelDrawComponent::Health);
                }
            }
        }
        IMISC_REJUV => {
            crate::player::restore_partial_life(ctx, pnum);
            crate::player::restore_partial_mana(ctx, pnum);
            if is_me {
                redraw_component(ctx, PanelDrawComponent::Health);
                redraw_component(ctx, PanelDrawComponent::Mana);
            }
        }
        IMISC_FULLREJUV | IMISC_ARENAPOT => {
            ctx.players.Players[pnum].restore_full_life();
            ctx.players.Players[pnum].restore_full_mana();
            if is_me {
                redraw_component(ctx, PanelDrawComponent::Health);
                redraw_component(ctx, PanelDrawComponent::Mana);
            }
        }
        IMISC_SCROLL | IMISC_SCROLLT => {
            if ctx.controls.control_mode == crate::controls::ControlTypes::KeyboardAndMouse && get_spell_data(spell_id).flags.has_any_of(SpellDataFlags::Targeted) {
                prepare_spell_id = Some(spell_id);
            } else {
                let spell_level = ctx.players.Players[pnum].get_spell_level(spell_id);
                let mut target = ctx.cursor.cursPosition;
                if !in_dungeon_bounds(target) {
                    let p = &ctx.players.Players[pnum];
                    target = p.position.future + Displacement::from_direction(p._pdir);
                }
                assert!(crate::spells::is_valid_spell_from(spell_from));
                crate::msg::net_send_cmd_loc_param4(
                    ctx,
                    true,
                    CMD_SPELLXY,
                    target,
                    spell_id as i8 as u16,
                    SpellType::Scroll as u8 as u16,
                    spell_level as u16,
                    spell_from as u16,
                );
            }
        }
        IMISC_BOOK => {
            let si = spell_id as i8 as usize;
            let new_spell_level = ctx.players.Players[pnum]._pSplLvl[si].wrapping_add(1);
            if new_spell_level <= crate::player::MaxSpellLevel {
                ctx.players.Players[pnum]._pSplLvl[si] = new_spell_level;
                crate::msg::net_send_cmd_param2(ctx, true, CMD_CHANGE_SPELL_LEVEL, spell_id as i8 as u16, new_spell_level as u16);
            }
            {
                let player = &mut ctx.players.Players[pnum];
                if player._pIFlags.has_none_of(ItemSpecialEffect::NoMana) {
                    let cost = (get_spell_data(spell_id).sManaCost as i32) << 6;
                    player._pMana += cost;
                    player._pMana = player._pMana.min(player._pMaxMana);
                    player._pManaBase += cost;
                    player._pManaBase = player._pManaBase.min(player._pMaxManaBase);
                }
            }
            if is_me {
                let snapshot = ctx.players.Players[pnum].clone();
                let player = &mut ctx.players.Players[pnum];
                let n = player._pNumInv as usize;
                for item in player.InvList[..n].iter_mut() {
                    item.update_required_stats_cache_for_player(&snapshot);
                }
                if ctx.stash.IsStashOpen {
                    crate::qol::stash::refresh_item_stat_flags(ctx);
                }
            }
            redraw_component(ctx, PanelDrawComponent::Mana);
        }
        IMISC_MAPOFDOOM => crate::doom::doom_init(ctx),
        IMISC_OILACC | IMISC_OILMAST | IMISC_OILSHARP | IMISC_OILDEATH | IMISC_OILSKILL | IMISC_OILBSMTH | IMISC_OILFORT | IMISC_OILPERM | IMISC_OILHARD | IMISC_OILIMP => {
            ctx.players.Players[pnum]._pOilType = mid;
            if !is_me {
                return;
            }
            if ctx.control.sbookflag {
                ctx.control.sbookflag = false;
            }
            if !ctx.inv.invflag {
                ctx.inv.invflag = true;
            }
            crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_OIL);
        }
        IMISC_SPECELIX => {
            crate::player::modify_plr_str(ctx, pnum, 3);
            crate::player::modify_plr_mag(ctx, pnum, 3);
            crate::player::modify_plr_dex(ctx, pnum, 3);
            crate::player::modify_plr_vit(ctx, pnum, 3);
        }
        IMISC_RUNEF => prepare_spell_id = Some(SpellID::RuneOfFire),
        IMISC_RUNEL => prepare_spell_id = Some(SpellID::RuneOfLight),
        IMISC_GR_RUNEL => prepare_spell_id = Some(SpellID::RuneOfNova),
        IMISC_GR_RUNEF => prepare_spell_id = Some(SpellID::RuneOfImmolation),
        IMISC_RUNES => prepare_spell_id = Some(SpellID::RuneOfStone),
        _ => {}
    }
    if let Some(sp) = prepare_spell_id {
        assert!(crate::spells::is_valid_spell_from(spell_from));
        ctx.players.Players[pnum].inventorySpell = sp;
        ctx.players.Players[pnum].spellFrom = spell_from as i8;
        if is_me {
            crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_TELEPORT);
        }
    }
}

/// Original: `devilution::UseItemOpensHive` (items.cpp).
// @port items.cpp|devilution::UseItemOpensHive(const Item &item, Point position) sha=b8a2e9405189
pub fn use_item_opens_hive(ctx: &Ctx, item: &Item, position: Point) -> bool {
    if item.IDidx != IDI_RUNEBOMB {
        return false;
    }
    crate::engine::path::PATH_DIRS.iter().any(|&dir| crate::levels::town::opens_hive(position + dir))
}

/// Original: `devilution::UseItemOpensGrave` (items.cpp).
// @port items.cpp|devilution::UseItemOpensGrave(const Item &item, Point position) sha=17e6c3da82c2
pub fn use_item_opens_grave(ctx: &Ctx, item: &Item, position: Point) -> bool {
    if item.IDidx != IDI_MAPOFDOOM {
        return false;
    }
    crate::engine::path::PATH_DIRS.iter().any(|&dir| crate::levels::town::opens_grave(position + dir))
}

/// Original: `devilution::SpawnSmith` (items.cpp).
// @port items.cpp|devilution::SpawnSmith(int lvl) sha=2272c581dc4b
pub fn spawn_smith(ctx: &mut Ctx, lvl: i32) {
    const PINNED_ITEM_COUNT: usize = 0;
    let mut max_value = 140000;
    let mut max_items = 20;
    if ctx.init.gb_is_hellfire {
        max_value = 200000;
        max_items = 25;
    }
    let i_cnt = (ctx.rng.generate_rnd(max_items - 10) + 10) as usize;
    let me = my_player_copy(ctx);
    for i in 0..i_cnt {
        let mut new_item;
        loop {
            new_item = Item::default();
            new_item._iSeed = ctx.rng.advance_rnd_seed() as u32;
            ctx.rng.set_rnd_seed(new_item._iSeed);
            let item_data = rnd_smith_item(ctx, &me, lvl);
            get_item_attrs(ctx, &mut new_item, item_data, lvl);
            if new_item._iIvalue <= max_value {
                break;
            }
        }
        new_item._iCreateInfo = (lvl | CF_SMITH) as u16;
        new_item._iIdentified = true;
        ctx.stores.smithitem[i] = new_item;
    }
    for i in i_cnt..crate::stores::SMITH_ITEMS {
        ctx.stores.smithitem[i].clear();
    }
    sort_vendor(&mut ctx.stores.smithitem[PINNED_ITEM_COUNT..]);
}

/// Original: `devilution::SpawnPremium` (items.cpp).
// @port items.cpp|devilution::SpawnPremium(const Player &player) sha=3f0cc7684711
pub fn spawn_premium(ctx: &mut Ctx, player: &Player) {
    let lvl = player._pLevel as i32;
    let hf = ctx.init.gb_is_hellfire;
    let max_items = if hf { crate::stores::SMITH_PREMIUM_ITEMS as i32 } else { 6 };
    if ctx.stores.numpremium < max_items {
        for i in 0..max_items as usize {
            if ctx.stores.premiumitems[i].is_empty() {
                let plvl = ctx.stores.premiumlevel + if hf { premiumLvlAddHellfire[i] } else { premiumlvladd[i] };
                let mut it = std::mem::take(&mut ctx.stores.premiumitems[i]);
                spawn_one_premium(ctx, &mut it, plvl, player);
                ctx.stores.premiumitems[i] = it;
            }
        }
        ctx.stores.numpremium = max_items;
    }
    let spawn_at = |ctx: &mut Ctx, i: usize, add: i32| {
        let plvl = ctx.stores.premiumlevel + add;
        let mut it = std::mem::take(&mut ctx.stores.premiumitems[i]);
        spawn_one_premium(ctx, &mut it, plvl, player);
        ctx.stores.premiumitems[i] = it;
    };
    while ctx.stores.premiumlevel < lvl {
        ctx.stores.premiumlevel += 1;
        if hf {
            // Discard first 3 items and shift next 10
            for k in 0..10 {
                ctx.stores.premiumitems[k] = ctx.stores.premiumitems[k + 3].clone();
            }
            spawn_at(ctx, 10, premiumLvlAddHellfire[10]);
            ctx.stores.premiumitems[11] = ctx.stores.premiumitems[13].clone();
            spawn_at(ctx, 12, premiumLvlAddHellfire[12]);
            ctx.stores.premiumitems[13] = ctx.stores.premiumitems[14].clone();
            spawn_at(ctx, 14, premiumLvlAddHellfire[14]);
        } else {
            // Discard first 2 items and shift next 3
            for k in 0..3 {
                ctx.stores.premiumitems[k] = ctx.stores.premiumitems[k + 2].clone();
            }
            spawn_at(ctx, 3, premiumlvladd[3]);
            ctx.stores.premiumitems[4] = ctx.stores.premiumitems[5].clone();
            spawn_at(ctx, 5, premiumlvladd[5]);
        }
    }
}

/// Original: `devilution::SpawnWitch` (items.cpp).
// @port items.cpp|devilution::SpawnWitch(int lvl) sha=29cd81a83d58
pub fn spawn_witch(ctx: &mut Ctx, lvl: i32) {
    const PINNED_ITEM_COUNT: usize = 3;
    const PINNED_ITEM_TYPES: [_item_indexes; PINNED_ITEM_COUNT] = [IDI_MANA, IDI_FULLMANA, IDI_PORTAL];
    const MAX_PINNED_BOOK_COUNT: i32 = 4;
    const PINNED_BOOK_TYPES: [_item_indexes; 4] = [IDI_BOOK1, IDI_BOOK2, IDI_BOOK3, IDI_BOOK4];
    let hf = ctx.init.gb_is_hellfire;
    let mut book_count = 0;
    let pinned_book_count = if hf { ctx.rng.generate_rnd(MAX_PINNED_BOOK_COUNT) } else { 0 };
    let reserved_items = if hf { 10 } else { 17 };
    let item_count = ctx.rng.generate_rnd(crate::stores::WITCH_ITEMS as i32 - reserved_items) + 10;
    let max_value = if hf { 200000 } else { 140000 };
    let me = my_player_copy(ctx);
    for i in 0..crate::stores::WITCH_ITEMS {
        let mut item = Item::default();
        if i < PINNED_ITEM_COUNT {
            item._iSeed = ctx.rng.advance_rnd_seed() as u32;
            get_item_attrs(ctx, &mut item, PINNED_ITEM_TYPES[i], 1);
            item._iCreateInfo = lvl as u16;
            item._iStatFlag = true;
            ctx.stores.witchitem[i] = item;
            continue;
        }
        if hf && i < PINNED_ITEM_COUNT + MAX_PINNED_BOOK_COUNT as usize && book_count < pinned_book_count {
            let book_type = PINNED_BOOK_TYPES[i - PINNED_ITEM_COUNT];
            if lvl >= AllItemsList[book_type as usize].iMinMLvl as i32 {
                item._iSeed = ctx.rng.advance_rnd_seed() as u32;
                ctx.rng.set_rnd_seed(item._iSeed);
                ctx.rng.discard_random_values(1);
                get_item_attrs(ctx, &mut item, book_type, lvl);
                item._iCreateInfo = (lvl | CF_WITCH) as u16;
                item._iIdentified = true;
                book_count += 1;
                ctx.stores.witchitem[i] = item;
                continue;
            }
        }
        if i as i32 >= item_count {
            item.clear();
            ctx.stores.witchitem[i] = item;
            continue;
        }
        loop {
            item = Item::default();
            item._iSeed = ctx.rng.advance_rnd_seed() as u32;
            ctx.rng.set_rnd_seed(item._iSeed);
            let item_data = rnd_witch_item(ctx, &me, lvl);
            get_item_attrs(ctx, &mut item, item_data, lvl);
            let mut maxlvl = -1;
            if ctx.rng.generate_rnd(100) <= 5 {
                maxlvl = 2 * lvl;
            }
            if maxlvl == -1 && item._iMiscId == IMISC_STAFF {
                maxlvl = 2 * lvl;
            }
            if maxlvl != -1 {
                get_item_bonus(ctx, &me, &mut item, maxlvl / 2, maxlvl, true, true);
            }
            if item._iIvalue <= max_value {
                break;
            }
        }
        item._iCreateInfo = (lvl | CF_WITCH) as u16;
        item._iIdentified = true;
        ctx.stores.witchitem[i] = item;
    }
    sort_vendor(&mut ctx.stores.witchitem[PINNED_ITEM_COUNT..]);
}

/// Original: `devilution::SpawnBoy` (items.cpp).
// @port items.cpp|devilution::SpawnBoy(int lvl) sha=8ac4798f46cf
pub fn spawn_boy(ctx: &mut Ctx, lvl: i32) {
    let mut ivalue;
    let mut keepgoing;
    let mut count = 0;
    let my_player = my_player_copy(ctx);
    let pc = my_player._pClass;
    let mut strength = my_player.get_maximum_attribute_value(CharacterAttribute::Strength).max(my_player._pStrength);
    let mut dexterity = my_player.get_maximum_attribute_value(CharacterAttribute::Dexterity).max(my_player._pDexterity);
    let mut magic = my_player.get_maximum_attribute_value(CharacterAttribute::Magic).max(my_player._pMagic);
    strength += strength / 5;
    dexterity += dexterity / 5;
    magic += magic / 5;
    if ctx.stores.boylevel >= lvl / 2 && !ctx.stores.boyitem.is_empty() {
        return;
    }
    loop {
        keepgoing = false;
        ivalue = 0;
        let mut boyitem = Item::default();
        boyitem._iSeed = ctx.rng.advance_rnd_seed() as u32;
        ctx.rng.set_rnd_seed(boyitem._iSeed);
        let itype = rnd_boy_item(ctx, &my_player, lvl);
        get_item_attrs(ctx, &mut boyitem, itype, lvl);
        get_item_bonus(ctx, &my_player, &mut boyitem, lvl, 2 * lvl, true, true);
        let mut skip_rest = false;
        if !ctx.init.gb_is_hellfire {
            if boyitem._iIvalue > 90000 {
                keepgoing = true;
                skip_rest = true; // continue
            } else {
                ctx.stores.boyitem = boyitem;
                break;
            }
        }
        if !skip_rest {
            let item_type = boyitem._itype;
            ivalue = match item_type {
                ItemType::LightArmor | ItemType::MediumArmor | ItemType::HeavyArmor => most_valuable_player_item_value(&my_player, &|item| {
                    matches!(item._itype, ItemType::LightArmor | ItemType::MediumArmor | ItemType::HeavyArmor)
                }),
                ItemType::Shield
                | ItemType::Axe
                | ItemType::Bow
                | ItemType::Mace
                | ItemType::Sword
                | ItemType::Helm
                | ItemType::Staff
                | ItemType::Ring
                | ItemType::Amulet => most_valuable_player_item_value(&my_player, &|item| item._itype == item_type),
                _ => crate::appfat::app_fatal(ctx, "Invalid item spawn"),
            };
            ivalue = ivalue * 4 / 5;
            count += 1;
            if count < 200 {
                let bad = match pc {
                    HeroClass::Warrior => matches!(item_type, ItemType::Bow | ItemType::Staff),
                    HeroClass::Rogue => matches!(item_type, ItemType::Sword | ItemType::Staff | ItemType::Axe | ItemType::Mace | ItemType::Shield),
                    HeroClass::Sorcerer => matches!(item_type, ItemType::Staff | ItemType::Axe | ItemType::Bow | ItemType::Mace),
                    HeroClass::Monk => matches!(item_type, ItemType::Bow | ItemType::MediumArmor | ItemType::Shield | ItemType::Mace),
                    HeroClass::Bard => matches!(item_type, ItemType::Axe | ItemType::Mace | ItemType::Staff),
                    HeroClass::Barbarian => matches!(item_type, ItemType::Bow | ItemType::Staff),
                };
                if bad {
                    ivalue = i32::MAX;
                }
            }
        }
        let retry = keepgoing
            || ((boyitem._iIvalue > 200000
                || boyitem._iMinStr as i32 > strength
                || boyitem._iMinMag as i32 > magic
                || boyitem._iMinDex as i32 > dexterity
                || boyitem._iIvalue < ivalue)
                && count < 250);
        ctx.stores.boyitem = boyitem;
        if !retry {
            break;
        }
    }
    ctx.stores.boyitem._iCreateInfo = (lvl | CF_BOY) as u16;
    ctx.stores.boyitem._iIdentified = true;
    ctx.stores.boylevel = lvl / 2;
}

/// Original: `devilution::SpawnHealer` (items.cpp).
// @port items.cpp|devilution::SpawnHealer(int lvl) sha=a22c86fc59ee
pub fn spawn_healer(ctx: &mut Ctx, lvl: i32) {
    const PINNED_ITEM_COUNT: usize = 2;
    const PINNED_ITEM_TYPES: [_item_indexes; 3] = [IDI_HEAL, IDI_FULLHEAL, IDI_RESURRECT];
    let item_count = ctx.rng.generate_rnd(if ctx.init.gb_is_hellfire { 10 } else { 8 }) + 10;
    let me = my_player_copy(ctx);
    for i in 0..20 {
        let mut item = Item::default();
        if i < PINNED_ITEM_COUNT || (ctx.init.gb_is_multiplayer && i == PINNED_ITEM_COUNT) {
            item._iSeed = ctx.rng.advance_rnd_seed() as u32;
            get_item_attrs(ctx, &mut item, PINNED_ITEM_TYPES[i], 1);
            item._iCreateInfo = lvl as u16;
            item._iStatFlag = true;
            ctx.stores.healitem[i] = item;
            continue;
        }
        if i as i32 >= item_count {
            item.clear();
            ctx.stores.healitem[i] = item;
            continue;
        }
        item._iSeed = ctx.rng.advance_rnd_seed() as u32;
        ctx.rng.set_rnd_seed(item._iSeed);
        let itype = rnd_healer_item(ctx, &me, lvl);
        get_item_attrs(ctx, &mut item, itype, lvl);
        item._iCreateInfo = (lvl | CF_HEALER) as u16;
        item._iIdentified = true;
        ctx.stores.healitem[i] = item;
    }
    sort_vendor(&mut ctx.stores.healitem[PINNED_ITEM_COUNT..]);
}

/// Original: `devilution::MakeGoldStack` (items.cpp).
// @port items.cpp|devilution::MakeGoldStack(Item &goldItem, int value) sha=2f647d732db7
pub fn make_gold_stack(ctx: &mut Ctx, gold_item: &mut Item, value: i32) {
    initialize_item(ctx, gold_item, IDI_GOLD);
    generate_new_seed(ctx, gold_item);
    gold_item._iStatFlag = true;
    gold_item._ivalue = value;
    set_plr_hand_gold_curs(gold_item);
}

/// Original: `devilution::ItemNoFlippy` (items.cpp).
// @port items.cpp|devilution::ItemNoFlippy() sha=66a8571f8ece
pub fn item_no_flippy(ctx: &mut Ctx) -> i32 {
    let r = ctx.items.ActiveItems[ctx.items.ActiveItemCount as usize - 1] as usize;
    let item = &mut ctx.items.Items[r];
    item.AnimInfo.currentFrame = item.AnimInfo.numberOfFrames - 1;
    item._iAnimFlag = false;
    item._iSelFlag = 1;
    r as i32
}

/// Original: `devilution::CreateSpellBook` (items.cpp).
// @port items.cpp|devilution::CreateSpellBook(Point position, SpellID ispell, bool sendmsg, bool delta) sha=f79f6b1bdf54
pub fn create_spell_book(ctx: &mut Ctx, position: Point, ispell: SpellID, sendmsg: bool, delta: bool) {
    let mut lvl = ctx.gendung.currlevel as i32;
    if ctx.init.gb_is_hellfire {
        lvl = crate::spells::get_spell_book_level(ctx, ispell) + 1;
        if lvl < 1 {
            return;
        }
    }
    let idx = rnd_type_items(ctx, ItemType::Misc, IMISC_BOOK as i32, lvl);
    if ctx.items.ActiveItemCount as usize >= MAXITEMS {
        return;
    }
    let ii = allocate_item(ctx) as usize;
    let me = my_player_copy(ctx);
    loop {
        let seed = ctx.rng.advance_rnd_seed() as u32;
        let done = with_item(ctx, ii, |ctx, item| {
            *item = Item::default();
            setup_all_items(ctx, &me, item, idx, seed, 2 * lvl, 1, true, false, delta);
            item._iMiscId == IMISC_BOOK && item._iSpell == ispell
        });
        if done {
            break;
        }
    }
    get_super_item_space(ctx, position, ii as i8);
    if sendmsg {
        let pos = ctx.items.Items[ii].position;
        { let it = ctx.items.Items[ii as usize].clone(); crate::msg::net_send_cmd_p_item(ctx, false, CMD_DROPITEM, pos, &it); }
    }
    if delta {
        crate::msg::delta_add_item(ctx, ii as i32);
    }
}

/// Original: `devilution::CreateMagicArmor` (items.cpp).
// @port items.cpp|devilution::CreateMagicArmor(Point position, ItemType itemType, int icurs, bool sendmsg, bool delta) sha=63c621de4e5e
pub fn create_magic_armor(ctx: &mut Ctx, position: Point, item_type: ItemType, icurs: i32, sendmsg: bool, delta: bool) {
    let lvl = items_get_currlevel(ctx);
    create_magic_item(ctx, position, lvl, item_type, IMISC_NONE as i32, icurs, sendmsg, delta, false);
}

/// Original: `devilution::CreateAmulet` (items.cpp). `spawn` defaults to false.
// @port items.cpp|devilution::CreateAmulet(Point position, int lvl, bool sendmsg, bool delta, bool spawn) sha=120db09398fb
pub fn create_amulet(ctx: &mut Ctx, position: Point, lvl: i32, sendmsg: bool, delta: bool, spawn: bool) {
    create_magic_item(ctx, position, lvl, ItemType::Amulet, IMISC_AMULET as i32, ICURS_AMULET as i32, sendmsg, delta, spawn);
}

/// Original: `devilution::CreateMagicWeapon` (items.cpp).
// @port items.cpp|devilution::CreateMagicWeapon(Point position, ItemType itemType, int icurs, bool sendmsg, bool delta) sha=ffc230967e4b
pub fn create_magic_weapon(ctx: &mut Ctx, position: Point, item_type: ItemType, icurs: i32, sendmsg: bool, delta: bool) {
    let mut imid = IMISC_NONE as i32;
    if item_type == ItemType::Staff {
        imid = IMISC_STAFF as i32;
    }
    let curlv = items_get_currlevel(ctx);
    create_magic_item(ctx, position, curlv, item_type, imid, icurs, sendmsg, delta, false);
}

/// Original: `devilution::GetItemRecord` (items.cpp).
// @port items.cpp|devilution::GetItemRecord(uint32_t nSeed, uint16_t wCI, int nIndex) sha=c9548a42d78a
pub fn get_item_record(ctx: &mut Ctx, n_seed: u32, w_ci: u16, n_index: i32) -> bool {
    let ticks = ctx.platform.ticks();
    let mut i = 0i32;
    while i < ctx.items.gnNumGetRecords {
        let r = ctx.items.itemrecord[i as usize];
        if ticks.wrapping_sub(r.dwTimestamp) > 6000 {
            next_item_record(ctx, i as usize);
            i -= 1;
        } else if n_seed == r.nSeed && w_ci == r.wCI && n_index == r.nIndex {
            return false;
        }
        i += 1;
    }
    true
}

/// Original: `devilution::SetItemRecord` (items.cpp).
// @port items.cpp|devilution::SetItemRecord(uint32_t nSeed, uint16_t wCI, int nIndex) sha=2d8314e321f0
pub fn set_item_record(ctx: &mut Ctx, n_seed: u32, w_ci: u16, n_index: i32) {
    let ticks = ctx.platform.ticks();
    let s = &mut ctx.items;
    if s.gnNumGetRecords == MAXITEMS as i32 {
        return;
    }
    s.itemrecord[s.gnNumGetRecords as usize] = ItemGetRecordStruct { nSeed: n_seed, wCI: w_ci, nIndex: n_index, dwTimestamp: ticks };
    s.gnNumGetRecords += 1;
}

/// Original: `devilution::PutItemRecord` (items.cpp).
// @port items.cpp|devilution::PutItemRecord(uint32_t nSeed, uint16_t wCI, int nIndex) sha=5badfe5bd9d3
pub fn put_item_record(ctx: &mut Ctx, n_seed: u32, w_ci: u16, n_index: i32) {
    let ticks = ctx.platform.ticks();
    let mut i = 0i32;
    while i < ctx.items.gnNumGetRecords {
        let r = ctx.items.itemrecord[i as usize];
        if ticks.wrapping_sub(r.dwTimestamp) > 6000 {
            next_item_record(ctx, i as usize);
            i -= 1;
        } else if n_seed == r.nSeed && w_ci == r.wCI && n_index == r.nIndex {
            next_item_record(ctx, i as usize);
            break;
        }
        i += 1;
    }
}

/// Original: `devilution::initItemGetRecords` (items.cpp).
// @port items.cpp|devilution::initItemGetRecords() sha=898a3860c711
pub fn init_item_get_records(ctx: &mut Ctx) {
    ctx.items.itemrecord = [ItemGetRecordStruct::default(); MAXITEMS];
    ctx.items.gnNumGetRecords = 0;
}

/// Original: `devilution::RepairItem` (items.cpp).
// @port items.cpp|devilution::RepairItem(Item &item, int lvl) sha=db684af98341
pub fn repair_item(ctx: &mut Ctx, item: &mut Item, lvl: i32) {
    if item._iDurability == item._iMaxDur {
        return;
    }
    if item._iMaxDur <= 0 {
        item.clear();
        return;
    }
    let mut rep = 0;
    loop {
        rep += lvl + ctx.rng.generate_rnd(lvl);
        item._iMaxDur -= (item._iMaxDur / (lvl + 9)).max(1);
        if item._iMaxDur == 0 {
            item.clear();
            return;
        }
        if rep + item._iDurability >= item._iMaxDur {
            break;
        }
    }
    item._iDurability = (item._iDurability + rep).min(item._iMaxDur);
}

/// Original: `devilution::RechargeItem` (items.cpp). The item is `cii` of player `pnum`
/// (inventory or body), so the network sync can find it.
// @port items.cpp|devilution::RechargeItem(Item &item, Player &player) sha=1ec32798b152
pub fn recharge_item(ctx: &mut Ctx, pnum: usize, cii: i32) {
    let level = ctx.players.Players[pnum]._pLevel as i32;
    let hellfire = ctx.init.gb_is_hellfire;
    let item = inv_item_mut(&mut ctx.players.Players[pnum], cii);
    if item._itype != ItemType::Staff || !crate::spells::is_valid_spell_hf(item._iSpell, hellfire) {
        return;
    }
    if item._iCharges == item._iMaxCharges {
        return;
    }
    let spell = item._iSpell;
    let mut r = crate::spells::get_spell_staff_level(ctx, spell);
    r = ctx.rng.generate_rnd(level / r) + 1;
    let item = inv_item_mut(&mut ctx.players.Players[pnum], cii);
    loop {
        item._iMaxCharges -= 1;
        if item._iMaxCharges == 0 {
            return;
        }
        item._iCharges += r;
        if item._iCharges >= item._iMaxCharges {
            break;
        }
    }
    item._iCharges = item._iCharges.min(item._iMaxCharges);
    if Some(pnum) != ctx.players.MyPlayer {
        return;
    }
    if cii == INVLOC_HAND_LEFT as i32 {
        crate::msg::net_send_cmd_ch_item(ctx, true, INVLOC_HAND_LEFT as u8, false);
        return;
    }
    if cii == INVLOC_HAND_RIGHT as i32 {
        crate::msg::net_send_cmd_ch_item(ctx, true, INVLOC_HAND_RIGHT as u8, false);
        return;
    }
    if cii >= NUM_INVLOC as i32 {
        crate::msg::net_sync_inv_item(ctx, pnum, cii - NUM_INVLOC as i32);
    }
}

/// Original: `devilution::ApplyOilToItem` (items.cpp). The player's `_pOilType` is passed in.
// @port items.cpp|devilution::ApplyOilToItem(Item &item, Player &player) sha=80c0a1967d51
pub fn apply_oil_to_item(ctx: &mut Ctx, item: &mut Item, oil_type: item_misc_id) -> bool {
    if matches!(item._iClass, ICLASS_MISC | ICLASS_GOLD | ICLASS_QUEST) {
        return false;
    }
    match oil_type {
        IMISC_OILACC | IMISC_OILMAST | IMISC_OILSHARP => {
            if item._iClass == ICLASS_ARMOR {
                return false;
            }
        }
        IMISC_OILDEATH => {
            if item._iClass == ICLASS_ARMOR || item._itype == ItemType::Bow {
                return false;
            }
        }
        IMISC_OILHARD | IMISC_OILIMP => {
            if item._iClass == ICLASS_WEAPON {
                return false;
            }
        }
        _ => {}
    }
    match oil_type {
        IMISC_OILACC => {
            if item._iPLToHit < 50 {
                let r = ctx.rng.generate_rnd(2) + 1;
                item._iPLToHit = add16(item._iPLToHit, r);
            }
        }
        IMISC_OILMAST => {
            if item._iPLToHit < 100 {
                let r = ctx.rng.generate_rnd(3) + 3;
                item._iPLToHit = add16(item._iPLToHit, r);
            }
        }
        IMISC_OILSHARP => {
            if (item._iMaxDam as i32 - item._iMinDam as i32) < 30 && item._iMaxDam < 255 {
                item._iMaxDam += 1;
            }
        }
        IMISC_OILDEATH => {
            if (item._iMaxDam as i32 - item._iMinDam as i32) < 30 && item._iMaxDam < 254 {
                item._iMinDam = item._iMinDam.wrapping_add(1);
                item._iMaxDam += 2;
            }
        }
        IMISC_OILSKILL => {
            let r = ctx.rng.generate_rnd(6) + 5;
            item._iMinStr = 0.max(item._iMinStr as i32 - r) as i8;
            item._iMinMag = 0.max(item._iMinMag as i32 - r) as u8;
            item._iMinDex = 0.max(item._iMinDex as i32 - r) as i8;
        }
        IMISC_OILBSMTH => {
            if item._iMaxDur == DUR_INDESTRUCTIBLE {
                return true;
            }
            if item._iDurability < item._iMaxDur {
                item._iDurability = (item._iMaxDur + 4) / 5 + item._iDurability;
                item._iDurability = item._iDurability.min(item._iMaxDur);
            } else {
                if item._iMaxDur >= 100 {
                    return true;
                }
                item._iMaxDur += 1;
                item._iDurability = item._iMaxDur;
            }
        }
        IMISC_OILFORT => {
            if item._iMaxDur != DUR_INDESTRUCTIBLE && item._iMaxDur < 200 {
                let r = ctx.rng.generate_rnd(41) + 10;
                item._iMaxDur += r;
                item._iDurability += r;
            }
        }
        IMISC_OILPERM => {
            item._iDurability = DUR_INDESTRUCTIBLE;
            item._iMaxDur = DUR_INDESTRUCTIBLE;
        }
        IMISC_OILHARD => {
            if item._iAC < 60 {
                let r = ctx.rng.generate_rnd(2) + 1;
                item._iAC = add16(item._iAC, r);
            }
        }
        IMISC_OILIMP => {
            if item._iAC < 120 {
                let r = ctx.rng.generate_rnd(3) + 3;
                item._iAC = add16(item._iAC, r);
            }
        }
        _ => return false,
    }
    true
}

/// Original: `devilution::UpdateHellfireFlag` (items.cpp).
// @port items.cpp|devilution::UpdateHellfireFlag(Item &item, const char *identifiedItemName) sha=80d5b717a76b
pub fn update_hellfire_flag(ctx: &mut Ctx, item: &mut Item, identified_item_name: &str) {
    if item.dwBuff & CF_HELLFIRE as u32 != 0 {
        return;
    }
    if item._iMagical != ITEM_QUALITY_MAGIC {
        return;
    }
    if ctx.init.gb_is_multiplayer {
        return;
    }
    let diablo_short = get_translated_item_name_magical(ctx, item, false, false, Some(false));
    if diablo_short == identified_item_name {
        return;
    }
    let diablo_long = get_translated_item_name_magical(ctx, item, false, false, Some(true));
    if diablo_long == identified_item_name {
        return;
    }
    let hellfire_short = get_translated_item_name_magical(ctx, item, true, false, Some(false));
    let hellfire_long = get_translated_item_name_magical(ctx, item, true, false, Some(true));
    if hellfire_short == identified_item_name || hellfire_long == identified_item_name {
        item.dwBuff |= CF_HELLFIRE as u32;
    }
}

/// `Direction` unused import guard.
#[allow(dead_code)]
const _DIR: Direction = Direction::South;

/// `RechargeItem` for an item outside the player's inventory (the stash): the original's body
/// up to the network sync, which only applies to inventory and body items.
pub fn recharge_loose_item(ctx: &mut Ctx, item: &mut Item, player_level: i32) {
    let hellfire = ctx.init.gb_is_hellfire;
    if item._itype != ItemType::Staff || !crate::spells::is_valid_spell_hf(item._iSpell, hellfire) {
        return;
    }
    if item._iCharges == item._iMaxCharges {
        return;
    }
    let mut r = crate::spells::get_spell_staff_level(ctx, item._iSpell);
    r = ctx.rng.generate_rnd(player_level / r) + 1;
    loop {
        item._iMaxCharges -= 1;
        if item._iMaxCharges == 0 {
            return;
        }
        item._iCharges += r;
        if item._iCharges >= item._iMaxCharges {
            break;
        }
    }
    item._iCharges = item._iCharges.min(item._iMaxCharges);
}
