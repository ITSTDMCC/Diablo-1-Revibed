//! `Source/storm/storm_net.cpp`: the network provider facade (dvlnet), and the offline provider
//! (`dvlnet/loopback.cpp`). The TCP provider is in `crate::dvlnet`; ZeroTier is not ported.
//!
//! The original guards every call with `storm_net_mutex` because the multiplayer turn thread
//! (nthread) calls in concurrently. The port's game state lives on one thread, so the facade
//! takes `&mut Ctx` instead. Each call hands the provider the game state it reads
//! ([`NetEnv`]) and afterwards runs the event handlers the provider raised.

use crate::ctx::Ctx;
use crate::multi::{GameInfo, MAX_PLRS};

pub const PS_CONNECTED: u32 = 0x10000;
pub const PS_TURN_ARRIVED: u32 = 0x20000;
pub const PS_ACTIVE: u32 = 0x40000;

pub const LEAVE_ENDING: i32 = 0x40000004;
pub const LEAVE_DROP: i32 = 0x40000006;

pub const SNPLAYER_ALL: i32 = -1;
pub const SNPLAYER_OTHERS: i32 = -2;

pub const STORM_ERROR_GAME_TERMINATED: u32 = 0x85100069;
pub const STORM_ERROR_INVALID_PLAYER: u32 = 0x8510006a;
pub const STORM_ERROR_NO_MESSAGES_WAITING: u32 = 0x8510006b;
pub const STORM_ERROR_NOT_IN_GAME: u32 = 0x85100070;

/// `conn_type`
pub use crate::enums::{SELCONN_LOOPBACK, SELCONN_TCP, SELCONN_ZT};

/// `event_type`
pub type EventType = u8;
pub use crate::enums::{EVENT_TYPE_PLAYER_CREATE_GAME, EVENT_TYPE_PLAYER_LEAVE_GAME, EVENT_TYPE_PLAYER_MESSAGE};

/// `_SNETCAPS`
#[derive(Clone, Copy, Debug, Default)]
pub struct SnetCaps {
    pub size: u32,
    pub flags: u32,
    pub maxmessagesize: u32,
    pub maxqueuesize: u32,
    pub maxplayers: u32,
    pub bytessec: u32,
    pub latencyms: u32,
    pub defaultturnssec: u32,
    pub defaultturnsintransit: u32,
}

/// `_SNETEVENT`
#[derive(Clone, Debug, Default)]
pub struct SnetEvent {
    pub eventid: u32,
    pub playerid: u32,
    pub data: Option<Vec<u8>>,
}

/// `SEVTHANDLER`
pub type SevtHandler = fn(&mut Ctx, &SnetEvent);

/// `net::abstract_net`
pub trait AbstractNet {
    /// `create`; the loopback provider sets `IsLoopback`.
    fn create(&mut self, addrstr: &str, is_loopback: &mut bool) -> i32;
    fn join(&mut self, addrstr: &str) -> i32;
    /// `SNetReceiveMessage`: the sender and the message.
    fn snet_receive_message(&mut self) -> Option<(u8, Vec<u8>)>;
    fn snet_send_message(&mut self, dest: i32, data: &[u8]) -> bool;
    /// `SNetReceiveTurns`: fills the per-player turn data, sizes and status.
    fn snet_receive_turns(&mut self, players: usize, data: &mut [Option<Vec<u8>>; MAX_PLRS], size: &mut [usize; MAX_PLRS], status: &mut [u32; MAX_PLRS]) -> bool;
    fn snet_send_turn(&mut self, data: &[u8]) -> bool;
    fn snet_get_provider_caps(&self, caps: &mut SnetCaps);
    fn snet_register_event_handler(&mut self, evtype: EventType, func: SevtHandler) -> bool;
    fn snet_unregister_event_handler(&mut self, evtype: EventType) -> bool;
    /// `SNetLeaveGame`; the loopback provider clears `IsLoopback`.
    fn snet_leave_game(&mut self, type_: i32, is_loopback: &mut bool) -> bool;
    fn snet_drop_player(&mut self, playerid: i32, flags: u32) -> bool;
    fn snet_get_owner_turns_waiting(&mut self, turns: &mut u32) -> bool;
    fn snet_get_turns_in_transit(&mut self, turns: &mut u32) -> bool;
    fn setup_gameinfo(&mut self, info: Vec<u8>);
    fn make_default_gamename(&self) -> String;
    // @port dvlnet/abstract_net.h|devilution::net::abstract_net::setup_password(std::string passwd) sha=2b69e6b12800
    fn setup_password(&mut self, _passwd: String) {}
    // @port dvlnet/abstract_net.h|devilution::net::abstract_net::clear_password() sha=afbe79f5cc01
    fn clear_password(&mut self) {}
    // @port dvlnet/abstract_net.h|devilution::net::abstract_net::send_info_request() sha=1790d5b59521
    fn send_info_request(&mut self) -> bool {
        true
    }
    // @port dvlnet/abstract_net.h|devilution::net::abstract_net::clear_gamelist() sha=a2a592f67cee
    fn clear_gamelist(&mut self) {}
    // @port dvlnet/abstract_net.h|devilution::net::abstract_net::get_gamelist() sha=9e466e346dcc
    fn get_gamelist(&mut self) -> Vec<GameInfo> {
        Vec::new()
    }
    /// The game state the provider reads (`Players.size()`, `SDL_GetTicks`, network options).
    fn set_env(&mut self, _env: NetEnv) {}
    /// Event handler calls raised during the last call (see `dvlnet::base`).
    fn take_events(&mut self) -> Vec<(SevtHandler, SnetEvent)> {
        Vec::new()
    }
    /// `SDL_GetError()` text set by `create` / `join`.
    fn take_error(&mut self) -> Option<String> {
        None
    }
}

pub use crate::dvlnet::base::NetEnv;

/// `net::loopback` (dvlnet/loopback.cpp)
pub struct Loopback {
    message_queue: std::collections::VecDeque<Vec<u8>>,
    plr_single: u8,
}

impl Loopback {
    pub fn new() -> Loopback {
        Loopback { message_queue: std::collections::VecDeque::new(), plr_single: 0 }
    }
}

impl Default for Loopback {
    fn default() -> Self {
        Loopback::new()
    }
}

impl AbstractNet for Loopback {
    // @port dvlnet/loopback.cpp|devilution::net::loopback::create(std::string) sha=794ae69b1b63
    fn create(&mut self, _addrstr: &str, is_loopback: &mut bool) -> i32 {
        *is_loopback = true;
        self.plr_single as i32
    }

    // @port dvlnet/loopback.cpp|devilution::net::loopback::join(std::string) sha=31275e2260da
    fn join(&mut self, _addrstr: &str) -> i32 {
        panic!("ABORT: loopback::join");
    }

    // @port dvlnet/loopback.cpp|devilution::net::loopback::SNetReceiveMessage(uint8_t *sender, void **data, uint32_t *size) sha=cc4e15cd0deb
    fn snet_receive_message(&mut self) -> Option<(u8, Vec<u8>)> {
        let message_last = self.message_queue.pop_front()?;
        Some((self.plr_single, message_last))
    }

    // @port dvlnet/loopback.cpp|devilution::net::loopback::SNetSendMessage(int dest, void *data, unsigned int size) sha=9cd16c394185
    fn snet_send_message(&mut self, dest: i32, data: &[u8]) -> bool {
        if dest == self.plr_single as i32 || dest == SNPLAYER_ALL {
            self.message_queue.push_back(data.to_vec());
        }
        true
    }

    // @port dvlnet/loopback.cpp|devilution::net::loopback::SNetReceiveTurns(char **data, size_t *size, uint32_t *) sha=32274953c080
    fn snet_receive_turns(&mut self, players: usize, data: &mut [Option<Vec<u8>>; MAX_PLRS], size: &mut [usize; MAX_PLRS], _status: &mut [u32; MAX_PLRS]) -> bool {
        for i in 0..players {
            size[i] = 0;
            data[i] = None;
        }
        true
    }

    // @port dvlnet/loopback.cpp|devilution::net::loopback::SNetSendTurn(char * , unsigned int) sha=3116ecda261a
    fn snet_send_turn(&mut self, _data: &[u8]) -> bool {
        true
    }

    // @port dvlnet/loopback.cpp|devilution::net::loopback::SNetGetProviderCaps(struct _SNETCAPS *caps) sha=00606950eccc
    fn snet_get_provider_caps(&self, caps: &mut SnetCaps) {
        caps.size = 0;
        caps.flags = 0;
        caps.maxmessagesize = 512;
        caps.maxqueuesize = 0;
        caps.maxplayers = MAX_PLRS as u32;
        caps.bytessec = 1000000;
        caps.latencyms = 0;
        caps.defaultturnssec = 10;
        caps.defaultturnsintransit = 1;
    }

    // @port dvlnet/loopback.cpp|devilution::net::loopback::SNetRegisterEventHandler(event_type , SEVTHANDLER) sha=b7fe4c19b933
    fn snet_register_event_handler(&mut self, _evtype: EventType, _func: SevtHandler) -> bool {
        true
    }

    // @port dvlnet/loopback.cpp|devilution::net::loopback::SNetUnregisterEventHandler(event_type) sha=a5e1b6ee9324
    fn snet_unregister_event_handler(&mut self, _evtype: EventType) -> bool {
        true
    }

    // @port dvlnet/loopback.cpp|devilution::net::loopback::SNetLeaveGame(int) sha=ead3f2d93ac4
    fn snet_leave_game(&mut self, _type: i32, is_loopback: &mut bool) -> bool {
        *is_loopback = false;
        true
    }

    // @port dvlnet/loopback.cpp|devilution::net::loopback::SNetDropPlayer(int , uint32_t) sha=dfda60f5dd8a
    fn snet_drop_player(&mut self, _playerid: i32, _flags: u32) -> bool {
        true
    }

    // @port dvlnet/loopback.cpp|devilution::net::loopback::SNetGetOwnerTurnsWaiting(uint32_t *turns) sha=771e1a52354a
    fn snet_get_owner_turns_waiting(&mut self, turns: &mut u32) -> bool {
        *turns = 0;
        true
    }

    // @port dvlnet/loopback.cpp|devilution::net::loopback::SNetGetTurnsInTransit(uint32_t *turns) sha=e87d30cb6218
    fn snet_get_turns_in_transit(&mut self, turns: &mut u32) -> bool {
        *turns = 0;
        true
    }

    // @port dvlnet/loopback.cpp|devilution::net::loopback::setup_gameinfo(buffer_t info) sha=cba11a8b64f8
    fn setup_gameinfo(&mut self, _info: Vec<u8>) {}

    // @port dvlnet/loopback.cpp|devilution::net::loopback::make_default_gamename() sha=7fefb3972147
    fn make_default_gamename(&self) -> String {
        crate::utils::language::tr("loopback")
    }
}

/// Original: `abstract_net::MakeNet` (dvlnet/abstract_net.cpp).
// @port dvlnet/abstract_net.cpp|devilution::net::abstract_net::MakeNet(provider_t provider) sha=8032e6e4b645
pub fn make_net(provider: u32) -> Box<dyn AbstractNet> {
    match provider {
        p if p == SELCONN_TCP as u32 => Box::new(crate::dvlnet::cdwrap::CdWrap::<crate::dvlnet::tcp::TcpClient>::default()),
        p if p == SELCONN_ZT as u32 => make_zerotier(),
        p if p == SELCONN_LOOPBACK as u32 => Box::new(Loopback::new()),
        _ => panic!("ABORT: MakeNet({provider})"),
    }
}

/// ZeroTier needs libzt (a whole user-space network stack); it is not ported, so the
/// connection screen does not offer it (as a DevilutionX build with `DISABLE_ZERO_TIER`).
fn make_zerotier() -> Box<dyn AbstractNet> {
    panic!("ABORT: ZeroTier is not available in this port");
}

/// Globals of storm_net.cpp.
#[derive(Default)]
pub struct StormNetState {
    /// `dvlnet_inst`
    pub dvlnet_inst: Option<Box<dyn AbstractNet>>,
    /// `GameIsPublic`
    pub game_is_public: bool,
    /// `dwLastError`
    pub dw_last_error: u32,
    /// `SDL_GetError()` text set by the network code
    pub sdl_error: String,
}

fn inst(ctx: &mut Ctx) -> &mut Box<dyn AbstractNet> {
    ctx.storm_net.dvlnet_inst.as_mut().expect("dvlnet_inst")
}

/// Calls the provider with the current game state, then runs the event handlers it raised and
/// keeps its error text (`SDL_SetError`).
fn with_net<R>(ctx: &mut Ctx, f: impl FnOnce(&mut dyn AbstractNet) -> R) -> R {
    let env = NetEnv {
        players: ctx.players.Players.len(),
        ticks: ctx.platform.ticks(),
        port: ctx.options.network.port.get() as u16,
        bind_address: ctx.options.network.sz_bind_address.clone(),
    };
    let net = inst(ctx);
    net.set_env(env);
    let r = f(net.as_mut());
    let events = net.take_events();
    let error = net.take_error();
    if let Some(e) = error {
        ctx.storm_net.sdl_error = e;
    }
    for (handler, ev) in events {
        handler(ctx, &ev);
    }
    r
}

/// `SDL_ClearError`
pub fn clear_error(ctx: &mut Ctx) {
    ctx.storm_net.sdl_error.clear();
}

/// `SDL_GetError`
pub fn get_error(ctx: &Ctx) -> String {
    ctx.storm_net.sdl_error.clone()
}

/// Original: `devilution::SErrGetLastError` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SErrGetLastError() sha=0ef342a7ef94
pub fn serr_get_last_error(ctx: &Ctx) -> u32 {
    ctx.storm_net.dw_last_error
}

/// Original: `devilution::SErrSetLastError` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SErrSetLastError(uint32_t dwErrCode) sha=e5a7901276dc
pub fn serr_set_last_error(ctx: &mut Ctx, dw_err_code: u32) {
    ctx.storm_net.dw_last_error = dw_err_code;
}

/// Original: `devilution::SNetReceiveMessage` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetReceiveMessage(uint8_t *senderplayerid, void **data, uint32_t *databytes) sha=1488ccbab2dc
pub fn snet_receive_message(ctx: &mut Ctx) -> Option<(u8, Vec<u8>)> {
    let r = with_net(ctx, |n| n.snet_receive_message());
    if r.is_none() {
        serr_set_last_error(ctx, STORM_ERROR_NO_MESSAGES_WAITING);
    }
    r
}

/// Original: `devilution::SNetSendMessage` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetSendMessage(int playerID, void *data, unsigned int databytes) sha=92c9c2d03b8d
pub fn snet_send_message(ctx: &mut Ctx, player_id: i32, data: &[u8]) -> bool {
    with_net(ctx, |n| n.snet_send_message(player_id, data))
}

/// Original: `devilution::SNetReceiveTurns` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetReceiveTurns(int arraysize, char **arraydata, size_t *arraydatabytes, uint32_t *arrayplayerstatus) sha=72502c9aba31
pub fn snet_receive_turns(ctx: &mut Ctx) -> bool {
    let players = ctx.players.Players.len();
    let mut data = std::mem::take(&mut ctx.nthread.glpMsgTbl);
    let mut size = ctx.nthread.gdwMsgLenTbl;
    let mut status = ctx.multi.player_state;
    let ok = with_net(ctx, |n| n.snet_receive_turns(players, &mut data, &mut size, &mut status));
    ctx.nthread.glpMsgTbl = data;
    ctx.nthread.gdwMsgLenTbl = size;
    ctx.multi.player_state = status;
    if !ok {
        serr_set_last_error(ctx, STORM_ERROR_NO_MESSAGES_WAITING);
        return false;
    }
    true
}

/// Original: `devilution::SNetSendTurn` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetSendTurn(char *data, unsigned int databytes) sha=682ef495b4b8
pub fn snet_send_turn(ctx: &mut Ctx, data: &[u8]) -> bool {
    with_net(ctx, |n| n.snet_send_turn(data))
}

/// Original: `devilution::SNetGetProviderCaps` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetGetProviderCaps(struct _SNETCAPS *caps) sha=bec6436fc1f0
pub fn snet_get_provider_caps(ctx: &mut Ctx, caps: &mut SnetCaps) {
    with_net(ctx, |n| n.snet_get_provider_caps(caps));
}

/// Original: `devilution::SNetUnregisterEventHandler` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetUnregisterEventHandler(event_type evtype) sha=bc6f7875c654
pub fn snet_unregister_event_handler(ctx: &mut Ctx, evtype: EventType) -> bool {
    match ctx.storm_net.dvlnet_inst.as_mut() {
        None => true,
        Some(i) => i.snet_unregister_event_handler(evtype),
    }
}

/// Original: `devilution::SNetRegisterEventHandler` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetRegisterEventHandler(event_type evtype, SEVTHANDLER func) sha=a90337ca2489
pub fn snet_register_event_handler(ctx: &mut Ctx, evtype: EventType, func: SevtHandler) -> bool {
    with_net(ctx, |n| n.snet_register_event_handler(evtype, func))
}

/// Original: `devilution::SNetDestroy` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetDestroy() sha=a15dcd049354
pub fn snet_destroy(ctx: &mut Ctx) -> bool {
    ctx.storm_net.dvlnet_inst = None;
    true
}

/// Original: `devilution::SNetDropPlayer` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetDropPlayer(int playerid, uint32_t flags) sha=3ec61c42d7ea
pub fn snet_drop_player(ctx: &mut Ctx, playerid: i32, flags: u32) -> bool {
    with_net(ctx, |n| n.snet_drop_player(playerid, flags))
}

/// Original: `devilution::SNetLeaveGame` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetLeaveGame(int type) sha=5fcf6897e3fd
pub fn snet_leave_game(ctx: &mut Ctx, type_: i32) -> bool {
    let mut is_loopback = ctx.multi.IsLoopback;
    let r = match ctx.storm_net.dvlnet_inst.is_some() {
        false => true,
        true => with_net(ctx, |n| n.snet_leave_game(type_, &mut is_loopback)),
    };
    ctx.multi.IsLoopback = is_loopback;
    r
}

/// Original: `devilution::SNetInitializeProvider` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetInitializeProvider(uint32_t provider, struct GameData *gameData) sha=9780836b2907
pub fn snet_initialize_provider(ctx: &mut Ctx, provider: u32) -> bool {
    ctx.storm_net.dvlnet_inst = Some(make_net(provider));
    (ctx.diablo.headless_mode && !crate::engine::demomode::is_running(ctx)) || crate::menu::mainmenu_select_hero_dialog(ctx)
}

/// Original: `devilution::SNetCreateGame` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetCreateGame(const char *pszGameName, const char *pszGamePassword, char *gameTemplateData, int gameTemplateSize, int *playerID) sha=bb9b6eeaedfe
pub fn snet_create_game(ctx: &mut Ctx, game_name: Option<&str>, game_password: Option<&str>, game_template: &crate::multi::GameData, player_id: &mut i32) -> bool {
    let game_init_info = game_template.to_bytes();
    with_net(ctx, |n| n.setup_gameinfo(game_init_info));

    let name = match game_name {
        Some(n) => n.to_string(),
        None => with_net(ctx, |n| n.make_default_gamename()),
    };
    ctx.multi.GameName = name.clone();
    match game_password {
        Some(p) => dvlnet_set_password(ctx, p.to_string()),
        None => dvlnet_clear_password(ctx),
    }
    let mut is_loopback = ctx.multi.IsLoopback;
    *player_id = with_net(ctx, |n| n.create(&name, &mut is_loopback));
    ctx.multi.IsLoopback = is_loopback;
    *player_id != -1
}

/// Original: `devilution::SNetJoinGame` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetJoinGame(char *pszGameName, char *pszGamePassword, int *playerID) sha=1f633993ddbb
pub fn snet_join_game(ctx: &mut Ctx, game_name: &str, game_password: Option<&str>, player_id: &mut i32) -> bool {
    ctx.multi.GameName = game_name.to_string();
    match game_password {
        Some(p) => dvlnet_set_password(ctx, p.to_string()),
        None => dvlnet_clear_password(ctx),
    }
    *player_id = with_net(ctx, |n| n.join(game_name));
    *player_id != -1
}

/// Original: `devilution::SNetGetOwnerTurnsWaiting` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetGetOwnerTurnsWaiting(uint32_t *turns) sha=c829c6681f33
pub fn snet_get_owner_turns_waiting(ctx: &mut Ctx, turns: &mut u32) -> bool {
    with_net(ctx, |n| n.snet_get_owner_turns_waiting(turns))
}

/// Original: `devilution::SNetGetTurnsInTransit` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetGetTurnsInTransit(uint32_t *turns) sha=a5d0c9ee8fa7
pub fn snet_get_turns_in_transit(ctx: &mut Ctx, turns: &mut u32) -> bool {
    with_net(ctx, |n| n.snet_get_turns_in_transit(turns))
}

/// Original: `devilution::SNetSetBasePlayer` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetSetBasePlayer(int) sha=b78333a36409
pub fn snet_set_base_player(_ctx: &mut Ctx, _unused: i32) -> bool {
    true
}

/// Original: `devilution::DvlNet_SendInfoRequest` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::DvlNet_SendInfoRequest() sha=4420a87553e9
pub fn dvlnet_send_info_request(ctx: &mut Ctx) -> bool {
    with_net(ctx, |n| n.send_info_request())
}

/// Original: `devilution::DvlNet_ClearGamelist` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::DvlNet_ClearGamelist() sha=9e4ee848f08e
pub fn dvlnet_clear_gamelist(ctx: &mut Ctx) {
    with_net(ctx, |n| n.clear_gamelist());
}

/// Original: `devilution::DvlNet_GetGamelist` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::DvlNet_GetGamelist() sha=0914e7f71d36
pub fn dvlnet_get_gamelist(ctx: &mut Ctx) -> Vec<GameInfo> {
    with_net(ctx, |n| n.get_gamelist())
}

/// Original: `devilution::DvlNet_SetPassword` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::DvlNet_SetPassword(std::string pw) sha=53fa809743c2
pub fn dvlnet_set_password(ctx: &mut Ctx, pw: String) {
    ctx.storm_net.game_is_public = false;
    ctx.multi.GamePassword = pw.clone();
    with_net(ctx, |n| n.setup_password(pw));
}

/// Original: `devilution::DvlNet_ClearPassword` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::DvlNet_ClearPassword() sha=bbb5781fb82e
pub fn dvlnet_clear_password(ctx: &mut Ctx) {
    ctx.storm_net.game_is_public = true;
    ctx.multi.GamePassword.clear();
    with_net(ctx, |n| n.clear_password());
}

/// Original: `devilution::DvlNet_IsPublicGame` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::DvlNet_IsPublicGame() sha=db34a9133df9
pub fn dvlnet_is_public_game(ctx: &Ctx) -> bool {
    ctx.storm_net.game_is_public
}
