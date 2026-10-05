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
crate::pending_fn!(pub fn find_object_at_position(ctx: &Ctx, position: Point, consider_large_objects: bool) -> Option<usize>, "objects.cpp|devilution::FindObjectAtPosition(Point position, bool considerLargeObjects)");
crate::pending_fn!(pub fn is_object_at_position(ctx: &Ctx, position: Point) -> bool, "objects.cpp|devilution::IsObjectAtPosition(Point position)");
crate::pending_fn!(pub fn is_item_blocking_object_at_position(ctx: &Ctx, position: Point) -> bool, "objects.cpp|devilution::IsItemBlockingObjectAtPosition(Point position)");
crate::pending_fn!(pub fn break_object(ctx: &mut Ctx, pnum: usize, oi: usize), "objects.cpp|devilution::BreakObject(const Player &player, Object &object)");
crate::pending_fn!(pub fn operate_object(ctx: &mut Ctx, pnum: usize, oi: usize), "objects.cpp|devilution::OperateObject(Player &player, Object &object)");

crate::pending_fn!(pub fn sync_object_anim(ctx: &mut Ctx, oi: usize), "objects.cpp|devilution::SyncObjectAnim(Object &object)");
