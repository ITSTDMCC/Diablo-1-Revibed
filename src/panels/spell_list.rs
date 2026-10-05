//! `Source/panels/spell_list.cpp`: the current spell button and the speed book.

use crate::control::{add_panel_string, get_main_panel};
use crate::controls::ControlTypes;
use crate::ctx::Ctx;
use crate::engine::geometry::{Displacement, Point};
use crate::engine::render::text_render::{draw_string_at, get_line_width, GameFontTables, UiFlags};
use crate::engine::surface::Surface;
use crate::enums::*;
use crate::items::{get_spell_data, MAX_SPELLS};
use crate::panels::spell_icons::*;
use crate::player::NumHotkeys;
use crate::spells::{get_spell_bitmask, is_valid_spell};
use crate::utils::language::{is_small_font_tall, ngettext, pgettext, tr};

const SPLROWICONLS: i32 = 10;

/// `SpellListItem`
#[derive(Clone, Copy, Debug)]
pub struct SpellListItem {
    pub location: Point,
    pub type_: SpellType,
    pub id: SpellID,
    pub is_selected: bool,
}

const SPELL_TYPES: [SpellType; 5] = [SpellType::Skill, SpellType::Spell, SpellType::Scroll, SpellType::Charges, SpellType::Invalid];

fn my_player(ctx: &Ctx) -> usize {
    ctx.players.MyPlayer.expect("MyPlayer")
}

fn fmt1(fmt: &str, arg: &str) -> String {
    match (fmt.find('{'), fmt.find('}')) {
        (Some(a), Some(b)) if b > a => format!("{}{}{}", &fmt[..a], arg, &fmt[b + 1..]),
        _ => fmt.to_string(),
    }
}

/// The spell mask of `player` for a spell type (`None` for `SpellType::Invalid`).
fn spell_mask(ctx: &Ctx, pnum: usize, t: SpellType) -> Option<u64> {
    let p = &ctx.players.Players[pnum];
    match t {
        SpellType::Skill => Some(p._pAblSpells),
        SpellType::Spell => Some(p._pMemSpells),
        SpellType::Scroll => Some(p._pScrlSpells),
        SpellType::Charges => Some(p._pISpells),
        SpellType::Invalid => None,
    }
}

/// Original: `PrintSBookSpellType` (panels/spell_list.cpp).
// @port panels/spell_list.cpp|devilution::PrintSBookSpellType(const Surface &out, Point position, string_view text, uint8_t rectColorIndex) sha=c71a75f95673
fn print_s_book_spell_type(ctx: &mut Ctx, out: &Surface, mut position: Point, text: &str, rect_color_index: u8) {
    draw_large_spell_icon_border(ctx, out, position, rect_color_index);
    // Align the spell type text with bottom of spell icon
    let w = get_line_width(ctx, text, GameFontTables::GameFont12, 1, None);
    position = position + Displacement::new(SPLICONLENGTH / 2 - w / 2, if is_small_font_tall(ctx) { -19 } else { -15 });
    // Then draw the text over the top
    draw_string_at(ctx, out, text, (position.x, position.y), UiFlags::COLOR_WHITE | UiFlags::OUTLINED, 1, -1);
}

/// Original: `PrintSBookHotkey` (panels/spell_list.cpp).
// @port panels/spell_list.cpp|devilution::PrintSBookHotkey(const Surface &out, Point position, const string_view text) sha=f4791f4cdd66
fn print_s_book_hotkey(ctx: &mut Ctx, out: &Surface, mut position: Point, text: &str) {
    // Align the hot key text with the top-right corner of the spell icon
    let w = get_line_width(ctx, text, GameFontTables::GameFont12, 1, None);
    position = position + Displacement::new(SPLICONLENGTH - (w + 5), 5 - SPLICONLENGTH);
    // Then draw the text over the top
    draw_string_at(ctx, out, text, (position.x, position.y), UiFlags::COLOR_WHITE | UiFlags::OUTLINED, 1, -1);
}

/// Original: `GetSpellListSelection` (panels/spell_list.cpp).
// @port panels/spell_list.cpp|devilution::GetSpellListSelection(SpellID &pSpell, SpellType &pSplType) sha=5f2f567069f6
fn get_spell_list_selection(ctx: &Ctx) -> Option<(SpellID, SpellType)> {
    let me = my_player(ctx);
    for item in get_spell_list_items(ctx) {
        if item.is_selected {
            let mut spl_type = item.type_;
            if ctx.players.Players[me]._pClass == HeroClass::Monk && item.id == SpellID::Search {
                spl_type = SpellType::Skill;
            }
            return Some((item.id, spl_type));
        }
    }
    None
}

/// Original: `GetHotkeyName` (panels/spell_list.cpp).
// @port panels/spell_list.cpp|devilution::GetHotkeyName(SpellID spellId, SpellType spellType, bool useShortName = false) sha=da397c46783b
fn get_hotkey_name(ctx: &Ctx, spell_id: SpellID, spell_type: SpellType, use_short_name: bool) -> Option<String> {
    let p = &ctx.players.Players[my_player(ctx)];
    for t in 0..NumHotkeys {
        if p._pSplHotKey[t] != spell_id || p._pSplTHotKey[t] != spell_type {
            continue;
        }
        let quick_spell_action_key = format!("QuickSpell{}", t + 1);
        if ctx.controls.control_mode == ControlTypes::Gamepad {
            return Some(ctx.options.padmapper.input_name_for_action(ctx, &quick_spell_action_key, use_short_name));
        }
        return Some(ctx.options.keymapper.key_name_for_action(&quick_spell_action_key));
    }
    None
}

/// Original: `devilution::DrawSpell` (panels/spell_list.cpp).
// @port panels/spell_list.cpp|devilution::DrawSpell(const Surface &out) sha=081ed81c578d
pub fn draw_spell(ctx: &mut Ctx, out: &Surface) {
    let me = my_player(ctx);
    let mut spl = ctx.players.Players[me]._pRSpell;
    let mut st = ctx.players.Players[me]._pRSplType;
    if !is_valid_spell(ctx, spl) {
        st = SpellType::Invalid;
        spl = SpellID::Null;
    }
    if st == SpellType::Spell {
        let tlvl = ctx.players.Players[me].get_spell_level(spl);
        if crate::spells::check_spell(ctx, me, spl, st, true) != SpellCheckResult::Success {
            st = SpellType::Invalid;
        }
        if tlvl <= 0 {
            st = SpellType::Invalid;
        }
    }
    if ctx.gendung.leveltype == crate::levels::gendung::DungeonType::Town && st != SpellType::Invalid && !get_spell_data(spl).is_allowed_in_town() {
        st = SpellType::Invalid;
    }
    set_spell_trans(ctx, st);
    let main = get_main_panel(ctx);
    let position = Point::new(main.x + 565, main.y + 119);
    draw_large_spell_icon(ctx, out, position, spl);
    let r_type = ctx.players.Players[me]._pRSplType;
    if let Some(hotkey_name) = get_hotkey_name(ctx, spl, r_type, true) {
        print_s_book_hotkey(ctx, out, position, &hotkey_name);
    }
}

/// Original: `devilution::DrawSpellList` (panels/spell_list.cpp).
// @port panels/spell_list.cpp|devilution::DrawSpellList(const Surface &out) sha=8e84c06c9ab6
pub fn draw_spell_list(ctx: &mut Ctx, out: &Surface) {
    ctx.control.info_string.clear();
    let me = my_player(ctx);
    for item in get_spell_list_items(ctx) {
        let spell_id = item.id;
        let mut trans_type = item.type_;
        let mut spell_level = 0;
        let spell_data_item = get_spell_data(item.id);
        if ctx.gendung.leveltype == crate::levels::gendung::DungeonType::Town && !spell_data_item.is_allowed_in_town() {
            trans_type = SpellType::Invalid;
        }
        if item.type_ == SpellType::Spell {
            spell_level = ctx.players.Players[me].get_spell_level(item.id);
            if spell_level == 0 {
                trans_type = SpellType::Invalid;
            }
        }
        set_spell_trans(ctx, trans_type);
        draw_large_spell_icon(ctx, out, item.location, spell_id);
        if let Some(short_hotkey_name) = get_hotkey_name(ctx, spell_id, item.type_, true) {
            print_s_book_hotkey(ctx, out, item.location, &short_hotkey_name);
        }
        if !item.is_selected {
            continue;
        }
        let mut spell_color = PAL16_GRAY + 5;
        let name = pgettext("spell", spell_data_item.sNameText);
        let in_town = ctx.players.Players[me].is_on_level(0);
        match item.type_ {
            SpellType::Skill => {
                spell_color = PAL16_YELLOW - 46;
                print_s_book_spell_type(ctx, out, item.location, &tr("Skill"), spell_color);
                ctx.control.info_string = fmt1(&tr("{:s} Skill"), &name);
            }
            SpellType::Spell => {
                if !in_town {
                    spell_color = PAL16_BLUE + 5;
                }
                print_s_book_spell_type(ctx, out, item.location, &tr("Spell"), spell_color);
                ctx.control.info_string = fmt1(&tr("{:s} Spell"), &name);
                if spell_id == SpellID::HolyBolt {
                    add_panel_string(ctx, &tr("Damages undead only"));
                }
                if spell_level == 0 {
                    add_panel_string(ctx, &tr("Spell Level 0 - Unusable"));
                } else {
                    add_panel_string(ctx, &fmt1(&tr("Spell Level {:d}"), &spell_level.to_string()));
                }
            }
            SpellType::Scroll => {
                if !in_town {
                    spell_color = PAL16_RED - 59;
                }
                print_s_book_spell_type(ctx, out, item.location, &tr("Scroll"), spell_color);
                ctx.control.info_string = fmt1(&tr("Scroll of {:s}"), &name);
                let scroll_count = crate::control::count_scrolls_of(ctx, me, spell_id);
                add_panel_string(ctx, &fmt1(&ngettext("{:d} Scroll", "{:d} Scrolls", scroll_count), &scroll_count.to_string()));
            }
            SpellType::Charges => {
                if !in_town {
                    spell_color = PAL16_ORANGE + 5;
                }
                print_s_book_spell_type(ctx, out, item.location, &tr("Staff"), spell_color);
                ctx.control.info_string = fmt1(&tr("Staff of {:s}"), &name);
                let charges = ctx.players.Players[me].InvBody[INVLOC_HAND_LEFT as usize]._iCharges;
                add_panel_string(ctx, &fmt1(&ngettext("{:d} Charge", "{:d} Charges", charges), &charges.to_string()));
            }
            SpellType::Invalid => {}
        }
        if let Some(full_hotkey_name) = get_hotkey_name(ctx, spell_id, item.type_, false) {
            add_panel_string(ctx, &fmt1(&tr("Spell Hotkey {:s}"), &full_hotkey_name));
        }
    }
}

/// Original: `devilution::GetSpellListItems` (panels/spell_list.cpp).
// @port panels/spell_list.cpp|devilution::GetSpellListItems() sha=062001b9ac1d
pub fn get_spell_list_items(ctx: &Ctx) -> Vec<SpellListItem> {
    let mut spell_list_items = Vec::new();
    let mp = get_main_panel(ctx);
    let (mx, my) = ctx.diablo.mouse_position;
    let row_start = mp.x + 12 + SPLICONLENGTH * SPLROWICONLS;
    let row_end = mp.x + 12 - SPLICONLENGTH;
    let mut x = row_start;
    let mut y = mp.y - 17;
    let me = my_player(ctx);
    for i in SPELL_TYPES {
        let Some(mask) = spell_mask(ctx, me, i) else {
            continue;
        };
        let mut j = SpellID::Firebolt as i8;
        let mut spl: u64 = 1;
        while (j as i32) < MAX_SPELLS {
            if (mask & spl) != 0 {
                let lx = x;
                let ly = y - SPLICONLENGTH;
                let is_selected = mx >= lx && mx < lx + SPLICONLENGTH && my >= ly && my < ly + SPLICONLENGTH;
                spell_list_items.push(SpellListItem { location: Point::new(x, y), type_: i, id: SpellID::from_repr(j).expect("SpellID"), is_selected });
                x -= SPLICONLENGTH;
                if x == row_end {
                    x = row_start;
                    y -= SPLICONLENGTH;
                }
            }
            spl <<= 1;
            j += 1;
        }
        if mask != 0 && x != row_start {
            x -= SPLICONLENGTH;
        }
        if x == row_end {
            x = row_start;
            y -= SPLICONLENGTH;
        }
    }
    spell_list_items
}

/// Original: `devilution::SetSpell` (panels/spell_list.cpp).
// @port panels/spell_list.cpp|devilution::SetSpell() sha=357cd7f89120
pub fn set_spell(ctx: &mut Ctx) {
    ctx.control.spselflag = false;
    let Some((p_spell, p_spl_type)) = get_spell_list_selection(ctx) else {
        return;
    };
    let me = my_player(ctx);
    ctx.players.Players[me]._pRSpell = p_spell;
    ctx.players.Players[me]._pRSplType = p_spl_type;
    crate::engine::backbuffer_state::redraw_everything(ctx);
}

/// Original: `devilution::SetSpeedSpell` (panels/spell_list.cpp).
// @port panels/spell_list.cpp|devilution::SetSpeedSpell(size_t slot) sha=975dbcaa47d0
pub fn set_speed_spell(ctx: &mut Ctx, slot: usize) {
    let Some((p_spell, p_spl_type)) = get_spell_list_selection(ctx) else {
        return;
    };
    let me = my_player(ctx);
    let p = &mut ctx.players.Players[me];
    for i in 0..NumHotkeys {
        if p._pSplHotKey[i] == p_spell && p._pSplTHotKey[i] == p_spl_type {
            p._pSplHotKey[i] = SpellID::Invalid;
        }
    }
    p._pSplHotKey[slot] = p_spell;
    p._pSplTHotKey[slot] = p_spl_type;
}

/// Original: `devilution::ToggleSpell` (panels/spell_list.cpp).
// @port panels/spell_list.cpp|devilution::ToggleSpell(size_t slot) sha=b7ad25e9ab22
pub fn toggle_spell(ctx: &mut Ctx, slot: usize) {
    let me = my_player(ctx);
    let spell_id = ctx.players.Players[me]._pSplHotKey[slot];
    if !is_valid_spell(ctx, spell_id) {
        return;
    }
    let t = ctx.players.Players[me]._pSplTHotKey[slot];
    let Some(spells) = spell_mask(ctx, me, t) else {
        return;
    };
    if (spells & get_spell_bitmask(spell_id)) != 0 {
        ctx.players.Players[me]._pRSpell = spell_id;
        ctx.players.Players[me]._pRSplType = t;
        crate::engine::backbuffer_state::redraw_everything(ctx);
    }
}

/// Original: `devilution::DoSpeedBook` (panels/spell_list.cpp).
// @port panels/spell_list.cpp|devilution::DoSpeedBook() sha=d0dcd5505c40
pub fn do_speed_book(ctx: &mut Ctx) {
    ctx.control.spselflag = true;
    let mp = get_main_panel(ctx);
    let row_start = mp.x + 12 + SPLICONLENGTH * SPLROWICONLS;
    let row_end = mp.x + 12 - SPLICONLENGTH;
    let mut xo = mp.x + 12 + SPLICONLENGTH * 10;
    let mut yo = mp.y - 17;
    let mut x = xo + SPLICONLENGTH / 2;
    let mut y = yo - SPLICONLENGTH / 2;
    let me = my_player(ctx);
    let (r_spell, r_type) = (ctx.players.Players[me]._pRSpell, ctx.players.Players[me]._pRSplType);
    if is_valid_spell(ctx, r_spell) {
        for i in SPELL_TYPES {
            let Some(spells) = spell_mask(ctx, me, i) else {
                continue;
            };
            let mut spell: u64 = 1;
            for j in 1..MAX_SPELLS {
                if (spell & spells) != 0 {
                    if j == r_spell as i8 as i32 && i == r_type {
                        x = xo + SPLICONLENGTH / 2;
                        y = yo - SPLICONLENGTH / 2;
                    }
                    xo -= SPLICONLENGTH;
                    if xo == row_end {
                        xo = row_start;
                        yo -= SPLICONLENGTH;
                    }
                }
                spell <<= 1;
            }
            if spells != 0 && xo != row_start {
                xo -= SPLICONLENGTH;
            }
            if xo == row_end {
                xo = row_start;
                yo -= SPLICONLENGTH;
            }
        }
    }
    crate::controls::set_cursor_pos(ctx, (x, y));
}
