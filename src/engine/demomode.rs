//! `Source/engine/demomode.cpp`: demo recording (`--record`) and playback (`--demo`,
//! `--timedemo`). A demo file is `demo_<n>.dmo` in the pref folder: version byte, save number,
//! settings, then a stream of game ticks, renderings and input messages.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufWriter, Read, Write};

use crate::ctx::Ctx;
use crate::platform::events::Event;
use crate::platform::log;

/// `DemoMsgType`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum DemoMsgType {
    GameTick = 0,
    Rendering = 1,
    Message = 2,
}

/// The union of `DemoMsg`.
#[derive(Clone, Copy, Debug, Default)]
enum DemoData {
    #[default]
    None,
    Motion { x: u16, y: u16 },
    Button { button: u8, x: u16, y: u16, mod_: u16 },
    Wheel { x: i32, y: i32, mod_: u16 },
    Key { sym: u32, mod_: u16 },
}

/// `DemoMsg`
#[derive(Clone, Copy, Debug)]
struct DemoMsg {
    ty: u32,
    progress_to_next_game_tick: u8,
    event_type: u32,
    data: DemoData,
}

/// `DemoSettings`: options that affect gameplay, stored in the demo file.
#[derive(Clone, Copy, Debug)]
struct DemoSettings {
    run_in_town: bool,
    theo_quest: bool,
    cow_quest: bool,
    auto_gold_pickup: bool,
    auto_elixir_pickup: bool,
    auto_oil_pickup: bool,
    auto_pickup_in_town: bool,
    adria_refills_mana: bool,
    auto_equip_weapons: bool,
    auto_equip_armor: bool,
    auto_equip_helms: bool,
    auto_equip_shields: bool,
    auto_equip_jewelry: bool,
    randomize_quests: bool,
    show_item_labels: bool,
    auto_refill_belt: bool,
    disable_crippling_shrines: bool,
    num_heal_potion_pickup: u8,
    num_full_heal_potion_pickup: u8,
    num_mana_potion_pickup: u8,
    num_full_mana_potion_pickup: u8,
    num_reju_potion_pickup: u8,
    num_full_reju_potion_pickup: u8,
}

impl Default for DemoSettings {
    fn default() -> Self {
        DemoSettings {
            run_in_town: false,
            theo_quest: false,
            cow_quest: false,
            auto_gold_pickup: false,
            auto_elixir_pickup: false,
            auto_oil_pickup: false,
            auto_pickup_in_town: false,
            adria_refills_mana: false,
            auto_equip_weapons: false,
            auto_equip_armor: false,
            auto_equip_helms: false,
            auto_equip_shields: false,
            auto_equip_jewelry: false,
            randomize_quests: false,
            show_item_labels: false,
            auto_refill_belt: false,
            disable_crippling_shrines: false,
            num_heal_potion_pickup: 0,
            num_full_heal_potion_pickup: 0,
            num_mana_potion_pickup: 0,
            num_full_mana_potion_pickup: 0,
            num_reju_potion_pickup: 0,
            num_full_reju_potion_pickup: 0,
        }
    }
}

/// Globals of demomode.cpp.
pub struct DemoState {
    /// `DemoNumber`
    pub demo_number: i32,
    /// `Timedemo`
    pub timedemo: bool,
    /// `RecordNumber`
    pub record_number: i32,
    /// `CreateDemoReference`
    create_demo_reference: bool,
    /// `DemoSettings`
    settings: DemoSettings,
    /// `DemoRecording`
    recording: Option<BufWriter<File>>,
    /// `Demo_Message_Queue`
    queue: VecDeque<DemoMsg>,
    /// `DemoModeLastTick`
    demo_mode_last_tick: u32,
    /// `LogicTick`
    logic_tick: i32,
    /// `StartTime`
    start_time: u32,
    /// `DemoGraphicsWidth`, `DemoGraphicsHeight`
    graphics_width: u16,
    graphics_height: u16,
}

impl Default for DemoState {
    fn default() -> Self {
        DemoState {
            demo_number: -1,
            timedemo: false,
            record_number: -1,
            create_demo_reference: false,
            settings: DemoSettings::default(),
            recording: None,
            queue: VecDeque::new(),
            demo_mode_last_tick: 0,
            logic_tick: 0,
            start_time: 0,
            graphics_width: 640,
            graphics_height: 480,
        }
    }
}

const SDL_QUIT: u32 = 0x100;
const SDL_KEYDOWN: u32 = 0x300;
const SDL_KEYUP: u32 = 0x301;
const SDL_MOUSEMOTION: u32 = 0x400;
const SDL_MOUSEBUTTONDOWN: u32 = 0x401;
const SDL_MOUSEBUTTONUP: u32 = 0x402;
const SDL_MOUSEWHEEL: u32 = 0x403;
const SDL_USEREVENT: u32 = 0x8000;

/// Little-endian reader over the demo file (`utils/endian_stream.hpp`); reads past the end
/// return 0 and set `eof`, like `std::fread` + `std::feof`.
struct DemoReader {
    data: Vec<u8>,
    pos: usize,
    eof: bool,
}

impl DemoReader {
    fn bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut b = [0u8; N];
        if self.pos + N <= self.data.len() {
            b.copy_from_slice(&self.data[self.pos..self.pos + N]);
            self.pos += N;
        } else {
            self.pos = self.data.len();
            self.eof = true;
        }
        b
    }
    fn u8(&mut self) -> u8 {
        self.bytes::<1>()[0]
    }
    fn le16(&mut self) -> u16 {
        u16::from_le_bytes(self.bytes())
    }
    fn le32(&mut self) -> u32 {
        u32::from_le_bytes(self.bytes())
    }
}

fn write_byte(w: &mut BufWriter<File>, v: u8) {
    let _ = w.write_all(&[v]);
}

fn write_le16(w: &mut BufWriter<File>, v: u16) {
    let _ = w.write_all(&v.to_le_bytes());
}

fn write_le32(w: &mut BufWriter<File>, v: u32) {
    let _ = w.write_all(&v.to_le_bytes());
}

/// Original: `ReadSettings` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::ReadSettings(FILE *in, uint8_t version) sha=9f2ac35db288
fn read_settings(ctx: &mut Ctx, r: &mut DemoReader, version: u8) {
    let d = &mut ctx.demo;
    d.graphics_width = r.le16();
    d.graphics_height = r.le16();
    if version > 0 {
        let s = &mut d.settings;
        s.run_in_town = r.u8() != 0;
        s.theo_quest = r.u8() != 0;
        s.cow_quest = r.u8() != 0;
        s.auto_gold_pickup = r.u8() != 0;
        s.auto_elixir_pickup = r.u8() != 0;
        s.auto_oil_pickup = r.u8() != 0;
        s.auto_pickup_in_town = r.u8() != 0;
        s.adria_refills_mana = r.u8() != 0;
        s.auto_equip_weapons = r.u8() != 0;
        s.auto_equip_armor = r.u8() != 0;
        s.auto_equip_helms = r.u8() != 0;
        s.auto_equip_shields = r.u8() != 0;
        s.auto_equip_jewelry = r.u8() != 0;
        s.randomize_quests = r.u8() != 0;
        s.show_item_labels = r.u8() != 0;
        s.auto_refill_belt = r.u8() != 0;
        s.disable_crippling_shrines = r.u8() != 0;
        s.num_heal_potion_pickup = r.u8();
        s.num_full_heal_potion_pickup = r.u8();
        s.num_mana_potion_pickup = r.u8();
        s.num_full_mana_potion_pickup = r.u8();
        s.num_reju_potion_pickup = r.u8();
        s.num_full_reju_potion_pickup = r.u8();
    } else {
        d.settings = DemoSettings::default();
    }
}

/// Original: `WriteSettings` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::WriteSettings(FILE *out) sha=084f20c78cec
fn write_settings(ctx: &Ctx, out: &mut BufWriter<File>) {
    let g = &ctx.options.gameplay;
    write_le16(out, ctx.dx.gn_screen_width as u16);
    write_le16(out, ctx.dx.gn_screen_height as u16);
    for b in [
        g.run_in_town.get(),
        g.theo_quest.get(),
        g.cow_quest.get(),
        g.auto_gold_pickup.get(),
        g.auto_elixir_pickup.get(),
        g.auto_oil_pickup.get(),
        g.auto_pickup_in_town.get(),
        g.adria_refills_mana.get(),
        g.auto_equip_weapons.get(),
        g.auto_equip_armor.get(),
        g.auto_equip_helms.get(),
        g.auto_equip_shields.get(),
        g.auto_equip_jewelry.get(),
        g.randomize_quests.get(),
        g.show_item_labels.get(),
        g.auto_refill_belt.get(),
        g.disable_crippling_shrines.get(),
    ] {
        write_byte(out, b as u8);
    }
    for n in [
        g.num_heal_potion_pickup.get(),
        g.num_full_heal_potion_pickup.get(),
        g.num_mana_potion_pickup.get(),
        g.num_full_mana_potion_pickup.get(),
        g.num_reju_potion_pickup.get(),
        g.num_full_reju_potion_pickup.get(),
    ] {
        write_byte(out, n as u8);
    }
}

/// Original: `CreateSdlEvent` (engine/demomode.cpp, SDL2 build). `SDL_QUIT` messages are not
/// replayed (no case for them, as in the original).
// @port engine/demomode.cpp|devilution::CreateSdlEvent(const DemoMsg &dmsg, SDL_Event &event, uint16_t &modState) sha=293963e2444c
fn create_sdl_event(dmsg: &DemoMsg, mod_state: &mut u16) -> Option<Event> {
    match (dmsg.event_type, dmsg.data) {
        (SDL_MOUSEMOTION, DemoData::Motion { x, y }) => Some(Event::MouseMotion { x: x as i32, y: y as i32 }),
        (SDL_MOUSEBUTTONDOWN | SDL_MOUSEBUTTONUP, DemoData::Button { button, x, y, mod_ }) => {
            *mod_state = mod_;
            let (x, y) = (x as i32, y as i32);
            Some(if dmsg.event_type == SDL_MOUSEBUTTONDOWN {
                Event::MouseButtonDown { button, x, y, clicks: 1 }
            } else {
                Event::MouseButtonUp { button, x, y, clicks: 1 }
            })
        }
        (SDL_MOUSEWHEEL, DemoData::Wheel { x, y, mod_ }) => {
            *mod_state = mod_;
            Some(Event::MouseWheel { x, y })
        }
        (SDL_KEYDOWN | SDL_KEYUP, DemoData::Key { sym, mod_ }) => Some(if dmsg.event_type == SDL_KEYDOWN {
            Event::KeyDown { key: sym as i32, mods: mod_ }
        } else {
            Event::KeyUp { key: sym as i32, mods: mod_ }
        }),
        (t, _) if t >= SDL_USEREVENT => Some(Event::Custom((t - SDL_USEREVENT) as crate::enums::interface_mode)),
        (t, _) => {
            log::info!("Unsupported demo event (type={:x})", t);
            None
        }
    }
}

/// Original: `LogDemoMessage` (engine/demomode.cpp): logs only when LOG_DEMOMODE_MESSAGES is
/// defined, which the release build does not.
// @port engine/demomode.cpp|devilution::LogDemoMessage(const DemoMsg &msg) sha=19d3b00bacce
fn log_demo_message(_msg: &DemoMsg) {}

/// `LoadingStatus`
enum LoadingStatus {
    Success,
    FileNotFound,
    UnsupportedVersion,
}

/// Original: `LoadDemoMessages` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::LoadDemoMessages(int i) sha=98a08180b346
fn load_demo_messages(ctx: &mut Ctx, i: i32) -> LoadingStatus {
    let path = format!("{}demo_{}.dmo", ctx.paths.pref_path(), i);
    let mut data = Vec::new();
    match File::open(&path) {
        Ok(mut f) => {
            if f.read_to_end(&mut data).is_err() {
                return LoadingStatus::FileNotFound;
            }
        }
        Err(_) => return LoadingStatus::FileNotFound,
    }
    let mut r = DemoReader { data, pos: 0, eof: false };

    let version = r.u8();
    if version != 0 && version != 1 {
        return LoadingStatus::UnsupportedVersion;
    }

    ctx.menu.g_save_number = r.le32();
    read_settings(ctx, &mut r, version);

    loop {
        let type_num = r.le32();
        if r.eof {
            break;
        }

        let progress_to_next_game_tick = r.u8();

        if type_num == DemoMsgType::Message as u32 {
            let event_type = r.le32();
            let data = match event_type {
                SDL_MOUSEMOTION => DemoData::Motion { x: r.le16(), y: r.le16() },
                SDL_MOUSEBUTTONDOWN | SDL_MOUSEBUTTONUP => DemoData::Button { button: r.u8(), x: r.le16(), y: r.le16(), mod_: r.le16() },
                SDL_MOUSEWHEEL => DemoData::Wheel { x: r.le32() as i32, y: r.le32() as i32, mod_: r.le16() },
                SDL_KEYDOWN | SDL_KEYUP => DemoData::Key { sym: r.le32(), mod_: r.le16() },
                SDL_QUIT => DemoData::None,
                t => {
                    if t < SDL_USEREVENT {
                        crate::appfat::app_fatal(ctx, &format!("Unknown event {t}"));
                    }
                    DemoData::None
                }
            };
            ctx.demo.queue.push_back(DemoMsg { ty: type_num, progress_to_next_game_tick, event_type, data });
        } else {
            ctx.demo.queue.push_back(DemoMsg { ty: type_num, progress_to_next_game_tick, event_type: 0, data: DemoData::None });
        }
    }

    ctx.demo.demo_mode_last_tick = ctx.platform.ticks();

    LoadingStatus::Success
}

/// Original: `RecordEventHeader` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::RecordEventHeader(const SDL_Event &event) sha=434f32278954
fn record_event_header(ctx: &mut Ctx, event_type: u32) {
    let progress = ctx.nthread.ProgressToNextGameTick;
    if let Some(w) = ctx.demo.recording.as_mut() {
        write_le32(w, DemoMsgType::Message as u32);
        write_byte(w, progress);
        write_le32(w, event_type);
    }
}

/// Original: `demo::InitPlayBack` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::demo::InitPlayBack(int demoNumber, bool timedemo) sha=91de998d18e6
pub fn init_play_back(ctx: &mut Ctx, demo_number: i32, timedemo: bool) {
    ctx.demo.demo_number = demo_number;
    ctx.demo.timedemo = timedemo;
    ctx.controls.control_mode = crate::controls::ControlTypes::KeyboardAndMouse;

    match load_demo_messages(ctx, demo_number) {
        LoadingStatus::Success => return,
        LoadingStatus::FileNotFound => crate::utils::console::print_in_console("Demo file not found"),
        LoadingStatus::UnsupportedVersion => crate::utils::console::print_in_console("Unsupported Demo version"),
    }
    crate::utils::console::print_newline_in_console();
    crate::diablo::diablo_quit(ctx, 1);
}

/// Original: `demo::InitRecording` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::demo::InitRecording(int recordNumber, bool createDemoReference) sha=2ec256a5909c
pub fn init_recording(ctx: &mut Ctx, record_number: i32, create_demo_reference: bool) {
    ctx.demo.record_number = record_number;
    ctx.demo.create_demo_reference = create_demo_reference;
}

/// Original: `demo::OverrideOptions` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::demo::OverrideOptions() sha=8f7a97649ed2
pub fn override_options(ctx: &mut Ctx) {
    let mut callbacks = Vec::new();
    {
        let o = &mut ctx.options;
        callbacks.push(o.graphics.fit_to_screen.set_value(false));
        callbacks.push(o.graphics.hardware_cursor.set_value(false));
        if ctx.demo.timedemo {
            callbacks.push(o.graphics.v_sync.set_value(false));
            callbacks.push(o.graphics.limit_fps.set_value(false));
        }
    }
    ctx.dx.force_resolution = (ctx.demo.graphics_width as i32, ctx.demo.graphics_height as i32);

    let s = ctx.demo.settings;
    let g = &mut ctx.options.gameplay;
    callbacks.push(g.run_in_town.set_value(s.run_in_town));
    callbacks.push(g.theo_quest.set_value(s.theo_quest));
    callbacks.push(g.cow_quest.set_value(s.cow_quest));
    callbacks.push(g.auto_gold_pickup.set_value(s.auto_gold_pickup));
    callbacks.push(g.auto_elixir_pickup.set_value(s.auto_elixir_pickup));
    callbacks.push(g.auto_oil_pickup.set_value(s.auto_oil_pickup));
    callbacks.push(g.auto_pickup_in_town.set_value(s.auto_pickup_in_town));
    callbacks.push(g.adria_refills_mana.set_value(s.adria_refills_mana));
    callbacks.push(g.auto_equip_weapons.set_value(s.auto_equip_weapons));
    callbacks.push(g.auto_equip_armor.set_value(s.auto_equip_armor));
    callbacks.push(g.auto_equip_helms.set_value(s.auto_equip_helms));
    callbacks.push(g.auto_equip_shields.set_value(s.auto_equip_shields));
    callbacks.push(g.auto_equip_jewelry.set_value(s.auto_equip_jewelry));
    callbacks.push(g.randomize_quests.set_value(s.randomize_quests));
    callbacks.push(g.show_item_labels.set_value(s.show_item_labels));
    callbacks.push(g.auto_refill_belt.set_value(s.auto_refill_belt));
    callbacks.push(g.disable_crippling_shrines.set_value(s.disable_crippling_shrines));
    callbacks.push(g.num_heal_potion_pickup.set_value(s.num_heal_potion_pickup as i32));
    callbacks.push(g.num_full_heal_potion_pickup.set_value(s.num_full_heal_potion_pickup as i32));
    callbacks.push(g.num_mana_potion_pickup.set_value(s.num_mana_potion_pickup as i32));
    callbacks.push(g.num_full_mana_potion_pickup.set_value(s.num_full_mana_potion_pickup as i32));
    callbacks.push(g.num_reju_potion_pickup.set_value(s.num_reju_potion_pickup as i32));
    callbacks.push(g.num_full_reju_potion_pickup.set_value(s.num_full_reju_potion_pickup as i32));
    for cb in callbacks.into_iter().flatten() {
        crate::options::run_option_callback(ctx, cb);
    }
}

/// Original: `demo::IsRunning` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::demo::IsRunning() sha=c1bcd2c9a00b
pub fn is_running(ctx: &Ctx) -> bool {
    ctx.demo.demo_number != -1
}

/// Original: `demo::IsRecording` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::demo::IsRecording() sha=a3b359a4f217
pub fn is_recording(ctx: &Ctx) -> bool {
    ctx.demo.record_number != -1
}

/// Original: `demo::GetRunGameLoop` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::demo::GetRunGameLoop(bool &drawGame, bool &processInput) sha=5bbcbc5f898a
pub fn get_run_game_loop(ctx: &mut Ctx, draw_game: &mut bool, process_input: &mut bool) -> bool {
    let Some(dmsg) = ctx.demo.queue.front().copied() else {
        crate::appfat::app_fatal(ctx, "Demo queue empty");
    };
    log_demo_message(&dmsg);
    if dmsg.ty == DemoMsgType::Message as u32 {
        crate::appfat::app_fatal(ctx, "Unexpected Message");
    }
    let is_game_tick = dmsg.ty == DemoMsgType::GameTick as u32;
    if ctx.demo.timedemo {
        // disable additonal rendering to speedup replay
        *draw_game = is_game_tick && !ctx.diablo.headless_mode;
    } else {
        let current_tick_count = ctx.platform.ticks() as i32;
        let ticks_elapsed = current_tick_count - ctx.demo.demo_mode_last_tick as i32;
        let gn_tick_delay = ctx.diablo.gn_tick_delay as i32;
        let tick_due = ticks_elapsed >= gn_tick_delay;
        *draw_game = false;
        if tick_due {
            if is_game_tick {
                ctx.demo.demo_mode_last_tick = current_tick_count as u32;
            }
        } else {
            let base = crate::engine::animationinfo::AnimationInfo::BASE_VALUE_FRACTION as i32;
            let fraction = (ticks_elapsed * base / gn_tick_delay).clamp(0, base);
            let progress_to_next_game_tick = fraction as u8;
            if is_game_tick || dmsg.progress_to_next_game_tick > progress_to_next_game_tick {
                // we are ahead of the replay => add a additional rendering for smoothness
                if ctx.diablo.gb_run_game
                    && ctx.diablo.pause_mode == 0
                    && (ctx.init.gb_is_multiplayer || !crate::gmenu::gmenu_is_active(ctx))
                    && ctx.diablo.gb_process_players
                {
                    // if game is not running or paused there is no next gametick in the near future
                    ctx.nthread.ProgressToNextGameTick = progress_to_next_game_tick;
                }
                *process_input = false;
                *draw_game = true;
                return false;
            }
        }
    }
    ctx.nthread.ProgressToNextGameTick = dmsg.progress_to_next_game_tick;
    ctx.demo.queue.pop_front();
    if is_game_tick {
        ctx.demo.logic_tick += 1;
        // Test hook: DIABLO_DEMO_TRACE=1 prints the player's state every replayed game tick.
        if std::env::var_os("DIABLO_DEMO_TRACE").is_some() {
            if let Some(me) = ctx.players.MyPlayer {
                let pl = &ctx.players.Players[me];
                eprintln!(
                    "TRACE {} lvl {} tile {:?} mode {:?} dest {:?} hp {} exp {} mouse {:?} curs {} monst {} item {} obj {:?} queue {}",
                    ctx.demo.logic_tick,
                    ctx.gendung.currlevel,
                    pl.position.tile,
                    pl._pmode,
                    pl.destAction,
                    pl._pHitPoints >> 6,
                    pl._pExperience,
                    ctx.diablo.mouse_position,
                    ctx.cursor.pcurs,
                    ctx.cursor.pcursmonst,
                    ctx.cursor.pcursitem,
                    ctx.cursor.ObjectUnderCursor,
                    ctx.demo.queue.len()
                );
            }
        }
    }
    is_game_tick
}

fn input_disabled(ctx: &Ctx) -> bool {
    ctx.events.current_event_handler.is_some_and(|h| h as usize == crate::diablo_game::disable_input_event_handler as crate::engine::events::EventHandler as usize)
}

/// Original: `demo::FetchMessage` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::demo::FetchMessage(SDL_Event *event, uint16_t *modState) sha=93cbde771f93
pub fn fetch_message(ctx: &mut Ctx) -> Option<(Option<Event>, u16)> {
    use crate::platform::events::keys::{SDLK_ESCAPE, SDLK_KP_MINUS, SDLK_KP_PLUS, SDLK_MINUS, SDLK_PLUS};
    if input_disabled(ctx) {
        return None;
    }

    if let Some(e) = ctx.platform.poll_event() {
        if e == Event::Quit {
            return Some((Some(e), 0));
        }
        if let Event::KeyDown { key, .. } = e {
            if key == SDLK_ESCAPE {
                ctx.demo.queue.clear();
                ctx.demo.demo_number = -1;
                ctx.demo.timedemo = false;
                ctx.nthread.last_tick = ctx.platform.ticks() as i32;
            }
            if (key == SDLK_KP_PLUS || key == SDLK_PLUS) && ctx.multi.sgGameInitInfo.nTickRate < 255 {
                ctx.multi.sgGameInitInfo.nTickRate += 1;
                let rate = ctx.multi.sgGameInitInfo.nTickRate;
                if let Some(cb) = ctx.options.gameplay.tick_rate.set_value(rate as i32) {
                    crate::options::run_option_callback(ctx, cb);
                }
                ctx.diablo.gn_tick_delay = (1000 / rate as u32) as u16;
            }
            if (key == SDLK_KP_MINUS || key == SDLK_MINUS) && ctx.multi.sgGameInitInfo.nTickRate > 1 {
                ctx.multi.sgGameInitInfo.nTickRate -= 1;
                let rate = ctx.multi.sgGameInitInfo.nTickRate;
                if let Some(cb) = ctx.options.gameplay.tick_rate.set_value(rate as i32) {
                    crate::options::run_option_callback(ctx, cb);
                }
                ctx.diablo.gn_tick_delay = (1000 / rate as u32) as u16;
            }
        }
    }

    if let Some(dmsg) = ctx.demo.queue.front().copied() {
        log_demo_message(&dmsg);
        if dmsg.ty == DemoMsgType::Message as u32 {
            let mut mod_state = 0;
            let event = create_sdl_event(&dmsg, &mut mod_state);
            ctx.nthread.ProgressToNextGameTick = dmsg.progress_to_next_game_tick;
            ctx.demo.queue.pop_front();
            return event.map(|e| (Some(e), mod_state));
        }
    }

    None
}

/// Original: `demo::RecordGameLoopResult` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::demo::RecordGameLoopResult(bool runGameLoop) sha=72961a8132c0
pub fn record_game_loop_result(ctx: &mut Ctx, run_game_loop: bool) {
    let progress = ctx.nthread.ProgressToNextGameTick;
    if let Some(w) = ctx.demo.recording.as_mut() {
        write_le32(w, if run_game_loop { DemoMsgType::GameTick } else { DemoMsgType::Rendering } as u32);
        write_byte(w, progress);
    }
}

/// Original: `demo::RecordMessage` (engine/demomode.cpp). A closed window arrives as
/// `Event::Quit` and is recorded as `SDL_QUIT`, as the original records `SDL_WINDOWEVENT_CLOSE`.
// @port engine/demomode.cpp|devilution::demo::RecordMessage(const SDL_Event &event, uint16_t modState) sha=399cf6fd1911
pub fn record_message(ctx: &mut Ctx, event: Option<&Event>, mod_state: u16) {
    if !ctx.diablo.gb_run_game || ctx.demo.recording.is_none() {
        return;
    }
    if input_disabled(ctx) {
        return;
    }
    let Some(event) = event else { return };
    match *event {
        Event::MouseMotion { x, y } => {
            record_event_header(ctx, SDL_MOUSEMOTION);
            let w = ctx.demo.recording.as_mut().unwrap();
            write_le16(w, x as u16);
            write_le16(w, y as u16);
        }
        Event::MouseButtonDown { button, x, y, .. } | Event::MouseButtonUp { button, x, y, .. } => {
            record_event_header(ctx, if matches!(event, Event::MouseButtonDown { .. }) { SDL_MOUSEBUTTONDOWN } else { SDL_MOUSEBUTTONUP });
            let w = ctx.demo.recording.as_mut().unwrap();
            write_byte(w, button);
            write_le16(w, x as u16);
            write_le16(w, y as u16);
            write_le16(w, mod_state);
        }
        Event::MouseWheel { x, y } => {
            record_event_header(ctx, SDL_MOUSEWHEEL);
            let w = ctx.demo.recording.as_mut().unwrap();
            write_le32(w, x as u32);
            write_le32(w, y as u32);
            write_le16(w, mod_state);
        }
        Event::KeyDown { key, mods } | Event::KeyUp { key, mods } => {
            record_event_header(ctx, if matches!(event, Event::KeyDown { .. }) { SDL_KEYDOWN } else { SDL_KEYUP });
            let w = ctx.demo.recording.as_mut().unwrap();
            write_le32(w, key as u32);
            write_le16(w, mods);
        }
        Event::Quit => record_event_header(ctx, SDL_QUIT),
        Event::Custom(mode) => record_event_header(ctx, SDL_USEREVENT + mode as u32),
        _ => {}
    }
}

/// Original: `demo::NotifyGameLoopStart` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::demo::NotifyGameLoopStart() sha=369a9b27c392
pub fn notify_game_loop_start(ctx: &mut Ctx) {
    if is_recording(ctx) {
        let path = format!("{}demo_{}.dmo", ctx.paths.pref_path(), ctx.demo.record_number);
        let Ok(f) = File::create(&path) else {
            ctx.demo.record_number = -1;
            log::error!("Failed to open {} for writing", path);
            return;
        };
        let mut w = BufWriter::new(f);
        const VERSION: u8 = 1;
        write_byte(&mut w, VERSION);
        write_le32(&mut w, ctx.menu.g_save_number);
        write_settings(ctx, &mut w);
        ctx.demo.recording = Some(w);
    }

    if is_running(ctx) {
        ctx.demo.start_time = ctx.platform.ticks();
        ctx.demo.logic_tick = 0;
    }
}

/// Original: `demo::NotifyGameLoopEnd` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::demo::NotifyGameLoopEnd() sha=d44a4f2193d2
pub fn notify_game_loop_end(ctx: &mut Ctx) {
    if is_recording(ctx) {
        if let Some(mut w) = ctx.demo.recording.take() {
            let _ = w.flush();
        }
        if ctx.demo.create_demo_reference {
            let n = ctx.demo.record_number;
            crate::pfile::pfile_write_hero_demo(ctx, n);
        }

        ctx.demo.record_number = -1;
        ctx.demo.create_demo_reference = false;
    }

    if is_running(ctx) && !ctx.diablo.headless_mode {
        let seconds = ctx.platform.ticks().wrapping_sub(ctx.demo.start_time) as f32 / 1000.0;
        let ticks = ctx.demo.logic_tick;
        log::info!("{} frames, {:.2} seconds: {:.1} fps", ticks, seconds, ticks as f32 / seconds);
        ctx.diablo.gb_run_game_result = false;
        ctx.diablo.gb_run_game = false;

        let n = ctx.demo.demo_number;
        let (status, message) = crate::pfile::pfile_compare_hero_demo(ctx, n, false);
        match status {
            crate::pfile::HERO_COMPARE_REFERENCE_NOT_FOUND => log::info!("Timedemo: No final comparison cause reference is not present."),
            crate::pfile::HERO_COMPARE_SAME => log::info!("Timedemo: Same outcome as initial run. :)"),
            _ => log::info!("Timedemo: Different outcome than initial run. ;(\n{}", message),
        }
    }
}
