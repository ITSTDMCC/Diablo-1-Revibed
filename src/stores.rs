//! `Source/stores.cpp`: stores and towner dialogs.

use crate::ctx::Ctx;
use crate::engine::backbuffer_state::{redraw_component, PanelDrawComponent};
use crate::engine::geometry::{Point, Size};
use crate::engine::render::clx_render::{clx_draw, clx_draw_trn};
use crate::engine::render::text_render::{draw_string, get_line_width, GameFontTables, UiFlags};
use crate::engine::surface::{Rect, Surface};
use crate::enums::*;
use crate::items::Item;
use crate::utils::format_int::format_integer;
use crate::utils::language::{is_small_font_tall, tr};

pub const WITCH_ITEMS: usize = 25;
pub const SMITH_ITEMS: usize = 25;
pub const SMITH_PREMIUM_ITEMS: usize = 15;
pub const STORE_LINES: usize = 104;

/// `STextStruct::Type`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum STextType {
    #[default]
    Label,
    Divider,
    Selectable,
}

/// `STextStruct`
#[derive(Clone, Debug, Default)]
struct STextStruct {
    text: String,
    _sval: i32,
    y: i32,
    flags: UiFlags,
    type_: STextType,
    _sx: u8,
    _syoff: u8,
    curs_id: i32,
    curs_indent: bool,
}

impl STextStruct {
    // @port stores.cpp|devilution::STextStruct::isDivider() sha=863e7106aaf5
    fn is_divider(&self) -> bool {
        self.type_ == STextType::Divider
    }
    // @port stores.cpp|devilution::STextStruct::isSelectable() sha=ac77c2b49e23
    fn is_selectable(&self) -> bool {
        self.type_ == STextType::Selectable
    }
    // @port stores.cpp|devilution::STextStruct::hasText() sha=20b04c1e83e4
    fn has_text(&self) -> bool {
        !self.text.is_empty()
    }
}

/// Globals of stores.cpp.
pub struct StoresState {
    /// `stextflag`
    pub stextflag: TalkID,
    /// `storenumh`
    pub storenumh: i32,
    /// `storehidx`
    pub storehidx: [i8; 48],
    /// `storehold`
    pub storehold: Vec<Item>,
    pub smithitem: Vec<Item>,
    pub numpremium: i32,
    pub premiumlevel: i32,
    pub premiumitems: Vec<Item>,
    pub healitem: Vec<Item>,
    pub witchitem: Vec<Item>,
    pub boylevel: i32,
    pub boyitem: Item,
    /// The current towner being interacted with
    talker: _talker_id,
    /// Is the current dialog full size
    stextsize: bool,
    /// Number of text lines in the current dialog
    stextsmax: i32,
    /// Remember currently selected text line from stext while displaying a dialog
    stextlhold: i32,
    /// Currently selected text line from stext
    stextsel: i32,
    /// Text lines
    stext: Vec<STextStruct>,
    /// Whether to render the player's gold amount in the top left
    render_gold: bool,
    /// Does the current panel have a scrollbar
    stextscrl: bool,
    /// Remember last scoll position
    stextvhold: i32,
    /// Scoll position
    stextsval: i32,
    /// Next scoll position
    stextdown: i32,
    /// Previous scoll position
    stextup: i32,
    /// Count down for the push state of the scroll up button
    stextscrlubtn: i8,
    /// Count down for the push state of the scroll down button
    stextscrldbtn: i8,
    /// Remember current store while displaying a dialog
    stextshold: TalkID,
    /// Temporary item used to hold the the item being traided
    store_item: Item,
}

impl Default for StoresState {
    fn default() -> Self {
        StoresState {
            stextflag: TalkID::None,
            storenumh: 0,
            storehidx: [0; 48],
            storehold: vec![Item::default(); 48],
            smithitem: vec![Item::default(); SMITH_ITEMS],
            numpremium: 0,
            premiumlevel: 0,
            premiumitems: vec![Item::default(); SMITH_PREMIUM_ITEMS],
            healitem: vec![Item::default(); 20],
            witchitem: vec![Item::default(); WITCH_ITEMS],
            boylevel: 0,
            boyitem: Item::default(),
            talker: 0,
            stextsize: false,
            stextsmax: 0,
            stextlhold: 0,
            stextsel: 0,
            stext: vec![STextStruct::default(); STORE_LINES],
            render_gold: false,
            stextscrl: false,
            stextvhold: 0,
            stextsval: 0,
            stextdown: 0,
            stextup: 0,
            stextscrlubtn: 0,
            stextscrldbtn: 0,
            stextshold: TalkID::None,
            store_item: Item::default(),
        }
    }
}

/// `stextflag == TalkID::None`
pub fn stextflag_is_none(ctx: &Ctx) -> bool {
    ctx.stores.stextflag == TalkID::None
}

/// `stextflag = TalkID::None`
pub fn set_stextflag_none(ctx: &mut Ctx) {
    ctx.stores.stextflag = TalkID::None;
}

/// Maps from towner IDs to NPC names.
const TOWNER_NAMES: [&str; 9] = ["Griswold", "Pepin", "", "Ogden", "Cain", "Farnham", "Adria", "Gillian", "Wirt"];

const PADDING_TOP: i32 = 32;

// For most languages, line height is always 12.
// This includes blank lines and divider line.
const SMALL_LINE_HEIGHT: i32 = 12;
const SMALL_TEXT_HEIGHT: i32 = 12;

// For larger small fonts (Chinese and Japanese), text lines are
// taller and overflow.
// We space out blank lines a bit more to give space to 3-line store items.
const LARGE_LINE_HEIGHT: i32 = SMALL_LINE_HEIGHT + 1;
const LARGE_TEXT_HEIGHT: i32 = 18;

fn my_player(ctx: &Ctx) -> usize {
    ctx.players.MyPlayer.expect("MyPlayer")
}

fn fmt1(fmt: &str, arg: &str) -> String {
    match (fmt.find('{'), fmt.find('}')) {
        (Some(a), Some(b)) if b > a => format!("{}{}{}", &fmt[..a], arg, &fmt[b + 1..]),
        _ => fmt.to_string(),
    }
}

/// Original: `BackButtonLine` (stores.cpp): the line index with the Back / Leave button.
// @port stores.cpp|devilution::BackButtonLine() sha=5441c0c173aa
fn back_button_line(ctx: &Ctx) -> i32 {
    if is_small_font_tall(ctx) {
        return if ctx.stores.stextscrl { 21 } else { 20 };
    }
    22
}

// @port stores.cpp|devilution::LineHeight() sha=775df26fbea2
fn line_height(ctx: &Ctx) -> i32 {
    if is_small_font_tall(ctx) {
        LARGE_LINE_HEIGHT
    } else {
        SMALL_LINE_HEIGHT
    }
}

// @port stores.cpp|devilution::TextHeight() sha=028a1f6514a5
fn text_height(ctx: &Ctx) -> i32 {
    if is_small_font_tall(ctx) {
        LARGE_TEXT_HEIGHT
    } else {
        SMALL_TEXT_HEIGHT
    }
}

/// Original: `CalculateLineHeights` (stores.cpp).
// @port stores.cpp|devilution::CalculateLineHeights() sha=e4b125d42048
fn calculate_line_heights(ctx: &mut Ctx) {
    let tall = is_small_font_tall(ctx);
    let stext = &mut ctx.stores.stext;
    stext[0].y = 0;
    if tall {
        for i in 1..STORE_LINES {
            // Space out consecutive text lines, unless they are both selectable (never the case currently).
            if stext[i].has_text() && stext[i - 1].has_text() && !(stext[i].is_selectable() && stext[i - 1].is_selectable()) {
                stext[i].y = stext[i - 1].y + LARGE_TEXT_HEIGHT;
            } else {
                stext[i].y = i as i32 * LARGE_LINE_HEIGHT;
            }
        }
    } else {
        for i in 1..STORE_LINES {
            stext[i].y = i as i32 * SMALL_LINE_HEIGHT;
        }
    }
}

/// Original: `DrawSTextBack` (stores.cpp).
// @port stores.cpp|devilution::DrawSTextBack(const Surface &out) sha=0c9d01cc0191
fn draw_s_text_back(ctx: &Ctx, out: &Surface) {
    let ui = crate::utils::display::get_ui_rectangle(ctx);
    clx_draw(out, (ui.x + 320 + 24, 327 + ui.y), &ctx.info_box.p_s_text_box_cels.as_ref().expect("pSTextBoxCels").get(0));
    crate::engine::draw_half_transparent_rect_to(ctx, out, ui.x + 347, ui.y + 28, 265, 297);
}

/// Original: `DrawSSlider` (stores.cpp).
// @port stores.cpp|devilution::DrawSSlider(const Surface &out, int y1, int y2) sha=8d11f20c131f
fn draw_s_slider(ctx: &Ctx, out: &Surface, y1: i32, y2: i32) {
    let ui = crate::utils::display::get_ui_rectangle(ctx);
    let cels = ctx.info_box.p_s_text_slid_cels.as_ref().expect("pSTextSlidCels");
    let s = &ctx.stores;
    let mut yd1 = y1 * 12 + 44 + ui.y;
    let yd2 = y2 * 12 + 44 + ui.y;
    clx_draw(out, (ui.x + 601, yd1), &cels.get(if s.stextscrlubtn != -1 { 11 } else { 9 }));
    clx_draw(out, (ui.x + 601, yd2), &cels.get(if s.stextscrldbtn != -1 { 10 } else { 8 }));
    yd1 += 12;
    let mut yd3 = yd1;
    while yd3 < yd2 {
        clx_draw(out, (ui.x + 601, yd3), &cels.get(13));
        yd3 += 12;
    }
    yd3 = if s.stextsel == back_button_line(ctx) { s.stextlhold } else { s.stextsel };
    if s.storenumh > 1 {
        yd3 = 1000 * (s.stextsval + ((yd3 - s.stextup) / 4)) / (s.storenumh - 1) * (y2 * 12 - y1 * 12 - 24) / 1000;
    } else {
        yd3 = 0;
    }
    clx_draw(out, (ui.x + 601, (y1 + 1) * 12 + 44 + ui.y + yd3), &cels.get(12));
}

/// Original: `AddSLine` (stores.cpp).
// @port stores.cpp|devilution::AddSLine(size_t y) sha=9de14c36103e
fn add_s_line(ctx: &mut Ctx, y: usize) {
    let e = &mut ctx.stores.stext[y];
    e._sx = 0;
    e._syoff = 0;
    e.text = String::new();
    e.type_ = STextType::Divider;
    e.curs_id = -1;
    e.curs_indent = false;
}

/// Original: `AddSTextVal` (stores.cpp).
// @port stores.cpp|devilution::AddSTextVal(size_t y, int val) sha=2845a8d86d95
fn add_s_text_val(ctx: &mut Ctx, y: usize, val: i32) {
    ctx.stores.stext[y]._sval = val;
}

/// Original: `AddSText` (stores.cpp).
// @port stores.cpp|devilution::AddSText(uint8_t x, size_t y, string_view text, UiFlags flags, bool sel, int cursId = -1, bool cursIndent = false) sha=e4d77cf18145
#[allow(clippy::too_many_arguments)]
fn add_s_text_full(ctx: &mut Ctx, x: u8, y: usize, text: &str, flags: UiFlags, sel: bool, curs_id: i32, curs_indent: bool) {
    let e = &mut ctx.stores.stext[y];
    e._sx = x;
    e._syoff = 0;
    e.text.clear();
    e.text.push_str(text);
    e.flags = flags;
    e.type_ = if sel { STextType::Selectable } else { STextType::Label };
    e.curs_id = curs_id;
    e.curs_indent = curs_indent;
}

fn add_s_text(ctx: &mut Ctx, x: u8, y: usize, text: &str, flags: UiFlags, sel: bool) {
    add_s_text_full(ctx, x, y, text, flags, sel, -1, false);
}

/// Original: `AddOptionsBackButton` (stores.cpp).
// @port stores.cpp|devilution::AddOptionsBackButton() sha=d5736b440006
fn add_options_back_button(ctx: &mut Ctx) {
    let line = back_button_line(ctx) as usize;
    add_s_text(ctx, 0, line, &tr("Back"), UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true);
    ctx.stores.stext[line]._syoff = if is_small_font_tall(ctx) { 0 } else { 6 };
}

/// Original: `AddItemListBackButton` (stores.cpp).
// @port stores.cpp|devilution::AddItemListBackButton(bool selectable = false) sha=2f1fdcfc8123
fn add_item_list_back_button(ctx: &mut Ctx, selectable: bool) {
    let line = back_button_line(ctx) as usize;
    let text = tr("Back");
    if !selectable && is_small_font_tall(ctx) {
        add_s_text(ctx, 0, line, &text, UiFlags::COLOR_WHITE | UiFlags::ALIGN_RIGHT, selectable);
    } else {
        add_s_line(ctx, line - 1);
        add_s_text(ctx, 0, line, &text, UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, selectable);
        ctx.stores.stext[line]._syoff = 6;
    }
}

/// Original: `PrintStoreItem` (stores.cpp).
// @port stores.cpp|devilution::PrintStoreItem(const Item &item, int l, UiFlags flags, bool cursIndent = false) sha=5eb45dde5032
fn print_store_item(ctx: &mut Ctx, item: &Item, mut l: usize, flags: UiFlags, curs_indent: bool) {
    let mut product_line = String::new();
    if item._iIdentified {
        if item._iMagical != ITEM_QUALITY_UNIQUE && item._iPrePower != IPL_INVALID {
            product_line.push_str(&crate::items::print_item_power(item._iPrePower, item));
        }
        if item._iSufPower != IPL_INVALID {
            if !product_line.is_empty() {
                product_line.push_str(&tr(",  "));
            }
            product_line.push_str(&crate::items::print_item_power(item._iSufPower, item));
        }
    }
    if item._iMiscId == IMISC_STAFF && item._iMaxCharges != 0 {
        if !product_line.is_empty() {
            product_line.push_str(&tr(",  "));
        }
        product_line.push_str(&fmt1(&fmt1(&tr("Charges: {:d}/{:d}"), &item._iCharges.to_string()), &item._iMaxCharges.to_string()));
    }
    if !product_line.is_empty() {
        add_s_text_full(ctx, 40, l, &product_line, flags, false, -1, curs_indent);
        l += 1;
        product_line.clear();
    }
    if item._itype != ItemType::Misc {
        if item._iClass == ICLASS_WEAPON {
            product_line = fmt1(&fmt1(&tr("Damage: {:d}-{:d}  "), &item._iMinDam.to_string()), &item._iMaxDam.to_string());
        } else if item._iClass == ICLASS_ARMOR {
            product_line = fmt1(&tr("Armor: {:d}  "), &item._iAC.to_string());
        }
        if item._iMaxDur != crate::items::DUR_INDESTRUCTIBLE && item._iMaxDur != 0 {
            product_line += &fmt1(&fmt1(&tr("Dur: {:d}/{:d},  "), &item._iDurability.to_string()), &item._iMaxDur.to_string());
        } else {
            product_line.push_str(&tr("Indestructible,  "));
        }
    }
    let str_ = item._iMinStr;
    let mag = item._iMinMag;
    let dex = item._iMinDex;
    if str_ == 0 && mag == 0 && dex == 0 {
        product_line.push_str(&tr("No required attributes"));
    } else {
        product_line.push_str(&tr("Required:"));
        if str_ != 0 {
            product_line.push_str(&fmt1(&tr(" {:d} Str"), &str_.to_string()));
        }
        if mag != 0 {
            product_line.push_str(&fmt1(&tr(" {:d} Mag"), &mag.to_string()));
        }
        if dex != 0 {
            product_line.push_str(&fmt1(&tr(" {:d} Dex"), &dex.to_string()));
        }
    }
    add_s_text_full(ctx, 40, l, &product_line, flags, false, -1, curs_indent);
}

/// Original: `StoreAutoPlace` (stores.cpp).
// @port stores.cpp|devilution::StoreAutoPlace(Item &item, bool persistItem) sha=5da480d92cf9
fn store_auto_place(ctx: &mut Ctx, item: &Item, persist_item: bool) -> bool {
    let me = my_player(ctx);
    if crate::inv::auto_equip_enabled(ctx, me, item) && crate::inv::auto_equip(ctx, me, item, persist_item) {
        return true;
    }
    if crate::inv::auto_place_item_in_belt(ctx, me, item, persist_item) {
        return true;
    }
    crate::inv::auto_place_item_in_inventory(ctx, me, item, persist_item)
}

/// Original: `StartSmith` (stores.cpp).
// @port stores.cpp|devilution::StartSmith() sha=7bb238718d04
fn start_smith(ctx: &mut Ctx) {
    ctx.stores.stextsize = false;
    ctx.stores.stextscrl = false;
    let wgc = UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER;
    let wc = UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER;
    add_s_text(ctx, 0, 1, &tr("Welcome to the"), wgc, false);
    add_s_text(ctx, 0, 3, &tr("Blacksmith's shop"), wgc, false);
    add_s_text(ctx, 0, 7, &tr("Would you like to:"), wgc, false);
    add_s_text(ctx, 0, 10, &tr("Talk to Griswold"), UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true);
    add_s_text(ctx, 0, 12, &tr("Buy basic items"), wc, true);
    add_s_text(ctx, 0, 14, &tr("Buy premium items"), wc, true);
    add_s_text(ctx, 0, 16, &tr("Sell items"), wc, true);
    add_s_text(ctx, 0, 18, &tr("Repair items"), wc, true);
    add_s_text(ctx, 0, 20, &tr("Leave the shop"), wc, true);
    add_s_line(ctx, 5);
    ctx.stores.storenumh = 20;
}

/// Shared body of `ScrollSmithBuy`, `ScrollWitchBuy` and `ScrollHealerBuy`.
fn scroll_item_list_buy(ctx: &mut Ctx, list: fn(&mut Ctx) -> &mut Vec<Item>, mut idx: usize) {
    clear_s_text(ctx, 5, 21);
    ctx.stores.stextup = 5;
    let mut l = 5;
    while l < 20 {
        let item = list(ctx)[idx].clone();
        if !item.is_empty() {
            let item_color = item.get_text_color_with_stat_check();
            let name = item.get_name(ctx);
            add_s_text_full(ctx, 20, l, &name, item_color, true, item._iCurs as i32, true);
            add_s_text_val(ctx, l, item._iIvalue);
            print_store_item(ctx, &item, l + 1, item_color, true);
            ctx.stores.stextdown = l as i32;
            idx += 1;
        }
        l += 4;
    }
    let s = &mut ctx.stores;
    if s.stextsel != -1 && !s.stext[s.stextsel as usize].is_selectable() && s.stextsel != back_button_line(ctx) {
        ctx.stores.stextsel = ctx.stores.stextdown;
    }
}

/// Original: `ScrollSmithBuy` (stores.cpp).
// @port stores.cpp|devilution::ScrollSmithBuy(int idx) sha=458d13aa131f
fn scroll_smith_buy(ctx: &mut Ctx, idx: i32) {
    scroll_item_list_buy(ctx, |c| &mut c.stores.smithitem, idx as usize);
}

/// Original: `TotalPlayerGold` (stores.cpp).
// @port stores.cpp|devilution::TotalPlayerGold() sha=3e1b62abe366
fn total_player_gold(ctx: &Ctx) -> u32 {
    (ctx.players.Players[my_player(ctx)]._pGold as u32).wrapping_add(ctx.stash.Stash.gold as u32)
}

/// Original: `PlayerCanAfford` (stores.cpp).
// @port stores.cpp|devilution::PlayerCanAfford(int price) sha=941c75eed89f
fn player_can_afford(ctx: &Ctx, price: i32) -> bool {
    total_player_gold(ctx) >= price as u32
}

/// Sets `_iStatFlag` on each non-empty item of a store list and returns their count.
fn refresh_stat_flags(ctx: &mut Ctx, list: fn(&mut Ctx) -> &mut Vec<Item>) -> i32 {
    let me = my_player(ctx);
    let mut items = std::mem::take(list(ctx));
    let mut n = 0;
    for item in items.iter_mut() {
        if item.is_empty() {
            continue;
        }
        item._iStatFlag = ctx.players.Players[me].can_use_item(item);
        n += 1;
    }
    *list(ctx) = items;
    n
}

/// Original: `StartSmithBuy` (stores.cpp).
// @port stores.cpp|devilution::StartSmithBuy() sha=02433f1a7985
fn start_smith_buy(ctx: &mut Ctx) {
    ctx.stores.stextsize = true;
    ctx.stores.stextscrl = true;
    ctx.stores.stextsval = 0;
    ctx.stores.render_gold = true;
    add_s_text(ctx, 20, 1, &tr("I have these items for sale:"), UiFlags::COLOR_WHITEGOLD, false);
    add_s_line(ctx, 3);
    scroll_smith_buy(ctx, ctx.stores.stextsval);
    add_item_list_back_button(ctx, false);
    ctx.stores.storenumh = refresh_stat_flags(ctx, |c| &mut c.stores.smithitem);
    ctx.stores.stextsmax = (ctx.stores.storenumh - 4).max(0);
}

/// Original: `ScrollSmithPremiumBuy` (stores.cpp).
// @port stores.cpp|devilution::ScrollSmithPremiumBuy(int boughtitems) sha=c8f05f1b714a
fn scroll_smith_premium_buy(ctx: &mut Ctx, mut boughtitems: i32) {
    clear_s_text(ctx, 5, 21);
    ctx.stores.stextup = 5;
    let mut idx = 0usize;
    while boughtitems != 0 {
        if !ctx.stores.premiumitems[idx].is_empty() {
            boughtitems -= 1;
        }
        idx += 1;
    }
    let mut l: i32 = 5;
    while l < 20 && idx < SMITH_PREMIUM_ITEMS {
        let item = ctx.stores.premiumitems[idx].clone();
        if !item.is_empty() {
            let item_color = item.get_text_color_with_stat_check();
            let name = item.get_name(ctx);
            add_s_text_full(ctx, 20, l as usize, &name, item_color, true, item._iCurs as i32, true);
            add_s_text_val(ctx, l as usize, item._iIvalue);
            print_store_item(ctx, &item, l as usize + 1, item_color, true);
            ctx.stores.stextdown = l;
        } else {
            l -= 4;
        }
        idx += 1;
        l += 4;
    }
    let s = &ctx.stores;
    if s.stextsel != -1 && !s.stext[s.stextsel as usize].is_selectable() && s.stextsel != back_button_line(ctx) {
        ctx.stores.stextsel = ctx.stores.stextdown;
    }
}

/// Original: `StartSmithPremiumBuy` (stores.cpp).
// @port stores.cpp|devilution::StartSmithPremiumBuy() sha=da2dbfaf1d50
fn start_smith_premium_buy(ctx: &mut Ctx) -> bool {
    ctx.stores.storenumh = refresh_stat_flags(ctx, |c| &mut c.stores.premiumitems);
    if ctx.stores.storenumh == 0 {
        start_store(ctx, TalkID::Smith);
        ctx.stores.stextsel = 14;
        return false;
    }
    ctx.stores.stextsize = true;
    ctx.stores.stextscrl = true;
    ctx.stores.stextsval = 0;
    ctx.stores.render_gold = true;
    add_s_text(ctx, 20, 1, &tr("I have these premium items for sale:"), UiFlags::COLOR_WHITEGOLD, false);
    add_s_line(ctx, 3);
    add_item_list_back_button(ctx, false);
    ctx.stores.stextsmax = (ctx.stores.storenumh - 4).max(0);
    scroll_smith_premium_buy(ctx, ctx.stores.stextsval);
    true
}

/// `InvList[i]` for `i >= 0`, else `SpdList[-(i + 1)]`.
fn inv_or_belt_item(ctx: &Ctx, i: i32) -> &Item {
    let p = &ctx.players.Players[my_player(ctx)];
    if i >= 0 {
        &p.InvList[i as usize]
    } else {
        &p.SpdList[(-(i + 1)) as usize]
    }
}

/// Original: `SmithSellOk` (stores.cpp).
// @port stores.cpp|devilution::SmithSellOk(int i) sha=1e4aded7d09e
fn smith_sell_ok(ctx: &Ctx, i: i32) -> bool {
    let p_i = inv_or_belt_item(ctx, i);
    if p_i.is_empty() {
        return false;
    }
    if p_i._iMiscId > IMISC_OILFIRST && p_i._iMiscId < IMISC_OILLAST {
        return true;
    }
    if p_i._itype == ItemType::Misc {
        return false;
    }
    if p_i._itype == ItemType::Gold {
        return false;
    }
    if p_i._itype == ItemType::Staff && (!ctx.init.gb_is_hellfire || crate::spells::is_valid_spell(ctx, p_i._iSpell)) {
        return false;
    }
    if p_i._iClass == ICLASS_QUEST {
        return false;
    }
    if p_i.IDidx == IDI_LAZSTAFF {
        return false;
    }
    true
}

/// Original: `ScrollSmithSell` (stores.cpp).
// @port stores.cpp|devilution::ScrollSmithSell(int idx) sha=7bde4c13be48
fn scroll_smith_sell(ctx: &mut Ctx, mut idx: i32) {
    clear_s_text(ctx, 5, 21);
    ctx.stores.stextup = 5;
    let mut l = 5;
    while l < 20 {
        if idx >= ctx.stores.storenumh {
            break;
        }
        let item = ctx.stores.storehold[idx as usize].clone();
        if !item.is_empty() {
            let item_color = item.get_text_color_with_stat_check();
            let name = item.get_name(ctx);
            add_s_text_full(ctx, 20, l, &name, item_color, true, item._iCurs as i32, true);
            if item._iMagical != ITEM_QUALITY_NORMAL && item._iIdentified {
                add_s_text_val(ctx, l, item._iIvalue);
            } else {
                add_s_text_val(ctx, l, item._ivalue);
            }
            print_store_item(ctx, &item, l + 1, item_color, true);
            ctx.stores.stextdown = l as i32;
        }
        idx += 1;
        l += 4;
    }
    ctx.stores.stextsmax = (ctx.stores.storenumh - 4).max(0);
}

/// Adds `item` (an inventory or belt item) to `storehold` at a quarter of its value.
fn add_store_hold_sell(ctx: &mut Ctx, item: &Item, i: i8) {
    let n = ctx.stores.storenumh as usize;
    let h = &mut ctx.stores.storehold[n];
    *h = item.clone();
    if h._iMagical != ITEM_QUALITY_NORMAL && h._iIdentified {
        h._ivalue = h._iIvalue;
    }
    h._ivalue = (h._ivalue / 4).max(1);
    h._iIvalue = h._ivalue;
    ctx.stores.storehidx[n] = i;
    ctx.stores.storenumh += 1;
}

fn clear_store_hold(ctx: &mut Ctx) {
    for item in ctx.stores.storehold.iter_mut() {
        item.clear();
    }
}

/// Original: `StartSmithSell` (stores.cpp).
// @port stores.cpp|devilution::StartSmithSell() sha=db566d697939
fn start_smith_sell(ctx: &mut Ctx) {
    ctx.stores.stextsize = true;
    let mut sell_ok = false;
    ctx.stores.storenumh = 0;
    clear_store_hold(ctx);
    let me = my_player(ctx);
    let num_inv = ctx.players.Players[me]._pNumInv;
    for i in 0..num_inv {
        if ctx.stores.storenumh >= 48 {
            break;
        }
        if smith_sell_ok(ctx, i) {
            sell_ok = true;
            let item = ctx.players.Players[me].InvList[i as usize].clone();
            add_store_hold_sell(ctx, &item, i as i8);
        }
    }
    for i in 0..crate::player::MaxBeltItems as i32 {
        if ctx.stores.storenumh >= 48 {
            break;
        }
        if smith_sell_ok(ctx, -(i + 1)) {
            sell_ok = true;
            let item = ctx.players.Players[me].SpdList[i as usize].clone();
            add_store_hold_sell(ctx, &item, -(i + 1) as i8);
        }
    }
    if !sell_ok {
        ctx.stores.stextscrl = false;
        ctx.stores.render_gold = true;
        add_s_text(ctx, 20, 1, &tr("You have nothing I want."), UiFlags::COLOR_WHITEGOLD, false);
        add_s_line(ctx, 3);
        add_item_list_back_button(ctx, true);
        return;
    }
    ctx.stores.stextscrl = true;
    ctx.stores.stextsval = 0;
    ctx.stores.stextsmax = num_inv;
    ctx.stores.render_gold = true;
    add_s_text(ctx, 20, 1, &tr("Which item is for sale?"), UiFlags::COLOR_WHITEGOLD, false);
    add_s_line(ctx, 3);
    scroll_smith_sell(ctx, ctx.stores.stextsval);
    add_item_list_back_button(ctx, false);
}

/// Original: `SmithRepairOk` (stores.cpp).
// @port stores.cpp|devilution::SmithRepairOk(int i) sha=8ac02fbf971f
fn smith_repair_ok(ctx: &Ctx, i: i32) -> bool {
    let item = &ctx.players.Players[my_player(ctx)].InvList[i as usize];
    if item.is_empty() {
        return false;
    }
    if item._itype == ItemType::Misc {
        return false;
    }
    if item._itype == ItemType::Gold {
        return false;
    }
    if item._iDurability == item._iMaxDur {
        return false;
    }
    true
}

/// Original: `StartSmithRepair` (stores.cpp).
// @port stores.cpp|devilution::StartSmithRepair() sha=aa241027500c
fn start_smith_repair(ctx: &mut Ctx) {
    ctx.stores.stextsize = true;
    ctx.stores.storenumh = 0;
    clear_store_hold(ctx);
    let me = my_player(ctx);
    for (loc, idx) in [(INVLOC_HEAD, -1), (INVLOC_CHEST, -2), (INVLOC_HAND_LEFT, -3), (INVLOC_HAND_RIGHT, -4)] {
        let item = ctx.players.Players[me].InvBody[loc as usize].clone();
        if !item.is_empty() && item._iDurability != item._iMaxDur {
            add_store_hold_repair(ctx, &item, idx);
        }
    }
    let num_inv = ctx.players.Players[me]._pNumInv;
    for i in 0..num_inv {
        if ctx.stores.storenumh >= 48 {
            break;
        }
        if smith_repair_ok(ctx, i) {
            let item = ctx.players.Players[me].InvList[i as usize].clone();
            add_store_hold_repair(ctx, &item, i as i8);
        }
    }
    if ctx.stores.storenumh == 0 {
        ctx.stores.stextscrl = false;
        ctx.stores.render_gold = true;
        add_s_text(ctx, 20, 1, &tr("You have nothing to repair."), UiFlags::COLOR_WHITEGOLD, false);
        add_s_line(ctx, 3);
        add_item_list_back_button(ctx, true);
        return;
    }
    ctx.stores.stextscrl = true;
    ctx.stores.stextsval = 0;
    ctx.stores.stextsmax = num_inv;
    ctx.stores.render_gold = true;
    add_s_text(ctx, 20, 1, &tr("Repair which item?"), UiFlags::COLOR_WHITEGOLD, false);
    add_s_line(ctx, 3);
    scroll_smith_sell(ctx, ctx.stores.stextsval);
    add_item_list_back_button(ctx, false);
}

/// Original: `FillManaPlayer` (stores.cpp).
// @port stores.cpp|devilution::FillManaPlayer() sha=5d08e2625483
fn fill_mana_player(ctx: &mut Ctx) {
    if !ctx.options.gameplay.adria_refills_mana.get() {
        return;
    }
    let me = my_player(ctx);
    if ctx.players.Players[me]._pMana != ctx.players.Players[me]._pMaxMana {
        crate::effects::play_sfx(ctx, crate::effects::IS_CAST8);
    }
    let p = &mut ctx.players.Players[me];
    p._pMana = p._pMaxMana;
    p._pManaBase = p._pMaxManaBase;
    redraw_component(ctx, PanelDrawComponent::Mana);
}

/// Original: `StartWitch` (stores.cpp).
// @port stores.cpp|devilution::StartWitch() sha=ec2273415359
fn start_witch(ctx: &mut Ctx) {
    fill_mana_player(ctx);
    ctx.stores.stextsize = false;
    ctx.stores.stextscrl = false;
    let wgc = UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER;
    let wc = UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER;
    add_s_text(ctx, 0, 2, &tr("Witch's shack"), wgc, false);
    add_s_text(ctx, 0, 9, &tr("Would you like to:"), wgc, false);
    add_s_text(ctx, 0, 12, &tr("Talk to Adria"), UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true);
    add_s_text(ctx, 0, 14, &tr("Buy items"), wc, true);
    add_s_text(ctx, 0, 16, &tr("Sell items"), wc, true);
    add_s_text(ctx, 0, 18, &tr("Recharge staves"), wc, true);
    add_s_text(ctx, 0, 20, &tr("Leave the shack"), wc, true);
    add_s_line(ctx, 5);
    ctx.stores.storenumh = 20;
}

/// Original: `ScrollWitchBuy` (stores.cpp).
// @port stores.cpp|devilution::ScrollWitchBuy(int idx) sha=77ed3b7be52f
fn scroll_witch_buy(ctx: &mut Ctx, idx: i32) {
    scroll_item_list_buy(ctx, |c| &mut c.stores.witchitem, idx as usize);
}

/// Original: `WitchBookLevel` (stores.cpp).
// @port stores.cpp|devilution::WitchBookLevel(Item &bookItem) sha=65852a076282
fn witch_book_level(ctx: &Ctx, book_item: &mut Item) {
    if book_item._iMiscId != IMISC_BOOK {
        return;
    }
    book_item._iMinMag = crate::items::get_spell_data(book_item._iSpell).minInt;
    let mut spell_level = ctx.players.Players[my_player(ctx)]._pSplLvl[book_item._iSpell as i8 as usize];
    while spell_level > 0 {
        book_item._iMinMag = book_item._iMinMag.wrapping_add((20 * book_item._iMinMag as i32 / 100) as u8);
        spell_level -= 1;
        if book_item._iMinMag as i32 + 20 * book_item._iMinMag as i32 / 100 > 255 {
            book_item._iMinMag = 255;
            spell_level = 0;
        }
    }
}

/// Original: `StartWitchBuy` (stores.cpp).
// @port stores.cpp|devilution::StartWitchBuy() sha=bdc235ce4f48
fn start_witch_buy(ctx: &mut Ctx) {
    ctx.stores.stextsize = true;
    ctx.stores.stextscrl = true;
    ctx.stores.stextsval = 0;
    ctx.stores.stextsmax = 20;
    ctx.stores.render_gold = true;
    add_s_text(ctx, 20, 1, &tr("I have these items for sale:"), UiFlags::COLOR_WHITEGOLD, false);
    add_s_line(ctx, 3);
    scroll_witch_buy(ctx, ctx.stores.stextsval);
    add_item_list_back_button(ctx, false);
    ctx.stores.storenumh = 0;
    let me = my_player(ctx);
    let mut items = std::mem::take(&mut ctx.stores.witchitem);
    for item in items.iter_mut() {
        if item.is_empty() {
            continue;
        }
        witch_book_level(ctx, item);
        item._iStatFlag = ctx.players.Players[me].can_use_item(item);
        ctx.stores.storenumh += 1;
    }
    ctx.stores.witchitem = items;
    ctx.stores.stextsmax = (ctx.stores.storenumh - 4).max(0);
}

/// Original: `WitchSellOk` (stores.cpp).
// @port stores.cpp|devilution::WitchSellOk(int i) sha=7048a2870254
fn witch_sell_ok(ctx: &Ctx, i: i32) -> bool {
    let p_i = inv_or_belt_item(ctx, i);
    let mut rv = false;
    if p_i._itype == ItemType::Misc {
        rv = true;
    }
    if p_i._iMiscId > 29 && p_i._iMiscId < 41 {
        rv = false;
    }
    if p_i._iClass == ICLASS_QUEST {
        rv = false;
    }
    if p_i._itype == ItemType::Staff && (!ctx.init.gb_is_hellfire || crate::spells::is_valid_spell(ctx, p_i._iSpell)) {
        rv = true;
    }
    if p_i.IDidx >= IDI_FIRSTQUEST && p_i.IDidx <= IDI_LASTQUEST {
        rv = false;
    }
    if p_i.IDidx == IDI_LAZSTAFF {
        rv = false;
    }
    rv
}

/// Original: `StartWitchSell` (stores.cpp).
// @port stores.cpp|devilution::StartWitchSell() sha=82f0c021b0dc
fn start_witch_sell(ctx: &mut Ctx) {
    ctx.stores.stextsize = true;
    let mut sellok = false;
    ctx.stores.storenumh = 0;
    clear_store_hold(ctx);
    let me = my_player(ctx);
    let num_inv = ctx.players.Players[me]._pNumInv;
    for i in 0..num_inv {
        if ctx.stores.storenumh >= 48 {
            break;
        }
        if witch_sell_ok(ctx, i) {
            sellok = true;
            let item = ctx.players.Players[me].InvList[i as usize].clone();
            add_store_hold_sell(ctx, &item, i as i8);
        }
    }
    for i in 0..crate::player::MaxBeltItems as i32 {
        if ctx.stores.storenumh >= 48 {
            break;
        }
        if !ctx.players.Players[me].SpdList[i as usize].is_empty() && witch_sell_ok(ctx, -(i + 1)) {
            sellok = true;
            let item = ctx.players.Players[me].SpdList[i as usize].clone();
            add_store_hold_sell(ctx, &item, -(i + 1) as i8);
        }
    }
    if !sellok {
        ctx.stores.stextscrl = false;
        ctx.stores.render_gold = true;
        add_s_text(ctx, 20, 1, &tr("You have nothing I want."), UiFlags::COLOR_WHITEGOLD, false);
        add_s_line(ctx, 3);
        add_item_list_back_button(ctx, true);
        return;
    }
    ctx.stores.stextscrl = true;
    ctx.stores.stextsval = 0;
    ctx.stores.stextsmax = num_inv;
    ctx.stores.render_gold = true;
    add_s_text(ctx, 20, 1, &tr("Which item is for sale?"), UiFlags::COLOR_WHITEGOLD, false);
    add_s_line(ctx, 3);
    scroll_smith_sell(ctx, ctx.stores.stextsval);
    add_item_list_back_button(ctx, false);
}

/// Original: `WitchRechargeOk` (stores.cpp).
// @port stores.cpp|devilution::WitchRechargeOk(int i) sha=b56485a4611f
fn witch_recharge_ok(ctx: &Ctx, i: i32) -> bool {
    let item = &ctx.players.Players[my_player(ctx)].InvList[i as usize];
    if item._itype == ItemType::Staff && item._iCharges != item._iMaxCharges {
        return true;
    }
    if (item._iMiscId == IMISC_UNIQUE || item._iMiscId == IMISC_STAFF) && item._iCharges < item._iMaxCharges {
        return true;
    }
    false
}

/// Original: `AddStoreHoldRecharge` (stores.cpp).
// @port stores.cpp|devilution::AddStoreHoldRecharge(Item itm, int8_t i) sha=5953b83dd2f1
fn add_store_hold_recharge(ctx: &mut Ctx, itm: Item, i: i8) {
    let n = ctx.stores.storenumh as usize;
    let h = &mut ctx.stores.storehold[n];
    *h = itm;
    h._ivalue += crate::items::get_spell_data(h._iSpell).staffCost10 as i32 * 10;
    h._ivalue = h._ivalue * (h._iMaxCharges - h._iCharges) / (h._iMaxCharges * 2);
    h._iIvalue = h._ivalue;
    ctx.stores.storehidx[n] = i;
    ctx.stores.storenumh += 1;
}

/// Original: `StartWitchRecharge` (stores.cpp).
// @port stores.cpp|devilution::StartWitchRecharge() sha=8a053d5aa4b2
fn start_witch_recharge(ctx: &mut Ctx) {
    ctx.stores.stextsize = true;
    let mut rechargeok = false;
    ctx.stores.storenumh = 0;
    clear_store_hold(ctx);
    let me = my_player(ctx);
    let left_hand = ctx.players.Players[me].InvBody[INVLOC_HAND_LEFT as usize].clone();
    if (left_hand._itype == ItemType::Staff || left_hand._iMiscId == IMISC_UNIQUE) && left_hand._iCharges != left_hand._iMaxCharges {
        rechargeok = true;
        add_store_hold_recharge(ctx, left_hand, -1);
    }
    let num_inv = ctx.players.Players[me]._pNumInv;
    for i in 0..num_inv {
        if ctx.stores.storenumh >= 48 {
            break;
        }
        if witch_recharge_ok(ctx, i) {
            rechargeok = true;
            let item = ctx.players.Players[me].InvList[i as usize].clone();
            add_store_hold_recharge(ctx, item, i as i8);
        }
    }
    if !rechargeok {
        ctx.stores.stextscrl = false;
        ctx.stores.render_gold = true;
        add_s_text(ctx, 20, 1, &tr("You have nothing to recharge."), UiFlags::COLOR_WHITEGOLD, false);
        add_s_line(ctx, 3);
        add_item_list_back_button(ctx, true);
        return;
    }
    ctx.stores.stextscrl = true;
    ctx.stores.stextsval = 0;
    ctx.stores.stextsmax = num_inv;
    ctx.stores.render_gold = true;
    add_s_text(ctx, 20, 1, &tr("Recharge which item?"), UiFlags::COLOR_WHITEGOLD, false);
    add_s_line(ctx, 3);
    scroll_smith_sell(ctx, ctx.stores.stextsval);
    add_item_list_back_button(ctx, false);
}

/// Original: `StoreNoMoney` (stores.cpp).
// @port stores.cpp|devilution::StoreNoMoney() sha=c2da19809af5
fn store_no_money(ctx: &mut Ctx) {
    start_store(ctx, ctx.stores.stextshold);
    ctx.stores.stextscrl = false;
    ctx.stores.stextsize = true;
    ctx.stores.render_gold = true;
    clear_s_text(ctx, 5, 23);
    add_s_text(ctx, 0, 14, &tr("You do not have enough gold"), UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true);
}

/// Original: `StoreNoRoom` (stores.cpp).
// @port stores.cpp|devilution::StoreNoRoom() sha=3ac2ba5be7c2
fn store_no_room(ctx: &mut Ctx) {
    start_store(ctx, ctx.stores.stextshold);
    ctx.stores.stextscrl = false;
    clear_s_text(ctx, 5, 23);
    add_s_text(ctx, 0, 14, &tr("You do not have enough room in inventory"), UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true);
}

/// Original: `StoreConfirm` (stores.cpp).
// @port stores.cpp|devilution::StoreConfirm(Item &item) sha=a25f206beeac
fn store_confirm(ctx: &mut Ctx, item: &Item) {
    start_store(ctx, ctx.stores.stextshold);
    ctx.stores.stextscrl = false;
    clear_s_text(ctx, 5, 23);
    let item_color = item.get_text_color_with_stat_check();
    let name = item.get_name(ctx);
    add_s_text(ctx, 20, 8, &name, item_color, false);
    add_s_text_val(ctx, 8, item._iIvalue);
    print_store_item(ctx, item, 9, item_color, false);
    let prompt = match ctx.stores.stextshold {
        TalkID::BoyBuy => tr("Do we have a deal?"),
        TalkID::StorytellerIdentify => tr("Are you sure you want to identify this item?"),
        TalkID::HealerBuy | TalkID::SmithPremiumBuy | TalkID::WitchBuy | TalkID::SmithBuy => tr("Are you sure you want to buy this item?"),
        TalkID::WitchRecharge => tr("Are you sure you want to recharge this item?"),
        TalkID::SmithSell | TalkID::WitchSell => tr("Are you sure you want to sell this item?"),
        TalkID::SmithRepair => tr("Are you sure you want to repair this item?"),
        other => crate::appfat::app_fatal(ctx, &format!("Unknown store dialog {}", other as i32)),
    };
    add_s_text(ctx, 0, 15, &prompt, UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, false);
    add_s_text(ctx, 0, 18, &tr("Yes"), UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true);
    add_s_text(ctx, 0, 20, &tr("No"), UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true);
}

/// Original: `StartBoy` (stores.cpp).
// @port stores.cpp|devilution::StartBoy() sha=97c8b1709da4
fn start_boy(ctx: &mut Ctx) {
    ctx.stores.stextsize = false;
    ctx.stores.stextscrl = false;
    let wgc = UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER;
    let wc = UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER;
    add_s_text(ctx, 0, 2, &tr("Wirt the Peg-legged boy"), wgc, false);
    add_s_line(ctx, 5);
    if !ctx.stores.boyitem.is_empty() {
        add_s_text(ctx, 0, 8, &tr("Talk to Wirt"), UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true);
        add_s_text(ctx, 0, 12, &tr("I have something for sale,"), wgc, false);
        add_s_text(ctx, 0, 14, &tr("but it will cost 50 gold"), wgc, false);
        add_s_text(ctx, 0, 16, &tr("just to take a look. "), wgc, false);
        add_s_text(ctx, 0, 18, &tr("What have you got?"), wc, true);
        add_s_text(ctx, 0, 20, &tr("Say goodbye"), wc, true);
    } else {
        add_s_text(ctx, 0, 12, &tr("Talk to Wirt"), UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true);
        add_s_text(ctx, 0, 18, &tr("Say goodbye"), wc, true);
    }
}

/// Original: `SStartBoyBuy` (stores.cpp).
// @port stores.cpp|devilution::SStartBoyBuy() sha=6746aa1cf87e
fn s_start_boy_buy(ctx: &mut Ctx) {
    ctx.stores.stextsize = true;
    ctx.stores.stextscrl = false;
    ctx.stores.render_gold = true;
    add_s_text(ctx, 20, 1, &tr("I have this item for sale:"), UiFlags::COLOR_WHITEGOLD, false);
    add_s_line(ctx, 3);
    let me = my_player(ctx);
    let flag = ctx.players.Players[me].can_use_item(&ctx.stores.boyitem);
    ctx.stores.boyitem._iStatFlag = flag;
    let boyitem = ctx.stores.boyitem.clone();
    let item_color = boyitem.get_text_color_with_stat_check();
    let name = boyitem.get_name(ctx);
    add_s_text_full(ctx, 20, 10, &name, item_color, true, boyitem._iCurs as i32, true);
    if ctx.init.gb_is_hellfire {
        add_s_text_val(ctx, 10, boyitem._iIvalue - (boyitem._iIvalue / 4));
    } else {
        add_s_text_val(ctx, 10, boyitem._iIvalue + (boyitem._iIvalue / 2));
    }
    print_store_item(ctx, &boyitem, 11, item_color, true);
    {
        // Add a Leave button. Unlike the other item list back buttons,
        // this one has different text and different layout in LargerSmallFont locales.
        let line = back_button_line(ctx) as usize;
        add_s_line(ctx, line - 1);
        add_s_text(ctx, 0, line, &tr("Leave"), UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true);
        ctx.stores.stext[line]._syoff = 6;
    }
}

/// Original: `HealPlayer` (stores.cpp).
// @port stores.cpp|devilution::HealPlayer() sha=16b1830dda2a
fn heal_player(ctx: &mut Ctx) {
    let me = my_player(ctx);
    if ctx.players.Players[me]._pHitPoints != ctx.players.Players[me]._pMaxHP {
        crate::effects::play_sfx(ctx, crate::effects::IS_CAST8);
    }
    let p = &mut ctx.players.Players[me];
    p._pHitPoints = p._pMaxHP;
    p._pHPBase = p._pMaxHPBase;
    redraw_component(ctx, PanelDrawComponent::Health);
}

/// Original: `StartHealer` (stores.cpp).
// @port stores.cpp|devilution::StartHealer() sha=e0ac6df338ad
fn start_healer(ctx: &mut Ctx) {
    heal_player(ctx);
    ctx.stores.stextsize = false;
    ctx.stores.stextscrl = false;
    let wgc = UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER;
    let wc = UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER;
    add_s_text(ctx, 0, 1, &tr("Welcome to the"), wgc, false);
    add_s_text(ctx, 0, 3, &tr("Healer's home"), wgc, false);
    add_s_text(ctx, 0, 9, &tr("Would you like to:"), wgc, false);
    add_s_text(ctx, 0, 12, &tr("Talk to Pepin"), UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true);
    add_s_text(ctx, 0, 14, &tr("Buy items"), wc, true);
    add_s_text(ctx, 0, 18, &tr("Leave Healer's home"), wc, true);
    add_s_line(ctx, 5);
    ctx.stores.storenumh = 20;
}

/// Original: `ScrollHealerBuy` (stores.cpp).
// @port stores.cpp|devilution::ScrollHealerBuy(int idx) sha=1b193a150ccd
fn scroll_healer_buy(ctx: &mut Ctx, idx: i32) {
    scroll_item_list_buy(ctx, |c| &mut c.stores.healitem, idx as usize);
}

/// Original: `StartHealerBuy` (stores.cpp).
// @port stores.cpp|devilution::StartHealerBuy() sha=02c204111d0e
fn start_healer_buy(ctx: &mut Ctx) {
    ctx.stores.stextsize = true;
    ctx.stores.stextscrl = true;
    ctx.stores.stextsval = 0;
    ctx.stores.render_gold = true;
    add_s_text(ctx, 20, 1, &tr("I have these items for sale:"), UiFlags::COLOR_WHITEGOLD, false);
    add_s_line(ctx, 3);
    scroll_healer_buy(ctx, ctx.stores.stextsval);
    add_item_list_back_button(ctx, false);
    ctx.stores.storenumh = refresh_stat_flags(ctx, |c| &mut c.stores.healitem);
    ctx.stores.stextsmax = (ctx.stores.storenumh - 4).max(0);
}

/// Original: `StartStoryteller` (stores.cpp).
// @port stores.cpp|devilution::StartStoryteller() sha=c16f839c06a2
fn start_storyteller(ctx: &mut Ctx) {
    ctx.stores.stextsize = false;
    ctx.stores.stextscrl = false;
    let wgc = UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER;
    let wc = UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER;
    add_s_text(ctx, 0, 2, &tr("The Town Elder"), wgc, false);
    add_s_text(ctx, 0, 9, &tr("Would you like to:"), wgc, false);
    add_s_text(ctx, 0, 12, &tr("Talk to Cain"), UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true);
    add_s_text(ctx, 0, 14, &tr("Identify an item"), wc, true);
    add_s_text(ctx, 0, 18, &tr("Say goodbye"), wc, true);
    add_s_line(ctx, 5);
}

/// Original: `IdItemOk` (stores.cpp).
// @port stores.cpp|devilution::IdItemOk(Item *i) sha=14d5bb1fb146
fn id_item_ok(i: &Item) -> bool {
    if i.is_empty() {
        return false;
    }
    if i._iMagical == ITEM_QUALITY_NORMAL {
        return false;
    }
    !i._iIdentified
}

/// Original: `AddStoreHoldId` (stores.cpp).
// @port stores.cpp|devilution::AddStoreHoldId(Item itm, int8_t i) sha=eb1751af76c6
fn add_store_hold_id(ctx: &mut Ctx, itm: Item, i: i8) {
    let n = ctx.stores.storenumh as usize;
    let h = &mut ctx.stores.storehold[n];
    *h = itm;
    h._ivalue = 100;
    h._iIvalue = 100;
    ctx.stores.storehidx[n] = i;
    ctx.stores.storenumh += 1;
}

/// Original: `StartStorytellerIdentify` (stores.cpp).
// @port stores.cpp|devilution::StartStorytellerIdentify() sha=5680711eef1b
fn start_storyteller_identify(ctx: &mut Ctx) {
    let mut idok = false;
    ctx.stores.stextsize = true;
    ctx.stores.storenumh = 0;
    clear_store_hold(ctx);
    let me = my_player(ctx);
    let body = [(INVLOC_HEAD, -1), (INVLOC_CHEST, -2), (INVLOC_HAND_LEFT, -3), (INVLOC_HAND_RIGHT, -4), (INVLOC_RING_LEFT, -5), (INVLOC_RING_RIGHT, -6), (INVLOC_AMULET, -7)];
    for (loc, idx) in body {
        let item = ctx.players.Players[me].InvBody[loc as usize].clone();
        if id_item_ok(&item) {
            idok = true;
            add_store_hold_id(ctx, item, idx);
        }
    }
    let num_inv = ctx.players.Players[me]._pNumInv;
    for i in 0..num_inv {
        if ctx.stores.storenumh >= 48 {
            break;
        }
        let item = ctx.players.Players[me].InvList[i as usize].clone();
        if id_item_ok(&item) {
            idok = true;
            add_store_hold_id(ctx, item, i as i8);
        }
    }
    if !idok {
        ctx.stores.stextscrl = false;
        ctx.stores.render_gold = true;
        add_s_text(ctx, 20, 1, &tr("You have nothing to identify."), UiFlags::COLOR_WHITEGOLD, false);
        add_s_line(ctx, 3);
        add_item_list_back_button(ctx, true);
        return;
    }
    ctx.stores.stextscrl = true;
    ctx.stores.stextsval = 0;
    ctx.stores.stextsmax = num_inv;
    ctx.stores.render_gold = true;
    add_s_text(ctx, 20, 1, &tr("Identify which item?"), UiFlags::COLOR_WHITEGOLD, false);
    add_s_line(ctx, 3);
    scroll_smith_sell(ctx, ctx.stores.stextsval);
    add_item_list_back_button(ctx, false);
}

/// Original: `StartStorytellerIdentifyShow` (stores.cpp).
// @port stores.cpp|devilution::StartStorytellerIdentifyShow(Item &item) sha=7fd77b79d7e9
fn start_storyteller_identify_show(ctx: &mut Ctx, item: &Item) {
    start_store(ctx, ctx.stores.stextshold);
    ctx.stores.stextscrl = false;
    clear_s_text(ctx, 5, 23);
    let item_color = item.get_text_color_with_stat_check();
    add_s_text(ctx, 0, 7, &tr("This item is:"), UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, false);
    let name = item.get_name(ctx);
    add_s_text(ctx, 20, 11, &name, item_color, false);
    print_store_item(ctx, item, 12, item_color, false);
    add_s_text(ctx, 0, 18, &tr("Done"), UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true);
}

/// Whether `quest` has a dialog entry for the current talker (`StartTalk` / `TalkEnter`).
fn quest_has_talk(ctx: &Ctx, quest: &crate::quests::Quest) -> bool {
    quest._qactive == QUEST_ACTIVE && ctx.towners.QuestDialogTable[ctx.stores.talker as usize][quest._qidx as usize] != TEXT_NONE && quest._qlog
}

/// Original: `StartTalk` (stores.cpp).
// @port stores.cpp|devilution::StartTalk() sha=1b7833de9fa3
fn start_talk(ctx: &mut Ctx) {
    ctx.stores.stextsize = false;
    ctx.stores.stextscrl = false;
    let towner_name = tr(TOWNER_NAMES[ctx.stores.talker as usize]);
    add_s_text(ctx, 0, 2, &fmt1(&tr("Talk to {:s}"), &towner_name), UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, false);
    add_s_line(ctx, 5);
    if ctx.init.gb_is_spawn {
        let wc = UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER;
        add_s_text(ctx, 0, 10, &fmt1(&tr("Talking to {:s}"), &towner_name), wc, false);
        add_s_text(ctx, 0, 12, &tr("is not available"), wc, false);
        add_s_text(ctx, 0, 14, &tr("in the shareware"), wc, false);
        add_s_text(ctx, 0, 16, &tr("version"), wc, false);
        add_options_back_button(ctx);
        return;
    }
    let mut sn = ctx.quests.Quests.iter().filter(|q| quest_has_talk(ctx, q)).count() as i32;
    let la;
    if sn > 6 {
        sn = 14 - (sn / 2);
        la = 1;
    } else {
        sn = 15 - sn;
        la = 2;
    }
    let sn2 = sn - 2;
    let quests = ctx.quests.Quests;
    for quest in quests.iter() {
        if quest_has_talk(ctx, quest) {
            add_s_text(ctx, 0, sn as usize, &tr(crate::quests::QuestsData[quest._qidx as usize]._qlstr), UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true);
            sn += la;
        }
    }
    add_s_text(ctx, 0, sn2 as usize, &tr("Gossip"), UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true);
    add_options_back_button(ctx);
}

/// Original: `StartTavern` (stores.cpp).
// @port stores.cpp|devilution::StartTavern() sha=7ae50343d634
fn start_tavern(ctx: &mut Ctx) {
    ctx.stores.stextsize = false;
    ctx.stores.stextscrl = false;
    let wgc = UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER;
    add_s_text(ctx, 0, 1, &tr("Welcome to the"), wgc, false);
    add_s_text(ctx, 0, 3, &tr("Rising Sun"), wgc, false);
    add_s_text(ctx, 0, 9, &tr("Would you like to:"), wgc, false);
    add_s_text(ctx, 0, 12, &tr("Talk to Ogden"), UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true);
    add_s_text(ctx, 0, 18, &tr("Leave the tavern"), UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true);
    add_s_line(ctx, 5);
    ctx.stores.storenumh = 20;
}

/// Original: `StartBarmaid` (stores.cpp).
// @port stores.cpp|devilution::StartBarmaid() sha=430b8cf8c82c
fn start_barmaid(ctx: &mut Ctx) {
    ctx.stores.stextsize = false;
    ctx.stores.stextscrl = false;
    let wgc = UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER;
    let wc = UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER;
    add_s_text(ctx, 0, 2, &tr("Gillian"), wgc, false);
    add_s_text(ctx, 0, 9, &tr("Would you like to:"), wgc, false);
    add_s_text(ctx, 0, 12, &tr("Talk to Gillian"), UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true);
    add_s_text(ctx, 0, 14, &tr("Access Storage"), wc, true);
    add_s_text(ctx, 0, 18, &tr("Say goodbye"), wc, true);
    add_s_line(ctx, 5);
    ctx.stores.storenumh = 20;
}

/// Original: `StartDrunk` (stores.cpp).
// @port stores.cpp|devilution::StartDrunk() sha=bf92db456012
fn start_drunk(ctx: &mut Ctx) {
    ctx.stores.stextsize = false;
    ctx.stores.stextscrl = false;
    let wgc = UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER;
    add_s_text(ctx, 0, 2, &tr("Farnham the Drunk"), wgc, false);
    add_s_text(ctx, 0, 9, &tr("Would you like to:"), wgc, false);
    add_s_text(ctx, 0, 12, &tr("Talk to Farnham"), UiFlags::COLOR_BLUE | UiFlags::ALIGN_CENTER, true);
    add_s_text(ctx, 0, 18, &tr("Say Goodbye"), UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, true);
    add_s_line(ctx, 5);
    ctx.stores.storenumh = 20;
}

/// The common "talk to the towner" branch of the `*Enter` handlers.
fn enter_gossip(ctx: &mut Ctx, talker: _talker_id, lhold: i32, shold: TalkID) {
    ctx.stores.stextlhold = lhold;
    ctx.stores.talker = talker;
    ctx.stores.stextshold = shold;
    start_store(ctx, TalkID::Gossip);
}

/// Original: `SmithEnter` (stores.cpp).
// @port stores.cpp|devilution::SmithEnter() sha=051d5d3fbdcb
fn smith_enter(ctx: &mut Ctx) {
    match ctx.stores.stextsel {
        10 => enter_gossip(ctx, TOWN_SMITH, 10, TalkID::Smith),
        12 => start_store(ctx, TalkID::SmithBuy),
        14 => start_store(ctx, TalkID::SmithPremiumBuy),
        16 => start_store(ctx, TalkID::SmithSell),
        18 => start_store(ctx, TalkID::SmithRepair),
        20 => ctx.stores.stextflag = TalkID::None,
        _ => {}
    }
}

/// `stextvhold + ((stextlhold - stextup) / 4)`
fn held_index(ctx: &Ctx) -> i32 {
    let s = &ctx.stores;
    s.stextvhold + ((s.stextlhold - s.stextup) / 4)
}

/// `stextsval + ((stextsel - stextup) / 4)`
fn selected_index(ctx: &Ctx) -> i32 {
    let s = &ctx.stores;
    s.stextsval + ((s.stextsel - s.stextup) / 4)
}

/// Removes `list[idx]`, shifting the following items down (the buy item handlers).
fn remove_from_list(list: &mut [Item], mut idx: usize) {
    if idx == list.len() - 1 {
        list[idx].clear();
    } else {
        while !list[idx + 1].is_empty() {
            list[idx] = std::mem::take(&mut list[idx + 1]);
            idx += 1;
        }
        list[idx].clear();
    }
}

/// Original: `SmithBuyItem` (stores.cpp): purchases an item from the smith.
// @port stores.cpp|devilution::SmithBuyItem(Item &item) sha=767ce7a07313
fn smith_buy_item(ctx: &mut Ctx, item: &mut Item) {
    take_plrs_money(ctx, item._iIvalue);
    if item._iMagical == ITEM_QUALITY_NORMAL {
        item._iIdentified = false;
    }
    store_auto_place(ctx, item, true);
    let idx = held_index(ctx) as usize;
    remove_from_list(&mut ctx.stores.smithitem, idx);
    let me = my_player(ctx);
    crate::items::calc_plr_inv(ctx, me, true);
}

/// The common body of the buy `*Enter` handlers.
fn buy_enter(ctx: &mut Ctx, shold: TalkID, item: Item) {
    ctx.stores.stextlhold = ctx.stores.stextsel;
    ctx.stores.stextvhold = ctx.stores.stextsval;
    ctx.stores.stextshold = shold;
    if !player_can_afford(ctx, item._iIvalue) {
        start_store(ctx, TalkID::NoMoney);
        return;
    }
    if !store_auto_place(ctx, &item, false) {
        start_store(ctx, TalkID::NoRoom);
        return;
    }
    ctx.stores.store_item = item;
    start_store(ctx, TalkID::Confirm);
}

/// Original: `SmithBuyEnter` (stores.cpp).
// @port stores.cpp|devilution::SmithBuyEnter() sha=1880fa8ea8d5
fn smith_buy_enter(ctx: &mut Ctx) {
    if ctx.stores.stextsel == back_button_line(ctx) {
        start_store(ctx, TalkID::Smith);
        ctx.stores.stextsel = 12;
        return;
    }
    let idx = selected_index(ctx) as usize;
    let item = ctx.stores.smithitem[idx].clone();
    buy_enter(ctx, TalkID::SmithBuy, item);
}

/// Original: `SmithBuyPItem` (stores.cpp): purchases a premium item from the smith.
// @port stores.cpp|devilution::SmithBuyPItem(Item &item) sha=9344e4f982cb
fn smith_buy_p_item(ctx: &mut Ctx, item: &mut Item) {
    take_plrs_money(ctx, item._iIvalue);
    if item._iMagical == ITEM_QUALITY_NORMAL {
        item._iIdentified = false;
    }
    store_auto_place(ctx, item, true);
    let mut idx = held_index(ctx);
    let mut xx = 0;
    let mut i = 0;
    while idx >= 0 {
        if !ctx.stores.premiumitems[i].is_empty() {
            idx -= 1;
            xx = i;
        }
        i += 1;
    }
    ctx.stores.premiumitems[xx].clear();
    ctx.stores.numpremium -= 1;
    let me = my_player(ctx);
    let player = ctx.players.Players[me].clone();
    crate::items::spawn_premium(ctx, &player);
}

/// Original: `SmithPremiumBuyEnter` (stores.cpp).
// @port stores.cpp|devilution::SmithPremiumBuyEnter() sha=86c690459076
fn smith_premium_buy_enter(ctx: &mut Ctx) {
    if ctx.stores.stextsel == back_button_line(ctx) {
        start_store(ctx, TalkID::Smith);
        ctx.stores.stextsel = 14;
        return;
    }
    let mut xx = selected_index(ctx);
    let mut idx = 0;
    let mut i = 0;
    while xx >= 0 {
        if !ctx.stores.premiumitems[i].is_empty() {
            xx -= 1;
            idx = i;
        }
        i += 1;
    }
    let item = ctx.stores.premiumitems[idx].clone();
    // The original sets stextshold before stextlhold/stextvhold; the order does not matter.
    buy_enter(ctx, TalkID::SmithPremiumBuy, item);
}

/// Original: `StoreGoldFit` (stores.cpp).
// @port stores.cpp|devilution::StoreGoldFit(Item &item) sha=d90b257d4d81
fn store_gold_fit(ctx: &Ctx, item: &Item) -> bool {
    let cost = item._iIvalue;
    let item_size = crate::inv::get_inventory_size(item);
    let item_room_for_gold = item_size.width * item_size.height * ctx.items.MaxGold;
    if cost <= item_room_for_gold {
        return true;
    }
    cost <= item_room_for_gold + crate::inv::room_for_gold(ctx)
}

/// Original: `StoreSellItem` (stores.cpp): sells an item from the player's inventory or belt.
// @port stores.cpp|devilution::StoreSellItem() sha=5329f1e51880
fn store_sell_item(ctx: &mut Ctx) {
    let me = my_player(ctx);
    let mut idx = held_index(ctx) as usize;
    let hidx = ctx.stores.storehidx[idx];
    if hidx >= 0 {
        crate::player::remove_inv_item(ctx, me, hidx as i32, true);
    } else {
        crate::player::remove_spd_bar_item(ctx, me, -(hidx as i32 + 1));
    }
    let cost = ctx.stores.storehold[idx]._iIvalue;
    ctx.stores.storenumh -= 1;
    let n = ctx.stores.storenumh as usize;
    if idx != n {
        while idx < n {
            ctx.stores.storehold[idx] = ctx.stores.storehold[idx + 1].clone();
            ctx.stores.storehidx[idx] = ctx.stores.storehidx[idx + 1];
            idx += 1;
        }
    }
    crate::inv::add_gold_to_inventory(ctx, me, cost);
    ctx.players.Players[me]._pGold += cost;
}

/// The common body of the sell `*Enter` handlers.
fn sell_enter(ctx: &mut Ctx, shold: TalkID) {
    ctx.stores.stextlhold = ctx.stores.stextsel;
    ctx.stores.stextshold = shold;
    ctx.stores.stextvhold = ctx.stores.stextsval;
    let idx = selected_index(ctx) as usize;
    if !store_gold_fit(ctx, &ctx.stores.storehold[idx]) {
        start_store(ctx, TalkID::NoRoom);
        return;
    }
    ctx.stores.store_item = ctx.stores.storehold[idx].clone();
    start_store(ctx, TalkID::Confirm);
}

/// Original: `SmithSellEnter` (stores.cpp).
// @port stores.cpp|devilution::SmithSellEnter() sha=4c753c2030cc
fn smith_sell_enter(ctx: &mut Ctx) {
    if ctx.stores.stextsel == back_button_line(ctx) {
        start_store(ctx, TalkID::Smith);
        ctx.stores.stextsel = 16;
        return;
    }
    sell_enter(ctx, TalkID::SmithSell);
}

/// Original: `SmithRepairItem` (stores.cpp): repairs an item in the player's inventory or body.
// @port stores.cpp|devilution::SmithRepairItem(int price) sha=8985a00c7f2f
fn smith_repair_item(ctx: &mut Ctx, price: i32) {
    let idx = held_index(ctx) as usize;
    ctx.stores.storehold[idx]._iDurability = ctx.stores.storehold[idx]._iMaxDur;
    let i = ctx.stores.storehidx[idx];
    let me = my_player(ctx);
    let p = &mut ctx.players.Players[me];
    if i < 0 {
        let loc = match i {
            -1 => Some(INVLOC_HEAD),
            -2 => Some(INVLOC_CHEST),
            -3 => Some(INVLOC_HAND_LEFT),
            -4 => Some(INVLOC_HAND_RIGHT),
            _ => None,
        };
        if let Some(loc) = loc {
            let it = &mut p.InvBody[loc as usize];
            it._iDurability = it._iMaxDur;
        }
        return;
    }
    let it = &mut p.InvList[i as usize];
    it._iDurability = it._iMaxDur;
    take_plrs_money(ctx, price);
}

/// The common body of the `*Enter` handlers that act on `storehold` for a price.
fn storehold_price_enter(ctx: &mut Ctx, shold: TalkID) {
    ctx.stores.stextshold = shold;
    ctx.stores.stextlhold = ctx.stores.stextsel;
    ctx.stores.stextvhold = ctx.stores.stextsval;
    let idx = selected_index(ctx) as usize;
    if !player_can_afford(ctx, ctx.stores.storehold[idx]._iIvalue) {
        start_store(ctx, TalkID::NoMoney);
        return;
    }
    ctx.stores.store_item = ctx.stores.storehold[idx].clone();
    start_store(ctx, TalkID::Confirm);
}

/// Original: `SmithRepairEnter` (stores.cpp).
// @port stores.cpp|devilution::SmithRepairEnter() sha=5d5304f0892b
fn smith_repair_enter(ctx: &mut Ctx) {
    if ctx.stores.stextsel == back_button_line(ctx) {
        start_store(ctx, TalkID::Smith);
        ctx.stores.stextsel = 18;
        return;
    }
    storehold_price_enter(ctx, TalkID::SmithRepair);
}

/// Original: `WitchEnter` (stores.cpp).
// @port stores.cpp|devilution::WitchEnter() sha=ed34207311ae
fn witch_enter(ctx: &mut Ctx) {
    match ctx.stores.stextsel {
        12 => enter_gossip(ctx, TOWN_WITCH, 12, TalkID::Witch),
        14 => start_store(ctx, TalkID::WitchBuy),
        16 => start_store(ctx, TalkID::WitchSell),
        18 => start_store(ctx, TalkID::WitchRecharge),
        20 => ctx.stores.stextflag = TalkID::None,
        _ => {}
    }
}

/// Original: `WitchBuyItem` (stores.cpp): purchases an item from the witch.
// @port stores.cpp|devilution::WitchBuyItem(Item &item) sha=f07b5b3a4b14
fn witch_buy_item(ctx: &mut Ctx, item: &mut Item) {
    let idx = held_index(ctx);
    if idx < 3 {
        item._iSeed = ctx.rng.advance_rnd_seed() as u32;
    }
    take_plrs_money(ctx, item._iIvalue);
    store_auto_place(ctx, item, true);
    if idx >= 3 {
        remove_from_list(&mut ctx.stores.witchitem, idx as usize);
    }
    let me = my_player(ctx);
    crate::items::calc_plr_inv(ctx, me, true);
}

/// Original: `WitchBuyEnter` (stores.cpp).
// @port stores.cpp|devilution::WitchBuyEnter() sha=8c218fbf9e83
fn witch_buy_enter(ctx: &mut Ctx) {
    if ctx.stores.stextsel == back_button_line(ctx) {
        start_store(ctx, TalkID::Witch);
        ctx.stores.stextsel = 14;
        return;
    }
    let idx = selected_index(ctx) as usize;
    let item = ctx.stores.witchitem[idx].clone();
    buy_enter(ctx, TalkID::WitchBuy, item);
}

/// Original: `WitchSellEnter` (stores.cpp).
// @port stores.cpp|devilution::WitchSellEnter() sha=a8372621b2da
fn witch_sell_enter(ctx: &mut Ctx) {
    if ctx.stores.stextsel == back_button_line(ctx) {
        start_store(ctx, TalkID::Witch);
        ctx.stores.stextsel = 16;
        return;
    }
    sell_enter(ctx, TalkID::WitchSell);
}

/// Original: `WitchRechargeItem` (stores.cpp): recharges an item in the player's inventory or body.
// @port stores.cpp|devilution::WitchRechargeItem(int price) sha=28e307d86bd8
fn witch_recharge_item(ctx: &mut Ctx, price: i32) {
    let idx = held_index(ctx) as usize;
    ctx.stores.storehold[idx]._iCharges = ctx.stores.storehold[idx]._iMaxCharges;
    let me = my_player(ctx);
    let i = ctx.stores.storehidx[idx];
    if i < 0 {
        let it = &mut ctx.players.Players[me].InvBody[INVLOC_HAND_LEFT as usize];
        it._iCharges = it._iMaxCharges;
        crate::msg::net_send_cmd_ch_item(ctx, true, INVLOC_HAND_LEFT as u8, false);
    } else {
        let it = &mut ctx.players.Players[me].InvList[i as usize];
        it._iCharges = it._iMaxCharges;
        crate::msg::net_sync_inv_item(ctx, me, i as i32);
    }
    take_plrs_money(ctx, price);
    crate::items::calc_plr_inv(ctx, me, true);
}

/// Original: `WitchRechargeEnter` (stores.cpp).
// @port stores.cpp|devilution::WitchRechargeEnter() sha=f1d498d9c278
fn witch_recharge_enter(ctx: &mut Ctx) {
    if ctx.stores.stextsel == back_button_line(ctx) {
        start_store(ctx, TalkID::Witch);
        ctx.stores.stextsel = 18;
        return;
    }
    storehold_price_enter(ctx, TalkID::WitchRecharge);
}

/// Original: `BoyEnter` (stores.cpp).
// @port stores.cpp|devilution::BoyEnter() sha=a3c2574488f2
fn boy_enter(ctx: &mut Ctx) {
    let empty = ctx.stores.boyitem.is_empty();
    let sel = ctx.stores.stextsel;
    if !empty && sel == 18 {
        if !player_can_afford(ctx, 50) {
            ctx.stores.stextshold = TalkID::Boy;
            ctx.stores.stextlhold = 18;
            ctx.stores.stextvhold = ctx.stores.stextsval;
            start_store(ctx, TalkID::NoMoney);
        } else {
            take_plrs_money(ctx, 50);
            start_store(ctx, TalkID::BoyBuy);
        }
        return;
    }
    if (sel != 8 && !empty) || (sel != 12 && empty) {
        ctx.stores.stextflag = TalkID::None;
        return;
    }
    ctx.stores.talker = TOWN_PEGBOY;
    ctx.stores.stextshold = TalkID::Boy;
    ctx.stores.stextlhold = sel;
    start_store(ctx, TalkID::Gossip);
}

/// Original: `BoyBuyItem` (stores.cpp).
// @port stores.cpp|devilution::BoyBuyItem(Item &item) sha=1c424beda013
fn boy_buy_item(ctx: &mut Ctx, item: &mut Item) {
    take_plrs_money(ctx, item._iIvalue);
    store_auto_place(ctx, item, true);
    ctx.stores.boyitem.clear();
    ctx.stores.stextshold = TalkID::Boy;
    let me = my_player(ctx);
    crate::items::calc_plr_inv(ctx, me, true);
    ctx.stores.stextlhold = 12;
}

/// Original: `HealerBuyItem` (stores.cpp): purchases an item from the healer.
// @port stores.cpp|devilution::HealerBuyItem(Item &item) sha=3d9e3631ccd2
fn healer_buy_item(ctx: &mut Ctx, item: &mut Item) {
    let idx = held_index(ctx);
    let keep = if !ctx.init.gb_is_multiplayer { 2 } else { 3 };
    if idx < keep {
        item._iSeed = ctx.rng.advance_rnd_seed() as u32;
    }
    take_plrs_money(ctx, item._iIvalue);
    if item._iMagical == ITEM_QUALITY_NORMAL {
        item._iIdentified = false;
    }
    store_auto_place(ctx, item, true);
    if idx < keep {
        return;
    }
    let idx = held_index(ctx) as usize;
    remove_from_list(&mut ctx.stores.healitem, idx);
    let me = my_player(ctx);
    crate::items::calc_plr_inv(ctx, me, true);
}

/// Original: `BoyBuyEnter` (stores.cpp).
// @port stores.cpp|devilution::BoyBuyEnter() sha=b9f9c4549596
fn boy_buy_enter(ctx: &mut Ctx) {
    if ctx.stores.stextsel != 10 {
        ctx.stores.stextflag = TalkID::None;
        return;
    }
    ctx.stores.stextshold = TalkID::BoyBuy;
    ctx.stores.stextvhold = ctx.stores.stextsval;
    ctx.stores.stextlhold = 10;
    let boyitem = ctx.stores.boyitem.clone();
    let mut price = boyitem._iIvalue;
    if ctx.init.gb_is_hellfire {
        price -= boyitem._iIvalue / 4;
    } else {
        price += boyitem._iIvalue / 2;
    }
    if !player_can_afford(ctx, price) {
        start_store(ctx, TalkID::NoMoney);
        return;
    }
    if !store_auto_place(ctx, &boyitem, false) {
        start_store(ctx, TalkID::NoRoom);
        return;
    }
    ctx.stores.store_item = boyitem;
    ctx.stores.store_item._iIvalue = price;
    start_store(ctx, TalkID::Confirm);
}

/// Original: `StorytellerIdentifyItem` (stores.cpp).
// @port stores.cpp|devilution::StorytellerIdentifyItem(Item &item) sha=430f6fa979bf
fn storyteller_identify_item(ctx: &mut Ctx, item: &mut Item) {
    let me = my_player(ctx);
    let idx = ctx.stores.storehidx[held_index(ctx) as usize];
    let p = &mut ctx.players.Players[me];
    if idx < 0 {
        let loc = match idx {
            -1 => Some(INVLOC_HEAD),
            -2 => Some(INVLOC_CHEST),
            -3 => Some(INVLOC_HAND_LEFT),
            -4 => Some(INVLOC_HAND_RIGHT),
            -5 => Some(INVLOC_RING_LEFT),
            -6 => Some(INVLOC_RING_RIGHT),
            -7 => Some(INVLOC_AMULET),
            _ => None,
        };
        if let Some(loc) = loc {
            p.InvBody[loc as usize]._iIdentified = true;
        }
    } else {
        p.InvList[idx as usize]._iIdentified = true;
    }
    item._iIdentified = true;
    take_plrs_money(ctx, item._iIvalue);
    crate::items::calc_plr_inv(ctx, me, true);
}

/// Original: `ConfirmEnter` (stores.cpp).
// @port stores.cpp|devilution::ConfirmEnter(Item &item) sha=354f4ae26988
fn confirm_enter(ctx: &mut Ctx) {
    let mut item = std::mem::take(&mut ctx.stores.store_item);
    if ctx.stores.stextsel == 18 {
        match ctx.stores.stextshold {
            TalkID::SmithBuy => smith_buy_item(ctx, &mut item),
            TalkID::SmithSell | TalkID::WitchSell => store_sell_item(ctx),
            TalkID::SmithRepair => smith_repair_item(ctx, item._iIvalue),
            TalkID::WitchBuy => witch_buy_item(ctx, &mut item),
            TalkID::WitchRecharge => witch_recharge_item(ctx, item._iIvalue),
            TalkID::BoyBuy => boy_buy_item(ctx, &mut item),
            TalkID::HealerBuy => healer_buy_item(ctx, &mut item),
            TalkID::StorytellerIdentify => {
                storyteller_identify_item(ctx, &mut item);
                ctx.stores.store_item = item;
                start_store(ctx, TalkID::StorytellerIdentifyShow);
                return;
            }
            TalkID::SmithPremiumBuy => smith_buy_p_item(ctx, &mut item),
            _ => {}
        }
    }
    ctx.stores.store_item = item;
    start_store(ctx, ctx.stores.stextshold);
    if ctx.stores.stextsel == back_button_line(ctx) {
        return;
    }
    let s = &mut ctx.stores;
    s.stextsel = s.stextlhold;
    s.stextsval = s.stextvhold.min(s.stextsmax);
    while s.stextsel != -1 && !s.stext[s.stextsel as usize].is_selectable() {
        s.stextsel -= 1;
    }
}

/// Original: `HealerEnter` (stores.cpp).
// @port stores.cpp|devilution::HealerEnter() sha=a69abc090f00
fn healer_enter(ctx: &mut Ctx) {
    match ctx.stores.stextsel {
        12 => enter_gossip(ctx, TOWN_HEALER, 12, TalkID::Healer),
        14 => start_store(ctx, TalkID::HealerBuy),
        18 => ctx.stores.stextflag = TalkID::None,
        _ => {}
    }
}

/// Original: `HealerBuyEnter` (stores.cpp).
// @port stores.cpp|devilution::HealerBuyEnter() sha=7a3eb99dc40e
fn healer_buy_enter(ctx: &mut Ctx) {
    if ctx.stores.stextsel == back_button_line(ctx) {
        start_store(ctx, TalkID::Healer);
        ctx.stores.stextsel = 14;
        return;
    }
    let idx = selected_index(ctx) as usize;
    let item = ctx.stores.healitem[idx].clone();
    buy_enter(ctx, TalkID::HealerBuy, item);
}

/// Original: `StorytellerEnter` (stores.cpp).
// @port stores.cpp|devilution::StorytellerEnter() sha=fdcff31e5a17
fn storyteller_enter(ctx: &mut Ctx) {
    match ctx.stores.stextsel {
        12 => enter_gossip(ctx, TOWN_STORY, 12, TalkID::Storyteller),
        14 => start_store(ctx, TalkID::StorytellerIdentify),
        18 => ctx.stores.stextflag = TalkID::None,
        _ => {}
    }
}

/// Original: `StorytellerIdentifyEnter` (stores.cpp).
// @port stores.cpp|devilution::StorytellerIdentifyEnter() sha=5b4ac00442aa
fn storyteller_identify_enter(ctx: &mut Ctx) {
    if ctx.stores.stextsel == back_button_line(ctx) {
        start_store(ctx, TalkID::Storyteller);
        ctx.stores.stextsel = 14;
        return;
    }
    storehold_price_enter(ctx, TalkID::StorytellerIdentify);
}

/// Original: `TalkEnter` (stores.cpp).
// @port stores.cpp|devilution::TalkEnter() sha=463c5d5214b3
fn talk_enter(ctx: &mut Ctx) {
    if ctx.stores.stextsel == back_button_line(ctx) {
        start_store(ctx, ctx.stores.stextshold);
        ctx.stores.stextsel = ctx.stores.stextlhold;
        return;
    }
    let mut sn = ctx.quests.Quests.iter().filter(|q| quest_has_talk(ctx, q)).count() as i32;
    let mut la = 2;
    if sn > 6 {
        sn = 14 - (sn / 2);
        la = 1;
    } else {
        sn = 15 - sn;
    }
    if ctx.stores.stextsel == sn - 2 {
        let target = crate::towners::get_towner(ctx, ctx.stores.talker).expect("towner");
        let gossip = ctx.towners.towners[target].gossip;
        crate::minitext::init_q_text_msg(ctx, gossip);
        return;
    }
    let quests = ctx.quests.Quests;
    for quest in quests.iter() {
        if quest_has_talk(ctx, quest) {
            if sn == ctx.stores.stextsel {
                let msg = ctx.towners.QuestDialogTable[ctx.stores.talker as usize][quest._qidx as usize];
                crate::minitext::init_q_text_msg(ctx, msg);
            }
            sn += la;
        }
    }
}

/// Original: `TavernEnter` (stores.cpp).
// @port stores.cpp|devilution::TavernEnter() sha=fbe04ae8ccf1
fn tavern_enter(ctx: &mut Ctx) {
    match ctx.stores.stextsel {
        12 => enter_gossip(ctx, TOWN_TAVERN, 12, TalkID::Tavern),
        18 => ctx.stores.stextflag = TalkID::None,
        _ => {}
    }
}

/// Original: `BarmaidEnter` (stores.cpp).
// @port stores.cpp|devilution::BarmaidEnter() sha=c664bf5045bf
fn barmaid_enter(ctx: &mut Ctx) {
    match ctx.stores.stextsel {
        12 => enter_gossip(ctx, TOWN_BMAID, 12, TalkID::Barmaid),
        14 => {
            ctx.stores.stextflag = TalkID::None;
            ctx.stash.IsStashOpen = true;
            crate::qol::stash::refresh_item_stat_flags(ctx);
            ctx.inv.invflag = true;
            if ctx.controls.control_mode != crate::controls::ControlTypes::KeyboardAndMouse {
                if ctx.cursor.pcurs == crate::cursor::CURSOR_DISARM {
                    crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_HAND);
                }
                crate::controls::plrctrls::focus_on_inventory(ctx);
            }
        }
        18 => ctx.stores.stextflag = TalkID::None,
        _ => {}
    }
}

/// Original: `DrunkEnter` (stores.cpp).
// @port stores.cpp|devilution::DrunkEnter() sha=3892bfc28cb8
fn drunk_enter(ctx: &mut Ctx) {
    match ctx.stores.stextsel {
        12 => enter_gossip(ctx, TOWN_DRUNK, 12, TalkID::Drunk),
        18 => ctx.stores.stextflag = TalkID::None,
        _ => {}
    }
}

/// Original: `TakeGold` (stores.cpp).
// @port stores.cpp|devilution::TakeGold(Player &player, int cost, bool skipMaxPiles) sha=a4d84991834c
fn take_gold(ctx: &mut Ctx, pnum: usize, mut cost: i32, skip_max_piles: bool) -> i32 {
    let max_gold = ctx.items.MaxGold;
    let mut i: i32 = 0;
    while i < ctx.players.Players[pnum]._pNumInv {
        let item = &mut ctx.players.Players[pnum].InvList[i as usize];
        if item._itype != ItemType::Gold || (skip_max_piles && item._ivalue == max_gold) {
            i += 1;
            continue;
        }
        if cost < item._ivalue {
            item._ivalue -= cost;
            crate::items::set_plr_hand_gold_curs(item);
            return 0;
        }
        cost -= item._ivalue;
        crate::player::remove_inv_item(ctx, pnum, i, true);
        i = 0;
    }
    cost
}

/// Original: `DrawSelector` (stores.cpp).
// @port stores.cpp|devilution::DrawSelector(const Surface &out, const Rectangle &rect, string_view text, UiFlags flags) sha=5e74901b7d29
fn draw_selector(ctx: &mut Ctx, out: &Surface, rect: Rect, text: &str, flags: UiFlags) {
    let line_width = get_line_width(ctx, text, GameFontTables::GameFont12, 1, None);
    let mut x1 = rect.x - 20;
    if flags.has(UiFlags::ALIGN_CENTER) {
        x1 += (rect.w - line_width) / 2;
    }
    let cels = ctx.text_render.p_s_pent_spn2_cels.clone().expect("pSPentSpn2Cels");
    let spin = crate::engine::render::text_render::pent_spn2_spin(ctx) as usize;
    clx_draw(out, (x1, rect.y + 13), &cels.get(spin));
    let mut x2 = rect.x + rect.w + 5;
    if flags.has(UiFlags::ALIGN_CENTER) {
        x2 = rect.x + (rect.w - line_width) / 2 + line_width + 5;
    }
    let spin = crate::engine::render::text_render::pent_spn2_spin(ctx) as usize;
    clx_draw(out, (x2, rect.y + 13), &cels.get(spin));
}

/// Original: `devilution::AddStoreHoldRepair` (stores.cpp).
// @port stores.cpp|devilution::AddStoreHoldRepair(Item *itm, int8_t i) sha=e175bd7fe8d1
pub fn add_store_hold_repair(ctx: &mut Ctx, itm: &Item, i: i8) {
    let n = ctx.stores.storenumh as usize;
    ctx.stores.storehold[n] = itm.clone();
    let item = &mut ctx.stores.storehold[n];
    let due = item._iMaxDur - item._iDurability;
    let v;
    if item._iMagical != ITEM_QUALITY_NORMAL && item._iIdentified {
        v = 30 * item._iIvalue * due / (item._iMaxDur * 100 * 2);
        if v == 0 {
            return;
        }
    } else {
        v = (item._ivalue * due / (item._iMaxDur * 2)).max(1);
    }
    item._iIvalue = v;
    item._ivalue = v;
    ctx.stores.storehidx[n] = i;
    ctx.stores.storenumh += 1;
}

/// Original: `devilution::InitStores` (stores.cpp).
// @port stores.cpp|devilution::InitStores() sha=17ea5a705e3b
pub fn init_stores(ctx: &mut Ctx) {
    clear_s_text(ctx, 0, STORE_LINES as i32);
    let s = &mut ctx.stores;
    s.stextflag = TalkID::None;
    s.stextsize = false;
    s.stextscrl = false;
    s.numpremium = 0;
    s.premiumlevel = 1;
    for premiumitem in s.premiumitems.iter_mut() {
        premiumitem.clear();
    }
    s.boyitem.clear();
    s.boylevel = 0;
}

/// Original: `devilution::SetupTownStores` (stores.cpp).
// @port stores.cpp|devilution::SetupTownStores() sha=79ebac62aea1
pub fn setup_town_stores(ctx: &mut Ctx) {
    let me = my_player(ctx);
    let mut l = ctx.players.Players[me]._pLevel as i32 / 2;
    if !ctx.init.gb_is_multiplayer {
        l = 0;
        for i in 0..crate::player::NUMLEVELS {
            if ctx.players.Players[me]._pLvlVisited[i] {
                l = i as i32;
            }
        }
    } else {
        let seed = ctx.diablo.glSeedTbl[ctx.gendung.currlevel as usize].wrapping_mul(ctx.platform.ticks());
        ctx.rng.set_rnd_seed(seed);
    }
    l = (l + 2).clamp(6, 16);
    crate::items::spawn_smith(ctx, l);
    crate::items::spawn_witch(ctx, l);
    crate::items::spawn_healer(ctx, l);
    let plvl = ctx.players.Players[me]._pLevel as i32;
    crate::items::spawn_boy(ctx, plvl);
    let player = ctx.players.Players[me].clone();
    crate::items::spawn_premium(ctx, &player);
}

/// Original: `devilution::FreeStoreMem` (stores.cpp).
// @port stores.cpp|devilution::FreeStoreMem() sha=5620ac8a3ec1
pub fn free_store_mem(ctx: &mut Ctx) {
    if ctx.options.gameplay.show_item_graphics_in_stores.get() {
        crate::cursor::free_half_size_item_sprites(ctx);
    }
    ctx.stores.stextflag = TalkID::None;
    for entry in ctx.stores.stext.iter_mut() {
        entry.text = String::new();
    }
}

/// Original: `devilution::PrintSString` (stores.cpp).
// @port stores.cpp|devilution::PrintSString(const Surface &out, int margin, int line, string_view text, UiFlags flags, int price, int cursId, bool cursIndent) sha=c1de12191dd6
#[allow(clippy::too_many_arguments)]
pub fn print_s_string(ctx: &mut Ctx, out: &Surface, margin: i32, line: usize, text: &str, flags: UiFlags, price: i32, curs_id: i32, curs_indent: bool) {
    let ui = crate::utils::display::get_ui_rectangle(ctx);
    let s = &ctx.stores;
    let mut sx = ui.x + 32 + margin;
    if !s.stextsize {
        sx += 320;
    }
    let sy = ui.y + PADDING_TOP + s.stext[line].y + s.stext[line]._syoff as i32;
    let mut width = if s.stextsize { 575 } else { 255 };
    if s.stextscrl && (4..=20).contains(&line) {
        width -= 9; // Space for the selector
    }
    width -= margin * 2;
    let rect = Rect::new(sx, sy, width, 0);

    // Space reserved for item graphic is based on the size of 2x3 cursor sprites
    const CURS_WIDTH: i32 = crate::inv::INV_SLOT_SIZE_PX * 2;
    const HALF_CURS_WIDTH: i32 = CURS_WIDTH / 2;

    let show_graphics = ctx.options.gameplay.show_item_graphics_in_stores.get();
    if show_graphics && curs_id >= 0 {
        let size: Size = crate::cursor::get_inv_item_size(crate::cursor::CURSOR_FIRSTITEM + curs_id);
        let use_half_size = size.width > crate::inv::INV_SLOT_SIZE_PX || size.height > crate::inv::INV_SLOT_SIZE_PX;
        let use_red = flags.has(UiFlags::COLOR_RED);
        let sprite = if use_half_size {
            if use_red {
                crate::cursor::get_half_size_item_sprite_red(ctx, curs_id)
            } else {
                crate::cursor::get_half_size_item_sprite(ctx, curs_id)
            }
        } else {
            crate::cursor::get_inv_item_sprite(ctx, crate::cursor::CURSOR_FIRSTITEM + curs_id)
        };
        let position = Point::new(rect.x + (HALF_CURS_WIDTH - sprite.width() as i32) / 2, rect.y + (text_height(ctx) * 3 + sprite.height() as i32) / 2);
        if use_half_size || !use_red {
            clx_draw(out, (position.x, position.y), &sprite);
        } else {
            clx_draw_trn(out, (position.x, position.y), &sprite, crate::engine::trn::get_infravision_trn(ctx));
        }
    }

    if show_graphics && curs_indent {
        let text_rect = Rect::new(rect.x + HALF_CURS_WIDTH + 8, rect.y, rect.w - HALF_CURS_WIDTH + 8, rect.h);
        draw_string(ctx, out, text, text_rect, flags, 1, -1);
    } else {
        draw_string(ctx, out, text, rect, flags, 1, -1);
    }
    if price > 0 {
        draw_string(ctx, out, &format_integer(price), rect, flags | UiFlags::ALIGN_RIGHT, 1, -1);
    }
    if ctx.stores.stextsel == line as i32 {
        draw_selector(ctx, out, rect, text, flags);
    }
}

/// Original: `devilution::DrawSLine` (stores.cpp).
// @port stores.cpp|devilution::DrawSLine(const Surface &out, int sy) sha=ab8c6da6851b
pub fn draw_s_line(ctx: &mut Ctx, out: &Surface, sy: i32) {
    let ui = crate::utils::display::get_ui_rectangle(ctx);
    let mut sx = 26;
    let mut width = 587;
    if !ctx.stores.stextsize {
        sx += crate::control::SIDE_PANEL_SIZE.0;
        width -= crate::control::SIDE_PANEL_SIZE.0;
    }
    for i in 0..3 {
        for x in 0..width {
            let v = out.get(ui.x + sx + x, ui.y + 25 + i);
            out.put(ui.x + sx + x, sy + i, v);
        }
    }
}

/// Original: `devilution::DrawSTextHelp` (stores.cpp).
// @port stores.cpp|devilution::DrawSTextHelp() sha=3078e8f289e3
pub fn draw_s_text_help(ctx: &mut Ctx) {
    ctx.stores.stextsel = -1;
    ctx.stores.stextsize = true;
}

/// Original: `devilution::ClearSText` (stores.cpp).
// @port stores.cpp|devilution::ClearSText(int s, int e) sha=8635e0a91312
pub fn clear_s_text(ctx: &mut Ctx, s: i32, e: i32) {
    for i in s..e {
        let t = &mut ctx.stores.stext[i as usize];
        t._sx = 0;
        t._syoff = 0;
        t.text = String::new();
        t.flags = UiFlags::NONE;
        t.type_ = STextType::Label;
        t._sval = 0;
    }
}

/// Original: `devilution::StartStore` (stores.cpp).
// @port stores.cpp|devilution::StartStore(TalkID s) sha=23dc0e6ed884
pub fn start_store(ctx: &mut Ctx, s: TalkID) {
    if ctx.options.gameplay.show_item_graphics_in_stores.get() {
        crate::cursor::create_half_size_item_sprites(ctx);
    }
    ctx.control.sbookflag = false;
    crate::inv::close_inventory(ctx);
    crate::control::close_char_panel(ctx);
    ctx.stores.render_gold = false;
    ctx.quests.QuestLogIsOpen = false;
    crate::control::close_gold_drop(ctx);
    clear_s_text(ctx, 0, STORE_LINES as i32);
    release_store_btn(ctx);
    match s {
        TalkID::Smith => start_smith(ctx),
        TalkID::SmithBuy => {
            let has_any_items = !ctx.stores.smithitem[0].is_empty();
            if has_any_items {
                start_smith_buy(ctx);
            } else {
                ctx.stores.stextflag = TalkID::SmithBuy;
                ctx.stores.stextlhold = 12;
                store_esc(ctx);
                return;
            }
        }
        TalkID::SmithSell => start_smith_sell(ctx),
        TalkID::SmithRepair => start_smith_repair(ctx),
        TalkID::Witch => start_witch(ctx),
        TalkID::WitchBuy => {
            if ctx.stores.storenumh > 0 {
                start_witch_buy(ctx);
            }
        }
        TalkID::WitchSell => start_witch_sell(ctx),
        TalkID::WitchRecharge => start_witch_recharge(ctx),
        TalkID::NoMoney => store_no_money(ctx),
        TalkID::NoRoom => store_no_room(ctx),
        TalkID::Confirm => {
            let item = ctx.stores.store_item.clone();
            store_confirm(ctx, &item);
        }
        TalkID::Boy => start_boy(ctx),
        TalkID::BoyBuy => s_start_boy_buy(ctx),
        TalkID::Healer => start_healer(ctx),
        TalkID::Storyteller => start_storyteller(ctx),
        TalkID::HealerBuy => {
            if ctx.stores.storenumh > 0 {
                start_healer_buy(ctx);
            }
        }
        TalkID::StorytellerIdentify => start_storyteller_identify(ctx),
        TalkID::SmithPremiumBuy => {
            if !start_smith_premium_buy(ctx) {
                return;
            }
        }
        TalkID::Gossip => start_talk(ctx),
        TalkID::StorytellerIdentifyShow => {
            let item = ctx.stores.store_item.clone();
            start_storyteller_identify_show(ctx, &item);
        }
        TalkID::Tavern => start_tavern(ctx),
        TalkID::Drunk => start_drunk(ctx),
        TalkID::Barmaid => start_barmaid(ctx),
        TalkID::None => {}
    }
    ctx.stores.stextsel = -1;
    for i in 0..STORE_LINES {
        if ctx.stores.stext[i].is_selectable() {
            ctx.stores.stextsel = i as i32;
            break;
        }
    }
    ctx.stores.stextflag = s;
}

/// Original: `devilution::DrawSText` (stores.cpp).
// @port stores.cpp|devilution::DrawSText(const Surface &out) sha=30169e02452b
pub fn draw_s_text(ctx: &mut Ctx, out: &Surface) {
    if !ctx.stores.stextsize {
        draw_s_text_back(ctx, out);
    } else {
        crate::minitext::draw_q_text_back(ctx, out);
    }
    if ctx.stores.stextscrl {
        let sval = ctx.stores.stextsval;
        match ctx.stores.stextflag {
            TalkID::SmithBuy => scroll_smith_buy(ctx, sval),
            TalkID::SmithSell | TalkID::SmithRepair | TalkID::WitchSell | TalkID::WitchRecharge | TalkID::StorytellerIdentify => scroll_smith_sell(ctx, sval),
            TalkID::WitchBuy => scroll_witch_buy(ctx, sval),
            TalkID::HealerBuy => scroll_healer_buy(ctx, sval),
            TalkID::SmithPremiumBuy => scroll_smith_premium_buy(ctx, sval),
            _ => {}
        }
    }
    calculate_line_heights(ctx);
    let ui = crate::utils::display::get_ui_rectangle(ctx);
    for i in 0..STORE_LINES {
        let e = ctx.stores.stext[i].clone();
        if e.is_divider() {
            let y = ui.y + PADDING_TOP + e.y + text_height(ctx) / 2;
            draw_s_line(ctx, out, y);
        } else if e.has_text() {
            print_s_string(ctx, out, e._sx as i32, i, &e.text, e.flags, e._sval, e.curs_id, e.curs_indent);
        }
    }
    if ctx.stores.render_gold {
        let text = fmt1(&tr("Your gold: {:s}"), &format_integer(total_player_gold(ctx) as i32));
        print_s_string(ctx, out, 28, 1, &text, UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_RIGHT, 0, -1, false);
    }
    if ctx.stores.stextscrl {
        draw_s_slider(ctx, out, 4, 20);
    }
}

/// Closes the quest text if it is shown (`StoreESC`, `StoreEnter`, `CheckStoreBtn`).
fn close_q_text(ctx: &mut Ctx) {
    crate::minitext::set_qtextflag(ctx, false);
    if ctx.gendung.leveltype == crate::levels::gendung::DungeonType::Town {
        crate::effects::stream_stop(ctx);
    }
}

/// Original: `devilution::StoreESC` (stores.cpp).
// @port stores.cpp|devilution::StoreESC() sha=de5e43a22a95
pub fn store_esc(ctx: &mut Ctx) {
    if crate::minitext::qtextflag(ctx) {
        close_q_text(ctx);
        return;
    }
    let back = |ctx: &mut Ctx, store: TalkID, sel: i32| {
        start_store(ctx, store);
        ctx.stores.stextsel = sel;
    };
    match ctx.stores.stextflag {
        TalkID::Smith | TalkID::Witch | TalkID::Boy | TalkID::BoyBuy | TalkID::Healer | TalkID::Storyteller | TalkID::Tavern | TalkID::Drunk | TalkID::Barmaid => {
            ctx.stores.stextflag = TalkID::None;
        }
        TalkID::Gossip => back(ctx, ctx.stores.stextshold, ctx.stores.stextlhold),
        TalkID::SmithBuy => back(ctx, TalkID::Smith, 12),
        TalkID::SmithPremiumBuy => back(ctx, TalkID::Smith, 14),
        TalkID::SmithSell => back(ctx, TalkID::Smith, 16),
        TalkID::SmithRepair => back(ctx, TalkID::Smith, 18),
        TalkID::WitchBuy => back(ctx, TalkID::Witch, 14),
        TalkID::WitchSell => back(ctx, TalkID::Witch, 16),
        TalkID::WitchRecharge => back(ctx, TalkID::Witch, 18),
        TalkID::HealerBuy => back(ctx, TalkID::Healer, 14),
        TalkID::StorytellerIdentify => back(ctx, TalkID::Storyteller, 14),
        TalkID::StorytellerIdentifyShow => start_store(ctx, TalkID::StorytellerIdentify),
        TalkID::NoMoney | TalkID::NoRoom | TalkID::Confirm => {
            start_store(ctx, ctx.stores.stextshold);
            ctx.stores.stextsel = ctx.stores.stextlhold;
            ctx.stores.stextsval = ctx.stores.stextvhold;
        }
        TalkID::None => {}
    }
}

/// Moves `stextsel` backwards (wrapping) to the previous selectable line.
fn select_previous(s: &mut StoresState) {
    while !s.stext[s.stextsel as usize].is_selectable() {
        if s.stextsel == 0 {
            s.stextsel = STORE_LINES as i32 - 1;
        } else {
            s.stextsel -= 1;
        }
    }
}

/// Moves `stextsel` forwards (wrapping) to the next selectable line.
fn select_next(s: &mut StoresState) {
    while !s.stext[s.stextsel as usize].is_selectable() {
        if s.stextsel == STORE_LINES as i32 - 1 {
            s.stextsel = 0;
        } else {
            s.stextsel += 1;
        }
    }
}

/// Original: `devilution::StoreUp` (stores.cpp).
// @port stores.cpp|devilution::StoreUp() sha=991590c00c2e
pub fn store_up(ctx: &mut Ctx) {
    crate::effects::play_sfx(ctx, crate::effects::IS_TITLEMOV);
    let s = &mut ctx.stores;
    if s.stextsel == -1 {
        return;
    }
    if s.stextscrl {
        if s.stextsel == s.stextup {
            if s.stextsval != 0 {
                s.stextsval -= 1;
            }
            return;
        }
        s.stextsel -= 1;
        select_previous(s);
        return;
    }
    if s.stextsel == 0 {
        s.stextsel = STORE_LINES as i32 - 1;
    } else {
        s.stextsel -= 1;
    }
    select_previous(s);
}

/// Original: `devilution::StoreDown` (stores.cpp).
// @port stores.cpp|devilution::StoreDown() sha=ccf7f62dae1e
pub fn store_down(ctx: &mut Ctx) {
    crate::effects::play_sfx(ctx, crate::effects::IS_TITLEMOV);
    let s = &mut ctx.stores;
    if s.stextsel == -1 {
        return;
    }
    if s.stextscrl {
        if s.stextsel == s.stextdown {
            if s.stextsval < s.stextsmax {
                s.stextsval += 1;
            }
            return;
        }
        s.stextsel += 1;
        select_next(s);
        return;
    }
    if s.stextsel == STORE_LINES as i32 - 1 {
        s.stextsel = 0;
    } else {
        s.stextsel += 1;
    }
    select_next(s);
}

/// Original: `devilution::StorePrior` (stores.cpp).
// @port stores.cpp|devilution::StorePrior() sha=a1e4a3d2b70c
pub fn store_prior(ctx: &mut Ctx) {
    crate::effects::play_sfx(ctx, crate::effects::IS_TITLEMOV);
    let s = &mut ctx.stores;
    if s.stextsel != -1 && s.stextscrl {
        if s.stextsel == s.stextup {
            s.stextsval = (s.stextsval - 4).max(0);
        } else {
            s.stextsel = s.stextup;
        }
    }
}

/// Original: `devilution::StoreNext` (stores.cpp).
// @port stores.cpp|devilution::StoreNext() sha=94459b498272
pub fn store_next(ctx: &mut Ctx) {
    crate::effects::play_sfx(ctx, crate::effects::IS_TITLEMOV);
    let s = &mut ctx.stores;
    if s.stextsel != -1 && s.stextscrl {
        if s.stextsel == s.stextdown {
            if s.stextsval < s.stextsmax {
                s.stextsval += 4;
            }
            if s.stextsval > s.stextsmax {
                s.stextsval = s.stextsmax;
            }
        } else {
            s.stextsel = s.stextdown;
        }
    }
}

/// Original: `devilution::TakePlrsMoney` (stores.cpp).
// @port stores.cpp|devilution::TakePlrsMoney(int cost) sha=b0caa8d7eeb6
pub fn take_plrs_money(ctx: &mut Ctx, mut cost: i32) {
    let me = my_player(ctx);
    let gold = ctx.players.Players[me]._pGold;
    ctx.players.Players[me]._pGold -= cost.min(gold);
    cost = take_gold(ctx, me, cost, true);
    if cost != 0 {
        cost = take_gold(ctx, me, cost, false);
    }
    ctx.stash.Stash.gold -= cost;
    ctx.stash.Stash.dirty = true;
}

/// Original: `devilution::StoreEnter` (stores.cpp).
// @port stores.cpp|devilution::StoreEnter() sha=c7d61b399cc3
pub fn store_enter(ctx: &mut Ctx) {
    if crate::minitext::qtextflag(ctx) {
        close_q_text(ctx);
        return;
    }
    crate::effects::play_sfx(ctx, crate::effects::IS_TITLSLCT);
    match ctx.stores.stextflag {
        TalkID::Smith => smith_enter(ctx),
        TalkID::SmithPremiumBuy => smith_premium_buy_enter(ctx),
        TalkID::SmithBuy => smith_buy_enter(ctx),
        TalkID::SmithSell => smith_sell_enter(ctx),
        TalkID::SmithRepair => smith_repair_enter(ctx),
        TalkID::Witch => witch_enter(ctx),
        TalkID::WitchBuy => witch_buy_enter(ctx),
        TalkID::WitchSell => witch_sell_enter(ctx),
        TalkID::WitchRecharge => witch_recharge_enter(ctx),
        TalkID::NoMoney | TalkID::NoRoom => {
            start_store(ctx, ctx.stores.stextshold);
            ctx.stores.stextsel = ctx.stores.stextlhold;
            ctx.stores.stextsval = ctx.stores.stextvhold;
        }
        TalkID::Confirm => confirm_enter(ctx),
        TalkID::Boy => boy_enter(ctx),
        TalkID::BoyBuy => boy_buy_enter(ctx),
        TalkID::Healer => healer_enter(ctx),
        TalkID::Storyteller => storyteller_enter(ctx),
        TalkID::HealerBuy => healer_buy_enter(ctx),
        TalkID::StorytellerIdentify => storyteller_identify_enter(ctx),
        TalkID::Gossip => talk_enter(ctx),
        TalkID::StorytellerIdentifyShow => start_store(ctx, TalkID::StorytellerIdentify),
        TalkID::Drunk => drunk_enter(ctx),
        TalkID::Tavern => tavern_enter(ctx),
        TalkID::Barmaid => barmaid_enter(ctx),
        TalkID::None => {}
    }
}

/// Original: `devilution::CheckStoreBtn` (stores.cpp).
// @port stores.cpp|devilution::CheckStoreBtn() sha=4c79218fff94
pub fn check_store_btn(ctx: &mut Ctx) {
    let ui = crate::utils::display::get_ui_rectangle(ctx);
    let (mx, my) = ctx.diablo.mouse_position;
    if crate::minitext::qtextflag(ctx) {
        close_q_text(ctx);
    } else if ctx.stores.stextsel != -1 && my >= (PADDING_TOP + ui.y) && my <= (320 + ui.y) {
        if !ctx.stores.stextsize {
            if mx < 344 + ui.x || mx > 616 + ui.x {
                return;
            }
        } else if mx < 24 + ui.x || mx > 616 + ui.x {
            return;
        }
        let relative_y = my - (ui.y + PADDING_TOP);
        if ctx.stores.stextscrl && mx > 600 + ui.x {
            // Scroll bar is always measured in terms of the small line height.
            let y = relative_y / SMALL_LINE_HEIGHT;
            if y == 4 {
                if ctx.stores.stextscrlubtn <= 0 {
                    store_up(ctx);
                    ctx.stores.stextscrlubtn = 10;
                } else {
                    ctx.stores.stextscrlubtn -= 1;
                }
            }
            if y == 20 {
                if ctx.stores.stextscrldbtn <= 0 {
                    store_down(ctx);
                    ctx.stores.stextscrldbtn = 10;
                } else {
                    ctx.stores.stextscrldbtn -= 1;
                }
            }
            return;
        }
        let mut y = relative_y / line_height(ctx);
        // Large small fonts draw beyond LineHeight. Check if the click was on the overflow text.
        let st = &ctx.stores.stext;
        if is_small_font_tall(ctx) && y > 0 && y < STORE_LINES as i32 && st[y as usize - 1].has_text() && !st[y as usize].has_text() && relative_y < st[y as usize - 1].y + LARGE_TEXT_HEIGHT {
            y -= 1;
        }
        if y >= 5 {
            let back = back_button_line(ctx);
            if y >= back + 1 {
                y = back;
            }
            let st = &ctx.stores.stext;
            if ctx.stores.stextscrl && y <= 20 && !st[y as usize].is_selectable() {
                if st[y as usize - 2].is_selectable() {
                    y -= 2;
                } else if st[y as usize - 1].is_selectable() {
                    y -= 1;
                }
            }
            if ctx.stores.stext[y as usize].is_selectable() || (ctx.stores.stextscrl && y == back) {
                ctx.stores.stextsel = y;
                store_enter(ctx);
            }
        }
    }
}

/// Original: `devilution::ReleaseStoreBtn` (stores.cpp).
// @port stores.cpp|devilution::ReleaseStoreBtn() sha=17a1f6c20aff
pub fn release_store_btn(ctx: &mut Ctx) {
    ctx.stores.stextscrlubtn = -1;
    ctx.stores.stextscrldbtn = -1;
}

