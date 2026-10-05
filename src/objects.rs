//! `Source/objects.cpp`: dungeon objects (data model; most logic pending).

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::geometry::Point;
use crate::enums::*;

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
    pub fn is_broken(&self) -> bool {
        self._oBreak == -1
    }
    pub fn is_barrel(&self) -> bool {
        matches!(self._otype, OBJ_BARREL | OBJ_BARRELEX | OBJ_POD | OBJ_PODEX | OBJ_URN | OBJ_URNEX)
    }
    pub fn is_explosive(&self) -> bool {
        matches!(self._otype, OBJ_BARRELEX | OBJ_PODEX | OBJ_URNEX)
    }
    pub fn is_chest(&self) -> bool {
        matches!(self._otype, OBJ_CHEST1 | OBJ_CHEST2 | OBJ_CHEST3 | OBJ_TCHEST1 | OBJ_TCHEST2 | OBJ_TCHEST3)
    }
    pub fn is_trapped_chest(&self) -> bool {
        matches!(self._otype, OBJ_TCHEST1 | OBJ_TCHEST2 | OBJ_TCHEST3) && self._oTrapFlag
    }
    pub fn is_untrapped_chest(&self) -> bool {
        matches!(self._otype, OBJ_CHEST1 | OBJ_CHEST2 | OBJ_CHEST3) && !self._oTrapFlag
    }
    pub fn is_crux(&self) -> bool {
        matches!(self._otype, OBJ_CRUX1 | OBJ_CRUX2 | OBJ_CRUX3)
    }
    pub fn is_door(&self) -> bool {
        matches!(self._otype, OBJ_L1LDOOR | OBJ_L1RDOOR | OBJ_L2LDOOR | OBJ_L2RDOOR | OBJ_L3LDOOR | OBJ_L3RDOOR | OBJ_L5LDOOR | OBJ_L5RDOOR)
    }
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

crate::pending_fn!(pub fn set_map_objects(ctx: &mut Ctx, dun_data: &[u16], startx: i32, starty: i32), "objects.cpp|devilution::SetMapObjects(const uint16_t *dunData, int startx, int starty)");
crate::pending_fn!(pub fn break_object(ctx: &mut Ctx, pnum: usize, oi: usize), "objects.cpp|devilution::BreakObject(const Player &player, Object &object)");
crate::pending_fn!(pub fn operate_object(ctx: &mut Ctx, pnum: usize, oi: usize), "objects.cpp|devilution::OperateObject(Player &player, Object &object)");

crate::pending_fn!(pub fn sync_object_anim(ctx: &mut Ctx, oi: usize), "objects.cpp|devilution::SyncObjectAnim(Object &object)");

crate::pending_fn!(pub fn add_object(ctx: &mut Ctx, obj_type: _object_id, obj_pos: Point) -> Option<usize>, "objects.cpp|devilution::AddObject(_object_id objType, Point objPos)");
crate::pending_fn!(pub fn obj_change_map_resync(ctx: &mut Ctx, x1: i32, y1: i32, x2: i32, y2: i32), "objects.cpp|devilution::ObjChangeMapResync(int x1, int y1, int x2, int y2)");
crate::pending_fn!(pub fn sync_op_object(ctx: &mut Ctx, pnum: usize, cmd: i32, oi: usize), "objects.cpp|devilution::SyncOpObject(Player &player, int cmd, Object &object)");
crate::pending_fn!(pub fn sync_break_obj(ctx: &mut Ctx, pnum: usize, oi: usize), "objects.cpp|devilution::SyncBreakObj(const Player &player, Object &object)");
crate::pending_fn!(pub fn delta_sync_op_object(ctx: &mut Ctx, oi: usize), "objects.cpp|devilution::DeltaSyncOpObject(Object &object)");
crate::pending_fn!(pub fn delta_sync_close_obj(ctx: &mut Ctx, oi: usize), "objects.cpp|devilution::DeltaSyncCloseObj(Object &object)");
crate::pending_fn!(pub fn delta_sync_break_obj(ctx: &mut Ctx, oi: usize), "objects.cpp|devilution::DeltaSyncBreakObj(Object &object)");
crate::pending_fn!(pub fn update_trap_state(ctx: &mut Ctx, oi: usize), "objects.cpp|devilution::UpdateTrapState(Object &trap)");
crate::pending_fn!(pub fn sync_nakrul_room(ctx: &mut Ctx), "objects.cpp|devilution::SyncNakrulRoom()");

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

crate::pending_fn!(pub fn process_objects(ctx: &mut crate::ctx::Ctx), "objects.cpp|devilution::ProcessObjects()");

crate::pending_fn!(pub fn init_objects(ctx: &mut crate::ctx::Ctx), "objects.cpp|devilution::InitObjects()");

crate::pending_fn!(pub fn init_object_gfx(ctx: &mut crate::ctx::Ctx), "objects.cpp|devilution::InitObjectGFX()");

crate::pending_fn!(pub fn get_object_str(ctx: &mut crate::ctx::Ctx, oi: usize), "objects.cpp|devilution::GetObjectStr(const Object &object)");

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
