//! `Source/dvlnet/cdwrap.h`: recreates the provider for every create/join, so a failed or
//! finished game leaves no state behind, and replays the game info, password and event
//! handlers into the new instance.

use std::collections::BTreeMap;

use super::base::NetEnv;
use crate::multi::{GameInfo, MAX_PLRS};
use crate::storm::storm_net::{AbstractNet, EventType, SevtHandler, SnetCaps, SnetEvent};

/// `cdwrap<T>`
pub struct CdWrap<T: AbstractNet + Default> {
    dvlnet_wrap: Box<T>,
    registered_handlers: BTreeMap<EventType, SevtHandler>,
    game_init_info: Vec<u8>,
    game_pw: Option<String>,
    env: NetEnv,
}

impl<T: AbstractNet + Default> Default for CdWrap<T> {
    /// Original: `cdwrap<T>::cdwrap` (dvlnet/cdwrap.h).
    // @port dvlnet/cdwrap.h|devilution::net::cdwrap() sha=1fb2a5b5aa1d
    fn default() -> Self {
        let mut w = CdWrap { dvlnet_wrap: Box::default(), registered_handlers: BTreeMap::new(), game_init_info: Vec::new(), game_pw: None, env: NetEnv::default() };
        w.reset();
        w
    }
}

impl<T: AbstractNet + Default> CdWrap<T> {
    /// Original: `cdwrap<T>::reset` (dvlnet/cdwrap.h).
    // @port dvlnet/cdwrap.h|devilution::net::reset() sha=40a49006b0d2
    fn reset(&mut self) {
        self.dvlnet_wrap = Box::default();
        self.dvlnet_wrap.set_env(self.env.clone());
        self.dvlnet_wrap.setup_gameinfo(self.game_init_info.clone());
        match &self.game_pw {
            Some(pw) => self.dvlnet_wrap.setup_password(pw.clone()),
            None => self.dvlnet_wrap.clear_password(),
        }
        for (&evtype, &func) in &self.registered_handlers {
            self.dvlnet_wrap.snet_register_event_handler(evtype, func);
        }
    }
}

impl<T: AbstractNet + Default> AbstractNet for CdWrap<T> {
    // @port dvlnet/cdwrap.h|devilution::net::create(std::string addrstr) sha=0c5fa6dc96b3
    fn create(&mut self, addrstr: &str, is_loopback: &mut bool) -> i32 {
        self.reset();
        self.dvlnet_wrap.create(addrstr, is_loopback)
    }

    // @port dvlnet/cdwrap.h|devilution::net::join(std::string addrstr) sha=e42d59969815
    fn join(&mut self, addrstr: &str) -> i32 {
        self.game_init_info = Vec::new();
        self.reset();
        self.dvlnet_wrap.join(addrstr)
    }

    // @port dvlnet/cdwrap.h|devilution::net::setup_gameinfo(buffer_t info) sha=3960d7d07060
    fn setup_gameinfo(&mut self, info: Vec<u8>) {
        self.game_init_info = info;
        self.dvlnet_wrap.setup_gameinfo(self.game_init_info.clone());
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetReceiveMessage(uint8_t *sender, void **data, uint32_t *size) sha=09e47b8305df
    fn snet_receive_message(&mut self) -> Option<(u8, Vec<u8>)> {
        self.dvlnet_wrap.snet_receive_message()
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetSendMessage(int playerID, void *data, unsigned int size) sha=7cf8c6da8eb1
    fn snet_send_message(&mut self, dest: i32, data: &[u8]) -> bool {
        self.dvlnet_wrap.snet_send_message(dest, data)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetReceiveTurns(char **data, size_t *size, uint32_t *status) sha=cf3af175aa16
    fn snet_receive_turns(&mut self, players: usize, data: &mut [Option<Vec<u8>>; MAX_PLRS], size: &mut [usize; MAX_PLRS], status: &mut [u32; MAX_PLRS]) -> bool {
        self.dvlnet_wrap.snet_receive_turns(players, data, size, status)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetSendTurn(char *data, unsigned int size) sha=213fc3ed77fd
    fn snet_send_turn(&mut self, data: &[u8]) -> bool {
        self.dvlnet_wrap.snet_send_turn(data)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetGetProviderCaps(struct _SNETCAPS *caps) sha=05f5c330de93
    fn snet_get_provider_caps(&self, caps: &mut SnetCaps) {
        self.dvlnet_wrap.snet_get_provider_caps(caps)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetUnregisterEventHandler(event_type evtype) sha=c7e474764469
    fn snet_unregister_event_handler(&mut self, evtype: EventType) -> bool {
        self.registered_handlers.remove(&evtype);
        self.dvlnet_wrap.snet_unregister_event_handler(evtype)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetRegisterEventHandler(event_type evtype, SEVTHANDLER func) sha=4b1b985acec6
    fn snet_register_event_handler(&mut self, evtype: EventType, func: SevtHandler) -> bool {
        self.registered_handlers.insert(evtype, func);
        self.dvlnet_wrap.snet_register_event_handler(evtype, func)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetLeaveGame(int type) sha=8e6b94386bb3
    fn snet_leave_game(&mut self, type_: i32, is_loopback: &mut bool) -> bool {
        self.dvlnet_wrap.snet_leave_game(type_, is_loopback)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetDropPlayer(int playerid, uint32_t flags) sha=59907f9c57e1
    fn snet_drop_player(&mut self, playerid: i32, flags: u32) -> bool {
        self.dvlnet_wrap.snet_drop_player(playerid, flags)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetGetOwnerTurnsWaiting(uint32_t *turns) sha=608ab7006909
    fn snet_get_owner_turns_waiting(&mut self, turns: &mut u32) -> bool {
        self.dvlnet_wrap.snet_get_owner_turns_waiting(turns)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetGetTurnsInTransit(uint32_t *turns) sha=1ebfc79c34a4
    fn snet_get_turns_in_transit(&mut self, turns: &mut u32) -> bool {
        self.dvlnet_wrap.snet_get_turns_in_transit(turns)
    }

    // @port dvlnet/cdwrap.h|devilution::net::make_default_gamename() sha=1ea7c81fe1b5
    fn make_default_gamename(&self) -> String {
        self.dvlnet_wrap.make_default_gamename()
    }

    // @port dvlnet/cdwrap.h|devilution::net::send_info_request() sha=c25cd3449c16
    fn send_info_request(&mut self) -> bool {
        self.dvlnet_wrap.send_info_request()
    }

    // @port dvlnet/cdwrap.h|devilution::net::clear_gamelist() sha=7110a3467bc9
    fn clear_gamelist(&mut self) {
        self.dvlnet_wrap.clear_gamelist()
    }

    // @port dvlnet/cdwrap.h|devilution::net::get_gamelist() sha=4b725593e1a5
    fn get_gamelist(&mut self) -> Vec<GameInfo> {
        self.dvlnet_wrap.get_gamelist()
    }

    // @port dvlnet/cdwrap.h|devilution::net::setup_password(std::string pw) sha=8a9c33583bd1
    fn setup_password(&mut self, pw: String) {
        self.game_pw = Some(pw.clone());
        self.dvlnet_wrap.setup_password(pw)
    }

    // @port dvlnet/cdwrap.h|devilution::net::clear_password() sha=bf15c5b633d0
    fn clear_password(&mut self) {
        self.game_pw = None;
        self.dvlnet_wrap.clear_password()
    }

    fn set_env(&mut self, env: NetEnv) {
        self.env = env.clone();
        self.dvlnet_wrap.set_env(env);
    }

    fn take_events(&mut self) -> Vec<(SevtHandler, SnetEvent)> {
        self.dvlnet_wrap.take_events()
    }

    fn take_error(&mut self) -> Option<String> {
        self.dvlnet_wrap.take_error()
    }
}
