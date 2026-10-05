//! `Source/dvlnet/base.cpp`: the turn and message bookkeeping shared by the network providers.
//! `base` is a class the TCP client derives from; in the port the shared state is
//! [`BaseState`] and the shared methods are the provided methods of [`Base`], which a provider
//! implements by supplying `poll`, `send` and `is_game_host`.
//!
//! The original calls the game's event handlers from inside `poll()`. The port queues them
//! ([`BaseState::events`]) and `storm_net` runs them when the provider call returns, because the
//! handlers need the game state the provider is part of.

use std::collections::{HashMap, VecDeque};

use super::packet::*;
use crate::multi::MAX_PLRS;
use crate::storm::storm_net::{EventType, SevtHandler, SnetCaps, SnetEvent, EVENT_TYPE_PLAYER_CREATE_GAME, EVENT_TYPE_PLAYER_LEAVE_GAME};
use crate::storm::storm_net::{PS_ACTIVE, PS_CONNECTED, PS_TURN_ARRIVED, SNPLAYER_ALL, SNPLAYER_OTHERS};

/// `base::message_t`
#[derive(Clone, Debug)]
pub struct Message {
    pub sender: u8,
    pub payload: Vec<u8>,
}

impl Message {
    /// Original: `base::message_t::message_t(int s, buffer_t p)` (dvlnet/base.h).
    // @port dvlnet/base.h|devilution::net::base::message_t::message_t(int s, buffer_t p)
    pub fn new(s: u8, p: Vec<u8>) -> Message {
        Message { sender: s, payload: p }
    }
}

impl Default for Message {
    /// Original: `base::message_t::message_t()` (dvlnet/base.h).
    // @port dvlnet/base.h|devilution::net::base::message_t::message_t()
    fn default() -> Self {
        Message { sender: 0xFF, payload: Vec::new() }
    }
}

/// `base::PlayerState`
#[derive(Clone, Debug, Default)]
struct PlayerState {
    is_connected: bool,
    turn_queue: VecDeque<Turn>,
    last_turn_value: i32,
    round_trip_latency: u32,
}

/// The game state the original's network code reads directly (`Players.size()`,
/// `SDL_GetTicks()`, `sgOptions.Network`), handed in by `storm_net` before each call.
#[derive(Clone, Debug, Default)]
pub struct NetEnv {
    pub players: usize,
    pub ticks: u32,
    pub port: u16,
    pub bind_address: String,
}

/// The data members of `base`.
pub struct BaseState {
    registered_handlers: HashMap<EventType, SevtHandler>,
    pub game_init_info: Vec<u8>,
    current_turn: SeqT,
    next_turn: SeqT,
    message_queue: VecDeque<Message>,
    pub plr_self: PlrT,
    pub cookie_self: CookieT,
    pub pktfty: PacketFactory,
    player_state_table: [PlayerState; MAX_PLRS],
    awaiting_sequence_number: bool,
    /// Event handler calls made while polling, run by `storm_net` after the call.
    pub events: Vec<(SevtHandler, SnetEvent)>,
    /// `SDL_SetError` text from `create` / `join`
    pub error: Option<String>,
    pub env: NetEnv,
}

impl Default for BaseState {
    fn default() -> Self {
        BaseState {
            registered_handlers: HashMap::new(),
            game_init_info: Vec::new(),
            current_turn: 0,
            next_turn: 0,
            message_queue: VecDeque::new(),
            plr_self: PLR_BROADCAST,
            cookie_self: 0,
            pktfty: PacketFactory::new(),
            player_state_table: Default::default(),
            awaiting_sequence_number: true,
            events: Vec::new(),
            error: None,
            env: NetEnv::default(),
        }
    }
}

/// `base`: the provided methods are base.cpp; a provider supplies the pure virtual ones.
pub trait Base {
    fn base(&mut self) -> &mut BaseState;
    fn base_ref(&self) -> &BaseState;
    fn poll(&mut self);
    fn send(&mut self, pkt: &Packet);
    fn is_game_host(&self) -> bool;

    /// Original: `base::DisconnectNet` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::DisconnectNet(plr_t plr)
    fn disconnect_net(&mut self, _plr: PlrT) {}

    /// Original: `base::setup_gameinfo` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::setup_gameinfo(buffer_t info)
    fn base_setup_gameinfo(&mut self, info: Vec<u8>) {
        self.base().game_init_info = info;
    }

    /// Original: `base::setup_password` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::setup_password(std::string pw)
    fn base_setup_password(&mut self, pw: &str) {
        self.base().pktfty = PacketFactory::with_password(pw);
    }

    /// Original: `base::clear_password` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::clear_password()
    fn base_clear_password(&mut self) {
        self.base().pktfty = PacketFactory::new();
    }

    /// Original: `base::RunEventHandler` (dvlnet/base.cpp): queued, see the module comment.
    // @port dvlnet/base.cpp|devilution::net::base::RunEventHandler(_SNETEVENT &ev)
    fn run_event_handler(&mut self, ev: SnetEvent) {
        let b = self.base();
        if let Some(&f) = b.registered_handlers.get(&(ev.eventid as EventType)) {
            b.events.push((f, ev));
        }
    }

    /// Original: `base::SendEchoRequest` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::SendEchoRequest(plr_t player)
    fn send_echo_request(&mut self, player: PlrT) {
        let b = self.base();
        if b.plr_self == PLR_BROADCAST || player == b.plr_self {
            return;
        }
        let now = b.env.ticks;
        let echo = b.pktfty.echo_request(b.plr_self, player, now);
        self.send(&echo);
    }

    /// Original: `base::HandleAccept` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::HandleAccept(packet &pkt)
    fn handle_accept(&mut self, pkt: &Packet) {
        if self.base().plr_self != PLR_BROADCAST {
            return; // already have player id
        }
        if pkt.cookie() == self.base().cookie_self {
            let me = pkt.new_player();
            self.base().plr_self = me;
            self.connect(me);
        }
        if self.base().game_init_info != pkt.info() {
            if pkt.info().len() != crate::multi::GAME_DATA_SIZE {
                panic!("ABORT: game info of the wrong size");
            }
            // we joined and did not create
            self.base().game_init_info = pkt.info().to_vec();
            let ev = SnetEvent { eventid: EVENT_TYPE_PLAYER_CREATE_GAME as u32, playerid: self.base().plr_self as u32, data: Some(pkt.info().to_vec()) };
            self.run_event_handler(ev);
        }
    }

    /// Original: `base::HandleConnect` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::HandleConnect(packet &pkt)
    fn handle_connect(&mut self, pkt: &Packet) {
        let new_player = pkt.new_player();
        self.connect(new_player);
    }

    /// Original: `base::HandleTurn` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::HandleTurn(packet &pkt)
    fn handle_turn(&mut self, pkt: &Packet) {
        let src = pkt.source() as usize;
        let turn = pkt.turn();
        self.base().player_state_table[src].turn_queue.push_back(turn);
        self.make_ready(turn.sequence_number);
    }

    /// Original: `base::HandleDisconnect` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::HandleDisconnect(packet &pkt)
    fn handle_disconnect(&mut self, pkt: &Packet) {
        let new_player = pkt.new_player();
        if new_player != self.base().plr_self {
            if self.is_connected(new_player) {
                let leaveinfo = pkt.leave_info();
                let ev = SnetEvent { eventid: EVENT_TYPE_PLAYER_LEAVE_GAME as u32, playerid: new_player as u32, data: Some(leaveinfo.to_le_bytes().to_vec()) };
                self.run_event_handler(ev);
                self.disconnect_net(new_player);
                self.clear_msg(new_player);
                let ps = &mut self.base().player_state_table[new_player as usize];
                ps.is_connected = false;
                ps.turn_queue.clear();
            }
        } else {
            panic!("ABORT: we were dropped by the owner");
        }
    }

    /// Original: `base::HandleEchoRequest` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::HandleEchoRequest(packet &pkt)
    fn handle_echo_request(&mut self, pkt: &Packet) {
        let b = self.base();
        let reply = b.pktfty.echo_reply(b.plr_self, pkt.source(), pkt.time());
        self.send(&reply);
    }

    /// Original: `base::HandleEchoReply` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::HandleEchoReply(packet &pkt)
    fn handle_echo_reply(&mut self, pkt: &Packet) {
        let b = self.base();
        let now = b.env.ticks;
        b.player_state_table[pkt.source() as usize].round_trip_latency = now.wrapping_sub(pkt.time());
    }

    /// Original: `base::ClearMsg` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::ClearMsg(plr_t plr)
    fn clear_msg(&mut self, plr: PlrT) {
        self.base().message_queue.retain(|msg| msg.sender != plr);
    }

    /// Original: `base::Connect` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::Connect(plr_t player)
    fn connect(&mut self, player: PlrT) {
        let ps = &mut self.base().player_state_table[player as usize];
        let was_connected = ps.is_connected;
        ps.is_connected = true;
        if !was_connected {
            self.send_first_turn_if_ready(player);
        }
    }

    /// Original: `base::IsConnected` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::IsConnected(plr_t player)
    fn is_connected(&self, player: PlrT) -> bool {
        self.base_ref().player_state_table[player as usize].is_connected
    }

    /// Original: `base::RecvLocal` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::RecvLocal(packet &pkt)
    fn recv_local(&mut self, pkt: &Packet) {
        if (pkt.source() as usize) < MAX_PLRS {
            self.connect(pkt.source());
        }
        match pkt.type_() {
            PT_MESSAGE => {
                let m = Message::new(pkt.source(), pkt.message().to_vec());
                self.base().message_queue.push_back(m);
            }
            PT_TURN => self.handle_turn(pkt),
            PT_JOIN_ACCEPT => self.handle_accept(pkt),
            PT_CONNECT => self.handle_connect(pkt),
            PT_DISCONNECT => self.handle_disconnect(pkt),
            PT_ECHO_REQUEST => self.handle_echo_request(pkt),
            PT_ECHO_REPLY => self.handle_echo_reply(pkt),
            _ => {} // otherwise drop
        }
    }

    /// Original: `base::SNetReceiveMessage` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::SNetReceiveMessage(uint8_t *sender, void **data, uint32_t *size)
    fn base_snet_receive_message(&mut self) -> Option<(u8, Vec<u8>)> {
        self.poll();
        let m = self.base().message_queue.pop_front()?;
        Some((m.sender, m.payload))
    }

    /// Original: `base::SNetSendMessage` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::SNetSendMessage(int playerId, void *data, unsigned int size)
    fn base_snet_send_message(&mut self, player_id: i32, data: &[u8]) -> bool {
        if player_id != SNPLAYER_ALL && player_id != SNPLAYER_OTHERS && !(0..MAX_PLRS as i32).contains(&player_id) {
            panic!("abort: SNetSendMessage to player {player_id}");
        }
        let message = data.to_vec();
        let plr_self = self.base().plr_self;
        if player_id == plr_self as i32 || player_id == SNPLAYER_ALL {
            self.base().message_queue.push_back(Message::new(plr_self, message.clone()));
        }
        let dest = if player_id == SNPLAYER_ALL || player_id == SNPLAYER_OTHERS { PLR_BROADCAST } else { player_id as PlrT };
        if dest != plr_self {
            let pkt = self.base().pktfty.message(plr_self, dest, message);
            self.send(&pkt);
        }
        true
    }

    /// Original: `base::AllTurnsArrived` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::AllTurnsArrived()
    fn all_turns_arrived(&self) -> bool {
        let b = self.base_ref();
        for i in 0..b.env.players {
            let ps = &b.player_state_table[i];
            if !ps.is_connected {
                continue;
            }
            if ps.turn_queue.is_empty() {
                crate::platform::log::verbose!("Turn missing from player {}", i);
                return false;
            }
        }
        true
    }

    /// Original: `base::SNetReceiveTurns` (dvlnet/base.cpp). `data[i]` gets the 4 bytes of the
    /// player's turn value.
    // @port dvlnet/base.cpp|devilution::net::base::SNetReceiveTurns(char **data, size_t *size, uint32_t *status)
    fn base_snet_receive_turns(&mut self, data: &mut [Option<Vec<u8>>; MAX_PLRS], size: &mut [usize; MAX_PLRS], status: &mut [u32; MAX_PLRS]) -> bool {
        self.poll();
        let players = self.base().env.players;
        let current_turn = self.base().current_turn;
        for i in 0..players {
            status[i] = 0;
            let ps = &mut self.base().player_state_table[i];
            if !ps.is_connected {
                continue;
            }
            status[i] |= PS_CONNECTED;
            while let Some(turn) = ps.turn_queue.front() {
                let diff = turn.sequence_number.wrapping_sub(current_turn);
                if diff <= 0x7F {
                    break;
                }
                ps.turn_queue.pop_front();
            }
        }
        if self.all_turns_arrived() {
            let b = self.base();
            for i in 0..players {
                let ps = &mut b.player_state_table[i];
                if !ps.is_connected {
                    continue;
                }
                let Some(turn) = ps.turn_queue.front().copied() else { continue };
                if turn.sequence_number != b.current_turn {
                    continue;
                }
                ps.last_turn_value = turn.value;
                ps.turn_queue.pop_front();
                status[i] |= PS_ACTIVE;
                status[i] |= PS_TURN_ARRIVED;
                size[i] = 4;
                data[i] = Some(ps.last_turn_value.to_le_bytes().to_vec());
            }
            b.current_turn = b.current_turn.wrapping_add(1);
            return true;
        }
        let b = self.base();
        for i in 0..players {
            let ps = &b.player_state_table[i];
            if !ps.is_connected || ps.turn_queue.is_empty() {
                continue;
            }
            status[i] |= PS_ACTIVE;
        }
        false
    }

    /// Original: `base::SNetSendTurn` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::SNetSendTurn(char *data, unsigned int size)
    fn base_snet_send_turn(&mut self, data: &[u8]) -> bool {
        if data.len() != 4 {
            panic!("ABORT: SNetSendTurn size {}", data.len());
        }
        let b = self.base();
        let turn = Turn { sequence_number: b.next_turn, value: i32::from_le_bytes(data.try_into().unwrap()) };
        b.next_turn = b.next_turn.wrapping_add(1);
        let me = b.plr_self as usize;
        b.player_state_table[me].turn_queue.push_back(turn);
        self.send_turn_if_ready(turn);
        true
    }

    /// Original: `base::SendTurnIfReady` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::SendTurnIfReady(turn_t turn)
    fn send_turn_if_ready(&mut self, turn: Turn) {
        let host = self.is_game_host();
        let b = self.base();
        if b.awaiting_sequence_number {
            b.awaiting_sequence_number = !host;
        }
        if !b.awaiting_sequence_number {
            let pkt = b.pktfty.turn(b.plr_self, PLR_BROADCAST, turn);
            self.send(&pkt);
        }
    }

    /// Original: `base::SendFirstTurnIfReady` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::SendFirstTurnIfReady(plr_t player)
    fn send_first_turn_if_ready(&mut self, player: PlrT) {
        let b = self.base();
        if b.awaiting_sequence_number {
            return;
        }
        let me = b.plr_self as usize;
        let turns: Vec<Turn> = b.player_state_table[me].turn_queue.iter().copied().collect();
        for turn in turns {
            let me = self.base().plr_self;
            let pkt = self.base().pktfty.turn(me, player, turn);
            self.send(&pkt);
        }
    }

    /// Original: `base::MakeReady` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::MakeReady(seq_t sequenceNumber)
    fn make_ready(&mut self, sequence_number: SeqT) {
        let b = self.base();
        if !b.awaiting_sequence_number {
            return;
        }
        b.current_turn = sequence_number;
        b.next_turn = sequence_number;
        b.awaiting_sequence_number = false;
        let me = b.plr_self as usize;
        let n = b.player_state_table[me].turn_queue.len();
        for k in 0..n {
            let b = self.base();
            let seq = b.next_turn;
            b.player_state_table[me].turn_queue[k].sequence_number = seq;
            b.next_turn = b.next_turn.wrapping_add(1);
            let turn = b.player_state_table[me].turn_queue[k];
            self.send_turn_if_ready(turn);
        }
    }

    /// Original: `base::SNetGetProviderCaps` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::SNetGetProviderCaps(struct _SNETCAPS *caps)
    fn base_snet_get_provider_caps(&self, caps: &mut SnetCaps) {
        caps.size = 0; // engine writes only ?!?
        caps.flags = 0; // unused
        caps.maxmessagesize = 512; // capped to 512; underflow if < 24
        caps.maxqueuesize = 0; // unused
        caps.maxplayers = MAX_PLRS as u32; // capped to 4
        caps.bytessec = 1000000; // ?
        caps.latencyms = 0; // unused
        caps.defaultturnssec = 10; // ?
        caps.defaultturnsintransit = 2; // maximum acceptable number of turns in queue?
    }

    /// Original: `base::SNetUnregisterEventHandler` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::SNetUnregisterEventHandler(event_type evtype)
    fn base_snet_unregister_event_handler(&mut self, evtype: EventType) -> bool {
        self.base().registered_handlers.remove(&evtype);
        true
    }

    /// Original: `base::SNetRegisterEventHandler` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::SNetRegisterEventHandler(event_type evtype, SEVTHANDLER func)
    fn base_snet_register_event_handler(&mut self, evtype: EventType, func: SevtHandler) -> bool {
        self.base().registered_handlers.insert(evtype, func);
        true
    }

    /// Original: `base::SNetLeaveGame` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::SNetLeaveGame(int type)
    fn base_snet_leave_game(&mut self, type_: i32) -> bool {
        let b = self.base();
        let pkt = b.pktfty.disconnect(b.plr_self, PLR_BROADCAST, b.plr_self, type_);
        self.send(&pkt);
        self.base().plr_self = PLR_BROADCAST;
        true
    }

    /// Original: `base::SNetDropPlayer` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::SNetDropPlayer(int playerid, uint32_t flags)
    fn base_snet_drop_player(&mut self, playerid: i32, flags: u32) -> bool {
        let b = self.base();
        let pkt = b.pktfty.disconnect(b.plr_self, PLR_BROADCAST, playerid as PlrT, flags as LeaveinfoT);
        self.send(&pkt);
        self.recv_local(&pkt);
        true
    }

    /// Original: `base::GetOwner` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::GetOwner()
    fn get_owner(&self) -> PlrT {
        for i in 0..self.base_ref().env.players {
            if self.is_connected(i as PlrT) {
                return i as PlrT;
            }
        }
        PLR_BROADCAST // should be unreachable
    }

    /// Original: `base::SNetGetOwnerTurnsWaiting` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::SNetGetOwnerTurnsWaiting(uint32_t *turns)
    fn base_snet_get_owner_turns_waiting(&mut self, turns: &mut u32) -> bool {
        self.poll();
        let owner = self.get_owner() as usize;
        // The original indexes the table with PLR_BROADCAST if nobody is connected.
        *turns = self.base().player_state_table.get(owner).map_or(0, |ps| ps.turn_queue.len() as u32);
        true
    }

    /// Original: `base::SNetGetTurnsInTransit` (dvlnet/base.cpp).
    // @port dvlnet/base.cpp|devilution::net::base::SNetGetTurnsInTransit(uint32_t *turns)
    fn base_snet_get_turns_in_transit(&mut self, turns: &mut u32) -> bool {
        let b = self.base();
        let me = b.plr_self as usize;
        *turns = b.player_state_table.get(me).map_or(0, |ps| ps.turn_queue.len() as u32);
        true
    }
}
