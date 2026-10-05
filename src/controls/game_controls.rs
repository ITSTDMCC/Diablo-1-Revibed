//! `Source/controls/game_controls.cpp`

#[allow(unused_imports)]
use crate::ctx::Ctx;
use super::controller_buttons::ControllerButton;

/// `GameActionType`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum GameActionType {
    #[default]
    None,
    UseHealthPotion,
    UseManaPotion,
    PrimaryAction,
    SecondaryAction,
    CastSpell,
    ToggleInventory,
    ToggleCharacterInfo,
    ToggleQuickSpellMenu,
    ToggleSpellBook,
    ToggleQuestLog,
    SendKey,
}

/// Original: `devilution::ToString` (controls/controller_buttons.cpp).
// @port controls/controller_buttons.cpp|devilution::ToString(ControllerButton button) sha=b417d11f46c9
pub fn to_string(ctx: &Ctx, button: ControllerButton) -> String {
    use super::controller_buttons::*;
    match ctx.controls.gamepad_type {
        GamepadLayout::PlayStation => to_play_station_icon(button),
        GamepadLayout::Nintendo => to_nintendo_icon(button),
        GamepadLayout::Xbox => to_xbox_icon(button),
        GamepadLayout::Generic => to_generic_button_text(button),
    }
    .to_string()
}

/// `GamepadLayout`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GamepadLayout {
    #[default]
    Generic,
    Nintendo,
    PlayStation,
    Xbox,
}

/// Original: `devilution::TranslateTo` (controls/game_controls.cpp), with `GamepadType`.
// @port controls/game_controls.cpp|devilution::TranslateTo(GamepadLayout layout, ControllerButton button) sha=9a97d638d084
pub fn translate_to(ctx: &Ctx, button: ControllerButton) -> ControllerButton {
    if ctx.controls.gamepad_type != GamepadLayout::Nintendo {
        return button;
    }
    match button {
        ControllerButton::ButtonA => ControllerButton::ButtonB,
        ControllerButton::ButtonB => ControllerButton::ButtonA,
        ControllerButton::ButtonX => ControllerButton::ButtonY,
        ControllerButton::ButtonY => ControllerButton::ButtonX,
        b => b,
    }
}

/// Original: `devilution::SkipsMovie` (controls/game_controls.cpp).
// @port controls/game_controls.cpp|devilution::SkipsMovie(ControllerButtonEvent ctrlEvent) sha=66842f51137e
pub fn skips_movie(e: super::controller::ControllerButtonEvent) -> bool {
    matches!(e.button, ControllerButton::ButtonA | ControllerButton::ButtonB | ControllerButton::ButtonStart | ControllerButton::ButtonBack)
}

/// Original: `devilution::IsSimulatedMouseClickBinding` (controls/game_controls.cpp).
// @port controls/game_controls.cpp|devilution::IsSimulatedMouseClickBinding(ControllerButtonEvent ctrlEvent) sha=f3483229192e
pub fn is_simulated_mouse_click_binding(ctx: &Ctx, e: super::controller::ControllerButtonEvent) -> bool {
    if e.button == ControllerButton::None {
        return false;
    }
    if !e.up && e.button == ctx.controls.suppressed_button {
        return false;
    }
    let action = crate::options::padmapper_action_name_triggered_by_button_event(ctx, e.button, e.up);
    matches!(action.as_str(), "LeftMouseClick1" | "LeftMouseClick2" | "RightMouseClick1" | "RightMouseClick2")
}

/// Original: `devilution::GetMoveDirection` (controls/game_controls.cpp).
// @port controls/game_controls.cpp|devilution::GetMoveDirection() sha=29758e684ec2
pub fn get_move_direction(ctx: &Ctx) -> super::controller::AxisDirection {
    super::controller::get_left_stick_or_dpad_direction(ctx, true)
}

/// `GameAction`: a `GameActionType`, and for `SendKey` the key and whether it is released.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct GameAction {
    pub type_: GameActionType,
    /// `send_key.vk_code`
    pub vk_code: u32,
    /// `send_key.up`
    pub up: bool,
}

impl GameAction {
    /// `GameActionSendKey { vk_code, up }`
    fn send_key(vk_code: i32, up: bool) -> GameAction {
        GameAction { type_: GameActionType::SendKey, vk_code: vk_code as u32, up }
    }
}

use crate::platform::events::keys::*;

/// Original: `TranslateControllerButtonToGameMenuKey` (controls/game_controls.cpp).
// @port controls/game_controls.cpp|devilution::TranslateControllerButtonToGameMenuKey(ControllerButton controllerButton) sha=69dd56b9dbab
fn translate_controller_button_to_game_menu_key(ctx: &Ctx, controller_button: ControllerButton) -> i32 {
    use ControllerButton as B;
    match translate_to(ctx, controller_button) {
        B::ButtonA | B::ButtonY => SDLK_RETURN,
        B::ButtonB | B::ButtonBack | B::ButtonStart => SDLK_ESCAPE,
        B::ButtonLeftStick => SDLK_TAB, // Map
        _ => SDLK_UNKNOWN,
    }
}

/// Original: `TranslateControllerButtonToMenuKey` (controls/game_controls.cpp).
// @port controls/game_controls.cpp|devilution::TranslateControllerButtonToMenuKey(ControllerButton controllerButton) sha=f85dec194235
fn translate_controller_button_to_menu_key(ctx: &Ctx, controller_button: ControllerButton) -> i32 {
    use ControllerButton as B;
    match translate_to(ctx, controller_button) {
        B::ButtonA => SDLK_SPACE,
        B::ButtonB | B::ButtonBack | B::ButtonStart => SDLK_ESCAPE,
        B::ButtonY => SDLK_RETURN,
        B::ButtonLeftStick => SDLK_TAB, // Map
        B::ButtonDpadLeft => SDLK_LEFT,
        B::ButtonDpadRight => SDLK_RIGHT,
        B::ButtonDpadUp => SDLK_UP,
        B::ButtonDpadDown => SDLK_DOWN,
        _ => SDLK_UNKNOWN,
    }
}

/// Original: `TranslateControllerButtonToQuestLogKey` (controls/game_controls.cpp).
// @port controls/game_controls.cpp|devilution::TranslateControllerButtonToQuestLogKey(ControllerButton controllerButton) sha=cbecd981b173
fn translate_controller_button_to_quest_log_key(ctx: &Ctx, controller_button: ControllerButton) -> i32 {
    use ControllerButton as B;
    match translate_to(ctx, controller_button) {
        B::ButtonA | B::ButtonY => SDLK_RETURN,
        B::ButtonB => SDLK_SPACE,
        B::ButtonLeftStick => SDLK_TAB, // Map
        _ => SDLK_UNKNOWN,
    }
}

/// Original: `TranslateControllerButtonToSpellbookKey` (controls/game_controls.cpp).
// @port controls/game_controls.cpp|devilution::TranslateControllerButtonToSpellbookKey(ControllerButton controllerButton) sha=60810897d33a
fn translate_controller_button_to_spellbook_key(ctx: &Ctx, controller_button: ControllerButton) -> i32 {
    use ControllerButton as B;
    match translate_to(ctx, controller_button) {
        B::ButtonB => SDLK_SPACE,
        B::ButtonY => SDLK_RETURN,
        B::ButtonLeftStick => SDLK_TAB, // Map
        B::ButtonDpadLeft => SDLK_LEFT,
        B::ButtonDpadRight => SDLK_RIGHT,
        B::ButtonDpadUp => SDLK_UP,
        B::ButtonDpadDown => SDLK_DOWN,
        _ => SDLK_UNKNOWN,
    }
}

/// Original: `GetGameAction` (controls/game_controls.cpp). The virtual gamepad (touch) branch
/// is left out: the port has no touch input.
// @port controls/game_controls.cpp|devilution::GetGameAction(const SDL_Event &event, ControllerButtonEvent ctrlEvent, GameAction *action) sha=96f83be592d9
fn get_game_action(ctx: &Ctx, ctrl_event: super::controller::ControllerButtonEvent, action: &mut GameAction) -> bool {
    let in_game_menu = super::plrctrls::in_game_menu(ctx);
    if ctx.controls.pad_menu_navigator_active || ctx.controls.pad_hotspell_menu_active {
        return false;
    }
    let mut translation = SDLK_UNKNOWN;
    if crate::gmenu::gmenu_is_active(ctx) || !crate::stores::stextflag_is_none(ctx) {
        translation = translate_controller_button_to_game_menu_key(ctx, ctrl_event.button);
    } else if in_game_menu {
        translation = translate_controller_button_to_menu_key(ctx, ctrl_event.button);
    } else if ctx.quests.QuestLogIsOpen {
        translation = translate_controller_button_to_quest_log_key(ctx, ctrl_event.button);
    } else if ctx.control.sbookflag {
        translation = translate_controller_button_to_spellbook_key(ctx, ctrl_event.button);
    }
    if translation != SDLK_UNKNOWN {
        *action = GameAction::send_key(translation, ctrl_event.up);
        return true;
    }
    false
}

/// Original: `PressControllerButton` (controls/game_controls.cpp).
// @port controls/game_controls.cpp|devilution::PressControllerButton(ControllerButton button) sha=63b96cc2181d
fn press_controller_button(ctx: &mut Ctx, button: ControllerButton) {
    use ControllerButton as B;
    if ctx.stash.IsStashOpen {
        match button {
            B::ButtonBack => {
                crate::qol::stash::start_gold_withdraw(ctx);
                return;
            }
            B::ButtonLeftShoulder => {
                ctx.stash.Stash.previous_page(1);
                return;
            }
            B::ButtonRightShoulder => {
                ctx.stash.Stash.next_page(1);
                return;
            }
            _ => {}
        }
    }
    if ctx.controls.pad_hotspell_menu_active {
        let quick_spell_action = |ctx: &mut Ctx, slot: usize| {
            if ctx.control.spselflag {
                crate::panels::spell_list::set_speed_spell(ctx, slot);
                return;
            }
            if !ctx.options.gameplay.quick_cast.get() {
                crate::panels::spell_list::toggle_spell(ctx, slot);
            } else {
                super::plrctrls::quick_cast(ctx, slot);
            }
        };
        match button {
            B::ButtonA => return quick_spell_action(ctx, 2),
            B::ButtonB => return quick_spell_action(ctx, 3),
            B::ButtonX => return quick_spell_action(ctx, 0),
            B::ButtonY => return quick_spell_action(ctx, 1),
            _ => {}
        }
    }
    if ctx.controls.pad_menu_navigator_active {
        match button {
            B::ButtonDpadUp => {
                crate::diablo_game::press_esc_key(ctx);
                ctx.diablo.last_mouse_button_action = crate::diablo::MouseActionType::None;
                ctx.controls.pad_hotspell_menu_active = false;
                ctx.controls.pad_menu_navigator_active = false;
                crate::gamemenu::gamemenu_on(ctx);
                return;
            }
            B::ButtonDpadDown => return crate::control::do_auto_map(ctx),
            B::ButtonDpadLeft => return super::plrctrls::process_game_action(ctx, GameActionType::ToggleCharacterInfo),
            B::ButtonDpadRight => return super::plrctrls::process_game_action(ctx, GameActionType::ToggleInventory),
            B::ButtonA => return super::plrctrls::process_game_action(ctx, GameActionType::ToggleSpellBook),
            B::ButtonB => return,
            B::ButtonX => return super::plrctrls::process_game_action(ctx, GameActionType::ToggleQuestLog),
            B::ButtonY => return, // zoom toggle on 3DS only
            _ => {}
        }
    }
    crate::options::padmapper_button_pressed(ctx, button);
}

/// Original: `devilution::HandleControllerButtonEvent` (controls/game_controls.cpp).
// @port controls/game_controls.cpp|devilution::HandleControllerButtonEvent(const SDL_Event &event, const ControllerButtonEvent ctrlEvent, GameAction &action) sha=8956ec338e68
pub fn handle_controller_button_event(
    ctx: &mut Ctx,
    event: &crate::platform::events::Event,
    ctrl_event: super::controller::ControllerButtonEvent,
    action: &mut GameAction,
) -> bool {
    if ctrl_event.button == ControllerButton::Ignore {
        return false;
    }
    let handled = handle_controller_button_event_body(ctx, event, ctrl_event, action);
    // ButtonReleaser: runs when the original's function returns.
    if ctrl_event.up {
        crate::options::padmapper_button_released(ctx, ctrl_event.button, false);
    }
    handled
}

fn handle_controller_button_event_body(
    ctx: &mut Ctx,
    event: &crate::platform::events::Event,
    ctrl_event: super::controller::ControllerButtonEvent,
    action: &mut GameAction,
) -> bool {
    let is_gamepad_motion = super::controller::is_controller_motion(event);
    if !is_gamepad_motion {
        super::controller::simulate_right_stick_with_padmapper(ctx, ctrl_event);
    }
    super::plrctrls::detect_input_method(ctx, event, ctrl_event);
    if is_gamepad_motion {
        return true;
    }
    if ctrl_event.button != ControllerButton::None && ctrl_event.button == ctx.controls.suppressed_button {
        if !ctrl_event.up {
            return true;
        }
        ctx.controls.suppressed_button = ControllerButton::None;
    }
    if ctrl_event.up && !crate::options::padmapper_action_name_triggered_by_button_event(ctx, ctrl_event.button, ctrl_event.up).is_empty() {
        // Button press may have brought up a menu;
        // do not confuse release of that button with intent to interact with the menu
        crate::options::padmapper_button_released(ctx, ctrl_event.button, true);
        true
    } else if get_game_action(ctx, ctrl_event, action) {
        super::plrctrls::process_game_action(ctx, action.type_);
        true
    } else if ctrl_event.button != ControllerButton::None {
        if !ctrl_event.up {
            press_controller_button(ctx, ctrl_event.button);
        }
        true
    } else {
        false
    }
}
