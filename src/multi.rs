//! `Source/multi.cpp`: game sessions (single and multiplayer) and keeping them in sync.

use crate::ctx::Ctx;
use crate::engine::geometry::{Direction, Point};
use crate::enums::*;
use crate::msg::{CmdView, SIZE_TCMDPLRINFOHDR};
use crate::storm::storm_net::*;
use crate::utils::language::tr;

/// `MAX_PLRS`
pub const MAX_PLRS: usize = 4;

/// `PROJECT_VERSION_*` (config.h)
pub const PROJECT_VERSION: &str = crate::diablo::PROJECT_VERSION;
pub fn project_version() -> (u8, u8, u8) {
    (1, 5, 3)
}

/// `sizeof(TPktHdr)`
pub const PKT_HDR_SIZE: usize = 23;
/// `sizeof(TPkt::body)`
pub const PKT_BODY_SIZE: usize = 493;

/// `HeaderCheckVal` = `LoadBE16("ip")` on little-endian machines
const HeaderCheckVal: u16 = u16::from_be_bytes(*b"ip");

/// `GameData`
#[derive(Clone, Copy, Debug, Default)]
pub struct GameData {
    pub size: i32,
    pub dwSeed: u32,
    pub programid: u32,
    pub versionMajor: u8,
    pub versionMinor: u8,
    pub versionPatch: u8,
    pub nDifficulty: _difficulty,
    pub nTickRate: u8,
    pub bRunInTown: u8,
    pub bTheoQuest: u8,
    pub bCowQuest: u8,
    pub bFriendlyFire: u8,
    pub fullQuests: u8,
}

/// `sizeof(GameData)` (no packing: 4+4+4 + 9 bytes, padded to 4)
pub const GAME_DATA_SIZE: usize = 24;

impl GameData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(GAME_DATA_SIZE);
        b.extend_from_slice(&self.size.to_le_bytes());
        b.extend_from_slice(&self.dwSeed.to_le_bytes());
        b.extend_from_slice(&self.programid.to_le_bytes());
        b.extend_from_slice(&[
            self.versionMajor,
            self.versionMinor,
            self.versionPatch,
            self.nDifficulty,
            self.nTickRate,
            self.bRunInTown,
            self.bTheoQuest,
            self.bCowQuest,
            self.bFriendlyFire,
            self.fullQuests,
        ]);
        b.resize(GAME_DATA_SIZE, 0);
        b
    }

    pub fn from_bytes(raw: &[u8]) -> GameData {
        let v = CmdView(raw);
        GameData {
            size: v.i32(0),
            dwSeed: v.u32(4),
            programid: v.u32(8),
            versionMajor: v.u8(12),
            versionMinor: v.u8(13),
            versionPatch: v.u8(14),
            nDifficulty: v.u8(15),
            nTickRate: v.u8(16),
            bRunInTown: v.u8(17),
            bTheoQuest: v.u8(18),
            bCowQuest: v.u8(19),
            bFriendlyFire: v.u8(20),
            fullQuests: v.u8(21),
        }
    }
}

/// `GameInfo`
#[derive(Clone, Debug, Default)]
pub struct GameInfo {
    pub name: String,
    pub gameData: GameData,
    pub players: Vec<String>,
}

/// `TBuffer`
#[derive(Clone)]
pub struct TBuffer {
    pub dwNextWriteOffset: u32,
    pub bData: Box<[u8; 4096]>,
}

impl Default for TBuffer {
    fn default() -> Self {
        TBuffer { dwNextWriteOffset: 0, bData: Box::new([0; 4096]) }
    }
}

/// `EventTypes`
const EventTypes: [EventType; 3] = [EVENT_TYPE_PLAYER_LEAVE_GAME, EVENT_TYPE_PLAYER_CREATE_GAME, EVENT_TYPE_PLAYER_MESSAGE];

/// Globals of multi.cpp.
#[derive(Default)]
pub struct MultiState {
    pub gbSomebodyWonGameKludge: bool,
    pub highPriorityBuffer: TBuffer,
    pub sgwPackPlrOffsetTbl: [u16; MAX_PLRS],
    pub sgbPlayerTurnBitTbl: [bool; MAX_PLRS],
    pub sgbPlayerLeftGameTbl: [bool; MAX_PLRS],
    pub shareNextHighPriorityMessage: bool,
    pub gbActivePlayers: u8,
    pub gbGameDestroyed: bool,
    pub sgbSendDeltaTbl: [bool; MAX_PLRS],
    /// `sgGameInitInfo`
    pub sgGameInitInfo: GameData,
    pub gbSelectProvider: bool,
    pub sglTimeoutStart: i32,
    pub sgdwPlayerLeftReasonTbl: [i32; MAX_PLRS],
    pub lowPriorityBuffer: TBuffer,
    pub sgdwGameLoops: u32,
    pub sgbTimeout: bool,
    pub GameName: String,
    pub GamePassword: String,
    pub PublicGame: bool,
    pub gbDeltaSender: u8,
    /// `sgbNetInited`
    pub sgb_net_inited: bool,
    pub player_state: [u32; MAX_PLRS],
    pub playerInfoTimers: [u32; MAX_PLRS],
    pub IsLoopback: bool,
    sgbSentThisCycle: u32,
    /// `PackedPlayerBuffer` (static in recv_plrinfo)
    packed_player_buffer: Vec<Vec<u8>>,
}

/// Original: `BufferInit` (multi.cpp).
// @port multi.cpp|devilution::BufferInit(TBuffer *pBuf) sha=11c8fb85cfe0
fn buffer_init(buf: &mut TBuffer) {
    buf.dwNextWriteOffset = 0;
    buf.bData[0] = 0;
}

/// Original: `CopyPacket` (multi.cpp).
// @port multi.cpp|devilution::CopyPacket(TBuffer *buf, const byte *packet, size_t size) sha=12c79dfcaf34
fn copy_packet(buf: &mut TBuffer, packet: &[u8]) {
    let size = packet.len();
    if buf.dwNextWriteOffset as usize + size + 2 > 0x1000 {
        return;
    }
    let p = buf.dwNextWriteOffset as usize;
    buf.dwNextWriteOffset += size as u32 + 1;
    buf.bData[p] = size as u8;
    buf.bData[p + 1..p + 1 + size].copy_from_slice(packet);
    buf.bData[p + 1 + size] = 0;
}

/// Original: `CopyBufferedPackets` (multi.cpp). Appends to `destination`, decreasing `size`.
// @port multi.cpp|devilution::CopyBufferedPackets(byte *destination, TBuffer *source, size_t *size) sha=8c6e3e57044c
fn copy_buffered_packets(destination: &mut Vec<u8>, source: &mut TBuffer, size: &mut usize) {
    if source.dwNextWriteOffset == 0 {
        return;
    }
    let mut src = 0usize;
    loop {
        let chunk_size = source.bData[src] as usize;
        if chunk_size == 0 {
            break;
        }
        if chunk_size > *size {
            break;
        }
        src += 1;
        destination.extend_from_slice(&source.bData[src..src + chunk_size]);
        src += chunk_size;
        *size -= chunk_size;
    }
    // memcpy(source->bData, srcPtr, (source->bData - srcPtr) + dwNextWriteOffset + 1)
    let n = (source.dwNextWriteOffset as usize + 1) - src;
    source.bData.copy_within(src..src + n, 0);
    source.dwNextWriteOffset = (source.dwNextWriteOffset as i64 - src as i64) as u32;
}

/// The `TPktHdr` fields before `wCheck`, as `NetReceivePlayerData` fills them.
fn pkt_hdr_bytes(ctx: &Ctx) -> [u8; PKT_HDR_SIZE] {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let my_player = &ctx.players.Players[me];
    let mut target = my_player.get_target_position();
    if my_player._pmode == PM_SPELL && matches!(my_player.executedSpell.spellId, SpellID::Teleport | SpellID::Phasing | SpellID::Warp) {
        target = Point::default();
    }
    let mut h = [0u8; PKT_HDR_SIZE];
    h[0] = my_player.position.tile.x as u8;
    h[1] = my_player.position.tile.y as u8;
    h[2] = target.x as u8;
    h[3] = target.y as u8;
    h[4..8].copy_from_slice(&my_player._pHitPoints.to_le_bytes());
    h[8..12].copy_from_slice(&my_player._pMaxHP.to_le_bytes());
    h[12..16].copy_from_slice(&my_player._pMana.to_le_bytes());
    h[16..20].copy_from_slice(&my_player._pMaxMana.to_le_bytes());
    h[20] = my_player._pBaseStr as u8;
    h[21] = my_player._pBaseMag as u8;
    h[22] = my_player._pBaseDex as u8;
    h
}

/// `TPktHdr` is 23 bytes of fields plus wCheck and wLen; the full header is 27 bytes.
const FULL_PKT_HDR_SIZE: usize = PKT_HDR_SIZE + 4;

/// Original: `NetReceivePlayerData` (multi.cpp): the packet header with our player's state.
// @port multi.cpp|devilution::NetReceivePlayerData(TPkt *pkt) sha=09da88a5bb18
fn net_receive_player_data(ctx: &Ctx) -> Vec<u8> {
    let mut hdr = pkt_hdr_bytes(ctx).to_vec();
    hdr.extend_from_slice(&HeaderCheckVal.to_le_bytes());
    hdr.extend_from_slice(&0u16.to_le_bytes());
    hdr
}

fn set_pkt_len(pkt: &mut [u8], len: u16) {
    pkt[PKT_HDR_SIZE + 2..PKT_HDR_SIZE + 4].copy_from_slice(&len.to_le_bytes());
}

/// Original: `IsNetPlayerValid` (multi.cpp).
// @port multi.cpp|devilution::IsNetPlayerValid(const Player &player) sha=2bb8132981f4
fn is_net_player_valid(ctx: &Ctx, pnum: usize) -> bool {
    let player = &ctx.players.Players[pnum];
    player._pLevel >= 1
        && player._pLevel as i32 <= crate::player::MaxCharacterLevel
        && (player._pClass as u8) < 6
        && (player.plrlevel as usize) < crate::player::NUMLEVELS
        && crate::levels::gendung::in_dungeon_bounds(player.position.tile)
        && !player._pName.is_empty()
}

/// Original: `CheckPlayerInfoTimeouts` (multi.cpp).
// @port multi.cpp|devilution::CheckPlayerInfoTimeouts() sha=d20bc098ba0a
fn check_player_info_timeouts(ctx: &mut Ctx) {
    for i in 0..ctx.players.Players.len() {
        if ctx.players.MyPlayer == Some(i) {
            continue;
        }
        let is_player_connected = (ctx.multi.player_state[i] & PS_CONNECTED) != 0;
        let is_player_valid = is_player_connected && is_net_player_valid(ctx, i);
        if is_player_connected && !is_player_valid && ctx.multi.playerInfoTimers[i] == 0 {
            ctx.multi.playerInfoTimers[i] = ctx.platform.ticks();
        }
        if !is_player_connected || is_player_valid {
            ctx.multi.playerInfoTimers[i] = 0;
        }
        if ctx.multi.playerInfoTimers[i] == 0 {
            continue;
        }
        if ctx.platform.ticks().wrapping_sub(ctx.multi.playerInfoTimers[i]) >= 15000 {
            snet_drop_player(ctx, i as i32, LEAVE_DROP as u32);
            ctx.multi.playerInfoTimers[i] = 0;
        }
    }
}

/// Original: `SendPacket` (multi.cpp).
// @port multi.cpp|devilution::SendPacket(int playerId, const byte *packet, size_t size) sha=5884dd1d6462
fn send_packet(ctx: &mut Ctx, player_id: i32, packet: &[u8]) {
    let mut pkt = net_receive_player_data(ctx);
    let size_with_header = packet.len() + FULL_PKT_HDR_SIZE;
    set_pkt_len(&mut pkt, size_with_header as u16);
    pkt.extend_from_slice(packet);
    if !snet_send_message(ctx, player_id, &pkt) {
        crate::nthread::nthread_terminate_game(ctx, "SNetSendMessage0");
    }
}

/// Original: `MonsterSeeds` (multi.cpp).
// @port multi.cpp|devilution::MonsterSeeds() sha=c801ede3f68b
fn monster_seeds(ctx: &mut Ctx) {
    ctx.multi.sgdwGameLoops = ctx.multi.sgdwGameLoops.wrapping_add(1);
    let l = ctx.multi.sgdwGameLoops;
    let seed = (l >> 8) | (l << 24);
    for (i, m) in ctx.monster.Monsters.iter_mut().enumerate() {
        m.aiSeed = seed.wrapping_add(i as u32);
    }
}

/// Original: `HandleTurnUpperBit` (multi.cpp).
// @port multi.cpp|devilution::HandleTurnUpperBit(size_t pnum) sha=dda16ea78c44
fn handle_turn_upper_bit(ctx: &mut Ctx, pnum: usize) {
    let mut i = 0;
    while i < ctx.players.Players.len() {
        if (ctx.multi.player_state[i] & PS_CONNECTED) != 0 && i != pnum {
            break;
        }
        i += 1;
    }
    if ctx.players.MyPlayerId == i {
        ctx.multi.sgbSendDeltaTbl[pnum] = true;
    } else if pnum == ctx.players.MyPlayerId {
        ctx.multi.gbDeltaSender = i as u8;
    }
}

/// Original: `ParseTurn` (multi.cpp).
// @port multi.cpp|devilution::ParseTurn(size_t pnum, uint32_t turn) sha=e2e563f97455
fn parse_turn(ctx: &mut Ctx, pnum: usize, turn: u32) {
    if (turn & 0x80000000) != 0 {
        handle_turn_upper_bit(ctx, pnum);
    }
    let mut abs_turns = turn & 0x7FFFFFFF;
    let in_transit = ctx.nthread.gdwTurnsInTransit;
    if ctx.multi.sgbSentThisCycle < in_transit.wrapping_add(abs_turns) {
        if abs_turns >= 0x7FFFFFFF {
            abs_turns &= 0xFFFF;
        }
        ctx.multi.sgbSentThisCycle = abs_turns.wrapping_add(in_transit);
        ctx.multi.sgdwGameLoops = 4u32.wrapping_mul(abs_turns).wrapping_mul(ctx.nthread.sgbNetUpdateRate as u32);
    }
}

/// Original: `PlayerLeftMsg` (multi.cpp).
// @port multi.cpp|devilution::PlayerLeftMsg(int pnum, bool left) sha=acb8e541927e
fn player_left_msg(ctx: &mut Ctx, pnum: usize, left: bool) {
    if ctx.players.InspectPlayer == Some(pnum) {
        ctx.players.InspectPlayer = ctx.players.MyPlayer;
    }
    if ctx.players.MyPlayer == Some(pnum) {
        return;
    }
    if !ctx.players.Players[pnum].plractive {
        return;
    }
    crate::player::fix_plr_walk_tags(ctx, pnum);
    crate::portal::remove_portal_missile(ctx, pnum);
    crate::portal::deactivate_portal(ctx, pnum);
    crate::msg::delta_close_portal(ctx, pnum);
    crate::player::remove_plr_missiles(ctx, pnum);
    if left {
        let mut psz_fmt = tr("Player '{:s}' just left the game");
        match ctx.multi.sgdwPlayerLeftReasonTbl[pnum] {
            LEAVE_ENDING => {
                psz_fmt = tr("Player '{:s}' killed Diablo and left the game!");
                ctx.multi.gbSomebodyWonGameKludge = true;
            }
            LEAVE_DROP => psz_fmt = tr("Player '{:s}' dropped due to timeout"),
            _ => {}
        }
        let msg = psz_fmt.replacen("{:s}", ctx.players.Players[pnum]._pName.as_str(), 1);
        crate::plrmsg::event_plr_msg(ctx, &msg);
    }
    ctx.players.Players[pnum].plractive = false;
    ctx.players.Players[pnum]._pName.clear();
    crate::player::reset_player_gfx(ctx, pnum);
    ctx.multi.gbActivePlayers = ctx.multi.gbActivePlayers.wrapping_sub(1);
}

/// Original: `ClearPlayerLeftState` (multi.cpp).
// @port multi.cpp|devilution::ClearPlayerLeftState() sha=4e52ec8cd658
fn clear_player_left_state(ctx: &mut Ctx) {
    for i in 0..ctx.players.Players.len() {
        if ctx.multi.sgbPlayerLeftGameTbl[i] {
            if ctx.msg.gbBufferMsgs == 1 {
                let reason = ctx.multi.sgdwPlayerLeftReasonTbl[i];
                crate::msg::msg_send_drop_pkt(ctx, i as i32, reason);
            } else {
                player_left_msg(ctx, i, true);
            }
            ctx.multi.sgbPlayerLeftGameTbl[i] = false;
            ctx.multi.sgdwPlayerLeftReasonTbl[i] = 0;
        }
    }
}

/// Original: `CheckDropPlayer` (multi.cpp).
// @port multi.cpp|devilution::CheckDropPlayer() sha=f4d1837034ac
fn check_drop_player(ctx: &mut Ctx) {
    for i in 0..ctx.players.Players.len() {
        if (ctx.multi.player_state[i] & PS_ACTIVE) == 0 && (ctx.multi.player_state[i] & PS_CONNECTED) != 0 {
            snet_drop_player(ctx, i as i32, LEAVE_DROP as u32);
        }
    }
}

/// Original: `BeginTimeout` (multi.cpp).
// @port multi.cpp|devilution::BeginTimeout() sha=6360a242abfc
fn begin_timeout(ctx: &mut Ctx) {
    if !ctx.multi.sgbTimeout {
        return;
    }
    let n_ticks = ctx.platform.ticks().wrapping_sub(ctx.multi.sglTimeoutStart as u32);
    if n_ticks > 20000 {
        ctx.diablo.gb_run_game = false;
        return;
    }
    if n_ticks < 10000 {
        return;
    }
    check_drop_player(ctx);
}

/// Original: `HandleAllPackets` (multi.cpp).
// @port multi.cpp|devilution::HandleAllPackets(size_t pnum, const byte *data, size_t size) sha=34349f070480
fn handle_all_packets(ctx: &mut Ctx, pnum: usize, data: &[u8]) {
    let mut offset = 0usize;
    while offset < data.len() {
        let message_size = crate::msg::parse_cmd(ctx, pnum, &data[offset..]);
        if message_size == 0 {
            break;
        }
        offset += message_size;
    }
}

/// Original: `ProcessTmsgs` (multi.cpp).
// @port multi.cpp|devilution::ProcessTmsgs() sha=677c685bcfcb
fn process_tmsgs(ctx: &mut Ctx) {
    while let Some(msg) = crate::tmsg::tmsg_get(ctx) {
        if msg.is_empty() {
            break;
        }
        let my = ctx.players.MyPlayerId;
        handle_all_packets(ctx, my, &msg);
    }
}

/// Original: `SendPlayerInfo` (multi.cpp).
// @port multi.cpp|devilution::SendPlayerInfo(int pnum, _cmd_id cmd) sha=923ce4fddbbb
fn send_player_info(ctx: &mut Ctx, pnum: i32, cmd: _cmd_id) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let packed = crate::pack::pack_net_player(ctx, me);
    multi_send_zero_packet(ctx, pnum as usize, cmd, &packed);
}

/// Original: `SetupLocalPositions` (multi.cpp).
// @port multi.cpp|devilution::SetupLocalPositions() sha=6e57bb5d5293
fn setup_local_positions(ctx: &mut Ctx) {
    ctx.gendung.currlevel = 0;
    ctx.gendung.leveltype = crate::levels::gendung::DungeonType::Town;
    ctx.gendung.setlevel = false;
    const SPAWNS: [(i32, i32); 9] = [(75, 68), (77, 70), (75, 70), (77, 68), (76, 69), (75, 69), (76, 68), (77, 69), (76, 70)];
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let (sx, sy) = SPAWNS[ctx.players.MyPlayerId];
    let currlevel = ctx.gendung.currlevel;
    let my_player = &mut ctx.players.Players[me];
    my_player.position.tile = Point::new(sx, sy);
    my_player.position.future = my_player.position.tile;
    my_player.set_level(currlevel);
    my_player._pLvlChanging = true;
    my_player.pLvlLoad = 0;
    my_player._pmode = PM_NEWLVL;
    my_player.destAction = ACTION_NONE;
}

/// Original: `HandleEvents` (multi.cpp).
// @port multi.cpp|devilution::HandleEvents(_SNETEVENT *pEvt) sha=d216b1f97730
fn handle_events(ctx: &mut Ctx, evt: &SnetEvent) {
    match evt.eventid as EventType {
        EVENT_TYPE_PLAYER_CREATE_GAME => {
            let data = evt.data.clone().unwrap_or_default();
            let game_data = GameData::from_bytes(&data);
            if game_data.size as usize != GAME_DATA_SIZE {
                crate::appfat::app_fatal(ctx, &format!("Invalid size of game data: {}", game_data.size));
            }
            ctx.multi.sgGameInitInfo = game_data;
            ctx.multi.sgbPlayerTurnBitTbl[evt.playerid as usize] = true;
        }
        EVENT_TYPE_PLAYER_LEAVE_GAME => {
            let p = evt.playerid as usize;
            ctx.multi.sgbPlayerLeftGameTbl[p] = true;
            ctx.multi.sgbPlayerTurnBitTbl[p] = false;
            let mut left_reason = 0;
            if let Some(d) = &evt.data {
                if d.len() >= 4 {
                    left_reason = i32::from_le_bytes([d[0], d[1], d[2], d[3]]);
                }
            }
            ctx.multi.sgdwPlayerLeftReasonTbl[p] = left_reason;
            if left_reason == LEAVE_ENDING {
                ctx.multi.gbSomebodyWonGameKludge = true;
            }
            ctx.multi.sgbSendDeltaTbl[p] = false;
            if ctx.multi.gbDeltaSender as usize == p {
                ctx.multi.gbDeltaSender = MAX_PLRS as u8;
            }
        }
        EVENT_TYPE_PLAYER_MESSAGE => {
            let data = evt.data.clone().unwrap_or_default();
            let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
            let s = String::from_utf8_lossy(&data[..end]).into_owned();
            crate::plrmsg::event_plr_msg(ctx, &s);
        }
        _ => {}
    }
}

/// Original: `RegisterNetEventHandlers` (multi.cpp).
// @port multi.cpp|devilution::RegisterNetEventHandlers() sha=a63e08aaaefe
fn register_net_event_handlers(ctx: &mut Ctx) {
    for event_type in EventTypes {
        if !snet_register_event_handler(ctx, event_type, handle_events) {
            let e = get_error(ctx);
            crate::appfat::app_fatal(ctx, &format!("SNetRegisterEventHandler:\n{e}"));
        }
    }
}

/// Original: `UnregisterNetEventHandlers` (multi.cpp).
// @port multi.cpp|devilution::UnregisterNetEventHandlers() sha=ca4e457d15f3
fn unregister_net_event_handlers(ctx: &mut Ctx) {
    for event_type in EventTypes {
        snet_unregister_event_handler(ctx, event_type);
    }
}

/// Original: `InitSingle` (multi.cpp).
// @port multi.cpp|devilution::InitSingle(GameData *gameData) sha=d2924e31615d
fn init_single(ctx: &mut Ctx) -> bool {
    ctx.players.Players.resize_with(1, Default::default);
    if !snet_initialize_provider(ctx, SELCONN_LOOPBACK as u32) {
        return false;
    }
    let mut unused = 0;
    let data = ctx.multi.sgGameInitInfo;
    if !snet_create_game(ctx, Some("local"), Some("local"), &data, &mut unused) {
        let e = get_error(ctx);
        crate::appfat::app_fatal(ctx, &format!("SNetCreateGame1:\n{e}"));
    }
    ctx.players.MyPlayerId = 0;
    ctx.players.MyPlayer = Some(0);
    ctx.players.InspectPlayer = ctx.players.MyPlayer;
    ctx.init.gb_is_multiplayer = false;
    let save_number = ctx.menu.g_save_number;
    crate::pfile::pfile_read_player_from_save(ctx, save_number, 0);
    true
}

/// Original: `InitMulti` (multi.cpp).
// @port multi.cpp|devilution::InitMulti(GameData *gameData) sha=adbd765fc444
fn init_multi(ctx: &mut Ctx) -> bool {
    ctx.players.Players.resize_with(MAX_PLRS, Default::default);
    let mut player_id = 0i32;
    loop {
        if ctx.multi.gbSelectProvider && !crate::diablo_ui::selconn::ui_select_provider(ctx) {
            return false;
        }
        register_net_event_handlers(ctx);
        if crate::diablo_ui::selgame::ui_select_game(ctx, &mut player_id) {
            break;
        }
        ctx.multi.gbSelectProvider = true;
    }
    if player_id as usize >= ctx.players.Players.len() {
        return false;
    }
    ctx.players.MyPlayerId = player_id as usize;
    ctx.players.MyPlayer = Some(player_id as usize);
    ctx.players.InspectPlayer = ctx.players.MyPlayer;
    ctx.init.gb_is_multiplayer = true;
    let save_number = ctx.menu.g_save_number;
    crate::pfile::pfile_read_player_from_save(ctx, save_number, player_id as usize);
    true
}

/// Original: `devilution::InitGameInfo` (multi.cpp).
// @port multi.cpp|devilution::InitGameInfo() sha=e05908a997cf
pub fn init_game_info(ctx: &mut Ctx) {
    let now = ctx.platform.time();
    let game_id = crate::diablo_ui::selgame::game_id(ctx);
    let g = &mut ctx.multi.sgGameInitInfo;
    g.size = GAME_DATA_SIZE as i32;
    g.dwSeed = now as u32;
    g.programid = game_id;
    let (major, minor, patch) = project_version();
    g.versionMajor = major;
    g.versionMinor = minor;
    g.versionPatch = patch;
    g.nTickRate = ctx.options.gameplay.tick_rate.get() as u8;
    g.bRunInTown = ctx.options.gameplay.run_in_town.get() as u8;
    g.bTheoQuest = ctx.options.gameplay.theo_quest.get() as u8;
    g.bCowQuest = ctx.options.gameplay.cow_quest.get() as u8;
    g.bFriendlyFire = ctx.options.gameplay.friendly_fire.get() as u8;
    g.fullQuests = (!ctx.init.gb_is_multiplayer || ctx.options.gameplay.multiplayer_full_quests.get()) as u8;
}

/// Original: `devilution::NetSendLoPri` (multi.cpp).
// @port multi.cpp|devilution::NetSendLoPri(int playerId, const byte *data, size_t size) sha=60eee19e29bf
pub fn net_send_lo_pri(ctx: &mut Ctx, player_id: i32, data: &[u8]) {
    if !data.is_empty() {
        copy_packet(&mut ctx.multi.lowPriorityBuffer, data);
        send_packet(ctx, player_id, data);
    }
}

/// Original: `devilution::NetSendHiPri` (multi.cpp).
// @port multi.cpp|devilution::NetSendHiPri(int playerId, const byte *data, size_t size) sha=0c8526bdac5f
pub fn net_send_hi_pri(ctx: &mut Ctx, player_id: i32, data: &[u8]) {
    if !data.is_empty() {
        copy_packet(&mut ctx.multi.highPriorityBuffer, data);
        send_packet(ctx, player_id, data);
    }
    if ctx.multi.shareNextHighPriorityMessage {
        ctx.multi.shareNextHighPriorityMessage = false;
        let mut pkt = net_receive_player_data(ctx);
        let normal = ctx.nthread.gdwNormalMsgSize as usize;
        let mut remaining_space = normal - FULL_PKT_HDR_SIZE;
        let mut body = Vec::new();
        let mut hi = std::mem::take(&mut ctx.multi.highPriorityBuffer);
        copy_buffered_packets(&mut body, &mut hi, &mut remaining_space);
        ctx.multi.highPriorityBuffer = hi;
        let mut lo = std::mem::take(&mut ctx.multi.lowPriorityBuffer);
        copy_buffered_packets(&mut body, &mut lo, &mut remaining_space);
        ctx.multi.lowPriorityBuffer = lo;
        let mut sync_buf = vec![0u8; remaining_space];
        let left = crate::sync::sync_all_monsters(ctx, &mut sync_buf, remaining_space as u32) as usize;
        body.extend_from_slice(&sync_buf[..remaining_space - left]);
        remaining_space = left;
        let len = normal - remaining_space;
        set_pkt_len(&mut pkt, len as u16);
        pkt.extend_from_slice(&body);
        pkt.truncate(len);
        if !snet_send_message(ctx, SNPLAYER_OTHERS, &pkt) {
            crate::nthread::nthread_terminate_game(ctx, "SNetSendMessage");
        }
    }
}

/// Original: `devilution::multi_send_msg_packet` (multi.cpp).
// @port multi.cpp|devilution::multi_send_msg_packet(uint32_t pmask, const byte *data, size_t size) sha=0bd197b37033
pub fn multi_send_msg_packet(ctx: &mut Ctx, pmask: u32, data: &[u8]) {
    let mut pkt = net_receive_player_data(ctx);
    let len = data.len() + FULL_PKT_HDR_SIZE;
    set_pkt_len(&mut pkt, len as u16);
    pkt.extend_from_slice(data);
    let mut player_id = 0usize;
    let mut v: u32 = 1;
    while player_id < ctx.players.Players.len() {
        if (v & pmask) != 0 && !snet_send_message(ctx, player_id as i32, &pkt) && serr_get_last_error(ctx) != STORM_ERROR_INVALID_PLAYER {
            crate::nthread::nthread_terminate_game(ctx, "SNetSendMessage");
            return;
        }
        player_id += 1;
        v <<= 1;
    }
}

/// Original: `devilution::multi_msg_countdown` (multi.cpp).
// @port multi.cpp|devilution::multi_msg_countdown() sha=d28b2b6c875b
pub fn multi_msg_countdown(ctx: &mut Ctx) {
    for i in 0..ctx.players.Players.len() {
        if (ctx.multi.player_state[i] & PS_TURN_ARRIVED) != 0 && ctx.nthread.gdwMsgLenTbl[i] == 4 {
            let turn = ctx.nthread.glpMsgTbl[i].as_ref().map(|d| CmdView(d).u32(0)).unwrap_or(0);
            parse_turn(ctx, i, turn);
        }
    }
}

/// Original: `devilution::multi_player_left` (multi.cpp).
// @port multi.cpp|devilution::multi_player_left(int pnum, int reason) sha=3cdb796d64b4
pub fn multi_player_left(ctx: &mut Ctx, pnum: usize, reason: i32) {
    ctx.multi.sgbPlayerLeftGameTbl[pnum] = true;
    ctx.multi.sgdwPlayerLeftReasonTbl[pnum] = reason;
    clear_player_left_state(ctx);
}

/// Original: `devilution::multi_net_ping` (multi.cpp).
// @port multi.cpp|devilution::multi_net_ping() sha=1f9d4ae833be
pub fn multi_net_ping(ctx: &mut Ctx) {
    ctx.multi.sgbTimeout = true;
    ctx.multi.sglTimeoutStart = ctx.platform.ticks() as i32;
}

/// Original: `devilution::multi_handle_delta` (multi.cpp).
// @port multi.cpp|devilution::multi_handle_delta() sha=67d34a24a4e5
pub fn multi_handle_delta(ctx: &mut Ctx) -> bool {
    if ctx.multi.gbGameDestroyed {
        ctx.diablo.gb_run_game = false;
        return false;
    }
    for i in 0..ctx.players.Players.len() {
        if ctx.multi.sgbSendDeltaTbl[i] {
            ctx.multi.sgbSendDeltaTbl[i] = false;
            crate::msg::delta_export_data(ctx, i as i32);
        }
    }
    let sent = ctx.multi.sgbSentThisCycle;
    ctx.multi.sgbSentThisCycle = crate::nthread::nthread_send_and_recv_turn(ctx, sent, 1);
    let mut received = false;
    if !crate::nthread::nthread_recv_turns(ctx, Some(&mut received)) {
        begin_timeout(ctx);
        return false;
    }
    ctx.multi.sgbTimeout = false;
    if received {
        let my = ctx.players.MyPlayerId as i32;
        if !ctx.multi.shareNextHighPriorityMessage {
            ctx.multi.shareNextHighPriorityMessage = true;
            if ctx.multi.highPriorityBuffer.dwNextWriteOffset != 0 {
                net_send_hi_pri(ctx, my, &[]);
            }
        } else {
            net_send_hi_pri(ctx, my, &[]);
            ctx.multi.shareNextHighPriorityMessage = true;
        }
    }
    monster_seeds(ctx);
    true
}

/// Original: `devilution::multi_process_network_packets` (multi.cpp).
// @port multi.cpp|devilution::multi_process_network_packets() sha=0dd4b5d29d30
pub fn multi_process_network_packets(ctx: &mut Ctx) {
    clear_player_left_state(ctx);
    process_tmsgs(ctx);
    while let Some((player_id, pkt)) = snet_receive_message(ctx) {
        ctx.msg.dwRecCount += 1;
        clear_player_left_state(ctx);
        let dw_msg_size = pkt.len();
        if dw_msg_size < FULL_PKT_HDR_SIZE {
            continue;
        }
        let pid = player_id as usize;
        if pid >= ctx.players.Players.len() {
            continue;
        }
        let v = CmdView(&pkt);
        if v.u16(PKT_HDR_SIZE) != HeaderCheckVal {
            continue;
        }
        if v.u16(PKT_HDR_SIZE + 2) as usize != dw_msg_size {
            continue;
        }
        if !is_net_player_valid(ctx, pid) {
            let cmd = v.u8(FULL_PKT_HDR_SIZE);
            if ctx.msg.gbBufferMsgs == 0 && cmd != CMD_SEND_PLRINFO && cmd != CMD_ACK_PLRINFO {
                continue;
            }
        }
        let sync_position = Point::new(v.u8(0) as i32, v.u8(1) as i32);
        ctx.players.Players[pid].position.last = sync_position;
        if ctx.players.MyPlayer != Some(pid) {
            assert!(ctx.msg.gbBufferMsgs != 2);
            {
                let p = &mut ctx.players.Players[pid];
                p._pHitPoints = v.i32(4);
                p._pMaxHP = v.i32(8);
                p._pMana = v.i32(12);
                p._pMaxMana = v.i32(16);
                p._pBaseStr = v.u8(20) as i32;
                p._pBaseMag = v.u8(21) as i32;
                p._pBaseDex = v.u8(22) as i32;
            }
            let cond = ctx.msg.gbBufferMsgs == 1;
            if !cond && ctx.players.Players[pid].plractive && ctx.players.Players[pid]._pHitPoints != 0 {
                if crate::player::is_on_active_level(ctx, pid) && !ctx.players.Players[pid]._pLvlChanging {
                    if ctx.players.Players[pid].position.tile.walking_distance(sync_position) > 3 && crate::player::pos_ok_player(ctx, pid, sync_position) {
                        crate::player::fix_plr_walk_tags(ctx, pid);
                        let t = ctx.players.Players[pid].position.tile;
                        ctx.players.Players[pid].position.old = t;
                        crate::player::fix_plr_walk_tags(ctx, pid);
                        let walking = ctx.players.Players[pid].is_walking();
                        let p = &mut ctx.players.Players[pid];
                        p.position.tile = sync_position;
                        p.position.future = sync_position;
                        if walking {
                            p.position.temp = sync_position;
                        }
                        crate::player::set_player_old(ctx, pid);
                        let t = ctx.players.Players[pid].position.tile;
                        ctx.gendung.dPlayer[t.x as usize][t.y as usize] = player_id as i8 + 1;
                    }
                    let p = &mut ctx.players.Players[pid];
                    if p.position.future.walking_distance(p.position.tile) > 1 {
                        p.position.future = p.position.tile;
                    }
                    let target = Point::new(v.u8(2) as i32, v.u8(3) as i32);
                    if target != Point::default() {
                        crate::player::make_plr_path(ctx, pid, target, true);
                    }
                } else {
                    let p = &mut ctx.players.Players[pid];
                    p.position.tile = sync_position;
                    p.position.future = sync_position;
                    crate::player::set_player_old(ctx, pid);
                }
            }
        }
        handle_all_packets(ctx, pid, &pkt[FULL_PKT_HDR_SIZE..]);
    }
    if serr_get_last_error(ctx) != STORM_ERROR_NO_MESSAGES_WAITING {
        crate::nthread::nthread_terminate_game(ctx, "SNetReceiveMsg");
    }
    check_player_info_timeouts(ctx);
}

/// Original: `devilution::multi_send_zero_packet` (multi.cpp).
// @port multi.cpp|devilution::multi_send_zero_packet(size_t pnum, _cmd_id bCmd, const byte *data, size_t size) sha=6a535ad70f7b
pub fn multi_send_zero_packet(ctx: &mut Ctx, pnum: usize, b_cmd: _cmd_id, data: &[u8]) {
    assert!(pnum != ctx.players.MyPlayerId);
    let size = data.len();
    assert!(size <= 0x0ffff);
    let mut offset = 0usize;
    while offset < size {
        let mut pkt = vec![0u8; FULL_PKT_HDR_SIZE];
        pkt[PKT_HDR_SIZE..PKT_HDR_SIZE + 2].copy_from_slice(&HeaderCheckVal.to_le_bytes());
        let mut dw_body = ctx.nthread.gdwLargestMsgSize as usize - FULL_PKT_HDR_SIZE - SIZE_TCMDPLRINFOHDR;
        dw_body = dw_body.min(size - offset);
        assert!(dw_body <= 0x0ffff);
        pkt.push(b_cmd);
        pkt.extend_from_slice(&(offset as u16).to_le_bytes());
        pkt.extend_from_slice(&(dw_body as u16).to_le_bytes());
        pkt.extend_from_slice(&data[offset..offset + dw_body]);
        let dw_msg = FULL_PKT_HDR_SIZE + SIZE_TCMDPLRINFOHDR + dw_body;
        set_pkt_len(&mut pkt, dw_msg as u16);
        // `size_t pnum` holds SNPLAYER_OTHERS (-2) for broadcasts
        if !snet_send_message(ctx, pnum as i32, &pkt) {
            crate::nthread::nthread_terminate_game(ctx, "SNetSendMessage2");
            return;
        }
        offset += dw_body;
    }
}

/// Original: `devilution::NetClose` (multi.cpp).
// @port multi.cpp|devilution::NetClose() sha=83e70aeb87f9
pub fn net_close(ctx: &mut Ctx) {
    if !ctx.multi.sgb_net_inited {
        return;
    }
    ctx.multi.sgb_net_inited = false;
    crate::nthread::nthread_cleanup(ctx);
    crate::tmsg::tmsg_cleanup(ctx);
    unregister_net_event_handlers(ctx);
    snet_leave_game(ctx, 3);
    if ctx.init.gb_is_multiplayer {
        ctx.platform.delay(2000);
    }
    if !crate::engine::demomode::is_running(ctx) {
        crate::player::clear_players(ctx);
        ctx.players.MyPlayer = None;
    }
}

/// Original: `devilution::NetInit` (multi.cpp).
// @port multi.cpp|devilution::NetInit(bool bSinglePlayer) sha=9757e8b5984f
pub fn net_init(ctx: &mut Ctx, b_single_player: bool) -> bool {
    loop {
        ctx.rng.set_rnd_seed(0);
        init_game_info(ctx);
        ctx.multi.sgbPlayerTurnBitTbl = [false; MAX_PLRS];
        ctx.multi.gbGameDestroyed = false;
        ctx.multi.sgbPlayerLeftGameTbl = [false; MAX_PLRS];
        ctx.multi.sgdwPlayerLeftReasonTbl = [0; MAX_PLRS];
        ctx.multi.sgbSendDeltaTbl = [false; MAX_PLRS];
        crate::player::clear_players(ctx);
        ctx.players.MyPlayer = None;
        ctx.multi.sgwPackPlrOffsetTbl = [0; MAX_PLRS];
        snet_set_base_player(ctx, 0);
        if b_single_player {
            if !init_single(ctx) {
                return false;
            }
        } else if !init_multi(ctx) {
            return false;
        }
        ctx.multi.sgb_net_inited = true;
        ctx.multi.sgbTimeout = false;
        crate::msg::delta_init(ctx);
        crate::plrmsg::init_plr_msg(ctx);
        buffer_init(&mut ctx.multi.highPriorityBuffer);
        buffer_init(&mut ctx.multi.lowPriorityBuffer);
        ctx.multi.shareNextHighPriorityMessage = true;
        crate::sync::sync_init(ctx);
        let my = ctx.players.MyPlayerId;
        let bit = ctx.multi.sgbPlayerTurnBitTbl[my];
        crate::nthread::nthread_start(ctx, bit);
        crate::tmsg::tmsg_start(ctx);
        ctx.multi.sgdwGameLoops = 0;
        ctx.multi.sgbSentThisCycle = 0;
        ctx.multi.gbDeltaSender = my as u8;
        ctx.multi.gbSomebodyWonGameKludge = false;
        crate::nthread::nthread_send_and_recv_turn(ctx, 0, 0);
        setup_local_positions(ctx);
        send_player_info(ctx, SNPLAYER_OTHERS, CMD_SEND_PLRINFO);

        let me = ctx.players.MyPlayer.expect("MyPlayer");
        crate::player::reset_player_gfx(ctx, me);
        ctx.players.Players[me].plractive = true;
        ctx.multi.gbActivePlayers = 1;

        if !ctx.multi.sgbPlayerTurnBitTbl[my] || crate::msg::msg_wait_resync(ctx) {
            break;
        }
        net_close(ctx);
        ctx.multi.gbSelectProvider = false;
    }
    let seed = ctx.multi.sgGameInitInfo.dwSeed;
    ctx.rng.set_rnd_seed(seed as u32);
    ctx.diablo.gn_tick_delay = (1000 / ctx.multi.sgGameInitInfo.nTickRate as u32) as u16;

    for i in 0..crate::player::NUMLEVELS {
        ctx.diablo.glSeedTbl[i] = ctx.rng.advance_rnd_seed() as u32;
    }
    ctx.multi.PublicGame = dvlnet_is_public_game(ctx);

    let me = ctx.players.MyPlayer.expect("MyPlayer");
    crate::qol::chatlog::add_message_to_chat_log(ctx, &tr("New Game"), None, crate::engine::render::text_render::UiFlags::COLOR_RED);
    let p = &ctx.players.Players[me];
    let msg = tr("Player '{:s}' (level {:d}) just joined the game").replacen("{:s}", p._pName.as_str(), 1).replacen("{:d}", &p._pLevel.to_string(), 1);
    crate::qol::chatlog::add_message_to_chat_log(ctx, &msg, None, crate::engine::render::text_render::UiFlags::COLOR_WHITE);
    true
}

/// Original: `devilution::recv_plrinfo` (multi.cpp).
// @port multi.cpp|devilution::recv_plrinfo(int pnum, const TCmdPlrInfoHdr &header, bool recv) sha=448d166e6fda
pub fn recv_plrinfo(ctx: &mut Ctx, pnum: usize, header: CmdView, recv: bool) {
    assert!(pnum < MAX_PLRS);
    if ctx.players.MyPlayer == Some(pnum) {
        return;
    }
    if ctx.multi.packed_player_buffer.len() < MAX_PLRS {
        ctx.multi.packed_player_buffer = vec![vec![0u8; crate::pack::PLAYER_NET_PACK_SIZE]; MAX_PLRS];
    }
    let w_offset = header.u16(1);
    let w_bytes = header.u16(3);
    if ctx.multi.sgwPackPlrOffsetTbl[pnum] != w_offset {
        ctx.multi.sgwPackPlrOffsetTbl[pnum] = 0;
        if w_offset != 0 {
            return;
        }
    }
    if !recv && ctx.multi.sgwPackPlrOffsetTbl[pnum] == 0 {
        send_player_info(ctx, pnum as i32, CMD_ACK_PLRINFO);
    }
    let body = header.bytes(SIZE_TCMDPLRINFOHDR, w_bytes as usize);
    let o = w_offset as usize;
    let buf = &mut ctx.multi.packed_player_buffer[pnum];
    let end = (o + body.len()).min(buf.len());
    buf[o..end].copy_from_slice(&body[..end - o]);
    ctx.multi.sgwPackPlrOffsetTbl[pnum] = ctx.multi.sgwPackPlrOffsetTbl[pnum].wrapping_add(w_bytes);
    if ctx.multi.sgwPackPlrOffsetTbl[pnum] as usize != crate::pack::PLAYER_NET_PACK_SIZE {
        return;
    }
    ctx.multi.sgwPackPlrOffsetTbl[pnum] = 0;

    player_left_msg(ctx, pnum, false);
    let packed = ctx.multi.packed_player_buffer[pnum].clone();
    if !crate::pack::unpack_net_player(ctx, &packed, pnum) {
        ctx.players.Players[pnum] = Default::default();
        snet_drop_player(ctx, pnum as i32, LEAVE_DROP as u32);
        return;
    }
    if !recv {
        return;
    }
    crate::player::reset_player_gfx(ctx, pnum);
    ctx.players.Players[pnum].plractive = true;
    ctx.multi.gbActivePlayers += 1;

    let sz_event = if ctx.multi.sgbPlayerTurnBitTbl[pnum] {
        tr("Player '{:s}' (level {:d}) just joined the game")
    } else {
        tr("Player '{:s}' (level {:d}) is already in the game")
    };
    let p = &ctx.players.Players[pnum];
    let msg = sz_event.replacen("{:s}", p._pName.as_str(), 1).replacen("{:d}", &p._pLevel.to_string(), 1);
    crate::plrmsg::event_plr_msg(ctx, &msg);

    crate::player::sync_init_plr(ctx, pnum);
    if !crate::player::is_on_active_level(ctx, pnum) {
        return;
    }
    if ctx.players.Players[pnum]._pHitPoints >> 6 > 0 {
        crate::player::start_stand(ctx, pnum, Direction::South);
        return;
    }
    ctx.players.Players[pnum]._pgfxnum &= !0xF;
    ctx.players.Players[pnum]._pmode = PM_DEATH;
    crate::player::new_plr_anim(ctx, pnum, player_graphic::Death, Direction::South, AnimationDistributionFlags::None, 0, 0);
    let p = &mut ctx.players.Players[pnum];
    p.AnimInfo.currentFrame = p.AnimInfo.numberOfFrames - 2;
    let t = p.position.tile;
    ctx.gendung.dFlags[t.x as usize][t.y as usize] |= DungeonFlag::DeadPlayer;
}
