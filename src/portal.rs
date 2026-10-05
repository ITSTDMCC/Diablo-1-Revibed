//! `Source/portal.cpp`: town portals.

use crate::ctx::Ctx;
use crate::engine::geometry::{Direction, Displacement, Point};
use crate::enums::*;
use crate::levels::gendung::DungeonType;

pub const MAXPORTAL: usize = 4;

/// `Portal`
#[derive(Clone, Copy, Debug)]
pub struct Portal {
    pub open: bool,
    pub position: Point,
    pub level: i32,
    pub ltype: DungeonType,
    pub setlvl: bool,
}

impl Default for Portal {
    fn default() -> Self {
        Portal { open: false, position: Point::default(), level: 0, ltype: DungeonType::Town, setlvl: false }
    }
}

/// Globals of portal.cpp.
#[derive(Default)]
pub struct PortalState {
    pub Portals: [Portal; MAXPORTAL],
    /// `portalindex`
    pub portalindex: usize,
}

/// `WarpDrop`: where each player's portal appears in town.
const WARP_DROP: [Point; MAXPORTAL] = [Point::new(57, 40), Point::new(59, 40), Point::new(61, 40), Point::new(63, 40)];

/// Original: `devilution::InitPortals` (portal.cpp).
// @port portal.cpp|devilution::InitPortals() sha=e690c08b15d0
pub fn init_portals(ctx: &mut Ctx) {
    for portal in ctx.portal.Portals.iter_mut() {
        portal.open = false;
    }
}

/// Original: `devilution::SetPortalStats` (portal.cpp).
// @port portal.cpp|devilution::SetPortalStats(int i, bool o, Point position, int lvl, dungeon_type lvltype, bool isSetLevel) sha=9f4e0312e1ed
pub fn set_portal_stats(ctx: &mut Ctx, i: usize, o: bool, position: Point, lvl: i32, lvltype: DungeonType, is_set_level: bool) {
    let p = &mut ctx.portal.Portals[i];
    p.open = o;
    p.position = position;
    p.level = lvl;
    p.ltype = lvltype;
    p.setlvl = is_set_level;
}

/// Original: `devilution::AddWarpMissile` (portal.cpp).
// @port portal.cpp|devilution::AddWarpMissile(int i, Point position, bool sync) sha=7e2c996872ae
pub fn add_warp_missile(ctx: &mut Ctx, i: usize, position: Point, sync: bool) {
    let missile = crate::missiles::add_missile_sfx(
        ctx,
        Point::new(0, 0),
        position,
        Direction::South,
        MissileID::TownPortal,
        crate::missiles::TARGET_MONSTERS,
        i as i32,
        0,
        0,
        None,
        Some(crate::effects_data::SFX_NONE),
    );
    if let Some(mi) = missile {
        if sync {
            crate::missiles::set_miss_dir(ctx, mi, 1);
        }
        if ctx.gendung.leveltype != DungeonType::Town {
            let tile = ctx.missiles.Missiles[mi].position.tile;
            ctx.missiles.Missiles[mi]._mlid = crate::lighting::add_light(ctx, tile, 15);
        }
    }
}

/// Original: `devilution::SyncPortals` (portal.cpp).
// @port portal.cpp|devilution::SyncPortals() sha=5384fb7abd20
pub fn sync_portals(ctx: &mut Ctx) {
    for i in 0..MAXPORTAL {
        if !ctx.portal.Portals[i].open {
            continue;
        }
        if ctx.gendung.leveltype == DungeonType::Town {
            add_warp_missile(ctx, i, WARP_DROP[i], true);
        } else {
            let mut lvl = ctx.gendung.currlevel as i32;
            if ctx.gendung.setlevel {
                lvl = ctx.gendung.setlvlnum as i32;
            }
            let p = ctx.portal.Portals[i];
            if p.level == lvl && p.setlvl == ctx.gendung.setlevel {
                add_warp_missile(ctx, i, p.position, true);
            }
        }
    }
}

/// Original: `devilution::AddInTownPortal` (portal.cpp).
// @port portal.cpp|devilution::AddInTownPortal(int i) sha=bcfb37083986
pub fn add_in_town_portal(ctx: &mut Ctx, i: usize) {
    add_warp_missile(ctx, i, WARP_DROP[i], false);
}

/// Original: `devilution::ActivatePortal` (portal.cpp).
// @port portal.cpp|devilution::ActivatePortal(int i, Point position, int lvl, dungeon_type dungeonType, bool isSetLevel) sha=3f7d0e758e17
pub fn activate_portal(ctx: &mut Ctx, i: usize, position: Point, lvl: i32, dungeon_type: DungeonType, is_set_level: bool) {
    let p = &mut ctx.portal.Portals[i];
    p.open = true;
    if lvl != 0 {
        p.position = position;
        p.level = lvl;
        p.ltype = dungeon_type;
        p.setlvl = is_set_level;
    }
}

/// Original: `devilution::DeactivatePortal` (portal.cpp).
// @port portal.cpp|devilution::DeactivatePortal(int i) sha=e5b75638b30d
pub fn deactivate_portal(ctx: &mut Ctx, i: usize) {
    ctx.portal.Portals[i].open = false;
}

/// Original: `devilution::PortalOnLevel` (portal.cpp). The condition is the original's
/// precedence: `(setlvl == setlevel && level == setlevel) ? setlvlnum : currlevel`.
// @port portal.cpp|devilution::PortalOnLevel(size_t i) sha=ed8bc4e9e460
pub fn portal_on_level(ctx: &Ctx, i: usize) -> bool {
    let p = &ctx.portal.Portals[i];
    let setlevel = ctx.gendung.setlevel;
    let v = if p.setlvl == setlevel && p.level == setlevel as i32 { ctx.gendung.setlvlnum as i32 } else { ctx.gendung.currlevel as i32 };
    if v != 0 {
        return true;
    }
    ctx.gendung.leveltype == DungeonType::Town
}

/// Original: `devilution::RemovePortalMissile` (portal.cpp).
// @port portal.cpp|devilution::RemovePortalMissile(int id) sha=9c9ee26f042f
pub fn remove_portal_missile(ctx: &mut Ctx, id: usize) {
    let mut i = 0;
    while i < ctx.missiles.Missiles.len() {
        let m = &ctx.missiles.Missiles[i];
        if m._mitype == MissileID::TownPortal && m._misource == id as i32 {
            let tile = m.position.tile;
            let mlid = m._mlid;
            ctx.gendung.dFlags[tile.x as usize][tile.y as usize].0 &= !DungeonFlag::Missile.0;
            if ctx.portal.Portals[id].level != 0 {
                crate::lighting::add_un_light(ctx, mlid);
            }
            ctx.missiles.Missiles.remove(i);
            continue;
        }
        i += 1;
    }
}

/// Original: `devilution::SetCurrentPortal` (portal.cpp).
// @port portal.cpp|devilution::SetCurrentPortal(size_t p) sha=5a6d40f441ab
pub fn set_current_portal(ctx: &mut Ctx, p: usize) {
    ctx.portal.portalindex = p;
}

/// Original: `devilution::GetPortalLevel` (portal.cpp).
// @port portal.cpp|devilution::GetPortalLevel() sha=cbe21b53ad2c
pub fn get_portal_level(ctx: &mut Ctx) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    if ctx.gendung.leveltype != DungeonType::Town {
        ctx.gendung.setlevel = false;
        ctx.gendung.currlevel = 0;
        ctx.players.Players[me].set_level(0);
        ctx.gendung.leveltype = DungeonType::Town;
        return;
    }
    let pi = ctx.portal.portalindex;
    let p = ctx.portal.Portals[pi];
    if p.setlvl {
        ctx.gendung.setlevel = true;
        ctx.gendung.setlvlnum = p.level as _setlevels;
        ctx.gendung.currlevel = p.level as u8;
        let s = ctx.gendung.setlvlnum;
        ctx.players.Players[me].set_set_level(s);
        ctx.gendung.leveltype = p.ltype;
        ctx.gendung.setlvltype = p.ltype;
    } else {
        ctx.gendung.setlevel = false;
        ctx.gendung.currlevel = p.level as u8;
        let c = ctx.gendung.currlevel;
        ctx.players.Players[me].set_level(c);
        ctx.gendung.leveltype = p.ltype;
    }
    if pi == ctx.players.MyPlayerId {
        crate::msg::net_send_cmd(ctx, true, CMD_DEACTIVATEPORTAL);
        deactivate_portal(ctx, pi);
    }
}

/// Original: `devilution::GetPortalLvlPos` (portal.cpp).
// @port portal.cpp|devilution::GetPortalLvlPos() sha=beafbba8c8ac
pub fn get_portal_lvl_pos(ctx: &mut Ctx) {
    let pi = ctx.portal.portalindex;
    if ctx.gendung.leveltype == DungeonType::Town {
        ctx.gendung.ViewPosition = WARP_DROP[pi] + Displacement::new(1, 1);
    } else {
        ctx.gendung.ViewPosition = ctx.portal.Portals[pi].position;
        if pi != ctx.players.MyPlayerId {
            ctx.gendung.ViewPosition.x += 1;
            ctx.gendung.ViewPosition.y += 1;
        }
    }
}

/// Original: `devilution::PosOkPortal` (portal.cpp).
// @port portal.cpp|devilution::PosOkPortal(int lvl, Point position) sha=992dd4ed80af
pub fn pos_ok_portal(ctx: &Ctx, lvl: i32, position: Point) -> bool {
    for portal in ctx.portal.Portals.iter() {
        if portal.open && portal.level == lvl && (portal.position == position || portal.position == position - Displacement::new(1, 1)) {
            return true;
        }
    }
    false
}
