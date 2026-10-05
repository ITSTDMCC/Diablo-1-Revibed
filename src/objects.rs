//! `Source/objects.cpp`: dungeon objects: placement, interaction and per-tick updates.

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::geometry::Point;
use crate::enums::*;
use crate::levels::gendung::in_dungeon_bounds;
use crate::utils::language::tr;

pub const MAXOBJECTS: usize = 127;

/// `Object`
#[derive(Clone, Debug)]
pub struct Object {
    pub _otype: _object_id,
    pub applyLighting: bool,
    pub _oTrapFlag: bool,
    pub _oDoorFlag: bool,
    pub position: Point,
    pub _oAnimFlag: u32,
    pub _oAnimData: Option<ClxSpriteList>,
    pub _oAnimDelay: i32,
    pub _oAnimCnt: i32,
    pub _oAnimLen: u32,
    pub _oAnimFrame: u32,
    pub _oAnimWidth: u16,
    pub _oDelFlag: bool,
    pub _oBreak: i8,
    pub _oSolidFlag: bool,
    pub _oMissFlag: bool,
    pub _oSelFlag: u8,
    pub _oPreFlag: bool,
    pub _olid: i32,
    pub _oRndSeed: u32,
    pub _oVar1: i32,
    pub _oVar2: i32,
    pub _oVar3: i32,
    pub _oVar4: i32,
    pub _oVar5: i32,
    pub _oVar6: u32,
    pub _oVar8: i32,
    pub bookMessage: _speech_id,
}

impl Default for Object {
    fn default() -> Self {
        Object {
            _otype: OBJ_NULL,
            applyLighting: false,
            _oTrapFlag: false,
            _oDoorFlag: false,
            position: Point::default(),
            _oAnimFlag: 0,
            _oAnimData: None,
            _oAnimDelay: 0,
            _oAnimCnt: 0,
            _oAnimLen: 0,
            _oAnimFrame: 0,
            _oAnimWidth: 0,
            _oDelFlag: false,
            _oBreak: 0,
            _oSolidFlag: false,
            _oMissFlag: false,
            _oSelFlag: 0,
            _oPreFlag: false,
            _olid: 0,
            _oRndSeed: 0,
            _oVar1: 0,
            _oVar2: 0,
            _oVar3: 0,
            _oVar4: 0,
            _oVar5: 0,
            _oVar6: 0,
            _oVar8: 0,
            bookMessage: TEXT_NONE,
        }
    }
}

impl Object {
    pub fn is_breakable(&self) -> bool {
        self._oBreak == 1
    }
    // @port objects.h|devilution::Object::IsBroken() sha=38740a1b2425
    pub fn is_broken(&self) -> bool {
        self._oBreak == -1
    }
    // @port objects.h|devilution::Object::IsBarrel() sha=ad278960c9b1
    pub fn is_barrel(&self) -> bool {
        matches!(self._otype, OBJ_BARREL | OBJ_BARRELEX | OBJ_POD | OBJ_PODEX | OBJ_URN | OBJ_URNEX)
    }
    // @port objects.h|devilution::Object::isExplosive() sha=3184a32043cb
    pub fn is_explosive(&self) -> bool {
        matches!(self._otype, OBJ_BARRELEX | OBJ_PODEX | OBJ_URNEX)
    }
    // @port objects.h|devilution::Object::IsChest() sha=2832eeb05261
    pub fn is_chest(&self) -> bool {
        matches!(self._otype, OBJ_CHEST1 | OBJ_CHEST2 | OBJ_CHEST3 | OBJ_TCHEST1 | OBJ_TCHEST2 | OBJ_TCHEST3)
    }
    // @port objects.h|devilution::Object::IsTrappedChest() sha=b166cc55b5f3
    pub fn is_trapped_chest(&self) -> bool {
        matches!(self._otype, OBJ_TCHEST1 | OBJ_TCHEST2 | OBJ_TCHEST3) && self._oTrapFlag
    }
    // @port objects.h|devilution::Object::IsUntrappedChest() sha=851c85e311d5
    pub fn is_untrapped_chest(&self) -> bool {
        matches!(self._otype, OBJ_CHEST1 | OBJ_CHEST2 | OBJ_CHEST3) && !self._oTrapFlag
    }
    // @port objects.h|devilution::Object::IsCrux() sha=0fc6549a73db
    pub fn is_crux(&self) -> bool {
        matches!(self._otype, OBJ_CRUX1 | OBJ_CRUX2 | OBJ_CRUX3)
    }
    // @port objects.h|devilution::Object::isDoor() sha=e12cd73efc2d
    pub fn is_door(&self) -> bool {
        matches!(self._otype, OBJ_L1LDOOR | OBJ_L1RDOOR | OBJ_L2LDOOR | OBJ_L2RDOOR | OBJ_L3LDOOR | OBJ_L3RDOOR | OBJ_L5LDOOR | OBJ_L5RDOOR)
    }
    // @port objects.h|devilution::Object::IsShrine() sha=55478a23da35
    pub fn is_shrine(&self) -> bool {
        matches!(self._otype, OBJ_SHRINEL | OBJ_SHRINER)
    }
    pub fn is_trap(&self) -> bool {
        matches!(self._otype, OBJ_TRAPL | OBJ_TRAPR)
    }
}

/// Globals of objects.cpp.
pub struct ObjectsState {
    pub Objects: Vec<Object>,
    pub AvailableObjects: [i32; MAXOBJECTS],
    pub ActiveObjects: [i32; MAXOBJECTS],
    pub ActiveObjectCount: i32,
    pub LoadingMapObjects: bool,
    /// `pObjCels`
    pub p_obj_cels: Vec<Option<ClxSpriteList>>,
    /// `numobjfiles`
    pub numobjfiles: i32,
    /// `ObjFileList`
    pub ObjFileList: [object_graphic_id; 40],
    /// `trapid`
    pub trapid: i32,
    /// `trapdir`
    pub trapdir: i32,
    /// `leverid`
    pub leverid: i32,
    /// `NaKrulTomeSequence`: progress through the tome sequence that spawns Na-Krul
    pub NaKrulTomeSequence: i32,
}

impl Default for ObjectsState {
    fn default() -> Self {
        ObjectsState {
            Objects: vec![Object::default(); MAXOBJECTS],
            AvailableObjects: [0; MAXOBJECTS],
            ActiveObjects: [0; MAXOBJECTS],
            ActiveObjectCount: 0,
            LoadingMapObjects: false,
            p_obj_cels: (0..40).map(|_| None).collect(),
            numobjfiles: 0,
            ObjFileList: [0; 40],
            trapid: 0,
            trapdir: 0,
            leverid: 0,
            NaKrulTomeSequence: 0,
        }
    }
}

/// Original: `devilution::FreeObjectGFX` (objects.cpp).
// @port objects.cpp|devilution::FreeObjectGFX() sha=39f3f2da4e30
pub fn free_object_gfx(ctx: &mut Ctx) {
    for i in 0..ctx.objects.numobjfiles as usize {
        ctx.objects.p_obj_cels[i] = None;
    }
    ctx.objects.numobjfiles = 0;
}

/// The `OBJ_STAND` object's position (the search loop of `SpawnRock` in items.cpp).
pub fn find_stand_position(ctx: &Ctx) -> Option<Point> {
    for i in 0..ctx.objects.ActiveObjectCount as usize {
        let object = &ctx.objects.Objects[ctx.objects.ActiveObjects[i] as usize];
        if object._otype == OBJ_STAND {
            return Some(object.position);
        }
    }
    None
}




impl Object {
    /// Original: `devilution::Object::IsDisabled` (objects.cpp).
    // @port objects.cpp|devilution::Object::IsDisabled() sha=7c4740d5b420
    pub fn is_disabled_opt(&self, disable_crippling_shrines: bool) -> bool {
        if !disable_crippling_shrines {
            return false;
        }
        if matches!(self._otype, OBJ_GOATSHRINE | OBJ_CAULDRON) {
            return true;
        }
        if !self.is_shrine() {
            return false;
        }
        matches!(self._oVar1, x if x == shrine_type::ShrineFascinating || x == shrine_type::ShrineOrnate || x == shrine_type::ShrineSacred)
    }
}

/// `shrine_type` (objects.cpp)
#[allow(non_upper_case_globals)]
pub mod shrine_type {
    pub const ShrineMysterious: i32 = 0;
    pub const ShrineHidden: i32 = 1;
    pub const ShrineGloomy: i32 = 2;
    pub const ShrineWeird: i32 = 3;
    pub const ShrineMagical: i32 = 4;
    pub const ShrineStone: i32 = 5;
    pub const ShrineReligious: i32 = 6;
    pub const ShrineEnchanted: i32 = 7;
    pub const ShrineThaumaturgic: i32 = 8;
    pub const ShrineFascinating: i32 = 9;
    pub const ShrineCryptic: i32 = 10;
    pub const ShrineMagicaL2: i32 = 11;
    pub const ShrineEldritch: i32 = 12;
    pub const ShrineEerie: i32 = 13;
    pub const ShrineDivine: i32 = 14;
    pub const ShrineHoly: i32 = 15;
    pub const ShrineSacred: i32 = 16;
    pub const ShrineSpiritual: i32 = 17;
    pub const ShrineSpooky: i32 = 18;
    pub const ShrineAbandoned: i32 = 19;
    pub const ShrineCreepy: i32 = 20;
    pub const ShrineQuiet: i32 = 21;
    pub const ShrineSecluded: i32 = 22;
    pub const ShrineOrnate: i32 = 23;
    pub const ShrineGlimmering: i32 = 24;
    pub const ShrineTainted: i32 = 25;
    pub const ShrineOily: i32 = 26;
    pub const ShrineGlowing: i32 = 27;
    pub const ShrineMendicant: i32 = 28;
    pub const ShrineSparkling: i32 = 29;
    pub const ShrineTown: i32 = 30;
    pub const ShrineShimmering: i32 = 31;
    pub const ShrineSolar: i32 = 32;
    pub const ShrineMurphys: i32 = 33;
    pub const NumberOfShrineTypes: i32 = 34;
}





/// Original: `devilution::FindObjectAtPosition` (objects.cpp): the index into `Objects` of the
/// object at `position`.
// @port objects.cpp|devilution::FindObjectAtPosition(Point position, bool considerLargeObjects) sha=6bea8d30b3b4
pub fn find_object_at_position(ctx: &Ctx, position: Point, consider_large_objects: bool) -> Option<usize> {
    if !crate::levels::gendung::in_dungeon_bounds(position) {
        return None;
    }
    let object_id = ctx.gendung.dObject[position.x as usize][position.y as usize];
    if object_id > 0 || (consider_large_objects && object_id != 0) {
        return Some((object_id as i32).unsigned_abs() as usize - 1);
    }
    // nothing at this position
    None
}

/// Original: `devilution::IsObjectAtPosition` (objects.h).
// @port objects.h|devilution::IsObjectAtPosition(Point position) sha=df5205688c00
pub fn is_object_at_position(ctx: &Ctx, position: Point) -> bool {
    find_object_at_position(ctx, position, true).is_some()
}

/// Original: `devilution::IsItemBlockingObjectAtPosition` (objects.cpp).
// @port objects.cpp|devilution::IsItemBlockingObjectAtPosition(Point position) sha=cb86707cdda7
pub fn is_item_blocking_object_at_position(ctx: &Ctx, position: Point) -> bool {
    use crate::engine::geometry::Direction;
    if let Some(o) = find_object_at_position(ctx, position, true) {
        if ctx.objects.Objects[o]._oSolidFlag {
            // solid object
            return true;
        }
    }
    if let Some(o) = find_object_at_position(ctx, position + Direction::South, true) {
        if ctx.objects.Objects[o]._oSelFlag != 0 {
            // An unopened container or breakable object exists which potentially overlaps this tile, the player might not be able to pick up an item dropped here.
            return true;
        }
    }
    if let Some(o) = find_object_at_position(ctx, position + Direction::SouthEast, false) {
        if let Some(other_door) = find_object_at_position(ctx, position + Direction::SouthWest, false) {
            if ctx.objects.Objects[o]._oSelFlag != 0 && ctx.objects.Objects[other_door]._oSelFlag != 0 {
                // Two interactive objects potentially overlap both sides of this tile, as above the player might not be able to pick up an item which is dropped here.
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------------------------
// objects.cpp, part 1: placement helpers, level object setup, doors.
// ---------------------------------------------------------------------------------------------

use crate::engine::geometry::{Direction, Displacement, Rectangle, Size};
use crate::engine::path::is_tile_not_solid;
use crate::levels::gendung::{tile_contains_set_piece, tile_has_any, DungeonType, MAXDUNX, MAXDUNY, DMAXX, DMAXY};
use crate::tables::objdat::{AllObjects, ObjectData, ObjTypeConv};

const DOOR_CLOSED: i32 = 0;
const DOOR_OPEN: i32 = 1;
const DOOR_BLOCKED: i32 = 2;

/// `bxadd`: the X-coordinate delta between barrels.
const BXADD: [i32; 8] = [-1, 0, 1, -1, 1, -1, 0, 1];
/// `byadd`: the Y-coordinate delta between barrels.
const BYADD: [i32; 8] = [-1, -1, -1, 0, 0, 1, 1, 1];

/// `ShrineNames`: maps from shrine_id to shrine name.
const SHRINE_NAMES: [&str; 34] = [
    "Mysterious", "Hidden", "Gloomy", "Weird", "Magical", "Stone", "Religious", "Enchanted", "Thaumaturgic", "Fascinating", "Cryptic", "Magical", "Eldritch", "Eerie", "Divine", "Holy", "Sacred", "Spiritual",
    "Spooky", "Abandoned", "Creepy", "Quiet", "Secluded", "Ornate", "Glimmering", "Tainted", "Oily", "Glowing", "Mendicant's", "Sparkling", "Town", "Shimmering", "Solar", "Murphy's",
];

const MAX_LVLS: i32 = 24;

/// `shrinemin`: the minimum dungeon level on which each shrine will appear.
const SHRINEMIN: [i32; 34] = [1; 34];

/// `shrinemax`: the maximum dungeon level on which each shrine will appear.
const SHRINEMAX: [i32; 34] = {
    let mut a = [MAX_LVLS; 34];
    a[shrine_type::ShrineEnchanted as usize] = 8;
    a
};

// `shrine_gametype`
const SHRINE_TYPE_ANY: u8 = 0;
const SHRINE_TYPE_SINGLE: u8 = 1;
const SHRINE_TYPE_MULTI: u8 = 2;

/// `shrineavail`: the game type for which each shrine may appear.
const SHRINEAVAIL: [u8; 34] = [
    SHRINE_TYPE_ANY,    // Mysterious
    SHRINE_TYPE_ANY,    // Hidden
    SHRINE_TYPE_SINGLE, // Gloomy
    SHRINE_TYPE_SINGLE, // Weird
    SHRINE_TYPE_ANY,    // Magical
    SHRINE_TYPE_ANY,    // Stone
    SHRINE_TYPE_ANY,    // Religious
    SHRINE_TYPE_ANY,    // Enchanted
    SHRINE_TYPE_SINGLE, // Thaumaturgic
    SHRINE_TYPE_ANY,    // Fascinating
    SHRINE_TYPE_ANY,    // Cryptic
    SHRINE_TYPE_ANY,    // Magical
    SHRINE_TYPE_ANY,    // Eldritch
    SHRINE_TYPE_ANY,    // Eerie
    SHRINE_TYPE_ANY,    // Divine
    SHRINE_TYPE_ANY,    // Holy
    SHRINE_TYPE_ANY,    // Sacred
    SHRINE_TYPE_ANY,    // Spiritual
    SHRINE_TYPE_MULTI,  // Spooky
    SHRINE_TYPE_ANY,    // Abandoned
    SHRINE_TYPE_ANY,    // Creepy
    SHRINE_TYPE_ANY,    // Quiet
    SHRINE_TYPE_ANY,    // Secluded
    SHRINE_TYPE_ANY,    // Ornate
    SHRINE_TYPE_ANY,    // Glimmering
    SHRINE_TYPE_MULTI,  // Tainted
    SHRINE_TYPE_ANY,    // Oily
    SHRINE_TYPE_ANY,    // Glowing
    SHRINE_TYPE_ANY,    // Mendicant's
    SHRINE_TYPE_ANY,    // Sparkling
    SHRINE_TYPE_ANY,    // Town
    SHRINE_TYPE_ANY,    // Shimmering
    SHRINE_TYPE_SINGLE, // Solar
    SHRINE_TYPE_ANY,    // Murphy's
];

/// `StoryBookName`: maps from book_id to book name.
const STORY_BOOK_NAME: [&str; 16] = [
    "The Great Conflict",
    "The Wages of Sin are War",
    "The Tale of the Horadrim",
    "The Dark Exile",
    "The Sin War",
    "The Binding of the Three",
    "The Realms Beyond",
    "Tale of the Three",
    "The Black King",
    "Journal: The Ensorcellment",
    "Journal: The Meeting",
    "Journal: The Tirade",
    "Journal: His Power Grows",
    "Journal: NA-KRUL",
    "Journal: The End",
    "A Spellbook",
];

/// `StoryText`: speech IDs of each dungeon type narrator book, for each player class.
const STORY_TEXT: [[_speech_id; 3]; 3] = [[TEXT_BOOK11, TEXT_BOOK12, TEXT_BOOK13], [TEXT_BOOK21, TEXT_BOOK22, TEXT_BOOK23], [TEXT_BOOK31, TEXT_BOOK32, TEXT_BOOK33]];

impl ObjectData {
    /// `isAnimated`
    // @port objdat.h|devilution::ObjectData::isAnimated() sha=6c50a0bfa8f5
    pub fn is_animated(&self) -> bool {
        self.flags.has_any_of(ObjectDataFlags::Animated)
    }
    /// `isSolid`
    // @port objdat.h|devilution::ObjectData::isSolid() sha=a335f538df98
    pub fn is_solid(&self) -> bool {
        self.flags.has_any_of(ObjectDataFlags::Solid)
    }
    /// `missilesPassThrough`
    // @port objdat.h|devilution::ObjectData::missilesPassThrough() sha=05b221e518ef
    pub fn missiles_pass_through(&self) -> bool {
        self.flags.has_any_of(ObjectDataFlags::MissilesPassThrough)
    }
    /// `applyLighting`
    // @port objdat.h|devilution::ObjectData::applyLighting() sha=a5c27ebf10a9
    pub fn apply_lighting(&self) -> bool {
        self.flags.has_any_of(ObjectDataFlags::Light)
    }
    /// `isTrap`
    // @port objdat.h|devilution::ObjectData::isTrap() sha=1956128f8498
    pub fn is_trap(&self) -> bool {
        self.flags.has_any_of(ObjectDataFlags::Trap)
    }
    /// `isBreakable`
    // @port objdat.h|devilution::ObjectData::isBreakable() sha=5986c2e694d9
    pub fn is_breakable(&self) -> bool {
        self.flags.has_any_of(ObjectDataFlags::Breakable)
    }
}

impl Object {
    /// `SetMapRange(topLeftPosition, bottomRightPosition)`
    // @port objects.h|devilution::Object::SetMapRange(WorldTilePosition topLeftPosition, WorldTilePosition bottomRightPosition) sha=89faf45fea65
    pub fn set_map_range(&mut self, top_left: Point, bottom_right: Point) {
        self._oVar1 = top_left.x;
        self._oVar2 = top_left.y;
        self._oVar3 = bottom_right.x;
        self._oVar4 = bottom_right.y;
    }

    /// `SetMapRange(WorldTileRectangle)`
    // @port objects.h|devilution::Object::SetMapRange(WorldTileRectangle mapRange) sha=84af289bcdb8
    pub fn set_map_range_rect(&mut self, map_range: Rectangle) {
        let p = map_range.position;
        // WorldTilePosition + DisplacementOf<uint8_t>: uint8_t arithmetic
        let br = Point::new((p.x + map_range.size.width) & 0xff, (p.y + map_range.size.height) & 0xff);
        self.set_map_range(p, br);
    }

    /// `InitializeBook`
    // @port objects.h|devilution::Object::InitializeBook(WorldTileRectangle mapRange) sha=da022d750181
    pub fn initialize_book(&mut self, map_range: Rectangle) {
        self.set_map_range_rect(map_range);
        self._oVar6 = self._oAnimFrame + 1; // Save the frame number for the open book frame
    }

    /// `InitializeQuestBook`
    // @port objects.h|devilution::Object::InitializeQuestBook(WorldTileRectangle mapRange, int leverID, _speech_id message) sha=9c8391a30793
    pub fn initialize_quest_book(&mut self, map_range: Rectangle, lever_id: i32, message: _speech_id) {
        self.initialize_book(map_range);
        self._oVar8 = lever_id;
        self.bookMessage = message;
    }

    /// `InitializeLoadedObject`
    // @port objects.h|devilution::Object::InitializeLoadedObject(WorldTileRectangle mapRange, int leverID) sha=8f6de8eac3c0
    pub fn initialize_loaded_object(&mut self, map_range: Rectangle, lever_id: i32) {
        self.set_map_range_rect(map_range);
        self._oVar8 = lever_id;
    }
}

fn rnd(ctx: &mut Ctx, v: i32) -> i32 {
    ctx.rng.generate_rnd(v)
}

fn piece(ctx: &Ctx, x: i32, y: i32) -> i32 {
    ctx.gendung.dPiece[x as usize][y as usize] as i32
}

/// Original: `RndLocOk` (objects.cpp).
// @port objects.cpp|devilution::RndLocOk(int xp, int yp) sha=bc1bc53b5e7a
fn rnd_loc_ok(ctx: &Ctx, xp: i32, yp: i32) -> bool {
    let (x, y) = (xp as usize, yp as usize);
    if ctx.gendung.dMonster[x][y] != 0 {
        return false;
    }
    if ctx.gendung.dPlayer[x][y] != 0 {
        return false;
    }
    if is_object_at_position(ctx, Point::new(xp, yp)) {
        return false;
    }
    if tile_contains_set_piece(ctx, Point::new(xp, yp)) {
        return false;
    }
    if tile_has_any(ctx, piece(ctx, xp, yp), TileProperties::Solid) {
        return false;
    }
    !matches!(ctx.gendung.leveltype, DungeonType::Cathedral | DungeonType::Crypt) || piece(ctx, xp, yp) <= 125 || piece(ctx, xp, yp) >= 143
}

/// Original: `CanPlaceWallTrap` (objects.cpp).
// @port objects.cpp|devilution::CanPlaceWallTrap(int xp, int yp) sha=585b02b5cf3a
fn can_place_wall_trap(ctx: &Ctx, xp: i32, yp: i32) -> bool {
    // AddObjTraps can walk off the map edge (xp or yp == -1) when a trigger sits in open dirt;
    // the original then reads `dObject[x][-1]`, i.e. the flat array element before. Reproduce
    // that, and read zero for anything outside the whole array.
    let flat = xp * MAXDUNY as i32 + yp;
    let in_array = (0..(MAXDUNX * MAXDUNY) as i32).contains(&flat);
    let (fx, fy) = ((flat.max(0) / MAXDUNY as i32) as usize, (flat.max(0) % MAXDUNY as i32) as usize);
    let object = if in_array { ctx.gendung.dObject[fx][fy] } else { 0 };
    if object != 0 {
        return false;
    }
    if tile_contains_set_piece(ctx, Point::new(xp, yp)) {
        return false;
    }
    let pn = if in_array { ctx.gendung.dPiece[fx][fy] as i32 } else { 0 };
    tile_has_any(ctx, pn, TileProperties::Trap)
}

fn rnd_loc_ok_3x3(ctx: &Ctx, xp: i32, yp: i32) -> bool {
    rnd_loc_ok(ctx, xp - 1, yp - 1)
        && rnd_loc_ok(ctx, xp, yp - 1)
        && rnd_loc_ok(ctx, xp + 1, yp - 1)
        && rnd_loc_ok(ctx, xp - 1, yp)
        && rnd_loc_ok(ctx, xp, yp)
        && rnd_loc_ok(ctx, xp + 1, yp)
        && rnd_loc_ok(ctx, xp - 1, yp + 1)
        && rnd_loc_ok(ctx, xp, yp + 1)
        && rnd_loc_ok(ctx, xp + 1, yp + 1)
}

/// Original: `InitRndLocObj` (objects.cpp).
// @port objects.cpp|devilution::InitRndLocObj(int min, int max, _object_id objtype) sha=aa176d7014ba
fn init_rnd_loc_obj(ctx: &mut Ctx, min: i32, max: i32, objtype: _object_id) {
    let numobjs = rnd(ctx, max - min) + min;
    for _ in 0..numobjs {
        loop {
            let xp = rnd(ctx, 80) + 16;
            let yp = rnd(ctx, 80) + 16;
            if rnd_loc_ok_3x3(ctx, xp, yp) {
                add_object(ctx, objtype, Point::new(xp, yp));
                break;
            }
        }
    }
}

/// Original: `InitRndLocBigObj` (objects.cpp).
// @port objects.cpp|devilution::InitRndLocBigObj(int min, int max, _object_id objtype) sha=05e63cf5ac25
fn init_rnd_loc_big_obj(ctx: &mut Ctx, min: i32, max: i32, objtype: _object_id) {
    let numobjs = rnd(ctx, max - min) + min;
    for _ in 0..numobjs {
        loop {
            let xp = rnd(ctx, 80) + 16;
            let yp = rnd(ctx, 80) + 16;
            if rnd_loc_ok(ctx, xp - 1, yp - 2) && rnd_loc_ok(ctx, xp, yp - 2) && rnd_loc_ok(ctx, xp + 1, yp - 2) && rnd_loc_ok_3x3(ctx, xp, yp) {
                add_object(ctx, objtype, Point::new(xp, yp));
                break;
            }
        }
    }
}

/// Original: `CanPlaceRandomObject` (objects.cpp).
// @port objects.cpp|devilution::CanPlaceRandomObject(Point position, Displacement standoff) sha=8b0d05db2180
fn can_place_random_object(ctx: &Ctx, position: Point, standoff: Displacement) -> bool {
    for yy in -standoff.delta_y..=standoff.delta_y {
        for xx in -standoff.delta_x..=standoff.delta_x {
            let tile = position + Displacement::new(xx, yy);
            if !rnd_loc_ok(ctx, tile.x, tile.y) {
                return false;
            }
        }
    }
    true
}

/// Original: `GetRandomObjectPosition` (objects.cpp).
// @port objects.cpp|devilution::GetRandomObjectPosition(Displacement standoff) sha=35aff7c4fee2
fn get_random_object_position(ctx: &mut Ctx, standoff: Displacement) -> Option<Point> {
    for _ in 0..=20000 {
        let x = rnd(ctx, 80);
        let y = rnd(ctx, 80);
        let position = Point::new(x, y) + Displacement::new(16, 16);
        if can_place_random_object(ctx, position, standoff) {
            return Some(position);
        }
    }
    None
}

/// Original: `InitRndLocObj5x5` (objects.cpp).
// @port objects.cpp|devilution::InitRndLocObj5x5(int min, int max, _object_id objtype) sha=c1dd5a25170f
fn init_rnd_loc_obj5x5(ctx: &mut Ctx, min: i32, max: i32, objtype: _object_id) {
    let numobjs = min + rnd(ctx, max - min);
    for _ in 0..numobjs {
        let Some(position) = get_random_object_position(ctx, Displacement::new(2, 2)) else {
            return;
        };
        add_object(ctx, objtype, position);
    }
}

/// Original: `ClrAllObjects` (objects.cpp).
// @port objects.cpp|devilution::ClrAllObjects() sha=582f78e5bb25
fn clr_all_objects(ctx: &mut Ctx) {
    let s = &mut ctx.objects;
    for object in s.Objects.iter_mut() {
        *object = Object::default();
    }
    s.ActiveObjectCount = 0;
    for i in 0..MAXOBJECTS {
        s.AvailableObjects[i] = i as i32;
    }
    s.ActiveObjects = [0; MAXOBJECTS];
    s.trapdir = 0;
    s.trapid = 1;
    s.leverid = 1;
}

/// Original: `AddTortures` (objects.cpp).
// @port objects.cpp|devilution::AddTortures() sha=7d6a04ecd745
fn add_tortures(ctx: &mut Ctx) {
    for oy in 0..MAXDUNY as i32 {
        for ox in 0..MAXDUNX as i32 {
            if piece(ctx, ox, oy) == 366 {
                add_object(ctx, OBJ_TORTURE1, Point::new(ox, oy + 1));
                add_object(ctx, OBJ_TORTURE3, Point::new(ox + 2, oy - 1));
                add_object(ctx, OBJ_TORTURE2, Point::new(ox, oy + 3));
                add_object(ctx, OBJ_TORTURE4, Point::new(ox + 4, oy - 1));
                add_object(ctx, OBJ_TORTURE5, Point::new(ox, oy + 5));
                add_object(ctx, OBJ_TNUDEM1, Point::new(ox + 1, oy + 3));
                add_object(ctx, OBJ_TNUDEM2, Point::new(ox + 4, oy + 5));
                add_object(ctx, OBJ_TNUDEM3, Point::new(ox + 2, oy));
                add_object(ctx, OBJ_TNUDEM4, Point::new(ox + 3, oy + 2));
                add_object(ctx, OBJ_TNUDEW1, Point::new(ox + 2, oy + 4));
                add_object(ctx, OBJ_TNUDEW2, Point::new(ox + 2, oy + 1));
                add_object(ctx, OBJ_TNUDEW3, Point::new(ox + 4, oy + 2));
            }
        }
    }
}

/// Original: `AddCandles` (objects.cpp).
// @port objects.cpp|devilution::AddCandles() sha=48343a43b772
fn add_candles(ctx: &mut Ctx) {
    let p = ctx.quests.Quests[Q_PWATER as usize].position;
    let (tx, ty) = (p.x, p.y);
    add_object(ctx, OBJ_STORYCANDLE, Point::new(tx - 2, ty + 1));
    add_object(ctx, OBJ_STORYCANDLE, Point::new(tx + 3, ty + 1));
    add_object(ctx, OBJ_STORYCANDLE, Point::new(tx - 1, ty + 2));
    add_object(ctx, OBJ_STORYCANDLE, Point::new(tx + 2, ty + 2));
}

/// Original: `AddBookLever` (objects.cpp): spawns a book which changes a region of the map when activated.
// @port objects.cpp|devilution::AddBookLever(_object_id type, WorldTileRectangle affectedArea, _speech_id msg) sha=265d32e61967
fn add_book_lever(ctx: &mut Ctx, type_: _object_id, affected_area: Rectangle, msg: _speech_id) {
    let Some(mut position) = get_random_object_position(ctx, Displacement::new(2, 2)) else {
        return;
    };
    if type_ == OBJ_BLOODBOOK {
        position = ctx.gendung.SetPiece.position.mega_to_world() + Displacement::new(9, 24);
    }
    let lever = add_object(ctx, type_, position).expect("lever");
    let leverid = ctx.objects.leverid;
    ctx.objects.Objects[lever].initialize_quest_book(affected_area, leverid, msg);
    ctx.objects.leverid += 1;
}

/// Original: `InitRndBarrels` (objects.cpp).
// @port objects.cpp|devilution::InitRndBarrels() sha=f9b037c859f2
fn init_rnd_barrels(ctx: &mut Ctx) {
    let (mut barrel_id, mut explosive_barrel_id) = (OBJ_BARREL, OBJ_BARRELEX);
    if ctx.gendung.leveltype == DungeonType::Nest {
        barrel_id = OBJ_POD;
        explosive_barrel_id = OBJ_PODEX;
    } else if ctx.gendung.leveltype == DungeonType::Crypt {
        barrel_id = OBJ_URN;
        explosive_barrel_id = OBJ_URNEX;
    }
    // number of groups of barrels to generate
    let numobjs = rnd(ctx, 5) + 3;
    for _ in 0..numobjs {
        let (mut xp, mut yp);
        loop {
            xp = rnd(ctx, 80) + 16;
            yp = rnd(ctx, 80) + 16;
            if rnd_loc_ok(ctx, xp, yp) {
                break;
            }
        }
        let mut o = if ctx.rng.flip_coin(4) { explosive_barrel_id } else { barrel_id };
        add_object(ctx, o, Point::new(xp, yp));
        let mut found = true;
        // regulates chance to stop placing barrels in current group
        let mut p = 0;
        // number of barrels in current group
        let mut c = 1;
        while ctx.rng.flip_coin(p as u32) && found {
            // number of tries of placing next barrel in current group
            let mut t = 0;
            found = false;
            loop {
                if t >= 3 {
                    break;
                }
                let dir = rnd(ctx, 8) as usize;
                xp += BXADD[dir];
                yp += BYADD[dir];
                found = rnd_loc_ok(ctx, xp, yp);
                t += 1;
                if found {
                    break;
                }
            }
            if found {
                o = if ctx.rng.flip_coin(5) { explosive_barrel_id } else { barrel_id };
                add_object(ctx, o, Point::new(xp, yp));
                c += 1;
            }
            p = c / 2;
        }
    }
}

/// Original: `AddL2Torches` (objects.cpp).
// @port objects.cpp|devilution::AddL2Torches() sha=3e4b1b2a1e29
fn add_l2_torches(ctx: &mut Ctx) {
    for j in 0..MAXDUNY as i32 {
        for i in 0..MAXDUNX as i32 {
            let test_position = Point::new(i, j);
            if tile_contains_set_piece(ctx, test_position) {
                continue;
            }
            let pn = piece(ctx, i, j);
            if pn == 0 && ctx.rng.flip_coin(3) {
                add_object(ctx, OBJ_TORCHL2, test_position);
            }
            if pn == 4 && ctx.rng.flip_coin(3) {
                add_object(ctx, OBJ_TORCHR2, test_position);
            }
            if pn == 36 && ctx.rng.flip_coin(10) && !is_object_at_position(ctx, test_position + Direction::NorthWest) {
                add_object(ctx, OBJ_TORCHL, test_position + Direction::NorthWest);
            }
            if pn == 40 && ctx.rng.flip_coin(10) && !is_object_at_position(ctx, test_position + Direction::NorthEast) {
                add_object(ctx, OBJ_TORCHR, test_position + Direction::NorthEast);
            }
        }
    }
}

/// Original: `AddObjTraps` (objects.cpp).
// @port objects.cpp|devilution::AddObjTraps() sha=edb38480fa2d
fn add_obj_traps(ctx: &mut Ctx) {
    let currlevel = ctx.gendung.currlevel;
    // uninitialised in the original if currlevel == 0 (never called in town)
    let mut rndv = 0;
    if currlevel == 1 {
        rndv = 10;
    }
    if currlevel >= 2 {
        rndv = 15;
    }
    if currlevel >= 5 {
        rndv = 20;
    }
    if currlevel >= 7 {
        rndv = 25;
    }
    for j in 0..MAXDUNY as i32 {
        for i in 0..MAXDUNX as i32 {
            let Some(trigger_object) = find_object_at_position(ctx, Point::new(i, j), false) else {
                continue;
            };
            if rnd(ctx, 100) >= rndv {
                continue;
            }
            if !AllObjects[ctx.objects.Objects[trigger_object]._otype as usize].is_trap() {
                continue;
            }
            let trap_object;
            if ctx.rng.flip_coin(2) {
                let mut xp = i - 1;
                while is_tile_not_solid(ctx, Point::new(xp, j)) {
                    xp -= 1;
                }
                if !can_place_wall_trap(ctx, xp, j) || i - xp <= 1 {
                    continue;
                }
                trap_object = add_object(ctx, OBJ_TRAPL, Point::new(xp, j));
            } else {
                let mut yp = j - 1;
                while is_tile_not_solid(ctx, Point::new(i, yp)) {
                    yp -= 1;
                }
                if !can_place_wall_trap(ctx, i, yp) || j - yp <= 1 {
                    continue;
                }
                trap_object = add_object(ctx, OBJ_TRAPR, Point::new(i, yp));
            }
            if let Some(t) = trap_object {
                // nullptr check just in case we fail to find a valid location to place a trap in the chosen direction
                ctx.objects.Objects[t]._oVar1 = i;
                ctx.objects.Objects[t]._oVar2 = j;
                ctx.objects.Objects[trigger_object]._oTrapFlag = true;
            }
        }
    }
}

/// Original: `AddChestTraps` (objects.cpp).
// @port objects.cpp|devilution::AddChestTraps() sha=f6edfa133f38
fn add_chest_traps(ctx: &mut Ctx) {
    for j in 0..MAXDUNY as i32 {
        for i in 0..MAXDUNX as i32 {
            let Some(c) = find_object_at_position(ctx, Point::new(i, j), false) else {
                continue;
            };
            if ctx.objects.Objects[c].is_untrapped_chest() && rnd(ctx, 100) < 10 {
                let chest = &mut ctx.objects.Objects[c];
                match chest._otype {
                    OBJ_CHEST1 => chest._otype = OBJ_TCHEST1,
                    OBJ_CHEST2 => chest._otype = OBJ_TCHEST2,
                    OBJ_CHEST3 => chest._otype = OBJ_TCHEST3,
                    _ => {}
                }
                chest._oTrapFlag = true;
                let v4 = if ctx.gendung.leveltype == DungeonType::Catacombs { rnd(ctx, 2) } else { rnd(ctx, if ctx.init.gb_is_hellfire { 6 } else { 3 }) };
                ctx.objects.Objects[c]._oVar4 = v4;
            }
        }
    }
}

/// Original: `LoadMapObjects` (objects.cpp). Defaults: `mapRange = {}`, `leveridx = 0`.
// @port objects.cpp|devilution::LoadMapObjects(const char *path, Point start, WorldTileRectangle mapRange = {}, int leveridx = 0) sha=b7da02d8fddf
fn load_map_objects(ctx: &mut Ctx, path: &str, start: Point, map_range: Rectangle, leveridx: i32) {
    ctx.objects.LoadingMapObjects = true;
    let dun_data = crate::levels::gendung::load_u16_file(ctx, path);
    let mut width = dun_data[0] as usize;
    let mut height = dun_data[1] as usize;
    let layer2_offset = 2 + width * height;
    // The rest of the layers are at dPiece scale
    width *= 2;
    height *= 2;
    let object_layer = &dun_data[layer2_offset + width * height * 2..];
    for j in 0..height {
        for i in 0..width {
            let object_id = object_layer[j * width + i] as u8;
            if object_id != 0 {
                let map_pos = start + Displacement::new(i as i32, j as i32);
                let map_object = add_object(ctx, ObjTypeConv[object_id as usize], map_pos);
                if leveridx > 0 {
                    if let Some(o) = map_object {
                        ctx.objects.Objects[o].initialize_loaded_object(map_range, leveridx);
                    }
                }
            }
        }
    }
    ctx.objects.LoadingMapObjects = false;
}

/// Original: `AddDiabObjs` (objects.cpp).
// @port objects.cpp|devilution::AddDiabObjs() sha=e19dc71121ad
fn add_diab_objs(ctx: &mut Ctx) {
    let q = ctx.drlg_l4.diablo_quads;
    load_map_objects(ctx, "levels\\l4data\\diab1.dun", q[0].mega_to_world(), Rectangle::new(q[1], Size::new(11, 12)), 1);
    load_map_objects(ctx, "levels\\l4data\\diab2a.dun", q[1].mega_to_world(), Rectangle::new(q[2], Size::new(11, 11)), 2);
    load_map_objects(ctx, "levels\\l4data\\diab3a.dun", q[2].mega_to_world(), Rectangle::new(q[3], Size::new(9, 9)), 3);
}

/// Original: `AddCryptObject` (objects.cpp).
// @port objects.cpp|devilution::AddCryptObject(Object &object, int a2) sha=7c39ce0515a3
fn add_crypt_object(ctx: &mut Ctx, oi: usize, a2: i32) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let class = ctx.players.Players[me]._pClass;
    let object = &mut ctx.objects.Objects[oi];
    if a2 > 5 {
        let pick = |w: _speech_id, r: _speech_id, s: _speech_id, m: _speech_id, b: _speech_id| match class {
            HeroClass::Warrior | HeroClass::Barbarian => Some(w),
            HeroClass::Rogue => Some(r),
            HeroClass::Sorcerer => Some(s),
            HeroClass::Monk => Some(m),
            HeroClass::Bard => Some(b),
        };
        let v = match a2 {
            6 => pick(TEXT_BOOKA, TEXT_RBOOKA, TEXT_MBOOKA, TEXT_OBOOKA, TEXT_BBOOKA),
            7 => pick(TEXT_BOOKB, TEXT_RBOOKB, TEXT_MBOOKB, TEXT_OBOOKB, TEXT_BBOOKB),
            8 => pick(TEXT_BOOKC, TEXT_RBOOKC, TEXT_MBOOKC, TEXT_OBOOKC, TEXT_BBOOKC),
            _ => None,
        };
        if let Some(v) = v {
            object._oVar2 = v as i32;
        }
        object._oVar3 = 15;
        object._oVar8 = a2;
    } else {
        object._oVar2 = a2 + TEXT_SKLJRN as i32;
        object._oVar3 = a2 + 9;
        object._oVar8 = 0;
    }
    object._oVar1 = 1;
    object._oAnimFrame = (5 - 2 * object._oVar1) as u32;
    object._oVar4 = object._oAnimFrame as i32 + 1;
}

/// Original: `SetupObject` (objects.cpp).
// @port objects.cpp|devilution::SetupObject(Object &object, Point position, _object_id ot) sha=9e4afa125629
fn setup_object(ctx: &mut Ctx, oi: usize, position: Point, ot: _object_id) {
    let object_data = &AllObjects[ot as usize];
    ctx.objects.Objects[oi]._otype = ot;
    let ofi = object_data.ofindex;
    ctx.objects.Objects[oi].position = position;
    if !ctx.diablo.headless_mode {
        let found = ctx.objects.ObjFileList.iter().position(|&f| f == ofi);
        let Some(j) = found else {
            crate::platform::log::error!("Unable to find object_graphic_id {} in list of objects to load, level generation error.", ofi);
            return;
        };
        ctx.objects.Objects[oi]._oAnimData = ctx.objects.p_obj_cels[j].clone();
    }
    let anim_flag = object_data.is_animated();
    ctx.objects.Objects[oi]._oAnimFlag = anim_flag as u32;
    if anim_flag {
        let delay = object_data.animDelay as i32;
        let cnt = rnd(ctx, delay);
        let len = object_data.animLen as u32;
        let frame = rnd(ctx, len as i32 - 1) + 1;
        let object = &mut ctx.objects.Objects[oi];
        object._oAnimDelay = delay;
        object._oAnimCnt = cnt;
        object._oAnimLen = len;
        object._oAnimFrame = frame as u32;
    } else {
        let object = &mut ctx.objects.Objects[oi];
        object._oAnimDelay = 1000;
        object._oAnimCnt = 0;
        object._oAnimLen = object_data.animLen as u32;
        object._oAnimFrame = object_data.animDelay as u32;
    }
    let object = &mut ctx.objects.Objects[oi];
    object._oAnimWidth = object_data.animWidth as u16;
    object._oSolidFlag = object_data.is_solid();
    object._oMissFlag = object_data.missiles_pass_through();
    object.applyLighting = object_data.apply_lighting();
    object._oDelFlag = false;
    object._oBreak = object_data.is_breakable() as i8;
    object._oSelFlag = object_data.selFlag as u8;
    object._oPreFlag = false;
    object._oTrapFlag = false;
    object._oDoorFlag = false;
}

/// Original: `AddCryptBook` (objects.cpp).
// @port objects.cpp|devilution::AddCryptBook(_object_id ot, int v2, Point position) sha=657dafdb6a45
fn add_crypt_book(ctx: &mut Ctx, ot: _object_id, v2: i32, position: Point) {
    let s = &mut ctx.objects;
    if s.ActiveObjectCount as usize >= MAXOBJECTS {
        return;
    }
    let oi = s.AvailableObjects[0];
    s.AvailableObjects[0] = s.AvailableObjects[MAXOBJECTS - 1 - s.ActiveObjectCount as usize];
    s.ActiveObjects[s.ActiveObjectCount as usize] = oi;
    ctx.gendung.dObject[position.x as usize][position.y as usize] = (oi + 1) as i8;
    setup_object(ctx, oi as usize, position, ot);
    add_crypt_object(ctx, oi as usize, v2);
    ctx.objects.ActiveObjectCount += 1;
}

/// Original: `AddCryptStoryBook` (objects.cpp).
// @port objects.cpp|devilution::AddCryptStoryBook(int s) sha=6639467cc629
fn add_crypt_story_book(ctx: &mut Ctx, s: i32) {
    let Some(position) = get_random_object_position(ctx, Displacement::new(3, 2)) else {
        return;
    };
    add_crypt_book(ctx, OBJ_L5BOOKS, s, position);
    for d in [(-2, 1), (-2, 0), (-1, -1), (1, -1), (2, 0), (2, 1)] {
        add_object(ctx, OBJ_L5CANDLE, position + Displacement::new(d.0, d.1));
    }
}

/// Original: `AddNakrulLever` (objects.cpp).
// @port objects.cpp|devilution::AddNakrulLever() sha=5fa63c47252b
fn add_nakrul_lever(ctx: &mut Ctx) {
    loop {
        let xp = rnd(ctx, 80) + 16;
        let yp = rnd(ctx, 80) + 16;
        if rnd_loc_ok_3x3(ctx, xp, yp) {
            break;
        }
    }
    let (r, c) = (ctx.crypt.UberRow, ctx.crypt.UberCol);
    add_object(ctx, OBJ_L5LEVER, Point::new(r + 3, c - 1));
}

/// Original: `AddNakrulBook` (objects.cpp).
// @port objects.cpp|devilution::AddNakrulBook(int a1, Point position) sha=b975db29a021
fn add_nakrul_book(ctx: &mut Ctx, a1: i32, position: Point) {
    add_crypt_book(ctx, OBJ_L5BOOKS, a1, position);
}

/// Original: `AddNakrulGate` (objects.cpp).
// @port objects.cpp|devilution::AddNakrulGate() sha=25b5967a35bd
fn add_nakrul_gate(ctx: &mut Ctx) {
    add_nakrul_lever(ctx);
    let (r, c) = (ctx.crypt.UberRow, ctx.crypt.UberCol);
    let order = match rnd(ctx, 6) {
        0 => [6, 7, 8],
        1 => [6, 8, 7],
        2 => [7, 6, 8],
        3 => [7, 8, 6],
        4 => [8, 7, 6],
        5 => [8, 6, 7],
        _ => return,
    };
    add_nakrul_book(ctx, order[0], Point::new(r + 3, c));
    add_nakrul_book(ctx, order[1], Point::new(r + 2, c - 3));
    add_nakrul_book(ctx, order[2], Point::new(r + 2, c + 2));
}

/// Original: `AddStoryBooks` (objects.cpp).
// @port objects.cpp|devilution::AddStoryBooks() sha=fb2687c3c747
fn add_story_books(ctx: &mut Ctx) {
    let Some(position) = get_random_object_position(ctx, Displacement::new(3, 2)) else {
        return;
    };
    add_object(ctx, OBJ_STORYBOOK, position);
    for d in [(-2, 1), (-2, 0), (-1, -1), (1, -1), (2, 0), (2, 1)] {
        add_object(ctx, OBJ_STORYCANDLE, position + Displacement::new(d.0, d.1));
    }
}

/// Original: `AddHookedBodies` (objects.cpp).
// @port objects.cpp|devilution::AddHookedBodies(int freq) sha=de140cd9e6c8
fn add_hooked_bodies(ctx: &mut Ctx, freq: i32) {
    for j in 0..DMAXY as i32 {
        let jj = 16 + j * 2;
        for i in 0..DMAXX as i32 {
            let ii = 16 + i * 2;
            let d = |x: i32, y: i32| ctx.gendung.dungeon[x as usize][y as usize];
            if d(i, j) != 1 && d(i, j) != 2 {
                continue;
            }
            if !ctx.rng.flip_coin(freq as u32) {
                continue;
            }
            if crate::levels::gendung::is_near_theme_room(ctx, Point::new(i, j)) {
                continue;
            }
            let d = |x: i32, y: i32| ctx.gendung.dungeon[x as usize][y as usize];
            if d(i, j) == 1 && d(i + 1, j) == 6 {
                match rnd(ctx, 3) {
                    0 => {
                        add_object(ctx, OBJ_TORTURE1, Point::new(ii + 1, jj));
                    }
                    1 => {
                        add_object(ctx, OBJ_TORTURE2, Point::new(ii + 1, jj));
                    }
                    2 => {
                        add_object(ctx, OBJ_TORTURE5, Point::new(ii + 1, jj));
                    }
                    _ => {}
                }
                continue;
            }
            if d(i, j) == 2 && d(i, j + 1) == 6 {
                let o = ctx.rng.pick_randomly_among(&[OBJ_TORTURE3, OBJ_TORTURE4]);
                add_object(ctx, o, Point::new(ii, jj));
            }
        }
    }
}

/// Original: `AddL4Goodies` (objects.cpp).
// @port objects.cpp|devilution::AddL4Goodies() sha=c72cb0512793
fn add_l4_goodies(ctx: &mut Ctx) {
    add_hooked_bodies(ctx, 6);
    init_rnd_loc_obj(ctx, 2, 6, OBJ_TNUDEM1);
    init_rnd_loc_obj(ctx, 2, 6, OBJ_TNUDEM2);
    init_rnd_loc_obj(ctx, 2, 6, OBJ_TNUDEM3);
    init_rnd_loc_obj(ctx, 2, 6, OBJ_TNUDEM4);
    init_rnd_loc_obj(ctx, 2, 6, OBJ_TNUDEW1);
    init_rnd_loc_obj(ctx, 2, 6, OBJ_TNUDEW2);
    init_rnd_loc_obj(ctx, 2, 6, OBJ_TNUDEW3);
    init_rnd_loc_obj(ctx, 2, 6, OBJ_DECAP);
    init_rnd_loc_obj(ctx, 1, 3, OBJ_CAULDRON);
}

/// Original: `AddLazStand` (objects.cpp).
// @port objects.cpp|devilution::AddLazStand() sha=b69442a07e7d
fn add_laz_stand(ctx: &mut Ctx) {
    let mut cnt = 0;
    let (mut xp, mut yp) = (0, 0);
    let mut found = false;
    while !found {
        found = true;
        xp = rnd(ctx, 80) + 16;
        yp = rnd(ctx, 80) + 16;
        for yy in -3..=3 {
            for xx in -2..=3 {
                if !rnd_loc_ok(ctx, xp + xx, yp + yy) {
                    found = false;
                }
            }
        }
        if !found {
            cnt += 1;
            if cnt > 10000 {
                init_rnd_loc_obj(ctx, 1, 1, OBJ_LAZSTAND);
                return;
            }
        }
    }
    add_object(ctx, OBJ_LAZSTAND, Point::new(xp, yp));
    add_object(ctx, OBJ_TNUDEM2, Point::new(xp, yp + 2));
    add_object(ctx, OBJ_STORYCANDLE, Point::new(xp + 1, yp + 2));
    add_object(ctx, OBJ_TNUDEM3, Point::new(xp + 2, yp + 2));
    add_object(ctx, OBJ_TNUDEW1, Point::new(xp, yp - 2));
    add_object(ctx, OBJ_STORYCANDLE, Point::new(xp + 1, yp - 2));
    add_object(ctx, OBJ_TNUDEW2, Point::new(xp + 2, yp - 2));
    add_object(ctx, OBJ_STORYCANDLE, Point::new(xp - 1, yp - 1));
    add_object(ctx, OBJ_TNUDEW3, Point::new(xp - 1, yp));
    add_object(ctx, OBJ_STORYCANDLE, Point::new(xp - 1, yp + 1));
}

/// Original: `DeleteObject` (objects.cpp).
// @port objects.cpp|devilution::DeleteObject(int oi, int i) sha=ea645d1706bd
fn delete_object(ctx: &mut Ctx, oi: usize, i: usize) {
    let position = ctx.objects.Objects[oi].position;
    ctx.gendung.dObject[position.x as usize][position.y as usize] = 0;
    let s = &mut ctx.objects;
    s.AvailableObjects[MAXOBJECTS - s.ActiveObjectCount as usize] = oi as i32;
    s.ActiveObjectCount -= 1;
    if ctx.cursor.ObjectUnderCursor == Some(oi) {
        // Unselect object if this was highlighted by player
        ctx.cursor.ObjectUnderCursor = None;
    }
    let s = &mut ctx.objects;
    if s.ActiveObjectCount > 0 && i != s.ActiveObjectCount as usize {
        s.ActiveObjects[i] = s.ActiveObjects[s.ActiveObjectCount as usize];
    }
}

/// Original: `AddChest` (objects.cpp).
// @port objects.cpp|devilution::AddChest(Object &chest) sha=cdfe4def6804
fn add_chest(ctx: &mut Ctx, oi: usize) {
    if ctx.rng.flip_coin(2) {
        ctx.objects.Objects[oi]._oAnimFrame += 3;
    }
    ctx.objects.Objects[oi]._oRndSeed = ctx.rng.advance_rnd_seed() as u32;
    let setlevel = ctx.gendung.setlevel;
    let v1 = match ctx.objects.Objects[oi]._otype {
        OBJ_CHEST1 | OBJ_TCHEST1 => Some(if setlevel { 1 } else { rnd(ctx, 2) }),
        OBJ_TCHEST2 | OBJ_CHEST2 => Some(if setlevel { 2 } else { rnd(ctx, 3) }),
        OBJ_TCHEST3 | OBJ_CHEST3 => Some(if setlevel { 3 } else { rnd(ctx, 4) }),
        _ => None,
    };
    if let Some(v) = v1 {
        ctx.objects.Objects[oi]._oVar1 = v;
    }
    ctx.objects.Objects[oi]._oVar2 = rnd(ctx, 8);
}

/// Original: `ObjSetMicro` (objects.cpp).
// @port objects.cpp|devilution::ObjSetMicro(Point position, int pn) sha=da741bc92d40
fn obj_set_micro(ctx: &mut Ctx, position: Point, pn: i32) {
    ctx.gendung.dPiece[position.x as usize][position.y as usize] = pn as u16;
}

/// Original: `DoorSet` (objects.cpp).
// @port objects.cpp|devilution::DoorSet(Point position, bool isLeftDoor) sha=f7a72b158bbf
fn door_set(ctx: &mut Ctx, position: Point, is_left_door: bool) {
    let pn = piece(ctx, position.x, position.y);
    let v = match pn {
        42 => 391,
        44 => 393,
        49 => {
            if is_left_door {
                410
            } else {
                411
            }
        }
        53 => 396,
        54 => 397,
        60 => 398,
        66 => 399,
        67 => 400,
        68 => 402,
        69 => 403,
        71 => 405,
        211 => 406,
        353 => 408,
        354 => 409,
        410 | 411 => 395,
        _ => return,
    };
    obj_set_micro(ctx, position, v);
}

/// Original: `CryptDoorSet` (objects.cpp).
// @port objects.cpp|devilution::CryptDoorSet(Point position, bool isLeftDoor) sha=8eb76c87e965
fn crypt_door_set(ctx: &mut Ctx, position: Point, is_left_door: bool) {
    let pn = piece(ctx, position.x, position.y);
    let v = match pn {
        74 => 203,
        78 => 207,
        85 => {
            if is_left_door {
                231
            } else {
                233
            }
        }
        90 => 214,
        92 => 217,
        98 => 219,
        110 => 221,
        112 => 223,
        114 => 225,
        116 => 227,
        118 => 229,
        231 | 233 => 211,
        _ => return,
    };
    obj_set_micro(ctx, position, v);
}

fn set_special(ctx: &mut Ctx, p: Point, v: i8) {
    ctx.gendung.dSpecial[p.x as usize][p.y as usize] = v;
}

/// Original: `SetDoorStateOpen` (objects.cpp).
// @port objects.cpp|devilution::SetDoorStateOpen(Object &door) sha=85b43fdc9863
fn set_door_state_open(ctx: &mut Ctx, oi: usize) {
    let door = &mut ctx.objects.Objects[oi];
    door._oVar4 = DOOR_OPEN;
    door._oPreFlag = true;
    door._oMissFlag = true;
    door._oSelFlag = 2;
    let (pos, otype, var1) = (door.position, door._otype, door._oVar1);
    match otype {
        OBJ_L1LDOOR => {
            // 214: blood splater
            // 407: blood pool
            // 392: open door (no frame)
            obj_set_micro(ctx, pos, if var1 == 214 { 407 } else { 392 });
            set_special(ctx, pos, 7);
            door_set(ctx, pos + Direction::NorthEast, true);
        }
        OBJ_L1RDOOR => {
            obj_set_micro(ctx, pos, 394);
            set_special(ctx, pos, 8);
            door_set(ctx, pos + Direction::NorthWest, false);
        }
        OBJ_L2LDOOR => {
            obj_set_micro(ctx, pos, 12);
            set_special(ctx, pos, 5);
        }
        OBJ_L2RDOOR => {
            obj_set_micro(ctx, pos, 16);
            set_special(ctx, pos, 6);
        }
        OBJ_L3LDOOR => obj_set_micro(ctx, pos, 537),
        OBJ_L3RDOOR => obj_set_micro(ctx, pos, 540),
        OBJ_L5LDOOR => {
            obj_set_micro(ctx, pos, 205);
            crypt_door_set(ctx, pos + Direction::NorthEast, true);
        }
        OBJ_L5RDOOR => {
            obj_set_micro(ctx, pos, 208);
            crypt_door_set(ctx, pos + Direction::NorthWest, false);
        }
        _ => {}
    }
}

/// Original: `SetDoorStateClosed` (objects.cpp).
// @port objects.cpp|devilution::SetDoorStateClosed(Object &door) sha=e1d1c06385c2
fn set_door_state_closed(ctx: &mut Ctx, oi: usize) {
    let door = &mut ctx.objects.Objects[oi];
    door._oVar4 = DOOR_CLOSED;
    door._oPreFlag = false;
    door._oMissFlag = false;
    door._oSelFlag = 3;
    let (pos, otype, var1, var2) = (door.position, door._otype, door._oVar1, door._oVar2);
    match otype {
        OBJ_L1LDOOR | OBJ_L1RDOOR => {
            // Clear overlapping arches
            set_special(ctx, pos, 0);
            obj_set_micro(ctx, pos, var1 - 1);
            // Restore the normal tile where the open door used to be
            let (open_position, alt) = if otype == OBJ_L1LDOOR { (pos + Direction::NorthEast, 411) } else { (pos + Direction::NorthWest, 410) };
            if var2 == 50 && piece(ctx, open_position.x, open_position.y) == 395 {
                obj_set_micro(ctx, open_position, alt);
            } else {
                obj_set_micro(ctx, open_position, var2 - 1);
            }
        }
        OBJ_L2LDOOR => {
            // Clear overlapping arches
            set_special(ctx, pos, 0);
            obj_set_micro(ctx, pos, 537);
        }
        OBJ_L2RDOOR => {
            // Clear overlapping arches
            set_special(ctx, pos, 0);
            obj_set_micro(ctx, pos, 539);
        }
        OBJ_L3LDOOR => obj_set_micro(ctx, pos, 530),
        OBJ_L3RDOOR => obj_set_micro(ctx, pos, 533),
        OBJ_L5LDOOR | OBJ_L5RDOOR => {
            obj_set_micro(ctx, pos, var1 - 1);
            // Restore the normal tile where the open door used to be
            let (open_position, alt) = if otype == OBJ_L5LDOOR { (pos + Direction::NorthEast, 233) } else { (pos + Direction::NorthWest, 231) };
            if var2 == 86 && piece(ctx, open_position.x, open_position.y) == 209 {
                obj_set_micro(ctx, open_position, alt);
            } else {
                obj_set_micro(ctx, open_position, var2 - 1);
            }
        }
        _ => {}
    }
}

/// Original: `AddDoor` (objects.cpp).
// @port objects.cpp|devilution::AddDoor(Object &door) sha=8a6b9a9ee8f9
fn add_door(ctx: &mut Ctx, oi: usize) {
    ctx.objects.Objects[oi]._oDoorFlag = true;
    let p = ctx.objects.Objects[oi].position;
    match ctx.objects.Objects[oi]._otype {
        OBJ_L1LDOOR | OBJ_L5LDOOR => {
            let (v1, v2) = (piece(ctx, p.x, p.y) + 1, piece(ctx, p.x, p.y - 1) + 1);
            ctx.objects.Objects[oi]._oVar1 = v1;
            ctx.objects.Objects[oi]._oVar2 = v2;
        }
        OBJ_L1RDOOR | OBJ_L5RDOOR => {
            let (v1, v2) = (piece(ctx, p.x, p.y) + 1, piece(ctx, p.x - 1, p.y) + 1);
            ctx.objects.Objects[oi]._oVar1 = v1;
            ctx.objects.Objects[oi]._oVar2 = v2;
        }
        _ => {}
    }
    set_door_state_closed(ctx, oi);
}

/// Original: `AddSarcophagus` (objects.cpp).
// @port objects.cpp|devilution::AddSarcophagus(Object &sarcophagus) sha=a7145902a643
fn add_sarcophagus(ctx: &mut Ctx, oi: usize) {
    let p = ctx.objects.Objects[oi].position;
    ctx.gendung.dObject[p.x as usize][(p.y - 1) as usize] = -((oi + 1) as i8);
    ctx.objects.Objects[oi]._oVar1 = rnd(ctx, 10);
    ctx.objects.Objects[oi]._oRndSeed = ctx.rng.advance_rnd_seed() as u32;
    if ctx.objects.Objects[oi]._oVar1 >= 8 {
        let v2 = match crate::monster::pre_spawn_skeleton(ctx) {
            Some(m) => m as i32,
            None => -1,
        };
        ctx.objects.Objects[oi]._oVar2 = v2;
    }
}

/// Original: `AddFlameTrap` (objects.cpp).
// @port objects.cpp|devilution::AddFlameTrap(Object &flameTrap) sha=04c2eb9e89d6
fn add_flame_trap(ctx: &mut Ctx, oi: usize) {
    let (trapid, trapdir) = (ctx.objects.trapid, ctx.objects.trapdir);
    let o = &mut ctx.objects.Objects[oi];
    o._oVar1 = trapid;
    o._oVar2 = 0;
    o._oVar3 = trapdir;
    o._oVar4 = 0;
}

/// Original: `AddFlameLever` (objects.cpp).
// @port objects.cpp|devilution::AddFlameLever(Object &flameLever) sha=d4853459c718
fn add_flame_lever(ctx: &mut Ctx, oi: usize) {
    let trapid = ctx.objects.trapid;
    let o = &mut ctx.objects.Objects[oi];
    o._oVar1 = trapid;
    o._oVar2 = MissileID::InfernoControl as i8 as i32;
}

/// Original: `AddTrap` (objects.cpp).
// @port objects.cpp|devilution::AddTrap(Object &trap) sha=51c534cb4ee2
fn add_trap(ctx: &mut Ctx, oi: usize) {
    let mut effective_level = ctx.gendung.currlevel as i32;
    if ctx.gendung.leveltype == DungeonType::Nest {
        effective_level -= 4;
    } else if ctx.gendung.leveltype == DungeonType::Crypt {
        effective_level -= 8;
    }
    let missile_type = rnd(ctx, effective_level / 3 + 1);
    let trap = &mut ctx.objects.Objects[oi];
    if missile_type == 0 {
        trap._oVar3 = MissileID::Arrow as i8 as i32;
    }
    if missile_type == 1 {
        trap._oVar3 = MissileID::Firebolt as i8 as i32;
    }
    if missile_type == 2 {
        trap._oVar3 = MissileID::LightningControl as i8 as i32;
    }
    trap._oVar4 = 0;
}

// ---------------------------------------------------------------------------------------------
// objects.cpp, part 2: per-type setup, per-tick updates, doors, levers and books.
// ---------------------------------------------------------------------------------------------

/// Original: `AddObjectLight` (objects.cpp).
// @port objects.cpp|devilution::AddObjectLight(Object &object) sha=87865fa41a2b
fn add_object_light(ctx: &mut Ctx, oi: usize) {
    let radius = match ctx.objects.Objects[oi]._otype {
        OBJ_STORYCANDLE | OBJ_L5CANDLE => 3,
        OBJ_L1LIGHT | OBJ_SKFIRE | OBJ_CANDLE1 | OBJ_CANDLE2 | OBJ_BOOKCANDLE | OBJ_BCROSS | OBJ_TBCROSS => 5,
        OBJ_TORCHL | OBJ_TORCHR | OBJ_TORCHL2 | OBJ_TORCHR2 => 8,
        _ => return,
    };
    let position = ctx.objects.Objects[oi].position;
    crate::lighting::do_lighting(ctx, position, radius, Displacement::default());
    if ctx.objects.LoadingMapObjects {
        crate::lighting::do_un_light(ctx, position, radius);
        ctx.lighting.UpdateLighting = true;
    }
    ctx.objects.Objects[oi]._oVar1 = -1;
}

/// Original: `AddBarrel` (objects.cpp).
// @port objects.cpp|devilution::AddBarrel(Object &barrel) sha=eb08b00c54eb
fn add_barrel(ctx: &mut Ctx, oi: usize) {
    ctx.objects.Objects[oi]._oVar1 = 0;
    ctx.objects.Objects[oi]._oRndSeed = ctx.rng.advance_rnd_seed() as u32;
    let v2 = if ctx.objects.Objects[oi].is_explosive() { 0 } else { rnd(ctx, 10) };
    ctx.objects.Objects[oi]._oVar2 = v2;
    ctx.objects.Objects[oi]._oVar3 = rnd(ctx, 3);
    if v2 >= 8 {
        let v4 = match crate::monster::pre_spawn_skeleton(ctx) {
            Some(m) => m as i32,
            None => -1,
        };
        ctx.objects.Objects[oi]._oVar4 = v4;
    }
}

/// Original: `AddShrine` (objects.cpp).
// @port objects.cpp|devilution::AddShrine(Object &shrine) sha=ccd1b937ea27
fn add_shrine(ctx: &mut Ctx, oi: usize) {
    ctx.objects.Objects[oi]._oRndSeed = ctx.rng.advance_rnd_seed() as u32;
    let mut slist = [false; shrine_type::NumberOfShrineTypes as usize];
    ctx.objects.Objects[oi]._oPreFlag = true;
    let shrines = if ctx.init.gb_is_hellfire { shrine_type::NumberOfShrineTypes } else { 26 };
    let currlevel = ctx.gendung.currlevel as i32;
    let mp = ctx.init.gb_is_multiplayer;
    for j in 0..shrines as usize {
        slist[j] = currlevel >= SHRINEMIN[j] && currlevel <= SHRINEMAX[j];
        if mp && SHRINEAVAIL[j] == SHRINE_TYPE_SINGLE {
            slist[j] = false;
        } else if !mp && SHRINEAVAIL[j] == SHRINE_TYPE_MULTI {
            slist[j] = false;
        }
    }
    let mut val;
    loop {
        val = rnd(ctx, shrines);
        if slist[val as usize] {
            break;
        }
    }
    ctx.objects.Objects[oi]._oVar1 = val;
    if !ctx.rng.flip_coin(2) {
        ctx.objects.Objects[oi]._oAnimFrame = 12;
        ctx.objects.Objects[oi]._oAnimLen = 22;
    }
}

/// Original: `AddBookcase` (objects.cpp).
// @port objects.cpp|devilution::AddBookcase(Object &bookcase) sha=7df4281def8d
fn add_bookcase(ctx: &mut Ctx, oi: usize) {
    ctx.objects.Objects[oi]._oRndSeed = ctx.rng.advance_rnd_seed() as u32;
    ctx.objects.Objects[oi]._oPreFlag = true;
}

/// Original: `AddLargeFountain` (objects.cpp).
// @port objects.cpp|devilution::AddLargeFountain(Object &fountain) sha=b99105606f08
fn add_large_fountain(ctx: &mut Ctx, oi: usize) {
    let p = ctx.objects.Objects[oi].position;
    let (ox, oy) = (p.x as usize, p.y as usize);
    let id = -((oi + 1) as i8);
    ctx.gendung.dObject[ox][oy - 1] = id;
    ctx.gendung.dObject[ox - 1][oy] = id;
    ctx.gendung.dObject[ox - 1][oy - 1] = id;
    ctx.objects.Objects[oi]._oRndSeed = ctx.rng.advance_rnd_seed() as u32;
}

/// Original: `AddArmorStand` (objects.cpp).
// @port objects.cpp|devilution::AddArmorStand(Object &armorStand) sha=e0e3a973bd7e
fn add_armor_stand(ctx: &mut Ctx, oi: usize) {
    if !ctx.themes.armorFlag {
        ctx.objects.Objects[oi]._oAnimFlag = 1;
        ctx.objects.Objects[oi]._oSelFlag = 0;
    }
    ctx.objects.Objects[oi]._oRndSeed = ctx.rng.advance_rnd_seed() as u32;
}

/// Original: `AddDecapitatedBody` (objects.cpp).
// @port objects.cpp|devilution::AddDecapitatedBody(Object &decapitatedBody) sha=57e7a50acddc
fn add_decapitated_body(ctx: &mut Ctx, oi: usize) {
    ctx.objects.Objects[oi]._oRndSeed = ctx.rng.advance_rnd_seed() as u32;
    ctx.objects.Objects[oi]._oAnimFrame = (rnd(ctx, 8) + 1) as u32;
    ctx.objects.Objects[oi]._oPreFlag = true;
}

/// Original: `AddBookOfVileness` (objects.cpp).
// @port objects.cpp|devilution::AddBookOfVileness(Object &bookOfVileness) sha=6e7ddcebcd97
fn add_book_of_vileness(ctx: &mut Ctx, oi: usize) {
    if ctx.gendung.setlevel && ctx.gendung.setlvlnum == SL_VILEBETRAYER {
        ctx.objects.Objects[oi]._oAnimFrame = 4;
    }
}

/// Original: `AddMagicCircle` (objects.cpp).
// @port objects.cpp|devilution::AddMagicCircle(Object &magicCircle) sha=5fb55889218e
fn add_magic_circle(ctx: &mut Ctx, oi: usize) {
    ctx.objects.Objects[oi]._oRndSeed = ctx.rng.advance_rnd_seed() as u32;
    let o = &mut ctx.objects.Objects[oi];
    o._oPreFlag = true;
    o._oVar6 = 0;
    o._oVar5 = 1;
}

/// Original: `AddPedestalOfBlood` (objects.cpp).
// @port objects.cpp|devilution::AddPedestalOfBlood(Object &pedestalOfBlood) sha=9d67c83bc4f4
fn add_pedestal_of_blood(ctx: &mut Ctx, oi: usize) {
    let sp = ctx.gendung.SetPiece;
    let o = &mut ctx.objects.Objects[oi];
    o._oVar1 = sp.position.x;
    o._oVar2 = sp.position.y;
    o._oVar3 = sp.position.x + sp.size.width;
    o._oVar4 = sp.position.y + sp.size.height;
    o._oVar6 = 0;
}

/// Original: `AddStoryBook` (objects.cpp).
// @port objects.cpp|devilution::AddStoryBook(Object &storyBook) sha=291982cbe633
fn add_story_book(ctx: &mut Ctx, oi: usize) {
    let seed = ctx.diablo.glSeedTbl[16];
    ctx.rng.set_rnd_seed(seed);
    let v1 = rnd(ctx, 3);
    let currlevel = ctx.gendung.currlevel as i32;
    let o = &mut ctx.objects.Objects[oi];
    o._oVar1 = v1;
    if currlevel == 4 {
        o._oVar2 = STORY_TEXT[v1 as usize][0] as i32;
    } else if currlevel == 8 {
        o._oVar2 = STORY_TEXT[v1 as usize][1] as i32;
    } else if currlevel == 12 {
        o._oVar2 = STORY_TEXT[v1 as usize][2] as i32;
    }
    o._oVar3 = (currlevel / 4) + 3 * v1 - 1;
    o._oAnimFrame = (5 - 2 * v1) as u32;
    o._oVar4 = o._oAnimFrame as i32 + 1;
}

/// Original: `AddWeaponRack` (objects.cpp).
// @port objects.cpp|devilution::AddWeaponRack(Object &weaponRack) sha=03019bc792c3
fn add_weapon_rack(ctx: &mut Ctx, oi: usize) {
    if !ctx.themes.weaponFlag {
        ctx.objects.Objects[oi]._oAnimFlag = 1;
        ctx.objects.Objects[oi]._oSelFlag = 0;
    }
    ctx.objects.Objects[oi]._oRndSeed = ctx.rng.advance_rnd_seed() as u32;
}

/// Original: `AddTorturedBody` (objects.cpp).
// @port objects.cpp|devilution::AddTorturedBody(Object &torturedBody) sha=558f95df90d0
fn add_tortured_body(ctx: &mut Ctx, oi: usize) {
    ctx.objects.Objects[oi]._oRndSeed = ctx.rng.advance_rnd_seed() as u32;
    ctx.objects.Objects[oi]._oAnimFrame = (rnd(ctx, 4) + 1) as u32;
    ctx.objects.Objects[oi]._oPreFlag = true;
}

/// Original: `GetRndObjLoc` (objects.cpp).
// @port objects.cpp|devilution::GetRndObjLoc(int randarea) sha=397eb3026c98
fn get_rnd_obj_loc(ctx: &mut Ctx, mut randarea: i32) -> Point {
    if randarea == 0 {
        return Point::new(0, 0);
    }
    let mut tries = 0;
    let (mut x, mut y);
    loop {
        tries += 1;
        if tries > 1000 && randarea > 1 {
            randarea -= 1;
        }
        x = rnd(ctx, MAXDUNX as i32);
        y = rnd(ctx, MAXDUNY as i32);
        let mut failed = false;
        let mut i = 0;
        while i < randarea && !failed {
            let mut j = 0;
            while j < randarea && !failed {
                failed = !rnd_loc_ok(ctx, i + x, j + y);
                j += 1;
            }
            i += 1;
        }
        if !failed {
            break;
        }
    }
    Point::new(x, y)
}

/// Original: `AddMushPatch` (objects.cpp).
// @port objects.cpp|devilution::AddMushPatch() sha=8044d2da0e71
fn add_mush_patch(ctx: &mut Ctx) {
    if (ctx.objects.ActiveObjectCount as usize) < MAXOBJECTS {
        let i = ctx.objects.AvailableObjects[0];
        let loc = get_rnd_obj_loc(ctx, 5);
        let id = -((i + 1) as i8);
        let d = &mut ctx.gendung.dObject;
        d[(loc.x + 1) as usize][(loc.y + 1) as usize] = id;
        d[(loc.x + 2) as usize][(loc.y + 1) as usize] = id;
        d[(loc.x + 1) as usize][(loc.y + 2) as usize] = id;
        add_object(ctx, OBJ_MUSHPATCH, Point::new(loc.x + 2, loc.y + 2));
    }
}

/// Original: `IsLightVisible` (objects.cpp).
// @port objects.cpp|devilution::IsLightVisible(Object &light, int lightRadius) sha=1bef802c4de9
fn is_light_visible(ctx: &Ctx, oi: usize, light_radius: i32) -> bool {
    let lp = ctx.objects.Objects[oi].position;
    for pnum in 0..ctx.players.Players.len() {
        let player = &ctx.players.Players[pnum];
        if !player.plractive {
            continue;
        }
        if !crate::player::is_on_active_level(ctx, pnum) {
            continue;
        }
        if player.position.tile.walking_distance(lp) < light_radius + 10 {
            return true;
        }
    }
    false
}

/// Original: `UpdateObjectLight` (objects.cpp).
// @port objects.cpp|devilution::UpdateObjectLight(Object &light, int lightRadius) sha=9d7485a9a3ca
fn update_object_light(ctx: &mut Ctx, oi: usize, light_radius: i32) {
    if ctx.objects.Objects[oi]._oVar1 == -1 {
        return;
    }
    if is_light_visible(ctx, oi, light_radius) {
        if ctx.objects.Objects[oi]._oVar1 == 0 {
            let p = ctx.objects.Objects[oi].position;
            ctx.objects.Objects[oi]._olid = crate::lighting::add_light(ctx, p, light_radius as u8);
        }
        ctx.objects.Objects[oi]._oVar1 = 1;
    } else {
        if ctx.objects.Objects[oi]._oVar1 == 1 {
            let l = ctx.objects.Objects[oi]._olid;
            crate::lighting::add_un_light(ctx, l);
        }
        ctx.objects.Objects[oi]._oVar1 = 0;
    }
}

/// Original: `UpdateCircle` (objects.cpp).
// @port objects.cpp|devilution::UpdateCircle(Object &circle) sha=de78dc602acc
fn update_circle(ctx: &mut Ctx, oi: usize) {
    let cpos = ctx.objects.Objects[oi].position;
    let player_on_circle = crate::player::player_at_position(ctx, cpos);
    let Some(pnum) = player_on_circle else {
        let circle = &mut ctx.objects.Objects[oi];
        if circle._otype == OBJ_MCIRCLE1 {
            circle._oAnimFrame = 1;
        }
        if circle._otype == OBJ_MCIRCLE2 {
            circle._oAnimFrame = 3;
        }
        circle._oVar6 = 0;
        return;
    };
    let circle = &mut ctx.objects.Objects[oi];
    if circle._otype == OBJ_MCIRCLE1 {
        circle._oAnimFrame = 2;
    }
    if circle._otype == OBJ_MCIRCLE2 {
        circle._oAnimFrame = 4;
    }
    if circle.position == Point::new(45, 47) {
        circle._oVar6 = 2;
    } else if circle.position == Point::new(26, 46) {
        circle._oVar6 = 1;
    } else {
        circle._oVar6 = 0;
    }
    if circle.position == Point::new(35, 36) && circle._oVar5 == 3 {
        circle._oVar6 = 4;
        let (v1, v2, v3, v4) = (circle._oVar1, circle._oVar2, circle._oVar3, circle._oVar4);
        if ctx.quests.Quests[Q_BETRAYER as usize]._qvar1 <= 4 {
            ctx.objects.LoadingMapObjects = true;
            obj_change_map(ctx, v1, v2, v3, v4);
            ctx.objects.LoadingMapObjects = false;
            ctx.quests.Quests[Q_BETRAYER as usize]._qvar1 = 4;
            crate::msg::net_send_cmd_quest(ctx, true, Q_BETRAYER as usize);
        }
        let ptile = ctx.players.Players[pnum].position.tile;
        crate::missiles::add_missile(ctx, ptile, Point::new(35, 46), Direction::South, MissileID::Phasing, TARGET_BOTH, pnum as i32, 0, 0, None);
        if Some(pnum) == ctx.players.MyPlayer {
            ctx.diablo.last_mouse_button_action = crate::diablo::MouseActionType::None;
            ctx.diablo.sgb_mouse_down = CLICK_NONE;
        }
        crate::player::clr_plr_path(ctx, pnum);
        crate::player::start_stand(ctx, pnum, Direction::South);
    }
}

/// Original: `ObjectStopAnim` (objects.cpp).
// @port objects.cpp|devilution::ObjectStopAnim(Object &object) sha=fa224806b3b9
fn object_stop_anim(ctx: &mut Ctx, oi: usize) {
    let object = &mut ctx.objects.Objects[oi];
    if object._oAnimFrame == object._oAnimLen {
        object._oAnimCnt = 0;
        object._oAnimDelay = 1000;
    }
}

/// Original: `IsDoorClear` (objects.cpp): the closed door's tile is free of bodies, monsters, players and items.
// @port objects.cpp|devilution::IsDoorClear(const Object &door) sha=68c467b4a956
fn is_door_clear(ctx: &Ctx, oi: usize) -> bool {
    let p = ctx.objects.Objects[oi].position;
    let (x, y) = (p.x as usize, p.y as usize);
    ctx.gendung.dCorpse[x][y] == 0 && ctx.gendung.dMonster[x][y] == 0 && ctx.items.dItem[x][y] == 0 && ctx.gendung.dPlayer[x][y] == 0
}

/// Original: `UpdateDoor` (objects.cpp).
// @port objects.cpp|devilution::UpdateDoor(Object &door) sha=db7c73110cb1
fn update_door(ctx: &mut Ctx, oi: usize) {
    if ctx.objects.Objects[oi]._oVar4 == DOOR_CLOSED {
        return;
    }
    ctx.objects.Objects[oi]._oVar4 = if is_door_clear(ctx, oi) { DOOR_OPEN } else { DOOR_BLOCKED };
}

/// Original: `UpdateSarcophagus` (objects.cpp).
// @port objects.cpp|devilution::UpdateSarcophagus(Object &sarcophagus) sha=3b562ef39c20
fn update_sarcophagus(ctx: &mut Ctx, oi: usize) {
    let o = &mut ctx.objects.Objects[oi];
    if o._oAnimFrame == o._oAnimLen {
        o._oAnimFlag = 0;
    }
}

/// Original: `ActivateTrapLine` (objects.cpp).
// @port objects.cpp|devilution::ActivateTrapLine(int ttype, int tid) sha=febe1d716014
fn activate_trap_line(ctx: &mut Ctx, ttype: _object_id, tid: i32) {
    for i in 0..ctx.objects.ActiveObjectCount as usize {
        let t = ctx.objects.ActiveObjects[i] as usize;
        let trap = &mut ctx.objects.Objects[t];
        if trap._otype == ttype && trap._oVar1 == tid {
            trap._oVar4 = 1;
            trap._oAnimFlag = 1;
            trap._oAnimDelay = 1;
            let p = trap.position;
            ctx.objects.Objects[t]._olid = crate::lighting::add_light(ctx, p, 1);
        }
    }
}

/// Original: `UpdateFlameTrap` (objects.cpp).
// @port objects.cpp|devilution::UpdateFlameTrap(Object &trap) sha=d7277aa568b4
fn update_flame_trap(ctx: &mut Ctx, oi: usize) {
    let t = ctx.objects.Objects[oi].clone();
    if t._oVar2 != 0 {
        if t._oVar4 != 0 {
            ctx.objects.Objects[oi]._oAnimFrame -= 1;
            let frame = ctx.objects.Objects[oi]._oAnimFrame;
            if frame == 1 {
                ctx.objects.Objects[oi]._oVar4 = 0;
                crate::lighting::add_un_light(ctx, t._olid);
            } else if frame <= 4 {
                crate::lighting::change_light_radius(ctx, t._olid, frame as u8);
            }
        }
    } else if t._oVar4 == 0 {
        let mut hit = false;
        if t._oVar3 == 2 {
            let mut x = t.position.x - 2;
            let y = t.position.y;
            for _ in 0..5 {
                if ctx.gendung.dPlayer[x as usize][y as usize] != 0 || ctx.gendung.dMonster[x as usize][y as usize] != 0 {
                    hit = true;
                }
                x += 1;
            }
        } else {
            let x = t.position.x;
            let mut y = t.position.y - 2;
            for _ in 0..5 {
                if ctx.gendung.dPlayer[x as usize][y as usize] != 0 || ctx.gendung.dMonster[x as usize][y as usize] != 0 {
                    hit = true;
                }
                y += 1;
            }
        }
        if hit {
            ctx.objects.Objects[oi]._oVar4 = 1;
        }
        if ctx.objects.Objects[oi]._oVar4 != 0 {
            activate_trap_line(ctx, t._otype, t._oVar1);
        }
    } else {
        const DAMAGE: [i32; 6] = [6, 8, 10, 12, 10, 12];
        let mindam = DAMAGE[ctx.gendung.leveltype as i8 as usize - 1];
        let maxdam = mindam * 2;
        let (x, y) = (t.position.x as usize, t.position.y as usize);
        const TRAP_MISSILE: MissileID = MissileID::FireWallControl;
        let dt = crate::missiles::get_missile_data(TRAP_MISSILE).damage_type();
        if ctx.gendung.dMonster[x][y] > 0 {
            let m = (ctx.gendung.dMonster[x][y] - 1) as usize;
            crate::missiles::monster_trap_hit(ctx, m, mindam / 2, maxdam / 2, 0, TRAP_MISSILE, dt, false);
        }
        if ctx.gendung.dPlayer[x][y] > 0 {
            let mut unused = false;
            let p = (ctx.gendung.dPlayer[x][y] - 1) as usize;
            crate::missiles::player_m_hit(ctx, p, None, 0, mindam, maxdam, TRAP_MISSILE, dt, false, DeathReason::MonsterOrTrap, &mut unused);
        }
        let o = &mut ctx.objects.Objects[oi];
        if o._oAnimFrame == o._oAnimLen {
            o._oAnimFrame = 11;
        }
        if o._oAnimFrame <= 5 {
            let (l, f) = (o._olid, o._oAnimFrame);
            crate::lighting::change_light_radius(ctx, l, f as u8);
        }
    }
}

/// Original: `UpdateBurningCrossDamage` (objects.cpp).
// @port objects.cpp|devilution::UpdateBurningCrossDamage(Object &cross) sha=5a9d8fed6cad
fn update_burning_cross_damage(ctx: &mut Ctx, oi: usize) {
    let mut damage = [6, 8, 10, 12, 10, 12];
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    if ctx.players.Players[me]._pmode == PM_DEATH {
        return;
    }
    let lt = ctx.gendung.leveltype as i8 as usize - 1;
    let fire_resist = ctx.players.Players[me]._pFireResist as i32;
    if fire_resist > 0 {
        damage[lt] -= fire_resist * damage[lt] / 100;
    }
    if ctx.players.Players[me].position.tile != ctx.objects.Objects[oi].position + Displacement::new(0, -1) {
        return;
    }
    crate::player::apply_plr_damage(ctx, DamageType::Fire, me, 0, 0, damage[lt], DeathReason::MonsterOrTrap);
    if ctx.players.Players[me]._pHitPoints >> 6 > 0 {
        crate::player::player_say(ctx, me, HeroSpeech::Argh);
    }
}

/// Original: `ObjSetMini` (objects.cpp).
// @port objects.cpp|devilution::ObjSetMini(Point position, int v) sha=9779c8590562
fn obj_set_mini(ctx: &mut Ctx, position: Point, v: i32) {
    let mega = ctx.gendung.pMegaTiles.as_ref().expect("pMegaTiles")[(v - 1) as usize];
    let mega_origin = position.mega_to_world();
    obj_set_micro(ctx, mega_origin, mega.micro1 as i32);
    obj_set_micro(ctx, mega_origin + Direction::SouthEast, mega.micro2 as i32);
    obj_set_micro(ctx, mega_origin + Direction::SouthWest, mega.micro3 as i32);
    obj_set_micro(ctx, mega_origin + Direction::South, mega.micro4 as i32);
}

/// Original: `ObjL1Special` (objects.cpp).
// @port objects.cpp|devilution::ObjL1Special(int x1, int y1, int x2, int y2) sha=40b23822093c
fn obj_l1_special(ctx: &mut Ctx, x1: i32, y1: i32, x2: i32, y2: i32) {
    let g = &mut ctx.gendung;
    for i in y1..=y2 {
        for j in x1..=x2 {
            let (j, i) = (j as usize, i as usize);
            g.dSpecial[j][i] = 0;
            let v = match g.dPiece[j][i] {
                11 | 70 | 320 | 210 | 340 | 417 => 1,
                10 | 248 | 324 | 343 | 330 | 420 => 2,
                252 => 3,
                254 => 4,
                258 => 5,
                266 => 6,
                _ => continue,
            };
            g.dSpecial[j][i] = v;
        }
    }
}

/// Original: `ObjL2Special` (objects.cpp).
// @port objects.cpp|devilution::ObjL2Special(int x1, int y1, int x2, int y2) sha=07ce52354df7
fn obj_l2_special(ctx: &mut Ctx, x1: i32, y1: i32, x2: i32, y2: i32) {
    let g = &mut ctx.gendung;
    for j in y1..=y2 {
        for i in x1..=x2 {
            let (i, j) = (i as usize, j as usize);
            g.dSpecial[i][j] = 0;
            match g.dPiece[i][j] {
                540 | 177 | 550 => g.dSpecial[i][j] = 5,
                541 | 552 => g.dSpecial[i][j] = 6,
                _ => {}
            }
        }
    }
    for j in y1..=y2 {
        for i in x1..=x2 {
            let (i, j) = (i as usize, j as usize);
            if g.dPiece[i][j] == 131 {
                g.dSpecial[i][j + 1] = 2;
                g.dSpecial[i][j + 2] = 1;
            }
            if g.dPiece[i][j] == 134 || g.dPiece[i][j] == 138 {
                g.dSpecial[i + 1][j] = 3;
                g.dSpecial[i + 2][j] = 4;
            }
        }
    }
}

/// Original: `OpenDoor` (objects.cpp).
// @port objects.cpp|devilution::OpenDoor(Object &door) sha=719c2169cd20
fn open_door(ctx: &mut Ctx, oi: usize) {
    ctx.objects.Objects[oi]._oAnimFrame += 2;
    set_door_state_open(ctx, oi);
}

/// Original: `CloseDoor` (objects.cpp).
// @port objects.cpp|devilution::CloseDoor(Object &door) sha=a35cacdde27d
fn close_door(ctx: &mut Ctx, oi: usize) {
    ctx.objects.Objects[oi]._oAnimFrame -= 2;
    set_door_state_closed(ctx, oi);
}

/// Original: `OperateDoor` (objects.cpp).
// @port objects.cpp|devilution::OperateDoor(Object &door, bool sendflag) sha=13c722497830
fn operate_door(ctx: &mut Ctx, oi: usize, sendflag: bool) {
    use crate::effects_data::*;
    let (otype, pos, var4) = (ctx.objects.Objects[oi]._otype, ctx.objects.Objects[oi].position, ctx.objects.Objects[oi]._oVar4);
    let is_crypt = matches!(otype, OBJ_L5LDOOR | OBJ_L5RDOOR);
    let open_door_flag = var4 == DOOR_CLOSED;
    if !open_door_flag && !is_door_clear(ctx, oi) {
        crate::effects::play_sfx_loc(ctx, if is_crypt { IS_CRCLOS } else { IS_DOORCLOS }, pos, true);
        ctx.objects.Objects[oi]._oVar4 = DOOR_BLOCKED;
        return;
    }
    if open_door_flag {
        crate::effects::play_sfx_loc(ctx, if is_crypt { IS_CROPEN } else { IS_DOOROPEN }, pos, true);
        open_door(ctx, oi);
    } else {
        crate::effects::play_sfx_loc(ctx, if is_crypt { IS_CRCLOS } else { IS_DOORCLOS }, pos, true);
        close_door(ctx, oi);
    }
    redo_player_vision(ctx);
    if sendflag {
        let id = ctx.players.MyPlayerId;
        crate::msg::net_send_cmd_loc(ctx, id, true, if open_door_flag { CMD_OPENDOOR } else { CMD_CLOSEDOOR }, pos);
    }
}

/// Original: `AreAllLeversActivated` (objects.cpp).
// @port objects.cpp|devilution::AreAllLeversActivated(int leverId) sha=0e537bacde48
fn are_all_levers_activated(ctx: &Ctx, lever_id: i32) -> bool {
    for j in 0..ctx.objects.ActiveObjectCount as usize {
        let lever = &ctx.objects.Objects[ctx.objects.ActiveObjects[j] as usize];
        if lever._otype == OBJ_SWITCHSKL && lever._oVar8 == lever_id && lever._oSelFlag != 0 {
            return false;
        }
    }
    true
}

/// `ObjectAtPosition`: the object whose tile (or large-object footprint) covers `position`.
// @port objects.h|devilution::ObjectAtPosition(Point position) sha=f5d485084297
pub fn object_at_position(ctx: &Ctx, position: Point) -> usize {
    (ctx.gendung.dObject[position.x as usize][position.y as usize] as i32).unsigned_abs() as usize - 1
}

/// Original: `UpdateLeverState` (objects.cpp).
// @port objects.cpp|devilution::UpdateLeverState(Object &object) sha=1a190b118622
fn update_lever_state(ctx: &mut Ctx, oi: usize) {
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    ctx.objects.Objects[oi]._oSelFlag = 0;
    ctx.objects.Objects[oi]._oAnimFrame += 1;
    let o = ctx.objects.Objects[oi].clone();
    if ctx.gendung.currlevel == 16 && !are_all_levers_activated(ctx, o._oVar8) {
        return;
    }
    if ctx.gendung.currlevel == 24 {
        sync_nakrul_room(ctx);
        ctx.crypt.IsUberLeverActivated = true;
        return;
    }
    if ctx.gendung.setlevel && ctx.gendung.setlvlnum == SL_VILEBETRAYER {
        let c = object_at_position(ctx, Point::new(35, 36));
        ctx.objects.Objects[c]._oVar5 += 1;
    }
    obj_change_map(ctx, o._oVar1, o._oVar2, o._oVar3, o._oVar4);
}

/// Original: `OperateLever` (objects.cpp).
// @port objects.cpp|devilution::OperateLever(Object &object, bool sendmsg) sha=1f52c260c3e8
fn operate_lever(ctx: &mut Ctx, oi: usize, sendmsg: bool) {
    use crate::effects_data::*;
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    let pos = ctx.objects.Objects[oi].position;
    crate::effects::play_sfx_loc(ctx, IS_LEVER, pos, true);
    update_lever_state(ctx, oi);
    if ctx.gendung.currlevel == 24 {
        let (r, c) = (ctx.crypt.UberRow, ctx.crypt.UberCol);
        crate::effects::play_sfx_loc(ctx, IS_CROPEN, Point::new(r, c), true);
        ctx.quests.Quests[Q_NAKRUL as usize]._qactive = QUEST_DONE;
        crate::msg::net_send_cmd_quest(ctx, true, Q_NAKRUL as usize);
    }
    if sendmsg {
        let id = ctx.players.MyPlayerId;
        crate::msg::net_send_cmd_loc(ctx, id, false, CMD_OPERATEOBJ, pos);
    }
}

/// Original: `OperateBook` (objects.cpp).
// @port objects.cpp|devilution::OperateBook(Player &player, Object &book, bool sendmsg) sha=16fc2590b4c9
fn operate_book(ctx: &mut Ctx, pnum: usize, oi: usize, sendmsg: bool) {
    use crate::effects_data::*;
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    let bpos = ctx.objects.Objects[oi].position;
    if ctx.gendung.setlevel && ctx.gendung.setlvlnum == SL_VILEBETRAYER {
        let target = if bpos == Point::new(26, 45) {
            Point::new(27, 29)
        } else if bpos == Point::new(45, 46) {
            Point::new(43, 29)
        } else {
            return;
        };
        let circle = object_at_position(ctx, bpos + Direction::SouthWest);
        assert!(ctx.objects.Objects[circle]._otype == OBJ_MCIRCLE2);
        // Only verfiy that the player stands on the circle when it's the local player (sendmsg), cause for remote players the position could be desynced
        if sendmsg && ctx.objects.Objects[circle].position != ctx.players.Players[pnum].position.tile {
            return;
        }
        ctx.objects.Objects[circle]._oVar6 = 4;
        let c = object_at_position(ctx, Point::new(35, 36));
        ctx.objects.Objects[c]._oVar5 += 1;
        let ptile = ctx.players.Players[pnum].position.tile;
        crate::missiles::add_missile(ctx, ptile, target, Direction::South, MissileID::Phasing, TARGET_BOTH, pnum as i32, 0, 0, None);
    }
    ctx.objects.Objects[oi]._oSelFlag = 0;
    ctx.objects.Objects[oi]._oAnimFrame += 1;
    if sendmsg {
        let id = ctx.players.MyPlayerId;
        crate::msg::net_send_cmd_loc(ctx, id, false, CMD_OPERATEOBJ, bpos);
    }
    if !ctx.gendung.setlevel {
        return;
    }
    if ctx.gendung.setlvlnum == SL_BONECHAMB {
        if sendmsg {
            let g = SpellID::Guardian as i8 as usize;
            let new_spell_level = ctx.players.Players[pnum]._pSplLvl[g] + 1;
            if new_spell_level <= crate::player::MaxSpellLevel {
                ctx.players.Players[pnum]._pSplLvl[g] = new_spell_level;
                crate::msg::net_send_cmd_param2(ctx, true, CMD_CHANGE_SPELL_LEVEL, SpellID::Guardian as i8 as u16, new_spell_level as u16);
            }
            if Some(pnum) == ctx.players.MyPlayer {
                refresh_inventory_stat_cache(ctx, pnum);
            }
            ctx.quests.Quests[Q_SCHAMB as usize]._qactive = QUEST_DONE;
            crate::msg::net_send_cmd_quest(ctx, true, Q_SCHAMB as usize);
        }
        crate::effects::play_sfx_loc(ctx, IS_QUESTDN, bpos, true);
        crate::error::init_diablo_msg_id(ctx, EMSG_BONECHAMB as usize, 3500);
        let (ptile, pdir) = (ctx.players.Players[pnum].position.tile, ctx.players.Players[pnum]._pdir);
        crate::missiles::add_missile(ctx, ptile, bpos + Displacement::new(-2, -4), pdir, MissileID::Guardian, TARGET_MONSTERS, pnum as i32, 0, 0, None);
    }
    if ctx.gendung.setlvlnum == SL_VILEBETRAYER {
        let o = ctx.objects.Objects[oi].clone();
        obj_change_map(ctx, o._oVar1, o._oVar2, o._oVar3, o._oVar4);
        for j in 0..ctx.objects.ActiveObjectCount as usize {
            let a = ctx.objects.ActiveObjects[j] as usize;
            sync_object_anim(ctx, a);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// objects.cpp, part 3: quest books, containers and the first shrines.
// ---------------------------------------------------------------------------------------------

/// `for (Item &item : InventoryPlayerItemsRange { player }) item.updateRequiredStatsCacheForPlayer(player);`
/// followed by `if (IsStashOpen) Stash.RefreshItemStatFlags();`
fn refresh_inventory_stat_cache(ctx: &mut Ctx, pnum: usize) {
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

/// `PlayerItemsRange`: body, inventory and belt items (non-empty), applied through `f`.
fn for_player_items(ctx: &mut Ctx, pnum: usize, f: &mut dyn FnMut(&mut crate::items::Item)) {
    let player = &mut ctx.players.Players[pnum];
    let n = player._pNumInv as usize;
    for item in player.InvBody.iter_mut().chain(player.InvList[..n].iter_mut()).chain(player.SpdList.iter_mut()) {
        if !item.is_empty() {
            f(item);
        }
    }
}

/// `InventoryPlayerItemsRange`: inventory items (non-empty), applied through `f`.
fn for_inventory_items(ctx: &mut Ctx, pnum: usize, f: &mut dyn FnMut(&mut crate::items::Item)) {
    let player = &mut ctx.players.Players[pnum];
    let n = player._pNumInv as usize;
    for item in player.InvList[..n].iter_mut() {
        if !item.is_empty() {
            f(item);
        }
    }
}

fn send_operate(ctx: &mut Ctx, b_hi_pri: bool, position: Point) {
    let id = ctx.players.MyPlayerId;
    crate::msg::net_send_cmd_loc(ctx, id, b_hi_pri, CMD_OPERATEOBJ, position);
}

fn is_my_player(ctx: &Ctx, pnum: usize) -> bool {
    Some(pnum) == ctx.players.MyPlayer
}

fn diablo_msg(ctx: &mut Ctx, m: diablo_message) {
    crate::error::init_diablo_msg_id(ctx, m as usize, 3500);
}

/// Original: `OperateBookLever` (objects.cpp).
// @port objects.cpp|devilution::OperateBookLever(Object &questBook, bool sendmsg) sha=7a1275264ea3
fn operate_book_lever(ctx: &mut Ctx, oi: usize, sendmsg: bool) {
    if ctx.items.ActiveItemCount as usize >= crate::items::MAXITEMS {
        return;
    }
    if ctx.objects.Objects[oi]._oSelFlag == 0 || crate::minitext::qtextflag(ctx) {
        return;
    }
    let book = ctx.objects.Objects[oi].clone();
    let sp_world = ctx.gendung.SetPiece.position.mega_to_world();
    if book._otype == OBJ_BLINDBOOK && ctx.quests.Quests[Q_BLIND as usize]._qvar1 == 0 {
        let q = &mut ctx.quests.Quests[Q_BLIND as usize];
        q._qactive = QUEST_ACTIVE;
        q._qlog = true;
        q._qvar1 = 1;
        crate::msg::net_send_cmd_quest(ctx, true, Q_BLIND as usize);
    }
    if book._otype == OBJ_BLOODBOOK && ctx.quests.Quests[Q_BLOOD as usize]._qvar1 == 0 {
        let q = &mut ctx.quests.Quests[Q_BLOOD as usize];
        q._qactive = QUEST_ACTIVE;
        q._qlog = true;
        q._qvar1 = 1;
        crate::msg::net_send_cmd_quest(ctx, true, Q_BLOOD as usize);
        if sendmsg {
            crate::items::spawn_quest_item(ctx, IDI_BLDSTONE, sp_world + Displacement::new(9, 17), 0, 1, true);
        }
    }
    if book._otype == OBJ_STEELTOME && ctx.quests.Quests[Q_WARLORD as usize]._qvar1 == QS_WARLORD_INIT as u8 {
        let q = &mut ctx.quests.Quests[Q_WARLORD as usize];
        q._qactive = QUEST_ACTIVE;
        q._qlog = true;
        q._qvar1 = QS_WARLORD_STEELTOME_READ as u8;
        crate::msg::net_send_cmd_quest(ctx, true, Q_WARLORD as usize);
    }
    if book._oAnimFrame != book._oVar6 {
        if book._otype != OBJ_BLOODBOOK {
            obj_change_map(ctx, book._oVar1, book._oVar2, book._oVar3, book._oVar4);
        }
        if book._otype == OBJ_BLINDBOOK {
            if sendmsg {
                crate::items::spawn_unique(ctx, UITEM_OPTAMULET, sp_world + Displacement::new(5, 5), None, true, true);
            }
            let tren = ctx.gendung.TransVal;
            ctx.gendung.TransVal = 9;
            crate::levels::gendung::drlg_m_rect_trans_points(ctx, Point::new(book._oVar1, book._oVar2), Point::new(book._oVar3, book._oVar4));
            ctx.gendung.TransVal = tren;
        }
    }
    ctx.objects.Objects[oi]._oAnimFrame = book._oVar6;
    crate::minitext::init_q_text_msg(ctx, book.bookMessage);
    if sendmsg {
        send_operate(ctx, false, book.position);
    }
}

/// Original: `OperateChamberOfBoneBook` (objects.cpp).
// @port objects.cpp|devilution::OperateChamberOfBoneBook(Object &questBook, bool sendmsg) sha=435755282690
fn operate_chamber_of_bone_book(ctx: &mut Ctx, oi: usize, sendmsg: bool) {
    if ctx.objects.Objects[oi]._oSelFlag == 0 || crate::minitext::qtextflag(ctx) {
        return;
    }
    let book = ctx.objects.Objects[oi].clone();
    if book._oAnimFrame != book._oVar6 {
        obj_change_map_resync(ctx, book._oVar1, book._oVar2, book._oVar3, book._oVar4);
        for j in 0..ctx.objects.ActiveObjectCount as usize {
            let a = ctx.objects.ActiveObjects[j] as usize;
            sync_object_anim(ctx, a);
        }
    }
    ctx.objects.Objects[oi]._oAnimFrame = book._oVar6;
    if ctx.quests.Quests[Q_SCHAMB as usize]._qactive == QUEST_INIT {
        ctx.quests.Quests[Q_SCHAMB as usize]._qactive = QUEST_ACTIVE;
        ctx.quests.Quests[Q_SCHAMB as usize]._qlog = true;
    }
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let textdef = match ctx.players.Players[me]._pClass {
        HeroClass::Warrior => TEXT_BONER,
        HeroClass::Rogue => TEXT_RBONER,
        HeroClass::Sorcerer => TEXT_MBONER,
        HeroClass::Monk => TEXT_HBONER,
        HeroClass::Bard => TEXT_RBONER,
        HeroClass::Barbarian => TEXT_BONER,
    };
    if sendmsg {
        ctx.quests.Quests[Q_SCHAMB as usize]._qmsg = textdef;
        crate::msg::net_send_cmd_quest(ctx, true, Q_SCHAMB as usize);
        send_operate(ctx, false, book.position);
        crate::minitext::init_q_text_msg(ctx, textdef);
    }
}

/// Original: `OperateChest` (objects.cpp).
// @port objects.cpp|devilution::OperateChest(const Player &player, Object &chest, bool sendLootMsg) sha=4d40314b9c42
fn operate_chest(ctx: &mut Ctx, pnum: usize, oi: usize, send_loot_msg: bool) {
    use crate::effects_data::*;
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    let pos = ctx.objects.Objects[oi].position;
    crate::effects::play_sfx_loc(ctx, IS_CHEST, pos, true);
    ctx.objects.Objects[oi]._oSelFlag = 0;
    ctx.objects.Objects[oi]._oAnimFrame += 2;
    let chest = ctx.objects.Objects[oi].clone();
    ctx.rng.set_rnd_seed(chest._oRndSeed);
    if ctx.gendung.setlevel {
        for _ in 0..chest._oVar1 {
            crate::items::create_rnd_item(ctx, pos, true, send_loot_msg, false);
        }
    } else {
        for _ in 0..chest._oVar1 {
            if chest._oVar2 != 0 {
                crate::items::create_rnd_item(ctx, pos, false, send_loot_msg, false);
            } else {
                crate::items::create_rnd_useful(ctx, pos, send_loot_msg);
            }
        }
    }
    if chest.is_trapped_chest() {
        let ptile = ctx.players.Players[pnum].position.tile;
        let mdir = crate::engine::get_direction(pos, ptile);
        let mtype = match chest._oVar4 {
            0 => MissileID::Arrow,
            1 => MissileID::FireArrow,
            2 => MissileID::Nova,
            3 => MissileID::RingOfFire,
            4 => MissileID::StealPotions,
            5 => MissileID::StealMana,
            _ => MissileID::Arrow,
        };
        crate::missiles::add_missile(ctx, pos, ptile, mdir, mtype, TARGET_PLAYERS, -1, 0, 0, None);
        ctx.objects.Objects[oi]._oTrapFlag = false;
    }
    if is_my_player(ctx, pnum) {
        send_operate(ctx, false, pos);
    }
}

/// Original: `OperateMushroomPatch` (objects.cpp).
// @port objects.cpp|devilution::OperateMushroomPatch(const Player &player, Object &mushroomPatch) sha=d21e920aef8f
fn operate_mushroom_patch(ctx: &mut Ctx, pnum: usize, oi: usize) {
    use crate::effects_data::*;
    if ctx.items.ActiveItemCount as usize >= crate::items::MAXITEMS {
        return;
    }
    if ctx.quests.Quests[Q_MUSHROOM as usize]._qactive != QUEST_ACTIVE {
        if is_my_player(ctx, pnum) {
            crate::player::player_say(ctx, pnum, HeroSpeech::ICantUseThisYet);
        }
        return;
    }
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    ctx.objects.Objects[oi]._oSelFlag = 0;
    ctx.objects.Objects[oi]._oAnimFrame += 1;
    let p = ctx.objects.Objects[oi].position;
    crate::effects::play_sfx_loc(ctx, IS_CHEST, p, true);
    let pos = crate::items::get_super_item_loc(ctx, p);
    if is_my_player(ctx, pnum) {
        crate::items::spawn_quest_item(ctx, IDI_MUSHROOM, pos, 0, 0, true);
        ctx.quests.Quests[Q_MUSHROOM as usize]._qvar1 = QS_MUSHSPAWNED as u8;
        crate::msg::net_send_cmd_quest(ctx, true, Q_MUSHROOM as usize);
        send_operate(ctx, false, p);
    }
}

/// Original: `OperateInnSignChest` (objects.cpp).
// @port objects.cpp|devilution::OperateInnSignChest(const Player &player, Object &questContainer, bool sendmsg) sha=cf9308c730f6
fn operate_inn_sign_chest(ctx: &mut Ctx, pnum: usize, oi: usize, sendmsg: bool) {
    use crate::effects_data::*;
    if ctx.items.ActiveItemCount as usize >= crate::items::MAXITEMS {
        return;
    }
    if ctx.quests.Quests[Q_LTBANNER as usize]._qvar1 != 2 {
        if is_my_player(ctx, pnum) {
            crate::player::player_say(ctx, pnum, HeroSpeech::ICantOpenThisYet);
        }
        return;
    }
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    ctx.objects.Objects[oi]._oSelFlag = 0;
    ctx.objects.Objects[oi]._oAnimFrame += 2;
    let p = ctx.objects.Objects[oi].position;
    crate::effects::play_sfx_loc(ctx, IS_CHEST, p, true);
    if sendmsg {
        let pos = crate::items::get_super_item_loc(ctx, p);
        crate::items::spawn_quest_item(ctx, IDI_BANNER, pos, 0, 0, true);
        send_operate(ctx, true, p);
    }
}

/// Original: `OperateSlainHero` (objects.cpp).
// @port objects.cpp|devilution::OperateSlainHero(const Player &player, Object &corpse, bool sendmsg) sha=76592e26ec1d
fn operate_slain_hero(ctx: &mut Ctx, pnum: usize, oi: usize, sendmsg: bool) {
    use crate::items::{create_magic_armor, create_magic_weapon, create_spell_book};
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    ctx.objects.Objects[oi]._oSelFlag = 0;
    let (p, seed) = (ctx.objects.Objects[oi].position, ctx.objects.Objects[oi]._oRndSeed);
    ctx.rng.set_rnd_seed(seed);
    match ctx.players.Players[pnum]._pClass {
        HeroClass::Warrior => create_magic_armor(ctx, p, ItemType::HeavyArmor, ICURS_BREAST_PLATE as i32, sendmsg, false),
        HeroClass::Rogue => create_magic_weapon(ctx, p, ItemType::Bow, ICURS_LONG_BATTLE_BOW as i32, sendmsg, false),
        HeroClass::Sorcerer => create_spell_book(ctx, p, SpellID::Lightning, sendmsg, false),
        HeroClass::Monk => create_magic_weapon(ctx, p, ItemType::Staff, ICURS_WAR_STAFF as i32, sendmsg, false),
        HeroClass::Bard => create_magic_weapon(ctx, p, ItemType::Sword, ICURS_BASTARD_SWORD as i32, sendmsg, false),
        HeroClass::Barbarian => create_magic_weapon(ctx, p, ItemType::Axe, ICURS_BATTLE_AXE as i32, sendmsg, false),
    }
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    crate::player::player_say(ctx, me, HeroSpeech::RestInPeaceMyFriend);
    if sendmsg {
        send_operate(ctx, false, p);
    }
}

/// Original: `OperateTrapLever` (objects.cpp).
// @port objects.cpp|devilution::OperateTrapLever(Object &flameLever) sha=a0f6f6cbdb9b
fn operate_trap_lever(ctx: &mut Ctx, oi: usize) {
    use crate::effects_data::*;
    let lever = ctx.objects.Objects[oi].clone();
    crate::effects::play_sfx_loc(ctx, IS_LEVER, lever.position, true);
    if lever._oAnimFrame == 1 {
        ctx.objects.Objects[oi]._oAnimFrame = 2;
        for j in 0..ctx.objects.ActiveObjectCount as usize {
            let t = ctx.objects.ActiveObjects[j] as usize;
            let target = &mut ctx.objects.Objects[t];
            if target._otype as i32 == lever._oVar2 && target._oVar1 == lever._oVar1 {
                target._oVar2 = 1;
                target._oAnimFlag = 0;
            }
        }
        return;
    }
    ctx.objects.Objects[oi]._oAnimFrame -= 1;
    for j in 0..ctx.objects.ActiveObjectCount as usize {
        let t = ctx.objects.ActiveObjects[j] as usize;
        let target = &mut ctx.objects.Objects[t];
        if target._otype as i32 == lever._oVar2 && target._oVar1 == lever._oVar1 {
            target._oVar2 = 0;
            if target._oVar4 != 0 {
                target._oAnimFlag = 1;
            }
        }
    }
}

/// Original: `OperateSarcophagus` (objects.cpp).
// @port objects.cpp|devilution::OperateSarcophagus(Object &sarcophagus, bool sendMsg, bool sendLootMsg) sha=bab393227c60
fn operate_sarcophagus(ctx: &mut Ctx, oi: usize, send_msg: bool, send_loot_msg: bool) {
    use crate::effects_data::*;
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    let p = ctx.objects.Objects[oi].position;
    crate::effects::play_sfx_loc(ctx, IS_SARC, p, true);
    {
        let s = &mut ctx.objects.Objects[oi];
        s._oSelFlag = 0;
        s._oAnimFlag = 1;
        s._oAnimDelay = 3;
    }
    let s = ctx.objects.Objects[oi].clone();
    ctx.rng.set_rnd_seed(s._oRndSeed);
    if s._oVar1 <= 2 {
        crate::items::create_rnd_item(ctx, p, false, send_loot_msg, false);
    }
    if s._oVar1 >= 8 && s._oVar2 >= 0 {
        crate::monster::activate_skeleton(ctx, s._oVar2 as usize, p);
    }
    if send_msg {
        send_operate(ctx, false, p);
    }
}

/// Original: `OperatePedestal` (objects.cpp).
// @port objects.cpp|devilution::OperatePedestal(Player &player, Object &pedestal, bool sendmsg) sha=1660b0f4d438
fn operate_pedestal(ctx: &mut Ctx, pnum: usize, oi: usize, sendmsg: bool) {
    use crate::effects_data::*;
    if ctx.items.ActiveItemCount as usize >= crate::items::MAXITEMS {
        return;
    }
    if ctx.objects.Objects[oi]._oVar6 == 3 || (sendmsg && !crate::inv::remove_inventory_item_by_id(ctx, pnum, IDI_BLDSTONE)) {
        return;
    }
    let p = ctx.objects.Objects[oi].position;
    if sendmsg {
        send_operate(ctx, false, p);
        if ctx.init.gb_is_multiplayer {
            // Store added stones to pedestal in qvar2, cause we get only one CMD_OPERATEOBJ from DeltaLoadLevel even if we add multiple stones
            ctx.quests.Quests[Q_BLOOD as usize]._qvar2 += 1;
            crate::msg::net_send_cmd_quest(ctx, true, Q_BLOOD as usize);
        }
    }
    ctx.objects.Objects[oi]._oAnimFrame += 1;
    ctx.objects.Objects[oi]._oVar6 += 1;
    let sp = ctx.gendung.SetPiece;
    let sp_world = sp.position.mega_to_world();
    let v6 = ctx.objects.Objects[oi]._oVar6;
    if v6 == 1 {
        crate::effects::play_sfx_loc(ctx, LS_PUDDLE, p, true);
        obj_change_map(ctx, sp.position.x, sp.position.y + 3, sp.position.x + 2, sp.position.y + 7);
        if sendmsg {
            crate::items::spawn_quest_item(ctx, IDI_BLDSTONE, sp_world + Displacement::new(3, 10), 0, 1, true);
        }
    }
    if v6 == 2 {
        crate::effects::play_sfx_loc(ctx, LS_PUDDLE, p, true);
        obj_change_map(ctx, sp.position.x + 6, sp.position.y + 3, sp.position.x + sp.size.width, sp.position.y + 7);
        if sendmsg {
            crate::items::spawn_quest_item(ctx, IDI_BLDSTONE, sp_world + Displacement::new(15, 10), 0, 1, true);
        }
    }
    if v6 == 3 {
        crate::effects::play_sfx_loc(ctx, LS_BLODSTAR, p, true);
        let o = ctx.objects.Objects[oi].clone();
        obj_change_map(ctx, o._oVar1, o._oVar2, o._oVar3, o._oVar4);
        load_map_objects(ctx, "levels\\l2data\\blood2.dun", sp_world, Rectangle::default(), 0);
        if sendmsg {
            crate::items::spawn_unique(ctx, UITEM_ARMOFVAL, sp_world + Displacement::new(9, 3), None, true, true);
        }
        ctx.objects.Objects[oi]._oSelFlag = 0;
    }
}

/// Original: `OperateShrineMysterious` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineMysterious(Player &player) sha=c7d94a470d42
fn operate_shrine_mysterious(ctx: &mut Ctx, pnum: usize) {
    use crate::player::{modify_plr_dex, modify_plr_mag, modify_plr_str, modify_plr_vit};
    if !is_my_player(ctx, pnum) {
        return;
    }
    modify_plr_str(ctx, pnum, -1);
    modify_plr_mag(ctx, pnum, -1);
    modify_plr_dex(ctx, pnum, -1);
    modify_plr_vit(ctx, pnum, -1);
    match rnd(ctx, 4) {
        0 => modify_plr_str(ctx, pnum, 6),
        1 => modify_plr_mag(ctx, pnum, 6),
        2 => modify_plr_dex(ctx, pnum, 6),
        3 => modify_plr_vit(ctx, pnum, 6),
        _ => {}
    }
    crate::player::check_stats(&mut ctx.players.Players[pnum]);
    crate::items::calc_plr_inv(ctx, pnum, true);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    diablo_msg(ctx, EMSG_SHRINE_MYSTERIOUS);
}

/// Original: `OperateShrineHidden` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineHidden(Player &player) sha=61ed8811bcce
fn operate_shrine_hidden(ctx: &mut Ctx, pnum: usize) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    let damageable = |item: &crate::items::Item| !item.is_empty() && item._iMaxDur != crate::items::DUR_INDESTRUCTIBLE && item._iMaxDur != 0;
    let cnt = ctx.players.Players[pnum].InvBody.iter().filter(|i| !i.is_empty()).count();
    if cnt > 0 {
        for item in ctx.players.Players[pnum].InvBody.iter_mut() {
            if damageable(item) {
                item._iDurability += 10;
                item._iMaxDur += 10;
                if item._iDurability > item._iMaxDur {
                    item._iDurability = item._iMaxDur;
                }
            }
        }
        loop {
            let cnt = ctx.players.Players[pnum].InvBody.iter().filter(|i| damageable(i)).count();
            if cnt == 0 {
                break;
            }
            let r = rnd(ctx, NUM_INVLOC as i32) as usize;
            let item = &mut ctx.players.Players[pnum].InvBody[r];
            if item.is_empty() || item._iMaxDur == crate::items::DUR_INDESTRUCTIBLE || item._iMaxDur == 0 {
                continue;
            }
            item._iDurability -= 20;
            item._iMaxDur -= 20;
            if item._iDurability <= 0 {
                item._iDurability = 1;
            }
            if item._iMaxDur <= 0 {
                item._iMaxDur = 1;
            }
            break;
        }
    }
    diablo_msg(ctx, EMSG_SHRINE_HIDDEN);
}

/// Original: `OperateShrineGloomy` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineGloomy(Player &player) sha=60701e64e16b
fn operate_shrine_gloomy(ctx: &mut Ctx, pnum: usize) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    // Increment armor class by 2 and decrements max damage by 1.
    for_player_items(ctx, pnum, &mut |item| match item._itype {
        ItemType::Sword | ItemType::Axe | ItemType::Bow | ItemType::Mace | ItemType::Staff => {
            item._iMaxDam -= 1;
            if item._iMaxDam < item._iMinDam {
                item._iMaxDam = item._iMinDam;
            }
        }
        ItemType::Shield | ItemType::Helm | ItemType::LightArmor | ItemType::MediumArmor | ItemType::HeavyArmor => {
            item._iAC += 2;
        }
        _ => {}
    });
    crate::items::calc_plr_inv(ctx, pnum, true);
    diablo_msg(ctx, EMSG_SHRINE_GLOOMY);
}

/// Original: `OperateShrineWeird` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineWeird(Player &player) sha=423a792adb6a
fn operate_shrine_weird(ctx: &mut Ctx, pnum: usize) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    {
        let player = &mut ctx.players.Players[pnum];
        for loc in [INVLOC_HAND_LEFT, INVLOC_HAND_RIGHT] {
            let item = &mut player.InvBody[loc as usize];
            if !item.is_empty() && item._itype != ItemType::Shield {
                item._iMaxDam += 1;
            }
        }
    }
    for_inventory_items(ctx, pnum, &mut |item| {
        if matches!(item._itype, ItemType::Sword | ItemType::Axe | ItemType::Bow | ItemType::Mace | ItemType::Staff) {
            item._iMaxDam += 1;
        }
    });
    crate::items::calc_plr_inv(ctx, pnum, true);
    diablo_msg(ctx, EMSG_SHRINE_WEIRD);
}

/// Original: `OperateShrineMagical` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineMagical(const Player &player) sha=8b76b32b9036
fn operate_shrine_magical(ctx: &mut Ctx, pnum: usize) {
    let (t, d) = (ctx.players.Players[pnum].position.tile, ctx.players.Players[pnum]._pdir);
    let lt = ctx.gendung.leveltype as i8 as i32;
    crate::missiles::add_missile(ctx, t, t, d, MissileID::ManaShield, TARGET_MONSTERS, pnum as i32, 0, 2 * lt, None);
    if !is_my_player(ctx, pnum) {
        return;
    }
    diablo_msg(ctx, EMSG_SHRINE_MAGICAL);
}

/// Original: `OperateShrineStone` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineStone(Player &player) sha=0b05cc30b86c
fn operate_shrine_stone(ctx: &mut Ctx, pnum: usize) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    for_player_items(ctx, pnum, &mut |item| {
        if item._itype == ItemType::Staff {
            item._iCharges = item._iMaxCharges;
        }
    });
    crate::items::calc_plr_inv(ctx, pnum, true);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    diablo_msg(ctx, EMSG_SHRINE_STONE);
}

/// Original: `OperateShrineReligious` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineReligious(Player &player) sha=11e83cbd196e
fn operate_shrine_religious(ctx: &mut Ctx, pnum: usize) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    for_player_items(ctx, pnum, &mut |item| item._iDurability = item._iMaxDur);
    diablo_msg(ctx, EMSG_SHRINE_RELIGIOUS);
}

/// Original: `OperateShrineEnchanted` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineEnchanted(Player &player) sha=19c3be0ba0d5
fn operate_shrine_enchanted(ctx: &mut Ctx, pnum: usize) {
    use crate::player::MaxSpellLevel;
    if !is_my_player(ctx, pnum) {
        return;
    }
    let mut cnt = 0;
    let mut spell: u64 = 1;
    let max_spells: u8 = if ctx.init.gb_is_hellfire { crate::items::MAX_SPELLS as u8 } else { 37 };
    let spells = ctx.players.Players[pnum]._pMemSpells;
    for _ in 0..max_spells {
        if (spell & spells) != 0 {
            cnt += 1;
        }
        spell = spell.wrapping_mul(2);
    }
    if cnt > 1 {
        let mut spell_to_reduce;
        loop {
            spell_to_reduce = rnd(ctx, max_spells as i32) + 1;
            if (ctx.players.Players[pnum]._pMemSpells & crate::spells::get_spell_bitmask(SpellID::from_repr(spell_to_reduce as i8).unwrap_or(SpellID::Null))) != 0 {
                break;
            }
        }
        spell = 1;
        for j in SpellID::Firebolt as i8 as u8..max_spells {
            let lvl = ctx.players.Players[pnum]._pSplLvl[j as usize];
            if (ctx.players.Players[pnum]._pMemSpells & spell) != 0 && lvl < MaxSpellLevel && j as i32 != spell_to_reduce {
                let new_spell_level = lvl + 1;
                ctx.players.Players[pnum]._pSplLvl[j as usize] = new_spell_level;
                crate::msg::net_send_cmd_param2(ctx, true, CMD_CHANGE_SPELL_LEVEL, j as u16, new_spell_level as u16);
            }
            spell = spell.wrapping_mul(2);
        }
        let lvl = ctx.players.Players[pnum]._pSplLvl[spell_to_reduce as usize];
        if lvl > 0 {
            let new_spell_level = lvl - 1;
            ctx.players.Players[pnum]._pSplLvl[spell_to_reduce as usize] = new_spell_level;
            crate::msg::net_send_cmd_param2(ctx, true, CMD_CHANGE_SPELL_LEVEL, spell_to_reduce as u16, new_spell_level as u16);
        }
        if is_my_player(ctx, pnum) {
            refresh_inventory_stat_cache(ctx, pnum);
        }
    }
    diablo_msg(ctx, EMSG_SHRINE_ENCHANTED);
}

/// Original: `OperateShrineThaumaturgic` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineThaumaturgic(const Player &player) sha=14606163d5c1
fn operate_shrine_thaumaturgic(ctx: &mut Ctx, pnum: usize) {
    for j in 0..ctx.objects.ActiveObjectCount as usize {
        let o = ctx.objects.ActiveObjects[j] as usize;
        if ctx.objects.Objects[o].is_chest() && ctx.objects.Objects[o]._oSelFlag == 0 {
            ctx.objects.Objects[o]._oRndSeed = ctx.rng.advance_rnd_seed() as u32;
            ctx.objects.Objects[o]._oSelFlag = 1;
            ctx.objects.Objects[o]._oAnimFrame -= 2;
        }
    }
    if !is_my_player(ctx, pnum) {
        return;
    }
    diablo_msg(ctx, EMSG_SHRINE_THAUMATURGIC);
}

/// Original: `OperateShrineCostOfWisdom` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineCostOfWisdom(Player &player, SpellID spellId, diablo_message message) sha=b900c2b0cc28
fn operate_shrine_cost_of_wisdom(ctx: &mut Ctx, pnum: usize, spell_id: SpellID, message: diablo_message) {
    use crate::player::MaxSpellLevel;
    if !is_my_player(ctx, pnum) {
        return;
    }
    ctx.players.Players[pnum]._pMemSpells |= crate::spells::get_spell_bitmask(spell_id);
    let si = spell_id as i8 as usize;
    let cur_spell_level = ctx.players.Players[pnum]._pSplLvl[si];
    if cur_spell_level < MaxSpellLevel {
        let new_spell_level = (cur_spell_level + 2).min(MaxSpellLevel);
        ctx.players.Players[pnum]._pSplLvl[si] = new_spell_level;
        crate::msg::net_send_cmd_param2(ctx, true, CMD_CHANGE_SPELL_LEVEL, spell_id as i8 as u16, new_spell_level as u16);
    }
    if is_my_player(ctx, pnum) {
        refresh_inventory_stat_cache(ctx, pnum);
    }
    let player = &mut ctx.players.Players[pnum];
    let t = (player._pMaxManaBase as u32 / 10) as i32;
    let v1 = player._pMana - player._pManaBase;
    let v2 = player._pMaxMana - player._pMaxManaBase;
    player._pManaBase -= t;
    player._pMana -= t;
    player._pMaxMana -= t;
    player._pMaxManaBase -= t;
    if player._pMana >> 6 <= 0 {
        player._pMana = v1;
        player._pManaBase = 0;
    }
    if player._pMaxMana >> 6 <= 0 {
        player._pMaxMana = v2;
        player._pMaxManaBase = 0;
    }
    crate::engine::backbuffer_state::redraw_everything(ctx);
    diablo_msg(ctx, message);
}

/// Original: `OperateShrineCryptic` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineCryptic(Player &player) sha=f22efa676af6
fn operate_shrine_cryptic(ctx: &mut Ctx, pnum: usize) {
    let (t, d) = (ctx.players.Players[pnum].position.tile, ctx.players.Players[pnum]._pdir);
    let lt = ctx.gendung.leveltype as i8 as i32;
    crate::missiles::add_missile(ctx, t, t, d, MissileID::Nova, TARGET_MONSTERS, pnum as i32, 0, 2 * lt, None);
    if !is_my_player(ctx, pnum) {
        return;
    }
    let player = &mut ctx.players.Players[pnum];
    player._pMana = player._pMaxMana;
    player._pManaBase = player._pMaxManaBase;
    diablo_msg(ctx, EMSG_SHRINE_CRYPTIC);
    crate::engine::backbuffer_state::redraw_everything(ctx);
}

/// Original: `OperateShrineEldritch` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineEldritch(Player &player) sha=f07c4c4c246e
fn operate_shrine_eldritch(ctx: &mut Ctx, pnum: usize) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    let rejuv = item_misc_id_idx(IMISC_REJUV);
    let full_rejuv = item_misc_id_idx(IMISC_FULLREJUV);
    let n = ctx.players.Players[pnum]._pNumInv as usize;
    let total = n + ctx.players.Players[pnum].SpdList.len();
    for k in 0..total {
        let mut item = {
            let p = &ctx.players.Players[pnum];
            if k < n { p.InvList[k].clone() } else { p.SpdList[k - n].clone() }
        };
        if item.is_empty() || item._itype != ItemType::Misc {
            continue;
        }
        let idx = if matches!(item._iMiscId, IMISC_HEAL | IMISC_MANA) {
            rejuv
        } else if matches!(item._iMiscId, IMISC_FULLHEAL | IMISC_FULLMANA) {
            full_rejuv
        } else {
            continue;
        };
        // Reinitializing the item zeroes out the seed, we save and restore here to avoid triggering false
        // positives on duplicated item checks (e.g. when picking up the item).
        let seed = item._iSeed;
        crate::items::initialize_item(ctx, &mut item, idx);
        item._iSeed = seed;
        item._iStatFlag = true;
        let p = &mut ctx.players.Players[pnum];
        if k < n {
            p.InvList[k] = item;
        } else {
            p.SpdList[k - n] = item;
        }
    }
    crate::engine::backbuffer_state::redraw_everything(ctx);
    diablo_msg(ctx, EMSG_SHRINE_ELDRITCH);
}

/// Original: `OperateShrineEerie` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineEerie(Player &player) sha=a36a90fb064e
fn operate_shrine_eerie(ctx: &mut Ctx, pnum: usize) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    crate::player::modify_plr_mag(ctx, pnum, 2);
    crate::player::check_stats(&mut ctx.players.Players[pnum]);
    crate::items::calc_plr_inv(ctx, pnum, true);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    diablo_msg(ctx, EMSG_SHRINE_EERIE);
}

// ---------------------------------------------------------------------------------------------
// objects.cpp, part 4: remaining shrines, OperateShrine and the item containers.
// ---------------------------------------------------------------------------------------------

/// Original: `OperateShrineDivine` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineDivine(Player &player, Point spawnPosition) sha=c975ca5a9ff3
fn operate_shrine_divine(ctx: &mut Ctx, pnum: usize, spawn_position: Point) {
    use crate::items::create_type_item;
    if !is_my_player(ctx, pnum) {
        return;
    }
    if ctx.gendung.currlevel < 4 {
        create_type_item(ctx, spawn_position, false, ItemType::Misc, IMISC_FULLMANA as i32, false, false, true);
        create_type_item(ctx, spawn_position, false, ItemType::Misc, IMISC_FULLHEAL as i32, false, false, true);
    } else {
        create_type_item(ctx, spawn_position, false, ItemType::Misc, IMISC_FULLREJUV as i32, false, false, true);
        create_type_item(ctx, spawn_position, false, ItemType::Misc, IMISC_FULLREJUV as i32, false, false, true);
    }
    let player = &mut ctx.players.Players[pnum];
    player._pMana = player._pMaxMana;
    player._pManaBase = player._pMaxManaBase;
    player._pHitPoints = player._pMaxHP;
    player._pHPBase = player._pMaxHPBase;
    crate::engine::backbuffer_state::redraw_everything(ctx);
    diablo_msg(ctx, EMSG_SHRINE_DIVINE);
}

/// Original: `OperateShrineHoly` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineHoly(const Player &player) sha=2b74663b249a
fn operate_shrine_holy(ctx: &mut Ctx, pnum: usize) {
    let t = ctx.players.Players[pnum].position.tile;
    let lt = ctx.gendung.leveltype as i8 as i32;
    crate::missiles::add_missile(ctx, t, Point::new(0, 0), Direction::South, MissileID::Phasing, TARGET_MONSTERS, pnum as i32, 0, 2 * lt, None);
    if !is_my_player(ctx, pnum) {
        return;
    }
    diablo_msg(ctx, EMSG_SHRINE_HOLY);
}

/// Original: `OperateShrineSpiritual` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineSpiritual(Player &player) sha=3d37fe0e204f
fn operate_shrine_spiritual(ctx: &mut Ctx, pnum: usize) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    let lt = ctx.gendung.leveltype as i8 as i32;
    for k in 0..ctx.players.Players[pnum].InvGrid.len() {
        if ctx.players.Players[pnum].InvGrid[k] == 0 {
            let value = 5 * lt + rnd(ctx, 10 * lt);
            let n = ctx.players.Players[pnum]._pNumInv as usize;
            let mut gold_item = ctx.players.Players[pnum].InvList[n].clone();
            crate::items::make_gold_stack(ctx, &mut gold_item, value);
            let player = &mut ctx.players.Players[pnum];
            let v = gold_item._ivalue;
            player.InvList[n] = gold_item;
            player._pNumInv += 1;
            player.InvGrid[k] = player._pNumInv as i8;
            player._pGold += v;
        }
    }
    diablo_msg(ctx, EMSG_SHRINE_SPIRITUAL);
}

/// Original: `OperateShrineSpooky` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineSpooky(const Player &player) sha=7ba073d032d4
fn operate_shrine_spooky(ctx: &mut Ctx, pnum: usize) {
    if is_my_player(ctx, pnum) {
        diablo_msg(ctx, EMSG_SHRINE_SPOOKY1);
        return;
    }
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let my = &mut ctx.players.Players[me];
    my._pHitPoints = my._pMaxHP;
    my._pHPBase = my._pMaxHPBase;
    my._pMana = my._pMaxMana;
    my._pManaBase = my._pMaxManaBase;
    crate::engine::backbuffer_state::redraw_everything(ctx);
    diablo_msg(ctx, EMSG_SHRINE_SPOOKY2);
}

/// Shared body of the shrines that raise one attribute by 2.
fn operate_shrine_stat(ctx: &mut Ctx, pnum: usize, modify: fn(&mut Ctx, usize, i32), msg: diablo_message) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    modify(ctx, pnum, 2);
    crate::player::check_stats(&mut ctx.players.Players[pnum]);
    crate::items::calc_plr_inv(ctx, pnum, true);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    diablo_msg(ctx, msg);
}

/// Original: `OperateShrineAbandoned` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineAbandoned(Player &player) sha=32966c64948e
fn operate_shrine_abandoned(ctx: &mut Ctx, pnum: usize) {
    operate_shrine_stat(ctx, pnum, crate::player::modify_plr_dex, EMSG_SHRINE_ABANDONED);
}

/// Original: `OperateShrineCreepy` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineCreepy(Player &player) sha=89d88a7705c0
fn operate_shrine_creepy(ctx: &mut Ctx, pnum: usize) {
    operate_shrine_stat(ctx, pnum, crate::player::modify_plr_str, EMSG_SHRINE_CREEPY);
}

/// Original: `OperateShrineQuiet` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineQuiet(Player &player) sha=14d2267324e9
fn operate_shrine_quiet(ctx: &mut Ctx, pnum: usize) {
    operate_shrine_stat(ctx, pnum, crate::player::modify_plr_vit, EMSG_SHRINE_QUIET);
}

/// Original: `OperateShrineSecluded` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineSecluded(const Player &player) sha=1c208d6b92c2
fn operate_shrine_secluded(ctx: &mut Ctx, pnum: usize) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    for x in 0..DMAXX as i32 {
        for y in 0..DMAXY as i32 {
            crate::automap::update_automap_explorer(ctx, Point::new(x, y), crate::lighting::MAP_EXP_SHRINE);
        }
    }
    diablo_msg(ctx, EMSG_SHRINE_SECLUDED);
}

/// Original: `OperateShrineGlimmering` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineGlimmering(Player &player) sha=50c81d52718a
fn operate_shrine_glimmering(ctx: &mut Ctx, pnum: usize) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    for_player_items(ctx, pnum, &mut |item| {
        if item._iMagical != ITEM_QUALITY_NORMAL && !item._iIdentified {
            item._iIdentified = true;
        }
    });
    crate::items::calc_plr_inv(ctx, pnum, true);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    diablo_msg(ctx, EMSG_SHRINE_GLIMMERING);
}

/// Original: `OperateShrineTainted` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineTainted(const Player &player) sha=1f99cff1a454
fn operate_shrine_tainted(ctx: &mut Ctx, pnum: usize) {
    use crate::player::{modify_plr_dex, modify_plr_mag, modify_plr_str, modify_plr_vit};
    if is_my_player(ctx, pnum) {
        diablo_msg(ctx, EMSG_SHRINE_TAINTED1);
        return;
    }
    let r = rnd(ctx, 4);
    let v = |k: i32| if r == k { 1 } else { -1 };
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    modify_plr_str(ctx, me, v(0));
    modify_plr_mag(ctx, me, v(1));
    modify_plr_dex(ctx, me, v(2));
    modify_plr_vit(ctx, me, v(3));
    crate::player::check_stats(&mut ctx.players.Players[me]);
    crate::items::calc_plr_inv(ctx, me, true);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    diablo_msg(ctx, EMSG_SHRINE_TAINTED2);
}

/// Original: `OperateShrineOily` (objects.cpp): raises primary stats but spawns a firewall.
// @port objects.cpp|devilution::OperateShrineOily(Player &player, Point spawnPosition) sha=45a9233ff65a
fn operate_shrine_oily(ctx: &mut Ctx, pnum: usize, spawn_position: Point) {
    use crate::player::{modify_plr_dex, modify_plr_mag, modify_plr_str, modify_plr_vit};
    if !is_my_player(ctx, pnum) {
        return;
    }
    match ctx.players.Players[pnum]._pClass {
        HeroClass::Warrior => modify_plr_str(ctx, pnum, 2),
        HeroClass::Rogue => modify_plr_dex(ctx, pnum, 2),
        HeroClass::Sorcerer => modify_plr_mag(ctx, pnum, 2),
        HeroClass::Barbarian => modify_plr_vit(ctx, pnum, 2),
        HeroClass::Monk => {
            modify_plr_str(ctx, pnum, 1);
            modify_plr_dex(ctx, pnum, 1);
        }
        HeroClass::Bard => {
            modify_plr_dex(ctx, pnum, 1);
            modify_plr_mag(ctx, pnum, 1);
        }
    }
    crate::player::check_stats(&mut ctx.players.Players[pnum]);
    crate::items::calc_plr_inv(ctx, pnum, true);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    let (t, d) = (ctx.players.Players[pnum].position.tile, ctx.players.Players[pnum]._pdir);
    let cl = ctx.gendung.currlevel as i32;
    crate::missiles::add_missile(ctx, spawn_position, t, d, MissileID::FireWall, TARGET_PLAYERS, -1, 2 * cl + 2, 0, None);
    diablo_msg(ctx, EMSG_SHRINE_OILY);
}

/// Original: `OperateShrineGlowing` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineGlowing(Player &player) sha=d3e95cc1a9bc
fn operate_shrine_glowing(ctx: &mut Ctx, pnum: usize) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    // Add 0-5 points to Magic (0.1% of the players XP)
    let exp = ctx.players.Players[pnum]._pExperience;
    crate::player::modify_plr_mag(ctx, pnum, (exp / 1000).min(5) as i32);
    // Take 5% of the players experience to offset the bonus, unless they're very low level in which case take all their experience.
    let player = &mut ctx.players.Players[pnum];
    if player._pExperience > 5000 {
        player._pExperience = (player._pExperience as f64 * 0.95) as u32;
    } else {
        player._pExperience = 0;
    }
    crate::player::check_stats(&mut ctx.players.Players[pnum]);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    diablo_msg(ctx, EMSG_SHRINE_GLOWING);
}

/// Original: `OperateShrineMendicant` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineMendicant(Player &player) sha=9483b4800fab
fn operate_shrine_mendicant(ctx: &mut Ctx, pnum: usize) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    let gold = ctx.players.Players[pnum]._pGold / 2;
    let lvl = ctx.players.Players[pnum]._pLevel as i32;
    crate::player::add_plr_experience(ctx, pnum, lvl, gold);
    crate::stores::take_plrs_money(ctx, gold);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    diablo_msg(ctx, EMSG_SHRINE_MENDICANT);
}

/// Original: `OperateShrineSparkling` (objects.cpp): grants experience and triggers a magic trap.
// @port objects.cpp|devilution::OperateShrineSparkling(Player &player, Point spawnPosition) sha=d6d92a702b4e
fn operate_shrine_sparkling(ctx: &mut Ctx, pnum: usize, spawn_position: Point) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    let lvl = ctx.players.Players[pnum]._pLevel as i32;
    let cl = ctx.gendung.currlevel as i32;
    crate::player::add_plr_experience(ctx, pnum, lvl, 1000 * cl);
    let (t, d) = (ctx.players.Players[pnum].position.tile, ctx.players.Players[pnum]._pdir);
    crate::missiles::add_missile(ctx, spawn_position, t, d, MissileID::FlashBottom, TARGET_PLAYERS, -1, 3 * cl + 2, 0, None);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    diablo_msg(ctx, EMSG_SHRINE_SPARKLING);
}

/// Original: `OperateShrineTown` (objects.cpp): spawns a town portal near the active player.
// @port objects.cpp|devilution::OperateShrineTown(const Player &player, Point spawnPosition) sha=be4793918647
fn operate_shrine_town(ctx: &mut Ctx, pnum: usize, spawn_position: Point) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    let (t, d) = (ctx.players.Players[pnum].position.tile, ctx.players.Players[pnum]._pdir);
    crate::missiles::add_missile(ctx, spawn_position, t, d, MissileID::TownPortal, TARGET_MONSTERS, pnum as i32, 0, 0, None);
    diablo_msg(ctx, EMSG_SHRINE_TOWN);
}

/// Original: `OperateShrineShimmering` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineShimmering(Player &player) sha=38922005586d
fn operate_shrine_shimmering(ctx: &mut Ctx, pnum: usize) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    let player = &mut ctx.players.Players[pnum];
    player._pMana = player._pMaxMana;
    player._pManaBase = player._pMaxManaBase;
    crate::engine::backbuffer_state::redraw_everything(ctx);
    diablo_msg(ctx, EMSG_SHRINE_SHIMMERING);
}

/// Original: `OperateShrineSolar` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineSolar(Player &player) sha=c9a9adb678d0
fn operate_shrine_solar(ctx: &mut Ctx, pnum: usize) {
    use crate::player::{modify_plr_dex, modify_plr_mag, modify_plr_str, modify_plr_vit};
    if !is_my_player(ctx, pnum) {
        return;
    }
    let hour = crate::platform::win32::localtime_hms(ctx.platform.time()).map(|(h, _, _)| h as i32).unwrap_or(20);
    if hour >= 20 || hour < 4 {
        diablo_msg(ctx, EMSG_SHRINE_SOLAR4);
        modify_plr_vit(ctx, pnum, 2);
    } else if hour >= 18 {
        diablo_msg(ctx, EMSG_SHRINE_SOLAR3);
        modify_plr_mag(ctx, pnum, 2);
    } else if hour >= 12 {
        diablo_msg(ctx, EMSG_SHRINE_SOLAR2);
        modify_plr_str(ctx, pnum, 2);
    } else {
        // 4:00 to 11:59
        diablo_msg(ctx, EMSG_SHRINE_SOLAR1);
        modify_plr_dex(ctx, pnum, 2);
    }
    crate::player::check_stats(&mut ctx.players.Players[pnum]);
    crate::items::calc_plr_inv(ctx, pnum, true);
    crate::engine::backbuffer_state::redraw_everything(ctx);
}

/// Original: `OperateShrineMurphys` (objects.cpp).
// @port objects.cpp|devilution::OperateShrineMurphys(Player &player) sha=b2ce4b9da69b
fn operate_shrine_murphys(ctx: &mut Ctx, pnum: usize) {
    if !is_my_player(ctx, pnum) {
        return;
    }
    let mut broke = false;
    for k in 0..ctx.players.Players[pnum].InvBody.len() {
        if !ctx.players.Players[pnum].InvBody[k].is_empty() && ctx.rng.flip_coin(3) {
            let item = &mut ctx.players.Players[pnum].InvBody[k];
            if item._iDurability != crate::items::DUR_INDESTRUCTIBLE && item._iDurability > 0 {
                item._iDurability /= 2;
                broke = true;
                break;
            }
        }
    }
    if !broke {
        let g = ctx.players.Players[pnum]._pGold / 3;
        crate::stores::take_plrs_money(ctx, g);
    }
    diablo_msg(ctx, EMSG_SHRINE_MURPHYS);
}

/// Original: `OperateShrine` (objects.cpp).
// @port objects.cpp|devilution::OperateShrine(Player &player, Object &shrine, _sfx_id sType) sha=e53b21f46114
fn operate_shrine(ctx: &mut Ctx, pnum: usize, oi: usize, s_type: crate::effects_data::SfxId) {
    use shrine_type::*;
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    if ctx.control.drop_gold_flag {
        crate::control::close_gold_drop(ctx);
        ctx.control.drop_gold_value = 0;
    }
    let seed = ctx.objects.Objects[oi]._oRndSeed;
    ctx.rng.set_rnd_seed(seed);
    ctx.objects.Objects[oi]._oSelFlag = 0;
    let pos = ctx.objects.Objects[oi].position;
    crate::effects::play_sfx_loc(ctx, s_type, pos, true);
    ctx.objects.Objects[oi]._oAnimFlag = 1;
    ctx.objects.Objects[oi]._oAnimDelay = 1;
    match ctx.objects.Objects[oi]._oVar1 {
        ShrineMysterious => operate_shrine_mysterious(ctx, pnum),
        ShrineHidden => operate_shrine_hidden(ctx, pnum),
        ShrineGloomy => operate_shrine_gloomy(ctx, pnum),
        ShrineWeird => operate_shrine_weird(ctx, pnum),
        ShrineMagical | ShrineMagicaL2 => operate_shrine_magical(ctx, pnum),
        ShrineStone => operate_shrine_stone(ctx, pnum),
        ShrineReligious => operate_shrine_religious(ctx, pnum),
        ShrineEnchanted => operate_shrine_enchanted(ctx, pnum),
        ShrineThaumaturgic => operate_shrine_thaumaturgic(ctx, pnum),
        ShrineFascinating => operate_shrine_cost_of_wisdom(ctx, pnum, SpellID::Firebolt, EMSG_SHRINE_FASCINATING),
        ShrineCryptic => operate_shrine_cryptic(ctx, pnum),
        ShrineEldritch => operate_shrine_eldritch(ctx, pnum),
        ShrineEerie => operate_shrine_eerie(ctx, pnum),
        ShrineDivine => operate_shrine_divine(ctx, pnum, pos),
        ShrineHoly => operate_shrine_holy(ctx, pnum),
        ShrineSacred => operate_shrine_cost_of_wisdom(ctx, pnum, SpellID::ChargedBolt, EMSG_SHRINE_SACRED),
        ShrineSpiritual => operate_shrine_spiritual(ctx, pnum),
        ShrineSpooky => operate_shrine_spooky(ctx, pnum),
        ShrineAbandoned => operate_shrine_abandoned(ctx, pnum),
        ShrineCreepy => operate_shrine_creepy(ctx, pnum),
        ShrineQuiet => operate_shrine_quiet(ctx, pnum),
        ShrineSecluded => operate_shrine_secluded(ctx, pnum),
        ShrineOrnate => operate_shrine_cost_of_wisdom(ctx, pnum, SpellID::HolyBolt, EMSG_SHRINE_ORNATE),
        ShrineGlimmering => operate_shrine_glimmering(ctx, pnum),
        ShrineTainted => operate_shrine_tainted(ctx, pnum),
        ShrineOily => operate_shrine_oily(ctx, pnum, pos),
        ShrineGlowing => operate_shrine_glowing(ctx, pnum),
        ShrineMendicant => operate_shrine_mendicant(ctx, pnum),
        ShrineSparkling => operate_shrine_sparkling(ctx, pnum, pos),
        ShrineTown => operate_shrine_town(ctx, pnum, pos),
        ShrineShimmering => operate_shrine_shimmering(ctx, pnum),
        ShrineSolar => operate_shrine_solar(ctx, pnum),
        ShrineMurphys => operate_shrine_murphys(ctx, pnum),
        _ => {}
    }
    if is_my_player(ctx, pnum) {
        send_operate(ctx, false, pos);
    }
}

/// Original: `OperateBookStand` (objects.cpp).
// @port objects.cpp|devilution::OperateBookStand(Object &bookStand, bool sendmsg, bool sendLootMsg) sha=ccec11339ed1
fn operate_book_stand(ctx: &mut Ctx, oi: usize, sendmsg: bool, send_loot_msg: bool) {
    use crate::effects_data::*;
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    let pos = ctx.objects.Objects[oi].position;
    crate::effects::play_sfx_loc(ctx, IS_ISCROL, pos, true);
    ctx.objects.Objects[oi]._oSelFlag = 0;
    ctx.objects.Objects[oi]._oAnimFrame += 2;
    let seed = ctx.objects.Objects[oi]._oRndSeed;
    ctx.rng.set_rnd_seed(seed);
    let m = if ctx.rng.flip_coin(5) { IMISC_BOOK } else { IMISC_SCROLL };
    crate::items::create_type_item(ctx, pos, false, ItemType::Misc, m as i32, send_loot_msg, false, false);
    if sendmsg {
        send_operate(ctx, false, pos);
    }
}

/// Original: `OperateBookcase` (objects.cpp).
// @port objects.cpp|devilution::OperateBookcase(Object &bookcase, bool sendmsg, bool sendLootMsg) sha=27d45c2f7140
fn operate_bookcase(ctx: &mut Ctx, oi: usize, sendmsg: bool, send_loot_msg: bool) {
    use crate::effects_data::*;
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    let pos = ctx.objects.Objects[oi].position;
    crate::effects::play_sfx_loc(ctx, IS_ISCROL, pos, true);
    ctx.objects.Objects[oi]._oSelFlag = 0;
    ctx.objects.Objects[oi]._oAnimFrame -= 2;
    let seed = ctx.objects.Objects[oi]._oRndSeed;
    ctx.rng.set_rnd_seed(seed);
    crate::items::create_type_item(ctx, pos, false, ItemType::Misc, IMISC_BOOK as i32, send_loot_msg, false, false);
    if crate::quests::is_quest_available(ctx, Q_ZHAR) {
        let z = crate::player::MAX_PLRS;
        let zhar = &ctx.monster.Monsters[z];
        if zhar.mode == MonsterMode::Stand // prevents playing the "angry" message for the second time if zhar got aggroed by losing vision and talking again
            && zhar.uniqueType == UniqueMonsterType::Zhar
            && zhar.activeForTicks == u8::MAX
            && zhar.hitPoints > 0
        {
            ctx.monster.Monsters[z].talkMsg = TEXT_ZHAR2;
            let d = ctx.monster.Monsters[z].direction;
            crate::monster::m_start_stand(ctx, z, d); // BUGFIX: first parameter in call to M_StartStand should be MAX_PLRS, not 0. (fixed)
            ctx.monster.Monsters[z].goal = MonsterGoal::Attack;
            if sendmsg {
                ctx.monster.Monsters[z].mode = MonsterMode::Talk;
            }
        }
    }
    if sendmsg {
        send_operate(ctx, false, pos);
    }
}

/// Original: `OperateDecapitatedBody` (objects.cpp).
// @port objects.cpp|devilution::OperateDecapitatedBody(Object &corpse, bool sendmsg, bool sendLootMsg) sha=7c9353603ec6
fn operate_decapitated_body(ctx: &mut Ctx, oi: usize, sendmsg: bool, send_loot_msg: bool) {
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    ctx.objects.Objects[oi]._oSelFlag = 0;
    let (pos, seed) = (ctx.objects.Objects[oi].position, ctx.objects.Objects[oi]._oRndSeed);
    ctx.rng.set_rnd_seed(seed);
    crate::items::create_rnd_item(ctx, pos, false, send_loot_msg, false);
    if sendmsg {
        send_operate(ctx, false, pos);
    }
}

/// Original: `OperateArmorStand` (objects.cpp).
// @port objects.cpp|devilution::OperateArmorStand(Object &armorStand, bool sendmsg, bool sendLootMsg) sha=729a6ec0d626
fn operate_armor_stand(ctx: &mut Ctx, oi: usize, sendmsg: bool, send_loot_msg: bool) {
    use crate::items::create_type_item;
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    ctx.objects.Objects[oi]._oSelFlag = 0;
    ctx.objects.Objects[oi]._oAnimFrame += 1;
    let (pos, seed) = (ctx.objects.Objects[oi].position, ctx.objects.Objects[oi]._oRndSeed);
    ctx.rng.set_rnd_seed(seed);
    let unique_rnd = !ctx.rng.flip_coin(2);
    let currlevel = ctx.gendung.currlevel;
    let none = IMISC_NONE as i32;
    if currlevel <= 5 {
        create_type_item(ctx, pos, true, ItemType::LightArmor, none, send_loot_msg, false, false);
    } else if (6..=9).contains(&currlevel) {
        create_type_item(ctx, pos, unique_rnd, ItemType::MediumArmor, none, send_loot_msg, false, false);
    } else if (10..=12).contains(&currlevel) {
        create_type_item(ctx, pos, false, ItemType::HeavyArmor, none, send_loot_msg, false, false);
    } else if currlevel >= 13 {
        create_type_item(ctx, pos, true, ItemType::HeavyArmor, none, send_loot_msg, false, false);
    }
    if sendmsg {
        send_operate(ctx, false, pos);
    }
}

/// Original: `FindValidShrine` (objects.cpp).
// @port objects.cpp|devilution::FindValidShrine() sha=a99bb3027f65
fn find_valid_shrine(ctx: &mut Ctx) -> i32 {
    loop {
        let rv = rnd(ctx, if ctx.init.gb_is_hellfire { shrine_type::NumberOfShrineTypes } else { 26 });
        let currlevel = ctx.gendung.currlevel as i32;
        let r = rv as usize;
        if currlevel < SHRINEMIN[r] || currlevel > SHRINEMAX[r] || rv == shrine_type::ShrineThaumaturgic {
            continue;
        }
        if ctx.init.gb_is_multiplayer && SHRINEAVAIL[r] == SHRINE_TYPE_SINGLE {
            continue;
        }
        if !ctx.init.gb_is_multiplayer && SHRINEAVAIL[r] == SHRINE_TYPE_MULTI {
            continue;
        }
        return rv;
    }
}

/// Original: `OperateGoatShrine` (objects.cpp).
// @port objects.cpp|devilution::OperateGoatShrine(Player &player, Object &object, _sfx_id sType) sha=f7f6efac7c9c
fn operate_goat_shrine(ctx: &mut Ctx, pnum: usize, oi: usize, s_type: crate::effects_data::SfxId) {
    let seed = ctx.objects.Objects[oi]._oRndSeed;
    ctx.rng.set_rnd_seed(seed);
    ctx.objects.Objects[oi]._oVar1 = find_valid_shrine(ctx);
    operate_shrine(ctx, pnum, oi, s_type);
    ctx.objects.Objects[oi]._oAnimDelay = 2;
    crate::engine::backbuffer_state::redraw_everything(ctx);
}

/// Original: `OperateCauldron` (objects.cpp).
// @port objects.cpp|devilution::OperateCauldron(Player &player, Object &object, _sfx_id sType) sha=e01837878a28
fn operate_cauldron(ctx: &mut Ctx, pnum: usize, oi: usize, s_type: crate::effects_data::SfxId) {
    let seed = ctx.objects.Objects[oi]._oRndSeed;
    ctx.rng.set_rnd_seed(seed);
    ctx.objects.Objects[oi]._oVar1 = find_valid_shrine(ctx);
    operate_shrine(ctx, pnum, oi, s_type);
    ctx.objects.Objects[oi]._oAnimFrame = 3;
    ctx.objects.Objects[oi]._oAnimFlag = 0;
    crate::engine::backbuffer_state::redraw_everything(ctx);
}

// ---------------------------------------------------------------------------------------------
// objects.cpp, part 5: fountains, breakables, sync helpers and level object setup.
// ---------------------------------------------------------------------------------------------

/// Original: `OperateFountains` (objects.cpp).
// @port objects.cpp|devilution::OperateFountains(Player &player, Object &fountain) sha=4f53d1632186
fn operate_fountains(ctx: &mut Ctx, pnum: usize, oi: usize) -> bool {
    use crate::effects_data::*;
    use crate::player::{modify_plr_dex, modify_plr_mag, modify_plr_str, modify_plr_vit};
    let mut applied = false;
    let pos = ctx.objects.Objects[oi].position;
    match ctx.objects.Objects[oi]._otype {
        OBJ_BLOODFTN => {
            if !is_my_player(ctx, pnum) {
                return false;
            }
            crate::effects::play_sfx_loc(ctx, LS_FOUNTAIN, pos, true);
            let player = &mut ctx.players.Players[pnum];
            if player._pHitPoints < player._pMaxHP {
                player._pHitPoints += 64;
                player._pHPBase += 64;
                if player._pHitPoints > player._pMaxHP {
                    player._pHitPoints = player._pMaxHP;
                    player._pHPBase = player._pMaxHPBase;
                }
                applied = true;
            }
        }
        OBJ_PURIFYINGFTN => {
            if !is_my_player(ctx, pnum) {
                return false;
            }
            crate::effects::play_sfx_loc(ctx, LS_FOUNTAIN, pos, true);
            let player = &mut ctx.players.Players[pnum];
            if player._pMana < player._pMaxMana {
                player._pMana += 64;
                player._pManaBase += 64;
                if player._pMana > player._pMaxMana {
                    player._pMana = player._pMaxMana;
                    player._pManaBase = player._pMaxManaBase;
                }
                applied = true;
            }
        }
        OBJ_MURKYFTN => {
            if ctx.objects.Objects[oi]._oSelFlag != 0 {
                crate::effects::play_sfx_loc(ctx, LS_FOUNTAIN, pos, true);
                ctx.objects.Objects[oi]._oSelFlag = 0;
                let (t, d) = (ctx.players.Players[pnum].position.tile, ctx.players.Players[pnum]._pdir);
                let lt = ctx.gendung.leveltype as i8 as i32;
                crate::missiles::add_missile(ctx, t, t, d, MissileID::Infravision, TARGET_MONSTERS, pnum as i32, 0, 2 * lt, None);
                applied = true;
                if is_my_player(ctx, pnum) {
                    send_operate(ctx, false, pos);
                }
            }
        }
        OBJ_TEARFTN => {
            if ctx.objects.Objects[oi]._oSelFlag != 0 {
                crate::effects::play_sfx_loc(ctx, LS_FOUNTAIN, pos, true);
                ctx.objects.Objects[oi]._oSelFlag = 0;
                if !is_my_player(ctx, pnum) {
                    return false;
                }
                let random_value = (ctx.objects.Objects[oi]._oRndSeed >> 16) % 12;
                let from_stat = random_value / 3;
                let mut to_stat = random_value % 3;
                if to_stat >= from_stat {
                    to_stat += 1;
                }
                for (stat, delta) in [(from_stat, -1), (to_stat, 1)] {
                    match stat {
                        0 => modify_plr_str(ctx, pnum, delta),
                        1 => modify_plr_mag(ctx, pnum, delta),
                        2 => modify_plr_dex(ctx, pnum, delta),
                        3 => modify_plr_vit(ctx, pnum, delta),
                        _ => {}
                    }
                }
                crate::player::check_stats(&mut ctx.players.Players[pnum]);
                applied = true;
                if is_my_player(ctx, pnum) {
                    send_operate(ctx, false, pos);
                }
            }
        }
        _ => {}
    }
    crate::engine::backbuffer_state::redraw_everything(ctx);
    applied
}

/// Original: `OperateWeaponRack` (objects.cpp).
// @port objects.cpp|devilution::OperateWeaponRack(Object &weaponRack, bool sendmsg, bool sendLootMsg) sha=7a8e41ec159c
fn operate_weapon_rack(ctx: &mut Ctx, oi: usize, sendmsg: bool, send_loot_msg: bool) {
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    let seed = ctx.objects.Objects[oi]._oRndSeed;
    ctx.rng.set_rnd_seed(seed);
    let weapon_type = ctx.rng.pick_randomly_among(&[ItemType::Sword, ItemType::Axe, ItemType::Bow, ItemType::Mace]);
    ctx.objects.Objects[oi]._oSelFlag = 0;
    ctx.objects.Objects[oi]._oAnimFrame += 1;
    let pos = ctx.objects.Objects[oi].position;
    let onlygood = ctx.gendung.leveltype != DungeonType::Cathedral;
    crate::items::create_type_item(ctx, pos, onlygood, weapon_type, IMISC_NONE as i32, send_loot_msg, false, false);
    if sendmsg {
        send_operate(ctx, false, pos);
    }
}

/// Original: `OperateNakrulBook` (objects.cpp): tracks the order Na-Krul's tomes are read in.
// @port objects.cpp|devilution::OperateNakrulBook(int s) sha=40b2efb025f8
fn operate_nakrul_book(ctx: &mut Ctx, s: i32) -> bool {
    let seq = &mut ctx.objects.NaKrulTomeSequence;
    match s {
        6 => *seq = 1,
        7 => *seq = if *seq == 1 { 2 } else { 0 },
        8 => {
            if *seq == 2 {
                return true;
            }
            *seq = 0;
        }
        _ => {}
    }
    false
}

/// Original: `OperateStoryBook` (objects.cpp).
// @port objects.cpp|devilution::OperateStoryBook(Object &storyBook) sha=a9385626c20b
fn operate_story_book(ctx: &mut Ctx, oi: usize) {
    use crate::effects_data::*;
    if ctx.objects.Objects[oi]._oSelFlag == 0 || crate::minitext::qtextflag(ctx) {
        return;
    }
    ctx.objects.Objects[oi]._oAnimFrame = ctx.objects.Objects[oi]._oVar4 as u32;
    let book = ctx.objects.Objects[oi].clone();
    crate::effects::play_sfx_loc(ctx, IS_ISCROL, book.position, true);
    let msg = book._oVar2 as _speech_id;
    if book._oVar8 != 0 && ctx.gendung.currlevel == 24 {
        if !ctx.crypt.IsUberLeverActivated && ctx.quests.Quests[Q_NAKRUL as usize]._qactive != QUEST_DONE && operate_nakrul_book(ctx, book._oVar8) {
            crate::msg::net_send_cmd(ctx, false, CMD_NAKRUL);
            return;
        }
    } else if ctx.gendung.leveltype == DungeonType::Crypt {
        let q = &mut ctx.quests.Quests[Q_NAKRUL as usize];
        q._qactive = QUEST_ACTIVE;
        q._qlog = true;
        q._qmsg = msg;
        crate::msg::net_send_cmd_quest(ctx, true, Q_NAKRUL as usize);
    }
    crate::minitext::init_q_text_msg(ctx, msg);
    send_operate(ctx, false, book.position);
}

/// Original: `OperateLazStand` (objects.cpp).
// @port objects.cpp|devilution::OperateLazStand(Object &stand) sha=4c6512626b44
fn operate_laz_stand(ctx: &mut Ctx, oi: usize) {
    if ctx.items.ActiveItemCount as usize >= crate::items::MAXITEMS {
        return;
    }
    if ctx.objects.Objects[oi]._oSelFlag == 0 || crate::minitext::qtextflag(ctx) {
        return;
    }
    ctx.objects.Objects[oi]._oAnimFrame += 1;
    ctx.objects.Objects[oi]._oSelFlag = 0;
    let p = ctx.objects.Objects[oi].position;
    let pos = crate::items::get_super_item_loc(ctx, p);
    crate::items::spawn_quest_item(ctx, IDI_LAZSTAFF, pos, 0, 0, true);
    send_operate(ctx, false, p);
}

/// Original: `AreAllCruxesOfTypeBroken` (objects.cpp).
// @port objects.cpp|devilution::AreAllCruxesOfTypeBroken(int cruxType) sha=f30b308ba7a6
fn are_all_cruxes_of_type_broken(ctx: &Ctx, crux_type: i32) -> bool {
    for j in 0..ctx.objects.ActiveObjectCount as usize {
        let test_object = &ctx.objects.Objects[ctx.objects.ActiveObjects[j] as usize];
        if !test_object.is_crux() {
            continue; // Not a Crux object, keep searching
        }
        if crux_type != test_object._oVar8 || test_object._oBreak == -1 {
            continue; // Found either a different crux or a previously broken crux, keep searching
        }
        // Found an unbroken crux of this type
        return false;
    }
    true
}

/// Original: `BreakCrux` (objects.cpp).
// @port objects.cpp|devilution::BreakCrux(Object &crux, bool sendmsg) sha=622e2bc1b5cb
fn break_crux(ctx: &mut Ctx, oi: usize, sendmsg: bool) {
    use crate::effects_data::*;
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    {
        let crux = &mut ctx.objects.Objects[oi];
        crux._oAnimFlag = 1;
        crux._oAnimFrame = 1;
        crux._oAnimDelay = 1;
        crux._oSolidFlag = true;
        crux._oMissFlag = true;
        crux._oBreak = -1;
        crux._oSelFlag = 0;
    }
    let crux = ctx.objects.Objects[oi].clone();
    if sendmsg {
        let id = ctx.players.MyPlayerId;
        crate::msg::net_send_cmd_loc(ctx, id, false, CMD_BREAKOBJ, crux.position);
    }
    if !are_all_cruxes_of_type_broken(ctx, crux._oVar8) {
        return;
    }
    crate::effects::play_sfx_loc(ctx, IS_LEVER, crux.position, true);
    obj_change_map(ctx, crux._oVar1, crux._oVar2, crux._oVar3, crux._oVar4);
}

/// Original: `BreakBarrel` (objects.cpp).
// @port objects.cpp|devilution::BreakBarrel(const Player &player, Object &barrel, bool forcebreak, bool sendmsg) sha=d805e632d685
fn break_barrel(ctx: &mut Ctx, pnum: usize, oi: usize, forcebreak: bool, sendmsg: bool) {
    use crate::effects_data::*;
    if ctx.objects.Objects[oi]._oSelFlag == 0 {
        return;
    }
    if !forcebreak && !is_my_player(ctx, pnum) {
        return;
    }
    {
        let barrel = &mut ctx.objects.Objects[oi];
        barrel._oAnimFlag = 1;
        barrel._oAnimFrame = 1;
        barrel._oAnimDelay = 1;
        barrel._oSolidFlag = false;
        barrel._oMissFlag = true;
        barrel._oBreak = -1;
        barrel._oSelFlag = 0;
        barrel._oPreFlag = true;
    }
    let barrel = ctx.objects.Objects[oi].clone();
    if barrel.is_explosive() {
        let sfx = if barrel._otype == OBJ_URNEX {
            IS_POPPOP3
        } else if barrel._otype == OBJ_PODEX {
            IS_POPPOP8
        } else {
            IS_BARLFIRE
        };
        crate::effects::play_sfx_loc(ctx, sfx, barrel.position, true);
        for yp in barrel.position.y - 1..=barrel.position.y + 1 {
            for xp in barrel.position.x - 1..=barrel.position.x + 1 {
                const TRAP_MISSILE: MissileID = MissileID::Firebolt;
                let dt = crate::missiles::get_missile_data(TRAP_MISSILE).damage_type();
                let (x, y) = (xp as usize, yp as usize);
                if ctx.gendung.dMonster[x][y] > 0 {
                    let m = (ctx.gendung.dMonster[x][y] - 1) as usize;
                    crate::missiles::monster_trap_hit(ctx, m, 1, 4, 0, TRAP_MISSILE, dt, false);
                }
                if ctx.gendung.dPlayer[x][y] > 0 {
                    let mut unused = false;
                    let p = (ctx.gendung.dPlayer[x][y] - 1) as usize;
                    crate::missiles::player_m_hit(ctx, p, None, 0, 8, 16, TRAP_MISSILE, dt, false, DeathReason::MonsterOrTrap, &mut unused);
                }
                // don't really need to exclude large objects as explosive barrels are single tile objects, but using considerLargeObjects == false as this matches the old logic.
                if let Some(adjacent) = find_object_at_position(ctx, Point::new(xp, yp), false) {
                    if ctx.objects.Objects[adjacent].is_explosive() && !ctx.objects.Objects[adjacent].is_broken() {
                        break_barrel(ctx, pnum, adjacent, true, sendmsg);
                    }
                }
            }
        }
    } else {
        let sfx = if barrel._otype == OBJ_URN {
            IS_POPPOP2
        } else if barrel._otype == OBJ_POD {
            IS_POPPOP5
        } else {
            IS_BARREL
        };
        crate::effects::play_sfx_loc(ctx, sfx, barrel.position, true);
        ctx.rng.set_rnd_seed(barrel._oRndSeed);
        if barrel._oVar2 <= 1 {
            if barrel._oVar3 == 0 {
                crate::items::create_rnd_useful(ctx, barrel.position, sendmsg);
            } else {
                crate::items::create_rnd_item(ctx, barrel.position, false, sendmsg, false);
            }
        }
        if barrel._oVar2 >= 8 && barrel._oVar4 >= 0 {
            crate::monster::activate_skeleton(ctx, barrel._oVar4 as usize, barrel.position);
        }
    }
    if is_my_player(ctx, pnum) {
        let id = ctx.players.MyPlayerId;
        crate::msg::net_send_cmd_loc(ctx, id, false, CMD_BREAKOBJ, barrel.position);
    }
}

/// Original: `SyncCrux` (objects.cpp).
// @port objects.cpp|devilution::SyncCrux(const Object &crux) sha=f2a3b8e61b30
fn sync_crux(ctx: &mut Ctx, oi: usize) {
    let c = ctx.objects.Objects[oi].clone();
    if are_all_cruxes_of_type_broken(ctx, c._oVar8) {
        obj_change_map(ctx, c._oVar1, c._oVar2, c._oVar3, c._oVar4);
    }
}

/// Original: `SyncLever` (objects.cpp).
// @port objects.cpp|devilution::SyncLever(const Object &lever) sha=ab120b274a5c
fn sync_lever(ctx: &mut Ctx, oi: usize) {
    let l = ctx.objects.Objects[oi].clone();
    if l._oSelFlag != 0 {
        return;
    }
    if ctx.gendung.currlevel == 16 && !are_all_levers_activated(ctx, l._oVar8) {
        return;
    }
    obj_change_map(ctx, l._oVar1, l._oVar2, l._oVar3, l._oVar4);
}

/// Original: `SyncQSTLever` (objects.cpp).
// @port objects.cpp|devilution::SyncQSTLever(const Object &qstLever) sha=369f51c78271
fn sync_qst_lever(ctx: &mut Ctx, oi: usize) {
    let q = ctx.objects.Objects[oi].clone();
    if q._oAnimFrame as i64 == q._oVar6 as i64 {
        if q._otype != OBJ_BLOODBOOK {
            obj_change_map_resync(ctx, q._oVar1, q._oVar2, q._oVar3, q._oVar4);
        }
        if q._otype == OBJ_BLINDBOOK {
            let tren = ctx.gendung.TransVal;
            ctx.gendung.TransVal = 9;
            crate::levels::gendung::drlg_m_rect_trans_points(ctx, Point::new(q._oVar1, q._oVar2), Point::new(q._oVar3, q._oVar4));
            ctx.gendung.TransVal = tren;
        }
    }
}

/// Original: `SyncPedestal` (objects.cpp).
// @port objects.cpp|devilution::SyncPedestal(const Object &pedestal) sha=02d566371cc8
fn sync_pedestal(ctx: &mut Ctx, oi: usize) {
    let p = ctx.objects.Objects[oi].clone();
    let sp = ctx.gendung.SetPiece;
    let (x, y) = (sp.position.x, sp.position.y);
    if p._oVar6 == 1 {
        obj_change_map_resync(ctx, x, y + 3, x + 2, y + 7);
    }
    if p._oVar6 == 2 {
        obj_change_map_resync(ctx, x, y + 3, x + 2, y + 7);
        obj_change_map_resync(ctx, x + 6, y + 3, x + sp.size.width, y + 7);
    }
    if p._oVar6 >= 3 {
        obj_change_map_resync(ctx, p._oVar1, p._oVar2, p._oVar3, p._oVar4);
        load_map_objects(ctx, "levels\\l2data\\blood2.dun", sp.position.mega_to_world(), Rectangle::default(), 0);
    }
}

/// Original: `UpdatePedestalState` (objects.cpp).
// @port objects.cpp|devilution::UpdatePedestalState(Object &pedestal) sha=79ed1c2b743a
fn update_pedestal_state(ctx: &mut Ctx, oi: usize) {
    let added_stones = ctx.quests.Quests[Q_BLOOD as usize]._qvar2 as i32;
    ctx.objects.Objects[oi]._oAnimFrame = (ctx.objects.Objects[oi]._oAnimFrame as i32 + added_stones) as u32;
    ctx.objects.Objects[oi]._oVar6 = (ctx.objects.Objects[oi]._oVar6 as i32 + added_stones) as u32;
    sync_pedestal(ctx, oi);
    if ctx.objects.Objects[oi]._oVar6 >= 3 {
        ctx.objects.Objects[oi]._oSelFlag = 0;
    }
}

/// Original: `SyncDoor` (objects.cpp).
// @port objects.cpp|devilution::SyncDoor(Object &door) sha=5c74303e51d9
fn sync_door(ctx: &mut Ctx, oi: usize) {
    if ctx.objects.Objects[oi]._oVar4 == DOOR_CLOSED {
        set_door_state_closed(ctx, oi);
    } else {
        set_door_state_open(ctx, oi);
    }
}

/// Original: `ResyncDoors` (objects.cpp).
// @port objects.cpp|devilution::ResyncDoors(WorldTilePosition p1, WorldTilePosition p2, bool sendmsg) sha=6eca53b1d316
fn resync_doors(ctx: &mut Ctx, p1: Point, p2: Point, sendmsg: bool) {
    // WorldTileCoord is uint8_t
    let size = Size::new((p2.x - p1.x) & 0xff, (p2.y - p1.y) & 0xff);
    let area = Rectangle::new(p1, size);
    for p in crate::engine::geometry::points_in_rectangle(area) {
        let Some(obj) = find_object_at_position(ctx, p, true) else {
            continue;
        };
        if !ctx.objects.Objects[obj].is_door() {
            continue;
        }
        sync_door(ctx, obj);
        if sendmsg {
            let is_open = ctx.objects.Objects[obj]._oVar4 == DOOR_OPEN;
            let (id, pos) = (ctx.players.MyPlayerId, ctx.objects.Objects[obj].position);
            crate::msg::net_send_cmd_loc(ctx, id, true, if is_open { CMD_OPENDOOR } else { CMD_CLOSEDOOR }, pos);
        }
    }
}

/// Original: `UpdateState` (objects.cpp).
// @port objects.cpp|devilution::UpdateState(Object &object, int frame) sha=aa67de24bdab
fn update_state(ctx: &mut Ctx, oi: usize, frame: i32) {
    let o = &mut ctx.objects.Objects[oi];
    if o._oSelFlag == 0 {
        return;
    }
    o._oSelFlag = 0;
    o._oAnimFrame = frame as u32;
    o._oAnimFlag = 0;
}

impl Object {
    /// Original: `Object::GetId` (objects.cpp).
    // @port objects.cpp|devilution::Object::GetId() sha=1a89e17a2593
    pub fn get_id(&self, ctx: &Ctx) -> u32 {
        (ctx.gendung.dObject[self.position.x as usize][self.position.y as usize] as i32).unsigned_abs() - 1
    }

    /// `Object::IsDisabled` reading `sgOptions.Gameplay.disableCripplingShrines`.
    pub fn is_disabled(&self, ctx: &Ctx) -> bool {
        self.is_disabled_opt(ctx.options.gameplay.disable_crippling_shrines.get())
    }
}

/// Original: `devilution::LoadLevelObjects` (objects.cpp).
// @port objects.cpp|devilution::LoadLevelObjects(uint16_t filesWidths[65]) sha=cfe5aba45047
pub fn load_level_objects(ctx: &mut Ctx, files_widths: &mut [u16; 65]) {
    if ctx.diablo.headless_mode {
        return;
    }
    for object_data in AllObjects.iter() {
        if ctx.gendung.leveltype as i8 == object_data.olvltype {
            files_widths[object_data.ofindex as usize] = object_data.animWidth as u16;
        }
    }
    for i in OFILE_L1BRAZ as usize..=OFILE_L5BOOKS as usize {
        if files_widths[i] == 0 {
            continue;
        }
        let n = ctx.objects.numobjfiles as usize;
        ctx.objects.ObjFileList[n] = i as object_graphic_id;
        let filestr = format!("objects\\{}", crate::tables::objdat::ObjMasterLoadList[i]);
        ctx.objects.p_obj_cels[n] = Some(crate::engine::load_sprites::load_cel(ctx, &filestr, files_widths[i]));
        ctx.objects.numobjfiles += 1;
    }
}

/// Original: `devilution::InitObjectGFX` (objects.cpp).
// @port objects.cpp|devilution::InitObjectGFX() sha=8df0083bc51e
pub fn init_object_gfx(ctx: &mut Ctx) {
    let mut files_widths = [0u16; 65];
    let currlevel = ctx.gendung.currlevel;
    if matches!(currlevel, 4 | 8 | 12) {
        files_widths[OFILE_BKSLBRNT as usize] = AllObjects[OBJ_STORYBOOK as usize].animWidth as u16;
        files_widths[OFILE_CANDLE2 as usize] = AllObjects[OBJ_STORYCANDLE as usize].animWidth as u16;
    }
    for object_data in AllObjects.iter() {
        if object_data.minlvl != 0 && currlevel as i32 >= object_data.minlvl as i32 && currlevel as i32 <= object_data.maxlvl as i32 {
            if object_data.ofindex == OFILE_TRAPHOLE && ctx.gendung.leveltype == DungeonType::Hell {
                continue;
            }
            files_widths[object_data.ofindex as usize] = object_data.animWidth as u16;
        }
        if object_data.otheme != THEME_NONE {
            for j in 0..ctx.themes.numthemes as usize {
                if ctx.themes.themes[j].ttype == object_data.otheme {
                    files_widths[object_data.ofindex as usize] = object_data.animWidth as u16;
                }
            }
        }
        if object_data.oquest != Q_INVALID && crate::quests::is_quest_available(ctx, object_data.oquest) {
            files_widths[object_data.ofindex as usize] = object_data.animWidth as u16;
        }
    }
    load_level_objects(ctx, &mut files_widths);
}

fn add_objs_by_piece(ctx: &mut Ctx, x1: i32, y1: i32, x2: i32, y2: i32, table: &[(&[i32], _object_id)]) {
    for j in y1..y2 {
        for i in x1..x2 {
            let pn = piece(ctx, i, j);
            for (pieces, obj) in table {
                if pieces.contains(&pn) {
                    add_object(ctx, *obj, Point::new(i, j));
                }
            }
        }
    }
}

/// Original: `devilution::AddL1Objs` (objects.cpp).
// @port objects.cpp|devilution::AddL1Objs(int x1, int y1, int x2, int y2) sha=a5930c7ef38b
pub fn add_l1_objs(ctx: &mut Ctx, x1: i32, y1: i32, x2: i32, y2: i32) {
    add_objs_by_piece(ctx, x1, y1, x2, y2, &[(&[269], OBJ_L1LIGHT), (&[43, 50, 213], OBJ_L1LDOOR), (&[45, 55], OBJ_L1RDOOR)]);
}

/// Original: `devilution::AddL2Objs` (objects.cpp).
// @port objects.cpp|devilution::AddL2Objs(int x1, int y1, int x2, int y2) sha=91ea84a49153
pub fn add_l2_objs(ctx: &mut Ctx, x1: i32, y1: i32, x2: i32, y2: i32) {
    add_objs_by_piece(ctx, x1, y1, x2, y2, &[(&[12, 540], OBJ_L2LDOOR), (&[16, 541], OBJ_L2RDOOR)]);
}

/// Original: `devilution::AddL3Objs` (objects.cpp).
// @port objects.cpp|devilution::AddL3Objs(int x1, int y1, int x2, int y2) sha=90b6fadaffd8
pub fn add_l3_objs(ctx: &mut Ctx, x1: i32, y1: i32, x2: i32, y2: i32) {
    add_objs_by_piece(ctx, x1, y1, x2, y2, &[(&[530], OBJ_L3LDOOR), (&[533], OBJ_L3RDOOR)]);
}

/// Original: `devilution::AddCryptObjects` (objects.cpp).
// @port objects.cpp|devilution::AddCryptObjects(int x1, int y1, int x2, int y2) sha=1b6fc7bdf374
pub fn add_crypt_objects(ctx: &mut Ctx, x1: i32, y1: i32, x2: i32, y2: i32) {
    add_objs_by_piece(ctx, x1, y1, x2, y2, &[(&[76], OBJ_L5LDOOR), (&[79], OBJ_L5RDOOR)]);
}

/// Original: `devilution::AddSlainHero` (objects.cpp).
// @port objects.cpp|devilution::AddSlainHero() sha=956d69f9cbd7
pub fn add_slain_hero(ctx: &mut Ctx) {
    let loc = get_rnd_obj_loc(ctx, 5);
    add_object(ctx, OBJ_SLAINHERO, loc + Displacement::new(2, 2));
}

// ---------------------------------------------------------------------------------------------
// objects.cpp, part 6: InitObjects, AddObject, ProcessObjects and the public interface.
// ---------------------------------------------------------------------------------------------

fn class_text(ctx: &Ctx, w: _speech_id, r: _speech_id, s: _speech_id, m: _speech_id) -> _speech_id {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    match ctx.players.Players[me]._pClass {
        HeroClass::Warrior | HeroClass::Barbarian => w,
        HeroClass::Rogue | HeroClass::Bard => r,
        HeroClass::Sorcerer => s,
        HeroClass::Monk => m,
    }
}

/// Original: `devilution::InitObjects` (objects.cpp).
// @port objects.cpp|devilution::InitObjects() sha=fbe1158bc070
pub fn init_objects(ctx: &mut Ctx) {
    use crate::quests::is_quest_available;
    clr_all_objects(ctx);
    ctx.objects.NaKrulTomeSequence = 0;
    let currlevel = ctx.gendung.currlevel;
    if currlevel == 16 {
        add_diab_objs(ctx);
        return;
    }
    ctx.rng.discard_random_values(1);
    if currlevel == 9 && !crate::quests::use_multiplayer_quests(ctx) {
        add_slain_hero(ctx);
    }
    if is_quest_available(ctx, Q_MUSHROOM) {
        add_mush_patch(ctx);
    }
    if currlevel == 4 || currlevel == 8 || currlevel == 12 {
        add_story_books(ctx);
    }
    if currlevel == 21 {
        add_crypt_story_book(ctx, 1);
    } else if currlevel == 22 {
        add_crypt_story_book(ctx, 2);
        add_crypt_story_book(ctx, 3);
    } else if currlevel == 23 {
        add_crypt_story_book(ctx, 4);
        add_crypt_story_book(ctx, 5);
    }
    if currlevel == 24 {
        add_nakrul_gate(ctx);
    }
    let lt = ctx.gendung.leveltype;
    let (mx, my) = (MAXDUNX as i32, MAXDUNY as i32);
    let sp = ctx.gendung.SetPiece;
    if lt == DungeonType::Cathedral {
        if is_quest_available(ctx, Q_BUTCHER) {
            add_tortures(ctx);
        }
        if is_quest_available(ctx, Q_PWATER) {
            add_candles(ctx);
        }
        if is_quest_available(ctx, Q_LTBANNER) {
            add_object(ctx, OBJ_SIGNCHEST, sp.position.mega_to_world() + Displacement::new(10, 3));
        }
        init_rnd_loc_big_obj(ctx, 10, 15, OBJ_SARC);
        add_l1_objs(ctx, 0, 0, mx, my);
        init_rnd_barrels(ctx);
    }
    if lt == DungeonType::Catacombs {
        if is_quest_available(ctx, Q_ROCK) {
            init_rnd_loc_obj5x5(ctx, 1, 1, OBJ_STAND);
        }
        if is_quest_available(ctx, Q_SCHAMB) {
            init_rnd_loc_obj5x5(ctx, 1, 1, OBJ_BOOK2R);
        }
        add_l2_objs(ctx, 0, 0, mx, my);
        add_l2_torches(ctx);
        if is_quest_available(ctx, Q_BLIND) {
            let sp_id = class_text(ctx, TEXT_BLINDING, TEXT_RBLINDING, TEXT_MBLINDING, TEXT_HBLINDING);
            ctx.quests.Quests[Q_BLIND as usize]._qmsg = sp_id;
            let area = Rectangle::new(sp.position, Size::new(sp.size.width + 1, sp.size.height + 1));
            add_book_lever(ctx, OBJ_BLINDBOOK, area, sp_id);
            load_map_objects(ctx, "levels\\l2data\\blind2.dun", sp.position.mega_to_world(), Rectangle::default(), 0);
        }
        if is_quest_available(ctx, Q_BLOOD) {
            let sp_id = class_text(ctx, TEXT_BLOODY, TEXT_RBLOODY, TEXT_MBLOODY, TEXT_HBLOODY);
            ctx.quests.Quests[Q_BLOOD as usize]._qmsg = sp_id;
            add_book_lever(ctx, OBJ_BLOODBOOK, Rectangle::new(sp.position + Displacement::new(0, 3), Size::new(2, 4)), sp_id);
            add_object(ctx, OBJ_PEDESTAL, sp.position.mega_to_world() + Displacement::new(9, 16));
        }
        init_rnd_barrels(ctx);
    }
    if lt == DungeonType::Caves {
        add_l3_objs(ctx, 0, 0, mx, my);
        init_rnd_barrels(ctx);
    }
    if lt == DungeonType::Hell {
        if is_quest_available(ctx, Q_WARLORD) {
            let sp_id = class_text(ctx, TEXT_BLOODWAR, TEXT_RBLOODWAR, TEXT_MBLOODWAR, TEXT_HBLOODWAR);
            ctx.quests.Quests[Q_WARLORD as usize]._qmsg = sp_id;
            add_book_lever(ctx, OBJ_STEELTOME, sp, sp_id);
            load_map_objects(ctx, "levels\\l4data\\warlord.dun", sp.position.mega_to_world(), Rectangle::default(), 0);
        }
        if is_quest_available(ctx, Q_BETRAYER) && !crate::quests::use_multiplayer_quests(ctx) {
            add_laz_stand(ctx);
        }
        init_rnd_barrels(ctx);
        add_l4_goodies(ctx);
    }
    if lt == DungeonType::Nest {
        init_rnd_barrels(ctx);
    }
    if lt == DungeonType::Crypt {
        init_rnd_loc_big_obj(ctx, 10, 15, OBJ_L5SARC);
        add_crypt_objects(ctx, 0, 0, mx, my);
        init_rnd_barrels(ctx);
    }
    init_rnd_loc_obj(ctx, 5, 10, OBJ_CHEST1);
    init_rnd_loc_obj(ctx, 3, 6, OBJ_CHEST2);
    init_rnd_loc_obj(ctx, 1, 5, OBJ_CHEST3);
    if lt != DungeonType::Hell {
        add_obj_traps(ctx);
    }
    if matches!(lt, DungeonType::Catacombs | DungeonType::Caves | DungeonType::Hell | DungeonType::Nest) {
        add_chest_traps(ctx);
    }
}

/// Original: `devilution::SetMapObjects` (objects.cpp).
// @port objects.cpp|devilution::SetMapObjects(const uint16_t *dunData, int startx, int starty) sha=96f0386ea3f8
pub fn set_map_objects(ctx: &mut Ctx, dun_data: &[u16], startx: i32, starty: i32) {
    let mut files_widths = [0u16; 65];
    clr_all_objects(ctx);
    let mut width = dun_data[0] as usize;
    let mut height = dun_data[1] as usize;
    let layer2_offset = 2 + width * height;
    // The rest of the layers are at dPiece scale
    width *= 2;
    height *= 2;
    let object_layer = &dun_data[layer2_offset + width * height * 2..];
    for j in 0..height {
        for i in 0..width {
            let object_id = object_layer[j * width + i] as u8;
            if object_id != 0 {
                let object_data = &AllObjects[ObjTypeConv[object_id as usize] as usize];
                files_widths[object_data.ofindex as usize] = object_data.animWidth as u16;
            }
        }
    }
    load_level_objects(ctx, &mut files_widths);
    for j in 0..height {
        for i in 0..width {
            let object_id = object_layer[j * width + i] as u8;
            if object_id != 0 {
                add_object(ctx, ObjTypeConv[object_id as usize], Point::new(startx + 16 + i as i32, starty + 16 + j as i32));
            }
        }
    }
}

/// Original: `devilution::AddObject` (objects.cpp): spawns an object of the given type.
// @port objects.cpp|devilution::AddObject(_object_id objType, Point objPos) sha=cb7f56139091
pub fn add_object(ctx: &mut Ctx, obj_type: _object_id, obj_pos: Point) -> Option<usize> {
    let s = &mut ctx.objects;
    if s.ActiveObjectCount as usize >= MAXOBJECTS {
        return None;
    }
    let oi = s.AvailableObjects[0] as usize;
    s.AvailableObjects[0] = s.AvailableObjects[MAXOBJECTS - 1 - s.ActiveObjectCount as usize];
    s.ActiveObjects[s.ActiveObjectCount as usize] = oi as i32;
    ctx.gendung.dObject[obj_pos.x as usize][obj_pos.y as usize] = (oi + 1) as i8;
    setup_object(ctx, oi, obj_pos, obj_type);
    match ctx.objects.Objects[oi]._otype {
        OBJ_L1LDOOR | OBJ_L1RDOOR | OBJ_L2LDOOR | OBJ_L2RDOOR | OBJ_L3LDOOR | OBJ_L3RDOOR | OBJ_L5LDOOR | OBJ_L5RDOOR => add_door(ctx, oi),
        OBJ_BOOK2R => {
            let sp = ctx.gendung.SetPiece;
            let r = Rectangle::new(sp.position, Size::new((sp.size.width + 1) & 0xff, (sp.size.height + 1) & 0xff));
            ctx.objects.Objects[oi].initialize_book(r);
        }
        OBJ_CHEST1 | OBJ_CHEST2 | OBJ_CHEST3 => add_chest(ctx, oi),
        OBJ_TCHEST1 | OBJ_TCHEST2 | OBJ_TCHEST3 => {
            add_chest(ctx, oi);
            ctx.objects.Objects[oi]._oTrapFlag = true;
            let v4 = if ctx.gendung.leveltype == DungeonType::Catacombs { rnd(ctx, 2) } else { rnd(ctx, 3) };
            ctx.objects.Objects[oi]._oVar4 = v4;
        }
        OBJ_SARC | OBJ_L5SARC => add_sarcophagus(ctx, oi),
        OBJ_FLAMEHOLE => add_flame_trap(ctx, oi),
        OBJ_FLAMELVR => add_flame_lever(ctx, oi),
        OBJ_WATER => ctx.objects.Objects[oi]._oAnimFrame = 1,
        OBJ_TRAPL | OBJ_TRAPR => add_trap(ctx, oi),
        OBJ_BARREL | OBJ_BARRELEX | OBJ_POD | OBJ_PODEX | OBJ_URN | OBJ_URNEX => add_barrel(ctx, oi),
        OBJ_SHRINEL | OBJ_SHRINER => add_shrine(ctx, oi),
        OBJ_BOOKCASEL | OBJ_BOOKCASER => add_bookcase(ctx, oi),
        OBJ_SKELBOOK | OBJ_BOOKSTAND | OBJ_BLOODFTN | OBJ_GOATSHRINE | OBJ_CAULDRON | OBJ_TEARFTN | OBJ_SLAINHERO => {
            ctx.objects.Objects[oi]._oRndSeed = ctx.rng.advance_rnd_seed() as u32;
        }
        OBJ_DECAP => add_decapitated_body(ctx, oi),
        OBJ_PURIFYINGFTN | OBJ_MURKYFTN => add_large_fountain(ctx, oi),
        OBJ_ARMORSTAND | OBJ_WARARMOR => add_armor_stand(ctx, oi),
        OBJ_BOOK2L => add_book_of_vileness(ctx, oi),
        OBJ_MCIRCLE1 | OBJ_MCIRCLE2 => add_magic_circle(ctx, oi),
        OBJ_STORYBOOK | OBJ_L5BOOKS => add_story_book(ctx, oi),
        OBJ_BCROSS | OBJ_TBCROSS => {
            ctx.objects.Objects[oi]._oRndSeed = ctx.rng.advance_rnd_seed() as u32;
        }
        OBJ_PEDESTAL => add_pedestal_of_blood(ctx, oi),
        OBJ_WARWEAP | OBJ_WEAPONRACK => add_weapon_rack(ctx, oi),
        OBJ_TNUDEM2 => add_tortured_body(ctx, oi),
        _ => {}
    }
    add_object_light(ctx, oi);
    ctx.objects.ActiveObjectCount += 1;
    Some(oi)
}

/// Original: `devilution::UpdateTrapState` (objects.cpp).
// @port objects.cpp|devilution::UpdateTrapState(Object &trap) sha=8dd224ccc20e
pub fn update_trap_state(ctx: &mut Ctx, oi: usize) -> bool {
    let trap = &ctx.objects.Objects[oi];
    if trap._oVar4 != 0 {
        return false;
    }
    let trigger = object_at_position(ctx, Point::new(trap._oVar1, trap._oVar2));
    let t = &ctx.objects.Objects[trigger];
    match t._otype {
        OBJ_L1LDOOR | OBJ_L1RDOOR | OBJ_L2LDOOR | OBJ_L2RDOOR | OBJ_L3LDOOR | OBJ_L3RDOOR | OBJ_L5LDOOR | OBJ_L5RDOOR => {
            if t._oVar4 == DOOR_CLOSED && t._oTrapFlag {
                return false;
            }
        }
        OBJ_LEVER | OBJ_CHEST1 | OBJ_CHEST2 | OBJ_CHEST3 | OBJ_SWITCHSKL | OBJ_SARC | OBJ_L5LEVER | OBJ_L5SARC => {
            if t._oSelFlag != 0 && t._oTrapFlag {
                return false;
            }
        }
        _ => return false,
    }
    ctx.objects.Objects[oi]._oVar4 = 1;
    ctx.objects.Objects[trigger]._oTrapFlag = false;
    true
}

/// Original: `devilution::OperateTrap` (objects.cpp).
// @port objects.cpp|devilution::OperateTrap(Object &trap) sha=b483d714798b
pub fn operate_trap(ctx: &mut Ctx, oi: usize) {
    if !update_trap_state(ctx, oi) {
        return;
    }
    let trap = ctx.objects.Objects[oi].clone();
    // default to firing at the trigger object
    let trigger_position = Point::new(trap._oVar1, trap._oVar2);
    let mut target = trigger_position;
    let search_area: Vec<Point> = crate::engine::geometry::points_in_rectangle(Rectangle::from_center(target, 1)).collect();
    // look for a player near the trigger (using a reverse search to match vanilla behaviour)
    if let Some(found) = search_area.iter().rev().find(|p| in_dungeon_bounds(**p) && ctx.gendung.dPlayer[p.x as usize][p.y as usize] != 0) {
        // if a player is standing near the trigger then target them instead
        target = *found;
    }
    let dir = crate::engine::get_direction(trap.position, target);
    let mtype = MissileID::from_repr(trap._oVar3 as i8).unwrap_or(MissileID::Arrow);
    crate::missiles::add_missile(ctx, trap.position, target, dir, mtype, TARGET_PLAYERS, -1, 0, 0, None);
    crate::effects::play_sfx_loc(ctx, crate::effects_data::IS_TRAP, trigger_position, true);
}

/// Original: `devilution::ProcessObjects` (objects.cpp).
// @port objects.cpp|devilution::ProcessObjects() sha=b3672389ac87
pub fn process_objects(ctx: &mut Ctx) {
    let mut i = 0;
    while i < ctx.objects.ActiveObjectCount as usize {
        let oi = ctx.objects.ActiveObjects[i] as usize;
        match ctx.objects.Objects[oi]._otype {
            OBJ_L1LIGHT | OBJ_SKFIRE | OBJ_CANDLE1 | OBJ_CANDLE2 | OBJ_BOOKCANDLE => update_object_light(ctx, oi, 5),
            OBJ_STORYCANDLE | OBJ_L5CANDLE => update_object_light(ctx, oi, 3),
            OBJ_CRUX1 | OBJ_CRUX2 | OBJ_CRUX3 | OBJ_BARREL | OBJ_BARRELEX | OBJ_POD | OBJ_PODEX | OBJ_URN | OBJ_URNEX | OBJ_SHRINEL | OBJ_SHRINER => object_stop_anim(ctx, oi),
            OBJ_L1LDOOR | OBJ_L1RDOOR | OBJ_L2LDOOR | OBJ_L2RDOOR | OBJ_L3LDOOR | OBJ_L3RDOOR | OBJ_L5LDOOR | OBJ_L5RDOOR => update_door(ctx, oi),
            OBJ_TORCHL | OBJ_TORCHR | OBJ_TORCHL2 | OBJ_TORCHR2 => update_object_light(ctx, oi, 8),
            OBJ_SARC | OBJ_L5SARC => update_sarcophagus(ctx, oi),
            OBJ_FLAMEHOLE => update_flame_trap(ctx, oi),
            OBJ_TRAPL | OBJ_TRAPR => operate_trap(ctx, oi),
            OBJ_MCIRCLE1 | OBJ_MCIRCLE2 => update_circle(ctx, oi),
            OBJ_BCROSS | OBJ_TBCROSS => {
                update_object_light(ctx, oi, 5);
                update_burning_cross_damage(ctx, oi);
            }
            _ => {}
        }
        let object = &mut ctx.objects.Objects[oi];
        i += 1;
        if object._oAnimFlag == 0 {
            continue;
        }
        object._oAnimCnt += 1;
        if object._oAnimCnt < object._oAnimDelay {
            continue;
        }
        object._oAnimCnt = 0;
        object._oAnimFrame += 1;
        if object._oAnimFrame > object._oAnimLen {
            object._oAnimFrame = 1;
        }
    }
    let mut i = 0;
    while i < ctx.objects.ActiveObjectCount as usize {
        let oi = ctx.objects.ActiveObjects[i] as usize;
        if ctx.objects.Objects[oi]._oDelFlag {
            delete_object(ctx, oi, i);
        } else {
            i += 1;
        }
    }
}

/// Original: `devilution::RedoPlayerVision` (objects.cpp).
// @port objects.cpp|devilution::RedoPlayerVision() sha=1662878dfd2e
pub fn redo_player_vision(ctx: &mut Ctx) {
    for pnum in 0..ctx.players.Players.len() {
        if ctx.players.Players[pnum].plractive && crate::player::is_on_active_level(ctx, pnum) {
            let t = ctx.players.Players[pnum].position.tile;
            crate::lighting::change_vision_xy(ctx, pnum as i32, t);
        }
    }
}

/// Original: `devilution::MonstCheckDoors` (objects.cpp).
// @port objects.cpp|devilution::MonstCheckDoors(const Monster &monster) sha=c832731f3781
pub fn monst_check_doors(ctx: &mut Ctx, m: usize) {
    for dir in [Direction::NorthEast, Direction::SouthWest, Direction::North, Direction::East, Direction::South, Direction::West, Direction::NorthWest, Direction::SouthEast] {
        let p = ctx.monster.Monsters[m].position.tile + dir;
        let Some(door) = find_object_at_position(ctx, p, true) else {
            continue;
        };
        // Doors use _oVar4 to track open/closed state, non-zero values indicate an open door
        if !ctx.objects.Objects[door].is_door() || ctx.objects.Objects[door]._oVar4 != DOOR_CLOSED {
            continue;
        }
        operate_door(ctx, door, true);
    }
}

/// The shared head of `ObjChangeMap` and `ObjChangeMapResync`: restores the pre-quest tiles and
/// returns the affected world-tile corners.
fn obj_change_map_tiles(ctx: &mut Ctx, x1: i32, y1: i32, x2: i32, y2: i32) -> (Point, Point) {
    for j in y1..=y2 {
        for i in x1..=x2 {
            let v = ctx.gendung.pdungeon[i as usize][j as usize];
            obj_set_mini(ctx, Point::new(i, j), v as i32);
            ctx.gendung.dungeon[i as usize][j as usize] = v;
        }
    }
    // WorldTileCoord is uint8_t
    let mega1 = Point::new(x1 & 0xff, y1 & 0xff);
    let mega2 = Point::new(x2 & 0xff, y2 & 0xff);
    let w1 = mega1.mega_to_world();
    let w2 = mega2.mega_to_world() + Displacement::new(1, 1);
    (Point::new(w1.x & 0xff, w1.y & 0xff), Point::new(w2.x & 0xff, w2.y & 0xff))
}

/// Original: `devilution::ObjChangeMap` (objects.cpp).
// @port objects.cpp|devilution::ObjChangeMap(int x1, int y1, int x2, int y2) sha=90cde1ca8c9b
pub fn obj_change_map(ctx: &mut Ctx, x1: i32, y1: i32, x2: i32, y2: i32) {
    let (world1, world2) = obj_change_map_tiles(ctx, x1, y1, x2, y2);
    let lt = ctx.gendung.leveltype;
    if lt == DungeonType::Cathedral {
        obj_l1_special(ctx, world1.x, world1.y, world2.x, world2.y);
        add_l1_objs(ctx, world1.x, world1.y, world2.x, world2.y);
    }
    if lt == DungeonType::Catacombs {
        obj_l2_special(ctx, world1.x, world1.y, world2.x, world2.y);
        add_l2_objs(ctx, world1.x, world1.y, world2.x, world2.y);
    }
    if lt == DungeonType::Caves {
        add_l3_objs(ctx, world1.x, world1.y, world2.x, world2.y);
    }
    if lt == DungeonType::Crypt {
        add_crypt_objects(ctx, world1.x, world1.y, world2.x, world2.y);
    }
    resync_doors(ctx, world1, world2, true);
}

/// Original: `devilution::ObjChangeMapResync` (objects.cpp).
// @port objects.cpp|devilution::ObjChangeMapResync(int x1, int y1, int x2, int y2) sha=2c1aa5cfbf7e
pub fn obj_change_map_resync(ctx: &mut Ctx, x1: i32, y1: i32, x2: i32, y2: i32) {
    let (world1, world2) = obj_change_map_tiles(ctx, x1, y1, x2, y2);
    let lt = ctx.gendung.leveltype;
    if lt == DungeonType::Cathedral {
        obj_l1_special(ctx, world1.x, world1.y, world2.x, world2.y);
    }
    if lt == DungeonType::Catacombs {
        obj_l2_special(ctx, world1.x, world1.y, world2.x, world2.y);
    }
    resync_doors(ctx, world1, world2, false);
}

/// Original: `devilution::ItemMiscIdIdx` (objects.cpp).
// @port objects.cpp|devilution::ItemMiscIdIdx(item_misc_id imiscid) sha=a3265b35f866
pub fn item_misc_id_idx(imiscid: item_misc_id) -> _item_indexes {
    use crate::tables::itemdat::AllItemsList;
    let mut i = IDI_GOLD as usize;
    while AllItemsList[i].iRnd == IDROP_NEVER || AllItemsList[i].iMiscId != imiscid {
        i += 1;
    }
    i as _item_indexes
}

/// Original: `devilution::OperateObject` (objects.cpp).
// @port objects.cpp|devilution::OperateObject(Player &player, Object &object) sha=11f4fd1fc2c6
pub fn operate_object(ctx: &mut Ctx, pnum: usize, oi: usize) {
    use crate::effects_data::*;
    let sendmsg = is_my_player(ctx, pnum);
    match ctx.objects.Objects[oi]._otype {
        OBJ_L1LDOOR | OBJ_L1RDOOR | OBJ_L2LDOOR | OBJ_L2RDOOR | OBJ_L3LDOOR | OBJ_L3RDOOR | OBJ_L5LDOOR | OBJ_L5RDOOR => {
            if sendmsg {
                operate_door(ctx, oi, sendmsg);
            }
        }
        OBJ_LEVER | OBJ_L5LEVER | OBJ_SWITCHSKL => operate_lever(ctx, oi, sendmsg),
        OBJ_BOOK2L => {
            if sendmsg {
                operate_book(ctx, pnum, oi, sendmsg);
            }
        }
        OBJ_BOOK2R => operate_chamber_of_bone_book(ctx, oi, sendmsg),
        OBJ_CHEST1 | OBJ_CHEST2 | OBJ_CHEST3 | OBJ_TCHEST1 | OBJ_TCHEST2 | OBJ_TCHEST3 => operate_chest(ctx, pnum, oi, sendmsg),
        OBJ_SARC | OBJ_L5SARC => operate_sarcophagus(ctx, oi, sendmsg, sendmsg),
        OBJ_FLAMELVR => operate_trap_lever(ctx, oi),
        OBJ_BLINDBOOK | OBJ_BLOODBOOK | OBJ_STEELTOME => {
            if sendmsg {
                operate_book_lever(ctx, oi, sendmsg);
            }
        }
        OBJ_SHRINEL | OBJ_SHRINER => operate_shrine(ctx, pnum, oi, IS_MAGIC),
        OBJ_SKELBOOK | OBJ_BOOKSTAND => operate_book_stand(ctx, oi, sendmsg, sendmsg),
        OBJ_BOOKCASEL | OBJ_BOOKCASER => operate_bookcase(ctx, oi, sendmsg, sendmsg),
        OBJ_DECAP => operate_decapitated_body(ctx, oi, sendmsg, sendmsg),
        OBJ_ARMORSTAND | OBJ_WARARMOR => operate_armor_stand(ctx, oi, sendmsg, sendmsg),
        OBJ_GOATSHRINE => operate_goat_shrine(ctx, pnum, oi, LS_GSHRINE),
        OBJ_CAULDRON => operate_cauldron(ctx, pnum, oi, LS_CALDRON),
        OBJ_BLOODFTN | OBJ_PURIFYINGFTN | OBJ_MURKYFTN | OBJ_TEARFTN => {
            operate_fountains(ctx, pnum, oi);
        }
        OBJ_STORYBOOK | OBJ_L5BOOKS => {
            if sendmsg {
                operate_story_book(ctx, oi);
            }
        }
        OBJ_PEDESTAL => {
            if sendmsg {
                operate_pedestal(ctx, pnum, oi, sendmsg);
            }
        }
        OBJ_WARWEAP | OBJ_WEAPONRACK => operate_weapon_rack(ctx, oi, sendmsg, sendmsg),
        OBJ_MUSHPATCH => operate_mushroom_patch(ctx, pnum, oi),
        OBJ_LAZSTAND => {
            if sendmsg {
                operate_laz_stand(ctx, oi);
            }
        }
        OBJ_SLAINHERO => operate_slain_hero(ctx, pnum, oi, sendmsg),
        OBJ_SIGNCHEST => operate_inn_sign_chest(ctx, pnum, oi, sendmsg),
        _ => {}
    }
}

/// Original: `devilution::DeltaSyncOpObject` (objects.cpp).
// @port objects.cpp|devilution::DeltaSyncOpObject(Object &object) sha=e3f9ba1ffce0
pub fn delta_sync_op_object(ctx: &mut Ctx, oi: usize) {
    let o = ctx.objects.Objects[oi].clone();
    let frame = o._oAnimFrame as i32;
    match o._otype {
        OBJ_L1LDOOR | OBJ_L1RDOOR | OBJ_L2LDOOR | OBJ_L2RDOOR | OBJ_L3LDOOR | OBJ_L3RDOOR | OBJ_L5LDOOR | OBJ_L5RDOOR => open_door(ctx, oi),
        OBJ_LEVER | OBJ_L5LEVER | OBJ_SWITCHSKL | OBJ_BOOK2L => update_lever_state(ctx, oi),
        OBJ_CHEST1 | OBJ_CHEST2 | OBJ_CHEST3 | OBJ_TCHEST1 | OBJ_TCHEST2 | OBJ_TCHEST3 | OBJ_SKELBOOK | OBJ_BOOKSTAND => update_state(ctx, oi, frame + 2),
        OBJ_SARC | OBJ_L5SARC | OBJ_GOATSHRINE | OBJ_SHRINEL | OBJ_SHRINER => update_state(ctx, oi, o._oAnimLen as i32),
        OBJ_BLINDBOOK | OBJ_BLOODBOOK | OBJ_STEELTOME | OBJ_BOOK2R => {
            ctx.objects.Objects[oi]._oAnimFrame = o._oVar6;
            sync_qst_lever(ctx, oi);
        }
        OBJ_BOOKCASEL | OBJ_BOOKCASER => update_state(ctx, oi, frame - 2),
        OBJ_DECAP | OBJ_MURKYFTN | OBJ_TEARFTN | OBJ_SLAINHERO => update_state(ctx, oi, frame),
        OBJ_ARMORSTAND | OBJ_WARARMOR | OBJ_WARWEAP | OBJ_WEAPONRACK | OBJ_LAZSTAND => update_state(ctx, oi, frame + 1),
        OBJ_CAULDRON => update_state(ctx, oi, 3),
        OBJ_STORYBOOK | OBJ_L5BOOKS => ctx.objects.Objects[oi]._oAnimFrame = o._oVar4 as u32,
        OBJ_MUSHPATCH => {
            if ctx.quests.Quests[Q_MUSHROOM as usize]._qvar1 >= QS_MUSHSPAWNED as u8 {
                update_state(ctx, oi, frame + 1);
            }
        }
        OBJ_SIGNCHEST => {
            if ctx.quests.Quests[Q_LTBANNER as usize]._qvar1 >= 2 {
                update_state(ctx, oi, frame + 2);
            }
        }
        OBJ_PEDESTAL => update_pedestal_state(ctx, oi),
        _ => {}
    }
}

/// Original: `devilution::DeltaSyncCloseObj` (objects.cpp).
// @port objects.cpp|devilution::DeltaSyncCloseObj(Object &object) sha=9a0cfe929699
pub fn delta_sync_close_obj(ctx: &mut Ctx, oi: usize) {
    // Object was closed.
    // That means it was opened once, so all traps have been activated.
    ctx.objects.Objects[oi]._oTrapFlag = false;
}

/// Original: `devilution::SyncOpObject` (objects.cpp).
// @port objects.cpp|devilution::SyncOpObject(Player &player, int cmd, Object &object) sha=592a1bddeee6
pub fn sync_op_object(ctx: &mut Ctx, pnum: usize, cmd: i32, oi: usize) {
    use crate::effects_data::*;
    let sendmsg = is_my_player(ctx, pnum);
    match ctx.objects.Objects[oi]._otype {
        OBJ_L1LDOOR | OBJ_L1RDOOR | OBJ_L2LDOOR | OBJ_L2RDOOR | OBJ_L3LDOOR | OBJ_L3RDOOR | OBJ_L5LDOOR | OBJ_L5RDOOR => {
            let v4 = ctx.objects.Objects[oi]._oVar4;
            if sendmsg || (cmd == CMD_CLOSEDOOR as i32 && v4 == DOOR_CLOSED) || (cmd == CMD_OPENDOOR as i32 && v4 == DOOR_OPEN) {
                return;
            }
            operate_door(ctx, oi, false);
        }
        OBJ_LEVER | OBJ_L5LEVER | OBJ_SWITCHSKL => operate_lever(ctx, oi, sendmsg),
        OBJ_BOOK2L => {
            if !sendmsg {
                operate_book(ctx, pnum, oi, sendmsg);
            }
        }
        OBJ_CHEST1 | OBJ_CHEST2 | OBJ_CHEST3 | OBJ_TCHEST1 | OBJ_TCHEST2 | OBJ_TCHEST3 => operate_chest(ctx, pnum, oi, false),
        OBJ_SARC | OBJ_L5SARC => operate_sarcophagus(ctx, oi, sendmsg, false),
        OBJ_BLINDBOOK | OBJ_BLOODBOOK | OBJ_STEELTOME => {
            if sendmsg {
                return;
            }
            ctx.objects.Objects[oi]._oAnimFrame = ctx.objects.Objects[oi]._oVar6;
            sync_qst_lever(ctx, oi);
        }
        OBJ_SHRINEL | OBJ_SHRINER => operate_shrine(ctx, pnum, oi, IS_MAGIC),
        OBJ_SKELBOOK | OBJ_BOOKSTAND => operate_book_stand(ctx, oi, sendmsg, false),
        OBJ_BOOKCASEL | OBJ_BOOKCASER => operate_bookcase(ctx, oi, sendmsg, false),
        OBJ_DECAP => operate_decapitated_body(ctx, oi, sendmsg, false),
        OBJ_ARMORSTAND | OBJ_WARARMOR => operate_armor_stand(ctx, oi, sendmsg, false),
        OBJ_GOATSHRINE => operate_goat_shrine(ctx, pnum, oi, LS_GSHRINE),
        OBJ_LAZSTAND => {
            if !sendmsg {
                let f = ctx.objects.Objects[oi]._oAnimFrame as i32 + 1;
                update_state(ctx, oi, f);
            }
        }
        OBJ_CAULDRON => operate_cauldron(ctx, pnum, oi, LS_CALDRON),
        OBJ_MURKYFTN | OBJ_TEARFTN => {
            operate_fountains(ctx, pnum, oi);
        }
        OBJ_STORYBOOK | OBJ_L5BOOKS => {
            if sendmsg {
                operate_story_book(ctx, oi);
            }
        }
        OBJ_PEDESTAL => {
            if !sendmsg {
                operate_pedestal(ctx, pnum, oi, sendmsg);
            }
        }
        OBJ_WARWEAP | OBJ_WEAPONRACK => operate_weapon_rack(ctx, oi, sendmsg, false),
        OBJ_MUSHPATCH => operate_mushroom_patch(ctx, pnum, oi),
        OBJ_SLAINHERO => operate_slain_hero(ctx, pnum, oi, sendmsg),
        OBJ_SIGNCHEST => operate_inn_sign_chest(ctx, pnum, oi, sendmsg),
        _ => {}
    }
}

/// Original: `devilution::BreakObjectMissile` (objects.cpp).
// @port objects.cpp|devilution::BreakObjectMissile(const Player *player, Object &object) sha=be48bf86756d
pub fn break_object_missile(ctx: &mut Ctx, _pnum: Option<usize>, oi: usize) {
    if ctx.objects.Objects[oi].is_crux() {
        break_crux(ctx, oi, true);
    }
}

/// Original: `devilution::BreakObject` (objects.cpp).
// @port objects.cpp|devilution::BreakObject(const Player &player, Object &object) sha=41abe8a8bc8a
pub fn break_object(ctx: &mut Ctx, pnum: usize, oi: usize) {
    if ctx.objects.Objects[oi].is_barrel() {
        break_barrel(ctx, pnum, oi, false, true);
    } else if ctx.objects.Objects[oi].is_crux() {
        break_crux(ctx, oi, true);
    }
}

/// Original: `devilution::DeltaSyncBreakObj` (objects.cpp).
// @port objects.cpp|devilution::DeltaSyncBreakObj(Object &object) sha=7453f0665b55
pub fn delta_sync_break_obj(ctx: &mut Ctx, oi: usize) {
    {
        let object = &mut ctx.objects.Objects[oi];
        if !object.is_breakable() || object._oSelFlag == 0 {
            return;
        }
        object._oMissFlag = true;
        object._oBreak = -1;
        object._oSelFlag = 0;
        object._oPreFlag = true;
        object._oAnimFlag = 0;
        object._oAnimFrame = object._oAnimLen;
    }
    let o = ctx.objects.Objects[oi].clone();
    if o.is_barrel() {
        ctx.objects.Objects[oi]._oSolidFlag = false;
    } else if o.is_crux() && are_all_cruxes_of_type_broken(ctx, o._oVar8) {
        obj_change_map(ctx, o._oVar1, o._oVar2, o._oVar3, o._oVar4);
    }
}

/// Original: `devilution::SyncBreakObj` (objects.cpp).
// @port objects.cpp|devilution::SyncBreakObj(const Player &player, Object &object) sha=a788b2c72a99
pub fn sync_break_obj(ctx: &mut Ctx, pnum: usize, oi: usize) {
    if ctx.objects.Objects[oi].is_barrel() {
        break_barrel(ctx, pnum, oi, true, false);
    } else if ctx.objects.Objects[oi].is_crux() {
        break_crux(ctx, oi, false);
    }
}

/// Original: `devilution::SyncObjectAnim` (objects.cpp).
// @port objects.cpp|devilution::SyncObjectAnim(Object &object) sha=639f1f084731
pub fn sync_object_anim(ctx: &mut Ctx, oi: usize) {
    let index = AllObjects[ctx.objects.Objects[oi]._otype as usize].ofindex;
    if !ctx.diablo.headless_mode {
        let Some(i) = ctx.objects.ObjFileList.iter().position(|&f| f == index) else {
            crate::platform::log::error!("Unable to find object_graphic_id {} in list of objects to load, level generation error.", index);
            return;
        };
        ctx.objects.Objects[oi]._oAnimData = ctx.objects.p_obj_cels[i].clone();
    }
    match ctx.objects.Objects[oi]._otype {
        OBJ_L1LDOOR | OBJ_L1RDOOR | OBJ_L2LDOOR | OBJ_L2RDOOR | OBJ_L3LDOOR | OBJ_L3RDOOR | OBJ_L5LDOOR | OBJ_L5RDOOR => sync_door(ctx, oi),
        OBJ_CRUX1 | OBJ_CRUX2 | OBJ_CRUX3 => sync_crux(ctx, oi),
        OBJ_LEVER | OBJ_L5LEVER | OBJ_BOOK2L | OBJ_SWITCHSKL => sync_lever(ctx, oi),
        OBJ_BOOK2R | OBJ_BLINDBOOK | OBJ_STEELTOME => sync_qst_lever(ctx, oi),
        OBJ_PEDESTAL => sync_pedestal(ctx, oi),
        _ => {}
    }
}

/// Original: `devilution::Object::name` (objects.cpp).
// @port objects.cpp|devilution::Object::name() sha=307079e382eb
pub fn object_name(ctx: &Ctx, oi: usize) -> String {
    let o = &ctx.objects.Objects[oi];
    let s = match o._otype {
        OBJ_CRUX1 | OBJ_CRUX2 | OBJ_CRUX3 => "Crucified Skeleton",
        OBJ_LEVER | OBJ_L5LEVER | OBJ_FLAMELVR => "Lever",
        OBJ_L1LDOOR | OBJ_L1RDOOR | OBJ_L2LDOOR | OBJ_L2RDOOR | OBJ_L3LDOOR | OBJ_L3RDOOR | OBJ_L5LDOOR | OBJ_L5RDOOR => match o._oVar4 {
            DOOR_OPEN => "Open Door",
            DOOR_CLOSED => "Closed Door",
            DOOR_BLOCKED => "Blocked Door",
            _ => return String::new(),
        },
        OBJ_BOOK2L => {
            if ctx.gendung.setlevel && ctx.gendung.setlvlnum == SL_BONECHAMB {
                "Ancient Tome"
            } else if ctx.gendung.setlevel && ctx.gendung.setlvlnum == SL_VILEBETRAYER {
                "Book of Vileness"
            } else {
                return String::new();
            }
        }
        OBJ_SWITCHSKL => "Skull Lever",
        OBJ_BOOK2R => "Mythical Book",
        OBJ_CHEST1 | OBJ_TCHEST1 => "Small Chest",
        OBJ_CHEST2 | OBJ_TCHEST2 => "Chest",
        OBJ_CHEST3 | OBJ_TCHEST3 | OBJ_SIGNCHEST => "Large Chest",
        OBJ_SARC | OBJ_L5SARC => "Sarcophagus",
        OBJ_BOOKSHELF => "Bookshelf",
        OBJ_BOOKCASEL | OBJ_BOOKCASER => "Bookcase",
        OBJ_BARREL | OBJ_BARRELEX => "Barrel",
        OBJ_POD | OBJ_PODEX => "Pod",
        OBJ_URN | OBJ_URNEX => "Urn",
        OBJ_SHRINEL | OBJ_SHRINER => {
            return tr("{:s} Shrine").replacen("{:s}", &tr(SHRINE_NAMES[o._oVar1 as usize]), 1);
        }
        OBJ_SKELBOOK => "Skeleton Tome",
        OBJ_BOOKSTAND => "Library Book",
        OBJ_BLOODFTN => "Blood Fountain",
        OBJ_DECAP => "Decapitated Body",
        OBJ_BLINDBOOK => "Book of the Blind",
        OBJ_BLOODBOOK => "Book of Blood",
        OBJ_PURIFYINGFTN => "Purifying Spring",
        OBJ_ARMORSTAND | OBJ_WARARMOR => "Armor",
        OBJ_WARWEAP => "Weapon Rack",
        OBJ_GOATSHRINE => "Goat Shrine",
        OBJ_CAULDRON => "Cauldron",
        OBJ_MURKYFTN => "Murky Pool",
        OBJ_TEARFTN => "Fountain of Tears",
        OBJ_STEELTOME => "Steel Tome",
        OBJ_PEDESTAL => "Pedestal of Blood",
        OBJ_STORYBOOK | OBJ_L5BOOKS => STORY_BOOK_NAME[o._oVar3 as usize],
        OBJ_WEAPONRACK => "Weapon Rack",
        OBJ_MUSHPATCH => "Mushroom Patch",
        OBJ_LAZSTAND => "Vile Stand",
        OBJ_SLAINHERO => "Slain Hero",
        _ => return String::new(),
    };
    tr(s)
}

/// Original: `devilution::GetObjectStr` (objects.cpp): describes the highlighted object in the info box.
// @port objects.cpp|devilution::GetObjectStr(const Object &object) sha=f04064604ad8
pub fn get_object_str(ctx: &mut Ctx, oi: usize) {
    use crate::engine::render::text_render::UiFlags;
    ctx.control.info_string = object_name(ctx, oi);
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    if ctx.players.Players[me]._pClass == HeroClass::Rogue && ctx.objects.Objects[oi]._oTrapFlag {
        ctx.control.info_string = tr("Trapped {:s}").replacen("{:s}", &ctx.control.info_string, 1);
        ctx.control.info_color = UiFlags::COLOR_RED;
    }
    if ctx.objects.Objects[oi].is_disabled(ctx) {
        ctx.control.info_string = tr("{:s} (disabled)").replacen("{:s}", &ctx.control.info_string, 1);
        ctx.control.info_color = UiFlags::COLOR_RED;
    }
}

/// Original: `devilution::SyncNakrulRoom` (objects.cpp).
// @port objects.cpp|devilution::SyncNakrulRoom() sha=6e6ea4c45c88
pub fn sync_nakrul_room(ctx: &mut Ctx) {
    let (r, c) = (ctx.crypt.UberRow as usize, ctx.crypt.UberCol as usize);
    let d = &mut ctx.gendung.dPiece;
    d[r][c] = 297;
    d[r][c - 1] = 300;
    d[r][c - 2] = 299;
    d[r][c + 1] = 298;
}
