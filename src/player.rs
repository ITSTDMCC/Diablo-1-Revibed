//! `Source/player.cpp`: the heroes — creation, stats, animation, actions (walk, attack, spell),
//! damage, death and level changes.
//!
//! Functions take the player by index (`pnum`) into `ctx.players.Players`.

use crate::ctx::Ctx;
use crate::engine::actor_position::ActorPosition;
use crate::engine::animationinfo::AnimationInfo;
use crate::engine::backbuffer_state::{redraw_component, redraw_everything, PanelDrawComponent};
use crate::engine::clx_sprite::{ClxSprite, ClxSpriteList, ClxSpriteSheet};
use crate::engine::geometry::{left, right, Direction, Displacement, Point};
use crate::engine::path::MaxPathLength;
use crate::enums::*;
use crate::items::{Item, DUR_INDESTRUCTIBLE, GOLD_MAX_LIMIT};
use crate::levels::gendung::{in_dungeon_bounds, DungeonType, MAXDUNX, MAXDUNY};
use crate::lighting::NO_LIGHT;
use crate::tables::playerdat::*;
use crate::utils::cstr::CStr;

pub const InventoryGridCells: usize = 40;
pub const MaxBeltItems: usize = 8;
pub const MaxResistance: i32 = 75;
pub const MaxCharacterLevel: i32 = 50;
pub const MaxSpellLevel: u8 = 15;
pub const PlayerNameLength: usize = 32;
pub const NumHotkeys: usize = 12;
pub const BaseHitChance: i32 = 50;
pub const NUMLEVELS: usize = 25;
pub const MAX_PLRS: usize = 4;

/// Maps from armor animation to letter used in graphic files.
pub const ArmourChar: [u8; 4] = [b'l', b'm', b'h', 0];
/// Maps from weapon animation to letter used in graphic files.
pub const WepChar: [u8; 9] = [b'n', b'u', b's', b'd', b'b', b'a', b'm', b'h', b't'];
/// Maps from player class to letter used in graphic files.
pub const CharChar: [u8; 6] = [b'w', b'r', b's', b'm', b'b', b'c'];

/// `PlayerAnimationData`
#[derive(Clone, Debug, Default)]
pub struct PlayerAnimationData {
    pub sprites: Option<ClxSpriteSheet>,
}

impl PlayerAnimationData {
    /// `spritesForDirection`
    pub fn sprites_for_direction(&self, direction: Direction) -> Option<ClxSpriteList> {
        self.sprites.as_ref().map(|s| s.get(direction as usize))
    }
}

/// `SpellCastInfo`
#[derive(Clone, Copy, Debug, Default)]
pub struct SpellCastInfo {
    pub spellId: SpellID,
    pub spellType: SpellType,
    /// Inventory location for scrolls
    pub spellFrom: i8,
    /// Used for spell level
    pub spellLevel: i32,
}

/// `Player`
#[derive(Clone, Debug)]
pub struct Player {
    pub _pName: CStr<PlayerNameLength>,
    pub InvBody: [Item; NUM_INVLOC as usize],
    pub InvList: Vec<Item>,
    pub SpdList: [Item; MaxBeltItems],
    pub HoldItem: Item,
    pub lightId: i32,
    pub _pNumInv: i32,
    pub _pStrength: i32,
    pub _pBaseStr: i32,
    pub _pMagic: i32,
    pub _pBaseMag: i32,
    pub _pDexterity: i32,
    pub _pBaseDex: i32,
    pub _pVitality: i32,
    pub _pBaseVit: i32,
    pub _pStatPts: i32,
    pub _pDamageMod: i32,
    pub _pBaseToBlk: i32,
    pub _pHPBase: i32,
    pub _pMaxHPBase: i32,
    pub _pHitPoints: i32,
    pub _pMaxHP: i32,
    pub _pHPPer: i32,
    pub _pManaBase: i32,
    pub _pMaxManaBase: i32,
    pub _pMana: i32,
    pub _pMaxMana: i32,
    pub _pManaPer: i32,
    pub _pIMinDam: i32,
    pub _pIMaxDam: i32,
    pub _pIAC: i32,
    pub _pIBonusDam: i32,
    pub _pIBonusToHit: i32,
    pub _pIBonusAC: i32,
    pub _pIBonusDamMod: i32,
    pub _pIGetHit: i32,
    pub _pIEnAc: i32,
    pub _pIFMinDam: i32,
    pub _pIFMaxDam: i32,
    pub _pILMinDam: i32,
    pub _pILMaxDam: i32,
    pub _pExperience: u32,
    pub _pNextExper: u32,
    pub _pmode: PLR_MODE,
    pub walkpath: [i8; MaxPathLength],
    pub plractive: bool,
    pub destAction: action_id,
    pub destParam1: i32,
    pub destParam2: i32,
    pub destParam3: i32,
    pub destParam4: i32,
    pub _pGold: i32,
    pub AnimInfo: AnimationInfo,
    pub previewCelSprite: Option<ClxSprite>,
    pub progressToNextGameTickWhenPreviewWasSet: i8,
    pub _pIFlags: ItemSpecialEffect,
    pub AnimationData: [PlayerAnimationData; 9],
    pub _pNFrames: i8,
    pub _pWFrames: i8,
    pub _pAFrames: i8,
    pub _pAFNum: i8,
    pub _pSFrames: i8,
    pub _pSFNum: i8,
    pub _pHFrames: i8,
    pub _pDFrames: i8,
    pub _pBFrames: i8,
    pub InvGrid: [i8; InventoryGridCells],
    pub plrlevel: u8,
    pub plrIsOnSetLevel: bool,
    pub position: ActorPosition,
    /// Direction faced by player
    pub _pdir: Direction,
    pub _pClass: HeroClass,
    pub _pLevel: i8,
    pub _pMaxLvl: i8,
    /// Low 4 bits: weapon graphic, high bits: armour graphic.
    pub _pgfxnum: u8,
    pub _pISplLvlAdd: i8,
    pub friendlyMode: bool,
    pub queuedSpell: SpellCastInfo,
    pub executedSpell: SpellCastInfo,
    /// Which spell should be executed with CURSOR_TELEPORT
    pub inventorySpell: SpellID,
    /// Inventory location for scrolls with CURSOR_TELEPORT
    pub spellFrom: i8,
    pub _pRSpell: SpellID,
    pub _pRSplType: SpellType,
    pub _pSBkSpell: SpellID,
    pub _pSplLvl: [u8; 64],
    pub _pISpells: u64,
    pub _pMemSpells: u64,
    pub _pAblSpells: u64,
    pub _pScrlSpells: u64,
    pub _pSpellFlags: SpellFlag,
    pub _pSplHotKey: [SpellID; NumHotkeys],
    pub _pSplTHotKey: [SpellType; NumHotkeys],
    pub _pBlockFlag: bool,
    pub _pInvincible: bool,
    pub _pLightRad: i8,
    pub _pLvlChanging: bool,
    pub _pArmorClass: i8,
    pub _pMagResist: i8,
    pub _pFireResist: i8,
    pub _pLghtResist: i8,
    pub _pInfraFlag: bool,
    pub tempDirection: Direction,
    pub _pLvlVisited: [bool; NUMLEVELS],
    pub _pSLvlVisited: [bool; NUMLEVELS],
    pub _pOilType: item_misc_id,
    pub pTownWarps: u8,
    pub pDungMsgs: u8,
    pub pLvlLoad: u8,
    pub pManaShield: bool,
    pub pDungMsgs2: u8,
    pub pOriginalCathedral: bool,
    pub pDiabloKillLevel: u8,
    pub wReflections: u16,
    pub pDamAcFlags: ItemSpecialEffectHf,
}

impl Default for Player {
    /// `Player {}`: value-initialised (zeroes), `friendlyMode` true.
    fn default() -> Self {
        Player {
            _pName: CStr::default(),
            InvBody: Default::default(),
            InvList: vec![Item::default(); InventoryGridCells],
            SpdList: Default::default(),
            HoldItem: Item::default(),
            lightId: 0,
            _pNumInv: 0,
            _pStrength: 0,
            _pBaseStr: 0,
            _pMagic: 0,
            _pBaseMag: 0,
            _pDexterity: 0,
            _pBaseDex: 0,
            _pVitality: 0,
            _pBaseVit: 0,
            _pStatPts: 0,
            _pDamageMod: 0,
            _pBaseToBlk: 0,
            _pHPBase: 0,
            _pMaxHPBase: 0,
            _pHitPoints: 0,
            _pMaxHP: 0,
            _pHPPer: 0,
            _pManaBase: 0,
            _pMaxManaBase: 0,
            _pMana: 0,
            _pMaxMana: 0,
            _pManaPer: 0,
            _pIMinDam: 0,
            _pIMaxDam: 0,
            _pIAC: 0,
            _pIBonusDam: 0,
            _pIBonusToHit: 0,
            _pIBonusAC: 0,
            _pIBonusDamMod: 0,
            _pIGetHit: 0,
            _pIEnAc: 0,
            _pIFMinDam: 0,
            _pIFMaxDam: 0,
            _pILMinDam: 0,
            _pILMaxDam: 0,
            _pExperience: 0,
            _pNextExper: 0,
            _pmode: PM_STAND,
            walkpath: [0; MaxPathLength],
            plractive: false,
            destAction: 0,
            destParam1: 0,
            destParam2: 0,
            destParam3: 0,
            destParam4: 0,
            _pGold: 0,
            AnimInfo: AnimationInfo::default(),
            previewCelSprite: None,
            progressToNextGameTickWhenPreviewWasSet: 0,
            _pIFlags: ItemSpecialEffect::None,
            AnimationData: Default::default(),
            _pNFrames: 0,
            _pWFrames: 0,
            _pAFrames: 0,
            _pAFNum: 0,
            _pSFrames: 0,
            _pSFNum: 0,
            _pHFrames: 0,
            _pDFrames: 0,
            _pBFrames: 0,
            InvGrid: [0; InventoryGridCells],
            plrlevel: 0,
            plrIsOnSetLevel: false,
            position: ActorPosition::default(),
            _pdir: Direction::South,
            _pClass: HeroClass::Warrior,
            _pLevel: 0,
            _pMaxLvl: 0,
            _pgfxnum: 0,
            _pISplLvlAdd: 0,
            friendlyMode: true,
            queuedSpell: SpellCastInfo::default(),
            executedSpell: SpellCastInfo::default(),
            inventorySpell: SpellID::Null,
            spellFrom: 0,
            _pRSpell: SpellID::Null,
            _pRSplType: SpellType::Skill,
            _pSBkSpell: SpellID::Null,
            _pSplLvl: [0; 64],
            _pISpells: 0,
            _pMemSpells: 0,
            _pAblSpells: 0,
            _pScrlSpells: 0,
            _pSpellFlags: SpellFlag(0),
            _pSplHotKey: [SpellID::Null; NumHotkeys],
            _pSplTHotKey: [SpellType::Skill; NumHotkeys],
            _pBlockFlag: false,
            _pInvincible: false,
            _pLightRad: 0,
            _pLvlChanging: false,
            _pArmorClass: 0,
            _pMagResist: 0,
            _pFireResist: 0,
            _pLghtResist: 0,
            _pInfraFlag: false,
            tempDirection: Direction::South,
            _pLvlVisited: [false; NUMLEVELS],
            _pSLvlVisited: [false; NUMLEVELS],
            _pOilType: IMISC_NONE,
            pTownWarps: 0,
            pDungMsgs: 0,
            pLvlLoad: 0,
            pManaShield: false,
            pDungMsgs2: 0,
            pOriginalCathedral: false,
            pDiabloKillLevel: 0,
            wReflections: 0,
            pDamAcFlags: ItemSpecialEffectHf::None,
        }
    }
}

impl Player {
    /// `CanUseItem`
    pub fn can_use_item(&self, item: &Item) -> bool {
        self._pStrength >= item._iMinStr as i32 && self._pMagic >= item._iMinMag as i32 && self._pDexterity >= item._iMinDex as i32
    }

    /// `GetMostValuableItem`: belt, then body, then inventory.
    pub fn get_most_valuable_item(&self, pred: &dyn Fn(&Item) -> bool) -> Option<&Item> {
        let mut best: Option<&Item> = None;
        let n = self._pNumInv as usize;
        for item in self.SpdList.iter().chain(self.InvBody.iter()).chain(self.InvList[..n].iter()) {
            if item.is_empty() || !pred(item) {
                continue;
            }
            if best.is_none_or(|b| item._iIvalue > b._iIvalue) {
                best = Some(item);
            }
        }
        best
    }

    /// Original: `Player::GetBaseAttributeValue` (player.cpp).
    // @port player.cpp|devilution::Player::GetBaseAttributeValue(CharacterAttribute attribute) sha=5e516c6275ba
    pub fn get_base_attribute_value(&self, attribute: CharacterAttribute) -> i32 {
        match attribute {
            CharacterAttribute::Dexterity => self._pBaseDex,
            CharacterAttribute::Magic => self._pBaseMag,
            CharacterAttribute::Strength => self._pBaseStr,
            CharacterAttribute::Vitality => self._pBaseVit,
        }
    }

    /// Original: `Player::GetCurrentAttributeValue` (player.cpp).
    // @port player.cpp|devilution::Player::GetCurrentAttributeValue(CharacterAttribute attribute) sha=cd2da45670c2
    pub fn get_current_attribute_value(&self, attribute: CharacterAttribute) -> i32 {
        match attribute {
            CharacterAttribute::Dexterity => self._pDexterity,
            CharacterAttribute::Magic => self._pMagic,
            CharacterAttribute::Strength => self._pStrength,
            CharacterAttribute::Vitality => self._pVitality,
        }
    }

    /// Original: `Player::GetMaximumAttributeValue` (player.cpp).
    // @port player.cpp|devilution::Player::GetMaximumAttributeValue(CharacterAttribute attribute) sha=f1538c1c34c1
    pub fn get_maximum_attribute_value(&self, attribute: CharacterAttribute) -> i32 {
        let d = &PlayersData[self._pClass as usize];
        match attribute {
            CharacterAttribute::Strength => d.maxStr as i32,
            CharacterAttribute::Magic => d.maxMag as i32,
            CharacterAttribute::Dexterity => d.maxDex as i32,
            CharacterAttribute::Vitality => d.maxVit as i32,
        }
    }

    /// Original: `Player::GetTargetPosition` (player.cpp).
    // @port player.cpp|devilution::Player::GetTargetPosition() sha=61fceec012e3
    pub fn get_target_position(&self) -> Point {
        const DIRECTION_OFFSET_X: [i32; 8] = [0, -1, 1, 0, -1, 1, 1, -1];
        const DIRECTION_OFFSET_Y: [i32; 8] = [-1, 0, 0, 1, -1, -1, 1, 1];
        let mut target = self.position.future;
        for &step in self.walkpath.iter() {
            if step as i32 == WALK_NONE {
                break;
            }
            if step > 0 {
                target.x += DIRECTION_OFFSET_X[step as usize - 1];
                target.y += DIRECTION_OFFSET_Y[step as usize - 1];
            }
        }
        target
    }

    /// Original: `Player::IsPositionInPath` (player.cpp).
    // @port player.cpp|devilution::Player::IsPositionInPath(Point pos) sha=8db266df7ce4
    pub fn is_position_in_path(&self, pos: Point) -> bool {
        const DIRECTION_OFFSET: [Displacement; 8] = [
            Displacement::new(0, -1),
            Displacement::new(-1, 0),
            Displacement::new(1, 0),
            Displacement::new(0, 1),
            Displacement::new(-1, -1),
            Displacement::new(1, -1),
            Displacement::new(1, 1),
            Displacement::new(-1, 1),
        ];
        let mut target = self.position.future;
        for &step in self.walkpath.iter() {
            if target == pos {
                return true;
            }
            if step as i32 == WALK_NONE {
                break;
            }
            if step > 0 {
                target += DIRECTION_OFFSET[step as usize - 1];
            }
        }
        false
    }

    /// Original: `Player::isWalking` (player.cpp).
    // @port player.cpp|devilution::Player::isWalking() sha=53ea10f7fd2f
    pub fn is_walking(&self) -> bool {
        matches!(self._pmode, PM_WALK_NORTHWARDS | PM_WALK_SOUTHWARDS | PM_WALK_SIDEWAYS)
    }

    /// Original: `Player::GetManaShieldDamageReduction` (player.cpp).
    // @port player.cpp|devilution::Player::GetManaShieldDamageReduction() sha=8cac608fd535
    pub fn get_mana_shield_damage_reduction(&self) -> i32 {
        const MAX: u8 = 7;
        24 - self._pSplLvl[SpellID::ManaShield as usize].min(MAX) as i32 * 3
    }

    /// `GetItemLocation`
    pub fn get_item_location(&self, item: &Item) -> item_equip_type {
        if self._pClass == HeroClass::Barbarian && item._iLoc == ILOC_TWOHAND && matches!(item._itype, ItemType::Sword | ItemType::Mace) {
            return ILOC_ONEHAND;
        }
        item._iLoc
    }

    /// `GetArmor`
    pub fn get_armor(&self) -> i32 {
        self._pIBonusAC + self._pIAC + self._pDexterity / 5
    }

    /// `GetMeleeToHit`
    pub fn get_melee_to_hit(&self) -> i32 {
        let mut hper = self._pLevel as i32 + self._pDexterity / 2 + self._pIBonusToHit + BaseHitChance;
        if self._pClass == HeroClass::Warrior {
            hper += 20;
        }
        hper
    }

    /// `GetMeleePiercingToHit`
    pub fn get_melee_piercing_to_hit(&self, hellfire: bool) -> i32 {
        let mut hper = self.get_melee_to_hit();
        if !hellfire {
            hper += self._pIEnAc;
        }
        hper
    }

    /// `GetRangedToHit`
    pub fn get_ranged_to_hit(&self) -> i32 {
        let mut hper = self._pLevel as i32 + self._pDexterity + self._pIBonusToHit + BaseHitChance;
        if self._pClass == HeroClass::Rogue {
            hper += 20;
        } else if self._pClass == HeroClass::Warrior || self._pClass == HeroClass::Bard {
            hper += 10;
        }
        hper
    }

    /// `GetRangedPiercingToHit`
    pub fn get_ranged_piercing_to_hit(&self, hellfire: bool) -> i32 {
        let mut hper = self.get_ranged_to_hit();
        if !hellfire {
            hper += self._pIEnAc;
        }
        hper
    }

    /// `GetMagicToHit`
    pub fn get_magic_to_hit(&self) -> i32 {
        let mut hper = self._pMagic + BaseHitChance;
        if self._pClass == HeroClass::Sorcerer {
            hper += 20;
        } else if self._pClass == HeroClass::Bard {
            hper += 10;
        }
        hper
    }

    /// `GetBlockChance` (`useLevel` defaults to true)
    pub fn get_block_chance(&self, use_level: bool) -> i32 {
        let mut blkper = self._pDexterity + self._pBaseToBlk;
        if use_level {
            blkper += self._pLevel as i32 * 2;
        }
        blkper
    }

    /// `GetSpellLevel`
    pub fn get_spell_level(&self, spell: SpellID) -> i32 {
        if spell == SpellID::Invalid || spell as i8 as usize >= self._pSplLvl.len() {
            return 0;
        }
        (self._pISplLvlAdd as i32 + self._pSplLvl[spell as i8 as usize] as i32).max(0)
    }

    /// `CalculateArmorPierce`
    pub fn calculate_armor_pierce(&self, monster_armor: i32, is_melee: bool, hellfire: bool) -> i32 {
        let mut tmac = monster_armor;
        if self._pIEnAc > 0 {
            if hellfire {
                let p_i_en_ac = self._pIEnAc - 1;
                if p_i_en_ac > 0 {
                    tmac >>= p_i_en_ac;
                } else {
                    tmac -= tmac / 4;
                }
            }
            if is_melee && self._pClass == HeroClass::Barbarian {
                tmac -= monster_armor / 8;
            }
        }
        tmac.max(0)
    }

    /// `UpdateHitPointPercentage`
    pub fn update_hit_point_percentage(&mut self) -> i32 {
        if self._pMaxHP <= 0 {
            self._pHPPer = 0;
        } else {
            self._pHPPer = (self._pHitPoints * 80 / self._pMaxHP).clamp(0, 80);
        }
        self._pHPPer
    }

    /// `UpdateManaPercentage`
    pub fn update_mana_percentage(&mut self) -> i32 {
        if self._pMaxMana <= 0 {
            self._pManaPer = 0;
        } else {
            self._pManaPer = (self._pMana * 80 / self._pMaxMana).clamp(0, 80);
        }
        self._pManaPer
    }

    /// `RestoreFullLife`
    pub fn restore_full_life(&mut self) {
        self._pHitPoints = self._pMaxHP;
        self._pHPBase = self._pMaxHPBase;
    }

    /// `RestoreFullMana`
    pub fn restore_full_mana(&mut self) {
        if self._pIFlags.has_none_of(ItemSpecialEffect::NoMana) {
            self._pMana = self._pMaxMana;
            self._pManaBase = self._pMaxManaBase;
        }
    }

    /// `UsesRangedWeapon`
    pub fn uses_ranged_weapon(&self) -> bool {
        self._pgfxnum & 0xF == PlayerWeaponGraphic::Bow as u8
    }

    /// `CanChangeAction`
    pub fn can_change_action(&self) -> bool {
        match self._pmode {
            PM_STAND => true,
            PM_ATTACK | PM_RATTACK if self.AnimInfo.currentFrame >= self._pAFNum => true,
            PM_SPELL if self.AnimInfo.currentFrame >= self._pSFNum => true,
            _ => self.is_walking() && self.AnimInfo.is_last_frame(),
        }
    }

    /// Original: `Player::getGraphic` (player.cpp).
    // @port player.cpp|devilution::Player::getGraphic() sha=79e574a8c115
    pub fn get_graphic(&self) -> player_graphic {
        match self._pmode {
            PM_STAND | PM_NEWLVL | PM_QUIT => player_graphic::Stand,
            PM_WALK_NORTHWARDS | PM_WALK_SOUTHWARDS | PM_WALK_SIDEWAYS => player_graphic::Walk,
            PM_ATTACK | PM_RATTACK => player_graphic::Attack,
            PM_BLOCK => player_graphic::Block,
            PM_SPELL => get_player_graphic_for_spell(self.executedSpell.spellId),
            PM_GOTHIT => player_graphic::Hit,
            PM_DEATH => player_graphic::Death,
            _ => panic!("SyncPlrAnim"),
        }
    }

    /// Original: `Player::getAnimationFramesAndTicksPerFrame` (player.cpp): (frames, ticks).
    // @port player.cpp|devilution::Player::getAnimationFramesAndTicksPerFrame(player_graphic graphics, int8_t &numberOfFrames, int8_t &ticksPerFrame) sha=4ddb3b36ec78
    pub fn get_animation_frames_and_ticks_per_frame(&self, graphics: player_graphic) -> (i8, i8) {
        match graphics {
            player_graphic::Stand => (self._pNFrames, 4),
            player_graphic::Walk => (self._pWFrames, 1),
            player_graphic::Attack => (self._pAFrames, 1),
            player_graphic::Hit => (self._pHFrames, 1),
            player_graphic::Lightning | player_graphic::Fire | player_graphic::Magic => (self._pSFrames, 1),
            player_graphic::Death => (self._pDFrames, 2),
            player_graphic::Block => (self._pBFrames, 3),
        }
    }

    /// Original: `Player::calculateBaseLife` (player.cpp).
    // @port player.cpp|devilution::Player::calculateBaseLife() sha=c61909aa479b
    pub fn calculate_base_life(&self) -> i32 {
        let d = &PlayersData[self._pClass as usize];
        d.adjLife as i32 + d.lvlLife as i32 * self._pLevel as i32 + d.chrLife as i32 * self._pBaseVit
    }

    /// Original: `Player::calculateBaseMana` (player.cpp).
    // @port player.cpp|devilution::Player::calculateBaseMana() sha=410bda6ceafc
    pub fn calculate_base_mana(&self) -> i32 {
        let d = &PlayersData[self._pClass as usize];
        d.adjMana as i32 + d.lvlMana as i32 * self._pLevel as i32 + d.chrMana as i32 * self._pBaseMag
    }

    /// `isOnLevel(uint8_t)`
    pub fn is_on_level(&self, level: u8) -> bool {
        !self.plrIsOnSetLevel && self.plrlevel == level
    }

    /// `isOnLevel(_setlevels)`
    pub fn is_on_set_level(&self, level: _setlevels) -> bool {
        self.plrIsOnSetLevel && self.plrlevel == level as u8
    }

    /// `isOnArenaLevel`
    pub fn is_on_arena_level(&self) -> bool {
        self.plrIsOnSetLevel && crate::levels::gendung::is_arena_level(self.plrlevel as _setlevels)
    }

    /// `setLevel(uint8_t)`
    pub fn set_level(&mut self, level: u8) {
        self.plrlevel = level;
        self.plrIsOnSetLevel = false;
    }

    /// `setLevel(_setlevels)`
    pub fn set_set_level(&mut self, level: _setlevels) {
        self.plrlevel = level as u8;
        self.plrIsOnSetLevel = true;
    }
}

/// Globals of player.cpp.
#[derive(Default)]
pub struct PlayerState {
    /// `MyPlayerId`
    pub MyPlayerId: usize,
    /// `MyPlayer` (index into `Players`)
    pub MyPlayer: Option<usize>,
    pub Players: Vec<Player>,
    /// `InspectPlayer` (index into `Players`)
    pub InspectPlayer: Option<usize>,
    pub MyPlayerIsDead: bool,
}

impl PlayerState {
    pub fn my_player_id(&self) -> usize {
        self.MyPlayer.expect("MyPlayer")
    }
}

/// `MyPlayer != nullptr`
pub fn my_player_exists(ctx: &Ctx) -> bool {
    ctx.players.MyPlayer.is_some()
}

/// `IsInspectingPlayer`
pub fn is_inspecting_player(ctx: &Ctx) -> bool {
    ctx.players.MyPlayer != ctx.players.InspectPlayer
}

/// `Player::isOnActiveLevel`
pub fn is_on_active_level(ctx: &Ctx, pnum: usize) -> bool {
    let p = &ctx.players.Players[pnum];
    if ctx.gendung.setlevel {
        return p.is_on_set_level(ctx.gendung.setlvlnum);
    }
    p.is_on_level(ctx.gendung.currlevel)
}

/// `MyPlayer->HoldItem.clear()` from `NewCursor`.
pub fn clear_my_hold_item(ctx: &mut Ctx) {
    if let Some(m) = ctx.players.MyPlayer {
        ctx.players.Players[m].HoldItem.clear();
    }
}

/// `Players.clear()` (NetClose).
pub fn clear_players(ctx: &mut Ctx) {
    ctx.players.Players.clear();
}

fn is_my(ctx: &Ctx, pnum: usize) -> bool {
    ctx.players.MyPlayer == Some(pnum)
}

fn invincible_dead_me(ctx: &Ctx, pnum: usize) -> bool {
    let p = &ctx.players.Players[pnum];
    p._pInvincible && p._pHitPoints == 0 && is_my(ctx, pnum)
}

/// `DirectionSettings`
struct DirectionSettings {
    dir: Direction,
    tile_add: Displacement,
    map: Displacement,
    walk_mode: PLR_MODE,
    walk_mode_handler: fn(&mut Ctx, usize, &DirectionSettings),
}

/// Original: `UpdatePlayerLightOffset` (player.cpp).
// @port player.cpp|devilution::UpdatePlayerLightOffset(Player &player) sha=ade06373fa19
fn update_player_light_offset(ctx: &mut Ctx, pnum: usize) {
    let p = &ctx.players.Players[pnum];
    if p.lightId == NO_LIGHT {
        return;
    }
    let progress = ctx.nthread.ProgressToNextGameTick;
    let offset = p.position.calculate_walking_offset(p._pdir, &p.AnimInfo, progress);
    let id = p.lightId;
    crate::lighting::change_light_offset(ctx, id, offset.screen_to_light());
}

/// Original: `WalkNorthwards` (player.cpp).
// @port player.cpp|devilution::WalkNorthwards(Player &player, const DirectionSettings &walkParams) sha=ad098000610b
fn walk_northwards(ctx: &mut Ctx, pnum: usize, walk_params: &DirectionSettings) {
    let f = ctx.players.Players[pnum].position.future;
    ctx.gendung.dPlayer[f.x as usize][f.y as usize] = -((pnum + 1) as i8);
    let p = &mut ctx.players.Players[pnum];
    p.position.temp = p.position.tile + walk_params.tile_add;
}

/// Original: `WalkSouthwards` (player.cpp).
// @port player.cpp|devilution::WalkSouthwards(Player &player, const DirectionSettings &) sha=b0df398b00d1
fn walk_southwards(ctx: &mut Ctx, pnum: usize, _walk_params: &DirectionSettings) {
    let t = ctx.players.Players[pnum].position.tile;
    ctx.gendung.dPlayer[t.x as usize][t.y as usize] = -((pnum + 1) as i8);
    let p = &mut ctx.players.Players[pnum];
    p.position.temp = p.position.tile;
    p.position.tile = p.position.future;
    let t = p.position.tile;
    ctx.gendung.dPlayer[t.x as usize][t.y as usize] = (pnum + 1) as i8;
    // BUGFIX: missing `if (leveltype != DTYPE_TOWN) {` for call to ChangeLightXY and PM_ChangeLightOff.
    let id = ctx.players.Players[pnum].lightId;
    crate::lighting::change_light_xy(ctx, id, t);
    update_player_light_offset(ctx, pnum);
}

/// Original: `WalkSideways` (player.cpp).
// @port player.cpp|devilution::WalkSideways(Player &player, const DirectionSettings &walkParams) sha=d3b4db3ffe4f
fn walk_sideways(ctx: &mut Ctx, pnum: usize, walk_params: &DirectionSettings) {
    let p = &ctx.players.Players[pnum];
    let next_position = p.position.tile + walk_params.map;
    let (t, f) = (p.position.tile, p.position.future);
    ctx.gendung.dPlayer[t.x as usize][t.y as usize] = -((pnum + 1) as i8);
    ctx.gendung.dPlayer[f.x as usize][f.y as usize] = (pnum + 1) as i8;
    if ctx.gendung.leveltype != DungeonType::Town {
        let id = ctx.players.Players[pnum].lightId;
        crate::lighting::change_light_xy(ctx, id, next_position);
        update_player_light_offset(ctx, pnum);
    }
    let p = &mut ctx.players.Players[pnum];
    p.position.temp = p.position.future;
}

const WALK_SETTINGS: [DirectionSettings; 8] = [
    DirectionSettings { dir: Direction::South, tile_add: Displacement::new(1, 1), map: Displacement::new(0, 0), walk_mode: PM_WALK_SOUTHWARDS, walk_mode_handler: walk_southwards },
    DirectionSettings { dir: Direction::SouthWest, tile_add: Displacement::new(0, 1), map: Displacement::new(0, 0), walk_mode: PM_WALK_SOUTHWARDS, walk_mode_handler: walk_southwards },
    DirectionSettings { dir: Direction::West, tile_add: Displacement::new(-1, 1), map: Displacement::new(0, 1), walk_mode: PM_WALK_SIDEWAYS, walk_mode_handler: walk_sideways },
    DirectionSettings { dir: Direction::NorthWest, tile_add: Displacement::new(-1, 0), map: Displacement::new(0, 0), walk_mode: PM_WALK_NORTHWARDS, walk_mode_handler: walk_northwards },
    DirectionSettings { dir: Direction::North, tile_add: Displacement::new(-1, -1), map: Displacement::new(0, 0), walk_mode: PM_WALK_NORTHWARDS, walk_mode_handler: walk_northwards },
    DirectionSettings { dir: Direction::NorthEast, tile_add: Displacement::new(0, -1), map: Displacement::new(0, 0), walk_mode: PM_WALK_NORTHWARDS, walk_mode_handler: walk_northwards },
    DirectionSettings { dir: Direction::East, tile_add: Displacement::new(1, -1), map: Displacement::new(1, 0), walk_mode: PM_WALK_SIDEWAYS, walk_mode_handler: walk_sideways },
    DirectionSettings { dir: Direction::SouthEast, tile_add: Displacement::new(1, 0), map: Displacement::new(0, 0), walk_mode: PM_WALK_SOUTHWARDS, walk_mode_handler: walk_southwards },
];

/// Original: `PlrDirOK` (player.cpp).
// @port player.cpp|devilution::PlrDirOK(const Player &player, Direction dir) sha=2c94902a6a45
fn plr_dir_ok(ctx: &Ctx, pnum: usize, dir: Direction) -> bool {
    let position = ctx.players.Players[pnum].position.tile;
    let future_position = position + dir;
    if future_position.x < 0 || !pos_ok_player(ctx, pnum, future_position) {
        return false;
    }
    if dir == Direction::East {
        return !crate::engine::path::is_tile_solid(ctx, position + Direction::SouthEast);
    }
    if dir == Direction::West {
        return !crate::engine::path::is_tile_solid(ctx, position + Direction::SouthWest);
    }
    true
}

/// Original: `HandleWalkMode` (player.cpp).
// @port player.cpp|devilution::HandleWalkMode(Player &player, Direction dir) sha=9c0536e670e5
fn handle_walk_mode(ctx: &mut Ctx, pnum: usize, dir: Direction) {
    let params = &WALK_SETTINGS[dir as usize];
    set_player_old(ctx, pnum);
    if !plr_dir_ok(ctx, pnum, dir) {
        return;
    }
    {
        let p = &mut ctx.players.Players[pnum];
        p._pdir = dir;
        p.position.future = p.position.tile + params.tile_add;
    }
    (params.walk_mode_handler)(ctx, pnum, params);
    let p = &mut ctx.players.Players[pnum];
    p.tempDirection = params.dir;
    p._pmode = params.walk_mode;
}

/// Original: `StartWalkAnimation` (player.cpp).
// @port player.cpp|devilution::StartWalkAnimation(Player &player, Direction dir, bool pmWillBeCalled) sha=271d03717272
fn start_walk_animation(ctx: &mut Ctx, pnum: usize, dir: Direction, pm_will_be_called: bool) {
    let mut skipped_frames: i8 = -2;
    if ctx.gendung.leveltype == DungeonType::Town && ctx.multi.sgGameInitInfo.bRunInTown != 0 {
        skipped_frames = 2;
    }
    if pm_will_be_called {
        skipped_frames += 1;
    }
    new_plr_anim(ctx, pnum, player_graphic::Walk, dir, AnimationDistributionFlags::ProcessAnimationPending, skipped_frames, 0);
}

/// Original: `StartWalk` (player.cpp).
// @port player.cpp|devilution::StartWalk(Player &player, Direction dir, bool pmWillBeCalled) sha=9ac7e64a0513
fn start_walk(ctx: &mut Ctx, pnum: usize, dir: Direction, pm_will_be_called: bool) {
    if invincible_dead_me(ctx, pnum) {
        sync_plr_kill(ctx, pnum, DeathReason::Unknown);
        return;
    }
    start_walk_animation(ctx, pnum, dir, pm_will_be_called);
    handle_walk_mode(ctx, pnum, dir);
}

/// Original: `ClearStateVariables` (player.cpp).
// @port player.cpp|devilution::ClearStateVariables(Player &player) sha=01e70d6a3146
fn clear_state_variables(ctx: &mut Ctx, pnum: usize) {
    let p = &mut ctx.players.Players[pnum];
    p.position.temp = Point::new(0, 0);
    p.tempDirection = Direction::South;
    p.queuedSpell.spellLevel = 0;
}

/// Original: `StartAttack` (player.cpp).
// @port player.cpp|devilution::StartAttack(Player &player, Direction d, bool includesFirstFrame) sha=c95b2418a3cc
fn start_attack(ctx: &mut Ctx, pnum: usize, d: Direction, includes_first_frame: bool) {
    if invincible_dead_me(ctx, pnum) {
        sync_plr_kill(ctx, pnum, DeathReason::Unknown);
        return;
    }
    use ItemSpecialEffect as E;
    let p = &ctx.players.Players[pnum];
    let f = p._pIFlags;
    let mut skipped_animation_frames: i8 = 0;
    if includes_first_frame {
        if f.has_any_of(E::FastestAttack) && f.has_any_of(E::QuickAttack | E::FastAttack) {
            skipped_animation_frames = 3;
        } else if f.has_any_of(E::FastestAttack) {
            skipped_animation_frames = 4;
        } else if f.has_any_of(E::FasterAttack) {
            skipped_animation_frames = 3;
        } else if f.has_any_of(E::FastAttack) {
            skipped_animation_frames = 2;
        } else if f.has_any_of(E::QuickAttack) {
            skipped_animation_frames = 1;
        }
    } else if f.has_any_of(E::FasterAttack) {
        skipped_animation_frames = 2;
    } else if f.has_any_of(E::FastAttack) {
        skipped_animation_frames = 1;
    } else if f.has_any_of(E::FastestAttack) {
        skipped_animation_frames = 2;
    }
    let mut animation_flags = AnimationDistributionFlags::ProcessAnimationPending;
    if p._pmode == PM_ATTACK {
        animation_flags = animation_flags | AnimationDistributionFlags::RepeatedAction;
    }
    let af = p._pAFNum;
    new_plr_anim(ctx, pnum, player_graphic::Attack, d, animation_flags, skipped_animation_frames, af);
    ctx.players.Players[pnum]._pmode = PM_ATTACK;
    fix_player_location(ctx, pnum, d);
    set_player_old(ctx, pnum);
}

/// Original: `StartRangeAttack` (player.cpp).
// @port player.cpp|devilution::StartRangeAttack(Player &player, Direction d, WorldTileCoord cx, WorldTileCoord cy, bool includesFirstFrame) sha=a0b361c79b63
fn start_range_attack(ctx: &mut Ctx, pnum: usize, d: Direction, cx: i32, cy: i32, includes_first_frame: bool) {
    if invincible_dead_me(ctx, pnum) {
        sync_plr_kill(ctx, pnum, DeathReason::Unknown);
        return;
    }
    use ItemSpecialEffect as E;
    let p = &ctx.players.Players[pnum];
    let mut skipped_animation_frames: i8 = 0;
    if !ctx.init.gb_is_hellfire {
        if includes_first_frame && p._pIFlags.has_any_of(E::QuickAttack | E::FastAttack) {
            skipped_animation_frames += 1;
        }
        if p._pIFlags.has_any_of(E::FastAttack) {
            skipped_animation_frames += 1;
        }
    }
    let mut animation_flags = AnimationDistributionFlags::ProcessAnimationPending;
    if p._pmode == PM_RATTACK {
        animation_flags = animation_flags | AnimationDistributionFlags::RepeatedAction;
    }
    let af = p._pAFNum;
    new_plr_anim(ctx, pnum, player_graphic::Attack, d, animation_flags, skipped_animation_frames, af);
    ctx.players.Players[pnum]._pmode = PM_RATTACK;
    fix_player_location(ctx, pnum, d);
    set_player_old(ctx, pnum);
    // WorldTileCoord is uint8_t
    ctx.players.Players[pnum].position.temp = Point::new(cx & 0xff, cy & 0xff);
}

/// Original: `GetPlayerGraphicForSpell` (player.cpp).
// @port player.cpp|devilution::GetPlayerGraphicForSpell(SpellID spellId) sha=0cbc4f2476c0
fn get_player_graphic_for_spell(spell_id: SpellID) -> player_graphic {
    match crate::items::spell_magic_type(spell_id) {
        MagicType::Fire => player_graphic::Fire,
        MagicType::Lightning => player_graphic::Lightning,
        _ => player_graphic::Magic,
    }
}

/// Original: `StartSpell` (player.cpp).
// @port player.cpp|devilution::StartSpell(Player &player, Direction d, WorldTileCoord cx, WorldTileCoord cy) sha=aaead6ed2347
fn start_spell(ctx: &mut Ctx, pnum: usize, d: Direction, cx: i32, cy: i32) {
    if invincible_dead_me(ctx, pnum) {
        sync_plr_kill(ctx, pnum, DeathReason::Unknown);
        return;
    }
    let qs = ctx.players.Players[pnum].queuedSpell;
    let is_valid = match qs.spellType {
        SpellType::Skill | SpellType::Spell => crate::spells::check_spell(ctx, pnum, qs.spellId, qs.spellType, true) == SpellCheckResult::Success,
        SpellType::Scroll => crate::spells::can_use_scroll(ctx, pnum, qs.spellId),
        SpellType::Charges => crate::spells::can_use_staff(ctx, pnum, qs.spellId),
        SpellType::Invalid => false,
    };
    if !is_valid {
        return;
    }
    let p = &ctx.players.Players[pnum];
    let mut animation_flags = AnimationDistributionFlags::ProcessAnimationPending;
    if p._pmode == PM_SPELL {
        animation_flags = animation_flags | AnimationDistributionFlags::RepeatedAction;
    }
    let sf = p._pSFNum;
    new_plr_anim(ctx, pnum, get_player_graphic_for_spell(qs.spellId), d, animation_flags, 0, sf);
    let tile = ctx.players.Players[pnum].position.tile;
    crate::effects::play_sfx_loc(ctx, crate::items::get_spell_data(qs.spellId).sSFX, tile, true);
    ctx.players.Players[pnum]._pmode = PM_SPELL;
    fix_player_location(ctx, pnum, d);
    set_player_old(ctx, pnum);
    let p = &mut ctx.players.Players[pnum];
    p.position.temp = Point::new(cx & 0xff, cy & 0xff);
    p.queuedSpell.spellLevel = p.get_spell_level(p.queuedSpell.spellId);
    p.executedSpell = p.queuedSpell;
}

/// Original: `RespawnDeadItem` (player.cpp).
// @port player.cpp|devilution::RespawnDeadItem(Item &&itm, Point target) sha=38b1f43b0b4e
fn respawn_dead_item(ctx: &mut Ctx, itm: Item, target: Point) {
    if ctx.items.ActiveItemCount as usize >= crate::items::MAXITEMS {
        return;
    }
    let ii = crate::items::allocate_item(ctx) as usize;
    ctx.items.dItem[target.x as usize][target.y as usize] = (ii + 1) as i8;
    ctx.items.Items[ii] = itm;
    ctx.items.Items[ii].position = target;
    crate::items::with_item(ctx, ii, |ctx, it| crate::items::respawn_item(ctx, it, true));
    { let it = ctx.items.Items[ii as usize].clone(); crate::msg::net_send_cmd_p_item(ctx, false, CMD_SPAWNITEM, target, &it); }
}

/// Original: `DeadItem` (player.cpp).
// @port player.cpp|devilution::DeadItem(Player &player, Item &&itm, Displacement direction) sha=3b07a9c66ba8
fn dead_item(ctx: &mut Ctx, pnum: usize, itm: Item, direction: Displacement) {
    if itm.is_empty() {
        return;
    }
    let tile = ctx.players.Players[pnum].position.tile;
    let target = tile + direction;
    if direction != Displacement::new(0, 0) && crate::items::item_space_ok(ctx, target) {
        respawn_dead_item(ctx, itm, target);
        return;
    }
    for k in 1..50 {
        for j in -k..=k {
            for i in -k..=k {
                let next = tile + Displacement::new(i, j);
                if crate::items::item_space_ok(ctx, next) {
                    respawn_dead_item(ctx, itm, next);
                    return;
                }
            }
        }
    }
}

/// Original: `DropGold` (player.cpp).
// @port player.cpp|devilution::DropGold(Player &player, int amount, bool skipFullStacks) sha=62729a270359
fn drop_gold(ctx: &mut Ctx, pnum: usize, mut amount: i32, skip_full_stacks: bool) -> i32 {
    let mut i: i32 = 0;
    while i < ctx.players.Players[pnum]._pNumInv && amount > 0 {
        let item = &ctx.players.Players[pnum].InvList[i as usize];
        if item._itype != ItemType::Gold || (skip_full_stacks && item._ivalue == ctx.items.MaxGold) {
            i += 1;
            continue;
        }
        if amount < item._ivalue {
            let mut gold_item = Item::default();
            crate::items::make_gold_stack(ctx, &mut gold_item, amount);
            dead_item(ctx, pnum, gold_item, Displacement::new(0, 0));
            ctx.players.Players[pnum].InvList[i as usize]._ivalue -= amount;
            return 0;
        }
        amount -= item._ivalue;
        let it = ctx.players.Players[pnum].InvList[i as usize].clone();
        dead_item(ctx, pnum, it, Displacement::new(0, 0));
        remove_inv_item(ctx, pnum, i, true);
        i = 0;
    }
    amount
}

/// Original: `DropHalfPlayersGold` (player.cpp).
// @port player.cpp|devilution::DropHalfPlayersGold(Player &player) sha=4ab05e377401
fn drop_half_players_gold(ctx: &mut Ctx, pnum: usize) {
    let half = ctx.players.Players[pnum]._pGold / 2;
    let remaining_gold = drop_gold(ctx, pnum, half, true);
    if remaining_gold > 0 {
        drop_gold(ctx, pnum, remaining_gold, false);
    }
    ctx.players.Players[pnum]._pGold /= 2;
}

/// Original: `InitLevelChange` (player.cpp).
// @port player.cpp|devilution::InitLevelChange(Player &player) sha=58975694a9e0
fn init_level_change(ctx: &mut Ctx, pnum: usize) {
    let me = ctx.players.my_player_id();
    remove_plr_missiles(ctx, pnum);
    {
        let p = &mut ctx.players.Players[pnum];
        p.pManaShield = false;
        p.wReflections = 0;
    }
    if pnum != me {
        if ctx.players.Players[me].pManaShield {
            crate::msg::net_send_cmd(ctx, true, CMD_SETSHIELD);
        }
        let r = ctx.players.Players[me].wReflections;
        crate::msg::net_send_cmd_param1(ctx, true, CMD_SETREFLECT, r);
    } else if crate::minitext::qtextflag(ctx) {
        crate::minitext::set_qtextflag(ctx, false);
        crate::effects::stream_stop(ctx);
    }
    fix_plr_walk_tags(ctx, pnum);
    set_player_old(ctx, pnum);
    if pnum == me {
        let t = ctx.players.Players[pnum].position.tile;
        ctx.gendung.dPlayer[t.x as usize][t.y as usize] = (pnum + 1) as i8;
    } else {
        let p = &mut ctx.players.Players[pnum];
        let l = p.plrlevel as usize;
        p._pLvlVisited[l] = true;
    }
    clr_plr_path(ctx, pnum);
    let p = &mut ctx.players.Players[pnum];
    p.destAction = ACTION_NONE;
    p._pLvlChanging = true;
    if pnum == me {
        p.pLvlLoad = 10;
    }
}

/// Original: `DoWalk` (player.cpp).
// @port player.cpp|devilution::DoWalk(Player &player, int variant) sha=de242efa76c1
fn do_walk(ctx: &mut Ctx, pnum: usize, variant: PLR_MODE) -> bool {
    if ctx.options.audio.walking_sound.get() && (ctx.gendung.leveltype != DungeonType::Town || ctx.multi.sgGameInitInfo.bRunInTown == 0) {
        let cf = ctx.players.Players[pnum].AnimInfo.currentFrame;
        if cf == 0 || cf == 4 {
            let t = ctx.players.Players[pnum].position.tile;
            crate::effects::play_sfx_loc(ctx, crate::effects_data::PS_WALK1, t, true);
        }
    }
    if !ctx.players.Players[pnum].AnimInfo.is_last_frame() {
        update_player_light_offset(ctx, pnum);
        return false;
    }
    let id = (pnum + 1) as i8;
    {
        let g = &mut ctx.gendung;
        let p = &mut ctx.players.Players[pnum];
        match variant {
            PM_WALK_NORTHWARDS => {
                g.dPlayer[p.position.tile.x as usize][p.position.tile.y as usize] = 0;
                p.position.tile = p.position.temp;
                g.dPlayer[p.position.tile.x as usize][p.position.tile.y as usize] = id;
            }
            PM_WALK_SOUTHWARDS => {
                g.dPlayer[p.position.temp.x as usize][p.position.temp.y as usize] = 0;
            }
            PM_WALK_SIDEWAYS => {
                g.dPlayer[p.position.tile.x as usize][p.position.tile.y as usize] = 0;
                p.position.tile = p.position.temp;
                g.dPlayer[p.position.tile.x as usize][p.position.tile.y as usize] = id;
            }
            _ => {}
        }
    }
    let tile = ctx.players.Players[pnum].position.tile;
    if ctx.gendung.leveltype != DungeonType::Town {
        let lid = ctx.players.Players[pnum].lightId;
        crate::lighting::change_light_xy(ctx, lid, tile);
        crate::lighting::change_vision_xy(ctx, pnum as i32, tile);
    }
    let td = ctx.players.Players[pnum].tempDirection;
    start_stand(ctx, pnum, td);
    clear_state_variables(ctx, pnum);
    if ctx.gendung.leveltype != DungeonType::Town {
        let lid = ctx.players.Players[pnum].lightId;
        crate::lighting::change_light_offset(ctx, lid, Displacement::new(0, 0));
    }
    crate::qol::autopickup::auto_pickup(ctx, pnum);
    true
}

/// Original: `WeaponDecay` (player.cpp).
// @port player.cpp|devilution::WeaponDecay(Player &player, int ii) sha=871ecaf5ac4f
fn weapon_decay(ctx: &mut Ctx, pnum: usize, ii: usize) -> bool {
    let it = &mut ctx.players.Players[pnum].InvBody[ii];
    if !it.is_empty() && it._iClass == ICLASS_WEAPON && it._iDamAcFlags.has_any_of(ItemSpecialEffectHf::Decay) {
        it._iPLDam = (it._iPLDam as i32 - 5) as i16;
        if it._iPLDam <= -100 {
            crate::inv::remove_equipment(ctx, pnum, ii as inv_body_loc, true);
            crate::items::calc_plr_inv(ctx, pnum, true);
            return true;
        }
        crate::items::calc_plr_inv(ctx, pnum, true);
    }
    false
}

/// Durability loss on one hand slot for `DamageWeapon` / `DamageParryItem`.
/// Returns `None` to continue, `Some(r)` to return `r` from the caller.
fn wear_body_item(ctx: &mut Ctx, pnum: usize, loc: inv_body_loc, break_at_le_zero: bool) -> Option<bool> {
    let it = &mut ctx.players.Players[pnum].InvBody[loc as usize];
    if it._iDurability == DUR_INDESTRUCTIBLE {
        return Some(false);
    }
    it._iDurability -= 1;
    let broke = if break_at_le_zero { it._iDurability <= 0 } else { it._iDurability == 0 };
    if broke {
        crate::inv::remove_equipment(ctx, pnum, loc, true);
        crate::items::calc_plr_inv(ctx, pnum, true);
        return Some(true);
    }
    None
}

/// Original: `DamageWeapon` (player.cpp).
// @port player.cpp|devilution::DamageWeapon(Player &player, unsigned damageFrequency) sha=1da1ebfb1ac9
fn damage_weapon(ctx: &mut Ctx, pnum: usize, damage_frequency: u32) -> bool {
    if !is_my(ctx, pnum) {
        return false;
    }
    if weapon_decay(ctx, pnum, INVLOC_HAND_LEFT as usize) {
        return true;
    }
    if weapon_decay(ctx, pnum, INVLOC_HAND_RIGHT as usize) {
        return true;
    }
    if !ctx.rng.flip_coin(damage_frequency) {
        return false;
    }
    let (l, r) = (INVLOC_HAND_LEFT as usize, INVLOC_HAND_RIGHT as usize);
    let body = |ctx: &Ctx, i: usize| -> (bool, item_class, ItemType) {
        let it = &ctx.players.Players[pnum].InvBody[i];
        (it.is_empty(), it._iClass, it._itype)
    };
    let (le, lc, _) = body(ctx, l);
    if !le && lc == ICLASS_WEAPON {
        if let Some(v) = wear_body_item(ctx, pnum, INVLOC_HAND_LEFT, true) {
            return v;
        }
    }
    let (re, rc, _) = body(ctx, r);
    if !re && rc == ICLASS_WEAPON {
        if let Some(v) = wear_body_item(ctx, pnum, INVLOC_HAND_RIGHT, false) {
            return v;
        }
    }
    let (le, _, _) = body(ctx, l);
    let (_, _, rt) = body(ctx, r);
    if le && rt == ItemType::Shield {
        if let Some(v) = wear_body_item(ctx, pnum, INVLOC_HAND_RIGHT, false) {
            return v;
        }
    }
    let (re, _, _) = body(ctx, r);
    let (_, _, lt) = body(ctx, l);
    if re && lt == ItemType::Shield {
        if let Some(v) = wear_body_item(ctx, pnum, INVLOC_HAND_LEFT, false) {
            return v;
        }
    }
    false
}

/// Original: `PlrHitMonst` (player.cpp). `adjacentDamage` defaults to false.
// @port player.cpp|devilution::PlrHitMonst(Player &player, Monster &monster, bool adjacentDamage = false) sha=f1f938155068
fn plr_hit_monst(ctx: &mut Ctx, pnum: usize, m: usize, adjacent_damage: bool) -> bool {
    use crate::monster as mon;
    let mut hper = 0;
    if !mon::is_possible_to_hit(ctx, m) {
        return false;
    }
    let plevel = ctx.players.Players[pnum]._pLevel as i32;
    if adjacent_damage {
        if plevel > 20 {
            hper -= 30;
        } else {
            hper -= (35 - plevel) * 2;
        }
    }
    let mut hit = ctx.rng.generate_rnd(100);
    if ctx.monster.Monsters[m].mode == MonsterMode::Petrified {
        hit = 0;
    }
    let hf = ctx.init.gb_is_hellfire;
    {
        let p = &ctx.players.Players[pnum];
        hper += p.get_melee_piercing_to_hit(hf) - p.calculate_armor_pierce(ctx.monster.Monsters[m].armorClass as i32, true, hf);
    }
    hper = hper.clamp(5, 95);
    if mon::try_lift_gargoyle(ctx, m) {
        return true;
    }
    if hit >= hper {
        return false;
    }
    use ItemSpecialEffect as E;
    use ItemSpecialEffectHf as H;
    let p = ctx.players.Players[pnum].clone();
    if hf && p._pIFlags.has_all_of(E::FireDamage | E::LightningDamage) {
        let midam = p._pIFMinDam + ctx.rng.generate_rnd(p._pIFMaxDam - p._pIFMinDam);
        crate::missiles::add_missile(ctx, p.position.tile, p.position.temp, p._pdir, MissileID::SpectralArrow, crate::missiles::TARGET_MONSTERS, pnum as i32, midam, 0, None);
    }
    let mind = p._pIMinDam;
    let maxd = p._pIMaxDam;
    let mut dam = ctx.rng.generate_rnd(maxd - mind + 1) + mind;
    dam += dam * p._pIBonusDam / 100;
    dam += p._pIBonusDamMod;
    let mut dam2 = dam << 6;
    dam += p._pDamageMod;
    if (p._pClass == HeroClass::Warrior || p._pClass == HeroClass::Barbarian) && ctx.rng.generate_rnd(100) < p._pLevel as i32 {
        dam *= 2;
    }
    let mut phanditype = ItemType::None;
    let (lt, rt) = (p.InvBody[INVLOC_HAND_LEFT as usize]._itype, p.InvBody[INVLOC_HAND_RIGHT as usize]._itype);
    if lt == ItemType::Sword || rt == ItemType::Sword {
        phanditype = ItemType::Sword;
    }
    if lt == ItemType::Mace || rt == ItemType::Mace {
        phanditype = ItemType::Mace;
    }
    match mon::monster_data(ctx, m).monsterClass {
        MonsterClass::Undead => {
            if phanditype == ItemType::Sword {
                dam -= dam / 2;
            } else if phanditype == ItemType::Mace {
                dam += dam / 2;
            }
        }
        MonsterClass::Animal => {
            if phanditype == ItemType::Mace {
                dam -= dam / 2;
            } else if phanditype == ItemType::Sword {
                dam += dam / 2;
            }
        }
        MonsterClass::Demon => {
            if p._pIFlags.has_any_of(E::TripleDemonDamage) {
                dam *= 3;
            }
        }
    }
    if p.pDamAcFlags.has_any_of(H::Devastation) && ctx.rng.generate_rnd(100) < 5 {
        dam *= 3;
    }
    if p.pDamAcFlags.has_any_of(H::Doppelganger)
        && mon::monster_type_id(ctx, m) != MT_DIABLO
        && !ctx.monster.Monsters[m].is_unique()
        && ctx.rng.generate_rnd(100) < 10
    {
        mon::add_doppelganger(ctx, m);
    }
    dam <<= 6;
    if p.pDamAcFlags.has_any_of(H::Jesters) {
        let mut r = ctx.rng.generate_rnd(201);
        if r >= 100 {
            r = 100 + (r - 100) * 5;
        }
        dam = dam * r / 100;
    }
    if adjacent_damage {
        dam >>= 2;
    }
    if is_my(ctx, pnum) {
        if p.pDamAcFlags.has_any_of(H::Peril) {
            dam2 += p._pIGetHit << 6;
            if dam2 >= 0 {
                apply_plr_damage(ctx, DamageType::Physical, pnum, 0, 1, dam2, DeathReason::MonsterOrTrap);
            }
            dam *= 2;
        }
        mon::apply_monster_damage(ctx, DamageType::Physical, m, dam);
    }
    let mut skdam;
    let flags = ctx.players.Players[pnum]._pIFlags;
    if flags.has_any_of(E::RandomStealLife) {
        skdam = ctx.rng.generate_rnd(dam / 8);
        let pl = &mut ctx.players.Players[pnum];
        pl._pHitPoints = (pl._pHitPoints + skdam).min(pl._pMaxHP);
        pl._pHPBase = (pl._pHPBase + skdam).min(pl._pMaxHPBase);
        redraw_component(ctx, PanelDrawComponent::Health);
    }
    if flags.has_any_of(E::StealMana3 | E::StealMana5) && flags.has_none_of(E::NoMana) {
        skdam = 0;
        if flags.has_any_of(E::StealMana3) {
            skdam = 3 * dam / 100;
        }
        if flags.has_any_of(E::StealMana5) {
            skdam = 5 * dam / 100;
        }
        let pl = &mut ctx.players.Players[pnum];
        pl._pMana = (pl._pMana + skdam).min(pl._pMaxMana);
        pl._pManaBase = (pl._pManaBase + skdam).min(pl._pMaxManaBase);
        redraw_component(ctx, PanelDrawComponent::Mana);
    }
    if flags.has_any_of(E::StealLife3 | E::StealLife5) {
        skdam = 0;
        if flags.has_any_of(E::StealLife3) {
            skdam = 3 * dam / 100;
        }
        if flags.has_any_of(E::StealLife5) {
            skdam = 5 * dam / 100;
        }
        let pl = &mut ctx.players.Players[pnum];
        pl._pHitPoints = (pl._pHitPoints + skdam).min(pl._pMaxHP);
        pl._pHPBase = (pl._pHPBase + skdam).min(pl._pMaxHPBase);
        redraw_component(ctx, PanelDrawComponent::Health);
    }
    if (ctx.monster.Monsters[m].hitPoints >> 6) <= 0 {
        mon::m_start_kill(ctx, m, pnum);
    } else {
        if ctx.monster.Monsters[m].mode != MonsterMode::Petrified && flags.has_any_of(E::Knockback) {
            mon::m_get_knockback(ctx, m);
        }
        mon::m_start_hit(ctx, m, pnum, dam);
    }
    true
}

/// Original: `PlrHitPlr` (player.cpp).
// @port player.cpp|devilution::PlrHitPlr(Player &attacker, Player &target) sha=c1da8bb56840
fn plr_hit_plr(ctx: &mut Ctx, attacker: usize, target: usize) -> bool {
    {
        let t = &ctx.players.Players[target];
        if t._pInvincible || t._pSpellFlags.has_any_of(SpellFlag::Etherealize) {
            return false;
        }
    }
    let hit = ctx.rng.generate_rnd(100);
    let a = ctx.players.Players[attacker].clone();
    let t = ctx.players.Players[target].clone();
    let hper = (a.get_melee_to_hit() - t.get_armor()).clamp(5, 95);
    let mut blk = 100;
    if (t._pmode == PM_STAND || t._pmode == PM_ATTACK) && t._pBlockFlag {
        blk = ctx.rng.generate_rnd(100);
    }
    let blkper = (t.get_block_chance(true) - a._pLevel as i32 * 2).clamp(0, 100);
    if hit >= hper {
        return false;
    }
    if blk < blkper {
        let dir = crate::engine::get_direction(t.position.tile, a.position.tile);
        start_plr_block(ctx, target, dir);
        return true;
    }
    let mind = a._pIMinDam;
    let maxd = a._pIMaxDam;
    let mut dam = ctx.rng.generate_rnd(maxd - mind + 1) + mind;
    dam += dam * a._pIBonusDam / 100;
    dam += a._pIBonusDamMod + a._pDamageMod;
    if (a._pClass == HeroClass::Warrior || a._pClass == HeroClass::Barbarian) && ctx.rng.generate_rnd(100) < a._pLevel as i32 {
        dam *= 2;
    }
    let skdam = dam << 6;
    if a._pIFlags.has_any_of(ItemSpecialEffect::RandomStealLife) {
        let tac = ctx.rng.generate_rnd(skdam / 8);
        let pl = &mut ctx.players.Players[attacker];
        pl._pHitPoints = (pl._pHitPoints + tac).min(pl._pMaxHP);
        pl._pHPBase = (pl._pHPBase + tac).min(pl._pMaxHPBase);
        redraw_component(ctx, PanelDrawComponent::Health);
    }
    if is_my(ctx, attacker) {
        crate::msg::net_send_cmd_damage(ctx, true, target as u8, skdam as u32, DamageType::Physical);
    }
    start_plr_hit(ctx, target, skdam, false);
    true
}

/// Original: `PlrHitObj` (player.cpp).
// @port player.cpp|devilution::PlrHitObj(const Player &player, Object &targetObject) sha=63018f0e1f17
fn plr_hit_obj(ctx: &mut Ctx, pnum: usize, oi: usize) -> bool {
    if ctx.objects.Objects[oi].is_breakable() {
        crate::objects::break_object(ctx, pnum, oi);
        return true;
    }
    false
}

/// Whether the weapon set allows cleave (the long condition in `DoAttack`).
fn has_cleave(p: &Player) -> bool {
    let l = &p.InvBody[INVLOC_HAND_LEFT as usize];
    let r = &p.InvBody[INVLOC_HAND_RIGHT as usize];
    (p._pClass == HeroClass::Monk && (l._itype == ItemType::Staff || r._itype == ItemType::Staff))
        || (p._pClass == HeroClass::Bard && l._itype == ItemType::Sword && r._itype == ItemType::Sword)
        || (p._pClass == HeroClass::Barbarian
            && (l._itype == ItemType::Axe
                || r._itype == ItemType::Axe
                || (((l._itype == ItemType::Mace && l._iLoc == ILOC_TWOHAND)
                    || (r._itype == ItemType::Mace && r._iLoc == ILOC_TWOHAND)
                    || (l._itype == ItemType::Sword && l._iLoc == ILOC_TWOHAND)
                    || (r._itype == ItemType::Sword && r._iLoc == ILOC_TWOHAND))
                    && !(l._itype == ItemType::Shield || r._itype == ItemType::Shield))))
}

/// Original: `DoAttack` (player.cpp).
// @port player.cpp|devilution::DoAttack(Player &player) sha=ba6e7f40e178
fn do_attack(ctx: &mut Ctx, pnum: usize) -> bool {
    let (cf, af, tile, pdir) = {
        let p = &ctx.players.Players[pnum];
        (p.AnimInfo.currentFrame, p._pAFNum, p.position.tile, p._pdir)
    };
    if cf == af - 2 {
        crate::effects::play_sfx_loc(ctx, crate::effects_data::PS_SWING, tile, true);
    }
    let mut didhit = false;
    if cf == af - 1 {
        let mut position = tile + pdir;
        let mut monster = crate::monster::find_monster_at_position(ctx, position, false);
        if let Some(m) = monster {
            if crate::monster::can_talk_to_monst(ctx, m) {
                ctx.players.Players[pnum].position.temp.x = 0;
                return false;
            }
        }
        let flags = ctx.players.Players[pnum]._pIFlags;
        use ItemSpecialEffect as E;
        if !ctx.init.gb_is_hellfire || !flags.has_all_of(E::FireDamage | E::LightningDamage) {
            if flags.has_any_of(E::FireDamage) {
                crate::missiles::add_missile(ctx, position, Point::new(1, 0), Direction::South, MissileID::WeaponExplosion, crate::missiles::TARGET_MONSTERS, pnum as i32, 0, 0, None);
            }
            if flags.has_any_of(E::LightningDamage) {
                crate::missiles::add_missile(ctx, position, Point::new(2, 0), Direction::South, MissileID::WeaponExplosion, crate::missiles::TARGET_MONSTERS, pnum as i32, 0, 0, None);
            }
        }
        if let Some(m) = monster {
            didhit = plr_hit_monst(ctx, pnum, m, false);
        } else if let Some(t) = player_at_position(ctx, position).filter(|_| !ctx.players.Players[pnum].friendlyMode) {
            didhit = plr_hit_plr(ctx, pnum, t);
        } else if let Some(o) = crate::objects::find_object_at_position(ctx, position, false) {
            didhit = plr_hit_obj(ctx, pnum, o);
        }
        if has_cleave(&ctx.players.Players[pnum]) {
            let pdir = ctx.players.Players[pnum]._pdir;
            for dir in [right(pdir), left(pdir)] {
                position = ctx.players.Players[pnum].position.tile + dir;
                monster = crate::monster::find_monster_at_position(ctx, position, false);
                if let Some(m) = monster {
                    if !crate::monster::can_talk_to_monst(ctx, m) && ctx.monster.Monsters[m].position.old == position && plr_hit_monst(ctx, pnum, m, true) {
                        didhit = true;
                    }
                }
            }
        }
        if didhit && damage_weapon(ctx, pnum, 30) {
            let d = ctx.players.Players[pnum]._pdir;
            start_stand(ctx, pnum, d);
            clear_state_variables(ctx, pnum);
            return true;
        }
    }
    if ctx.players.Players[pnum].AnimInfo.is_last_frame() {
        let d = ctx.players.Players[pnum]._pdir;
        start_stand(ctx, pnum, d);
        clear_state_variables(ctx, pnum);
        return true;
    }
    false
}

/// Original: `DoRangeAttack` (player.cpp).
// @port player.cpp|devilution::DoRangeAttack(Player &player) sha=1cad594ee73b
fn do_range_attack(ctx: &mut Ctx, pnum: usize) -> bool {
    use ItemSpecialEffect as E;
    let mut arrows = 0;
    {
        let p = &ctx.players.Players[pnum];
        if p.AnimInfo.currentFrame == p._pAFNum - 1 {
            arrows = 1;
        }
        if p._pIFlags.has_any_of(E::MultipleArrows) && p.AnimInfo.currentFrame == p._pAFNum + 1 {
            arrows = 2;
        }
    }
    for arrow in 0..arrows {
        let p = ctx.players.Players[pnum].clone();
        let mut xoff = 0;
        let mut yoff = 0;
        if arrows != 1 {
            let angle = if arrow == 0 { -1 } else { 1 };
            let x = p.position.temp.x - p.position.tile.x;
            if x != 0 {
                yoff = if x < 0 { angle } else { -angle };
            }
            let y = p.position.temp.y - p.position.tile.y;
            if y != 0 {
                xoff = if y < 0 { -angle } else { angle };
            }
        }
        let mut dmg = 4;
        let mut mistype = MissileID::Arrow;
        if p._pIFlags.has_any_of(E::FireArrows) {
            mistype = MissileID::FireArrow;
        }
        if p._pIFlags.has_any_of(E::LightningArrows) {
            mistype = MissileID::LightningArrow;
        }
        if p._pIFlags.has_all_of(E::FireArrows | E::LightningArrows) {
            dmg = p._pIFMinDam + ctx.rng.generate_rnd(p._pIFMaxDam - p._pIFMinDam);
            mistype = MissileID::SpectralArrow;
        }
        crate::missiles::add_missile(
            ctx,
            p.position.tile,
            p.position.temp + Displacement::new(xoff, yoff),
            p._pdir,
            mistype,
            crate::missiles::TARGET_MONSTERS,
            pnum as i32,
            dmg,
            0,
            None,
        );
        if arrow == 0 && mistype != MissileID::SpectralArrow {
            let sfx = if arrows != 1 { crate::effects_data::IS_STING1 } else { crate::effects_data::PS_BFIRE };
            crate::effects::play_sfx_loc(ctx, sfx, p.position.tile, true);
        }
        if damage_weapon(ctx, pnum, 40) {
            let d = ctx.players.Players[pnum]._pdir;
            start_stand(ctx, pnum, d);
            clear_state_variables(ctx, pnum);
            return true;
        }
    }
    if ctx.players.Players[pnum].AnimInfo.is_last_frame() {
        let d = ctx.players.Players[pnum]._pdir;
        start_stand(ctx, pnum, d);
        clear_state_variables(ctx, pnum);
        return true;
    }
    false
}

/// Original: `DamageParryItem` (player.cpp).
// @port player.cpp|devilution::DamageParryItem(Player &player) sha=6b9ccb238a33
fn damage_parry_item(ctx: &mut Ctx, pnum: usize) {
    if !is_my(ctx, pnum) {
        return;
    }
    let lt = ctx.players.Players[pnum].InvBody[INVLOC_HAND_LEFT as usize]._itype;
    if lt == ItemType::Shield || lt == ItemType::Staff {
        let it = &mut ctx.players.Players[pnum].InvBody[INVLOC_HAND_LEFT as usize];
        if it._iDurability == DUR_INDESTRUCTIBLE {
            return;
        }
        it._iDurability -= 1;
        if it._iDurability == 0 {
            crate::inv::remove_equipment(ctx, pnum, INVLOC_HAND_LEFT, true);
            crate::items::calc_plr_inv(ctx, pnum, true);
        }
    }
    if ctx.players.Players[pnum].InvBody[INVLOC_HAND_RIGHT as usize]._itype == ItemType::Shield {
        let it = &mut ctx.players.Players[pnum].InvBody[INVLOC_HAND_RIGHT as usize];
        if it._iDurability != DUR_INDESTRUCTIBLE {
            it._iDurability -= 1;
            if it._iDurability == 0 {
                crate::inv::remove_equipment(ctx, pnum, INVLOC_HAND_RIGHT, true);
                crate::items::calc_plr_inv(ctx, pnum, true);
            }
        }
    }
}

/// Original: `DoBlock` (player.cpp).
// @port player.cpp|devilution::DoBlock(Player &player) sha=a259401793ab
fn do_block(ctx: &mut Ctx, pnum: usize) -> bool {
    if ctx.players.Players[pnum].AnimInfo.is_last_frame() {
        let d = ctx.players.Players[pnum]._pdir;
        start_stand(ctx, pnum, d);
        clear_state_variables(ctx, pnum);
        if ctx.rng.flip_coin(10) {
            damage_parry_item(ctx, pnum);
        }
        return true;
    }
    false
}

/// Original: `DamageArmor` (player.cpp).
// @port player.cpp|devilution::DamageArmor(Player &player) sha=c1d837c75baa
fn damage_armor(ctx: &mut Ctx, pnum: usize) {
    if !is_my(ctx, pnum) {
        return;
    }
    let (chest_empty, head_empty) = {
        let p = &ctx.players.Players[pnum];
        (p.InvBody[INVLOC_CHEST as usize].is_empty(), p.InvBody[INVLOC_HEAD as usize].is_empty())
    };
    if chest_empty && head_empty {
        return;
    }
    let mut target_head = ctx.rng.flip_coin(3);
    if !chest_empty && head_empty {
        target_head = false;
    }
    if chest_empty && !head_empty {
        target_head = true;
    }
    let loc = if target_head { INVLOC_HEAD } else { INVLOC_CHEST };
    let pi = &mut ctx.players.Players[pnum].InvBody[loc as usize];
    if pi._iDurability == DUR_INDESTRUCTIBLE {
        return;
    }
    pi._iDurability -= 1;
    if pi._iDurability != 0 {
        return;
    }
    crate::inv::remove_equipment(ctx, pnum, loc, true);
    crate::items::calc_plr_inv(ctx, pnum, true);
}

/// Original: `DoSpell` (player.cpp).
// @port player.cpp|devilution::DoSpell(Player &player) sha=fe72c5995a3d
fn do_spell(ctx: &mut Ctx, pnum: usize) -> bool {
    let p = &ctx.players.Players[pnum];
    if p.AnimInfo.currentFrame == p._pSFNum {
        let (sp, lvl, ty) = (p.executedSpell.spellId, p.executedSpell.spellLevel, p.executedSpell.spellType);
        let (t, tmp) = (p.position.tile, p.position.temp);
        crate::spells::cast_spell(ctx, pnum as i32, sp, t.x, t.y, tmp.x, tmp.y, lvl);
        if matches!(ty, SpellType::Scroll | SpellType::Charges) {
            ensure_valid_readied_spell(ctx, pnum);
        }
    }
    if ctx.players.Players[pnum].AnimInfo.is_last_frame() {
        let d = ctx.players.Players[pnum]._pdir;
        start_stand(ctx, pnum, d);
        clear_state_variables(ctx, pnum);
        return true;
    }
    false
}

/// Original: `DoGotHit` (player.cpp).
// @port player.cpp|devilution::DoGotHit(Player &player) sha=d36c484d03a3
fn do_got_hit(ctx: &mut Ctx, pnum: usize) -> bool {
    if ctx.players.Players[pnum].AnimInfo.is_last_frame() {
        let d = ctx.players.Players[pnum]._pdir;
        start_stand(ctx, pnum, d);
        clear_state_variables(ctx, pnum);
        if !ctx.rng.flip_coin(4) {
            damage_armor(ctx, pnum);
        }
        return true;
    }
    false
}

/// Original: `DoDeath` (player.cpp).
// @port player.cpp|devilution::DoDeath(Player &player) sha=f9306262951c
fn do_death(ctx: &mut Ctx, pnum: usize) -> bool {
    let p = &mut ctx.players.Players[pnum];
    if p.AnimInfo.is_last_frame() {
        if p.AnimInfo.tickCounterOfCurrentFrame == 0 {
            p.AnimInfo.ticksPerFrame = 100;
            let t = p.position.tile;
            ctx.gendung.dFlags[t.x as usize][t.y as usize] |= DungeonFlag::DeadPlayer;
        } else if ctx.players.MyPlayer == Some(pnum) && p.AnimInfo.tickCounterOfCurrentFrame == 30 {
            ctx.players.MyPlayerIsDead = true;
            if !ctx.init.gb_is_multiplayer {
                crate::gamemenu::gamemenu_on(ctx);
            }
        }
    }
    false
}

/// Original: `IsPlayerAdjacentToObject` (player.cpp).
// @port player.cpp|devilution::IsPlayerAdjacentToObject(Player &player, Object &object) sha=5664f41827e5
fn is_player_adjacent_to_object(ctx: &Ctx, pnum: usize, oi: usize) -> bool {
    let pt = ctx.players.Players[pnum].position.tile;
    let op = ctx.objects.Objects[oi].position;
    let x = (pt.x - op.x).abs();
    let mut y = (pt.y - op.y).abs();
    if y > 1 && op.y >= 1 && crate::objects::find_object_at_position(ctx, op + Direction::NorthEast, true) == Some(oi) {
        y = (pt.y - op.y + 1).abs();
    }
    x <= 1 && y <= 1
}

/// Original: `TryDisarm` (player.cpp).
// @port player.cpp|devilution::TryDisarm(const Player &player, Object &object) sha=9c96082597a5
fn try_disarm(ctx: &mut Ctx, pnum: usize, oi: usize) {
    if is_my(ctx, pnum) {
        crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_HAND);
    }
    if !ctx.objects.Objects[oi]._oTrapFlag {
        return;
    }
    let trapdisper = 2 * ctx.players.Players[pnum]._pDexterity - 5 * ctx.gendung.currlevel as i32;
    if ctx.rng.generate_rnd(100) > trapdisper {
        return;
    }
    for j in 0..ctx.objects.ActiveObjectCount as usize {
        let ti = ctx.objects.ActiveObjects[j] as usize;
        let trap = &ctx.objects.Objects[ti];
        if trap.is_trap() && crate::objects::find_object_at_position(ctx, Point::new(trap._oVar1, trap._oVar2), true) == Some(oi) {
            ctx.objects.Objects[ti]._oVar4 = 1;
            ctx.objects.Objects[oi]._oTrapFlag = false;
        }
    }
    if ctx.objects.Objects[oi].is_trapped_chest() {
        ctx.objects.Objects[oi]._oTrapFlag = false;
    }
}

/// Original: `CheckNewPath` (player.cpp).
// @port player.cpp|devilution::CheckNewPath(Player &player, bool pmWillBeCalled) sha=2419d591e696
fn check_new_path(ctx: &mut Ctx, pnum: usize, pm_will_be_called: bool) {
    use crate::engine::get_direction;
    let target_id = ctx.players.Players[pnum].destParam1;
    let dest_action = ctx.players.Players[pnum].destAction;
    let mut monster: usize = 0;
    let mut target: usize = 0;
    let mut object: usize = 0;
    let mut item: usize = 0;
    match dest_action {
        ACTION_ATTACKMON | ACTION_RATTACKMON | ACTION_SPELLMON => {
            monster = target_id as usize;
            if (ctx.monster.Monsters[monster].hitPoints >> 6) <= 0 {
                player_stop(ctx, pnum);
                return;
            }
            if dest_action == ACTION_ATTACKMON {
                let f = ctx.monster.Monsters[monster].position.future;
                make_plr_path(ctx, pnum, f, false);
            }
        }
        ACTION_ATTACKPLR | ACTION_RATTACKPLR | ACTION_SPELLPLR => {
            target = target_id as usize;
            if (ctx.players.Players[target]._pHitPoints >> 6) <= 0 {
                player_stop(ctx, pnum);
                return;
            }
            if dest_action == ACTION_ATTACKPLR {
                let f = ctx.players.Players[target].position.future;
                make_plr_path(ctx, pnum, f, false);
            }
        }
        ACTION_OPERATE | ACTION_DISARM | ACTION_OPERATETK => object = target_id as usize,
        ACTION_PICKUPITEM | ACTION_PICKUPAITEM => item = target_id as usize,
        _ => {}
    }
    let mfut = |ctx: &Ctx| ctx.monster.Monsters[monster].position.future;
    let tfut = |ctx: &Ctx| ctx.players.Players[target].position.future;
    let talk_msg_ok = |ctx: &Ctx| {
        let t = ctx.monster.Monsters[monster].talkMsg;
        t != TEXT_NONE && t != TEXT_VILE14
    };
    if ctx.players.Players[pnum].walkpath[0] as i32 != WALK_NONE {
        if ctx.players.Players[pnum]._pmode == PM_STAND {
            if is_my(ctx, pnum) {
                let da = ctx.players.Players[pnum].destAction;
                if da == ACTION_ATTACKMON || da == ACTION_ATTACKPLR {
                    let pf = ctx.players.Players[pnum].position.future;
                    let (x, y, d) = if da == ACTION_ATTACKMON {
                        let mf = mfut(ctx);
                        ((pf.x - mf.x).abs(), (pf.y - mf.y).abs(), get_direction(pf, mf))
                    } else {
                        let tf = tfut(ctx);
                        ((pf.x - tf.x).abs(), (pf.y - tf.y).abs(), get_direction(pf, tf))
                    };
                    if x < 2 && y < 2 {
                        clr_plr_path(ctx, pnum);
                        if da == ACTION_ATTACKMON && talk_msg_ok(ctx) {
                            crate::monster::talkto_monster(ctx, pnum, monster);
                        } else {
                            start_attack(ctx, pnum, d, pm_will_be_called);
                        }
                        ctx.players.Players[pnum].destAction = ACTION_NONE;
                    }
                }
            }
            let dir = match ctx.players.Players[pnum].walkpath[0] as i32 {
                WALK_N => Some(Direction::North),
                WALK_NE => Some(Direction::NorthEast),
                WALK_E => Some(Direction::East),
                WALK_SE => Some(Direction::SouthEast),
                WALK_S => Some(Direction::South),
                WALK_SW => Some(Direction::SouthWest),
                WALK_W => Some(Direction::West),
                WALK_NW => Some(Direction::NorthWest),
                _ => None,
            };
            if let Some(d) = dir {
                start_walk(ctx, pnum, d, pm_will_be_called);
            }
            let p = &mut ctx.players.Players[pnum];
            for j in 1..MaxPathLength {
                p.walkpath[j - 1] = p.walkpath[j];
            }
            p.walkpath[MaxPathLength - 1] = WALK_NONE as i8;
            if p._pmode == PM_STAND {
                let d = p._pdir;
                start_stand(ctx, pnum, d);
                ctx.players.Players[pnum].destAction = ACTION_NONE;
            }
        }
        return;
    }
    if ctx.players.Players[pnum].destAction == ACTION_NONE {
        return;
    }
    let p = ctx.players.Players[pnum].clone();
    let dp = Point::new(p.destParam1, p.destParam2);
    if p._pmode == PM_STAND {
        match p.destAction {
            ACTION_ATTACK => {
                let d = get_direction(p.position.tile, dp);
                start_attack(ctx, pnum, d, pm_will_be_called);
            }
            ACTION_ATTACKMON => {
                let mf = mfut(ctx);
                let x = (p.position.tile.x - mf.x).abs();
                let y = (p.position.tile.y - mf.y).abs();
                if x <= 1 && y <= 1 {
                    let d = get_direction(p.position.future, mf);
                    if talk_msg_ok(ctx) {
                        crate::monster::talkto_monster(ctx, pnum, monster);
                    } else {
                        start_attack(ctx, pnum, d, pm_will_be_called);
                    }
                }
            }
            ACTION_ATTACKPLR => {
                let tf = tfut(ctx);
                let x = (p.position.tile.x - tf.x).abs();
                let y = (p.position.tile.y - tf.y).abs();
                if x <= 1 && y <= 1 {
                    let d = get_direction(p.position.future, tf);
                    start_attack(ctx, pnum, d, pm_will_be_called);
                }
            }
            ACTION_RATTACK => {
                let d = get_direction(p.position.tile, dp);
                start_range_attack(ctx, pnum, d, p.destParam1, p.destParam2, pm_will_be_called);
            }
            ACTION_RATTACKMON => {
                let mf = mfut(ctx);
                let d = get_direction(p.position.future, mf);
                if talk_msg_ok(ctx) {
                    crate::monster::talkto_monster(ctx, pnum, monster);
                } else {
                    start_range_attack(ctx, pnum, d, mf.x, mf.y, pm_will_be_called);
                }
            }
            ACTION_RATTACKPLR => {
                let tf = tfut(ctx);
                let d = get_direction(p.position.future, tf);
                start_range_attack(ctx, pnum, d, tf.x, tf.y, pm_will_be_called);
            }
            ACTION_SPELL => {
                let d = get_direction(p.position.tile, dp);
                start_spell(ctx, pnum, d, p.destParam1, p.destParam2);
                ctx.players.Players[pnum].executedSpell.spellLevel = p.destParam3;
            }
            ACTION_SPELLWALL => {
                let d = Direction::from_u8(p.destParam3 as u8);
                start_spell(ctx, pnum, d, p.destParam1, p.destParam2);
                ctx.players.Players[pnum].tempDirection = d;
                ctx.players.Players[pnum].executedSpell.spellLevel = p.destParam4;
            }
            ACTION_SPELLMON => {
                let mf = mfut(ctx);
                let d = get_direction(p.position.tile, mf);
                start_spell(ctx, pnum, d, mf.x, mf.y);
                ctx.players.Players[pnum].executedSpell.spellLevel = p.destParam2;
            }
            ACTION_SPELLPLR => {
                let tf = tfut(ctx);
                let d = get_direction(p.position.tile, tf);
                start_spell(ctx, pnum, d, tf.x, tf.y);
                ctx.players.Players[pnum].executedSpell.spellLevel = p.destParam2;
            }
            ACTION_OPERATE => {
                if is_player_adjacent_to_object(ctx, pnum, object) {
                    if ctx.objects.Objects[object]._oBreak == 1 {
                        let d = get_direction(p.position.tile, ctx.objects.Objects[object].position);
                        start_attack(ctx, pnum, d, pm_will_be_called);
                    } else {
                        crate::objects::operate_object(ctx, pnum, object);
                    }
                }
            }
            ACTION_DISARM => {
                if is_player_adjacent_to_object(ctx, pnum, object) {
                    if ctx.objects.Objects[object]._oBreak == 1 {
                        let d = get_direction(p.position.tile, ctx.objects.Objects[object].position);
                        start_attack(ctx, pnum, d, pm_will_be_called);
                    } else {
                        try_disarm(ctx, pnum, object);
                        crate::objects::operate_object(ctx, pnum, object);
                    }
                }
            }
            ACTION_OPERATETK => {
                if ctx.objects.Objects[object]._oBreak != 1 {
                    crate::objects::operate_object(ctx, pnum, object);
                }
            }
            ACTION_PICKUPITEM => {
                if is_my(ctx, pnum) {
                    let ip = ctx.items.Items[item].position;
                    let x = (p.position.tile.x - ip.x).abs();
                    let y = (p.position.tile.y - ip.y).abs();
                    if x <= 1 && y <= 1 && ctx.cursor.pcurs == crate::cursor::CURSOR_HAND && !ctx.items.Items[item]._iRequest {
                        crate::msg::net_send_cmd_g_item(ctx, true, CMD_REQUESTGITEM, pnum as u8, target_id as u8);
                        ctx.items.Items[item]._iRequest = true;
                    }
                }
            }
            ACTION_PICKUPAITEM => {
                if is_my(ctx, pnum) {
                    let ip = ctx.items.Items[item].position;
                    let x = (p.position.tile.x - ip.x).abs();
                    let y = (p.position.tile.y - ip.y).abs();
                    if x <= 1 && y <= 1 && ctx.cursor.pcurs == crate::cursor::CURSOR_HAND {
                        crate::msg::net_send_cmd_g_item(ctx, true, CMD_REQUESTAGITEM, pnum as u8, target_id as u8);
                    }
                }
            }
            ACTION_TALK => {
                if is_my(ctx, pnum) {
                    ctx.help.HelpFlag = false;
                    crate::towners::talk_to_towner(ctx, pnum, p.destParam1);
                }
            }
            _ => {}
        }
        let d = ctx.players.Players[pnum]._pdir;
        fix_player_location(ctx, pnum, d);
        ctx.players.Players[pnum].destAction = ACTION_NONE;
        return;
    }
    let cf = ctx.players.Players[pnum].AnimInfo.currentFrame;
    let da = ctx.players.Players[pnum].destAction;
    if p._pmode == PM_ATTACK && cf >= p._pAFNum {
        if da == ACTION_ATTACK {
            let d = get_direction(p.position.future, dp);
            start_attack(ctx, pnum, d, pm_will_be_called);
            ctx.players.Players[pnum].destAction = ACTION_NONE;
        } else if da == ACTION_ATTACKMON {
            let mf = mfut(ctx);
            let x = (p.position.tile.x - mf.x).abs();
            let y = (p.position.tile.y - mf.y).abs();
            if x <= 1 && y <= 1 {
                let d = get_direction(p.position.future, mf);
                start_attack(ctx, pnum, d, pm_will_be_called);
            }
            ctx.players.Players[pnum].destAction = ACTION_NONE;
        } else if da == ACTION_ATTACKPLR {
            let tf = tfut(ctx);
            let x = (p.position.tile.x - tf.x).abs();
            let y = (p.position.tile.y - tf.y).abs();
            if x <= 1 && y <= 1 {
                let d = get_direction(p.position.future, tf);
                start_attack(ctx, pnum, d, pm_will_be_called);
            }
            ctx.players.Players[pnum].destAction = ACTION_NONE;
        } else if da == ACTION_OPERATE && is_player_adjacent_to_object(ctx, pnum, object) && ctx.objects.Objects[object]._oBreak == 1 {
            let d = get_direction(p.position.tile, ctx.objects.Objects[object].position);
            start_attack(ctx, pnum, d, pm_will_be_called);
        }
    }
    let p = ctx.players.Players[pnum].clone();
    let da = p.destAction;
    if p._pmode == PM_RATTACK && p.AnimInfo.currentFrame >= p._pAFNum {
        if da == ACTION_RATTACK {
            let d = get_direction(p.position.tile, dp);
            start_range_attack(ctx, pnum, d, p.destParam1, p.destParam2, pm_will_be_called);
            ctx.players.Players[pnum].destAction = ACTION_NONE;
        } else if da == ACTION_RATTACKMON {
            let mf = mfut(ctx);
            let d = get_direction(p.position.tile, mf);
            start_range_attack(ctx, pnum, d, mf.x, mf.y, pm_will_be_called);
            ctx.players.Players[pnum].destAction = ACTION_NONE;
        } else if da == ACTION_RATTACKPLR {
            let tf = tfut(ctx);
            let d = get_direction(p.position.tile, tf);
            start_range_attack(ctx, pnum, d, tf.x, tf.y, pm_will_be_called);
            ctx.players.Players[pnum].destAction = ACTION_NONE;
        }
    }
    let p = ctx.players.Players[pnum].clone();
    let da = p.destAction;
    if p._pmode == PM_SPELL && p.AnimInfo.currentFrame >= p._pSFNum {
        if da == ACTION_SPELL {
            let d = get_direction(p.position.tile, dp);
            start_spell(ctx, pnum, d, p.destParam1, p.destParam2);
            ctx.players.Players[pnum].destAction = ACTION_NONE;
        } else if da == ACTION_SPELLMON {
            let mf = mfut(ctx);
            let d = get_direction(p.position.tile, mf);
            start_spell(ctx, pnum, d, mf.x, mf.y);
            ctx.players.Players[pnum].destAction = ACTION_NONE;
        } else if da == ACTION_SPELLPLR {
            let tf = tfut(ctx);
            let d = get_direction(p.position.tile, tf);
            start_spell(ctx, pnum, d, tf.x, tf.y);
            ctx.players.Players[pnum].destAction = ACTION_NONE;
        }
    }
}

/// Original: `PlrDeathModeOK` (player.cpp).
// @port player.cpp|devilution::PlrDeathModeOK(Player &player) sha=2af80fe1a2a6
fn plr_death_mode_ok(ctx: &Ctx, pnum: usize) -> bool {
    if !is_my(ctx, pnum) {
        return true;
    }
    matches!(ctx.players.Players[pnum]._pmode, PM_DEATH | PM_QUIT | PM_NEWLVL)
}

/// Original: `ValidatePlayer` (player.cpp).
// @port player.cpp|devilution::ValidatePlayer() sha=95169e902ff7
fn validate_player(ctx: &mut Ctx) {
    let me = ctx.players.my_player_id();
    let hf = ctx.init.gb_is_hellfire;
    let mut redraw = false;
    {
        let p = &mut ctx.players.Players[me];
        if p._pLevel as i32 > MaxCharacterLevel {
            p._pLevel = MaxCharacterLevel as i8;
        }
        if p._pExperience > p._pNextExper {
            p._pExperience = p._pNextExper;
            redraw = true;
        }
    }
    if redraw && ctx.options.gameplay.experience_bar.get() {
        redraw_everything(ctx);
    }
    let p = &mut ctx.players.Players[me];
    let mut gt = 0;
    for i in 0..p._pNumInv as usize {
        if p.InvList[i]._itype == ItemType::Gold {
            let mut max_gold = GOLD_MAX_LIMIT;
            if hf {
                max_gold *= 2;
            }
            if p.InvList[i]._ivalue > max_gold {
                p.InvList[i]._ivalue = max_gold;
            }
            gt += p.InvList[i]._ivalue;
        }
    }
    if gt != p._pGold {
        p._pGold = gt;
    }
    for attr in [CharacterAttribute::Strength, CharacterAttribute::Magic, CharacterAttribute::Dexterity, CharacterAttribute::Vitality] {
        let max = p.get_maximum_attribute_value(attr);
        let base = match attr {
            CharacterAttribute::Strength => &mut p._pBaseStr,
            CharacterAttribute::Magic => &mut p._pBaseMag,
            CharacterAttribute::Dexterity => &mut p._pBaseDex,
            CharacterAttribute::Vitality => &mut p._pBaseVit,
        };
        if *base > max {
            *base = max;
        }
    }
    let mut msk: u64 = 0;
    for b in SpellID::Firebolt as i32..crate::items::MAX_SPELLS {
        let sp = SpellID::from_raw(b as i8);
        if crate::spells::get_spell_book_level(ctx, sp) != -1 {
            msk |= crate::spells::get_spell_bitmask(sp);
            let p = &mut ctx.players.Players[me];
            if p._pSplLvl[b as usize] > MaxSpellLevel {
                p._pSplLvl[b as usize] = MaxSpellLevel;
            }
        }
    }
    ctx.players.Players[me]._pMemSpells &= msk;
}

/// Original: `CheckCheatStats` (player.cpp).
// @port player.cpp|devilution::CheckCheatStats(Player &player) sha=727df9bf0695
fn check_cheat_stats(p: &mut Player) {
    p._pStrength = p._pStrength.min(750);
    p._pDexterity = p._pDexterity.min(750);
    p._pMagic = p._pMagic.min(750);
    p._pVitality = p._pVitality.min(750);
    p._pHitPoints = p._pHitPoints.min(128000);
    p._pMana = p._pMana.min(128000);
}

/// Original: `GetPlayerSpriteClass` (player.cpp).
// @port player.cpp|devilution::GetPlayerSpriteClass(HeroClass cls) sha=ed310a2a3c24
fn get_player_sprite_class(ctx: &Ctx, cls: HeroClass) -> HeroClass {
    if cls == HeroClass::Bard && !ctx.diablo.gb_bard {
        return HeroClass::Rogue;
    }
    if cls == HeroClass::Barbarian && !ctx.diablo.gb_barbarian {
        return HeroClass::Warrior;
    }
    cls
}

/// Original: `GetPlayerWeaponGraphic` (player.cpp).
// @port player.cpp|devilution::GetPlayerWeaponGraphic(player_graphic graphic, PlayerWeaponGraphic weaponGraphic) sha=ec2af7de2d50
fn get_player_weapon_graphic(ctx: &Ctx, graphic: player_graphic, weapon_graphic: PlayerWeaponGraphic) -> PlayerWeaponGraphic {
    if ctx.gendung.leveltype == DungeonType::Town && matches!(graphic, player_graphic::Lightning | player_graphic::Fire | player_graphic::Magic) {
        match weapon_graphic {
            PlayerWeaponGraphic::Mace | PlayerWeaponGraphic::Sword => return PlayerWeaponGraphic::Unarmed,
            PlayerWeaponGraphic::SwordShield | PlayerWeaponGraphic::MaceShield => return PlayerWeaponGraphic::UnarmedShield,
            _ => {}
        }
    }
    weapon_graphic
}

/// Original: `GetPlayerSpriteWidth` (player.cpp).
// @port player.cpp|devilution::GetPlayerSpriteWidth(HeroClass cls, player_graphic graphic, PlayerWeaponGraphic weaponGraphic) sha=071ea3f02112
fn get_player_sprite_width(cls: HeroClass, graphic: player_graphic, weapon_graphic: PlayerWeaponGraphic) -> u16 {
    let s = &PlayersSpriteData[cls as usize];
    (match graphic {
        player_graphic::Stand => s.stand,
        player_graphic::Walk => s.walk,
        player_graphic::Attack => {
            if weapon_graphic == PlayerWeaponGraphic::Bow {
                s.bow
            } else {
                s.attack
            }
        }
        player_graphic::Hit => s.swHit,
        player_graphic::Block => s.block,
        player_graphic::Lightning => s.lightning,
        player_graphic::Fire => s.fire,
        player_graphic::Magic => s.magic,
        player_graphic::Death => s.death,
    }) as u16
}

fn weapon_graphic_of(gfxnum: u8) -> PlayerWeaponGraphic {
    PlayerWeaponGraphic::from_raw(gfxnum & 0xF)
}

/// Original: `Player::CalcScrolls` (player.cpp).
// @port player.cpp|devilution::Player::CalcScrolls() sha=2c535f071663
pub fn calc_scrolls(ctx: &mut Ctx, pnum: usize) {
    let p = &mut ctx.players.Players[pnum];
    p._pScrlSpells = 0;
    let n = p._pNumInv as usize;
    let mut bits = 0;
    for item in p.InvList[..n].iter().chain(p.SpdList.iter()) {
        if !item.is_empty() && item.is_scroll() && item._iStatFlag {
            bits |= crate::spells::get_spell_bitmask(item._iSpell);
        }
    }
    p._pScrlSpells = bits;
    ensure_valid_readied_spell(ctx, pnum);
}

/// Original: `Player::RemoveInvItem` (player.cpp). `calcScrolls` defaults to true.
// @port player.cpp|devilution::Player::RemoveInvItem(int iv, bool calcScrolls) sha=b8b6a8691435
pub fn remove_inv_item(ctx: &mut Ctx, pnum: usize, iv: i32, calc_scrolls_flag: bool) {
    if is_my(ctx, pnum) {
        let grid = ctx.players.Players[pnum].InvGrid;
        for (i, &item_index) in grid.iter().enumerate() {
            if (item_index as i32).abs() - 1 == iv {
                crate::msg::net_send_cmd_param1(ctx, false, CMD_DELINVITEMS, i as u16);
                break;
            }
        }
    }
    let p = &mut ctx.players.Players[pnum];
    for item_index in p.InvGrid.iter_mut() {
        if (*item_index as i32).abs() - 1 == iv {
            *item_index = 0;
        }
    }
    p.InvList[iv as usize].clear();
    p._pNumInv -= 1;
    let n = p._pNumInv;
    if n > 0 && n != iv {
        let moved = p.InvList[n as usize].pop();
        p.InvList[iv as usize] = moved;
        for item_index in p.InvGrid.iter_mut() {
            if *item_index as i32 == n + 1 {
                *item_index = (iv + 1) as i8;
            }
            if *item_index as i32 == -(n + 1) {
                *item_index = -(iv + 1) as i8;
            }
        }
    }
    if calc_scrolls_flag {
        calc_scrolls(ctx, pnum);
    }
}

/// Original: `Player::RemoveSpdBarItem` (player.cpp).
// @port player.cpp|devilution::Player::RemoveSpdBarItem(int iv) sha=b15643e015e1
pub fn remove_spd_bar_item(ctx: &mut Ctx, pnum: usize, iv: i32) {
    if is_my(ctx, pnum) {
        crate::msg::net_send_cmd_param1(ctx, false, CMD_DELBELTITEMS, iv as u16);
    }
    ctx.players.Players[pnum].SpdList[iv as usize].clear();
    calc_scrolls(ctx, pnum);
    redraw_everything(ctx);
}

/// Original: `Player::Say(HeroSpeech)` (player.cpp).
// @port player.cpp|devilution::Player::Say(HeroSpeech speechId) sha=9f3acc98cd6a
pub fn player_say(ctx: &mut Ctx, pnum: usize, speech_id: HeroSpeech) {
    let p = &ctx.players.Players[pnum];
    let sound_effect = herosounds[p._pClass as usize][speech_id as usize];
    if sound_effect == crate::effects_data::SFX_NONE {
        return;
    }
    let t = p.position.tile;
    crate::effects::play_sfx_loc(ctx, sound_effect, t, true);
}

/// Original: `Player::SaySpecific` (player.cpp).
// @port player.cpp|devilution::Player::SaySpecific(HeroSpeech speechId) sha=8d219b53e6ed
pub fn player_say_specific(ctx: &mut Ctx, pnum: usize, speech_id: HeroSpeech) {
    let p = &ctx.players.Players[pnum];
    let sound_effect = herosounds[p._pClass as usize][speech_id as usize];
    if sound_effect == crate::effects_data::SFX_NONE || crate::effects::effect_is_playing(ctx, sound_effect as i32) {
        return;
    }
    let t = p.position.tile;
    crate::effects::play_sfx_loc(ctx, sound_effect, t, false);
}

/// Original: `Player::Say(HeroSpeech, int delay)` (player.cpp).
// @port player.cpp|devilution::Player::Say(HeroSpeech speechId, int delay) sha=0dfb9e8b218f
pub fn player_say_delayed(ctx: &mut Ctx, pnum: usize, speech_id: HeroSpeech, delay: i32) {
    ctx.effects.sfxdelay = delay;
    ctx.effects.sfxdnum = herosounds[ctx.players.Players[pnum]._pClass as usize][speech_id as usize];
}

/// Original: `Player::Stop` (player.cpp).
// @port player.cpp|devilution::Player::Stop() sha=e68325883731
pub fn player_stop(ctx: &mut Ctx, pnum: usize) {
    clr_plr_path(ctx, pnum);
    ctx.players.Players[pnum].destAction = ACTION_NONE;
}

/// `StopHero` keymapper action: `MyPlayer->Stop()`.
pub fn stop_my_player(ctx: &mut Ctx) {
    if let Some(m) = ctx.players.MyPlayer {
        player_stop(ctx, m);
    }
}

/// Original: `Player::RestorePartialLife` (player.cpp).
// @port player.cpp|devilution::Player::RestorePartialLife() sha=09a4f871a740
pub fn restore_partial_life(ctx: &mut Ctx, pnum: usize) {
    let whole = ctx.players.Players[pnum]._pMaxHP >> 6;
    let mut l = ((whole / 8) + ctx.rng.generate_rnd(whole / 4)) << 6;
    let p = &mut ctx.players.Players[pnum];
    if matches!(p._pClass, HeroClass::Warrior | HeroClass::Barbarian) {
        l *= 2;
    }
    if matches!(p._pClass, HeroClass::Rogue | HeroClass::Monk | HeroClass::Bard) {
        l += l / 2;
    }
    p._pHitPoints = (p._pHitPoints + l).min(p._pMaxHP);
    p._pHPBase = (p._pHPBase + l).min(p._pMaxHPBase);
}

/// Original: `Player::RestorePartialMana` (player.cpp).
// @port player.cpp|devilution::Player::RestorePartialMana() sha=a9894634d9d3
pub fn restore_partial_mana(ctx: &mut Ctx, pnum: usize) {
    let whole = ctx.players.Players[pnum]._pMaxMana >> 6;
    let mut l = ((whole / 8) + ctx.rng.generate_rnd(whole / 4)) << 6;
    let p = &mut ctx.players.Players[pnum];
    if p._pClass == HeroClass::Sorcerer {
        l *= 2;
    }
    if matches!(p._pClass, HeroClass::Rogue | HeroClass::Monk | HeroClass::Bard) {
        l += l / 2;
    }
    if p._pIFlags.has_none_of(ItemSpecialEffect::NoMana) {
        p._pMana = (p._pMana + l).min(p._pMaxMana);
        p._pManaBase = (p._pManaBase + l).min(p._pMaxManaBase);
    }
}

/// Original: `Player::ReadySpellFromEquipment` (player.cpp).
// @port player.cpp|devilution::Player::ReadySpellFromEquipment(inv_body_loc bodyLocation, bool forceSpell) sha=4fa12554723e
pub fn ready_spell_from_equipment(ctx: &mut Ctx, pnum: usize, body_location: inv_body_loc, force_spell: bool) {
    let hellfire = ctx.init.gb_is_hellfire;
    let p = &mut ctx.players.Players[pnum];
    let item = &p.InvBody[body_location as usize];
    if item._itype == ItemType::Staff && crate::spells::is_valid_spell_hf(item._iSpell, hellfire) && item._iCharges > 0 && (force_spell || p._pRSpell == SpellID::Invalid || p._pRSplType == SpellType::Invalid) {
        p._pRSpell = item._iSpell;
        p._pRSplType = SpellType::Charges;
        redraw_everything(ctx);
    }
}

/// Original: `Player::getSpriteWidth` (player.cpp).
// @port player.cpp|devilution::Player::getSpriteWidth() sha=d0d62a292c71
pub fn get_sprite_width(ctx: &Ctx, pnum: usize) -> u16 {
    let p = &ctx.players.Players[pnum];
    if !ctx.diablo.headless_mode {
        return p.AnimInfo.sprites.as_ref().expect("sprites").get(0).width();
    }
    let graphic = p.get_graphic();
    let cls = get_player_sprite_class(ctx, p._pClass);
    let weapon_graphic = get_player_weapon_graphic(ctx, graphic, weapon_graphic_of(p._pgfxnum));
    get_player_sprite_width(cls, graphic, weapon_graphic)
}

/// Original: `Player::UpdatePreviewCelSprite` (player.cpp).
// @port player.cpp|devilution::Player::UpdatePreviewCelSprite(_cmd_id cmdId, Point point, uint16_t wParam1, uint16_t wParam2) sha=485ec52ba289
pub fn update_preview_cel_sprite(ctx: &mut Ctx, pnum: usize, cmd_id: _cmd_id, mut point: Point, w_param1: u16, w_param2: u16) {
    use crate::engine::get_direction;
    if !ctx.diablo.gb_run_game || ctx.diablo.pause_mode != 0 || !ctx.diablo.gb_process_players {
        return;
    }
    if ctx.players.Players[pnum]._pmode != PM_STAND {
        return;
    }
    let pf = ctx.players.Players[pnum].position.future;
    let ptile = ctx.players.Players[pnum].position.tile;
    let mut graphic: Option<player_graphic> = None;
    let mut dir = Direction::South;
    let mut minimal_walk_distance = -1;
    let spell = |v: u16| get_player_graphic_for_spell(SpellID::from_raw(v as i8));
    match cmd_id {
        CMD_RATTACKID => {
            dir = get_direction(pf, ctx.monster.Monsters[w_param1 as usize].position.future);
            graphic = Some(player_graphic::Attack);
        }
        CMD_SPELLID => {
            dir = get_direction(pf, ctx.monster.Monsters[w_param1 as usize].position.future);
            graphic = Some(spell(w_param2));
        }
        CMD_ATTACKID => {
            let mf = ctx.monster.Monsters[w_param1 as usize].position.future;
            point = mf;
            minimal_walk_distance = 2;
            if !crate::monster::can_talk_to_monst(ctx, w_param1 as usize) {
                dir = get_direction(pf, mf);
                graphic = Some(player_graphic::Attack);
            }
        }
        CMD_RATTACKPID => {
            dir = get_direction(pf, ctx.players.Players[w_param1 as usize].position.future);
            graphic = Some(player_graphic::Attack);
        }
        CMD_SPELLPID => {
            dir = get_direction(pf, ctx.players.Players[w_param1 as usize].position.future);
            graphic = Some(spell(w_param2));
        }
        CMD_ATTACKPID => {
            let tf = ctx.players.Players[w_param1 as usize].position.future;
            point = tf;
            minimal_walk_distance = 2;
            dir = get_direction(pf, tf);
            graphic = Some(player_graphic::Attack);
        }
        CMD_ATTACKXY => {
            dir = get_direction(ptile, point);
            graphic = Some(player_graphic::Attack);
            minimal_walk_distance = 2;
        }
        CMD_RATTACKXY | CMD_SATTACKXY => {
            dir = get_direction(ptile, point);
            graphic = Some(player_graphic::Attack);
        }
        CMD_SPELLXY => {
            dir = get_direction(ptile, point);
            graphic = Some(spell(w_param1));
        }
        CMD_SPELLXYD => {
            dir = Direction::from_u8(w_param2 as u8);
            graphic = Some(spell(w_param1));
        }
        CMD_WALKXY => minimal_walk_distance = 1,
        CMD_TALKXY | CMD_DISARMXY | CMD_OPOBJXY | CMD_GOTOGETITEM | CMD_GOTOAGETITEM => minimal_walk_distance = 2,
        _ => return,
    }
    if minimal_walk_distance >= 0 && pf != point {
        let mut test_walk_path = [0i8; MaxPathLength];
        let steps = crate::engine::path::find_path(ctx, &|ctx, pos| pos_ok_player(ctx, pnum, pos), pf, point, &mut test_walk_path);
        if steps == 0 {
            return;
        }
        if steps >= minimal_walk_distance {
            graphic = Some(player_graphic::Walk);
            dir = match test_walk_path[0] as i32 {
                WALK_N => Direction::North,
                WALK_NE => Direction::NorthEast,
                WALK_E => Direction::East,
                WALK_SE => Direction::SouthEast,
                WALK_S => Direction::South,
                WALK_SW => Direction::SouthWest,
                WALK_W => Direction::West,
                WALK_NW => Direction::NorthWest,
                _ => dir,
            };
            if !plr_dir_ok(ctx, pnum, dir) {
                return;
            }
        }
    }
    let Some(graphic) = graphic else { return };
    if ctx.diablo.headless_mode {
        return;
    }
    load_plr_gfx(ctx, pnum, graphic);
    let sprites = ctx.players.Players[pnum].AnimationData[graphic as usize].sprites_for_direction(dir).expect("sprites");
    let first = sprites.get(0);
    let progress = ctx.nthread.ProgressToNextGameTick;
    let p = &mut ctx.players.Players[pnum];
    if p.previewCelSprite.as_ref().is_none_or(|s| *s != first) {
        p.previewCelSprite = Some(first);
        p.progressToNextGameTickWhenPreviewWasSet = progress as i8;
    }
}

/// Original: `devilution::PlayerAtPosition` (player.cpp).
// @port player.cpp|devilution::PlayerAtPosition(Point position) sha=08077092df5b
pub fn player_at_position(ctx: &Ctx, position: Point) -> Option<usize> {
    if !in_dungeon_bounds(position) {
        return None;
    }
    let player_index = ctx.gendung.dPlayer[position.x as usize][position.y as usize];
    if player_index == 0 {
        return None;
    }
    Some((player_index as i32).unsigned_abs() as usize - 1)
}

/// Original: `devilution::LoadPlrGFX` (player.cpp).
// @port player.cpp|devilution::LoadPlrGFX(Player &player, player_graphic graphic) sha=c818be18bb86
pub fn load_plr_gfx(ctx: &mut Ctx, pnum: usize, graphic: player_graphic) {
    if ctx.diablo.headless_mode {
        return;
    }
    if ctx.players.Players[pnum].AnimationData[graphic as usize].sprites.is_some() {
        return;
    }
    let (class, gfxnum, block_flag) = {
        let p = &ctx.players.Players[pnum];
        (p._pClass, p._pgfxnum, p._pBlockFlag)
    };
    let cls = get_player_sprite_class(ctx, class);
    let anim_weapon_id = get_player_weapon_graphic(ctx, graphic, weapon_graphic_of(gfxnum));
    let path = PlayersData[cls as usize].classPath;
    let town = ctx.gendung.leveltype == DungeonType::Town;
    let sz_cel = match graphic {
        player_graphic::Stand => {
            if town {
                "st"
            } else {
                "as"
            }
        }
        player_graphic::Walk => {
            if town {
                "wl"
            } else {
                "aw"
            }
        }
        player_graphic::Attack => {
            if town {
                return;
            }
            "at"
        }
        player_graphic::Hit => {
            if town {
                return;
            }
            "ht"
        }
        player_graphic::Lightning => "lm",
        player_graphic::Fire => "fm",
        player_graphic::Magic => "qm",
        player_graphic::Death => {
            if anim_weapon_id != PlayerWeaponGraphic::Unarmed {
                return;
            }
            "dt"
        }
        player_graphic::Block => {
            if town || !block_flag {
                return;
            }
            "bl"
        }
    };
    let prefix = format!(
        "{}{}{}",
        CharChar[cls as usize] as char,
        ArmourChar[(gfxnum >> 4) as usize] as char,
        WepChar[anim_weapon_id as usize] as char
    );
    let psz_name = format!("plrgfx\\{path}\\{prefix}\\{prefix}{sz_cel}");
    let animation_width = get_player_sprite_width(cls, graphic, anim_weapon_id);
    let mut sheet = crate::engine::load_sprites::load_cl2_sheet(ctx, &psz_name, animation_width);
    if let Some(trn) = crate::engine::trn::get_class_trn(ctx, class) {
        crate::engine::render::clx_render::clx_apply_trans_sheet(&mut sheet, &trn);
    }
    ctx.players.Players[pnum].AnimationData[graphic as usize].sprites = Some(sheet);
}

/// Original: `devilution::InitPlayerGFX` (player.cpp).
// @port player.cpp|devilution::InitPlayerGFX(Player &player) sha=a6553665ad81
pub fn init_player_gfx(ctx: &mut Ctx, pnum: usize) {
    if ctx.diablo.headless_mode {
        return;
    }
    reset_player_gfx(ctx, pnum);
    if ctx.players.Players[pnum]._pHitPoints >> 6 == 0 {
        ctx.players.Players[pnum]._pgfxnum &= !0xF;
        load_plr_gfx(ctx, pnum, player_graphic::Death);
        return;
    }
    for i in 0..9u8 {
        let graphic = player_graphic::from_raw(i);
        if graphic == player_graphic::Death {
            continue;
        }
        load_plr_gfx(ctx, pnum, graphic);
    }
}

/// Original: `devilution::ResetPlayerGFX` (player.cpp).
// @port player.cpp|devilution::ResetPlayerGFX(Player &player) sha=a97827def5a1
pub fn reset_player_gfx(ctx: &mut Ctx, pnum: usize) {
    let p = &mut ctx.players.Players[pnum];
    p.AnimInfo.sprites = None;
    for a in p.AnimationData.iter_mut() {
        a.sprites = None;
    }
}

/// Original: `devilution::NewPlrAnim` (player.cpp). Defaults: flags None, numSkippedFrames 0,
/// distributeFramesBeforeFrame 0.
// @port player.cpp|devilution::NewPlrAnim(Player &player, player_graphic graphic, Direction dir, AnimationDistributionFlags flags , int8_t numSkippedFrames , int8_t distributeFramesBeforeFrame) sha=a9454ddb5824
pub fn new_plr_anim(ctx: &mut Ctx, pnum: usize, graphic: player_graphic, dir: Direction, flags: AnimationDistributionFlags, num_skipped_frames: i8, distribute_frames_before_frame: i8) {
    load_plr_gfx(ctx, pnum, graphic);
    let mut sprites = None;
    let mut preview_shown = 0;
    if !ctx.diablo.headless_mode {
        let p = &ctx.players.Players[pnum];
        sprites = p.AnimationData[graphic as usize].sprites_for_direction(dir);
        if let (Some(prev), Some(s)) = (&p.previewCelSprite, &sprites) {
            if s.get(0) == *prev && !p.is_walking() {
                preview_shown = (AnimationInfo::BASE_VALUE_FRACTION - p.progressToNextGameTickWhenPreviewWasSet as i32).clamp(0, AnimationInfo::BASE_VALUE_FRACTION);
            }
        }
    }
    let p = &mut ctx.players.Players[pnum];
    let (frames, ticks) = p.get_animation_frames_and_ticks_per_frame(graphic);
    p.AnimInfo.set_new_animation(sprites, frames, ticks, flags, num_skipped_frames, distribute_frames_before_frame, preview_shown as u8);
}

/// Original: `devilution::SetPlrAnims` (player.cpp).
// @port player.cpp|devilution::SetPlrAnims(Player &player) sha=de8c7e7b86a2
pub fn set_plr_anims(ctx: &mut Ctx, pnum: usize) {
    let town = ctx.gendung.leveltype == DungeonType::Town;
    let p = &mut ctx.players.Players[pnum];
    let pc = p._pClass;
    let d = &PlayersAnimData[pc as usize];
    let gn = weapon_graphic_of(p._pgfxnum);
    if town {
        p._pNFrames = d.townIdleFrames;
        p._pWFrames = d.townWalkingFrames;
    } else {
        p._pNFrames = d.idleFrames;
        p._pWFrames = d.walkingFrames;
        p._pHFrames = d.recoveryFrames;
        p._pBFrames = d.blockingFrames;
        let (f, a) = match gn {
            PlayerWeaponGraphic::Unarmed => (d.unarmedFrames, d.unarmedActionFrame),
            PlayerWeaponGraphic::UnarmedShield => (d.unarmedShieldFrames, d.unarmedShieldActionFrame),
            PlayerWeaponGraphic::Sword => (d.swordFrames, d.swordActionFrame),
            PlayerWeaponGraphic::SwordShield => (d.swordShieldFrames, d.swordShieldActionFrame),
            PlayerWeaponGraphic::Bow => (d.bowFrames, d.bowActionFrame),
            PlayerWeaponGraphic::Axe => (d.axeFrames, d.axeActionFrame),
            PlayerWeaponGraphic::Mace => (d.maceFrames, d.maceActionFrame),
            PlayerWeaponGraphic::MaceShield => (d.maceShieldFrames, d.maceShieldActionFrame),
            PlayerWeaponGraphic::Staff => (d.staffFrames, d.staffActionFrame),
        };
        p._pAFrames = f;
        p._pAFNum = a;
    }
    p._pDFrames = d.deathFrames;
    p._pSFrames = d.castingFrames;
    p._pSFNum = d.castingActionFrame;
    let armor_graphic_index = p._pgfxnum & !0xF;
    if matches!(pc, HeroClass::Warrior | HeroClass::Barbarian) {
        if gn == PlayerWeaponGraphic::Bow && !town {
            p._pNFrames = 8;
        }
        if armor_graphic_index > 0 {
            p._pDFrames = 15;
        }
    }
}

/// `EnsureValidReadiedSpell` (spells.cpp), for callers that think of it as a player helper.
pub fn ensure_valid_readied_spell(ctx: &mut Ctx, pnum: usize) {
    crate::spells::ensure_valid_readied_spell(ctx, pnum);
}

/// Original: `devilution::CreatePlayer` (player.cpp).
// @port player.cpp|devilution::CreatePlayer(Player &player, HeroClass c) sha=73e7afaaa84b
pub fn create_player(ctx: &mut Ctx, pnum: usize, c: HeroClass) {
    ctx.players.Players[pnum] = Player::default();
    let t = ctx.platform.ticks();
    ctx.rng.set_rnd_seed(t);
    let pd = &PlayersData[c as usize];
    {
        let p = &mut ctx.players.Players[pnum];
        p._pLevel = 1;
        p._pClass = c;
        p._pBaseStr = pd.baseStr as i32;
        p._pStrength = p._pBaseStr;
        p._pBaseMag = pd.baseMag as i32;
        p._pMagic = p._pBaseMag;
        p._pBaseDex = pd.baseDex as i32;
        p._pDexterity = p._pBaseDex;
        p._pBaseVit = pd.baseVit as i32;
        p._pVitality = p._pBaseVit;
        p._pBaseToBlk = pd.blockBonus as i32;
        p._pHitPoints = p.calculate_base_life();
        p._pMaxHP = p._pHitPoints;
        p._pHPBase = p._pHitPoints;
        p._pMaxHPBase = p._pHitPoints;
        p._pMana = p.calculate_base_mana();
        p._pMaxMana = p._pMana;
        p._pManaBase = p._pMana;
        p._pMaxManaBase = p._pMana;
        p._pMaxLvl = p._pLevel;
        p._pExperience = 0;
        p._pNextExper = ExpLvlsTbl[1];
        p._pArmorClass = 0;
        p._pLightRad = 10;
        p._pInfraFlag = false;
        p._pRSplType = SpellType::Skill;
        let s = pd.skill;
        p._pAblSpells = crate::spells::get_spell_bitmask(s);
        p._pRSpell = s;
        if c == HeroClass::Sorcerer {
            p._pMemSpells = crate::spells::get_spell_bitmask(SpellID::Firebolt);
            p._pRSplType = SpellType::Spell;
            p._pRSpell = SpellID::Firebolt;
        } else {
            p._pMemSpells = 0;
        }
        p._pSplLvl = [0; 64];
        p._pSpellFlags = SpellFlag(0);
        if p._pClass == HeroClass::Sorcerer {
            p._pSplLvl[SpellID::Firebolt as usize] = 2;
        }
        p._pSplHotKey = [SpellID::Invalid; NumHotkeys];
        let anim_weapon_id = match c {
            HeroClass::Warrior | HeroClass::Bard | HeroClass::Barbarian => PlayerWeaponGraphic::SwordShield,
            HeroClass::Rogue => PlayerWeaponGraphic::Bow,
            HeroClass::Sorcerer | HeroClass::Monk => PlayerWeaponGraphic::Staff,
        };
        p._pgfxnum = anim_weapon_id as u8;
        p._pLvlVisited = [false; NUMLEVELS];
        for i in 0..10 {
            p._pSLvlVisited[i] = false;
        }
        p._pLvlChanging = false;
        p.pTownWarps = 0;
        p.pLvlLoad = 0;
        p.pManaShield = false;
        p.pDamAcFlags = ItemSpecialEffectHf::None;
        p.wReflections = 0;
    }
    init_dung_msgs(ctx, pnum);
    crate::items::create_plr_items(ctx, pnum);
    ctx.rng.set_rnd_seed(0);
}

const ALL_ATTRIBUTES: [CharacterAttribute; 4] = [CharacterAttribute::Strength, CharacterAttribute::Magic, CharacterAttribute::Dexterity, CharacterAttribute::Vitality];

/// Original: `devilution::CalcStatDiff` (player.cpp).
// @port player.cpp|devilution::CalcStatDiff(Player &player) sha=522e0d86931a
pub fn calc_stat_diff(p: &Player) -> i32 {
    ALL_ATTRIBUTES.iter().map(|&a| p.get_maximum_attribute_value(a) - p.get_base_attribute_value(a)).sum()
}

/// Original: `devilution::NextPlrLevel` (player.cpp) (non-debug since `AddPlrExperience` uses it).
// @port player.cpp|devilution::NextPlrLevel(Player &player) sha=553a5f5d8459
pub fn next_plr_level(ctx: &mut Ctx, pnum: usize) {
    {
        let p = &mut ctx.players.Players[pnum];
        p._pLevel += 1;
        p._pMaxLvl += 1;
    }
    crate::items::calc_plr_inv(ctx, pnum, true);
    let me = is_my(ctx, pnum);
    {
        let p = &mut ctx.players.Players[pnum];
        if calc_stat_diff(p) < 5 {
            p._pStatPts = calc_stat_diff(p);
        } else {
            p._pStatPts += 5;
        }
        p._pNextExper = ExpLvlsTbl[(p._pLevel as i32).min(MaxCharacterLevel - 1) as usize];
        let hp = PlayersData[p._pClass as usize].lvlLife as i32;
        p._pMaxHP += hp;
        p._pHitPoints = p._pMaxHP;
        p._pMaxHPBase += hp;
        p._pHPBase = p._pMaxHPBase;
    }
    if me {
        redraw_component(ctx, PanelDrawComponent::Health);
    }
    {
        let p = &mut ctx.players.Players[pnum];
        let mana = PlayersData[p._pClass as usize].lvlMana as i32;
        p._pMaxMana += mana;
        p._pMaxManaBase += mana;
        if p._pIFlags.has_none_of(ItemSpecialEffect::NoMana) {
            p._pMana = p._pMaxMana;
            p._pManaBase = p._pMaxManaBase;
        }
    }
    if me {
        redraw_component(ctx, PanelDrawComponent::Mana);
    }
    if ctx.controls.control_mode != crate::controls::ControlTypes::KeyboardAndMouse {
        crate::controls::plrctrls::focus_on_char_info(ctx);
    }
    crate::items::calc_plr_inv(ctx, pnum, true);
}

/// Original: `devilution::AddPlrExperience` (player.cpp).
// @port player.cpp|devilution::AddPlrExperience(Player &player, int lvl, int exp) sha=e2ab579b7501
pub fn add_plr_experience(ctx: &mut Ctx, pnum: usize, lvl: i32, exp: i32) {
    if !is_my(ctx, pnum) || ctx.players.Players[pnum]._pHitPoints <= 0 {
        return;
    }
    let plevel = ctx.players.Players[pnum]._pLevel as i32;
    if plevel >= MaxCharacterLevel {
        ctx.players.Players[pnum]._pLevel = MaxCharacterLevel as i8;
        return;
    }
    let mut clamped_exp = ((exp as f64 * (1.0 + (lvl - plevel) as f64 / 10.0)) as i32).max(0) as u32;
    if ctx.init.gb_is_multiplayer {
        let clamped_player_level = plevel.clamp(1, MaxCharacterLevel) as u32;
        clamped_exp = clamped_exp.min(ExpLvlsTbl[clamped_player_level as usize] / 20).min(200 * clamped_player_level);
    }
    let max_experience = ExpLvlsTbl[MaxCharacterLevel as usize - 1];
    {
        let p = &mut ctx.players.Players[pnum];
        p._pExperience = p._pExperience.wrapping_add(clamped_exp).min(max_experience);
    }
    if ctx.options.gameplay.experience_bar.get() {
        redraw_everything(ctx);
    }
    let mut new_lvl = plevel;
    while new_lvl < MaxCharacterLevel && ctx.players.Players[pnum]._pExperience >= ExpLvlsTbl[new_lvl as usize] {
        new_lvl += 1;
    }
    if new_lvl != plevel {
        for _ in 0..(new_lvl - plevel) {
            next_plr_level(ctx, pnum);
        }
    }
    let l = ctx.players.Players[pnum]._pLevel;
    crate::msg::net_send_cmd_param1(ctx, false, CMD_PLRLEVEL, l as u16);
}

/// Original: `devilution::AddPlrMonstExper` (player.cpp).
// @port player.cpp|devilution::AddPlrMonstExper(int lvl, int exp, char pmask) sha=e805578e6f37
pub fn add_plr_monst_exper(ctx: &mut Ctx, lvl: i32, exp: i32, pmask: i8) {
    let mut totplrs = 0;
    for i in 0..ctx.players.Players.len() {
        if ((1i32 << i) & pmask as i32) != 0 {
            totplrs += 1;
        }
    }
    if totplrs != 0 {
        let e = exp / totplrs;
        if (pmask as i32 & (1 << ctx.players.MyPlayerId)) != 0 {
            let me = ctx.players.my_player_id();
            add_plr_experience(ctx, me, lvl, e);
        }
    }
}

/// Original: `devilution::InitPlayer` (player.cpp).
// @port player.cpp|devilution::InitPlayer(Player &player, bool firstTime) sha=e6085630eed8
pub fn init_player(ctx: &mut Ctx, pnum: usize, first_time: bool) {
    let me = is_my(ctx, pnum);
    if first_time {
        {
            let p = &mut ctx.players.Players[pnum];
            p._pRSplType = SpellType::Invalid;
            p._pRSpell = SpellID::Invalid;
        }
        if me {
            crate::loadsave::load_hotkeys(ctx);
        }
        let p = &mut ctx.players.Players[pnum];
        p._pSBkSpell = SpellID::Invalid;
        p.queuedSpell.spellId = p._pRSpell;
        p.queuedSpell.spellType = p._pRSplType;
        p.pManaShield = false;
        p.wReflections = 0;
    }
    if is_on_active_level(ctx, pnum) {
        set_plr_anims(ctx, pnum);
        clear_state_variables(ctx, pnum);
        if ctx.players.Players[pnum]._pHitPoints >> 6 > 0 {
            ctx.players.Players[pnum]._pmode = PM_STAND;
            new_plr_anim(ctx, pnum, player_graphic::Stand, Direction::South, AnimationDistributionFlags::None, 0, 0);
            let nf = ctx.players.Players[pnum]._pNFrames as i32;
            let cf = ctx.rng.generate_rnd(nf - 1);
            let tc = ctx.rng.generate_rnd(3);
            let p = &mut ctx.players.Players[pnum];
            p.AnimInfo.currentFrame = cf as i8;
            p.AnimInfo.tickCounterOfCurrentFrame = tc as i8;
        } else {
            ctx.players.Players[pnum]._pgfxnum &= !0xF;
            ctx.players.Players[pnum]._pmode = PM_DEATH;
            new_plr_anim(ctx, pnum, player_graphic::Death, Direction::South, AnimationDistributionFlags::None, 0, 0);
            let p = &mut ctx.players.Players[pnum];
            p.AnimInfo.currentFrame = p.AnimInfo.numberOfFrames - 2;
        }
        ctx.players.Players[pnum]._pdir = Direction::South;
        if me && (!first_time || ctx.gendung.leveltype != DungeonType::Town) {
            ctx.players.Players[pnum].position.tile = ctx.gendung.ViewPosition;
        }
        set_player_old(ctx, pnum);
        {
            let p = &mut ctx.players.Players[pnum];
            p.walkpath[0] = WALK_NONE as i8;
            p.destAction = ACTION_NONE;
        }
        let (tile, rad) = (ctx.players.Players[pnum].position.tile, ctx.players.Players[pnum]._pLightRad);
        if me {
            let id = crate::lighting::add_light(ctx, tile, rad as u8);
            ctx.players.Players[pnum].lightId = id;
            crate::lighting::change_light_xy(ctx, id, tile);
        } else {
            ctx.players.Players[pnum].lightId = NO_LIGHT;
        }
        crate::lighting::activate_vision(ctx, tile, rad as i32, pnum as i32);
    }
    let p = &mut ctx.players.Players[pnum];
    let s = PlayersData[p._pClass as usize].skill;
    p._pAblSpells = crate::spells::get_spell_bitmask(s);
    p._pNextExper = ExpLvlsTbl[(p._pLevel as i32).min(MaxCharacterLevel - 1) as usize];
    p._pInvincible = false;
    if me {
        ctx.players.MyPlayerIsDead = false;
    }
}

/// Original: `devilution::InitMultiView` (player.cpp).
// @port player.cpp|devilution::InitMultiView() sha=7c86f830bdb2
pub fn init_multi_view(ctx: &mut Ctx) {
    let me = ctx.players.my_player_id();
    ctx.gendung.ViewPosition = ctx.players.Players[me].position.tile;
}

/// Original: `devilution::PlrClrTrans` (player.cpp).
// @port player.cpp|devilution::PlrClrTrans(Point position) sha=e37b65dad826
pub fn plr_clr_trans(ctx: &mut Ctx, position: Point) {
    for i in position.y - 1..=position.y + 1 {
        for j in position.x - 1..=position.x + 1 {
            let tv = ctx.gendung.dTransVal[j as usize][i as usize] as u8;
            ctx.gendung.TransList[tv as usize] = false;
        }
    }
}

/// Original: `devilution::PlrDoTrans` (player.cpp).
// @port player.cpp|devilution::PlrDoTrans(Point position) sha=c6dab71f048f
pub fn plr_do_trans(ctx: &mut Ctx, position: Point) {
    if !matches!(ctx.gendung.leveltype, DungeonType::Cathedral | DungeonType::Catacombs | DungeonType::Crypt) {
        ctx.gendung.TransList[1] = true;
        return;
    }
    for i in position.y - 1..=position.y + 1 {
        for j in position.x - 1..=position.x + 1 {
            let tv = ctx.gendung.dTransVal[j as usize][i as usize];
            if crate::engine::path::is_tile_not_solid(ctx, Point::new(j, i)) && tv != 0 {
                ctx.gendung.TransList[tv as u8 as usize] = true;
            }
        }
    }
}

/// Original: `devilution::SetPlayerOld` (player.cpp).
// @port player.cpp|devilution::SetPlayerOld(Player &player) sha=1a39898e5d9b
pub fn set_player_old(ctx: &mut Ctx, pnum: usize) {
    let p = &mut ctx.players.Players[pnum];
    p.position.old = p.position.tile;
}

/// Original: `devilution::FixPlayerLocation` (player.cpp).
// @port player.cpp|devilution::FixPlayerLocation(Player &player, Direction bDir) sha=418662a0cf60
pub fn fix_player_location(ctx: &mut Ctx, pnum: usize, b_dir: Direction) {
    let tile;
    {
        let p = &mut ctx.players.Players[pnum];
        p.position.future = p.position.tile;
        p._pdir = b_dir;
        tile = p.position.tile;
    }
    if is_my(ctx, pnum) {
        ctx.gendung.ViewPosition = tile;
    }
    let id = ctx.players.Players[pnum].lightId;
    crate::lighting::change_light_xy(ctx, id, tile);
    crate::lighting::change_vision_xy(ctx, pnum as i32, tile);
}

/// Original: `devilution::StartStand` (player.cpp).
// @port player.cpp|devilution::StartStand(Player &player, Direction dir) sha=d62c2f08198f
pub fn start_stand(ctx: &mut Ctx, pnum: usize, dir: Direction) {
    if invincible_dead_me(ctx, pnum) {
        sync_plr_kill(ctx, pnum, DeathReason::Unknown);
        return;
    }
    new_plr_anim(ctx, pnum, player_graphic::Stand, dir, AnimationDistributionFlags::None, 0, 0);
    ctx.players.Players[pnum]._pmode = PM_STAND;
    fix_player_location(ctx, pnum, dir);
    fix_plr_walk_tags(ctx, pnum);
    let t = ctx.players.Players[pnum].position.tile;
    ctx.gendung.dPlayer[t.x as usize][t.y as usize] = (pnum + 1) as i8;
    set_player_old(ctx, pnum);
}

/// Original: `devilution::StartPlrBlock` (player.cpp).
// @port player.cpp|devilution::StartPlrBlock(Player &player, Direction dir) sha=fb2d852f5891
pub fn start_plr_block(ctx: &mut Ctx, pnum: usize, dir: Direction) {
    if invincible_dead_me(ctx, pnum) {
        sync_plr_kill(ctx, pnum, DeathReason::Unknown);
        return;
    }
    let t = ctx.players.Players[pnum].position.tile;
    crate::effects::play_sfx_loc(ctx, crate::effects_data::IS_ISWORD, t, true);
    let p = &ctx.players.Players[pnum];
    let mut skipped: i8 = 0;
    if p._pIFlags.has_any_of(ItemSpecialEffect::FastBlock) {
        skipped = p._pBFrames - 2;
    }
    new_plr_anim(ctx, pnum, player_graphic::Block, dir, AnimationDistributionFlags::SkipsDelayOfLastFrame, skipped, 0);
    ctx.players.Players[pnum]._pmode = PM_BLOCK;
    fix_player_location(ctx, pnum, dir);
    set_player_old(ctx, pnum);
}

/// Original: `devilution::FixPlrWalkTags` (player.cpp).
// @port player.cpp|devilution::FixPlrWalkTags(const Player &player) sha=1059dd7ca68c
pub fn fix_plr_walk_tags(ctx: &mut Ctx, pnum: usize) {
    for y in 0..MAXDUNY {
        for x in 0..MAXDUNX {
            if player_at_position(ctx, Point::new(x as i32, y as i32)) == Some(pnum) {
                ctx.gendung.dPlayer[x][y] = 0;
            }
        }
    }
}

/// Original: `devilution::StartPlrHit` (player.cpp).
// @port player.cpp|devilution::StartPlrHit(Player &player, int dam, bool forcehit) sha=980eb0709be7
pub fn start_plr_hit(ctx: &mut Ctx, pnum: usize, dam: i32, forcehit: bool) {
    if invincible_dead_me(ctx, pnum) {
        sync_plr_kill(ctx, pnum, DeathReason::Unknown);
        return;
    }
    player_say(ctx, pnum, HeroSpeech::ArghClang);
    redraw_component(ctx, PanelDrawComponent::Health);
    let p = &ctx.players.Players[pnum];
    let lvl = p._pLevel as i32;
    if p._pClass == HeroClass::Barbarian {
        if dam >> 6 < lvl + lvl / 4 && !forcehit {
            return;
        }
    } else if dam >> 6 < lvl && !forcehit {
        return;
    }
    let pd = p._pdir;
    use ItemSpecialEffect as E;
    let skipped: i8 = if p._pIFlags.has_any_of(E::FastestHitRecovery) {
        3
    } else if p._pIFlags.has_any_of(E::FasterHitRecovery) {
        2
    } else if p._pIFlags.has_any_of(E::FastHitRecovery) {
        1
    } else {
        0
    };
    new_plr_anim(ctx, pnum, player_graphic::Hit, pd, AnimationDistributionFlags::None, skipped, 0);
    ctx.players.Players[pnum]._pmode = PM_GOTHIT;
    fix_player_location(ctx, pnum, pd);
    fix_plr_walk_tags(ctx, pnum);
    let t = ctx.players.Players[pnum].position.tile;
    ctx.gendung.dPlayer[t.x as usize][t.y as usize] = (pnum + 1) as i8;
    set_player_old(ctx, pnum);
}

/// Original: `devilution::StartPlayerKill` (player.cpp).
// @port player.cpp|devilution::StartPlayerKill(Player &player, DeathReason deathReason) sha=d6bd3fae2e04
pub fn start_player_kill(ctx: &mut Ctx, pnum: usize, death_reason: DeathReason) {
    {
        let p = &ctx.players.Players[pnum];
        if p._pHitPoints <= 0 && p._pmode == PM_DEATH {
            return;
        }
    }
    let me = is_my(ctx, pnum);
    if me {
        crate::msg::net_send_cmd_param1(ctx, true, CMD_PLRDEAD, death_reason as u16);
    }
    let drop_gold = {
        let p = &ctx.players.Players[pnum];
        !ctx.init.gb_is_multiplayer || !(p.is_on_level(16) || p.is_on_arena_level())
    };
    let drop_items = drop_gold && death_reason == DeathReason::MonsterOrTrap;
    let drop_ear = drop_gold && death_reason == DeathReason::Player;
    player_say(ctx, pnum, HeroSpeech::AuughUh);
    if ctx.players.Players[pnum]._pgfxnum != 0 {
        if drop_items {
            ctx.players.Players[pnum]._pgfxnum = 0;
        } else {
            ctx.players.Players[pnum]._pgfxnum &= !0xF;
        }
        reset_player_gfx(ctx, pnum);
        set_plr_anims(ctx, pnum);
    }
    let pd = ctx.players.Players[pnum]._pdir;
    new_plr_anim(ctx, pnum, player_graphic::Death, pd, AnimationDistributionFlags::None, 0, 0);
    {
        let p = &mut ctx.players.Players[pnum];
        p._pBlockFlag = false;
        p._pmode = PM_DEATH;
        p._pInvincible = true;
    }
    set_player_hit_points(ctx, pnum, 0);
    if !me && drop_items {
        for item in ctx.players.Players[pnum].InvBody.iter_mut() {
            item.clear();
        }
        crate::items::calc_plr_inv(ctx, pnum, false);
    }
    if is_on_active_level(ctx, pnum) {
        let pd = ctx.players.Players[pnum]._pdir;
        fix_player_location(ctx, pnum, pd);
        fix_plr_walk_tags(ctx, pnum);
        let t = ctx.players.Players[pnum].position.tile;
        ctx.gendung.dFlags[t.x as usize][t.y as usize] |= DungeonFlag::DeadPlayer;
        set_player_old(ctx, pnum);
        if me {
            redraw_component(ctx, PanelDrawComponent::Health);
            if !ctx.players.Players[pnum].HoldItem.is_empty() {
                let hold = ctx.players.Players[pnum].HoldItem.pop();
                dead_item(ctx, pnum, hold, Displacement::new(0, 0));
                crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_HAND);
            }
            if drop_gold {
                drop_half_players_gold(ctx, pnum);
            }
            if drop_ear {
                let mut ear = Item::default();
                crate::items::initialize_item(ctx, &mut ear, IDI_EAR);
                let name = ctx.players.Players[pnum]._pName;
                ear._iName.set(&format!("Ear of {}", name.as_str()));
                ear._iIName = CStr::from_raw(name.bytes());
                ear._iCurs = match ctx.players.Players[pnum]._pClass {
                    HeroClass::Sorcerer => ICURS_EAR_SORCERER,
                    HeroClass::Warrior => ICURS_EAR_WARRIOR,
                    _ => ICURS_EAR_ROGUE,
                } as u8;
                let nb = name.bytes();
                // char is signed in the original
                let c = |i: usize| nb[i] as i8 as i32;
                ear._iCreateInfo = ((c(0) << 8) | c(1)) as u16;
                ear._iSeed = ((c(2) << 24) | (c(3) << 16) | (c(4) << 8) | c(5)) as u32;
                ear._ivalue = ctx.players.Players[pnum]._pLevel as i32;
                if crate::inv::find_get_item(ctx, ear._iSeed, IDI_EAR, ear._iCreateInfo) == -1 {
                    dead_item(ctx, pnum, ear, Displacement::new(0, 0));
                }
            }
            if drop_items {
                let mut pdd = ctx.players.Players[pnum]._pdir;
                for i in 0..NUM_INVLOC as usize {
                    pdd = left(pdd);
                    let it = ctx.players.Players[pnum].InvBody[i].pop();
                    dead_item(ctx, pnum, it, Displacement::from_direction(pdd));
                }
                crate::items::calc_plr_inv(ctx, pnum, false);
            }
        }
    }
    set_player_hit_points(ctx, pnum, 0);
}

/// Original: `devilution::StripTopGold` (player.cpp).
// @port player.cpp|devilution::StripTopGold(Player &player) sha=18b028d28b99
pub fn strip_top_gold(ctx: &mut Ctx, pnum: usize) {
    let n = ctx.players.Players[pnum]._pNumInv as usize;
    for i in 0..n {
        let item = &ctx.players.Players[pnum].InvList[i];
        if item.is_empty() || item._itype != ItemType::Gold {
            continue;
        }
        let max_gold = ctx.items.MaxGold;
        if item._ivalue > max_gold {
            let mut excess_gold = Item::default();
            let v = item._ivalue - max_gold;
            crate::items::make_gold_stack(ctx, &mut excess_gold, v);
            ctx.players.Players[pnum].InvList[i]._ivalue = max_gold;
            if !crate::inv::gold_auto_place(ctx, pnum, &mut excess_gold) {
                dead_item(ctx, pnum, excess_gold, Displacement::new(0, 0));
            }
        }
    }
    let g = crate::inv::calculate_gold(ctx, pnum);
    ctx.players.Players[pnum]._pGold = g;
}

/// Original: `devilution::ApplyPlrDamage` (player.cpp). Defaults: minHP 0, frac 0,
/// deathReason MonsterOrTrap.
// @port player.cpp|devilution::ApplyPlrDamage(DamageType damageType, Player &player, int dam, int minHP , int frac , DeathReason deathReason) sha=e20b9cf66835
pub fn apply_plr_damage(ctx: &mut Ctx, damage_type: DamageType, pnum: usize, dam: i32, min_hp: i32, frac: i32, death_reason: DeathReason) {
    let mut total_damage = (dam << 6) + frac;
    let me = is_my(ctx, pnum);
    if me && ctx.players.Players[pnum]._pHitPoints > 0 {
        crate::qol::floatingnumbers::add_floating_number_player(ctx, damage_type, pnum, total_damage);
    }
    if total_damage > 0 && ctx.players.Players[pnum].pManaShield {
        let mana_shield_level = ctx.players.Players[pnum]._pSplLvl[SpellID::ManaShield as usize];
        if mana_shield_level > 0 {
            total_damage += total_damage / -ctx.players.Players[pnum].get_mana_shield_damage_reduction();
        }
        if me {
            redraw_component(ctx, PanelDrawComponent::Mana);
        }
        let p = &mut ctx.players.Players[pnum];
        if p._pMana >= total_damage {
            p._pMana -= total_damage;
            p._pManaBase -= total_damage;
            total_damage = 0;
        } else {
            total_damage -= p._pMana;
            if mana_shield_level > 0 {
                total_damage += total_damage / (p.get_mana_shield_damage_reduction() - 1);
            }
            p._pMana = 0;
            p._pManaBase = p._pMaxManaBase - p._pMaxMana;
            if me {
                crate::msg::net_send_cmd(ctx, true, CMD_REMSHIELD);
            }
        }
    }
    if total_damage == 0 {
        return;
    }
    redraw_component(ctx, PanelDrawComponent::Health);
    {
        let p = &mut ctx.players.Players[pnum];
        p._pHitPoints -= total_damage;
        p._pHPBase -= total_damage;
        if p._pHitPoints > p._pMaxHP {
            p._pHitPoints = p._pMaxHP;
            p._pHPBase = p._pMaxHPBase;
        }
    }
    let min_hit_points = min_hp << 6;
    if ctx.players.Players[pnum]._pHitPoints < min_hit_points {
        set_player_hit_points(ctx, pnum, min_hit_points);
    }
    if ctx.players.Players[pnum]._pHitPoints >> 6 <= 0 {
        sync_plr_kill(ctx, pnum, death_reason);
    }
}

/// Original: `devilution::SyncPlrKill` (player.cpp).
// @port player.cpp|devilution::SyncPlrKill(Player &player, DeathReason deathReason) sha=d65813b5ced1
pub fn sync_plr_kill(ctx: &mut Ctx, pnum: usize, death_reason: DeathReason) {
    if ctx.players.Players[pnum]._pHitPoints <= 0 && ctx.gendung.leveltype == DungeonType::Town {
        set_player_hit_points(ctx, pnum, 64);
        return;
    }
    set_player_hit_points(ctx, pnum, 0);
    start_player_kill(ctx, pnum, death_reason);
}

/// Original: `devilution::RemovePlrMissiles` (player.cpp).
// @port player.cpp|devilution::RemovePlrMissiles(const Player &player) sha=d36320b9f5e9
pub fn remove_plr_missiles(ctx: &mut Ctx, pnum: usize) {
    if ctx.gendung.leveltype != DungeonType::Town && is_my(ctx, pnum) {
        let gid = ctx.players.MyPlayerId;
        let gt = ctx.monster.Monsters[gid].position.tile;
        if gt.x != 1 || gt.y != 0 {
            crate::monster::kill_my_golem(ctx);
            let (corpse, dir) = (crate::monster::monster_corpse_id(ctx, gid), ctx.monster.Monsters[gid].direction);
            crate::dead::add_corpse(ctx, gt, corpse, dir);
            ctx.gendung.dMonster[gt.x as usize][gt.y as usize] = 0;
            ctx.monster.Monsters[gid].isInvalid = true;
            crate::monster::delete_monster_list(ctx);
        }
    }
    for i in 0..ctx.missiles.Missiles.len() {
        let m = &ctx.missiles.Missiles[i];
        if m._mitype == MissileID::StoneCurse && m._misource as usize == pnum {
            let (v2, v1) = (m.var2 as usize, m.var1);
            ctx.monster.Monsters[v2].mode = MonsterMode::from_raw(v1 as u8);
        }
    }
}

/// Original: `devilution::StartNewLvl` (player.cpp).
// @port player.cpp|devilution::StartNewLvl(Player &player, interface_mode fom, int lvl) sha=514e139a689e
pub fn start_new_lvl(ctx: &mut Ctx, pnum: usize, fom: interface_mode, lvl: i32) {
    init_level_change(ctx, pnum);
    let me = is_my(ctx, pnum);
    match fom {
        WM_DIABNEXTLVL | WM_DIABPREVLVL | WM_DIABRTNLVL | WM_DIABTOWNWARP => ctx.players.Players[pnum].set_level(lvl as u8),
        WM_DIABSETLVL => {
            if me {
                ctx.gendung.setlvlnum = lvl as _setlevels;
            }
            let s = ctx.gendung.setlvlnum;
            ctx.players.Players[pnum].set_set_level(s);
        }
        WM_DIABTWARPUP => {
            let my = ctx.players.my_player_id();
            ctx.players.Players[my].pTownWarps |= 1 << (ctx.gendung.leveltype as i8 - 2);
            ctx.players.Players[pnum].set_level(lvl as u8);
        }
        WM_DIABRETOWN => {}
        _ => crate::appfat::app_fatal(ctx, "StartNewLvl"),
    }
    if me {
        ctx.players.Players[pnum]._pmode = PM_NEWLVL;
        ctx.players.Players[pnum]._pInvincible = true;
        ctx.platform.push_event(crate::platform::events::Event::Custom(fom));
        if ctx.init.gb_is_multiplayer {
            crate::msg::net_send_cmd_param2(ctx, true, CMD_NEWLVL, fom, lvl as u16);
        }
    }
}

/// Original: `devilution::RestartTownLvl` (player.cpp).
// @port player.cpp|devilution::RestartTownLvl(Player &player) sha=d2740911458b
pub fn restart_town_lvl(ctx: &mut Ctx, pnum: usize) {
    init_level_change(ctx, pnum);
    ctx.players.Players[pnum].set_level(0);
    ctx.players.Players[pnum]._pInvincible = false;
    set_player_hit_points(ctx, pnum, 64);
    {
        let p = &mut ctx.players.Players[pnum];
        p._pMana = 0;
        p._pManaBase = p._pMana - (p._pMaxMana - p._pMaxManaBase);
    }
    crate::items::calc_plr_inv(ctx, pnum, false);
    ctx.players.Players[pnum]._pmode = PM_NEWLVL;
    if is_my(ctx, pnum) {
        ctx.players.Players[pnum]._pInvincible = true;
        ctx.platform.push_event(crate::platform::events::Event::Custom(WM_DIABRETOWN));
    }
}

/// Original: `devilution::StartWarpLvl` (player.cpp).
// @port player.cpp|devilution::StartWarpLvl(Player &player, size_t pidx) sha=68bce19a9a69
pub fn start_warp_lvl(ctx: &mut Ctx, pnum: usize, pidx: usize) {
    init_level_change(ctx, pnum);
    if ctx.init.gb_is_multiplayer {
        if !ctx.players.Players[pnum].is_on_level(0) {
            ctx.players.Players[pnum].set_level(0);
        } else {
            let portal = &ctx.portal.Portals[pidx];
            let (setlvl, level) = (portal.setlvl, portal.level);
            if setlvl {
                ctx.players.Players[pnum].set_set_level(level as _setlevels);
            } else {
                ctx.players.Players[pnum].set_level(level as u8);
            }
        }
    }
    if is_my(ctx, pnum) {
        crate::portal::set_current_portal(ctx, pidx);
        ctx.players.Players[pnum]._pmode = PM_NEWLVL;
        ctx.players.Players[pnum]._pInvincible = true;
        ctx.platform.push_event(crate::platform::events::Event::Custom(WM_DIABWARPLVL));
    }
}

/// Original: `devilution::ProcessPlayers` (player.cpp).
// @port player.cpp|devilution::ProcessPlayers() sha=404cf2bf8f44
pub fn process_players(ctx: &mut Ctx) {
    let me = ctx.players.my_player_id();
    if ctx.players.Players[me].pLvlLoad > 0 {
        ctx.players.Players[me].pLvlLoad -= 1;
    }
    if ctx.effects.sfxdelay > 0 {
        ctx.effects.sfxdelay -= 1;
        if ctx.effects.sfxdelay == 0 {
            use crate::effects_data::*;
            match ctx.effects.sfxdnum {
                USFX_DEFILER1 => crate::minitext::init_q_text_msg(ctx, TEXT_DEFILER1),
                USFX_DEFILER2 => crate::minitext::init_q_text_msg(ctx, TEXT_DEFILER2),
                USFX_DEFILER3 => crate::minitext::init_q_text_msg(ctx, TEXT_DEFILER3),
                USFX_DEFILER4 => crate::minitext::init_q_text_msg(ctx, TEXT_DEFILER4),
                n => crate::effects::play_sfx(ctx, n),
            }
        }
    }
    validate_player(ctx);
    for pnum in 0..ctx.players.Players.len() {
        let active = {
            let p = &ctx.players.Players[pnum];
            p.plractive && is_on_active_level(ctx, pnum) && (pnum == me || !p._pLvlChanging)
        };
        if !active {
            continue;
        }
        check_cheat_stats(&mut ctx.players.Players[pnum]);
        if !plr_death_mode_ok(ctx, pnum) && (ctx.players.Players[pnum]._pHitPoints >> 6) <= 0 {
            sync_plr_kill(ctx, pnum, DeathReason::Unknown);
        }
        if pnum == me {
            let flags = ctx.players.Players[pnum]._pIFlags;
            if flags.has_any_of(ItemSpecialEffect::DrainLife) && ctx.gendung.leveltype != DungeonType::Town {
                apply_plr_damage(ctx, DamageType::Physical, pnum, 0, 0, 4, DeathReason::MonsterOrTrap);
            }
            let p = &mut ctx.players.Players[pnum];
            if p._pIFlags.has_any_of(ItemSpecialEffect::NoMana) && p._pManaBase > 0 {
                p._pManaBase -= p._pMana;
                p._pMana = 0;
                redraw_component(ctx, PanelDrawComponent::Mana);
            }
        }
        loop {
            let mode = ctx.players.Players[pnum]._pmode;
            let tplayer = match mode {
                PM_STAND | PM_NEWLVL | PM_QUIT => false,
                PM_WALK_NORTHWARDS | PM_WALK_SOUTHWARDS | PM_WALK_SIDEWAYS => do_walk(ctx, pnum, mode),
                PM_ATTACK => do_attack(ctx, pnum),
                PM_RATTACK => do_range_attack(ctx, pnum),
                PM_BLOCK => do_block(ctx, pnum),
                PM_SPELL => do_spell(ctx, pnum),
                PM_GOTHIT => do_got_hit(ctx, pnum),
                PM_DEATH => do_death(ctx, pnum),
                _ => false,
            };
            check_new_path(ctx, pnum, tplayer);
            if !tplayer {
                break;
            }
        }
        let p = &mut ctx.players.Players[pnum];
        p.previewCelSprite = None;
        if p._pmode != PM_DEATH || p.AnimInfo.tickCounterOfCurrentFrame != 40 {
            p.AnimInfo.process_animation(false);
        }
    }
}

/// Original: `devilution::ClrPlrPath` (player.cpp).
// @port player.cpp|devilution::ClrPlrPath(Player &player) sha=3c37b39bd2c0
pub fn clr_plr_path(ctx: &mut Ctx, pnum: usize) {
    ctx.players.Players[pnum].walkpath = [WALK_NONE as i8; MaxPathLength];
}

/// Original: `devilution::PosOkPlayer` (player.cpp).
// @port player.cpp|devilution::PosOkPlayer(const Player &player, Point position) sha=b1f5381195f4
pub fn pos_ok_player(ctx: &Ctx, pnum: usize, position: Point) -> bool {
    if !in_dungeon_bounds(position) {
        return false;
    }
    if !crate::engine::path::is_tile_walkable(ctx, position, false) {
        return false;
    }
    let (x, y) = (position.x as usize, position.y as usize);
    let dp = ctx.gendung.dPlayer[x][y];
    if dp != 0 {
        let other = (dp as i32).unsigned_abs() as usize - 1;
        if other != pnum && ctx.players.Players[other]._pHitPoints != 0 {
            return false;
        }
    }
    let dm = ctx.gendung.dMonster[x][y];
    if dm != 0 {
        if ctx.gendung.leveltype == DungeonType::Town {
            return false;
        }
        if dm <= 0 {
            return false;
        }
        if (ctx.monster.Monsters[(dm - 1) as usize].hitPoints >> 6) > 0 {
            return false;
        }
    }
    true
}

/// Original: `devilution::MakePlrPath` (player.cpp).
// @port player.cpp|devilution::MakePlrPath(Player &player, Point targetPosition, bool endspace) sha=5ebac1086aab
pub fn make_plr_path(ctx: &mut Ctx, pnum: usize, target_position: Point, endspace: bool) {
    let from = ctx.players.Players[pnum].position.future;
    if from == target_position {
        return;
    }
    let mut walkpath = ctx.players.Players[pnum].walkpath;
    let mut path = crate::engine::path::find_path(ctx, &|ctx, pos| pos_ok_player(ctx, pnum, pos), from, target_position, &mut walkpath);
    ctx.players.Players[pnum].walkpath = walkpath;
    if path == 0 {
        return;
    }
    if !endspace {
        path -= 1;
    }
    ctx.players.Players[pnum].walkpath[path as usize] = WALK_NONE as i8;
}

/// Original: `devilution::CalcPlrStaff` (player.cpp).
// @port player.cpp|devilution::CalcPlrStaff(Player &player) sha=945c92771d55
pub fn calc_plr_staff(ctx: &mut Ctx, pnum: usize) {
    let p = &mut ctx.players.Players[pnum];
    p._pISpells = 0;
    let l = &p.InvBody[INVLOC_HAND_LEFT as usize];
    if !l.is_empty() && l._iStatFlag && l._iCharges > 0 {
        p._pISpells |= crate::spells::get_spell_bitmask(l._iSpell);
    }
}

/// Original: `devilution::CheckPlrSpell` (player.cpp). Callers pass the readied spell for the
/// original's defaults.
// @port player.cpp|devilution::CheckPlrSpell(bool isShiftHeld, SpellID spellID, SpellType spellType) sha=0f5af219cf41
pub fn check_plr_spell(ctx: &mut Ctx, is_shift_held: bool, spell_id: SpellID, spell_type: SpellType) {
    let me = ctx.players.my_player_id();
    let mut addflag = false;
    if !crate::spells::is_valid_spell(ctx, spell_id) {
        player_say(ctx, me, HeroSpeech::IDontHaveASpellReady);
        return;
    }
    let mp = ctx.diablo.mouse_position;
    let mpos = Point::new(mp.0, mp.1);
    if ctx.controls.control_mode == crate::controls::ControlTypes::KeyboardAndMouse {
        if ctx.cursor.pcurs != crate::cursor::CURSOR_HAND {
            return;
        }
        if crate::control::get_main_panel(ctx).contains(mpos) {
            return;
        }
        if (crate::control::is_left_panel_open(ctx) && crate::control::get_left_panel(ctx).contains(mpos))
            || (crate::control::is_right_panel_open(ctx) && crate::control::get_right_panel(ctx).contains(mpos))
        {
            if !matches!(spell_id, SpellID::Healing | SpellID::Identify | SpellID::ItemRepair | SpellID::Infravision | SpellID::StaffRecharge) {
                return;
            }
        }
    }
    if ctx.gendung.leveltype == DungeonType::Town && !crate::items::get_spell_data(spell_id).flags.has_any_of(SpellDataFlags::AllowedInTown) {
        player_say(ctx, me, HeroSpeech::ICantCastThatHere);
        return;
    }
    let mut spellcheck = SpellCheckResult::Success;
    match spell_type {
        SpellType::Skill | SpellType::Spell => {
            spellcheck = crate::spells::check_spell(ctx, me, spell_id, spell_type, false);
            addflag = spellcheck == SpellCheckResult::Success;
        }
        SpellType::Scroll => addflag = ctx.cursor.pcurs == crate::cursor::CURSOR_HAND && crate::spells::can_use_scroll(ctx, me, spell_id),
        SpellType::Charges => addflag = ctx.cursor.pcurs == crate::cursor::CURSOR_HAND && crate::spells::can_use_staff(ctx, me, spell_id),
        SpellType::Invalid => return,
    }
    if !addflag {
        if spell_type == SpellType::Spell {
            match spellcheck {
                SpellCheckResult::Fail_NoMana => player_say(ctx, me, HeroSpeech::NotEnoughMana),
                SpellCheckResult::Fail_Level0 => player_say(ctx, me, HeroSpeech::ICantCastThatYet),
                _ => player_say(ctx, me, HeroSpeech::ICantDoThat),
            }
            ctx.diablo.last_mouse_button_action = crate::diablo::MouseActionType::None;
        }
        return;
    }
    let spell_level = ctx.players.Players[me].get_spell_level(spell_id);
    let spell_from = 0u16;
    let curs = ctx.cursor.cursPosition;
    let sid = spell_id as i8 as u8 as u16;
    let sty = spell_type as u8 as u16;
    use crate::diablo::MouseActionType as M;
    if crate::spells::is_wall_spell(spell_id) {
        ctx.diablo.last_mouse_button_action = M::Spell;
        let sd = crate::engine::get_direction(ctx.players.Players[me].position.tile, curs);
        crate::msg::net_send_cmd_loc_param5(ctx, true, CMD_SPELLXYD, curs, sid, sty, sd as u16, spell_level as u16, spell_from);
    } else if ctx.cursor.pcursmonst != -1 && !is_shift_held {
        ctx.diablo.last_mouse_button_action = M::SpellMonsterTarget;
        let m = ctx.cursor.pcursmonst as u16;
        crate::msg::net_send_cmd_param5(ctx, true, CMD_SPELLID, m, sid, sty, spell_level as u16, spell_from);
    } else if ctx.cursor.pcursplr != -1 && !is_shift_held && !ctx.players.Players[me].friendlyMode {
        ctx.diablo.last_mouse_button_action = M::SpellPlayerTarget;
        let pl = ctx.cursor.pcursplr as u16;
        crate::msg::net_send_cmd_param5(ctx, true, CMD_SPELLPID, pl, sid, sty, spell_level as u16, spell_from);
    } else {
        ctx.diablo.last_mouse_button_action = M::Spell;
        crate::msg::net_send_cmd_loc_param4(ctx, true, CMD_SPELLXY, curs, sid, sty, spell_level as u16, spell_from);
    }
}

/// Original: `devilution::SyncPlrAnim` (player.cpp).
// @port player.cpp|devilution::SyncPlrAnim(Player &player) sha=ff28c14cd4a3
pub fn sync_plr_anim(ctx: &mut Ctx, pnum: usize) {
    let graphic = ctx.players.Players[pnum].get_graphic();
    if !ctx.diablo.headless_mode {
        let p = &mut ctx.players.Players[pnum];
        p.AnimInfo.sprites = p.AnimationData[graphic as usize].sprites_for_direction(p._pdir);
    }
}

/// Original: `devilution::SyncInitPlrPos` (player.cpp).
// @port player.cpp|devilution::SyncInitPlrPos(Player &player) sha=bad063d4a5be
pub fn sync_init_plr_pos(ctx: &mut Ctx, pnum: usize) {
    if !is_on_active_level(ctx, pnum) {
        return;
    }
    const OFFSET: [Displacement; 9] = [
        Displacement::new(0, 0),
        Displacement::new(1, 0),
        Displacement::new(0, 1),
        Displacement::new(1, 1),
        Displacement::new(2, 0),
        Displacement::new(0, 2),
        Displacement::new(1, 2),
        Displacement::new(2, 1),
        Displacement::new(2, 2),
    ];
    let tile = ctx.players.Players[pnum].position.tile;
    let mut position = None;
    for off in OFFSET.iter().take(8) {
        let p = tile + *off;
        if pos_ok_player(ctx, pnum, p) {
            position = Some(p);
            break;
        }
    }
    let position = position.unwrap_or_else(|| {
        let cl = ctx.gendung.currlevel;
        crate::engine::path::find_closest_valid_position(
            ctx,
            &|ctx, test_position| {
                for i in 0..ctx.trigs.numtrigs as usize {
                    if ctx.trigs.trigs[i].position == test_position {
                        return false;
                    }
                }
                pos_ok_player(ctx, pnum, test_position) && !crate::portal::pos_ok_portal(ctx, cl as i32, test_position)
            },
            tile,
            1,
            50,
        )
        .unwrap_or(Point::new(0, 0))
    });
    ctx.players.Players[pnum].position.tile = position;
    ctx.gendung.dPlayer[position.x as usize][position.y as usize] = (pnum + 1) as i8;
    ctx.players.Players[pnum].position.future = position;
    if is_my(ctx, pnum) {
        ctx.gendung.ViewPosition = position;
    }
}

/// Original: `devilution::SyncInitPlr` (player.cpp).
// @port player.cpp|devilution::SyncInitPlr(Player &player) sha=9da84b536e6b
pub fn sync_init_plr(ctx: &mut Ctx, pnum: usize) {
    set_plr_anims(ctx, pnum);
    sync_init_plr_pos(ctx, pnum);
    if !is_my(ctx, pnum) {
        ctx.players.Players[pnum].lightId = NO_LIGHT;
    }
}

/// Original: `devilution::CheckStats` (player.cpp).
// @port player.cpp|devilution::CheckStats(Player &player) sha=3b9c70f74ccb
pub fn check_stats(p: &mut Player) {
    for attribute in ALL_ATTRIBUTES {
        let max = p.get_maximum_attribute_value(attribute);
        match attribute {
            CharacterAttribute::Strength => p._pBaseStr = p._pBaseStr.clamp(0, max),
            CharacterAttribute::Magic => p._pBaseMag = p._pBaseMag.clamp(0, max),
            CharacterAttribute::Dexterity => p._pBaseDex = p._pBaseDex.clamp(0, max),
            CharacterAttribute::Vitality => p._pBaseVit = p._pBaseVit.clamp(0, max),
        }
    }
}

/// Original: `devilution::ModifyPlrStr` (player.cpp).
// @port player.cpp|devilution::ModifyPlrStr(Player &player, int l) sha=6f7a8fe96113
pub fn modify_plr_str(ctx: &mut Ctx, pnum: usize, mut l: i32) {
    {
        let p = &mut ctx.players.Players[pnum];
        l = l.clamp(-p._pBaseStr, p.get_maximum_attribute_value(CharacterAttribute::Strength) - p._pBaseStr);
        p._pStrength += l;
        p._pBaseStr += l;
    }
    crate::items::calc_plr_inv(ctx, pnum, true);
    if is_my(ctx, pnum) {
        let v = ctx.players.Players[pnum]._pBaseStr;
        crate::msg::net_send_cmd_param1(ctx, false, CMD_SETSTR, v as u16);
    }
}

/// Original: `devilution::ModifyPlrMag` (player.cpp).
// @port player.cpp|devilution::ModifyPlrMag(Player &player, int l) sha=017cf6d65394
pub fn modify_plr_mag(ctx: &mut Ctx, pnum: usize, mut l: i32) {
    {
        let p = &mut ctx.players.Players[pnum];
        l = l.clamp(-p._pBaseMag, p.get_maximum_attribute_value(CharacterAttribute::Magic) - p._pBaseMag);
        p._pMagic += l;
        p._pBaseMag += l;
        let ms = l * PlayersData[p._pClass as usize].chrMana as i32;
        p._pMaxManaBase += ms;
        p._pMaxMana += ms;
        if p._pIFlags.has_none_of(ItemSpecialEffect::NoMana) {
            p._pManaBase += ms;
            p._pMana += ms;
        }
    }
    crate::items::calc_plr_inv(ctx, pnum, true);
    if is_my(ctx, pnum) {
        let v = ctx.players.Players[pnum]._pBaseMag;
        crate::msg::net_send_cmd_param1(ctx, false, CMD_SETMAG, v as u16);
    }
}

/// Original: `devilution::ModifyPlrDex` (player.cpp).
// @port player.cpp|devilution::ModifyPlrDex(Player &player, int l) sha=afbd873eb382
pub fn modify_plr_dex(ctx: &mut Ctx, pnum: usize, mut l: i32) {
    {
        let p = &mut ctx.players.Players[pnum];
        l = l.clamp(-p._pBaseDex, p.get_maximum_attribute_value(CharacterAttribute::Dexterity) - p._pBaseDex);
        p._pDexterity += l;
        p._pBaseDex += l;
    }
    crate::items::calc_plr_inv(ctx, pnum, true);
    if is_my(ctx, pnum) {
        let v = ctx.players.Players[pnum]._pBaseDex;
        crate::msg::net_send_cmd_param1(ctx, false, CMD_SETDEX, v as u16);
    }
}

/// Original: `devilution::ModifyPlrVit` (player.cpp).
// @port player.cpp|devilution::ModifyPlrVit(Player &player, int l) sha=6b903ea69167
pub fn modify_plr_vit(ctx: &mut Ctx, pnum: usize, mut l: i32) {
    {
        let p = &mut ctx.players.Players[pnum];
        l = l.clamp(-p._pBaseVit, p.get_maximum_attribute_value(CharacterAttribute::Vitality) - p._pBaseVit);
        p._pVitality += l;
        p._pBaseVit += l;
        let ms = l * PlayersData[p._pClass as usize].chrLife as i32;
        p._pHPBase += ms;
        p._pMaxHPBase += ms;
        p._pHitPoints += ms;
        p._pMaxHP += ms;
    }
    crate::items::calc_plr_inv(ctx, pnum, true);
    if is_my(ctx, pnum) {
        let v = ctx.players.Players[pnum]._pBaseVit;
        crate::msg::net_send_cmd_param1(ctx, false, CMD_SETVIT, v as u16);
    }
}

/// Original: `devilution::SetPlayerHitPoints` (player.cpp).
// @port player.cpp|devilution::SetPlayerHitPoints(Player &player, int val) sha=0073d8a208f6
pub fn set_player_hit_points(ctx: &mut Ctx, pnum: usize, val: i32) {
    let p = &mut ctx.players.Players[pnum];
    p._pHitPoints = val;
    p._pHPBase = val + p._pMaxHPBase - p._pMaxHP;
    if is_my(ctx, pnum) {
        redraw_component(ctx, PanelDrawComponent::Health);
    }
}

/// Original: `devilution::SetPlrStr` (player.cpp).
// @port player.cpp|devilution::SetPlrStr(Player &player, int v) sha=bba364edef4e
pub fn set_plr_str(ctx: &mut Ctx, pnum: usize, v: i32) {
    ctx.players.Players[pnum]._pBaseStr = v;
    crate::items::calc_plr_inv(ctx, pnum, true);
}

/// Original: `devilution::SetPlrMag` (player.cpp).
// @port player.cpp|devilution::SetPlrMag(Player &player, int v) sha=b762412b0723
pub fn set_plr_mag(ctx: &mut Ctx, pnum: usize, v: i32) {
    let p = &mut ctx.players.Players[pnum];
    p._pBaseMag = v;
    let m = v * PlayersData[p._pClass as usize].chrMana as i32;
    p._pMaxManaBase = m;
    p._pMaxMana = m;
    crate::items::calc_plr_inv(ctx, pnum, true);
}

/// Original: `devilution::SetPlrDex` (player.cpp).
// @port player.cpp|devilution::SetPlrDex(Player &player, int v) sha=47bad6d9d4d5
pub fn set_plr_dex(ctx: &mut Ctx, pnum: usize, v: i32) {
    ctx.players.Players[pnum]._pBaseDex = v;
    crate::items::calc_plr_inv(ctx, pnum, true);
}

/// Original: `devilution::SetPlrVit` (player.cpp).
// @port player.cpp|devilution::SetPlrVit(Player &player, int v) sha=a97dcb901515
pub fn set_plr_vit(ctx: &mut Ctx, pnum: usize, v: i32) {
    let p = &mut ctx.players.Players[pnum];
    p._pBaseVit = v;
    let hp = v * PlayersData[p._pClass as usize].chrLife as i32;
    p._pHPBase = hp;
    p._pMaxHPBase = hp;
    crate::items::calc_plr_inv(ctx, pnum, true);
}

/// Original: `devilution::InitDungMsgs` (player.cpp).
// @port player.cpp|devilution::InitDungMsgs(Player &player) sha=4598d49f793f
pub fn init_dung_msgs(ctx: &mut Ctx, pnum: usize) {
    let p = &mut ctx.players.Players[pnum];
    p.pDungMsgs = 0;
    p.pDungMsgs2 = 0;
}

const DUNG_MSG_CATHEDRAL: u8 = 1 << 0;
const DUNG_MSG_CATACOMBS: u8 = 1 << 1;
const DUNG_MSG_CAVES: u8 = 1 << 2;
const DUNG_MSG_HELL: u8 = 1 << 3;
const DUNG_MSG_DIABLO: u8 = 1 << 4;

/// Original: `devilution::PlayDungMsgs` (player.cpp).
// @port player.cpp|devilution::PlayDungMsgs() sha=33062e0fed77
pub fn play_dung_msgs(ctx: &mut Ctx) {
    use crate::effects_data::*;
    let me = ctx.players.my_player_id();
    let setlevel = ctx.gendung.setlevel;
    let cl = ctx.gendung.currlevel as usize;
    let (visited, msgs, msgs2) = {
        let p = &ctx.players.Players[me];
        (p._pLvlVisited[cl], p.pDungMsgs, p.pDungMsgs2)
    };
    if !setlevel && cl == 1 && !visited && msgs & DUNG_MSG_CATHEDRAL == 0 {
        player_say_delayed(ctx, me, HeroSpeech::TheSanctityOfThisPlaceHasBeenFouled, 40);
        ctx.players.Players[me].pDungMsgs |= DUNG_MSG_CATHEDRAL;
    } else if !setlevel && cl == 5 && !visited && msgs & DUNG_MSG_CATACOMBS == 0 {
        player_say_delayed(ctx, me, HeroSpeech::TheSmellOfDeathSurroundsMe, 40);
        ctx.players.Players[me].pDungMsgs |= DUNG_MSG_CATACOMBS;
    } else if !setlevel && cl == 9 && !visited && msgs & DUNG_MSG_CAVES == 0 {
        player_say_delayed(ctx, me, HeroSpeech::ItsHotDownHere, 40);
        ctx.players.Players[me].pDungMsgs |= DUNG_MSG_CAVES;
    } else if !setlevel && cl == 13 && !visited && msgs & DUNG_MSG_HELL == 0 {
        player_say_delayed(ctx, me, HeroSpeech::IMustBeGettingClose, 40);
        ctx.players.Players[me].pDungMsgs |= DUNG_MSG_HELL;
    } else if !setlevel && cl == 16 && !visited && msgs & DUNG_MSG_DIABLO == 0 {
        ctx.effects.sfxdelay = 40;
        ctx.effects.sfxdnum = PS_DIABLVLINT;
        ctx.players.Players[me].pDungMsgs |= DUNG_MSG_DIABLO;
    } else if !setlevel && cl == 17 && !visited && msgs2 & 1 == 0 {
        ctx.effects.sfxdelay = 10;
        ctx.effects.sfxdnum = USFX_DEFILER1;
        let q = &mut ctx.quests.Quests[Q_DEFILER as usize];
        q._qactive = QUEST_ACTIVE;
        q._qlog = true;
        q._qmsg = TEXT_DEFILER1;
        crate::msg::net_send_cmd_quest(ctx, true, Q_DEFILER as usize);
        ctx.players.Players[me].pDungMsgs2 |= 1;
    } else if !setlevel && cl == 19 && !visited && msgs2 & 4 == 0 {
        ctx.effects.sfxdelay = 10;
        ctx.effects.sfxdnum = USFX_DEFILER3;
        ctx.players.Players[me].pDungMsgs2 |= 4;
    } else if !setlevel && cl == 21 && !visited && msgs & 32 == 0 {
        player_say_delayed(ctx, me, HeroSpeech::ThisIsAPlaceOfGreatPower, 30);
        ctx.players.Players[me].pDungMsgs |= 32;
    } else if setlevel
        && ctx.gendung.setlvlnum == SL_SKELKING
        && !ctx.init.gb_is_spawn
        && !ctx.players.Players[me]._pSLvlVisited[SL_SKELKING as usize]
        && ctx.quests.Quests[Q_SKELKING as usize]._qactive == QUEST_ACTIVE
    {
        ctx.effects.sfxdelay = 10;
        ctx.effects.sfxdnum = USFX_SKING1;
    } else {
        ctx.effects.sfxdelay = 0;
    }
}

pub use crate::objects::redo_player_vision;
