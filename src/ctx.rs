//! The game context: the original's global variables, grouped by the source file that defines
//! them, owned by the game thread and passed to every ported function.

use crate::engine::random::DiabloRng;
use crate::options::{Options, OptionsIni};
use crate::platform::Platform;
use crate::utils::paths::Paths;

pub struct Ctx {
    pub platform: Platform,
    pub paths: Paths,
    pub options: Options,
    pub options_ini: OptionsIni,
    /// `init.cpp`
    pub init: crate::init::InitState,
    /// `diablo.cpp`
    pub diablo: crate::diablo::DiabloState,
    /// `control.cpp`
    pub control: crate::control::ControlState,
    /// `controls/*`
    pub controls: crate::controls::ControlsState,
    /// `engine/demomode.cpp`
    pub demo: crate::engine::demomode::DemoState,
    /// `engine/dx.cpp`, `engine/palette.cpp`, `utils/display.cpp`
    pub dx: crate::engine::dx::DxState,
    /// `engine/random.cpp`
    pub rng: DiabloRng,
    /// `appfat.cpp`
    pub appfat: crate::appfat::AppfatState,
    /// `levels/gendung.cpp`
    pub gendung: crate::levels::gendung::GendungState,
    /// `DiabloUI/dialogs.cpp`
    pub dialogs: crate::diablo_ui::dialogs::DialogsState,
    /// `engine/render/text_render.cpp`
    pub text_render: crate::engine::render::text_render::TextRenderState,
    /// `DiabloUI/diabloui.cpp`
    pub diablo_ui: crate::diablo_ui::diabloui::DiabloUiState,
    /// `hwcursor.cpp`
    pub hwcursor: crate::hwcursor::HwCursorState,
    /// `cursor.cpp`
    pub cursor: crate::cursor::CursorState,
    /// `automap.cpp`
    pub automap: crate::automap::AutomapState,
    /// `player.cpp`
    pub players: crate::player::PlayerState,
    /// `engine/sound.cpp`
    pub sound: crate::engine::sound::SoundState,
    /// `effects.cpp`
    pub effects: crate::effects::EffectsState,
    /// `monster.cpp`
    pub monster: crate::monster::MonsterState,
    /// `missiles.cpp`, `misdat.cpp`
    pub missiles: crate::missiles::MissilesState,
    /// `objects.cpp`
    pub objects: crate::objects::ObjectsState,
    /// `towners.cpp`
    pub towners: crate::towners::TownersState,
    /// `qol/stash.cpp`
    pub stash: crate::qol::stash::StashState,
    /// `items.cpp`
    pub items: crate::items::ItemsState,
    /// `multi.cpp`
    pub multi: crate::multi::MultiState,
    /// `menu.cpp`
    pub menu: crate::menu::MenuState,
    /// `error.cpp`
    pub error: crate::error::ErrorState,
    /// `storm/storm_net.cpp`
    pub storm_net: crate::storm::storm_net::StormNetState,
    /// `lighting.cpp`
    pub lighting: crate::lighting::LightingState,
    /// `stores.cpp`
    pub stores: crate::stores::StoresState,
    /// `quests.cpp`
    pub quests: crate::quests::QuestsState,
    /// `inv.cpp`
    pub inv: crate::inv::InvState,
    /// `nthread.cpp`
    pub nthread: crate::nthread::NthreadState,
    /// `portal.cpp`
    pub portal: crate::portal::PortalState,
    /// `levels/trigs.cpp`
    pub trigs: crate::levels::trigs::TrigsState,
    /// `help.cpp`
    pub help: crate::help::HelpState,
    /// `msg.cpp`
    pub msg: crate::msg::MsgState,
    /// `loadsave.cpp`
    pub loadsave: crate::loadsave::LoadsaveState,
    /// `pfile.cpp`
    pub pfile: crate::pfile::PfileState,
    /// `sync.cpp`
    pub sync: crate::sync::SyncState,
    /// `tmsg.cpp`
    pub tmsg: crate::tmsg::TmsgState,
    /// `plrmsg.cpp`
    pub plrmsg: crate::plrmsg::PlrMsgState,
    /// `qol/chatlog.cpp`
    pub chatlog: crate::qol::chatlog::ChatLogState,
    /// the C runtime's `rand()` state
    pub crt_rand: crate::utils::crt_rand::CrtRand,
    /// `minitext.cpp`
    pub minitext: crate::minitext::MinitextState,
    /// `levels/crypt.cpp`
    pub crypt: crate::levels::crypt::CryptState,
    /// `engine/events.cpp`
    pub events: crate::engine::events::EventsState,
    /// `interfac.cpp`
    pub interfac: crate::interfac::InterfacState,
    /// `doom.cpp`
    pub doom: crate::doom::DoomState,
    /// `engine/render/scrollrt.cpp`
    pub scrollrt: crate::engine::render::scrollrt::ScrollrtState,
    /// `dead.cpp`
    pub dead: crate::dead::DeadState,
    /// `panels/info_box.cpp`
    pub info_box: crate::panels::info_box::InfoBoxState,
    /// `qol/monhealthbar.cpp`
    pub monhealthbar: crate::qol::monhealthbar::MonHealthBarState,
    /// `qol/xpbar.cpp`
    pub xpbar: crate::qol::xpbar::XpBarState,
    /// `qol/itemlabels.cpp`
    pub itemlabels: crate::qol::itemlabels::ItemLabelsState,
    /// `qol/floatingnumbers.cpp`
    pub floatingnumbers: crate::qol::floatingnumbers::FloatingNumbersState,
    /// panels/mainpanel, charpanel, spell_icons, spell_book
    pub panels: crate::panels::PanelsState,
    /// gmenu.cpp
    pub gmenu: crate::gmenu::GmenuState,
    /// gamemenu.cpp
    pub gamemenu: crate::gamemenu::GamemenuState,
    /// controls/modifier_hints.cpp
    pub modifier_hints: crate::controls::modifier_hints::ModifierHintsState,
    /// levels/themes.cpp
    pub themes: crate::levels::themes::ThemesState,
    /// levels/drlg_l1.cpp
    pub drlg_l1: crate::levels::drlg_l1::DrlgL1State,
    /// levels/drlg_l4.cpp
    pub drlg_l4: crate::levels::drlg_l4::DrlgL4State,
    /// movie.cpp
    pub movie: crate::movie::MovieState,
}

impl Ctx {
    pub fn new(platform: Platform) -> Ctx {
        Ctx {
            platform,
            paths: Paths::default(),
            options: Options::default(),
            options_ini: OptionsIni::default(),
            init: Default::default(),
            diablo: Default::default(),
            control: Default::default(),
            controls: Default::default(),
            demo: Default::default(),
            dx: Default::default(),
            rng: DiabloRng::default(),
            appfat: Default::default(),
            gendung: Default::default(),
            dialogs: Default::default(),
            text_render: Default::default(),
            diablo_ui: Default::default(),
            hwcursor: Default::default(),
            cursor: Default::default(),
            automap: Default::default(),
            players: Default::default(),
            sound: Default::default(),
            effects: Default::default(),
            monster: Default::default(),
            missiles: Default::default(),
            objects: Default::default(),
            towners: Default::default(),
            stash: Default::default(),
            items: Default::default(),
            multi: Default::default(),
            menu: Default::default(),
            error: Default::default(),
            storm_net: Default::default(),
            lighting: Default::default(),
            stores: Default::default(),
            quests: Default::default(),
            inv: Default::default(),
            nthread: Default::default(),
            portal: Default::default(),
            trigs: Default::default(),
            help: Default::default(),
            msg: Default::default(),
            loadsave: Default::default(),
            pfile: Default::default(),
            sync: Default::default(),
            tmsg: Default::default(),
            plrmsg: Default::default(),
            chatlog: Default::default(),
            crt_rand: Default::default(),
            minitext: Default::default(),
            crypt: Default::default(),
            events: Default::default(),
            interfac: Default::default(),
            doom: Default::default(),
            scrollrt: Default::default(),
            dead: Default::default(),
            info_box: Default::default(),
            monhealthbar: Default::default(),
            xpbar: Default::default(),
            itemlabels: Default::default(),
            floatingnumbers: Default::default(),
            panels: Default::default(),
            gmenu: Default::default(),
            gamemenu: Default::default(),
            modifier_hints: Default::default(),
            themes: Default::default(),
            drlg_l1: Default::default(),
            drlg_l4: Default::default(),
            movie: Default::default(),
        }
    }
}
