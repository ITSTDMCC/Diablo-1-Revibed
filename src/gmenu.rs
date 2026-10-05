//! `Source/gmenu.cpp`: the in-game menu widget.

use crate::control::get_main_panel;
use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::render::clx_render::clx_draw;
use crate::engine::render::text_render::{draw_string, draw_string_at, get_line_width, GameFontTables, UiFlags};
use crate::engine::surface::{Rect, Surface};
use crate::platform::events::keys::*;
use crate::utils::language::tr;

pub const GMENU_SLIDER: u32 = 0x40000000;
pub const GMENU_ENABLED: u32 = 0x80000000;

/// `TMenuItem`
#[derive(Clone, Debug)]
pub struct TMenuItem {
    pub dw_flags: u32,
    /// `pszStr` (an untranslated string, or an already translated one)
    pub psz_str: Option<String>,
    /// `fnMenu`; `None` terminates a menu.
    pub fn_menu: Option<fn(&mut Ctx, bool)>,
}

impl TMenuItem {
    pub fn new(dw_flags: u32, psz_str: Option<&str>, fn_menu: Option<fn(&mut Ctx, bool)>) -> TMenuItem {
        TMenuItem { dw_flags, psz_str: psz_str.map(str::to_string), fn_menu }
    }
    pub fn enabled(&self) -> bool {
        (self.dw_flags & GMENU_ENABLED) != 0
    }
    pub fn is_slider(&self) -> bool {
        (self.dw_flags & GMENU_SLIDER) != 0
    }
    pub fn slider_step(&self) -> u16 {
        (self.dw_flags & 0xFFF) as u16
    }
    pub fn set_slider_step(&mut self, step: u16) {
        self.dw_flags &= 0xFFFFF000;
        self.dw_flags |= step as u32;
    }
    pub fn slider_steps(&self) -> u16 {
        ((self.dw_flags & 0xFFF000) >> 12) as u16
    }
    pub fn set_slider_steps(&mut self, steps: u16) {
        self.dw_flags |= ((steps as u32) << 12) & 0xFFF000;
    }
    pub fn add_flags(&mut self, flags: u32) {
        self.dw_flags |= flags;
    }
    pub fn remove_flags(&mut self, flags: u32) {
        self.dw_flags &= !flags;
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            self.add_flags(GMENU_ENABLED);
        } else {
            self.remove_flags(GMENU_ENABLED);
        }
    }
}

/// Which of the game menus (`sgSingleMenu`, `sgMultiMenu`, `sgOptionsMenu` in gamemenu.cpp) is
/// shown; stands in for the `TMenuItem *` the original passes around.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuId {
    Single,
    Multi,
    Options,
}

/// The menu items of `id`.
pub fn menu_items(ctx: &mut Ctx, id: MenuId) -> &mut Vec<TMenuItem> {
    match id {
        MenuId::Single => &mut ctx.gamemenu.sg_single_menu,
        MenuId::Multi => &mut ctx.gamemenu.sg_multi_menu,
        MenuId::Options => &mut ctx.gamemenu.sg_options_menu,
    }
}

/// Globals of gmenu.cpp.
#[derive(Default)]
pub struct GmenuState {
    optbar_cel: Option<ClxSpriteList>,
    pent_spin_cel: Option<ClxSpriteList>,
    option_cel: Option<ClxSpriteList>,
    sgp_logo: Option<ClxSpriteList>,
    is_dragging_slider: bool,
    /// `sgpCurrItem` (index into the current menu)
    sgp_curr_item: Option<usize>,
    logo_anim_tick: u32,
    logo_anim_frame: u8,
    gmenu_current_option: Option<fn(&mut Ctx)>,
    sg_current_menu_idx: i32,
    /// `sgpCurrentMenu`
    pub sgp_current_menu: Option<MenuId>,
    /// `repeater` (function-local static of `GameMenuMove`)
    repeater: crate::controls::controller::AxisDirectionRepeater,
}

// Width of the slider menu item, including the label.
const SLIDER_ITEM_WIDTH: i32 = 490;
// Horizontal dimensions of the slider value
const SLIDER_VALUE_BOX_LEFT: i32 = 16 + SLIDER_ITEM_WIDTH / 2;
const SLIDER_VALUE_BOX_WIDTH: i32 = 287;
const SLIDER_VALUE_BORDER_WIDTH: i32 = 2;
const SLIDER_VALUE_LEFT: i32 = SLIDER_VALUE_BOX_LEFT + SLIDER_VALUE_BORDER_WIDTH;
const SLIDER_VALUE_WIDTH: i32 = SLIDER_VALUE_BOX_WIDTH - 2 * SLIDER_VALUE_BORDER_WIDTH;
const SLIDER_VALUE_HEIGHT: i32 = 29;
const SLIDER_VALUE_PADDING_TOP: i32 = 10;
const SLIDER_MARKER_WIDTH: i32 = 27;
const SLIDER_FILL_MIN: i32 = SLIDER_MARKER_WIDTH / 2;
const SLIDER_FILL_MAX: i32 = SLIDER_VALUE_WIDTH - SLIDER_MARKER_WIDTH / 2 - 1;
const GMENU_TOP: i32 = 117;
const GMENU_ITEM_HEIGHT: i32 = 45;

/// The current item of the current menu.
fn curr_item(ctx: &mut Ctx) -> &mut TMenuItem {
    let id = ctx.gmenu.sgp_current_menu.expect("sgpCurrentMenu");
    let i = ctx.gmenu.sgp_curr_item.expect("sgpCurrItem");
    &mut menu_items(ctx, id)[i]
}

/// Calls the current item's `fnMenu`.
fn call_curr_item(ctx: &mut Ctx, activate: bool) {
    let f = curr_item(ctx).fn_menu.expect("fnMenu");
    f(ctx, activate);
}

/// Original: `GmenuUpDown` (gmenu.cpp).
// @port gmenu.cpp|devilution::GmenuUpDown(bool isDown) sha=fa26b5d9e5b5
fn gmenu_up_down(ctx: &mut Ctx, is_down: bool) {
    if ctx.gmenu.sgp_curr_item.is_none() {
        return;
    }
    ctx.gmenu.is_dragging_slider = false;
    let mut i = ctx.gmenu.sg_current_menu_idx;
    if ctx.gmenu.sg_current_menu_idx != 0 {
        let id = ctx.gmenu.sgp_current_menu.expect("sgpCurrentMenu");
        while i != 0 {
            i -= 1;
            let mut cur = ctx.gmenu.sgp_curr_item.unwrap();
            if is_down {
                cur += 1;
                if menu_items(ctx, id)[cur].fn_menu.is_none() {
                    cur = 0;
                }
            } else {
                if cur == 0 {
                    cur = ctx.gmenu.sg_current_menu_idx as usize;
                }
                cur -= 1;
            }
            ctx.gmenu.sgp_curr_item = Some(cur);
            if menu_items(ctx, id)[cur].enabled() {
                if i != 0 {
                    crate::effects::play_sfx(ctx, crate::effects::IS_TITLEMOV);
                }
                return;
            }
        }
    }
}

/// Original: `GmenuLeftRight` (gmenu.cpp).
// @port gmenu.cpp|devilution::GmenuLeftRight(bool isRight) sha=131c4f8f4fd1
fn gmenu_left_right(ctx: &mut Ctx, is_right: bool) {
    if !curr_item(ctx).is_slider() {
        return;
    }
    let mut step = curr_item(ctx).slider_step();
    if is_right {
        if step == curr_item(ctx).slider_steps() {
            return;
        }
        step += 1;
    } else {
        if step == 0 {
            return;
        }
        step -= 1;
    }
    curr_item(ctx).set_slider_step(step);
    call_curr_item(ctx, false);
}

/// Original: `GmenuGetLineWidth` (gmenu.cpp).
// @port gmenu.cpp|devilution::GmenuGetLineWidth(TMenuItem *pItem) sha=007bc52be2ca
fn gmenu_get_line_width(ctx: &mut Ctx, item: &TMenuItem) -> i32 {
    if item.is_slider() {
        return SLIDER_ITEM_WIDTH;
    }
    let s = tr(item.psz_str.as_deref().unwrap_or(""));
    get_line_width(ctx, &s, GameFontTables::GameFont46, 2, None)
}

/// Original: `GmenuDrawMenuItem` (gmenu.cpp).
// @port gmenu.cpp|devilution::GmenuDrawMenuItem(const Surface &out, TMenuItem *pItem, int y) sha=09300a70573b
fn gmenu_draw_menu_item(ctx: &mut Ctx, out: &Surface, index: usize, y: i32) {
    let id = ctx.gmenu.sgp_current_menu.expect("sgpCurrentMenu");
    let item = menu_items(ctx, id)[index].clone();
    let w = gmenu_get_line_width(ctx, &item);
    if item.is_slider() {
        let ui_position_x = crate::utils::display::get_ui_rectangle(ctx).x;
        clx_draw(out, (SLIDER_VALUE_BOX_LEFT + ui_position_x, y + 40), &ctx.gmenu.optbar_cel.as_ref().expect("optbar_cel").get(0));
        let step = (item.dw_flags & 0xFFF) as i32;
        let steps = (item.slider_steps().max(2)) as i32;
        let pos = (SLIDER_FILL_MIN + step * (SLIDER_FILL_MAX - SLIDER_FILL_MIN) / steps) as u16 as i32;
        // SDL_FillRect: clipped to the surface.
        let (x0, y0) = (SLIDER_VALUE_LEFT + ui_position_x, y + SLIDER_VALUE_PADDING_TOP);
        for yy in y0.max(0)..(y0 + SLIDER_VALUE_HEIGHT).min(out.h()) {
            for xx in x0.max(0)..(x0 + pos).min(out.w()) {
                out.put(xx, yy, 205);
            }
        }
        clx_draw(
            out,
            (SLIDER_VALUE_LEFT + pos - SLIDER_MARKER_WIDTH / 2 + ui_position_x, y + SLIDER_VALUE_PADDING_TOP + SLIDER_VALUE_HEIGHT - 1),
            &ctx.gmenu.option_cel.as_ref().expect("option_cel").get(0),
        );
    }
    let x = (ctx.dx.gn_screen_width - w) / 2;
    let style = if item.enabled() { UiFlags::COLOR_GOLD } else { UiFlags::COLOR_BLACK };
    let s = tr(item.psz_str.as_deref().unwrap_or(""));
    draw_string_at(ctx, out, &s, (x, y), style | UiFlags::FONT_SIZE_46, 2, -1);
    if ctx.gmenu.sgp_curr_item == Some(index) {
        let spin = crate::engine::render::text_render::pent_spn2_spin(ctx) as usize;
        let sprite = ctx.gmenu.pent_spin_cel.as_ref().expect("PentSpin_cel").get(spin);
        clx_draw(out, (x - 54, y + 51), &sprite);
        clx_draw(out, (x + 4 + w, y + 51), &sprite);
    }
}

/// Original: `GameMenuMove` (gmenu.cpp).
// @port gmenu.cpp|devilution::GameMenuMove() sha=b7cf2a06b9c0
fn game_menu_move(ctx: &mut Ctx) {
    use crate::controls::controller::*;
    let dir = get_left_stick_or_dpad_direction(ctx, false);
    let mut repeater = std::mem::take(&mut ctx.gmenu.repeater);
    let move_dir = repeater.get(ctx.platform.ticks(), dir);
    ctx.gmenu.repeater = repeater;
    if move_dir.x != AxisDirectionX::None {
        gmenu_left_right(ctx, move_dir.x == AxisDirectionX::Right);
    }
    if move_dir.y != AxisDirectionY::None {
        gmenu_up_down(ctx, move_dir.y == AxisDirectionY::Down);
    }
}

/// Original: `GmenuMouseIsOverSlider` (gmenu.cpp).
// @port gmenu.cpp|devilution::GmenuMouseIsOverSlider() sha=76b5abbfe4d0
fn gmenu_mouse_is_over_slider(ctx: &Ctx) -> bool {
    let ui_position_x = crate::utils::display::get_ui_rectangle(ctx).x;
    let mx = ctx.diablo.mouse_position.0;
    if mx < SLIDER_VALUE_LEFT + ui_position_x {
        return false;
    }
    if mx >= SLIDER_VALUE_LEFT + SLIDER_VALUE_WIDTH + ui_position_x {
        return false;
    }
    true
}

/// Original: `GmenuGetSliderFill` (gmenu.cpp).
// @port gmenu.cpp|devilution::GmenuGetSliderFill() sha=df862df46885
fn gmenu_get_slider_fill(ctx: &Ctx) -> i32 {
    (ctx.diablo.mouse_position.0 - SLIDER_VALUE_LEFT - crate::utils::display::get_ui_rectangle(ctx).x).clamp(SLIDER_FILL_MIN, SLIDER_FILL_MAX)
}

/// Original: `devilution::gmenu_draw_pause` (gmenu.cpp).
// @port gmenu.cpp|devilution::gmenu_draw_pause(const Surface &out) sha=906297f4fa87
pub fn gmenu_draw_pause(ctx: &mut Ctx, out: &Surface) {
    if ctx.gendung.leveltype != crate::levels::gendung::DungeonType::Town {
        crate::control::red_back(ctx, out);
    }
    if ctx.gmenu.sgp_current_menu.is_none() {
        ctx.scrollrt.LightTableIndex = 0;
        let rect = Rect::new(0, 0, ctx.dx.gn_screen_width, get_main_panel(ctx).y);
        draw_string(ctx, out, &tr("Pause"), rect, UiFlags::FONT_SIZE_46 | UiFlags::COLOR_GOLD | UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER, 2, -1);
    }
}

/// Original: `devilution::FreeGMenu` (gmenu.cpp).
// @port gmenu.cpp|devilution::FreeGMenu() sha=45f4881c34f9
pub fn free_gmenu(ctx: &mut Ctx) {
    ctx.gmenu.sgp_logo = None;
    ctx.gmenu.pent_spin_cel = None;
    ctx.gmenu.option_cel = None;
    ctx.gmenu.optbar_cel = None;
}

/// Original: `devilution::gmenu_init_menu` (gmenu.cpp).
// @port gmenu.cpp|devilution::gmenu_init_menu() sha=0efa57d381d8
pub fn gmenu_init_menu(ctx: &mut Ctx) {
    let g = &mut ctx.gmenu;
    g.logo_anim_frame = 0;
    g.sgp_current_menu = None;
    g.sgp_curr_item = None;
    g.gmenu_current_option = None;
    g.sg_current_menu_idx = 0;
    g.is_dragging_slider = false;
    if ctx.diablo.headless_mode {
        return;
    }
    use crate::engine::load_sprites::load_cel;
    ctx.gmenu.sgp_logo = Some(if ctx.init.gb_is_hellfire { load_cel(ctx, "data\\hf_logo3", 430) } else { load_cel(ctx, "data\\diabsmal", 296) });
    ctx.gmenu.pent_spin_cel = Some(load_cel(ctx, "data\\pentspin", 48));
    ctx.gmenu.option_cel = Some(load_cel(ctx, "data\\option", SLIDER_MARKER_WIDTH as u16));
    ctx.gmenu.optbar_cel = Some(load_cel(ctx, "data\\optbar", SLIDER_VALUE_BOX_WIDTH as u16));
}

/// Original: `devilution::gmenu_is_active` (gmenu.cpp).
// @port gmenu.cpp|devilution::gmenu_is_active() sha=7fb7d8109c04
pub fn gmenu_is_active(ctx: &Ctx) -> bool {
    ctx.gmenu.sgp_current_menu.is_some()
}

/// Original: `devilution::gmenu_set_items` (gmenu.cpp).
// @port gmenu.cpp|devilution::gmenu_set_items(TMenuItem *pItem, void (*gmFunc)()) sha=9a266eb33e0e
pub fn gmenu_set_items(ctx: &mut Ctx, item: Option<MenuId>, gm_func: Option<fn(&mut Ctx)>) {
    ctx.diablo.pause_mode = 0;
    ctx.gmenu.is_dragging_slider = false;
    ctx.gmenu.sgp_current_menu = item;
    ctx.gmenu.gmenu_current_option = gm_func;
    if let Some(f) = gm_func {
        f(ctx);
    }
    ctx.gmenu.sg_current_menu_idx = 0;
    if let Some(id) = item {
        ctx.gmenu.sg_current_menu_idx = menu_items(ctx, id).iter().take_while(|i| i.fn_menu.is_some()).count() as i32;
    }
    // BUGFIX: OOB access when sgCurrentMenuIdx is 0; should be set to NULL instead. (fixed)
    let n = ctx.gmenu.sg_current_menu_idx;
    ctx.gmenu.sgp_curr_item = if n > 0 { Some(n as usize - 1) } else { None };
    gmenu_up_down(ctx, true);
    if ctx.gmenu.sgp_current_menu.is_none() {
        crate::options::save_options(ctx);
    }
}

/// Original: `devilution::gmenu_draw` (gmenu.cpp).
// @port gmenu.cpp|devilution::gmenu_draw(const Surface &out) sha=eae75bc2ea19
pub fn gmenu_draw(ctx: &mut Ctx, out: &Surface) {
    let Some(id) = ctx.gmenu.sgp_current_menu else {
        return;
    };
    game_menu_move(ctx);
    if let Some(f) = ctx.gmenu.gmenu_current_option {
        f(ctx);
    }
    if ctx.init.gb_is_hellfire {
        let ticks = ctx.platform.ticks();
        if ticks.wrapping_sub(ctx.gmenu.logo_anim_tick) as i32 > 25 {
            ctx.gmenu.logo_anim_frame += 1;
            if ctx.gmenu.logo_anim_frame >= 16 {
                ctx.gmenu.logo_anim_frame = 0;
            }
            ctx.gmenu.logo_anim_tick = ticks;
        }
    }
    let ui_position_y = crate::utils::display::get_ui_rectangle(ctx).y;
    let sprite = ctx.gmenu.sgp_logo.as_ref().expect("sgpLogo").get(ctx.gmenu.logo_anim_frame as usize);
    clx_draw(out, ((ctx.dx.gn_screen_width - sprite.width() as i32) / 2, 102 + ui_position_y), &sprite);
    let mut y = 110 + ui_position_y;
    let mut i = 0;
    while menu_items(ctx, id)[i].fn_menu.is_some() {
        gmenu_draw_menu_item(ctx, out, i, y);
        i += 1;
        y += GMENU_ITEM_HEIGHT;
    }
}

/// Original: `devilution::gmenu_presskeys` (gmenu.cpp).
// @port gmenu.cpp|devilution::gmenu_presskeys(SDL_Keycode vkey) sha=02e2781dcc80
pub fn gmenu_presskeys(ctx: &mut Ctx, vkey: i32) -> bool {
    if ctx.gmenu.sgp_current_menu.is_none() {
        return false;
    }
    match vkey {
        SDLK_KP_ENTER | SDLK_RETURN => {
            if curr_item(ctx).enabled() {
                crate::effects::play_sfx(ctx, crate::effects::IS_TITLEMOV);
                call_curr_item(ctx, true);
            }
        }
        SDLK_ESCAPE => {
            crate::effects::play_sfx(ctx, crate::effects::IS_TITLEMOV);
            gmenu_set_items(ctx, None, None);
        }
        SDLK_SPACE => return false,
        SDLK_LEFT => gmenu_left_right(ctx, false),
        SDLK_RIGHT => gmenu_left_right(ctx, true),
        SDLK_UP => gmenu_up_down(ctx, false),
        SDLK_DOWN => gmenu_up_down(ctx, true),
        _ => {}
    }
    true
}

/// Original: `devilution::gmenu_on_mouse_move` (gmenu.cpp).
// @port gmenu.cpp|devilution::gmenu_on_mouse_move() sha=ac2c84524ae8
pub fn gmenu_on_mouse_move(ctx: &mut Ctx) -> bool {
    if !ctx.gmenu.is_dragging_slider {
        return false;
    }
    let fill = gmenu_get_slider_fill(ctx);
    let steps = curr_item(ctx).slider_steps() as i32;
    let step = (steps * (fill - SLIDER_FILL_MIN) / (SLIDER_FILL_MAX - SLIDER_FILL_MIN)) as u16;
    curr_item(ctx).set_slider_step(step);
    call_curr_item(ctx, false);
    true
}

/// Original: `devilution::gmenu_left_mouse` (gmenu.cpp).
// @port gmenu.cpp|devilution::gmenu_left_mouse(bool isDown) sha=e2c148e97542
pub fn gmenu_left_mouse(ctx: &mut Ctx, is_down: bool) -> bool {
    if !is_down {
        if ctx.gmenu.is_dragging_slider {
            ctx.gmenu.is_dragging_slider = false;
            return true;
        }
        return false;
    }
    let Some(id) = ctx.gmenu.sgp_current_menu else {
        return false;
    };
    let ui_position = crate::utils::display::get_ui_rectangle(ctx);
    let (mx, my) = ctx.diablo.mouse_position;
    if my >= get_main_panel(ctx).y {
        return false;
    }
    if my - (GMENU_TOP + ui_position.y) < 0 {
        return true;
    }
    let i = (my - (GMENU_TOP + ui_position.y)) / GMENU_ITEM_HEIGHT;
    if i >= ctx.gmenu.sg_current_menu_idx {
        return true;
    }
    let item = menu_items(ctx, id)[i as usize].clone();
    if !item.enabled() {
        return true;
    }
    let w = gmenu_get_line_width(ctx, &item);
    let screen_width = crate::utils::display::get_screen_width(ctx) as u16 as i32;
    if mx < screen_width / 2 - w / 2 {
        return true;
    }
    if mx > screen_width / 2 + w / 2 {
        return true;
    }
    ctx.gmenu.sgp_curr_item = Some(i as usize);
    crate::effects::play_sfx(ctx, crate::effects::IS_TITLEMOV);
    if item.is_slider() {
        ctx.gmenu.is_dragging_slider = gmenu_mouse_is_over_slider(ctx);
        gmenu_on_mouse_move(ctx);
    } else {
        call_curr_item(ctx, true);
    }
    true
}

/// Original: `devilution::gmenu_slider_set` (gmenu.cpp).
// @port gmenu.cpp|devilution::gmenu_slider_set(TMenuItem *pItem, int min, int max, int value) sha=5bdd1f6b09f3
pub fn gmenu_slider_set(item: &mut TMenuItem, min: i32, max: i32, value: i32) {
    let n_steps = item.slider_steps().max(2) as i32;
    item.set_slider_step((((max - min - 1) / 2 + (value - min) * n_steps) / (max - min)) as u16);
}

/// Original: `devilution::gmenu_slider_get` (gmenu.cpp).
// @port gmenu.cpp|devilution::gmenu_slider_get(TMenuItem *pItem, int min, int max) sha=aaaf8de5dd23
pub fn gmenu_slider_get(item: &TMenuItem, min: i32, max: i32) -> i32 {
    let step = item.slider_step() as i32;
    let steps = item.slider_steps().max(2) as i32;
    min + (step * (max - min) + (steps - 1) / 2) / steps
}

/// Original: `devilution::gmenu_slider_steps` (gmenu.cpp).
// @port gmenu.cpp|devilution::gmenu_slider_steps(TMenuItem *pItem, int steps) sha=79fef71e22d3
pub fn gmenu_slider_steps(item: &mut TMenuItem, steps: i32) {
    item.dw_flags &= 0xFF000FFF;
    item.set_slider_steps(steps as u16);
}
