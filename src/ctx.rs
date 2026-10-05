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
    pub player: crate::player::PlayerState,
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
            player: Default::default(),
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
        }
    }
}
