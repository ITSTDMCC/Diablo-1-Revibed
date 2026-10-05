//! `Source/sync.cpp`: monster and item synchronisation packets between players.

use crate::ctx::Ctx;
use crate::engine::geometry::Point;
use crate::enums::*;
use crate::monster::MaxMonsters;
use crate::multi::MAX_PLRS;

/// `sizeof(TSyncHeader)`
pub const SYNC_HEADER_SIZE: usize = 36;
/// `sizeof(TSyncMonster)`
pub const SYNC_MONSTER_SIZE: usize = 10;

/// `TSyncMonster`
#[derive(Clone, Copy, Debug, Default)]
pub struct TSyncMonster {
    pub _mndx: u8,
    pub _mx: u8,
    pub _my: u8,
    pub _menemy: u8,
    pub _mdelta: u8,
    pub _mhitpoints: i32,
    pub mWhoHit: i8,
}

impl TSyncMonster {
    pub fn to_bytes(&self) -> [u8; SYNC_MONSTER_SIZE] {
        let mut b = [0u8; SYNC_MONSTER_SIZE];
        b[0] = self._mndx;
        b[1] = self._mx;
        b[2] = self._my;
        b[3] = self._menemy;
        b[4] = self._mdelta;
        b[5..9].copy_from_slice(&self._mhitpoints.to_le_bytes());
        b[9] = self.mWhoHit as u8;
        b
    }

    pub fn from_bytes(b: &[u8]) -> TSyncMonster {
        let g = |i: usize| b.get(i).copied().unwrap_or(0);
        TSyncMonster {
            _mndx: g(0),
            _mx: g(1),
            _my: g(2),
            _menemy: g(3),
            _mdelta: g(4),
            _mhitpoints: i32::from_le_bytes([g(5), g(6), g(7), g(8)]),
            mWhoHit: g(9) as i8,
        }
    }
}

/// Globals of sync.cpp.
pub struct SyncState {
    sgn_monster_priority: [u16; MaxMonsters],
    sgn_monsters: usize,
    sgw_lru: [u16; MaxMonsters],
    sgn_sync_item: i32,
    sgn_sync_p_inv: i32,
}

impl Default for SyncState {
    fn default() -> Self {
        SyncState { sgn_monster_priority: [0; MaxMonsters], sgn_monsters: 0, sgw_lru: [0; MaxMonsters], sgn_sync_item: 0, sgn_sync_p_inv: 0 }
    }
}

/// Original: `SyncOneMonster` (sync.cpp).
// @port sync.cpp|devilution::SyncOneMonster() sha=920c099d0996
fn sync_one_monster(ctx: &mut Ctx) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let my_tile = ctx.players.Players[me].position.tile;
    for i in 0..ctx.monster.ActiveMonsterCount {
        let m = ctx.monster.ActiveMonsters[i] as usize;
        let monster = &ctx.monster.Monsters[m];
        ctx.sync.sgn_monster_priority[m] = my_tile.manhattan_distance(monster.position.tile) as u16;
        if monster.activeForTicks == 0 {
            ctx.sync.sgn_monster_priority[m] = ctx.sync.sgn_monster_priority[m].wrapping_add(0x1000);
        } else if ctx.sync.sgw_lru[m] != 0 {
            ctx.sync.sgw_lru[m] -= 1;
        }
    }
}

/// Original: `SyncMonsterPos` (sync.cpp).
// @port sync.cpp|devilution::SyncMonsterPos(TSyncMonster &monsterSync, int ndx) sha=9b808c0419f2
fn sync_monster_pos(ctx: &mut Ctx, monster_sync: &mut TSyncMonster, ndx: usize) {
    let enemy = crate::monster::encode_enemy(ctx, ndx);
    let monster = &ctx.monster.Monsters[ndx];
    monster_sync._mndx = ndx as u8;
    monster_sync._mx = monster.position.tile.x as u8;
    monster_sync._my = monster.position.tile.y as u8;
    monster_sync._menemy = enemy;
    let prio = ctx.sync.sgn_monster_priority[ndx];
    monster_sync._mdelta = if prio > 255 { 255 } else { prio as u8 };
    monster_sync.mWhoHit = monster.whoHit;
    monster_sync._mhitpoints = monster.hitPoints;
    ctx.sync.sgn_monster_priority[ndx] = 0xFFFF;
    ctx.sync.sgw_lru[ndx] = if monster.activeForTicks == 0 { 0xFFFF } else { 0xFFFE };
}

/// Original: `SyncMonsterActive` (sync.cpp).
// @port sync.cpp|devilution::SyncMonsterActive(TSyncMonster &monsterSync) sha=d419192d5ade
fn sync_monster_active(ctx: &mut Ctx, monster_sync: &mut TSyncMonster) -> bool {
    let mut ndx: i32 = -1;
    let mut lru: u32 = 0xFFFFFFFF;
    for i in 0..ctx.monster.ActiveMonsterCount {
        let m = ctx.monster.ActiveMonsters[i] as usize;
        if (ctx.sync.sgn_monster_priority[m] as u32) < lru && ctx.sync.sgw_lru[m] < 0xFFFE {
            lru = ctx.sync.sgn_monster_priority[m] as u32;
            ndx = ctx.monster.ActiveMonsters[i];
        }
    }
    if ndx == -1 {
        return false;
    }
    sync_monster_pos(ctx, monster_sync, ndx as usize);
    true
}

/// Original: `SyncMonsterActive2` (sync.cpp).
// @port sync.cpp|devilution::SyncMonsterActive2(TSyncMonster &monsterSync) sha=e48e604d18ee
fn sync_monster_active2(ctx: &mut Ctx, monster_sync: &mut TSyncMonster) -> bool {
    let mut ndx: i32 = -1;
    let mut lru: u32 = 0xFFFE;
    for _ in 0..ctx.monster.ActiveMonsterCount {
        if ctx.sync.sgn_monsters >= ctx.monster.ActiveMonsterCount {
            ctx.sync.sgn_monsters = 0;
        }
        let m = ctx.monster.ActiveMonsters[ctx.sync.sgn_monsters] as usize;
        if (ctx.sync.sgw_lru[m] as u32) < lru {
            lru = ctx.sync.sgw_lru[m] as u32;
            ndx = ctx.monster.ActiveMonsters[ctx.sync.sgn_monsters];
        }
        ctx.sync.sgn_monsters += 1;
    }
    if ndx == -1 {
        return false;
    }
    sync_monster_pos(ctx, monster_sync, ndx as usize);
    true
}

/// Original: `SyncPlrInv` (sync.cpp). Fills the item fields of the sync header in `hdr`.
// @port sync.cpp|devilution::SyncPlrInv(TSyncHeader *pHdr) sha=c73cfb388433
fn sync_plr_inv(ctx: &mut Ctx, hdr: &mut [u8]) {
    let put16 = |h: &mut [u8], o: usize, v: u16| h[o..o + 2].copy_from_slice(&v.to_le_bytes());
    let put32 = |h: &mut [u8], o: usize, v: u32| h[o..o + 4].copy_from_slice(&v.to_le_bytes());
    hdr[4] = 0xFF; // bItemI = -1
    if ctx.items.ActiveItemCount > 0 {
        if ctx.sync.sgn_sync_item >= ctx.items.ActiveItemCount as i32 {
            ctx.sync.sgn_sync_item = 0;
        }
        hdr[4] = ctx.items.ActiveItems[ctx.sync.sgn_sync_item as usize];
        ctx.sync.sgn_sync_item += 1;
        let item = &ctx.items.Items[hdr[4] as usize];
        hdr[5] = item.position.x as u8;
        hdr[6] = item.position.y as u8;
        put16(hdr, 7, item.IDidx as u16);
        if item.IDidx == IDI_EAR {
            // `char` is signed in the original build
            let n = item._iIName.bytes();
            let c = |i: usize| n[i] as i8 as i32;
            put16(hdr, 9, ((c(0) << 8) | c(1)) as u16);
            put32(hdr, 11, ((c(2) << 24) | (c(3) << 16) | (c(4) << 8) | c(5)) as u32);
            hdr[15] = n[6];
            hdr[16] = n[7];
            hdr[17] = n[8];
            hdr[18] = n[9];
            hdr[19] = n[10];
            put16(hdr, 20, ((c(11) << 8) | ((item._iCurs as i32 - ICURS_EAR_SORCERER as i32) << 6) | item._ivalue) as u16);
            put32(hdr, 22, ((c(12) << 24) | (c(13) << 16) | (c(14) << 8) | c(15)) as u32);
        } else {
            put16(hdr, 9, item._iCreateInfo);
            put32(hdr, 11, item._iSeed);
            hdr[15] = item._iIdentified as u8;
            hdr[16] = item._iDurability as u8;
            hdr[17] = item._iMaxDur as u8;
            hdr[18] = item._iCharges as u8;
            hdr[19] = item._iMaxCharges as u8;
            if item.IDidx == IDI_GOLD {
                put16(hdr, 20, item._ivalue as u16);
            }
        }
    }

    hdr[26] = 0xFF; // bPInvLoc = -1
    assert!(ctx.sync.sgn_sync_p_inv > -1 && ctx.sync.sgn_sync_p_inv < NUM_INVLOC as i32);
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let item = &ctx.players.Players[me].InvBody[ctx.sync.sgn_sync_p_inv as usize];
    if !item.is_empty() {
        hdr[26] = ctx.sync.sgn_sync_p_inv as u8;
        put16(hdr, 27, item.IDidx as u16);
        put16(hdr, 29, item._iCreateInfo);
        put32(hdr, 31, item._iSeed);
        hdr[35] = item._iIdentified as u8;
    }
    ctx.sync.sgn_sync_p_inv += 1;
    if ctx.sync.sgn_sync_p_inv >= NUM_INVLOC as i32 {
        ctx.sync.sgn_sync_p_inv = 0;
    }
}

/// Original: `SyncMonster` (sync.cpp).
// @port sync.cpp|devilution::SyncMonster(bool isOwner, const TSyncMonster &monsterSync) sha=755b681200d7
fn sync_monster(ctx: &mut Ctx, is_owner: bool, monster_sync: &TSyncMonster) {
    let monster_id = monster_sync._mndx as usize;
    {
        let monster = &ctx.monster.Monsters[monster_id];
        if monster.hitPoints <= 0 || monster.mode == MonsterMode::Death {
            return;
        }
    }
    let position = Point::new(monster_sync._mx as i32, monster_sync._my as i32);
    let enemy_id = monster_sync._menemy as i32;
    let me = ctx.players.MyPlayer.expect("MyPlayer");

    if ctx.monster.Monsters[monster_id].activeForTicks != 0 {
        let mut delta = ctx.players.Players[me].position.tile.manhattan_distance(ctx.monster.Monsters[monster_id].position.tile) as u32;
        if delta > 255 {
            delta = 255;
        }
        if delta < monster_sync._mdelta as u32 || (delta == monster_sync._mdelta as u32 && is_owner) {
            return;
        }
        if ctx.monster.Monsters[monster_id].position.future == position {
            return;
        }
    }
    if matches!(ctx.monster.Monsters[monster_id].mode, MonsterMode::Charge | MonsterMode::Petrified) {
        return;
    }

    if ctx.monster.Monsters[monster_id].position.tile.walking_distance(position) <= 2 {
        if !crate::monster::is_walking(ctx, monster_id) {
            let md = crate::engine::get_direction(ctx.monster.Monsters[monster_id].position.tile, position);
            if crate::monster::dir_ok(ctx, monster_id, md) {
                crate::monster::m_clear_squares(ctx, monster_id);
                let t = ctx.monster.Monsters[monster_id].position.tile;
                ctx.gendung.dMonster[t.x as usize][t.y as usize] = monster_id as i16 + 1;
                crate::monster::walk(ctx, monster_id, md);
                ctx.monster.Monsters[monster_id].activeForTicks = u8::MAX;
            }
        }
    } else if ctx.gendung.dMonster[position.x as usize][position.y as usize] == 0 {
        crate::monster::m_clear_squares(ctx, monster_id);
        ctx.gendung.dMonster[position.x as usize][position.y as usize] = monster_id as i16 + 1;
        ctx.monster.Monsters[monster_id].position.tile = position;
        let light_id = ctx.monster.Monsters[monster_id].lightId;
        if light_id as i32 != crate::lighting::NO_LIGHT {
            crate::lighting::change_light_xy(ctx, light_id as i32, position);
        }
        crate::monster::decode_enemy(ctx, monster_id, enemy_id);
        let md = crate::engine::get_direction(position, ctx.monster.Monsters[monster_id].enemyPosition);
        crate::monster::m_start_stand(ctx, monster_id, md);
        ctx.monster.Monsters[monster_id].activeForTicks = u8::MAX;
    }

    crate::monster::decode_enemy(ctx, monster_id, enemy_id);
    ctx.monster.Monsters[monster_id].whoHit |= monster_sync.mWhoHit;
}

/// Original: `IsEnemyIdValid` (sync.cpp).
// @port sync.cpp|devilution::IsEnemyIdValid(const Monster &monster, int enemyId) sha=1add391c6ee2
fn is_enemy_id_valid(ctx: &Ctx, monster: usize, mut enemy_id: i32) -> bool {
    if enemy_id < 0 {
        return false;
    }
    if enemy_id < MAX_PLRS as i32 {
        return ctx.players.Players.get(enemy_id as usize).is_some_and(|p| p.plractive);
    }
    enemy_id -= MAX_PLRS as i32;
    if enemy_id as usize >= MaxMonsters {
        return false;
    }
    if enemy_id as usize == monster {
        return false;
    }
    if ctx.monster.Monsters[enemy_id as usize].hitPoints <= 0 {
        return false;
    }
    true
}

/// Original: `IsTSyncMonsterValidate` (sync.cpp).
// @port sync.cpp|devilution::IsTSyncMonsterValidate(const TSyncMonster &monsterSync) sha=472a933a419d
fn is_t_sync_monster_validate(ctx: &Ctx, monster_sync: &TSyncMonster) -> bool {
    let monster_id = monster_sync._mndx as usize;
    if monster_id >= MaxMonsters {
        return false;
    }
    if !crate::levels::gendung::in_dungeon_bounds(Point::new(monster_sync._mx as i32, monster_sync._my as i32)) {
        return false;
    }
    if !is_enemy_id_valid(ctx, monster_id, monster_sync._menemy as i32) {
        return false;
    }
    true
}

/// Original: `devilution::sync_all_monsters` (sync.cpp). Writes into `buf` and returns the
/// space left (`dwMaxLen` minus what was written).
// @port sync.cpp|devilution::sync_all_monsters(byte *pbBuf, uint32_t dwMaxLen) sha=127239f8c752
pub fn sync_all_monsters(ctx: &mut Ctx, buf: &mut [u8], mut dw_max_len: u32) -> u32 {
    if ctx.monster.ActiveMonsterCount < 1 {
        return dw_max_len;
    }
    if (dw_max_len as usize) < SYNC_HEADER_SIZE + SYNC_MONSTER_SIZE {
        return dw_max_len;
    }
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    if ctx.players.Players[me]._pLvlChanging {
        return dw_max_len;
    }

    // Header fields SyncPlrInv leaves unset hold stack garbage in the original (the TPkt is
    // uninitialised); they start as zero here. Receivers ignore them.
    let mut hdr = [0u8; SYNC_HEADER_SIZE];
    dw_max_len -= SYNC_HEADER_SIZE as u32;
    hdr[0] = CMD_SYNCDATA;
    hdr[1] = crate::msg::get_level_for_multiplayer(ctx, me);
    let mut w_len: u16 = 0;
    sync_plr_inv(ctx, &mut hdr);
    assert!(dw_max_len <= 0xffff);
    sync_one_monster(ctx);

    let mut off = SYNC_HEADER_SIZE;
    let mut i = 0;
    while i < ctx.monster.ActiveMonsterCount && dw_max_len as usize >= SYNC_MONSTER_SIZE {
        let mut monster_sync = TSyncMonster::from_bytes(&buf[off..off + SYNC_MONSTER_SIZE]);
        let mut sync = false;
        if i < 2 {
            sync = sync_monster_active2(ctx, &mut monster_sync);
        }
        if !sync {
            sync = sync_monster_active(ctx, &mut monster_sync);
        }
        if !sync {
            break;
        }
        buf[off..off + SYNC_MONSTER_SIZE].copy_from_slice(&monster_sync.to_bytes());
        off += SYNC_MONSTER_SIZE;
        w_len += SYNC_MONSTER_SIZE as u16;
        dw_max_len -= SYNC_MONSTER_SIZE as u32;
        i += 1;
    }
    hdr[2..4].copy_from_slice(&w_len.to_le_bytes());
    buf[..SYNC_HEADER_SIZE].copy_from_slice(&hdr);
    dw_max_len
}

/// Original: `devilution::OnSyncData` (sync.cpp).
// @port sync.cpp|devilution::OnSyncData(const TCmd *pCmd, size_t pnum) sha=9441ad76bd38
pub fn on_sync_data(ctx: &mut Ctx, cmd: &[u8], pnum: usize) -> usize {
    let g = |i: usize| cmd.get(i).copied().unwrap_or(0);
    let w_len = u16::from_le_bytes([g(2), g(3)]) as usize;
    assert!(ctx.msg.gbBufferMsgs != 2);
    if ctx.msg.gbBufferMsgs == 1 {
        return w_len + SYNC_HEADER_SIZE;
    }
    if pnum == ctx.players.MyPlayerId {
        return w_len + SYNC_HEADER_SIZE;
    }
    assert!(w_len % SYNC_MONSTER_SIZE == 0);
    let monster_count = w_len / SYNC_MONSTER_SIZE;
    let level = g(1);
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let sync_local_level = !ctx.players.Players[me]._pLvlChanging && crate::msg::get_level_for_multiplayer(ctx, me) == level;

    if crate::msg::is_valid_level_for_multiplayer(level) {
        for i in 0..monster_count {
            let start = SYNC_HEADER_SIZE + i * SYNC_MONSTER_SIZE;
            let monster_sync = TSyncMonster::from_bytes(cmd.get(start..).unwrap_or(&[]));
            if !is_t_sync_monster_validate(ctx, &monster_sync) {
                continue;
            }
            if sync_local_level {
                sync_monster(ctx, pnum > ctx.players.MyPlayerId, &monster_sync);
            }
            crate::msg::delta_sync_monster(ctx, &monster_sync, level);
        }
    }
    w_len + SYNC_HEADER_SIZE
}

/// Original: `devilution::sync_init` (sync.cpp).
// @port sync.cpp|devilution::sync_init() sha=93a687deb7e4
pub fn sync_init(ctx: &mut Ctx) {
    ctx.sync.sgn_monsters = 16 * ctx.players.MyPlayerId;
    ctx.sync.sgw_lru = [0xFFFF; MaxMonsters];
}
