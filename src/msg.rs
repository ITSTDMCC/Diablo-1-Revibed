//! `Source/msg.cpp`: game commands (sent through the network layer even in single player, where
//! the loopback provider hands them straight back) and the multiplayer level delta state.
//!
//! The commands are `#pragma pack(1)` structs in the original; here they are byte buffers with
//! the same layout, read and written through the offset helpers below.

use std::collections::HashMap;

use crate::ctx::Ctx;
use crate::engine::geometry::{Direction, Point};
use crate::enums::*;
use crate::items::{Item, MAXITEMS};
use crate::levels::gendung::{in_dungeon_bounds, DungeonType, DMAXX, DMAXY};
use crate::monster::MaxMonsters;
use crate::multi::{net_send_hi_pri, net_send_lo_pri, MAX_PLRS};
use crate::platform::log;
use crate::player::{InventoryGridCells, MaxBeltItems, MaxCharacterLevel, MaxSpellLevel, NUMLEVELS};
use crate::portal::MAXPORTAL;
use crate::quests::MAXQUESTS;
use crate::storm::storm_net::*;
use crate::sync::TSyncMonster;
use crate::utils::language::tr;

pub const MAX_SEND_STR_LEN: usize = 80;

// Packed struct sizes.
pub const SIZE_TCMD: usize = 1;
pub const SIZE_TCMDLOC: usize = 3;
pub const SIZE_TCMDLOCPARAM1: usize = 5;
pub const SIZE_TCMDLOCPARAM2: usize = 7;
pub const SIZE_TCMDLOCPARAM3: usize = 9;
pub const SIZE_TCMDLOCPARAM4: usize = 11;
pub const SIZE_TCMDLOCPARAM5: usize = 13;
pub const SIZE_TCMDPARAM1: usize = 3;
pub const SIZE_TCMDPARAM2: usize = 5;
pub const SIZE_TCMDPARAM5: usize = 11;
pub const SIZE_TCMDGOLEM: usize = 10;
pub const SIZE_TCMDQUEST: usize = 8;
/// the item/ear/def union
pub const SIZE_NETITEM: usize = 26;
pub const SIZE_TCMDGITEM: usize = 3 + SIZE_NETITEM + 8;
pub const SIZE_TCMDPITEM: usize = 3 + SIZE_NETITEM;
pub const SIZE_TCMDCHITEM: usize = 3 + SIZE_NETITEM;
pub const SIZE_TCMDDELITEM: usize = 2;
pub const SIZE_TCMDDAMAGE: usize = 7;
pub const SIZE_TCMDMONDAMAGE: usize = 7;
pub const SIZE_TCMDPLRINFOHDR: usize = 5;
pub const SIZE_TFAKECMDPLR: usize = 2;
pub const SIZE_TFAKEDROPPLR: usize = 6;

/// `TCmdPItem::FloorItem`, `PickedUpItem`, `DroppedItem`
const FloorItem: _cmd_id = CMD_STAND;
const PickedUpItem: _cmd_id = CMD_WALKXY;
const DroppedItem: _cmd_id = CMD_ACK_PLRINFO;

const MAX_MULTIPLAYERLEVELS: usize = NUMLEVELS + SL_LAST as usize;
const MAX_CHUNKS: usize = MAX_MULTIPLAYERLEVELS + 4;

/// sizeof(DMonsterStr)
const SIZE_DMONSTERSTR: usize = 9;
/// sizeof(DPortal)
const SIZE_DPORTAL: usize = 5;
/// sizeof(MultiQuests)
const SIZE_MULTIQUESTS: usize = 6;

/// Reads little-endian fields of a received command (bytes past the end read as zero).
#[derive(Clone, Copy)]
pub struct CmdView<'a>(pub &'a [u8]);

impl<'a> CmdView<'a> {
    pub fn u8(&self, o: usize) -> u8 {
        self.0.get(o).copied().unwrap_or(0)
    }
    pub fn i8(&self, o: usize) -> i8 {
        self.u8(o) as i8
    }
    pub fn u16(&self, o: usize) -> u16 {
        u16::from_le_bytes([self.u8(o), self.u8(o + 1)])
    }
    pub fn i16(&self, o: usize) -> i16 {
        self.u16(o) as i16
    }
    pub fn u32(&self, o: usize) -> u32 {
        u32::from_le_bytes([self.u8(o), self.u8(o + 1), self.u8(o + 2), self.u8(o + 3)])
    }
    pub fn i32(&self, o: usize) -> i32 {
        self.u32(o) as i32
    }
    pub fn bytes(&self, o: usize, n: usize) -> Vec<u8> {
        (0..n).map(|i| self.u8(o + i)).collect()
    }
    pub fn cmd(&self) -> _cmd_id {
        self.u8(0)
    }
}

fn put16(b: &mut [u8], o: usize, v: u16) {
    b[o..o + 2].copy_from_slice(&v.to_le_bytes());
}

fn put32(b: &mut [u8], o: usize, v: u32) {
    b[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

/// The `TItemDef`/`TItem`/`TEar` union (26 bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetItem(pub [u8; SIZE_NETITEM]);

impl Default for NetItem {
    fn default() -> Self {
        NetItem([0; SIZE_NETITEM])
    }
}

impl NetItem {
    pub fn from_slice(b: &[u8]) -> NetItem {
        let mut n = [0u8; SIZE_NETITEM];
        for (i, v) in n.iter_mut().enumerate() {
            *v = b.get(i).copied().unwrap_or(0);
        }
        NetItem(n)
    }
    fn v(&self) -> CmdView<'_> {
        CmdView(&self.0)
    }
    pub fn w_indx(&self) -> _item_indexes {
        self.v().i16(0)
    }
    pub fn w_ci(&self) -> u16 {
        self.v().u16(2)
    }
    pub fn dw_seed(&self) -> u32 {
        self.v().u32(4)
    }
    pub fn b_id(&self) -> u8 {
        self.0[8]
    }
    pub fn b_dur(&self) -> u8 {
        self.0[9]
    }
    pub fn b_m_dur(&self) -> u8 {
        self.0[10]
    }
    pub fn b_ch(&self) -> u8 {
        self.0[11]
    }
    pub fn b_m_ch(&self) -> u8 {
        self.0[12]
    }
    pub fn w_value(&self) -> u16 {
        self.v().u16(13)
    }
    pub fn dw_buff(&self) -> u32 {
        self.v().u32(15)
    }
    pub fn w_to_hit(&self) -> u16 {
        self.v().u16(19)
    }
    pub fn w_max_dam(&self) -> u16 {
        self.v().u16(21)
    }
    pub fn b_cursval(&self) -> u8 {
        self.0[8]
    }
    /// `ear.heroname` up to the terminator
    pub fn heroname(&self) -> String {
        let n = &self.0[9..26];
        let end = n.iter().position(|&b| b == 0).unwrap_or(17);
        String::from_utf8_lossy(&n[..end]).into_owned()
    }
    pub fn set_def(&mut self, w_indx: _item_indexes, w_ci: u16, dw_seed: u32) {
        put16(&mut self.0, 0, w_indx as u16);
        put16(&mut self.0, 2, w_ci);
        put32(&mut self.0, 4, dw_seed);
    }
}

/// `DMonsterStr`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DMonsterStr {
    x: u8,
    y: u8,
    menemy: u8,
    mactive: u8,
    hit_points: i32,
    who_hit: i8,
}

impl DMonsterStr {
    const INVALID: DMonsterStr = DMonsterStr { x: 0xFF, y: 0xFF, menemy: 0xFF, mactive: 0xFF, hit_points: -1, who_hit: -1 };

    fn to_bytes(self) -> [u8; SIZE_DMONSTERSTR] {
        let mut b = [0u8; SIZE_DMONSTERSTR];
        b[0] = self.x;
        b[1] = self.y;
        b[2] = self.menemy;
        b[3] = self.mactive;
        b[4..8].copy_from_slice(&self.hit_points.to_le_bytes());
        b[8] = self.who_hit as u8;
        b
    }

    fn from_bytes(b: &[u8]) -> DMonsterStr {
        let v = CmdView(b);
        DMonsterStr { x: v.u8(0), y: v.u8(1), menemy: v.u8(2), mactive: v.u8(3), hit_points: v.i32(4), who_hit: v.i8(8) }
    }
}

/// `DLevel`
#[derive(Clone)]
struct DLevel {
    /// `TCmdPItem item[MAXITEMS]` as raw records
    item: Vec<[u8; SIZE_TCMDPITEM]>,
    /// `std::unordered_map<WorldTilePosition, DObjectStr>`, kept in insertion order (see NOTES)
    object: Vec<((u8, u8), _cmd_id)>,
    monster: Vec<DMonsterStr>,
}

impl DLevel {
    fn new() -> DLevel {
        DLevel { item: vec![[0xFF; SIZE_TCMDPITEM]; MAXITEMS], object: Vec::new(), monster: vec![DMonsterStr::INVALID; MaxMonsters] }
    }

    fn object_set(&mut self, pos: (u8, u8), cmd: _cmd_id) {
        match self.object.iter_mut().find(|(p, _)| *p == pos) {
            Some(e) => e.1 = cmd,
            None => self.object.push((pos, cmd)),
        }
    }
}

/// `DPortal`
#[derive(Clone, Copy, Debug)]
struct DPortal {
    x: u8,
    y: u8,
    level: u8,
    ltype: u8,
    setlvl: u8,
}

/// `MultiQuests`
#[derive(Clone, Copy, Debug)]
struct MultiQuests {
    qstate: quest_state,
    qlog: u8,
    qvar1: u8,
    qvar2: u8,
    qmsg: i16,
}

/// `DJunk`
#[derive(Clone, Copy, Debug)]
struct DJunk {
    portal: [DPortal; MAXPORTAL],
    quests: [MultiQuests; MAXQUESTS],
}

impl DJunk {
    /// `memset(&sgJunk, 0xFF, sizeof(sgJunk))`
    const ALL_FF: DJunk = DJunk {
        portal: [DPortal { x: 0xFF, y: 0xFF, level: 0xFF, ltype: 0xFF, setlvl: 0xFF }; MAXPORTAL],
        quests: [MultiQuests { qstate: 0xFF, qlog: 0xFF, qvar1: 0xFF, qvar2: 0xFF, qmsg: -1 }; MAXQUESTS],
    };
}

/// `TMegaPkt`
struct TMegaPkt {
    space_left: usize,
    data: Vec<u8>,
}

const MEGA_PKT_SIZE: usize = 32000;

impl TMegaPkt {
    fn new() -> TMegaPkt {
        TMegaPkt { space_left: MEGA_PKT_SIZE, data: vec![0; MEGA_PKT_SIZE] }
    }
}

/// Size of `sgRecvBuf`.
const RECV_BUF_SIZE: usize = 1 + MAXITEMS * SIZE_TCMDPITEM + 1 + (2 + 1) * crate::objects::MAXOBJECTS + MaxMonsters * SIZE_DMONSTERSTR;

/// Globals of msg.cpp.
pub struct MsgState {
    /// `gbBufferMsgs`
    pub gbBufferMsgs: u8,
    pub dwRecCount: i32,
    sgdw_owner_wait: u32,
    sgdw_recv_offset: u32,
    sgn_curr_mega_player: i32,
    delta_levels: HashMap<u8, DLevel>,
    sb_last_cmd: u8,
    sg_recv_buf: Vec<u8>,
    sgb_recv_cmd: _cmd_id,
    local_levels: HashMap<u8, Box<[[u8; DMAXY]; DMAXX]>>,
    sg_junk: DJunk,
    sgb_delta_chunks: u8,
    mega_pkt_list: Vec<TMegaPkt>,
    item_limbo: Item,
    /// `lastSentPlayerCmd` (a TCmdLocParam5)
    last_sent_player_cmd: [u8; SIZE_TCMDLOCPARAM5],
}

impl Default for MsgState {
    fn default() -> Self {
        MsgState {
            gbBufferMsgs: 0,
            dwRecCount: 0,
            sgdw_owner_wait: 0,
            sgdw_recv_offset: 0,
            sgn_curr_mega_player: 0,
            delta_levels: HashMap::new(),
            sb_last_cmd: 0,
            sg_recv_buf: vec![0; RECV_BUF_SIZE],
            sgb_recv_cmd: 0,
            local_levels: HashMap::new(),
            sg_junk: DJunk::ALL_FF,
            sgb_delta_chunks: 0,
            mega_pkt_list: Vec::new(),
            item_limbo: Item::default(),
            last_sent_player_cmd: [0; SIZE_TCMDLOCPARAM5],
        }
    }
}

/// Original: `devilution::EventFailedPacket` (msg.cpp).
// @port msg.cpp|devilution::EventFailedPacket(const char *playerName) sha=df53f0382649
fn event_failed_packet(ctx: &mut Ctx, player_name: &str) {
    let message = format!("Player '{player_name}' sent an invalid packet.");
    crate::plrmsg::event_plr_msg(ctx, &message);
}

/// `ValidateField` / `ValidateFields`
fn validate(ctx: &mut Ctx, pnum: usize, ok: bool, what: &str) -> bool {
    if !ok {
        log::verbose!("Remote player packet validation failed: {}", what);
        let name = ctx.players.Players[pnum]._pName.as_str().to_string();
        event_failed_packet(ctx, &name);
    }
    ok
}

/// Original: `GetLevelForMultiplayer(uint8_t level, bool isSetLevel)` (msg.cpp).
// @port msg.cpp|devilution::GetLevelForMultiplayer(uint8_t level, bool isSetLevel) sha=19e57ef5498a
fn get_level_for_multiplayer_raw(level: u8, is_set_level: bool) -> u8 {
    if is_set_level {
        return level.wrapping_add(NUMLEVELS as u8);
    }
    level
}

/// Original: `GetDeltaLevel(uint8_t level)` (msg.cpp).
// @port msg.cpp|devilution::GetDeltaLevel(uint8_t level) sha=e378db2a8091
fn get_delta_level(ctx: &mut Ctx, level: u8) -> &mut DLevel {
    ctx.msg.delta_levels.entry(level).or_insert_with(DLevel::new)
}

/// Original: `GetItemPosition` (msg.cpp).
// @port msg.cpp|devilution::GetItemPosition(Point position) sha=d6961298719f
fn get_item_position(ctx: &Ctx, position: Point) -> Point {
    if crate::inv::can_put(ctx, position) {
        return position;
    }
    for k in 1..50 {
        for j in -k..=k {
            let yy = position.y + j;
            for l in -k..=k {
                let xx = position.x + l;
                if crate::inv::can_put(ctx, Point::new(xx, yy)) {
                    return Point::new(xx, yy);
                }
            }
        }
    }
    position
}

/// Original: `WasPlayerCmdAlreadyRequested` (msg.cpp).
// @port msg.cpp|devilution::WasPlayerCmdAlreadyRequested(_cmd_id bCmd, Point position = {}, uint16_t wParam1 = 0, uint16_t wParam2 = 0, uint16_t wParam3 = 0, uint16_t wParam4 = 0, uint16_t wParam5 = 0) sha=caa590dbf90c
#[allow(clippy::too_many_arguments)]
fn was_player_cmd_already_requested(ctx: &mut Ctx, b_cmd: _cmd_id, position: Point, w1: u16, w2: u16, w3: u16, w4: u16, w5: u16) -> bool {
    match b_cmd {
        CMD_RATTACKID | CMD_SPELLID | CMD_ATTACKID | CMD_RATTACKPID | CMD_SPELLPID | CMD_ATTACKPID | CMD_ATTACKXY | CMD_SATTACKXY | CMD_RATTACKXY | CMD_SPELLXY
        | CMD_SPELLXYD | CMD_WALKXY | CMD_TALKXY | CMD_DISARMXY | CMD_OPOBJXY | CMD_GOTOGETITEM | CMD_GOTOAGETITEM => {}
        _ => return false,
    }
    let mut new_send = [0u8; SIZE_TCMDLOCPARAM5];
    new_send[0] = b_cmd;
    new_send[1] = position.x as u8;
    new_send[2] = position.y as u8;
    put16(&mut new_send, 3, w1);
    put16(&mut new_send, 5, w2);
    put16(&mut new_send, 7, w3);
    put16(&mut new_send, 9, w4);
    put16(&mut new_send, 11, w5);
    if ctx.msg.last_sent_player_cmd == new_send {
        return true;
    }
    ctx.msg.last_sent_player_cmd = new_send;
    false
}

/// Original: `GetNextPacket` (msg.cpp).
// @port msg.cpp|devilution::GetNextPacket() sha=eca54b6a8de0
fn get_next_packet(ctx: &mut Ctx) {
    ctx.msg.mega_pkt_list.push(TMegaPkt::new());
}

/// Original: `FreePackets` (msg.cpp).
// @port msg.cpp|devilution::FreePackets() sha=a7c59476178d
fn free_packets(ctx: &mut Ctx) {
    ctx.msg.mega_pkt_list.clear();
}

/// Original: `PrePacket` (msg.cpp).
// @port msg.cpp|devilution::PrePacket() sha=52e63e648a65
fn pre_packet(ctx: &mut Ctx) {
    let mut player_id: u8 = u8::MAX;
    let list = std::mem::take(&mut ctx.msg.mega_pkt_list);
    for pkt in list.iter() {
        let mut pos = 0usize;
        let mut space_left = MEGA_PKT_SIZE;
        while space_left != pkt.space_left {
            let data = &pkt.data[pos..];
            let cmd_id = data[0];
            if cmd_id == FAKE_CMD_SETID {
                pos += SIZE_TFAKECMDPLR;
                space_left -= SIZE_TFAKECMDPLR;
                player_id = data[1];
                continue;
            }
            if cmd_id == FAKE_CMD_DROPID {
                let v = CmdView(data);
                pos += SIZE_TFAKEDROPPLR;
                space_left -= SIZE_TFAKEDROPPLR;
                crate::multi::multi_player_left(ctx, v.u8(1) as usize, v.i32(2));
                continue;
            }
            if player_id as usize >= ctx.players.Players.len() {
                log::info!("Missing source of network message");
                ctx.msg.mega_pkt_list = list;
                return;
            }
            let size = parse_cmd(ctx, player_id as usize, &pkt.data[pos..MEGA_PKT_SIZE - pkt.space_left]);
            if size == 0 {
                log::info!("Discarding bad network message");
                ctx.msg.mega_pkt_list = list;
                return;
            }
            pos += size;
            space_left -= size;
        }
    }
    ctx.msg.mega_pkt_list = list;
}

/// Original: `SendPacket` (msg.cpp): buffers a command while resyncing.
// @port msg.cpp|devilution::SendPacket(int pnum, const void *packet, size_t dwSize) sha=de7c84b5aca1
fn send_packet(ctx: &mut Ctx, pnum: i32, packet: &[u8]) {
    if pnum != ctx.msg.sgn_curr_mega_player {
        ctx.msg.sgn_curr_mega_player = pnum;
        let cmd = [FAKE_CMD_SETID, pnum as u8];
        send_packet(ctx, pnum, &cmd);
    }
    let dw_size = packet.len();
    if ctx.msg.mega_pkt_list.last().unwrap().space_left < dw_size {
        get_next_packet(ctx);
    }
    let curr = ctx.msg.mega_pkt_list.last_mut().unwrap();
    let off = MEGA_PKT_SIZE - curr.space_left;
    curr.data[off..off + dw_size].copy_from_slice(packet);
    curr.space_left -= dw_size;
}

/// Original: `WaitForTurns` (msg.cpp).
// @port msg.cpp|devilution::WaitForTurns() sha=cee2958fee9a
fn wait_for_turns(ctx: &mut Ctx) -> i32 {
    let mut turns = 0u32;
    if ctx.msg.sgb_delta_chunks == 0 {
        crate::nthread::nthread_send_and_recv_turn(ctx, 0, 0);
        if !snet_get_owner_turns_waiting(ctx, &mut turns) && serr_get_last_error(ctx) == STORM_ERROR_NOT_IN_GAME {
            return 100;
        }
        if ctx.platform.ticks().wrapping_sub(ctx.msg.sgdw_owner_wait) <= 2000 && turns < ctx.nthread.gdwTurnsInTransit {
            return 0;
        }
        ctx.msg.sgb_delta_chunks += 1;
    }
    crate::multi::multi_process_network_packets(ctx);
    crate::nthread::nthread_send_and_recv_turn(ctx, 0, 0);
    if crate::nthread::nthread_has_500ms_passed(ctx, None) {
        crate::nthread::nthread_recv_turns(ctx, None);
    }
    if ctx.multi.gbGameDestroyed {
        return 100;
    }
    if ctx.multi.gbDeltaSender as usize >= ctx.players.Players.len() {
        ctx.msg.sgb_delta_chunks = 0;
        ctx.msg.sgb_recv_cmd = CMD_DLEVEL_END;
        ctx.multi.gbDeltaSender = ctx.players.MyPlayerId as u8;
        crate::nthread::nthread_set_turn_upper_bit(ctx);
    }
    if ctx.msg.sgb_delta_chunks as usize == MAX_CHUNKS - 1 {
        ctx.msg.sgb_delta_chunks = MAX_CHUNKS as u8;
        return 99;
    }
    (100 * ctx.msg.sgb_delta_chunks as usize / MAX_CHUNKS) as i32
}

/// Original: `DeltaExportItem` (msg.cpp).
// @port msg.cpp|devilution::DeltaExportItem(byte *dst, const TCmdPItem *src) sha=bacc4cbac1b4
fn delta_export_item(dst: &mut Vec<u8>, src: &[[u8; SIZE_TCMDPITEM]]) {
    for item in src.iter().take(MAXITEMS) {
        if item[0] == CMD_INVALID {
            dst.push(0xFF);
        } else {
            dst.extend_from_slice(item);
        }
    }
}

/// Original: `DeltaImportItem` (msg.cpp).
// @port msg.cpp|devilution::DeltaImportItem(const byte *src, TCmdPItem *dst) sha=eeb9ce76b367
fn delta_import_item(src: &[u8], dst: &mut [[u8; SIZE_TCMDPITEM]]) -> usize {
    let mut size = 0usize;
    for item in dst.iter_mut().take(MAXITEMS) {
        if src.get(size).copied().unwrap_or(0) == 0xFF {
            *item = [0xFF; SIZE_TCMDPITEM];
            size += 1;
        } else {
            for (i, b) in item.iter_mut().enumerate() {
                *b = src.get(size + i).copied().unwrap_or(0);
            }
            size += SIZE_TCMDPITEM;
        }
    }
    size
}

/// Original: `DeltaExportObject` (msg.cpp).
// @port msg.cpp|devilution::DeltaExportObject(byte *dst, const std::unordered_map<WorldTilePosition, DObjectStr> &src) sha=e8e6bb1a563f
fn delta_export_object(dst: &mut Vec<u8>, src: &[((u8, u8), _cmd_id)]) {
    dst.push(src.len() as u8);
    for &((x, y), cmd) in src {
        dst.push(x);
        dst.push(y);
        dst.push(cmd);
    }
}

/// Original: `DeltaImportObjects` (msg.cpp). Returns the bytes consumed.
// @port msg.cpp|devilution::DeltaImportObjects(const byte *src, std::unordered_map<WorldTilePosition, DObjectStr> &dst) sha=8d145491d44d
fn delta_import_objects(src: &[u8], dst: &mut Vec<((u8, u8), _cmd_id)>) -> usize {
    dst.clear();
    let v = CmdView(src);
    let num_deltas = v.u8(0) as usize;
    let mut pos = 1;
    for _ in 0..num_deltas {
        let p = (v.u8(pos), v.u8(pos + 1));
        pos += 2;
        let cmd = v.u8(pos);
        pos += 1;
        match dst.iter_mut().find(|(q, _)| *q == p) {
            Some(e) => e.1 = cmd,
            None => dst.push((p, cmd)),
        }
    }
    pos
}

/// Original: `DeltaExportMonster` (msg.cpp).
// @port msg.cpp|devilution::DeltaExportMonster(byte *dst, const DMonsterStr *src) sha=ce8c4ee96af5
fn delta_export_monster(dst: &mut Vec<u8>, src: &[DMonsterStr]) {
    for m in src.iter().take(MaxMonsters) {
        if m.x == 0xFF {
            dst.push(0xFF);
        } else {
            dst.extend_from_slice(&m.to_bytes());
        }
    }
}

/// Original: `DeltaImportMonster` (msg.cpp).
// @port msg.cpp|devilution::DeltaImportMonster(const byte *src, DMonsterStr *dst) sha=be2179bbcf01
fn delta_import_monster(src: &[u8], dst: &mut [DMonsterStr]) {
    let mut size = 0usize;
    for m in dst.iter_mut().take(MaxMonsters) {
        if src.get(size).copied().unwrap_or(0) == 0xFF {
            *m = DMonsterStr::INVALID;
            size += 1;
        } else {
            *m = DMonsterStr::from_bytes(src.get(size..).unwrap_or(&[]));
            size += SIZE_DMONSTERSTR;
        }
    }
}

/// Original: `DeltaExportJunk` (msg.cpp).
// @port msg.cpp|devilution::DeltaExportJunk(byte *dst) sha=7f119e1bb99a
fn delta_export_junk(ctx: &mut Ctx, dst: &mut Vec<u8>) {
    for portal in ctx.msg.sg_junk.portal.iter() {
        if portal.x == 0xFF {
            dst.push(0xFF);
        } else {
            dst.extend_from_slice(&[portal.x, portal.y, portal.level, portal.ltype, portal.setlvl]);
        }
    }
    let mut q = 0;
    for qi in 0..ctx.quests.Quests.len() {
        let quest = ctx.quests.Quests[qi];
        if crate::quests::quest_data_is_single_player_only(quest._qidx) && crate::quests::use_multiplayer_quests(ctx) {
            continue;
        }
        let mq = &mut ctx.msg.sg_junk.quests[q];
        mq.qlog = quest._qlog as u8;
        mq.qstate = quest._qactive;
        mq.qvar1 = quest._qvar1;
        mq.qvar2 = quest._qvar2;
        mq.qmsg = quest._qmsg as i16;
        dst.extend_from_slice(&[mq.qstate, mq.qlog, mq.qvar1, mq.qvar2]);
        dst.extend_from_slice(&mq.qmsg.to_le_bytes());
        q += 1;
    }
}

/// Original: `DeltaImportJunk` (msg.cpp).
// @port msg.cpp|devilution::DeltaImportJunk(const byte *src) sha=e0c047c5e677
fn delta_import_junk(ctx: &mut Ctx, src: &[u8]) {
    let v = CmdView(src);
    let mut pos = 0;
    for i in 0..MAXPORTAL {
        if v.u8(pos) == 0xFF {
            ctx.msg.sg_junk.portal[i] = DPortal { x: 0xFF, y: 0xFF, level: 0xFF, ltype: 0xFF, setlvl: 0xFF };
            pos += 1;
        } else {
            ctx.msg.sg_junk.portal[i] = DPortal { x: v.u8(pos), y: v.u8(pos + 1), level: v.u8(pos + 2), ltype: v.u8(pos + 3), setlvl: v.u8(pos + 4) };
            pos += SIZE_DPORTAL;
        }
    }
    let mut q = 0;
    for qidx in 0..MAXQUESTS {
        if crate::quests::quest_data_is_single_player_only(qidx as quest_id) && crate::quests::use_multiplayer_quests(ctx) {
            continue;
        }
        ctx.msg.sg_junk.quests[q] = MultiQuests { qstate: v.u8(pos), qlog: v.u8(pos + 1), qvar1: v.u8(pos + 2), qvar2: v.u8(pos + 3), qmsg: v.i16(pos + 4) };
        pos += SIZE_MULTIQUESTS;
        q += 1;
    }
}

/// Original: `CompressData` (msg.cpp). `buffer[0]` is the marker byte; returns the size to send.
// @port msg.cpp|devilution::CompressData(byte *buffer, byte *end) sha=218c63c93d4f
fn compress_data(buffer: &mut Vec<u8>) -> usize {
    let size = buffer.len() - 1;
    let mut body = buffer[1..].to_vec();
    let pk_size = crate::encrypt::pkware_compress(&mut body, size);
    buffer[0] = if size != pk_size { 1 } else { 0 };
    buffer.truncate(1);
    buffer.extend_from_slice(&body[..pk_size]);
    pk_size + 1
}

/// Original: `DeltaImportData` (msg.cpp).
// @port msg.cpp|devilution::DeltaImportData(_cmd_id cmd, uint32_t recvOffset) sha=8f60cd1d1bb9
fn delta_import_data(ctx: &mut Ctx, cmd: _cmd_id, recv_offset: u32) {
    if ctx.msg.sg_recv_buf[0] != 0 {
        let mut body = ctx.msg.sg_recv_buf[1..].to_vec();
        let n = crate::encrypt::pkware_decompress(&mut body, recv_offset as usize, RECV_BUF_SIZE - 1);
        ctx.msg.sg_recv_buf[1..1 + n].copy_from_slice(&body[..n]);
    }
    let src = ctx.msg.sg_recv_buf[1..].to_vec();
    if cmd == CMD_DLEVEL_JUNK {
        delta_import_junk(ctx, &src);
    } else if cmd == CMD_DLEVEL {
        let i = src[0];
        let mut pos = 1;
        let mut level = get_delta_level(ctx, i).clone();
        pos += delta_import_item(&src[pos..], &mut level.item);
        pos += delta_import_objects(&src[pos..], &mut level.object);
        delta_import_monster(&src[pos..], &mut level.monster);
        ctx.msg.delta_levels.insert(i, level);
    } else {
        crate::appfat::app_fatal(ctx, &format!("Unkown network message type: {cmd}"));
    }
    ctx.msg.sgb_delta_chunks += 1;
}

/// Original: `OnLevelData` (msg.cpp).
// @port msg.cpp|devilution::OnLevelData(int pnum, const TCmd *pCmd) sha=6d2b9890216e
fn on_level_data(ctx: &mut Ctx, pnum: usize, cmd: &[u8]) -> usize {
    let v = CmdView(cmd);
    let b_cmd = v.cmd();
    let w_offset = v.u16(1);
    let w_bytes = v.u16(3);
    let size = w_bytes as usize + SIZE_TCMDPLRINFOHDR;

    if ctx.multi.gbDeltaSender as usize != pnum {
        if b_cmd != CMD_DLEVEL_END && (b_cmd != CMD_DLEVEL || w_offset != 0) {
            return size;
        }
        ctx.multi.gbDeltaSender = pnum as u8;
        ctx.msg.sgb_recv_cmd = CMD_DLEVEL_END;
    }

    if ctx.msg.sgb_recv_cmd == CMD_DLEVEL_END {
        if b_cmd == CMD_DLEVEL_END {
            ctx.msg.sgb_delta_chunks = (MAX_CHUNKS - 1) as u8;
            return size;
        }
        if b_cmd != CMD_DLEVEL || w_offset != 0 {
            return size;
        }
        ctx.msg.sgdw_recv_offset = 0;
        ctx.msg.sgb_recv_cmd = b_cmd;
    } else if ctx.msg.sgb_recv_cmd != b_cmd || w_offset == 0 {
        let (recv_cmd, recv_offset) = (ctx.msg.sgb_recv_cmd, ctx.msg.sgdw_recv_offset);
        delta_import_data(ctx, recv_cmd, recv_offset);
        if b_cmd == CMD_DLEVEL_END {
            ctx.msg.sgb_delta_chunks = (MAX_CHUNKS - 1) as u8;
            ctx.msg.sgb_recv_cmd = CMD_DLEVEL_END;
            return size;
        }
        ctx.msg.sgdw_recv_offset = 0;
        ctx.msg.sgb_recv_cmd = b_cmd;
    }

    assert!(w_offset as u32 == ctx.msg.sgdw_recv_offset);
    let body = v.bytes(SIZE_TCMDPLRINFOHDR, w_bytes as usize);
    let o = w_offset as usize;
    ctx.msg.sg_recv_buf[o..o + body.len()].copy_from_slice(&body);
    ctx.msg.sgdw_recv_offset += w_bytes as u32;
    size
}

/// Original: `DeltaSyncGolem` (msg.cpp).
// @port msg.cpp|devilution::DeltaSyncGolem(const TCmdGolem &message, int pnum, uint8_t level) sha=9d143567ada2
fn delta_sync_golem(ctx: &mut Ctx, message: CmdView, pnum: usize, level: u8) {
    if !ctx.init.gb_is_multiplayer {
        return;
    }
    let monster = &mut get_delta_level(ctx, level).monster[pnum];
    monster.x = message.u8(1);
    monster.y = message.u8(2);
    monster.mactive = u8::MAX;
    monster.menemy = message.u8(4);
    monster.hit_points = message.i32(5);
}

/// Original: `DeltaLeaveSync` (msg.cpp).
// @port msg.cpp|devilution::DeltaLeaveSync(uint8_t bLevel) sha=53204c4cf517
fn delta_leave_sync(ctx: &mut Ctx, b_level: u8) {
    if !ctx.init.gb_is_multiplayer {
        return;
    }
    if ctx.gendung.leveltype == DungeonType::Town {
        ctx.diablo.glSeedTbl[0] = ctx.rng.advance_rnd_seed() as u32;
        return;
    }
    let mut level = get_delta_level(ctx, b_level).clone();
    for i in 0..ctx.monster.ActiveMonsterCount {
        let ma = ctx.monster.ActiveMonsters[i] as usize;
        let monster = &ctx.monster.Monsters[ma];
        if monster.hitPoints == 0 {
            continue;
        }
        let enemy = crate::monster::encode_enemy(ctx, ma);
        let monster = &ctx.monster.Monsters[ma];
        let delta = &mut level.monster[ma];
        delta.x = monster.position.tile.x as u8;
        delta.y = monster.position.tile.y as u8;
        delta.menemy = enemy;
        delta.hit_points = monster.hitPoints;
        delta.mactive = monster.activeForTicks;
        delta.who_hit = monster.whoHit;
    }
    ctx.msg.delta_levels.insert(b_level, level);
    let view = ctx.automap.AutomapView.clone();
    ctx.msg.local_levels.insert(b_level, view);
}

/// Original: `DeltaSyncObject` (msg.cpp).
// @port msg.cpp|devilution::DeltaSyncObject(WorldTilePosition position, _cmd_id bCmd, const Player &player) sha=61e367e2ded9
fn delta_sync_object(ctx: &mut Ctx, position: (u8, u8), b_cmd: _cmd_id, pnum: usize) {
    if !ctx.init.gb_is_multiplayer {
        return;
    }
    let level = get_level_for_multiplayer(ctx, pnum);
    get_delta_level(ctx, level).object_set(position, b_cmd);
}

/// Original: `DeltaGetItem` (msg.cpp).
// @port msg.cpp|devilution::DeltaGetItem(const TCmdGItem &message, uint8_t bLevel) sha=8c0c4a581100
fn delta_get_item(ctx: &mut Ctx, message: CmdView, b_level: u8) -> bool {
    if !ctx.init.gb_is_multiplayer {
        return true;
    }
    let mi = NetItem::from_slice(&message.0[3.min(message.0.len())..]);
    let mut fatal = false;
    {
        let level = get_delta_level(ctx, b_level);
        for item in level.item.iter_mut() {
            let ii = NetItem::from_slice(&item[3..]);
            if item[0] == CMD_INVALID || ii.w_indx() != mi.w_indx() || ii.w_ci() != mi.w_ci() || ii.dw_seed() != mi.dw_seed() {
                continue;
            }
            if item[0] == PickedUpItem {
                return true;
            }
            if item[0] == FloorItem {
                item[0] = PickedUpItem;
                return true;
            }
            if item[0] == DroppedItem {
                item[0] = CMD_INVALID;
                return true;
            }
            fatal = true;
            break;
        }
    }
    if fatal {
        crate::appfat::app_fatal(ctx, "delta:1");
    }
    if (mi.w_ci() & CF_PREGEN as u16) == 0 {
        return false;
    }
    let level = get_delta_level(ctx, b_level);
    for delta in level.item.iter_mut() {
        if delta[0] == CMD_INVALID {
            delta[0] = PickedUpItem;
            delta[1] = message.u8(1);
            delta[2] = message.u8(2);
            let mut d = NetItem::from_slice(&delta[3..]);
            d.set_def(mi.w_indx(), mi.w_ci(), mi.dw_seed());
            if mi.w_indx() == IDI_EAR {
                d.0[8] = mi.b_cursval();
                // CopyUtf8(delta.ear.heroname, message.ear.heroname, 17)
                let name = crate::utils::utf8::copy_utf8(&mi.heroname(), 17);
                for b in d.0[9..26].iter_mut() {
                    *b = 0;
                }
                d.0[9..9 + name.len()].copy_from_slice(name.as_bytes());
            } else {
                d.0[8..23].copy_from_slice(&mi.0[8..23]);
                // wMaxDam is not copied (as in the original)
            }
            delta[3..].copy_from_slice(&d.0);
            break;
        }
    }
    true
}

/// Original: `DeltaPutItem` (msg.cpp).
// @port msg.cpp|devilution::DeltaPutItem(const TCmdPItem &message, Point position, const Player &player) sha=3540f8aa9f97
fn delta_put_item(ctx: &mut Ctx, message: CmdView, position: Point, pnum: usize) {
    if !ctx.init.gb_is_multiplayer {
        return;
    }
    let mi = NetItem::from_slice(&message.0[3.min(message.0.len())..]);
    let lvl = get_level_for_multiplayer(ctx, pnum);
    let mut fatal = false;
    {
        let level = get_delta_level(ctx, lvl);
        for item in level.item.iter() {
            let ii = NetItem::from_slice(&item[3..]);
            if item[0] != PickedUpItem && item[0] != CMD_INVALID && ii.w_indx() == mi.w_indx() && ii.w_ci() == mi.w_ci() && ii.dw_seed() == mi.dw_seed() {
                if item[0] == DroppedItem {
                    return;
                }
                fatal = true;
                break;
            }
        }
    }
    if fatal {
        crate::appfat::app_fatal(ctx, &tr("Trying to drop a floor item?"));
    }
    let level = get_delta_level(ctx, lvl);
    for item in level.item.iter_mut() {
        if item[0] == CMD_INVALID {
            for (i, b) in item.iter_mut().enumerate() {
                *b = message.u8(i);
            }
            item[0] = DroppedItem;
            item[1] = position.x as u8;
            item[2] = position.y as u8;
            return;
        }
    }
}

/// Original: `IOwnLevel` (msg.cpp).
// @port msg.cpp|devilution::IOwnLevel(const Player &player) sha=fcf98f8eb756
fn i_own_level(ctx: &Ctx, pnum: usize) -> bool {
    let player = &ctx.players.Players[pnum];
    let me = ctx.players.MyPlayer;
    for (oi, other) in ctx.players.Players.iter().enumerate() {
        if !other.plractive || other._pLvlChanging || other._pmode == PM_NEWLVL {
            continue;
        }
        if other.plrlevel != player.plrlevel || other.plrIsOnSetLevel != player.plrIsOnSetLevel {
            continue;
        }
        if Some(oi) == me && ctx.msg.gbBufferMsgs != 0 {
            continue;
        }
        return Some(oi) == me;
    }
    false
}

/// Original: `DeltaOpenPortal` (msg.cpp).
// @port msg.cpp|devilution::DeltaOpenPortal(int pnum, Point position, uint8_t bLevel, dungeon_type bLType, bool bSetLvl) sha=489c846252fb
fn delta_open_portal(ctx: &mut Ctx, pnum: usize, position: Point, b_level: u8, b_l_type: DungeonType, b_set_lvl: bool) {
    ctx.msg.sg_junk.portal[pnum] = DPortal { x: position.x as u8, y: position.y as u8, level: b_level, ltype: b_l_type as i8 as u8, setlvl: b_set_lvl as u8 };
}

/// Original: `NetSendCmdGItem2` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdGItem2(bool usonly, _cmd_id bCmd, uint8_t mast, uint8_t pnum, const TCmdGItem &item) sha=2690baa829ce
fn net_send_cmd_g_item2(ctx: &mut Ctx, usonly: bool, b_cmd: _cmd_id, mast: u8, pnum: u8, item: CmdView) {
    let mut cmd = item.bytes(0, SIZE_TCMDGITEM);
    cmd[30] = pnum;
    cmd[0] = b_cmd;
    cmd[29] = mast;
    if !usonly {
        put32(&mut cmd, 33, 0);
        let my = ctx.players.MyPlayerId as i32;
        net_send_hi_pri(ctx, my, &cmd);
        return;
    }
    let ticks = ctx.platform.ticks() as i32;
    let dw_time = CmdView(&cmd).i32(33);
    if dw_time == 0 {
        put32(&mut cmd, 33, ticks as u32);
    } else if ticks.wrapping_sub(dw_time) > 5000 {
        return;
    }
    crate::tmsg::tmsg_add(ctx, &cmd);
}

/// Original: `NetSendCmdReq2` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdReq2(_cmd_id bCmd, uint8_t mast, uint8_t pnum, const TCmdGItem &item) sha=9d6eed5b4244
fn net_send_cmd_req2(ctx: &mut Ctx, b_cmd: _cmd_id, mast: u8, pnum: u8, item: CmdView) -> bool {
    let mut cmd = item.bytes(0, SIZE_TCMDGITEM);
    cmd[0] = b_cmd;
    cmd[30] = pnum;
    cmd[29] = mast;
    let ticks = ctx.platform.ticks() as i32;
    let dw_time = CmdView(&cmd).i32(33);
    if dw_time == 0 {
        put32(&mut cmd, 33, ticks as u32);
    } else if ticks.wrapping_sub(dw_time) > 5000 {
        return false;
    }
    crate::tmsg::tmsg_add(ctx, &cmd);
    true
}

/// Original: `NetSendCmdExtra` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdExtra(const TCmdGItem &item) sha=d7d3c53a9c91
fn net_send_cmd_extra(ctx: &mut Ctx, item: CmdView) {
    let mut cmd = item.bytes(0, SIZE_TCMDGITEM);
    put32(&mut cmd, 33, 0);
    cmd[0] = CMD_ITEMEXTRA;
    let my = ctx.players.MyPlayerId as i32;
    net_send_hi_pri(ctx, my, &cmd);
}

fn position_of(v: CmdView) -> Point {
    Point::new(v.u8(1) as i32, v.u8(2) as i32)
}

/// Original: `OnWalk` (msg.cpp).
// @port msg.cpp|devilution::OnWalk(const TCmd *pCmd, Player &player) sha=1445d77f2dc4
fn on_walk(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let position = position_of(v);
    if ctx.msg.gbBufferMsgs != 1 && crate::player::is_on_active_level(ctx, pnum) && in_dungeon_bounds(position) {
        crate::player::clr_plr_path(ctx, pnum);
        crate::player::make_plr_path(ctx, pnum, position, true);
        ctx.players.Players[pnum].destAction = ACTION_NONE;
    }
    SIZE_TCMDLOC
}

fn on_add_stat(ctx: &mut Ctx, v: CmdView, pnum: usize, f: fn(&mut Ctx, usize, i32)) -> usize {
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDPARAM1));
    } else if v.u16(1) <= 256 {
        f(ctx, pnum, v.u16(1) as i32);
    }
    SIZE_TCMDPARAM1
}

/// Original: `OnAddStrength` (msg.cpp).
// @port msg.cpp|devilution::OnAddStrength(const TCmd *pCmd, size_t pnum) sha=c90d8a5233bb
fn on_add_strength(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    on_add_stat(ctx, v, pnum, crate::player::modify_plr_str)
}

/// Original: `OnAddMagic` (msg.cpp).
// @port msg.cpp|devilution::OnAddMagic(const TCmd *pCmd, size_t pnum) sha=1482c1f1388d
fn on_add_magic(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    on_add_stat(ctx, v, pnum, crate::player::modify_plr_mag)
}

/// Original: `OnAddDexterity` (msg.cpp).
// @port msg.cpp|devilution::OnAddDexterity(const TCmd *pCmd, int pnum) sha=b27af9c7a9c4
fn on_add_dexterity(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    on_add_stat(ctx, v, pnum, crate::player::modify_plr_dex)
}

/// Original: `OnAddVitality` (msg.cpp).
// @port msg.cpp|devilution::OnAddVitality(const TCmd *pCmd, size_t pnum) sha=6ba4550f3e6e
fn on_add_vitality(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    on_add_stat(ctx, v, pnum, crate::player::modify_plr_vit)
}

/// Original: `OnGotoGetItem` (msg.cpp).
// @port msg.cpp|devilution::OnGotoGetItem(const TCmd *pCmd, Player &player) sha=e64cce132ffc
fn on_goto_get_item(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let position = position_of(v);
    let w1 = v.u16(3);
    if ctx.msg.gbBufferMsgs != 1 && crate::player::is_on_active_level(ctx, pnum) && in_dungeon_bounds(position) && (w1 as usize) < MAXITEMS + 1 {
        crate::player::make_plr_path(ctx, pnum, position, false);
        let p = &mut ctx.players.Players[pnum];
        p.destAction = ACTION_PICKUPITEM;
        p.destParam1 = w1 as i32;
    }
    SIZE_TCMDLOCPARAM1
}

/// Original: `IsGItemValid` (msg.cpp).
// @port msg.cpp|devilution::IsGItemValid(const TCmdGItem &message) sha=bfd284226422
fn is_g_item_valid(ctx: &Ctx, v: CmdView) -> bool {
    let players = ctx.players.Players.len();
    if v.u8(29) as usize >= players {
        return false;
    }
    if v.u8(30) as usize >= players {
        return false;
    }
    if v.u8(31) as usize >= MAXITEMS + 1 {
        return false;
    }
    if !is_valid_level_for_multiplayer(v.u8(32)) {
        return false;
    }
    if !in_dungeon_bounds(position_of(v)) {
        return false;
    }
    crate::items::is_item_available(ctx, v.i16(3) as i32)
}

/// Original: `IsPItemValid` (msg.cpp).
// @port msg.cpp|devilution::IsPItemValid(const TCmdPItem &message, const Player &player) sha=1b314e47740f
fn is_p_item_valid(ctx: &mut Ctx, v: CmdView, pnum: usize) -> bool {
    let position = position_of(v);
    if !in_dungeon_bounds(position) {
        return false;
    }
    let mi = NetItem::from_slice(&v.0[3.min(v.0.len())..]);
    let idx = mi.w_indx();
    if idx != IDI_EAR {
        let creation_flags = mi.w_ci();
        // `SDL_SwapLE16(message.item.dwBuff)`: the original truncates dwBuff to 16 bits here
        let dw_buff = mi.dw_buff() as u16 as u32;
        if idx != IDI_GOLD && !validate(ctx, pnum, crate::pack::is_creation_flag_combo_valid(creation_flags), "IsCreationFlagComboValid") {
            return false;
        }
        if (creation_flags & CF_TOWN as u16) != 0 {
            if !validate(ctx, pnum, crate::pack::is_town_item_valid(creation_flags), "IsTownItemValid") {
                return false;
            }
        } else if (creation_flags & CF_USEFUL as u16) == CF_UPER15 as u16 {
            if !validate(ctx, pnum, crate::pack::is_unique_monster_item_valid(creation_flags, dw_buff), "IsUniqueMonsterItemValid") {
                return false;
            }
        } else if (dw_buff & CF_HELLFIRE as u32) != 0 && crate::tables::itemdat::AllItemsList[idx as usize].iMiscId == IMISC_BOOK {
            let player = ctx.players.Players[pnum].clone();
            return crate::pack::recreate_hellfire_spell_book(ctx, &player, &mi, None, pnum);
        } else if !validate(ctx, pnum, crate::pack::is_dungeon_item_valid(creation_flags, dw_buff), "IsDungeonItemValid") {
            return false;
        }
    }
    crate::items::is_item_available(ctx, idx as i32)
}

/// Fills the item union of a command from an item (`PrepareItemForNetwork` overloads for
/// TCmdGItem, TCmdPItem and TCmdChItem).
fn prepare_item_union(item: &Item) -> NetItem {
    let mut n = NetItem::default();
    n.set_def(item.IDidx, item._iCreateInfo, item._iSeed);
    if item.IDidx == IDI_EAR {
        prepare_ear_for_network(item, &mut n);
    } else {
        prepare_item_for_network(item, &mut n);
    }
    n
}

/// Original: `RecreateItem(const Player &player, const TCmdPItem &message, Item &item)` (msg.cpp),
/// and the TCmdChItem overload.
// @port msg.cpp|devilution::RecreateItem(const Player &player, const TCmdPItem &message, Item &item) sha=d682f54b9d30
// @port msg.cpp|devilution::RecreateItem(const Player &player, const TCmdChItem &message, Item &item) sha=90881d9501ea
fn recreate_item_from_union(ctx: &mut Ctx, player: &crate::player::Player, mi: &NetItem, item: &mut Item) {
    if mi.w_indx() == IDI_EAR {
        crate::items::recreate_ear(ctx, item, mi.w_ci(), mi.dw_seed(), mi.b_cursval(), &mi.heroname());
    } else {
        recreate_item(ctx, player, mi, item);
    }
}

/// Original: `SyncDropItem(Point position, const TItem &item)` (msg.cpp).
// @port msg.cpp|devilution::SyncDropItem(Point position, const TItem &item) sha=855630402574
fn sync_drop_t_item(ctx: &mut Ctx, position: Point, mi: &NetItem) -> i32 {
    crate::inv::sync_drop_item(
        ctx,
        position,
        mi.w_indx(),
        mi.w_ci(),
        mi.dw_seed() as i32,
        mi.b_id() as i32,
        mi.b_dur() as i32,
        mi.b_m_dur() as i32,
        mi.b_ch() as i32,
        mi.b_m_ch() as i32,
        mi.w_value() as i32,
        mi.dw_buff(),
        mi.w_to_hit() as i32,
        mi.w_max_dam() as i32,
    )
}

/// Original: `SyncDropEar(Point position, const TEar &ear)` (msg.cpp).
// @port msg.cpp|devilution::SyncDropEar(Point position, const TEar &ear) sha=8b4b37a4679f
fn sync_drop_t_ear(ctx: &mut Ctx, position: Point, mi: &NetItem) -> i32 {
    crate::inv::sync_drop_ear(ctx, position, mi.w_ci(), mi.dw_seed(), mi.b_cursval(), &mi.heroname())
}

/// Original: `SyncDropItem(const TCmdGItem &message)` / `SyncDropItem(const TCmdPItem &message)` (msg.cpp).
// @port msg.cpp|devilution::SyncDropItem(const TCmdGItem &message) sha=0b61771be22a
// @port msg.cpp|devilution::SyncDropItem(const TCmdPItem &message) sha=2ed44d3a6a6f
fn sync_drop_cmd_item(ctx: &mut Ctx, v: CmdView) -> i32 {
    let position = get_item_position(ctx, position_of(v));
    let mi = NetItem::from_slice(&v.0[3.min(v.0.len())..]);
    if mi.w_indx() == IDI_EAR {
        return sync_drop_t_ear(ctx, position, &mi);
    }
    sync_drop_t_item(ctx, position, &mi)
}

fn g_item_keys(v: CmdView) -> (u32, u16, _item_indexes) {
    (v.u32(7), v.u16(5), v.i16(3))
}

/// Original: `OnRequestGetItem` (msg.cpp).
// @port msg.cpp|devilution::OnRequestGetItem(const TCmd *pCmd, Player &player) sha=9e838c5e88f6
fn on_request_get_item(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs != 1 && i_own_level(ctx, pnum) && is_g_item_valid(ctx, v) {
        let position = position_of(v);
        let (dw_seed, w_ci, w_indx) = g_item_keys(v);
        if crate::items::get_item_record(ctx, dw_seed, w_ci, w_indx as i32) {
            let mut ii: i32 = -1;
            if in_dungeon_bounds(position) {
                ii = (ctx.items.dItem[position.x as usize][position.y as usize] as i32).abs() - 1;
                if ii >= 0 && !ctx.items.Items[ii as usize].key_attributes_match(dw_seed, w_indx, w_ci) {
                    ii = -1;
                }
            }
            if ii == -1 {
                let active_item_index = crate::inv::find_get_item(ctx, dw_seed, w_indx, w_ci);
                if active_item_index != -1 {
                    ii = ctx.items.ActiveItems[active_item_index as usize] as i32;
                }
            }
            let my = ctx.players.MyPlayerId as u8;
            let b_pnum = v.u8(30);
            if ii != -1 {
                net_send_cmd_g_item2(ctx, false, CMD_GETITEM, my, b_pnum, v);
                if b_pnum != my {
                    crate::inv::sync_get_item(ctx, position, dw_seed, w_indx, w_ci);
                } else {
                    let me = ctx.players.MyPlayer.unwrap();
                    crate::inv::inv_get_item(ctx, me, ii);
                }
                crate::items::set_item_record(ctx, dw_seed, w_ci, w_indx as i32);
            } else if !net_send_cmd_req2(ctx, CMD_REQUESTGITEM, my, b_pnum, v) {
                net_send_cmd_extra(ctx, v);
            }
        }
    }
    SIZE_TCMDGITEM
}

/// Original: `OnGetItem` (msg.cpp).
// @port msg.cpp|devilution::OnGetItem(const TCmd *pCmd, size_t pnum) sha=fd822251a2c3
fn on_get_item(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDGITEM));
    } else if is_g_item_valid(ctx, v) {
        let position = position_of(v);
        let (dw_seed, w_ci, w_indx) = g_item_keys(v);
        let b_level = v.u8(32);
        if delta_get_item(ctx, v, b_level) {
            let me = ctx.players.MyPlayer.unwrap();
            let my = ctx.players.MyPlayerId as u8;
            let is_on_active_level = get_level_for_multiplayer(ctx, me) == b_level;
            if (is_on_active_level || v.u8(30) == my) && v.u8(29) != my {
                if v.u8(30) == my {
                    if !is_on_active_level {
                        let ii = sync_drop_cmd_item(ctx, v);
                        if ii != -1 {
                            crate::inv::inv_get_item(ctx, me, ii);
                        }
                    } else {
                        let active_item_index = crate::inv::find_get_item(ctx, dw_seed, w_indx, w_ci);
                        let ii = ctx.items.ActiveItems[active_item_index as usize] as i32;
                        crate::inv::inv_get_item(ctx, me, ii);
                    }
                } else {
                    crate::inv::sync_get_item(ctx, position, dw_seed, w_indx, w_ci);
                }
            }
        } else {
            net_send_cmd_g_item2(ctx, true, CMD_GETITEM, v.u8(29), v.u8(30), v);
        }
    }
    SIZE_TCMDGITEM
}

/// Original: `OnGotoAutoGetItem` (msg.cpp).
// @port msg.cpp|devilution::OnGotoAutoGetItem(const TCmd *pCmd, Player &player) sha=2a0d96fc81eb
fn on_goto_auto_get_item(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let position = position_of(v);
    let item_idx = v.u16(3);
    if ctx.msg.gbBufferMsgs != 1 && crate::player::is_on_active_level(ctx, pnum) && in_dungeon_bounds(position) && (item_idx as usize) < MAXITEMS + 1 {
        crate::player::make_plr_path(ctx, pnum, position, false);
        let p = &mut ctx.players.Players[pnum];
        p.destAction = ACTION_PICKUPAITEM;
        p.destParam1 = item_idx as i32;
    }
    SIZE_TCMDLOCPARAM1
}

/// Original: `OnRequestAutoGetItem` (msg.cpp).
// @port msg.cpp|devilution::OnRequestAutoGetItem(const TCmd *pCmd, Player &player) sha=a487a2323340
fn on_request_auto_get_item(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs != 1 && i_own_level(ctx, pnum) && is_g_item_valid(ctx, v) {
        let position = position_of(v);
        let (dw_seed, w_ci, w_indx) = g_item_keys(v);
        if crate::items::get_item_record(ctx, dw_seed, w_ci, w_indx as i32) {
            let my = ctx.players.MyPlayerId as u8;
            let b_pnum = v.u8(30);
            if crate::inv::find_get_item(ctx, dw_seed, w_indx, w_ci) != -1 {
                net_send_cmd_g_item2(ctx, false, CMD_AGETITEM, my, b_pnum, v);
                if b_pnum != my {
                    crate::inv::sync_get_item(ctx, position, dw_seed, w_indx, w_ci);
                } else {
                    let me = ctx.players.MyPlayer.unwrap();
                    let cursitem = v.u8(31) as i32;
                    crate::inv::auto_get_item(ctx, me, cursitem);
                }
                crate::items::set_item_record(ctx, dw_seed, w_ci, w_indx as i32);
            } else if !net_send_cmd_req2(ctx, CMD_REQUESTAGITEM, my, b_pnum, v) {
                net_send_cmd_extra(ctx, v);
            }
        }
    }
    SIZE_TCMDGITEM
}

/// Original: `OnAutoGetItem` (msg.cpp).
// @port msg.cpp|devilution::OnAutoGetItem(const TCmd *pCmd, size_t pnum) sha=785911e64872
fn on_auto_get_item(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDGITEM));
    } else if is_g_item_valid(ctx, v) {
        let position = position_of(v);
        let b_level = v.u8(32);
        if delta_get_item(ctx, v, b_level) {
            let me = ctx.players.MyPlayer.unwrap();
            let my = ctx.players.MyPlayerId as u8;
            let local_level = get_level_for_multiplayer(ctx, me);
            if (local_level == b_level || v.u8(30) == my) && v.u8(29) != my {
                if v.u8(30) == my {
                    if local_level != b_level {
                        let ii = sync_drop_cmd_item(ctx, v);
                        if ii != -1 {
                            crate::inv::auto_get_item(ctx, me, ii);
                        }
                    } else {
                        crate::inv::auto_get_item(ctx, me, v.u8(31) as i32);
                    }
                } else {
                    let (dw_seed, w_ci, w_indx) = g_item_keys(v);
                    crate::inv::sync_get_item(ctx, position, dw_seed, w_indx, w_ci);
                }
            }
        } else {
            net_send_cmd_g_item2(ctx, true, CMD_AGETITEM, v.u8(29), v.u8(30), v);
        }
    }
    SIZE_TCMDGITEM
}

/// Original: `OnItemExtra` (msg.cpp).
// @port msg.cpp|devilution::OnItemExtra(const TCmd *pCmd, size_t pnum) sha=67b96201d8c9
fn on_item_extra(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDGITEM));
    } else if is_g_item_valid(ctx, v) {
        delta_get_item(ctx, v, v.u8(32));
        if crate::player::is_on_active_level(ctx, pnum) {
            let (dw_seed, w_ci, w_indx) = g_item_keys(v);
            crate::inv::sync_get_item(ctx, position_of(v), dw_seed, w_indx, w_ci);
        }
    }
    SIZE_TCMDGITEM
}

/// Original: `OnPutItem` (msg.cpp).
// @port msg.cpp|devilution::OnPutItem(const TCmd *pCmd, size_t pnum) sha=644265f18920
fn on_put_item(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDPITEM));
    } else if is_p_item_valid(ctx, v, pnum) {
        let position = position_of(v);
        let is_self = ctx.players.MyPlayer == Some(pnum);
        let dw_seed = v.u32(7);
        let w_ci = v.u16(5);
        let w_indx = v.i16(3);
        if crate::player::is_on_active_level(ctx, pnum) {
            let ii: i32;
            if is_self {
                let tile = ctx.players.Players[pnum].position.tile;
                let item_tile = crate::inv::find_adjacent_position_for_item(ctx, tile, crate::engine::get_direction(tile, position));
                if let Some(t) = item_tile {
                    let limbo = std::mem::take(&mut ctx.msg.item_limbo);
                    ii = crate::items::place_item_in_world(ctx, limbo, t) as i32;
                } else {
                    ii = -1;
                }
            } else {
                ii = sync_drop_cmd_item(ctx, v);
            }
            if ii != -1 {
                crate::items::put_item_record(ctx, dw_seed, w_ci, w_indx as i32);
                let pos = ctx.items.Items[ii as usize].position;
                delta_put_item(ctx, v, pos, pnum);
                if is_self {
                    crate::pfile::pfile_update(ctx, true);
                }
            }
            return SIZE_TCMDPITEM;
        } else {
            crate::items::put_item_record(ctx, dw_seed, w_ci, w_indx as i32);
            delta_put_item(ctx, v, position, pnum);
            if is_self {
                crate::pfile::pfile_update(ctx, true);
            }
        }
    }
    SIZE_TCMDPITEM
}

/// Original: `OnSyncPutItem` (msg.cpp).
// @port msg.cpp|devilution::OnSyncPutItem(const TCmd *pCmd, size_t pnum) sha=fdfa55819c3a
fn on_sync_put_item(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDPITEM));
    } else if is_p_item_valid(ctx, v, pnum) {
        let is_self = ctx.players.MyPlayer == Some(pnum);
        let dw_seed = v.u32(7);
        let w_ci = v.u16(5);
        let w_indx = v.i16(3);
        if crate::player::is_on_active_level(ctx, pnum) {
            let ii = sync_drop_cmd_item(ctx, v);
            if ii != -1 {
                crate::items::put_item_record(ctx, dw_seed, w_ci, w_indx as i32);
                let pos = ctx.items.Items[ii as usize].position;
                delta_put_item(ctx, v, pos, pnum);
                if is_self {
                    crate::pfile::pfile_update(ctx, true);
                }
            }
            return SIZE_TCMDPITEM;
        } else {
            crate::items::put_item_record(ctx, dw_seed, w_ci, w_indx as i32);
            delta_put_item(ctx, v, position_of(v), pnum);
            if is_self {
                crate::pfile::pfile_update(ctx, true);
            }
        }
    }
    SIZE_TCMDPITEM
}

/// Original: `OnAttackTile` (msg.cpp).
// @port msg.cpp|devilution::OnAttackTile(const TCmd *pCmd, Player &player) sha=803ee8d91d3b
fn on_attack_tile(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let position = position_of(v);
    if ctx.msg.gbBufferMsgs != 1 && crate::player::is_on_active_level(ctx, pnum) && in_dungeon_bounds(position) {
        crate::player::make_plr_path(ctx, pnum, position, false);
        let p = &mut ctx.players.Players[pnum];
        p.destAction = ACTION_ATTACK;
        p.destParam1 = position.x;
        p.destParam2 = position.y;
    }
    SIZE_TCMDLOC
}

/// Original: `OnStandingAttackTile` (msg.cpp).
// @port msg.cpp|devilution::OnStandingAttackTile(const TCmd *pCmd, Player &player) sha=b66b0b07fd5f
fn on_standing_attack_tile(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let position = position_of(v);
    if ctx.msg.gbBufferMsgs != 1 && crate::player::is_on_active_level(ctx, pnum) && in_dungeon_bounds(position) {
        crate::player::clr_plr_path(ctx, pnum);
        let p = &mut ctx.players.Players[pnum];
        p.destAction = ACTION_ATTACK;
        p.destParam1 = position.x;
        p.destParam2 = position.y;
    }
    SIZE_TCMDLOC
}

/// Original: `OnRangedAttackTile` (msg.cpp).
// @port msg.cpp|devilution::OnRangedAttackTile(const TCmd *pCmd, Player &player) sha=506efc67b9d3
fn on_ranged_attack_tile(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let position = position_of(v);
    if ctx.msg.gbBufferMsgs != 1 && crate::player::is_on_active_level(ctx, pnum) && in_dungeon_bounds(position) {
        crate::player::clr_plr_path(ctx, pnum);
        let p = &mut ctx.players.Players[pnum];
        p.destAction = ACTION_RATTACK;
        p.destParam1 = position.x;
        p.destParam2 = position.y;
    }
    SIZE_TCMDLOC
}

/// Original: `InitNewSpell` (msg.cpp).
// @port msg.cpp|devilution::InitNewSpell(Player &player, uint16_t wParamSpellID, uint16_t wParamSpellType, uint16_t wParamSpellFrom) sha=70fac61c1c60
fn init_new_spell(ctx: &mut Ctx, pnum: usize, w_param_spell_id: u16, w_param_spell_type: u16, w_param_spell_from: u16) -> bool {
    if w_param_spell_id as i32 > SpellID::LAST as i8 as i32 {
        return false;
    }
    let spell_id = SpellID::from_raw(w_param_spell_id as i8);
    if !crate::spells::is_valid_spell(ctx, spell_id) {
        log::error!("{}", tr("{:s} has cast an invalid spell.").replacen("{:s}", ctx.players.Players[pnum]._pName.as_str(), 1));
        return false;
    }
    if ctx.gendung.leveltype == DungeonType::Town && !crate::items::get_spell_data(spell_id).flags.has_any_of(SpellDataFlags::AllowedInTown) {
        log::error!("{}", tr("{:s} has cast an illegal spell.").replacen("{:s}", ctx.players.Players[pnum]._pName.as_str(), 1));
        return false;
    }
    if w_param_spell_type as u32 > SpellType::Invalid as u8 as u32 {
        return false;
    }
    if w_param_spell_from as i32 > INVITEM_BELT_LAST as i32 {
        return false;
    }
    let spell_from = w_param_spell_from as i8;
    if !crate::spells::is_valid_spell_from(spell_from as i32) {
        return false;
    }
    let p = &mut ctx.players.Players[pnum];
    p.queuedSpell.spellId = spell_id;
    p.queuedSpell.spellType = SpellType::from_raw(w_param_spell_type as u8);
    p.queuedSpell.spellFrom = spell_from;
    true
}

/// Original: `OnSpellWall` (msg.cpp).
// @port msg.cpp|devilution::OnSpellWall(const TCmd *pCmd, Player &player) sha=1df6fc9c6476
fn on_spell_wall(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let position = position_of(v);
    if ctx.msg.gbBufferMsgs == 1 || !crate::player::is_on_active_level(ctx, pnum) || !in_dungeon_bounds(position) {
        return SIZE_TCMDLOCPARAM5;
    }
    let w_param_direction = v.i16(7);
    if w_param_direction as u16 > Direction::SouthEast as u16 {
        return SIZE_TCMDLOCPARAM5;
    }
    if !init_new_spell(ctx, pnum, v.u16(3), v.u16(5), v.u16(11)) {
        return SIZE_TCMDLOCPARAM5;
    }
    crate::player::clr_plr_path(ctx, pnum);
    let p = &mut ctx.players.Players[pnum];
    p.destAction = ACTION_SPELLWALL;
    p.destParam1 = position.x;
    p.destParam2 = position.y;
    p.destParam3 = w_param_direction as i32;
    p.destParam4 = v.u16(9) as i32;
    SIZE_TCMDLOCPARAM5
}

/// Original: `OnSpellTile` (msg.cpp).
// @port msg.cpp|devilution::OnSpellTile(const TCmd *pCmd, Player &player) sha=036591d4544a
fn on_spell_tile(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let position = position_of(v);
    if ctx.msg.gbBufferMsgs == 1 || !crate::player::is_on_active_level(ctx, pnum) || !in_dungeon_bounds(position) {
        return SIZE_TCMDLOCPARAM4;
    }
    if !init_new_spell(ctx, pnum, v.u16(3), v.u16(5), v.u16(9)) {
        return SIZE_TCMDLOCPARAM4;
    }
    crate::player::clr_plr_path(ctx, pnum);
    let p = &mut ctx.players.Players[pnum];
    p.destAction = ACTION_SPELL;
    p.destParam1 = position.x;
    p.destParam2 = position.y;
    p.destParam3 = v.u16(7) as i32;
    SIZE_TCMDLOCPARAM4
}

/// Original: `OnObjectTileAction` (msg.cpp).
// @port msg.cpp|devilution::OnObjectTileAction(const TCmd &cmd, Player &player, action_id action, bool pathToObject = true) sha=36bbdabb037c
fn on_object_tile_action(ctx: &mut Ctx, v: CmdView, pnum: usize, action: action_id, path_to_object: bool) -> usize {
    let position = position_of(v);
    let object = crate::objects::find_object_at_position(ctx, position, true);
    if ctx.msg.gbBufferMsgs != 1 && crate::player::is_on_active_level(ctx, pnum) {
        if let Some(oi) = object {
            if path_to_object {
                let o = &ctx.objects.Objects[oi];
                let endspace = !o._oSolidFlag && !o._oDoorFlag;
                crate::player::make_plr_path(ctx, pnum, position, endspace);
            }
            let p = &mut ctx.players.Players[pnum];
            p.destAction = action;
            p.destParam1 = oi as i32;
        }
    }
    SIZE_TCMDLOC
}

/// Original: `OnAttackMonster` (msg.cpp).
// @port msg.cpp|devilution::OnAttackMonster(const TCmd *pCmd, Player &player) sha=f247194c0077
fn on_attack_monster(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let monster_idx = v.u16(1) as usize;
    if ctx.msg.gbBufferMsgs != 1 && crate::player::is_on_active_level(ctx, pnum) && monster_idx < MaxMonsters {
        let position = ctx.monster.Monsters[monster_idx].position.future;
        if ctx.players.Players[pnum].position.tile.walking_distance(position) > 1 {
            crate::player::make_plr_path(ctx, pnum, position, false);
        }
        let p = &mut ctx.players.Players[pnum];
        p.destAction = ACTION_ATTACKMON;
        p.destParam1 = monster_idx as i32;
    }
    SIZE_TCMDPARAM1
}

/// Original: `OnAttackPlayer` (msg.cpp).
// @port msg.cpp|devilution::OnAttackPlayer(const TCmd *pCmd, Player &player) sha=b6e9efe5c4c4
fn on_attack_player(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let player_idx = v.u16(1) as usize;
    if ctx.msg.gbBufferMsgs != 1 && crate::player::is_on_active_level(ctx, pnum) && player_idx < ctx.players.Players.len() {
        let fut = ctx.players.Players[player_idx].position.future;
        crate::player::make_plr_path(ctx, pnum, fut, false);
        let p = &mut ctx.players.Players[pnum];
        p.destAction = ACTION_ATTACKPLR;
        p.destParam1 = player_idx as i32;
    }
    SIZE_TCMDPARAM1
}

/// Original: `OnRangedAttackMonster` (msg.cpp).
// @port msg.cpp|devilution::OnRangedAttackMonster(const TCmd *pCmd, Player &player) sha=34972d8b3819
fn on_ranged_attack_monster(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let monster_idx = v.u16(1) as usize;
    if ctx.msg.gbBufferMsgs != 1 && crate::player::is_on_active_level(ctx, pnum) && monster_idx < MaxMonsters {
        crate::player::clr_plr_path(ctx, pnum);
        let p = &mut ctx.players.Players[pnum];
        p.destAction = ACTION_RATTACKMON;
        p.destParam1 = monster_idx as i32;
    }
    SIZE_TCMDPARAM1
}

/// Original: `OnRangedAttackPlayer` (msg.cpp).
// @port msg.cpp|devilution::OnRangedAttackPlayer(const TCmd *pCmd, Player &player) sha=af09a8c57d72
fn on_ranged_attack_player(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let player_idx = v.u16(1) as usize;
    if ctx.msg.gbBufferMsgs != 1 && crate::player::is_on_active_level(ctx, pnum) && player_idx < ctx.players.Players.len() {
        crate::player::clr_plr_path(ctx, pnum);
        let p = &mut ctx.players.Players[pnum];
        p.destAction = ACTION_RATTACKPLR;
        p.destParam1 = player_idx as i32;
    }
    SIZE_TCMDPARAM1
}

/// Original: `OnSpellMonster` (msg.cpp).
// @port msg.cpp|devilution::OnSpellMonster(const TCmd *pCmd, Player &player) sha=2b659a85dc74
fn on_spell_monster(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs == 1 || !crate::player::is_on_active_level(ctx, pnum) {
        return SIZE_TCMDPARAM5;
    }
    let monster_idx = v.u16(1) as usize;
    if monster_idx >= MaxMonsters {
        return SIZE_TCMDPARAM5;
    }
    if !init_new_spell(ctx, pnum, v.u16(3), v.u16(5), v.u16(9)) {
        return SIZE_TCMDPARAM5;
    }
    crate::player::clr_plr_path(ctx, pnum);
    let p = &mut ctx.players.Players[pnum];
    p.destAction = ACTION_SPELLMON;
    p.destParam1 = monster_idx as i32;
    p.destParam2 = v.u16(7) as i32;
    SIZE_TCMDPARAM5
}

/// Original: `OnSpellPlayer` (msg.cpp).
// @port msg.cpp|devilution::OnSpellPlayer(const TCmd *pCmd, Player &player) sha=6e4f64277ce0
fn on_spell_player(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs == 1 || !crate::player::is_on_active_level(ctx, pnum) {
        return SIZE_TCMDPARAM5;
    }
    let player_idx = v.u16(1) as usize;
    if player_idx >= ctx.players.Players.len() {
        return SIZE_TCMDPARAM5;
    }
    if !init_new_spell(ctx, pnum, v.u16(3), v.u16(5), v.u16(9)) {
        return SIZE_TCMDPARAM5;
    }
    crate::player::clr_plr_path(ctx, pnum);
    let p = &mut ctx.players.Players[pnum];
    p.destAction = ACTION_SPELLPLR;
    p.destParam1 = player_idx as i32;
    p.destParam2 = v.u16(7) as i32;
    SIZE_TCMDPARAM5
}

/// Original: `OnKnockback` (msg.cpp).
// @port msg.cpp|devilution::OnKnockback(const TCmd *pCmd, size_t pnum) sha=0f78f8d936d9
fn on_knockback(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let monster_idx = v.u16(1) as usize;
    if ctx.msg.gbBufferMsgs != 1 && crate::player::is_on_active_level(ctx, pnum) && monster_idx < MaxMonsters {
        crate::monster::m_get_knockback(ctx, monster_idx);
        crate::monster::m_start_hit(ctx, monster_idx, pnum, 0);
    }
    SIZE_TCMDPARAM1
}

/// Original: `OnResurrect` (msg.cpp).
// @port msg.cpp|devilution::OnResurrect(const TCmd *pCmd, size_t pnum) sha=ec729a4ec73f
fn on_resurrect(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let player_idx = v.u16(1) as usize;
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDPARAM1));
    } else if player_idx < ctx.players.Players.len() {
        crate::spells::do_resurrect(ctx, pnum, player_idx);
        if pnum == ctx.players.MyPlayerId {
            crate::pfile::pfile_update(ctx, true);
        }
    }
    SIZE_TCMDPARAM1
}

/// Original: `OnHealOther` (msg.cpp).
// @port msg.cpp|devilution::OnHealOther(const TCmd *pCmd, const Player &caster) sha=2870efb2ddb1
fn on_heal_other(ctx: &mut Ctx, v: CmdView, caster: usize) -> usize {
    let player_idx = v.u16(1) as usize;
    if ctx.msg.gbBufferMsgs != 1 && crate::player::is_on_active_level(ctx, caster) && player_idx < ctx.players.Players.len() {
        crate::spells::do_heal_other(ctx, caster, player_idx);
    }
    SIZE_TCMDPARAM1
}

/// Original: `OnTalkXY` (msg.cpp).
// @port msg.cpp|devilution::OnTalkXY(const TCmd *pCmd, Player &player) sha=6ea5a01b4c79
fn on_talk_xy(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let position = position_of(v);
    let towner_idx = v.u16(3) as usize;
    if ctx.msg.gbBufferMsgs != 1 && crate::player::is_on_active_level(ctx, pnum) && in_dungeon_bounds(position) && towner_idx < crate::towners::NUM_TOWNERS {
        crate::player::make_plr_path(ctx, pnum, position, false);
        let p = &mut ctx.players.Players[pnum];
        p.destAction = ACTION_TALK;
        p.destParam1 = towner_idx as i32;
    }
    SIZE_TCMDLOCPARAM1
}

/// Original: `OnNewLevel` (msg.cpp).
// @port msg.cpp|devilution::OnNewLevel(const TCmd *pCmd, size_t pnum) sha=f3e3ea15b0ba
fn on_new_level(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let event_idx = v.u16(1);
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDPARAM2));
    } else if pnum != ctx.players.MyPlayerId {
        if event_idx < WM_FIRST || event_idx > WM_LAST {
            return SIZE_TCMDPARAM2;
        }
        let mode = event_idx as interface_mode;
        let level_id = v.u16(3) as u8;
        if !is_valid_level(level_id, mode == WM_DIABSETLVL) {
            return SIZE_TCMDPARAM2;
        }
        crate::player::start_new_lvl(ctx, pnum, mode, level_id as i32);
    }
    SIZE_TCMDPARAM2
}

/// Original: `OnWarp` (msg.cpp).
// @port msg.cpp|devilution::OnWarp(const TCmd *pCmd, size_t pnum) sha=91dc42881c6b
fn on_warp(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let portal_idx = v.u16(1) as usize;
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDPARAM1));
    } else if portal_idx < MAXPORTAL {
        crate::player::start_warp_lvl(ctx, pnum, portal_idx);
    }
    SIZE_TCMDPARAM1
}

/// Original: `OnMonstDeath` (msg.cpp).
// @port msg.cpp|devilution::OnMonstDeath(const TCmd *pCmd, size_t pnum) sha=7817378077b5
fn on_monst_death(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let position = position_of(v);
    let monster_idx = v.u16(3) as usize;
    if ctx.msg.gbBufferMsgs != 1 {
        if ctx.players.MyPlayer != Some(pnum) && in_dungeon_bounds(position) && monster_idx < MaxMonsters {
            if crate::player::is_on_active_level(ctx, pnum) {
                crate::monster::m_sync_start_kill(ctx, monster_idx, position, pnum);
            }
            delta_kill_monster(ctx, monster_idx, position, pnum);
        }
    } else {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDLOCPARAM1));
    }
    SIZE_TCMDLOCPARAM1
}

/// Original: `OnKillGolem` (msg.cpp).
// @port msg.cpp|devilution::OnKillGolem(const TCmd *pCmd, size_t pnum) sha=85b0e2902d40
fn on_kill_golem(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let position = position_of(v);
    if ctx.msg.gbBufferMsgs != 1 {
        if ctx.players.MyPlayer != Some(pnum) && in_dungeon_bounds(position) {
            if crate::player::is_on_active_level(ctx, pnum) {
                crate::monster::m_sync_start_kill(ctx, pnum, position, pnum);
            }
            delta_kill_monster(ctx, pnum, position, pnum);
        }
    } else {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDLOC));
    }
    SIZE_TCMDLOC
}

/// Original: `OnAwakeGolem` (msg.cpp).
// @port msg.cpp|devilution::OnAwakeGolem(const TCmd *pCmd, size_t pnum) sha=e9982d11cacb
fn on_awake_golem(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let position = position_of(v);
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDGOLEM));
    } else if in_dungeon_bounds(position) {
        if !crate::player::is_on_active_level(ctx, pnum) {
            delta_sync_golem(ctx, v, pnum, v.u8(9));
        } else if ctx.players.MyPlayer != Some(pnum) {
            for missile in ctx.missiles.Missiles.iter() {
                if missile._mitype == MissileID::Golem && missile._misource as usize == pnum {
                    return SIZE_TCMDGOLEM;
                }
            }
            let tile = ctx.players.Players[pnum].position.tile;
            let dir = Direction::from_u8(v.u8(3));
            crate::missiles::add_missile(ctx, tile, position, dir, MissileID::Golem, crate::missiles::TARGET_MONSTERS, pnum as i32, 0, 1, None);
        }
    }
    SIZE_TCMDGOLEM
}

/// Original: `OnMonstDamage` (msg.cpp).
// @port msg.cpp|devilution::OnMonstDamage(const TCmd *pCmd, size_t pnum) sha=b06a4afd36eb
fn on_monst_damage(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let monster_idx = v.u16(1) as usize;
    if ctx.msg.gbBufferMsgs != 1 {
        if ctx.players.MyPlayer != Some(pnum) && crate::player::is_on_active_level(ctx, pnum) && monster_idx < MaxMonsters {
            crate::monster::tag(ctx, monster_idx, pnum);
            let monster = &mut ctx.monster.Monsters[monster_idx];
            if monster.hitPoints > 0 {
                monster.hitPoints = monster.hitPoints.wrapping_sub(v.u32(3) as i32);
                if (monster.hitPoints >> 6) < 1 {
                    monster.hitPoints = 1 << 6;
                }
                delta_monster_hp(ctx, monster_idx, pnum);
            }
        }
    } else {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDMONDAMAGE));
    }
    SIZE_TCMDMONDAMAGE
}

/// Original: `OnPlayerDeath` (msg.cpp).
// @port msg.cpp|devilution::OnPlayerDeath(const TCmd *pCmd, size_t pnum) sha=0af6b22efff3
fn on_player_death(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let death_reason = DeathReason::from_raw(v.u16(1) as i32);
    if ctx.msg.gbBufferMsgs != 1 {
        if ctx.players.MyPlayer != Some(pnum) {
            crate::player::start_player_kill(ctx, pnum, death_reason);
        } else {
            crate::pfile::pfile_update(ctx, true);
        }
    } else {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDPARAM1));
    }
    SIZE_TCMDPARAM1
}

/// Original: `OnPlayerDamage` (msg.cpp).
// @port msg.cpp|devilution::OnPlayerDamage(const TCmd *pCmd, Player &player) sha=207b91928534
fn on_player_damage(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let damage = v.u32(2);
    let target = v.u8(1) as usize;
    if ctx.players.MyPlayer == Some(target) && ctx.gendung.leveltype != DungeonType::Town && ctx.msg.gbBufferMsgs != 1 {
        if crate::player::is_on_active_level(ctx, pnum) && damage <= 192000 && ctx.players.Players[target]._pHitPoints >> 6 > 0 {
            let damage_type = DamageType::from_raw(v.u8(6));
            crate::player::apply_plr_damage(ctx, damage_type, target, 0, 0, damage as i32, DeathReason::Player);
        }
    }
    SIZE_TCMDDAMAGE
}

/// Original: `OnOperateObject` (msg.cpp).
// @port msg.cpp|devilution::OnOperateObject(const TCmd &pCmd, size_t pnum) sha=492d87e7b368
fn on_operate_object(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDLOC));
    } else {
        let position = position_of(v);
        assert!(in_dungeon_bounds(position));
        if crate::player::is_on_active_level(ctx, pnum) {
            if let Some(oi) = crate::objects::find_object_at_position(ctx, position, true) {
                crate::objects::sync_op_object(ctx, pnum, v.cmd() as i32, oi);
            }
        }
        delta_sync_object(ctx, (v.u8(1), v.u8(2)), v.cmd(), pnum);
    }
    SIZE_TCMDLOC
}

/// Original: `OnBreakObject` (msg.cpp).
// @port msg.cpp|devilution::OnBreakObject(const TCmd &pCmd, size_t pnum) sha=165a68406028
fn on_break_object(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDLOC));
    } else {
        let position = position_of(v);
        assert!(in_dungeon_bounds(position));
        if crate::player::is_on_active_level(ctx, pnum) {
            if let Some(oi) = crate::objects::find_object_at_position(ctx, position, true) {
                crate::objects::sync_break_obj(ctx, pnum, oi);
            }
        }
        delta_sync_object(ctx, (v.u8(1), v.u8(2)), CMD_BREAKOBJ, pnum);
    }
    SIZE_TCMDLOC
}

/// Original: `OnChangePlayerItems` (msg.cpp).
// @port msg.cpp|devilution::OnChangePlayerItems(const TCmd *pCmd, size_t pnum) sha=772807853fc0
fn on_change_player_items(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let b_loc = v.u8(1);
    if b_loc >= NUM_INVLOC {
        return SIZE_TCMDCHITEM;
    }
    let body_location = b_loc as inv_body_loc;
    let mi = NetItem::from_slice(&v.0[3.min(v.0.len())..]);
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDCHITEM));
    } else if ctx.players.MyPlayer != Some(pnum) && crate::items::is_item_available(ctx, mi.w_indx() as i32) {
        ctx.players.Players[pnum].InvBody[b_loc as usize] = Item::default();
        let player = ctx.players.Players[pnum].clone();
        let mut item = Item::default();
        recreate_item_from_union(ctx, &player, &mi, &mut item);
        ctx.players.Players[pnum].InvBody[b_loc as usize] = item;
        crate::inv::check_inv_swap_body(ctx, pnum, body_location);
    }
    crate::player::ready_spell_from_equipment(ctx, pnum, body_location, v.u8(2) != 0);
    SIZE_TCMDCHITEM
}

/// Original: `OnDeletePlayerItems` (msg.cpp).
// @port msg.cpp|devilution::OnDeletePlayerItems(const TCmd *pCmd, size_t pnum) sha=8e19fc78a713
fn on_delete_player_items(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs != 1 {
        if ctx.players.MyPlayer != Some(pnum) && v.u8(1) < NUM_INVLOC {
            crate::inv::inv_update_rem_item(ctx, pnum, v.u8(1) as inv_body_loc);
        }
    } else {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDDELITEM));
    }
    SIZE_TCMDDELITEM
}

/// Original: `OnChangeInventoryItems` (msg.cpp).
// @port msg.cpp|devilution::OnChangeInventoryItems(const TCmd *pCmd, int pnum) sha=240245986d45
fn on_change_inventory_items(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let b_loc = v.u8(1) as usize;
    if b_loc >= InventoryGridCells {
        return SIZE_TCMDCHITEM;
    }
    let mi = NetItem::from_slice(&v.0[3.min(v.0.len())..]);
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDCHITEM));
    } else if ctx.players.MyPlayer != Some(pnum) && crate::items::is_item_available(ctx, mi.w_indx() as i32) {
        let player = ctx.players.Players[pnum].clone();
        let mut item = Item::default();
        recreate_item_from_union(ctx, &player, &mi, &mut item);
        crate::inv::check_inv_swap_grid(ctx, pnum, &item, b_loc as i32);
    }
    SIZE_TCMDCHITEM
}

/// Original: `OnDeleteInventoryItems` (msg.cpp).
// @port msg.cpp|devilution::OnDeleteInventoryItems(const TCmd *pCmd, int pnum) sha=c70a7a2f7ece
fn on_delete_inventory_items(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let inv_grid_index = v.u16(1) as usize;
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDPARAM1));
    } else if ctx.players.MyPlayer != Some(pnum) && inv_grid_index < InventoryGridCells {
        crate::inv::check_inv_remove(ctx, pnum, inv_grid_index as i32);
    }
    SIZE_TCMDPARAM1
}

/// Original: `OnChangeBeltItems` (msg.cpp).
// @port msg.cpp|devilution::OnChangeBeltItems(const TCmd *pCmd, int pnum) sha=87f89c85917a
fn on_change_belt_items(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let b_loc = v.u8(1) as usize;
    if b_loc >= MaxBeltItems {
        return SIZE_TCMDCHITEM;
    }
    let mi = NetItem::from_slice(&v.0[3.min(v.0.len())..]);
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDCHITEM));
    } else if ctx.players.MyPlayer != Some(pnum) && crate::items::is_item_available(ctx, mi.w_indx() as i32) {
        ctx.players.Players[pnum].SpdList[b_loc] = Item::default();
        let player = ctx.players.Players[pnum].clone();
        let mut item = Item::default();
        recreate_item_from_union(ctx, &player, &mi, &mut item);
        ctx.players.Players[pnum].SpdList[b_loc] = item;
    }
    SIZE_TCMDCHITEM
}

/// Original: `OnDeleteBeltItems` (msg.cpp).
// @port msg.cpp|devilution::OnDeleteBeltItems(const TCmd *pCmd, int pnum) sha=110a8869bb04
fn on_delete_belt_items(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let spd_bar_index = v.u16(1) as usize;
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDPARAM1));
    } else if ctx.players.MyPlayer != Some(pnum) && spd_bar_index < MaxBeltItems {
        crate::player::remove_spd_bar_item(ctx, pnum, spd_bar_index as i32);
    }
    SIZE_TCMDPARAM1
}

/// Original: `OnPlayerLevel` (msg.cpp).
// @port msg.cpp|devilution::OnPlayerLevel(const TCmd *pCmd, size_t pnum) sha=3f8405e18c80
fn on_player_level(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let player_level = v.u16(1);
    if ctx.msg.gbBufferMsgs != 1 {
        if player_level as i32 <= MaxCharacterLevel && ctx.players.MyPlayer != Some(pnum) {
            ctx.players.Players[pnum]._pLevel = player_level as i8;
        }
    } else {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDPARAM1));
    }
    SIZE_TCMDPARAM1
}

/// Original: `OnDropItem` (msg.cpp).
// @port msg.cpp|devilution::OnDropItem(const TCmd *pCmd, size_t pnum) sha=acc7b78fb832
fn on_drop_item(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDPITEM));
    } else if is_p_item_valid(ctx, v, pnum) {
        delta_put_item(ctx, v, position_of(v), pnum);
    }
    SIZE_TCMDPITEM
}

/// Original: `OnSpawnItem` (msg.cpp).
// @port msg.cpp|devilution::OnSpawnItem(const TCmd *pCmd, size_t pnum) sha=c165be05fea8
fn on_spawn_item(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDPITEM));
    } else if is_p_item_valid(ctx, v, pnum) {
        if crate::player::is_on_active_level(ctx, pnum) && ctx.players.MyPlayer != Some(pnum) {
            sync_drop_cmd_item(ctx, v);
        }
        crate::items::put_item_record(ctx, v.u32(7), v.u16(5), v.i16(3) as i32);
        delta_put_item(ctx, v, position_of(v), pnum);
    }
    SIZE_TCMDPITEM
}

/// Original: `OnSendPlayerInfo` (msg.cpp).
// @port msg.cpp|devilution::OnSendPlayerInfo(const TCmd *pCmd, size_t pnum) sha=b9df5793b1cf
fn on_send_player_info(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let w_bytes = v.u16(3) as usize;
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, w_bytes + SIZE_TCMDPLRINFOHDR));
    } else {
        crate::multi::recv_plrinfo(ctx, pnum, v, v.cmd() == CMD_ACK_PLRINFO);
    }
    w_bytes + SIZE_TCMDPLRINFOHDR
}

/// Original: `OnPlayerJoinLevel` (msg.cpp).
// @port msg.cpp|devilution::OnPlayerJoinLevel(const TCmd *pCmd, size_t pnum) sha=170a52dcc2d3
fn on_player_join_level(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let position = position_of(v);
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDLOCPARAM2));
        return SIZE_TCMDLOCPARAM2;
    }
    let player_level = v.u16(3);
    let is_set_level = v.u16(5) != 0;
    if !is_valid_level(player_level as u8, is_set_level) || !in_dungeon_bounds(position) {
        return SIZE_TCMDLOCPARAM2;
    }

    ctx.players.Players[pnum]._pLvlChanging = false;
    if !ctx.players.Players[pnum]._pName.is_empty() && !ctx.players.Players[pnum].plractive {
        crate::player::reset_player_gfx(ctx, pnum);
        ctx.players.Players[pnum].plractive = true;
        ctx.multi.gbActivePlayers += 1;
        let p = &ctx.players.Players[pnum];
        let msg = tr("Player '{:s}' (level {:d}) just joined the game")
            .replacen("{:s}", p._pName.as_str(), 1)
            .replacen("{:d}", &p._pLevel.to_string(), 1);
        crate::plrmsg::event_plr_msg(ctx, &msg);
    }

    if ctx.players.Players[pnum].plractive && ctx.players.MyPlayer != Some(pnum) {
        ctx.players.Players[pnum].position.tile = position;
        crate::player::set_player_old(ctx, pnum);
        if is_set_level {
            ctx.players.Players[pnum].set_set_level(player_level as _setlevels);
        } else {
            ctx.players.Players[pnum].set_level(player_level as u8);
        }
        crate::player::reset_player_gfx(ctx, pnum);
        if crate::player::is_on_active_level(ctx, pnum) {
            crate::player::sync_init_plr(ctx, pnum);
            if (ctx.players.Players[pnum]._pHitPoints >> 6) > 0 {
                crate::player::start_stand(ctx, pnum, Direction::South);
            } else {
                ctx.players.Players[pnum]._pgfxnum &= !0xF;
                ctx.players.Players[pnum]._pmode = PM_DEATH;
                crate::player::new_plr_anim(ctx, pnum, player_graphic::Death, Direction::South, AnimationDistributionFlags::None, 0, 0);
                let p = &mut ctx.players.Players[pnum];
                p.AnimInfo.currentFrame = p.AnimInfo.numberOfFrames - 2;
                let t = p.position.tile;
                ctx.gendung.dFlags[t.x as usize][t.y as usize] |= DungeonFlag::DeadPlayer;
            }
            let (tile, rad) = (ctx.players.Players[pnum].position.tile, ctx.players.Players[pnum]._pLightRad);
            crate::lighting::activate_vision(ctx, tile, rad as i32, pnum as i32);
        }
    }
    SIZE_TCMDLOCPARAM2
}

/// Original: `OnActivatePortal` (msg.cpp).
// @port msg.cpp|devilution::OnActivatePortal(const TCmd *pCmd, size_t pnum) sha=4b45d7182d5c
fn on_activate_portal(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let position = position_of(v);
    let level = v.u16(3);
    let dungeon_type_idx = v.u16(5);
    let is_set_level = v.u16(7) != 0;
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDLOCPARAM3));
    } else if in_dungeon_bounds(position) && is_valid_level(level as u8, is_set_level) && dungeon_type_idx <= DungeonType::Crypt as u16 {
        let dungeon_type = DungeonType::from_i8(dungeon_type_idx as i8);
        crate::portal::activate_portal(ctx, pnum, position, level as i32, dungeon_type, is_set_level);
        if ctx.players.MyPlayer != Some(pnum) {
            if ctx.gendung.leveltype == DungeonType::Town {
                crate::portal::add_in_town_portal(ctx, pnum);
            } else if crate::player::is_on_active_level(ctx, pnum) {
                let mut add_portal = true;
                for missile in ctx.missiles.Missiles.iter() {
                    if missile._mitype == MissileID::TownPortal && missile._misource as usize == pnum {
                        add_portal = false;
                        break;
                    }
                }
                if add_portal {
                    crate::portal::add_warp_missile(ctx, pnum, position, false);
                }
            } else {
                crate::portal::remove_portal_missile(ctx, pnum);
            }
        }
        delta_open_portal(ctx, pnum, position, level as u8, dungeon_type, is_set_level);
    }
    SIZE_TCMDLOCPARAM3
}

/// Original: `OnDeactivatePortal` (msg.cpp).
// @port msg.cpp|devilution::OnDeactivatePortal(const TCmd *pCmd, size_t pnum) sha=749a270a84cb
fn on_deactivate_portal(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMD));
    } else {
        if crate::portal::portal_on_level(ctx, pnum) {
            crate::portal::remove_portal_missile(ctx, pnum);
        }
        crate::portal::deactivate_portal(ctx, pnum);
        delta_close_portal(ctx, pnum);
    }
    SIZE_TCMD
}

/// Original: `OnRestartTown` (msg.cpp).
// @port msg.cpp|devilution::OnRestartTown(const TCmd *pCmd, size_t pnum) sha=b721923ee26c
fn on_restart_town(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMD));
    } else {
        if pnum == ctx.players.MyPlayerId {
            ctx.players.MyPlayerIsDead = false;
            crate::gamemenu::gamemenu_off(ctx);
        }
        crate::player::restart_town_lvl(ctx, pnum);
    }
    SIZE_TCMD
}

fn on_set_stat(ctx: &mut Ctx, v: CmdView, pnum: usize, f: fn(&mut Ctx, usize, i32)) -> usize {
    let value = v.u16(1);
    if ctx.msg.gbBufferMsgs != 1 {
        if value <= 750 && ctx.players.MyPlayer != Some(pnum) {
            f(ctx, pnum, value as i32);
        }
    } else {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDPARAM1));
    }
    SIZE_TCMDPARAM1
}

/// Original: `OnSetStrength` (msg.cpp).
// @port msg.cpp|devilution::OnSetStrength(const TCmd *pCmd, size_t pnum) sha=5838b37da45f
fn on_set_strength(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    on_set_stat(ctx, v, pnum, crate::player::set_plr_str)
}

/// Original: `OnSetDexterity` (msg.cpp).
// @port msg.cpp|devilution::OnSetDexterity(const TCmd *pCmd, size_t pnum) sha=1f5562ca93ba
fn on_set_dexterity(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    on_set_stat(ctx, v, pnum, crate::player::set_plr_dex)
}

/// Original: `OnSetMagic` (msg.cpp).
// @port msg.cpp|devilution::OnSetMagic(const TCmd *pCmd, size_t pnum) sha=79c2c3f2b536
fn on_set_magic(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    on_set_stat(ctx, v, pnum, crate::player::set_plr_mag)
}

/// Original: `OnSetVitality` (msg.cpp).
// @port msg.cpp|devilution::OnSetVitality(const TCmd *pCmd, size_t pnum) sha=3b485aad0137
fn on_set_vitality(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    on_set_stat(ctx, v, pnum, crate::player::set_plr_vit)
}

/// Original: `OnString` (msg.cpp).
// @port msg.cpp|devilution::OnString(const TCmd *pCmd, Player &player) sha=881746a84c3a
fn on_string(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let raw = &v.0[1.min(v.0.len())..];
    let len = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    if ctx.msg.gbBufferMsgs == 0 {
        let s = String::from_utf8_lossy(&raw[..len]).into_owned();
        crate::plrmsg::send_plr_msg(ctx, pnum, &s);
    }
    len + 2
}

/// Original: `OnFriendlyMode` (msg.cpp).
// @port msg.cpp|devilution::OnFriendlyMode(const TCmd *pCmd, Player &player) sha=50b1d00d3efc
fn on_friendly_mode(ctx: &mut Ctx, pnum: usize) -> usize {
    let p = &mut ctx.players.Players[pnum];
    p.friendlyMode = !p.friendlyMode;
    crate::engine::backbuffer_state::redraw_everything(ctx);
    SIZE_TCMD
}

/// Original: `OnSyncQuest` (msg.cpp).
// @port msg.cpp|devilution::OnSyncQuest(const TCmd *pCmd, size_t pnum) sha=ef37cc639d84
fn on_sync_quest(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs == 1 {
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMDQUEST));
    } else {
        let q = v.i8(1);
        let qstate = v.u8(2);
        if pnum != ctx.players.MyPlayerId && (q as i32) < MAXQUESTS as i32 && qstate <= QUEST_HIVE_DONE {
            crate::quests::set_multi_quest(ctx, q as i32, qstate, v.u8(3) != 0, v.u8(4) as i32, v.u8(5) as i32, v.i16(6) as i32);
        }
    }
    SIZE_TCMDQUEST
}

/// Original: `OnCheatExperience` (msg.cpp); does nothing in release builds.
// @port msg.cpp|devilution::OnCheatExperience(const TCmd *pCmd, size_t pnum) sha=ac19300cb33b
fn on_cheat_experience() -> usize {
    SIZE_TCMD
}

/// Original: `OnChangeSpellLevel` (msg.cpp).
// @port msg.cpp|devilution::OnChangeSpellLevel(const TCmd *pCmd, size_t pnum) sha=9ccedc5ab8ca
fn on_change_spell_level(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    let spell_id = SpellID::from_raw(v.u16(1) as i8);
    let spell_level = (v.u16(3) as u8).min(MaxSpellLevel);
    if ctx.msg.gbBufferMsgs == 1 {
        // The original sends sizeof(*pCmd) (one byte) here.
        send_packet(ctx, pnum as i32, &v.bytes(0, SIZE_TCMD));
    } else {
        let p = &mut ctx.players.Players[pnum];
        p._pMemSpells |= crate::spells::get_spell_bitmask(spell_id);
        p._pSplLvl[spell_id as i8 as u8 as usize] = spell_level;
    }
    SIZE_TCMDPARAM2
}

/// Original: `OnDebug` (msg.cpp).
// @port msg.cpp|devilution::OnDebug(const TCmd *pCmd) sha=eb52b49d5ccf
fn on_debug() -> usize {
    SIZE_TCMD
}

/// Original: `OnSetShield` (msg.cpp).
// @port msg.cpp|devilution::OnSetShield(const TCmd *pCmd, Player &player) sha=8d634b65d76b
fn on_set_shield(ctx: &mut Ctx, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs != 1 {
        ctx.players.Players[pnum].pManaShield = true;
    }
    SIZE_TCMD
}

/// Original: `OnRemoveShield` (msg.cpp).
// @port msg.cpp|devilution::OnRemoveShield(const TCmd *pCmd, Player &player) sha=d66d8ae8b39d
fn on_remove_shield(ctx: &mut Ctx, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs != 1 {
        ctx.players.Players[pnum].pManaShield = false;
    }
    SIZE_TCMD
}

/// Original: `OnSetReflect` (msg.cpp).
// @port msg.cpp|devilution::OnSetReflect(const TCmd *pCmd, Player &player) sha=568263a4f244
fn on_set_reflect(ctx: &mut Ctx, v: CmdView, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs != 1 {
        ctx.players.Players[pnum].wReflections = v.u16(1);
    }
    SIZE_TCMDPARAM1
}

/// Original: `OnNakrul` (msg.cpp).
// @port msg.cpp|devilution::OnNakrul(const TCmd *pCmd) sha=b462fabe36f6
fn on_nakrul(ctx: &mut Ctx) -> usize {
    if ctx.msg.gbBufferMsgs != 1 {
        if ctx.gendung.currlevel == 24 {
            let (row, col) = crate::levels::crypt::uber_row_col(ctx);
            crate::effects::play_sfx_loc(ctx, crate::effects::IS_CROPEN, Point::new(row, col), true);
            crate::objects::sync_nakrul_room(ctx);
        }
        crate::levels::crypt::set_is_uber_room_opened(ctx, true);
        ctx.quests.Quests[Q_NAKRUL as usize]._qactive = QUEST_DONE;
        crate::monster::weaken_na_krul(ctx);
    }
    SIZE_TCMD
}

/// Original: `OnOpenHive` (msg.cpp).
// @port msg.cpp|devilution::OnOpenHive(const TCmd *pCmd, size_t pnum) sha=991576a3a850
fn on_open_hive(ctx: &mut Ctx, pnum: usize) -> usize {
    if ctx.msg.gbBufferMsgs != 1 {
        crate::missiles::add_missile(ctx, Point::new(0, 0), Point::new(0, 0), Direction::South, MissileID::OpenNest, crate::missiles::TARGET_MONSTERS, pnum as i32, 0, 0, None);
        crate::levels::town::town_open_hive(ctx);
        crate::levels::trigs::init_town_triggers(ctx);
    }
    SIZE_TCMD
}

/// Original: `OnOpenGrave` (msg.cpp).
// @port msg.cpp|devilution::OnOpenGrave(const TCmd *pCmd) sha=31c12f0dac9f
fn on_open_grave(ctx: &mut Ctx) -> usize {
    if ctx.msg.gbBufferMsgs != 1 {
        crate::levels::town::town_open_grave(ctx);
        crate::levels::trigs::init_town_triggers(ctx);
        if ctx.gendung.leveltype == DungeonType::Town {
            crate::effects::play_sfx(ctx, crate::effects::IS_SARC);
        }
    }
    SIZE_TCMD
}

/// Original: `devilution::PrepareItemForNetwork(const Item &item, TItem &messageItem)` (msg.cpp).
// @port msg.cpp|devilution::PrepareItemForNetwork(const Item &item, TItem &messageItem) sha=0aad1868d38e
pub fn prepare_item_for_network(item: &Item, m: &mut NetItem) {
    m.0[8] = item._iIdentified as u8;
    m.0[9] = item._iDurability as u8;
    m.0[10] = item._iMaxDur as u8;
    m.0[11] = item._iCharges as u8;
    m.0[12] = item._iMaxCharges as u8;
    put16(&mut m.0, 13, item._ivalue as u16);
    put16(&mut m.0, 19, item._iPLToHit as u16);
    put16(&mut m.0, 21, item._iMaxDam as u16);
    put32(&mut m.0, 15, item.dwBuff);
}

/// Original: `devilution::PrepareEarForNetwork` (msg.cpp).
// @port msg.cpp|devilution::PrepareEarForNetwork(const Item &item, TEar &ear) sha=40f7fdce2cb5
pub fn prepare_ear_for_network(item: &Item, m: &mut NetItem) {
    m.0[8] = (item._ivalue | ((item._iCurs as i32 - ICURS_EAR_SORCERER as i32) << 6)) as u8;
    let name = crate::utils::utf8::copy_utf8(item._iIName.as_str(), 17);
    for b in m.0[9..26].iter_mut() {
        *b = 0;
    }
    m.0[9..9 + name.len()].copy_from_slice(name.as_bytes());
}

/// Original: `devilution::RecreateItem(const Player &player, const TItem &messageItem, Item &item)` (msg.cpp).
// @port msg.cpp|devilution::RecreateItem(const Player &player, const TItem &messageItem, Item &item) sha=97daa565ad97
pub fn recreate_item(ctx: &mut Ctx, player: &crate::player::Player, m: &NetItem, item: &mut Item) {
    let dw_buff = m.dw_buff();
    crate::items::recreate_item(ctx, player, item, m.w_indx(), m.w_ci(), m.dw_seed(), m.w_value() as i32, (dw_buff & CF_HELLFIRE as u32) != 0);
    if m.b_id() != 0 {
        item._iIdentified = true;
    }
    item._iMaxDur = m.b_m_dur() as i32;
    item._iDurability = crate::inv::clamp_durability(item, m.b_dur() as i32);
    item._iMaxCharges = (m.b_m_ch() as i32).clamp(0, item._iMaxCharges);
    item._iCharges = (m.b_ch() as i32).clamp(0, item._iMaxCharges);
    if ctx.init.gb_is_hellfire {
        item._iPLToHit = crate::inv::clamp_to_hit(item, m.w_to_hit() as i16);
        item._iMaxDam = crate::inv::clamp_max_dam(item, m.w_max_dam() as u8);
    }
    item.dwBuff = dw_buff;
}

/// Original: `devilution::ClearLastSentPlayerCmd` (msg.cpp).
// @port msg.cpp|devilution::ClearLastSentPlayerCmd() sha=aaae4429b33d
pub fn clear_last_sent_player_cmd(ctx: &mut Ctx) {
    ctx.msg.last_sent_player_cmd = [0; SIZE_TCMDLOCPARAM5];
}

/// Original: `devilution::msg_send_drop_pkt` (msg.cpp).
// @port msg.cpp|devilution::msg_send_drop_pkt(int pnum, int reason) sha=c05ca08d8b09
pub fn msg_send_drop_pkt(ctx: &mut Ctx, pnum: i32, reason: i32) {
    let mut cmd = [0u8; SIZE_TFAKEDROPPLR];
    put32(&mut cmd, 2, reason as u32);
    cmd[0] = FAKE_CMD_DROPID;
    cmd[1] = pnum as u8;
    send_packet(ctx, pnum, &cmd);
}

/// Original: `devilution::msg_wait_resync` (msg.cpp).
// @port msg.cpp|devilution::msg_wait_resync() sha=bfc32573c17a
pub fn msg_wait_resync(ctx: &mut Ctx) -> bool {
    get_next_packet(ctx);
    ctx.msg.sgb_delta_chunks = 0;
    ctx.msg.sgn_curr_mega_player = -1;
    ctx.msg.sgb_recv_cmd = CMD_DLEVEL_END;
    ctx.msg.gbBufferMsgs = 1;
    ctx.msg.sgdw_owner_wait = ctx.platform.ticks();
    let success = crate::diablo_ui::progress::ui_progress_dialog(ctx, wait_for_turns);
    ctx.msg.gbBufferMsgs = 0;
    if !success {
        free_packets(ctx);
        return false;
    }
    if ctx.multi.gbGameDestroyed {
        crate::diablo_ui::dialogs::ui_error_ok_dialog(ctx, crate::diablo::PROJECT_NAME, &tr("The game ended"), false);
        free_packets(ctx);
        return false;
    }
    if ctx.msg.sgb_delta_chunks as usize != MAX_CHUNKS {
        crate::diablo_ui::dialogs::ui_error_ok_dialog(ctx, crate::diablo::PROJECT_NAME, &tr("Unable to get level data"), false);
        free_packets(ctx);
        return false;
    }
    true
}

/// Original: `devilution::run_delta_info` (msg.cpp).
// @port msg.cpp|devilution::run_delta_info() sha=84fb52e39326
pub fn run_delta_info(ctx: &mut Ctx) {
    if !ctx.init.gb_is_multiplayer {
        return;
    }
    ctx.msg.gbBufferMsgs = 2;
    pre_packet(ctx);
    ctx.msg.gbBufferMsgs = 0;
    free_packets(ctx);
}

/// Original: `devilution::DeltaExportData` (msg.cpp). Level order follows the delta map's keys.
// @port msg.cpp|devilution::DeltaExportData(int pnum) sha=d1ef51b405a6
pub fn delta_export_data(ctx: &mut Ctx, pnum: i32) {
    let mut keys: Vec<u8> = ctx.msg.delta_levels.keys().copied().collect();
    keys.sort();
    for key in keys {
        let level = ctx.msg.delta_levels[&key].clone();
        let mut dst: Vec<u8> = vec![0, key];
        delta_export_item(&mut dst, &level.item);
        delta_export_object(&mut dst, &level.object);
        delta_export_monster(&mut dst, &level.monster);
        let size = compress_data(&mut dst);
        crate::multi::multi_send_zero_packet(ctx, pnum as usize, CMD_DLEVEL, &dst[..size]);
    }
    let mut dst: Vec<u8> = vec![0];
    delta_export_junk(ctx, &mut dst);
    let size = compress_data(&mut dst);
    crate::multi::multi_send_zero_packet(ctx, pnum as usize, CMD_DLEVEL_JUNK, &dst[..size]);
    crate::multi::multi_send_zero_packet(ctx, pnum as usize, CMD_DLEVEL_END, &[0]);
}

/// Original: `devilution::delta_init` (msg.cpp).
// @port msg.cpp|devilution::delta_init() sha=3e6327f35eb0
pub fn delta_init(ctx: &mut Ctx) {
    ctx.msg.sg_junk = DJunk::ALL_FF;
    ctx.msg.delta_levels.clear();
    ctx.msg.local_levels.clear();
}

/// Original: `devilution::DeltaClearLevel` (msg.cpp).
// @port msg.cpp|devilution::DeltaClearLevel(uint8_t level) sha=91042a0dea88
pub fn delta_clear_level(ctx: &mut Ctx, level: u8) {
    ctx.msg.delta_levels.remove(&level);
    ctx.msg.local_levels.remove(&level);
}

/// Original: `devilution::delta_kill_monster` (msg.cpp).
// @port msg.cpp|devilution::delta_kill_monster(const Monster &monster, Point position, const Player &player) sha=1f3b232d8c49
pub fn delta_kill_monster(ctx: &mut Ctx, m: usize, position: Point, pnum: usize) {
    if !ctx.init.gb_is_multiplayer {
        return;
    }
    let lvl = get_level_for_multiplayer(ctx, pnum);
    let d = &mut get_delta_level(ctx, lvl).monster[m];
    d.x = position.x as u8;
    d.y = position.y as u8;
    d.hit_points = 0;
}

/// Original: `devilution::delta_monster_hp` (msg.cpp).
// @port msg.cpp|devilution::delta_monster_hp(const Monster &monster, const Player &player) sha=5b630aa318ed
pub fn delta_monster_hp(ctx: &mut Ctx, m: usize, pnum: usize) {
    if !ctx.init.gb_is_multiplayer {
        return;
    }
    let hp = ctx.monster.Monsters[m].hitPoints;
    let lvl = get_level_for_multiplayer(ctx, pnum);
    let d = &mut get_delta_level(ctx, lvl).monster[m];
    if d.hit_points > hp {
        d.hit_points = hp;
    }
}

/// Original: `devilution::delta_sync_monster` (msg.cpp).
// @port msg.cpp|devilution::delta_sync_monster(const TSyncMonster &monsterSync, uint8_t level) sha=f27f07a135de
pub fn delta_sync_monster(ctx: &mut Ctx, monster_sync: &TSyncMonster, level: u8) {
    if !ctx.init.gb_is_multiplayer {
        return;
    }
    assert!(level as usize <= MAX_MULTIPLAYERLEVELS);
    let monster = &mut get_delta_level(ctx, level).monster[monster_sync._mndx as usize];
    if monster.hit_points == 0 {
        return;
    }
    monster.x = monster_sync._mx;
    monster.y = monster_sync._my;
    monster.mactive = u8::MAX;
    monster.menemy = monster_sync._menemy;
    monster.hit_points = monster_sync._mhitpoints;
    monster.who_hit = monster_sync.mWhoHit;
}

/// Original: `devilution::DeltaSyncJunk` (msg.cpp).
// @port msg.cpp|devilution::DeltaSyncJunk() sha=0c0ba5948eb6
pub fn delta_sync_junk(ctx: &mut Ctx) {
    for i in 0..MAXPORTAL {
        let p = ctx.msg.sg_junk.portal[i];
        if p.x == 0xFF {
            crate::portal::set_portal_stats(ctx, i, false, Point::new(0, 0), 0, DungeonType::Town, false);
        } else {
            crate::portal::set_portal_stats(ctx, i, true, Point::new(p.x as i32, p.y as i32), p.level as i32, DungeonType::from_i8(p.ltype as i8), p.setlvl != 0);
        }
    }
    let mut q = 0;
    for qi in 0..ctx.quests.Quests.len() {
        let qidx = ctx.quests.Quests[qi]._qidx;
        if crate::quests::quest_data_is_single_player_only(qidx) && crate::quests::use_multiplayer_quests(ctx) {
            continue;
        }
        let mq = ctx.msg.sg_junk.quests[q];
        if mq.qstate != QUEST_INVALID {
            let quest = &mut ctx.quests.Quests[qi];
            quest._qlog = mq.qlog != 0;
            quest._qactive = mq.qstate;
            quest._qvar1 = mq.qvar1;
            quest._qvar2 = mq.qvar2;
            quest._qmsg = mq.qmsg as _speech_id;
        }
        q += 1;
    }
}

/// Original: `devilution::DeltaAddItem` (msg.cpp).
// @port msg.cpp|devilution::DeltaAddItem(int ii) sha=c1bae1d06d1c
pub fn delta_add_item(ctx: &mut Ctx, ii: i32) {
    if !ctx.init.gb_is_multiplayer {
        return;
    }
    let me = ctx.players.MyPlayer.unwrap();
    let local_level = get_level_for_multiplayer(ctx, me);
    let item = ctx.items.Items[ii as usize].clone();
    let level = get_delta_level(ctx, local_level);
    for d in level.item.iter() {
        let n = NetItem::from_slice(&d[3..]);
        if d[0] != CMD_INVALID && n.w_indx() == item.IDidx && n.w_ci() == item._iCreateInfo && n.dw_seed() == item._iSeed && (d[0] == PickedUpItem || d[0] == FloorItem) {
            return;
        }
    }
    for d in level.item.iter_mut() {
        if d[0] != CMD_INVALID {
            continue;
        }
        d[0] = FloorItem;
        d[1] = item.position.x as u8;
        d[2] = item.position.y as u8;
        let n = prepare_item_union(&item);
        d[3..].copy_from_slice(&n.0);
        return;
    }
}

/// Original: `devilution::DeltaSaveLevel` (msg.cpp).
// @port msg.cpp|devilution::DeltaSaveLevel() sha=e81f2d955c8d
pub fn delta_save_level(ctx: &mut Ctx) {
    if !ctx.init.gb_is_multiplayer {
        return;
    }
    let me = ctx.players.MyPlayer;
    for pnum in 0..ctx.players.Players.len() {
        if Some(pnum) != me {
            crate::player::reset_player_gfx(ctx, pnum);
        }
    }
    let me = me.unwrap();
    let local_level;
    if ctx.gendung.setlevel {
        local_level = get_level_for_multiplayer_raw(ctx.gendung.setlvlnum as u8, true);
        let s = ctx.gendung.setlvlnum as usize;
        ctx.players.Players[me]._pSLvlVisited[s] = true;
    } else {
        local_level = get_level_for_multiplayer_raw(ctx.gendung.currlevel, false);
        let c = ctx.gendung.currlevel as usize;
        ctx.players.Players[me]._pLvlVisited[c] = true;
    }
    delta_leave_sync(ctx, local_level);
}

/// Original: `devilution::GetLevelForMultiplayer(const Player &player)` (msg.cpp).
// @port msg.cpp|devilution::GetLevelForMultiplayer(const Player &player) sha=573273ed1363
pub fn get_level_for_multiplayer(ctx: &Ctx, pnum: usize) -> u8 {
    let p = &ctx.players.Players[pnum];
    get_level_for_multiplayer_raw(p.plrlevel, p.plrIsOnSetLevel)
}

/// Original: `devilution::IsValidLevelForMultiplayer` (msg.cpp).
// @port msg.cpp|devilution::IsValidLevelForMultiplayer(uint8_t level) sha=2e2bdb9f1026
pub fn is_valid_level_for_multiplayer(level: u8) -> bool {
    level as usize <= MAX_MULTIPLAYERLEVELS
}

/// Original: `devilution::IsValidLevel` (msg.cpp).
// @port msg.cpp|devilution::IsValidLevel(uint8_t level, bool isSetLevel) sha=96425e3de203
pub fn is_valid_level(level: u8, is_set_level: bool) -> bool {
    if is_set_level {
        return level <= SL_LAST as u8;
    }
    (level as usize) < NUMLEVELS
}

/// Original: `devilution::DeltaLoadLevel` (msg.cpp).
// @port msg.cpp|devilution::DeltaLoadLevel() sha=c3a57ac67f32
pub fn delta_load_level(ctx: &mut Ctx) {
    if !ctx.init.gb_is_multiplayer {
        return;
    }
    let me = ctx.players.MyPlayer.unwrap();
    let local_level = get_level_for_multiplayer(ctx, me);
    let mut level = get_delta_level(ctx, local_level).clone();
    if ctx.gendung.leveltype != DungeonType::Town {
        for i in 0..MaxMonsters {
            let d = level.monster[i];
            if d.x == 0xFF {
                continue;
            }
            crate::monster::m_clear_squares(ctx, i);
            {
                let position = Point::new(d.x as i32, d.y as i32);
                let monster = &mut ctx.monster.Monsters[i];
                monster.position.tile = position;
                monster.position.old = position;
                monster.position.future = position;
                let light_id = monster.lightId;
                if light_id as i32 != crate::lighting::NO_LIGHT {
                    crate::lighting::change_light_xy(ctx, light_id as i32, position);
                }
            }
            if d.hit_points != -1 {
                ctx.monster.Monsters[i].hitPoints = d.hit_points;
                ctx.monster.Monsters[i].whoHit = d.who_hit;
            }
            if d.hit_points == 0 {
                crate::monster::m_clear_squares(ctx, i);
                let monster = &ctx.monster.Monsters[i];
                if monster.ai != MonsterAIID::Diablo {
                    let (tile, dir) = (monster.position.tile, monster.direction);
                    let corpse = if monster.is_unique() { monster.corpseId } else { crate::monster::monster_corpse_id(ctx, i) };
                    crate::dead::add_corpse(ctx, tile, corpse, dir);
                }
                ctx.monster.Monsters[i].isInvalid = true;
                crate::monster::m_update_relations(ctx, i);
            } else {
                crate::monster::decode_enemy(ctx, i, d.menemy as i32);
                let tile = ctx.monster.Monsters[i].position.tile;
                if tile != Point::new(0, 0) && tile != crate::monster::GOLEM_HOLDING_CELL {
                    ctx.gendung.dMonster[tile.x as usize][tile.y as usize] = i as i16 + 1;
                }
                if crate::monster::monster_type_id(ctx, i) == MT_GOLEM {
                    crate::monster::golum_ai(ctx, i);
                    ctx.monster.Monsters[i].flags |= (MFLAG_TARGETS_MONSTER | MFLAG_GOLEM) as u32;
                } else {
                    let dir = ctx.monster.Monsters[i].direction;
                    crate::monster::m_start_stand(ctx, i, dir);
                }
                ctx.monster.Monsters[i].activeForTicks = d.mactive;
            }
        }
        match ctx.msg.local_levels.get(&local_level) {
            Some(v) => ctx.automap.AutomapView = v.clone(),
            None => ctx.automap.AutomapView = Box::new([[0; DMAXY]; DMAXX]),
        }
    }

    if ctx.gendung.leveltype != DungeonType::Town {
        let mut kept = Vec::new();
        for &(pos, cmd) in level.object.iter() {
            let Some(oi) = crate::objects::find_object_at_position(ctx, Point::new(pos.0 as i32, pos.1 as i32), true) else {
                continue;
            };
            match cmd {
                CMD_OPENDOOR | CMD_OPERATEOBJ => {
                    crate::objects::delta_sync_op_object(ctx, oi);
                    kept.push((pos, cmd));
                }
                CMD_CLOSEDOOR => {
                    crate::objects::delta_sync_close_obj(ctx, oi);
                    kept.push((pos, cmd));
                }
                CMD_BREAKOBJ => {
                    crate::objects::delta_sync_break_obj(ctx, oi);
                    kept.push((pos, cmd));
                }
                _ => {}
            }
        }
        level.object = kept;
        for i in 0..ctx.objects.ActiveObjectCount as usize {
            let oi = ctx.objects.ActiveObjects[i] as usize;
            if ctx.objects.Objects[oi].is_trap() {
                crate::objects::update_trap_state(ctx, oi);
            }
        }
    }
    ctx.msg.delta_levels.insert(local_level, level.clone());

    for i in 0..MAXITEMS {
        let d = level.item[i];
        if d[0] == CMD_INVALID {
            continue;
        }
        let n = NetItem::from_slice(&d[3..]);
        if d[0] == PickedUpItem {
            let active_item_index = crate::inv::find_get_item(ctx, n.dw_seed(), n.w_indx(), n.w_ci());
            if active_item_index != -1 {
                let ai = ctx.items.ActiveItems[active_item_index as usize];
                let position = ctx.items.Items[ai as usize].position;
                if ctx.items.dItem[position.x as usize][position.y as usize] as i32 == ai as i32 + 1 {
                    ctx.items.dItem[position.x as usize][position.y as usize] = 0;
                }
                crate::items::delete_item(ctx, active_item_index);
            }
        }
        if d[0] == DroppedItem {
            let ii = crate::items::allocate_item(ctx) as usize;
            let player = ctx.players.Players[me].clone();
            let mut item = std::mem::take(&mut ctx.items.Items[ii]);
            recreate_item_from_union(ctx, &player, &n, &mut item);
            let pos = get_item_position(ctx, Point::new(d[1] as i32, d[2] as i32));
            item.position = pos;
            ctx.items.dItem[pos.x as usize][pos.y as usize] = (ii + 1) as i8;
            crate::items::respawn_item(ctx, &mut item, false);
            ctx.items.Items[ii] = item;
        }
    }
}

fn send_cmd(ctx: &mut Ctx, b_hi_pri: bool, cmd: &[u8]) {
    let my = ctx.players.MyPlayerId as i32;
    if b_hi_pri {
        net_send_hi_pri(ctx, my, cmd);
    } else {
        net_send_lo_pri(ctx, my, cmd);
    }
}

fn update_preview(ctx: &mut Ctx, b_cmd: _cmd_id, position: Point, w1: u16, w2: u16) {
    let me = ctx.players.MyPlayer.unwrap();
    crate::player::update_preview_cel_sprite(ctx, me, b_cmd, position, w1, w2);
}

/// Original: `devilution::NetSendCmd` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmd(bool bHiPri, _cmd_id bCmd) sha=c3477d8e9207
pub fn net_send_cmd(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id) {
    send_cmd(ctx, b_hi_pri, &[b_cmd]);
}

/// Original: `devilution::NetSendCmdGolem` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdGolem(uint8_t mx, uint8_t my, Direction dir, uint8_t menemy, int hp, uint8_t cl) sha=05e4d40ecbb7
pub fn net_send_cmd_golem(ctx: &mut Ctx, mx: u8, my: u8, dir: Direction, menemy: u8, hp: i32, cl: u8) {
    let mut cmd = [0u8; SIZE_TCMDGOLEM];
    cmd[0] = CMD_AWAKEGOLEM;
    cmd[1] = mx;
    cmd[2] = my;
    cmd[3] = dir as u8;
    cmd[4] = menemy;
    put32(&mut cmd, 5, hp as u32);
    cmd[9] = cl;
    let me = ctx.players.MyPlayerId as i32;
    net_send_lo_pri(ctx, me, &cmd);
}

/// Original: `devilution::NetSendCmdLoc` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdLoc(size_t playerId, bool bHiPri, _cmd_id bCmd, Point position) sha=88f6f8f79227
pub fn net_send_cmd_loc(ctx: &mut Ctx, player_id: usize, b_hi_pri: bool, b_cmd: _cmd_id, position: Point) {
    if player_id == ctx.players.MyPlayerId && was_player_cmd_already_requested(ctx, b_cmd, position, 0, 0, 0, 0, 0) {
        return;
    }
    let cmd = [b_cmd, position.x as u8, position.y as u8];
    if b_hi_pri {
        net_send_hi_pri(ctx, player_id as i32, &cmd);
    } else {
        net_send_lo_pri(ctx, player_id as i32, &cmd);
    }
    update_preview(ctx, b_cmd, position, 0, 0);
}

fn loc_params(b_cmd: _cmd_id, position: Point, params: &[u16]) -> Vec<u8> {
    let mut cmd = vec![b_cmd, position.x as u8, position.y as u8];
    for &p in params {
        cmd.extend_from_slice(&p.to_le_bytes());
    }
    cmd
}

/// Original: `devilution::NetSendCmdLocParam1` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdLocParam1(bool bHiPri, _cmd_id bCmd, Point position, uint16_t wParam1) sha=4fe24b56b509
pub fn net_send_cmd_loc_param1(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, position: Point, w1: u16) {
    if was_player_cmd_already_requested(ctx, b_cmd, position, w1, 0, 0, 0, 0) {
        return;
    }
    send_cmd(ctx, b_hi_pri, &loc_params(b_cmd, position, &[w1]));
    update_preview(ctx, b_cmd, position, w1, 0);
}

/// Original: `devilution::NetSendCmdLocParam2` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdLocParam2(bool bHiPri, _cmd_id bCmd, Point position, uint16_t wParam1, uint16_t wParam2) sha=2c67937f395a
pub fn net_send_cmd_loc_param2(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, position: Point, w1: u16, w2: u16) {
    if was_player_cmd_already_requested(ctx, b_cmd, position, w1, w2, 0, 0, 0) {
        return;
    }
    send_cmd(ctx, b_hi_pri, &loc_params(b_cmd, position, &[w1, w2]));
    update_preview(ctx, b_cmd, position, w1, w2);
}

/// Original: `devilution::NetSendCmdLocParam3` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdLocParam3(bool bHiPri, _cmd_id bCmd, Point position, uint16_t wParam1, uint16_t wParam2, uint16_t wParam3) sha=7ba08d0fb296
pub fn net_send_cmd_loc_param3(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, position: Point, w1: u16, w2: u16, w3: u16) {
    if was_player_cmd_already_requested(ctx, b_cmd, position, w1, w2, w3, 0, 0) {
        return;
    }
    send_cmd(ctx, b_hi_pri, &loc_params(b_cmd, position, &[w1, w2, w3]));
    update_preview(ctx, b_cmd, position, w1, w2);
}

/// Original: `devilution::NetSendCmdLocParam4` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdLocParam4(bool bHiPri, _cmd_id bCmd, Point position, uint16_t wParam1, uint16_t wParam2, uint16_t wParam3, uint16_t wParam4) sha=2d96f58208af
pub fn net_send_cmd_loc_param4(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, position: Point, w1: u16, w2: u16, w3: u16, w4: u16) {
    if was_player_cmd_already_requested(ctx, b_cmd, position, w1, w2, w3, w4, 0) {
        return;
    }
    send_cmd(ctx, b_hi_pri, &loc_params(b_cmd, position, &[w1, w2, w3, w4]));
    update_preview(ctx, b_cmd, position, w1, w3);
}

/// Original: `devilution::NetSendCmdLocParam5` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdLocParam5(bool bHiPri, _cmd_id bCmd, Point position, uint16_t wParam1, uint16_t wParam2, uint16_t wParam3, uint16_t wParam4, uint16_t wParam5) sha=efe39e7b69f6
#[allow(clippy::too_many_arguments)]
pub fn net_send_cmd_loc_param5(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, position: Point, w1: u16, w2: u16, w3: u16, w4: u16, w5: u16) {
    if was_player_cmd_already_requested(ctx, b_cmd, position, w1, w2, w3, w4, w5) {
        return;
    }
    send_cmd(ctx, b_hi_pri, &loc_params(b_cmd, position, &[w1, w2, w3, w4, w5]));
    update_preview(ctx, b_cmd, position, w1, w3);
}

/// Original: `devilution::NetSendCmdParam1` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdParam1(bool bHiPri, _cmd_id bCmd, uint16_t wParam1) sha=b9dc0188177c
pub fn net_send_cmd_param1(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, w1: u16) {
    if was_player_cmd_already_requested(ctx, b_cmd, Point::default(), w1, 0, 0, 0, 0) {
        return;
    }
    let mut cmd = vec![b_cmd];
    cmd.extend_from_slice(&w1.to_le_bytes());
    send_cmd(ctx, b_hi_pri, &cmd);
    update_preview(ctx, b_cmd, Point::default(), w1, 0);
}

/// Original: `devilution::NetSendCmdParam2` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdParam2(bool bHiPri, _cmd_id bCmd, uint16_t wParam1, uint16_t wParam2) sha=f14ed73c5efd
pub fn net_send_cmd_param2(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, w1: u16, w2: u16) {
    let mut cmd = vec![b_cmd];
    cmd.extend_from_slice(&w1.to_le_bytes());
    cmd.extend_from_slice(&w2.to_le_bytes());
    send_cmd(ctx, b_hi_pri, &cmd);
}

/// Original: `devilution::NetSendCmdParam5` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdParam5(bool bHiPri, _cmd_id bCmd, uint16_t wParam1, uint16_t wParam2, uint16_t wParam3, uint16_t wParam4, uint16_t wParam5) sha=ce9c4d25017d
#[allow(clippy::too_many_arguments)]
pub fn net_send_cmd_param5(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, w1: u16, w2: u16, w3: u16, w4: u16, w5: u16) {
    if was_player_cmd_already_requested(ctx, b_cmd, Point::default(), w1, w2, w3, w4, 0) {
        return;
    }
    let mut cmd = vec![b_cmd];
    for p in [w1, w2, w3, w4, w5] {
        cmd.extend_from_slice(&p.to_le_bytes());
    }
    send_cmd(ctx, b_hi_pri, &cmd);
    update_preview(ctx, b_cmd, Point::default(), w1, w2);
}

/// Original: `devilution::NetSendCmdQuest` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdQuest(bool bHiPri, const Quest &quest) sha=6193c029c7df
pub fn net_send_cmd_quest(ctx: &mut Ctx, b_hi_pri: bool, q: usize) {
    let quest = ctx.quests.Quests[q];
    let mut cmd = [0u8; SIZE_TCMDQUEST];
    cmd[0] = CMD_SYNCQUEST;
    cmd[1] = quest._qidx as u8;
    cmd[2] = quest._qactive;
    cmd[3] = quest._qlog as u8;
    cmd[4] = quest._qvar1;
    cmd[5] = quest._qvar2;
    put16(&mut cmd, 6, quest._qmsg as i16 as u16);
    send_cmd(ctx, b_hi_pri, &cmd);
}

/// Original: `devilution::NetSendCmdGItem` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdGItem(bool bHiPri, _cmd_id bCmd, uint8_t pnum, uint8_t ii) sha=cad812fccd87
pub fn net_send_cmd_g_item(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, pnum: u8, ii: u8) {
    // `TCmdGItem cmd;` is uninitialised in the original; only the union part may keep stack
    // bytes (the unused tail of TItem). They start as zero here.
    let mut cmd = [0u8; SIZE_TCMDGITEM];
    cmd[0] = b_cmd;
    cmd[30] = pnum;
    cmd[29] = pnum;
    let me = ctx.players.MyPlayer.unwrap();
    cmd[32] = get_level_for_multiplayer(ctx, me);
    cmd[31] = ii;
    put32(&mut cmd, 33, 0);
    let item = &ctx.items.Items[ii as usize];
    cmd[1] = item.position.x as u8;
    cmd[2] = item.position.y as u8;
    let n = prepare_item_union(item);
    cmd[3..29].copy_from_slice(&n.0);
    send_cmd(ctx, b_hi_pri, &cmd);
}

/// Original: `devilution::NetSendCmdPItem` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdPItem(bool bHiPri, _cmd_id bCmd, Point position, const Item &item) sha=1334496c7cd8
pub fn net_send_cmd_p_item(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, position: Point, item: &Item) {
    let mut cmd = [0u8; SIZE_TCMDPITEM];
    cmd[0] = b_cmd;
    cmd[1] = position.x as u8;
    cmd[2] = position.y as u8;
    let n = prepare_item_union(item);
    cmd[3..].copy_from_slice(&n.0);
    ctx.msg.item_limbo = item.clone();
    send_cmd(ctx, b_hi_pri, &cmd);
}

/// Original: `devilution::NetSendCmdChItem` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdChItem(bool bHiPri, uint8_t bLoc, bool forceSpellChange) sha=ae6da107141c
pub fn net_send_cmd_ch_item(ctx: &mut Ctx, b_hi_pri: bool, b_loc: u8, force_spell_change: bool) {
    let me = ctx.players.MyPlayer.unwrap();
    let item = &ctx.players.Players[me].InvBody[b_loc as usize];
    let mut cmd = [0u8; SIZE_TCMDCHITEM];
    cmd[0] = CMD_CHANGEPLRITEMS;
    cmd[1] = b_loc;
    cmd[2] = force_spell_change as u8;
    cmd[3..].copy_from_slice(&prepare_item_union(item).0);
    send_cmd(ctx, b_hi_pri, &cmd);
}

/// Original: `devilution::NetSendCmdDelItem` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdDelItem(bool bHiPri, uint8_t bLoc) sha=ef136acecdc8
pub fn net_send_cmd_del_item(ctx: &mut Ctx, b_hi_pri: bool, b_loc: u8) {
    send_cmd(ctx, b_hi_pri, &[CMD_DELPLRITEMS, b_loc]);
}

/// Original: `devilution::NetSyncInvItem` (msg.cpp).
// @port msg.cpp|devilution::NetSyncInvItem(const Player &player, int invListIndex) sha=142b23ba4e48
pub fn net_sync_inv_item(ctx: &mut Ctx, pnum: usize, inv_list_index: i32) {
    if ctx.players.MyPlayer != Some(pnum) {
        return;
    }
    for j in 0..InventoryGridCells {
        if ctx.players.Players[pnum].InvGrid[j] as i32 == inv_list_index + 1 {
            net_send_cmd_ch_inv_item(ctx, false, j as i32);
            break;
        }
    }
}

/// Original: `devilution::NetSendCmdChInvItem` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdChInvItem(bool bHiPri, int invGridIndex) sha=a158827d20d5
pub fn net_send_cmd_ch_inv_item(ctx: &mut Ctx, b_hi_pri: bool, inv_grid_index: i32) {
    let me = ctx.players.MyPlayer.unwrap();
    let inv_list_index = (ctx.players.Players[me].InvGrid[inv_grid_index as usize] as i32).abs() - 1;
    let item = &ctx.players.Players[me].InvList[inv_list_index as usize];
    let mut cmd = [0u8; SIZE_TCMDCHITEM];
    cmd[0] = CMD_CHANGEINVITEMS;
    cmd[1] = inv_grid_index as u8;
    cmd[3..].copy_from_slice(&prepare_item_union(item).0);
    send_cmd(ctx, b_hi_pri, &cmd);
}

/// Original: `devilution::NetSendCmdChBeltItem` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdChBeltItem(bool bHiPri, int beltIndex) sha=5def2f31a75e
pub fn net_send_cmd_ch_belt_item(ctx: &mut Ctx, b_hi_pri: bool, belt_index: i32) {
    let me = ctx.players.MyPlayer.unwrap();
    let item = &ctx.players.Players[me].SpdList[belt_index as usize];
    let mut cmd = [0u8; SIZE_TCMDCHITEM];
    cmd[0] = CMD_CHANGEBELTITEMS;
    cmd[1] = belt_index as u8;
    cmd[3..].copy_from_slice(&prepare_item_union(item).0);
    send_cmd(ctx, b_hi_pri, &cmd);
}

/// Original: `devilution::NetSendCmdDamage` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdDamage(bool bHiPri, uint8_t bPlr, uint32_t dwDam, DamageType damageType) sha=2006751aa37c
pub fn net_send_cmd_damage(ctx: &mut Ctx, b_hi_pri: bool, b_plr: u8, dw_dam: u32, damage_type: DamageType) {
    let mut cmd = [0u8; SIZE_TCMDDAMAGE];
    cmd[0] = CMD_PLRDAMAGE;
    cmd[1] = b_plr;
    put32(&mut cmd, 2, dw_dam);
    cmd[6] = damage_type as u8;
    send_cmd(ctx, b_hi_pri, &cmd);
}

/// Original: `devilution::NetSendCmdMonDmg` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdMonDmg(bool bHiPri, uint16_t wMon, uint32_t dwDam) sha=d79c66ec99fa
pub fn net_send_cmd_mon_dmg(ctx: &mut Ctx, b_hi_pri: bool, w_mon: u16, dw_dam: u32) {
    let mut cmd = [0u8; SIZE_TCMDMONDAMAGE];
    cmd[0] = CMD_MONSTDAMAGE;
    put16(&mut cmd, 1, w_mon);
    put32(&mut cmd, 3, dw_dam);
    send_cmd(ctx, b_hi_pri, &cmd);
}

/// Original: `devilution::NetSendCmdString` (msg.cpp).
// @port msg.cpp|devilution::NetSendCmdString(uint32_t pmask, const char *pszStr) sha=24fe78637865
pub fn net_send_cmd_string(ctx: &mut Ctx, pmask: u32, s: &str) {
    let t = crate::utils::utf8::copy_utf8(s, MAX_SEND_STR_LEN);
    let mut cmd = vec![CMD_STRING];
    cmd.extend_from_slice(t.as_bytes());
    cmd.push(0);
    crate::multi::multi_send_msg_packet(ctx, pmask, &cmd);
}

/// Original: `devilution::delta_close_portal` (msg.cpp).
// @port msg.cpp|devilution::delta_close_portal(int pnum) sha=d2f62dbaefcb
pub fn delta_close_portal(ctx: &mut Ctx, pnum: usize) {
    ctx.msg.sg_junk.portal[pnum] = DPortal { x: 0xFF, y: 0xFF, level: 0xFF, ltype: 0xFF, setlvl: 0xFF };
}

/// Original: `devilution::ParseCmd` (msg.cpp). `data` starts at the command; returns its size,
/// or 0 when it cannot be parsed.
// @port msg.cpp|devilution::ParseCmd(size_t pnum, const TCmd *pCmd) sha=eaf39fb8bdfd
pub fn parse_cmd(ctx: &mut Ctx, pnum: usize, data: &[u8]) -> usize {
    let v = CmdView(data);
    ctx.msg.sb_last_cmd = v.cmd();
    if ctx.multi.sgwPackPlrOffsetTbl[pnum] != 0 && ctx.msg.sb_last_cmd != CMD_ACK_PLRINFO && ctx.msg.sb_last_cmd != CMD_SEND_PLRINFO {
        return 0;
    }
    match v.cmd() {
        CMD_SYNCDATA => return crate::sync::on_sync_data(ctx, data, pnum),
        CMD_WALKXY => return on_walk(ctx, v, pnum),
        CMD_ADDSTR => return on_add_strength(ctx, v, pnum),
        CMD_ADDDEX => return on_add_dexterity(ctx, v, pnum),
        CMD_ADDMAG => return on_add_magic(ctx, v, pnum),
        CMD_ADDVIT => return on_add_vitality(ctx, v, pnum),
        CMD_GOTOGETITEM => return on_goto_get_item(ctx, v, pnum),
        CMD_REQUESTGITEM => return on_request_get_item(ctx, v, pnum),
        CMD_GETITEM => return on_get_item(ctx, v, pnum),
        CMD_GOTOAGETITEM => return on_goto_auto_get_item(ctx, v, pnum),
        CMD_REQUESTAGITEM => return on_request_auto_get_item(ctx, v, pnum),
        CMD_AGETITEM => return on_auto_get_item(ctx, v, pnum),
        CMD_ITEMEXTRA => return on_item_extra(ctx, v, pnum),
        CMD_PUTITEM => return on_put_item(ctx, v, pnum),
        CMD_SYNCPUTITEM => return on_sync_put_item(ctx, v, pnum),
        CMD_SPAWNITEM => return on_spawn_item(ctx, v, pnum),
        CMD_ATTACKXY => return on_attack_tile(ctx, v, pnum),
        CMD_SATTACKXY => return on_standing_attack_tile(ctx, v, pnum),
        CMD_RATTACKXY => return on_ranged_attack_tile(ctx, v, pnum),
        CMD_SPELLXYD => return on_spell_wall(ctx, v, pnum),
        CMD_SPELLXY => return on_spell_tile(ctx, v, pnum),
        CMD_OPOBJXY => return on_object_tile_action(ctx, v, pnum, ACTION_OPERATE, true),
        CMD_DISARMXY => return on_object_tile_action(ctx, v, pnum, ACTION_DISARM, true),
        CMD_OPOBJT => return on_object_tile_action(ctx, v, pnum, ACTION_OPERATETK, false),
        CMD_ATTACKID => return on_attack_monster(ctx, v, pnum),
        CMD_ATTACKPID => return on_attack_player(ctx, v, pnum),
        CMD_RATTACKID => return on_ranged_attack_monster(ctx, v, pnum),
        CMD_RATTACKPID => return on_ranged_attack_player(ctx, v, pnum),
        CMD_SPELLID => return on_spell_monster(ctx, v, pnum),
        CMD_SPELLPID => return on_spell_player(ctx, v, pnum),
        CMD_KNOCKBACK => return on_knockback(ctx, v, pnum),
        CMD_RESURRECT => return on_resurrect(ctx, v, pnum),
        CMD_HEALOTHER => return on_heal_other(ctx, v, pnum),
        CMD_TALKXY => return on_talk_xy(ctx, v, pnum),
        CMD_DEBUG => return on_debug(),
        CMD_NEWLVL => return on_new_level(ctx, v, pnum),
        CMD_WARP => return on_warp(ctx, v, pnum),
        CMD_MONSTDEATH => return on_monst_death(ctx, v, pnum),
        CMD_KILLGOLEM => return on_kill_golem(ctx, v, pnum),
        CMD_AWAKEGOLEM => return on_awake_golem(ctx, v, pnum),
        CMD_MONSTDAMAGE => return on_monst_damage(ctx, v, pnum),
        CMD_PLRDEAD => return on_player_death(ctx, v, pnum),
        CMD_PLRDAMAGE => return on_player_damage(ctx, v, pnum),
        CMD_OPENDOOR | CMD_CLOSEDOOR | CMD_OPERATEOBJ => return on_operate_object(ctx, v, pnum),
        CMD_BREAKOBJ => return on_break_object(ctx, v, pnum),
        CMD_CHANGEPLRITEMS => return on_change_player_items(ctx, v, pnum),
        CMD_DELPLRITEMS => return on_delete_player_items(ctx, v, pnum),
        CMD_CHANGEINVITEMS => return on_change_inventory_items(ctx, v, pnum),
        CMD_DELINVITEMS => return on_delete_inventory_items(ctx, v, pnum),
        CMD_CHANGEBELTITEMS => return on_change_belt_items(ctx, v, pnum),
        CMD_DELBELTITEMS => return on_delete_belt_items(ctx, v, pnum),
        CMD_PLRLEVEL => return on_player_level(ctx, v, pnum),
        CMD_DROPITEM => return on_drop_item(ctx, v, pnum),
        CMD_ACK_PLRINFO | CMD_SEND_PLRINFO => return on_send_player_info(ctx, v, pnum),
        CMD_PLAYER_JOINLEVEL => return on_player_join_level(ctx, v, pnum),
        CMD_ACTIVATEPORTAL => return on_activate_portal(ctx, v, pnum),
        CMD_DEACTIVATEPORTAL => return on_deactivate_portal(ctx, v, pnum),
        CMD_RETOWN => return on_restart_town(ctx, v, pnum),
        CMD_SETSTR => return on_set_strength(ctx, v, pnum),
        CMD_SETMAG => return on_set_magic(ctx, v, pnum),
        CMD_SETDEX => return on_set_dexterity(ctx, v, pnum),
        CMD_SETVIT => return on_set_vitality(ctx, v, pnum),
        CMD_STRING => return on_string(ctx, v, pnum),
        CMD_FRIENDLYMODE => return on_friendly_mode(ctx, pnum),
        CMD_SYNCQUEST => return on_sync_quest(ctx, v, pnum),
        CMD_CHEAT_EXPERIENCE => return on_cheat_experience(),
        CMD_CHANGE_SPELL_LEVEL => return on_change_spell_level(ctx, v, pnum),
        CMD_SETSHIELD => return on_set_shield(ctx, pnum),
        CMD_REMSHIELD => return on_remove_shield(ctx, pnum),
        CMD_SETREFLECT => return on_set_reflect(ctx, v, pnum),
        CMD_NAKRUL => return on_nakrul(ctx),
        CMD_OPENHIVE => return on_open_hive(ctx, pnum),
        CMD_OPENGRAVE => return on_open_grave(ctx),
        _ => {}
    }
    if v.cmd() < CMD_DLEVEL || v.cmd() > CMD_DLEVEL_END {
        snet_drop_player(ctx, pnum as i32, LEAVE_DROP as u32);
        return 0;
    }
    on_level_data(ctx, pnum, data)
}

/// `MAX_PLRS` re-exported for callers that only need msg.
pub const MAX_PLAYERS: usize = MAX_PLRS;
