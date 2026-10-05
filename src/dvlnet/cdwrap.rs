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
    // @port dvlnet/cdwrap.h|devilution::net::cdwrap()
    fn default() -> Self {
        let mut w = CdWrap { dvlnet_wrap: Box::default(), registered_handlers: BTreeMap::new(), game_init_info: Vec::new(), game_pw: None, env: NetEnv::default() };
        w.reset();
        w
    }
}

impl<T: AbstractNet + Default> CdWrap<T> {
    /// Original: `cdwrap<T>::reset` (dvlnet/cdwrap.h).
    // @port dvlnet/cdwrap.h|devilution::net::reset()
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
    // @port dvlnet/cdwrap.h|devilution::net::create(std::string addrstr)
    fn create(&mut self, addrstr: &str, is_loopback: &mut bool) -> i32 {
        self.reset();
        self.dvlnet_wrap.create(addrstr, is_loopback)
    }

    // @port dvlnet/cdwrap.h|devilution::net::join(std::string addrstr)
    fn join(&mut self, addrstr: &str) -> i32 {
        self.game_init_info = Vec::new();
        self.reset();
        self.dvlnet_wrap.join(addrstr)
    }

    // @port dvlnet/cdwrap.h|devilution::net::setup_gameinfo(buffer_t info)
    fn setup_gameinfo(&mut self, info: Vec<u8>) {
        self.game_init_info = info;
        self.dvlnet_wrap.setup_gameinfo(self.game_init_info.clone());
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetReceiveMessage(uint8_t *sender, void **data, uint32_t *size)
    fn snet_receive_message(&mut self) -> Option<(u8, Vec<u8>)> {
        self.dvlnet_wrap.snet_receive_message()
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetSendMessage(int playerID, void *data, unsigned int size)
    fn snet_send_message(&mut self, dest: i32, data: &[u8]) -> bool {
        self.dvlnet_wrap.snet_send_message(dest, data)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetReceiveTurns(char **data, size_t *size, uint32_t *status)
    fn snet_receive_turns(&mut self, players: usize, data: &mut [Option<Vec<u8>>; MAX_PLRS], size: &mut [usize; MAX_PLRS], status: &mut [u32; MAX_PLRS]) -> bool {
        self.dvlnet_wrap.snet_receive_turns(players, data, size, status)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetSendTurn(char *data, unsigned int size)
    fn snet_send_turn(&mut self, data: &[u8]) -> bool {
        self.dvlnet_wrap.snet_send_turn(data)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetGetProviderCaps(struct _SNETCAPS *caps)
    fn snet_get_provider_caps(&self, caps: &mut SnetCaps) {
        self.dvlnet_wrap.snet_get_provider_caps(caps)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetUnregisterEventHandler(event_type evtype)
    fn snet_unregister_event_handler(&mut self, evtype: EventType) -> bool {
        self.registered_handlers.remove(&evtype);
        self.dvlnet_wrap.snet_unregister_event_handler(evtype)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetRegisterEventHandler(event_type evtype, SEVTHANDLER func)
    fn snet_register_event_handler(&mut self, evtype: EventType, func: SevtHandler) -> bool {
        self.registered_handlers.insert(evtype, func);
        self.dvlnet_wrap.snet_register_event_handler(evtype, func)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetLeaveGame(int type)
    fn snet_leave_game(&mut self, type_: i32, is_loopback: &mut bool) -> bool {
        self.dvlnet_wrap.snet_leave_game(type_, is_loopback)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetDropPlayer(int playerid, uint32_t flags)
    fn snet_drop_player(&mut self, playerid: i32, flags: u32) -> bool {
        self.dvlnet_wrap.snet_drop_player(playerid, flags)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetGetOwnerTurnsWaiting(uint32_t *turns)
    fn snet_get_owner_turns_waiting(&mut self, turns: &mut u32) -> bool {
        self.dvlnet_wrap.snet_get_owner_turns_waiting(turns)
    }

    // @port dvlnet/cdwrap.h|devilution::net::SNetGetTurnsInTransit(uint32_t *turns)
    fn snet_get_turns_in_transit(&mut self, turns: &mut u32) -> bool {
        self.dvlnet_wrap.snet_get_turns_in_transit(turns)
    }

    // @port dvlnet/cdwrap.h|devilution::net::make_default_gamename()
    fn make_default_gamename(&self) -> String {
        self.dvlnet_wrap.make_default_gamename()
    }

    // @port dvlnet/cdwrap.h|devilution::net::send_info_request()
    fn send_info_request(&mut self) -> bool {
        self.dvlnet_wrap.send_info_request()
    }

    // @port dvlnet/cdwrap.h|devilution::net::clear_gamelist()
    fn clear_gamelist(&mut self) {
        self.dvlnet_wrap.clear_gamelist()
    }

    // @port dvlnet/cdwrap.h|devilution::net::get_gamelist()
    fn get_gamelist(&mut self) -> Vec<GameInfo> {
        self.dvlnet_wrap.get_gamelist()
    }

    // @port dvlnet/cdwrap.h|devilution::net::setup_password(std::string pw)
    fn setup_password(&mut self, pw: String) {
        self.game_pw = Some(pw.clone());
        self.dvlnet_wrap.setup_password(pw)
    }

    // @port dvlnet/cdwrap.h|devilution::net::clear_password()
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
