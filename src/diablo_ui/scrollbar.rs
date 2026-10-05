//! `Source/DiabloUI/scrollbar.h` / `scrollbar.cpp`: the list scroll bar.

use crate::ctx::Ctx;
use crate::engine::clx_sprite::{ClxSprite, ClxSpriteList};
use crate::engine::surface::Rect;

pub const SCROLL_BAR_BG_WIDTH: i32 = 25;
pub const SCROLL_BAR_ARROW_WIDTH: i32 = 25;

/// `ScrollBarArrowFrame`
pub const SCROLL_BAR_ARROW_FRAME_UP_ACTIVE: usize = 0;
pub const SCROLL_BAR_ARROW_FRAME_UP: usize = 1;
pub const SCROLL_BAR_ARROW_FRAME_DOWN_ACTIVE: usize = 2;
pub const SCROLL_BAR_ARROW_FRAME_DOWN: usize = 3;

/// Globals of scrollbar.cpp.
#[derive(Default)]
pub struct ScrollBarArt {
    pub art_scroll_bar_background: Option<ClxSpriteList>,
    pub art_scroll_bar_thumb: Option<ClxSpriteList>,
    pub art_scroll_bar_arrow: Option<ClxSpriteList>,
}

fn arrow_h(arrow: &ClxSpriteList) -> i32 {
    arrow.get(0).height() as i32
}

/// Original: `devilution::UpArrowRect` (DiabloUI/scrollbar.h).
// @port DiabloUI/scrollbar.h|devilution::UpArrowRect(const UiScrollbar &bar) sha=f57a10443a00
pub fn up_arrow_rect(r: Rect, arrow: &ClxSpriteList) -> Rect {
    Rect::new(r.x, r.y, SCROLL_BAR_ARROW_WIDTH, arrow_h(arrow))
}

/// Original: `devilution::DownArrowRect` (DiabloUI/scrollbar.h).
// @port DiabloUI/scrollbar.h|devilution::DownArrowRect(const UiScrollbar &bar) sha=ff87f4c15b56
pub fn down_arrow_rect(r: Rect, arrow: &ClxSpriteList) -> Rect {
    Rect::new(r.x, r.y + r.h - arrow_h(arrow), SCROLL_BAR_ARROW_WIDTH, arrow_h(arrow))
}

/// Original: `devilution::BarHeight` (DiabloUI/scrollbar.h). Uint16 in the original.
// @port DiabloUI/scrollbar.h|devilution::BarHeight(const UiScrollbar &bar) sha=c8328021c51c
pub fn bar_height(r: Rect, arrow: &ClxSpriteList) -> i32 {
    (r.h - 2 * arrow_h(arrow)) as u16 as i32
}

/// Original: `devilution::BarRect` (DiabloUI/scrollbar.h).
// @port DiabloUI/scrollbar.h|devilution::BarRect(const UiScrollbar &bar) sha=a1c70bd7074f
pub fn bar_rect(r: Rect, arrow: &ClxSpriteList) -> Rect {
    Rect::new(r.x, r.y + arrow_h(arrow), SCROLL_BAR_ARROW_WIDTH, bar_height(r, arrow))
}

/// Original: `devilution::ThumbRect` (DiabloUI/scrollbar.h).
// @port DiabloUI/scrollbar.h|devilution::ThumbRect(const UiScrollbar &bar, size_t selectedIndex, size_t numItems) sha=f31aaa5c0438
pub fn thumb_rect(r: Rect, arrow: &ClxSpriteList, thumb: &ClxSprite, selected_index: usize, num_items: usize) -> Rect {
    const THUMB_OFFSET_X: i32 = 3;
    let thumb_max_y = bar_height(r, arrow) - thumb.height() as i32;
    // size_t arithmetic in the original (thumbMaxY is non-negative for the shipped art)
    let thumb_y = (selected_index.wrapping_mul(thumb_max_y as usize) / (num_items - 1)) as i32;
    Rect::new(r.x + THUMB_OFFSET_X, r.y + arrow_h(arrow) + thumb_y, r.w - THUMB_OFFSET_X, thumb.height() as i32)
}

/// Original: `devilution::LoadScrollBar` (DiabloUI/scrollbar.cpp).
// @port DiabloUI/scrollbar.cpp|devilution::LoadScrollBar() sha=3648ef32f4ec
pub fn load_scroll_bar(ctx: &mut Ctx) {
    use crate::engine::load_sprites::{load_pcx, load_pcx_sprite_list};
    ctx.diablo_ui.scrollbar.art_scroll_bar_background = load_pcx(ctx, "ui_art\\sb_bg", None, None, true);
    ctx.diablo_ui.scrollbar.art_scroll_bar_thumb = load_pcx(ctx, "ui_art\\sb_thumb", None, None, true);
    ctx.diablo_ui.scrollbar.art_scroll_bar_arrow = load_pcx_sprite_list(ctx, "ui_art\\sb_arrow", 4, None, None, true);
}

/// Original: `devilution::UnloadScrollBar` (DiabloUI/scrollbar.cpp).
// @port DiabloUI/scrollbar.cpp|devilution::UnloadScrollBar() sha=8755058b0cb9
pub fn unload_scroll_bar(ctx: &mut Ctx) {
    ctx.diablo_ui.scrollbar = ScrollBarArt::default();
}
