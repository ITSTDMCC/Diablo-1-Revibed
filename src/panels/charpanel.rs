//! `Source/panels/charpanel.cpp`: the character panel.

use crate::control::get_panel_position;
use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSprite;
use crate::engine::geometry::Point;
use crate::engine::load_sprites::load_clx;
use crate::engine::render::clx_render::clx_draw;
use crate::engine::render::text_render::{draw_string, get_line_height, render_clx_sprite, word_wrap_string, GameFontTables, UiFlags};
use crate::engine::surface::{OwnedSurface, Rect, Surface};
use crate::enums::*;
use crate::player::{MaxCharacterLevel, MaxResistance, Player};
use crate::utils::format_int::format_integer;
use crate::utils::language::{is_small_font_tall, tr};
use crate::utils::surface_to_clx::surface_to_clx;

/// `StyledText`
struct StyledText {
    style: UiFlags,
    text: String,
    spacing: i32,
}

fn styled(style: UiFlags, text: String) -> StyledText {
    StyledText { style, text, spacing: 1 }
}

/// `PanelEntry`
struct PanelEntry {
    label: &'static str,
    position: Point,
    length: i32,
    /// max label's length - used for line wrapping
    label_length: i32,
    /// function responsible for displaying stat
    stat_display_func: Option<fn(&mut Ctx) -> StyledText>,
}

fn ip(ctx: &Ctx) -> &Player {
    &ctx.players.Players[ctx.players.InspectPlayer.expect("InspectPlayer")]
}

/// Original: `GetBaseStatColor` (panels/charpanel.cpp).
// @port panels/charpanel.cpp|devilution::GetBaseStatColor(CharacterAttribute attr) sha=fdb3ebd04c05
fn get_base_stat_color(ctx: &Ctx, attr: CharacterAttribute) -> UiFlags {
    let p = ip(ctx);
    if p.get_base_attribute_value(attr) == p.get_maximum_attribute_value(attr) {
        UiFlags::COLOR_WHITEGOLD
    } else {
        UiFlags::COLOR_WHITE
    }
}

/// Original: `GetCurrentStatColor` (panels/charpanel.cpp).
// @port panels/charpanel.cpp|devilution::GetCurrentStatColor(CharacterAttribute attr) sha=5a3b2818fb8a
fn get_current_stat_color(ctx: &Ctx, attr: CharacterAttribute) -> UiFlags {
    let p = ip(ctx);
    let current = p.get_current_attribute_value(attr);
    let base = p.get_base_attribute_value(attr);
    let mut style = UiFlags::COLOR_WHITE;
    if current > base {
        style = UiFlags::COLOR_BLUE;
    }
    if current < base {
        style = UiFlags::COLOR_RED;
    }
    style
}

/// Original: `GetValueColor` (panels/charpanel.cpp).
// @port panels/charpanel.cpp|devilution::GetValueColor(int value, bool flip = false) sha=780a9f5fb612
fn get_value_color(value: i32) -> UiFlags {
    let mut style = UiFlags::COLOR_WHITE;
    if value > 0 {
        style = UiFlags::COLOR_BLUE;
    }
    if value < 0 {
        style = UiFlags::COLOR_RED;
    }
    style
}

/// Original: `GetMaxManaColor` (panels/charpanel.cpp).
// @port panels/charpanel.cpp|devilution::GetMaxManaColor() sha=5a1192f0ad72
fn get_max_mana_color(ctx: &Ctx) -> UiFlags {
    if ip(ctx)._pMaxMana > ip(ctx)._pMaxManaBase {
        UiFlags::COLOR_BLUE
    } else {
        UiFlags::COLOR_WHITE
    }
}

/// Original: `GetMaxHealthColor` (panels/charpanel.cpp).
// @port panels/charpanel.cpp|devilution::GetMaxHealthColor() sha=31fe93eb45ca
fn get_max_health_color(ctx: &Ctx) -> UiFlags {
    if ip(ctx)._pMaxHP > ip(ctx)._pMaxHPBase {
        UiFlags::COLOR_BLUE
    } else {
        UiFlags::COLOR_WHITE
    }
}

/// Original: `GetDamage` (panels/charpanel.cpp).
// @port panels/charpanel.cpp|devilution::GetDamage() sha=d44b9b40fac1
fn get_damage(ctx: &Ctx) -> (i32, i32) {
    let p = ip(ctx);
    let mut damage_mod = p._pIBonusDamMod;
    if p.InvBody[INVLOC_HAND_LEFT as usize]._itype == ItemType::Bow && p._pClass != HeroClass::Rogue {
        damage_mod += p._pDamageMod / 2;
    } else {
        damage_mod += p._pDamageMod;
    }
    let mindam = p._pIMinDam + p._pIBonusDam * p._pIMinDam / 100 + damage_mod;
    let maxdam = p._pIMaxDam + p._pIBonusDam * p._pIMaxDam / 100 + damage_mod;
    (mindam, maxdam)
}

/// Original: `GetResistInfo` (panels/charpanel.cpp).
// @port panels/charpanel.cpp|devilution::GetResistInfo(int8_t resist) sha=f6b61d9998a8
fn get_resist_info(resist: i8) -> StyledText {
    let mut style = UiFlags::COLOR_BLUE;
    if resist == 0 {
        style = UiFlags::COLOR_WHITE;
    } else if resist < 0 {
        style = UiFlags::COLOR_RED;
    } else if resist as i32 >= MaxResistance {
        style = UiFlags::COLOR_WHITEGOLD;
    }
    styled(style, format!("{}%", resist))
}

const LEFT_COLUMN_LABEL_X: i32 = 88;
const TOP_RIGHT_LABEL_X: i32 = 211;
const RIGHT_COLUMN_LABEL_X: i32 = 253;

const LEFT_COLUMN_LABEL_WIDTH: i32 = 76;
const RIGHT_COLUMN_LABEL_WIDTH: i32 = 68;

// Indices in `panelEntries`.
const ATTRIBUTE_HEADER_ENTRY_INDICES: [usize; 2] = [5, 6];
const GOLD_HEADER_ENTRY_INDEX: usize = 16;

const fn entry(label: &'static str, x: i32, y: i32, length: i32, label_length: i32, f: Option<fn(&mut Ctx) -> StyledText>) -> PanelEntry {
    PanelEntry { label, position: Point::new(x, y), length, label_length, stat_display_func: f }
}

/// `panelEntries` (the dynamically placed header rows have y = 0 here; see `entry_position`).
const PANEL_ENTRIES: [PanelEntry; 28] = [
    entry("", 9, 14, 150, 0, Some(|ctx| styled(UiFlags::COLOR_WHITE, ip(ctx)._pName.as_str().to_string()))),
    entry("", 161, 14, 149, 0, Some(|ctx| styled(UiFlags::COLOR_WHITE, tr(crate::tables::playerdat::PlayersData[ip(ctx)._pClass as usize].className)))),
    entry("Level", 57, 52, 57, 45, Some(|ctx| styled(UiFlags::COLOR_WHITE, ip(ctx)._pLevel.to_string()))),
    entry(
        "Experience",
        TOP_RIGHT_LABEL_X,
        52,
        99,
        91,
        Some(|ctx| {
            let p = ip(ctx);
            let spacing = if p._pExperience >= 1000000000 { 0 } else { 1 };
            StyledText { style: UiFlags::COLOR_WHITE, text: format_integer(p._pExperience as i32), spacing }
        }),
    ),
    entry(
        "Next level",
        TOP_RIGHT_LABEL_X,
        80,
        99,
        198,
        Some(|ctx| {
            let p = ip(ctx);
            if p._pLevel as i32 == MaxCharacterLevel {
                return styled(UiFlags::COLOR_WHITEGOLD, tr("None"));
            }
            let spacing = if p._pNextExper >= 1000000000 { 0 } else { 1 };
            StyledText { style: UiFlags::COLOR_WHITE, text: format_integer(p._pNextExper as i32), spacing }
        }),
    ),
    entry("Base", LEFT_COLUMN_LABEL_X, 0, 0, 44, None),
    entry("Now", 135, 0, 0, 44, None),
    entry("Strength", LEFT_COLUMN_LABEL_X, 135, 45, LEFT_COLUMN_LABEL_WIDTH, Some(|ctx| styled(get_base_stat_color(ctx, CharacterAttribute::Strength), ip(ctx)._pBaseStr.to_string()))),
    entry("", 135, 135, 45, 0, Some(|ctx| styled(get_current_stat_color(ctx, CharacterAttribute::Strength), ip(ctx)._pStrength.to_string()))),
    entry("Magic", LEFT_COLUMN_LABEL_X, 163, 45, LEFT_COLUMN_LABEL_WIDTH, Some(|ctx| styled(get_base_stat_color(ctx, CharacterAttribute::Magic), ip(ctx)._pBaseMag.to_string()))),
    entry("", 135, 163, 45, 0, Some(|ctx| styled(get_current_stat_color(ctx, CharacterAttribute::Magic), ip(ctx)._pMagic.to_string()))),
    entry("Dexterity", LEFT_COLUMN_LABEL_X, 191, 45, LEFT_COLUMN_LABEL_WIDTH, Some(|ctx| styled(get_base_stat_color(ctx, CharacterAttribute::Dexterity), ip(ctx)._pBaseDex.to_string()))),
    entry("", 135, 191, 45, 0, Some(|ctx| styled(get_current_stat_color(ctx, CharacterAttribute::Dexterity), ip(ctx)._pDexterity.to_string()))),
    entry("Vitality", LEFT_COLUMN_LABEL_X, 219, 45, LEFT_COLUMN_LABEL_WIDTH, Some(|ctx| styled(get_base_stat_color(ctx, CharacterAttribute::Vitality), ip(ctx)._pBaseVit.to_string()))),
    entry("", 135, 219, 45, 0, Some(|ctx| styled(get_current_stat_color(ctx, CharacterAttribute::Vitality), ip(ctx)._pVitality.to_string()))),
    entry(
        "Points to distribute",
        LEFT_COLUMN_LABEL_X,
        248,
        45,
        LEFT_COLUMN_LABEL_WIDTH,
        Some(|ctx| {
            let i = ctx.players.InspectPlayer.expect("InspectPlayer");
            let diff = crate::player::calc_stat_diff(&ctx.players.Players[i]);
            let p = &mut ctx.players.Players[i];
            p._pStatPts = diff.min(p._pStatPts);
            styled(UiFlags::COLOR_RED, if p._pStatPts > 0 { p._pStatPts.to_string() } else { String::new() })
        }),
    ),
    entry("Gold", TOP_RIGHT_LABEL_X, 0, 0, 98, None),
    entry("", TOP_RIGHT_LABEL_X, 127, 99, 0, Some(|ctx| styled(UiFlags::COLOR_WHITE, format_integer(ip(ctx)._pGold)))),
    entry(
        "Armor class",
        RIGHT_COLUMN_LABEL_X,
        163,
        57,
        RIGHT_COLUMN_LABEL_WIDTH,
        Some(|ctx| {
            let p = ip(ctx);
            styled(get_value_color(p._pIBonusAC), (p.get_armor() + p._pLevel as i32 * 2).to_string())
        }),
    ),
    entry(
        "To hit",
        RIGHT_COLUMN_LABEL_X,
        191,
        57,
        RIGHT_COLUMN_LABEL_WIDTH,
        Some(|ctx| {
            let p = ip(ctx);
            let v = if p.InvBody[INVLOC_HAND_LEFT as usize]._itype == ItemType::Bow { p.get_ranged_to_hit() } else { p.get_melee_to_hit() };
            styled(get_value_color(p._pIBonusToHit), format!("{}%", v))
        }),
    ),
    entry(
        "Damage",
        RIGHT_COLUMN_LABEL_X,
        219,
        57,
        RIGHT_COLUMN_LABEL_WIDTH,
        Some(|ctx| {
            let dmg = get_damage(ctx);
            let spacing = if dmg.0 >= 100 { -1 } else { 1 };
            StyledText { style: get_value_color(ip(ctx)._pIBonusDam), text: format!("{}-{}", dmg.0, dmg.1), spacing }
        }),
    ),
    entry("Life", LEFT_COLUMN_LABEL_X, 284, 45, LEFT_COLUMN_LABEL_WIDTH, Some(|ctx| styled(get_max_health_color(ctx), (ip(ctx)._pMaxHP >> 6).to_string()))),
    entry(
        "",
        135,
        284,
        45,
        0,
        Some(|ctx| {
            let p = ip(ctx);
            let style = if p._pHitPoints != p._pMaxHP { UiFlags::COLOR_RED } else { get_max_health_color(ctx) };
            styled(style, (p._pHitPoints >> 6).to_string())
        }),
    ),
    entry("Mana", LEFT_COLUMN_LABEL_X, 312, 45, LEFT_COLUMN_LABEL_WIDTH, Some(|ctx| styled(get_max_mana_color(ctx), (ip(ctx)._pMaxMana >> 6).to_string()))),
    entry(
        "",
        135,
        312,
        45,
        0,
        Some(|ctx| {
            let p = ip(ctx);
            let style = if p._pMana != p._pMaxMana { UiFlags::COLOR_RED } else { get_max_mana_color(ctx) };
            styled(style, (p._pMana >> 6).to_string())
        }),
    ),
    entry("Resist magic", RIGHT_COLUMN_LABEL_X, 256, 57, RIGHT_COLUMN_LABEL_WIDTH, Some(|ctx| get_resist_info(ip(ctx)._pMagResist))),
    entry("Resist fire", RIGHT_COLUMN_LABEL_X, 284, 57, RIGHT_COLUMN_LABEL_WIDTH, Some(|ctx| get_resist_info(ip(ctx)._pFireResist))),
    entry("Resist lightning", RIGHT_COLUMN_LABEL_X, 313, 57, RIGHT_COLUMN_LABEL_WIDTH, Some(|ctx| get_resist_info(ip(ctx)._pLghtResist))),
];

/// `panelEntries[i].position`, including the y set by `LoadCharPanel` for the header rows.
fn entry_position(ctx: &Ctx, i: usize) -> Point {
    let mut p = PANEL_ENTRIES[i].position;
    if ATTRIBUTE_HEADER_ENTRY_INDICES.contains(&i) {
        p.y = ctx.panels.attribute_headers_y;
    } else if i == GOLD_HEADER_ENTRY_INDEX {
        p.y = ctx.panels.gold_header_y;
    }
    p
}

const PANEL_FIELD_HEIGHT: i32 = 24;
const PANEL_FIELD_PADDING_TOP: i32 = 3;
const PANEL_FIELD_PADDING_BOTTOM: i32 = 3;
const PANEL_FIELD_INNER_HEIGHT: i32 = PANEL_FIELD_HEIGHT - PANEL_FIELD_PADDING_TOP - PANEL_FIELD_PADDING_BOTTOM;

/// Original: `DrawPanelField` (panels/charpanel.cpp).
// @port panels/charpanel.cpp|devilution::DrawPanelField(const Surface &out, Point pos, int len, ClxSprite left, ClxSprite middle, ClxSprite right) sha=2909d38e79ca
fn draw_panel_field(out: &Surface, mut pos: Point, mut len: i32, left: &ClxSprite, middle: &ClxSprite, right: &ClxSprite) {
    render_clx_sprite(out, left, (pos.x, pos.y));
    pos.x += left.width() as i32;
    len -= left.width() as i32 + right.width() as i32;
    render_clx_sprite(&out.subregion(pos.x, pos.y, len, middle.height() as i32), middle, (0, 0));
    pos.x += len;
    render_clx_sprite(out, right, (pos.x, pos.y));
}

/// Original: `DrawShadowString` (panels/charpanel.cpp).
// @port panels/charpanel.cpp|devilution::DrawShadowString(const Surface &out, const PanelEntry &entry) sha=e1b9aaca3410
fn draw_shadow_string(ctx: &mut Ctx, out: &Surface, entry: &PanelEntry, position: Point) {
    if entry.label.is_empty() {
        return;
    }
    const SPACING: i32 = 0;
    let text_str = tr(entry.label);
    let mut wrapped = String::new();
    let text = if entry.label_length > 0 {
        wrapped = word_wrap_string(ctx, &text_str, entry.label_length as u32, GameFontTables::GameFont12, SPACING);
        wrapped.clone()
    } else {
        text_str
    };
    let mut style = UiFlags::VERTICAL_CENTER;
    let mut label_position = position;
    if entry.length == 0 {
        style = style | UiFlags::ALIGN_CENTER;
    } else {
        style = style | UiFlags::ALIGN_RIGHT;
        label_position.x += -entry.label_length - if is_small_font_tall(ctx) { 2 } else { 3 };
    }
    // If the text is less tall then the field, we center it vertically relative to the field.
    // Otherwise, we draw from the top of the field.
    let text_height = (wrapped.matches('\n').count() as i32 + 1) * get_line_height(ctx, &wrapped, GameFontTables::GameFont12);
    let label_height = PANEL_FIELD_HEIGHT.max(text_height);
    draw_string(ctx, out, &text, Rect::new(label_position.x - 2, label_position.y + 2, entry.label_length, label_height), style | UiFlags::COLOR_BLACK, SPACING, -1);
    draw_string(ctx, out, &text, Rect::new(label_position.x, label_position.y, entry.label_length, label_height), style | UiFlags::COLOR_WHITE, SPACING, -1);
}

/// Original: `DrawStatButtons` (panels/charpanel.cpp).
// @port panels/charpanel.cpp|devilution::DrawStatButtons(const Surface &out) sha=d386ccb6d465
fn draw_stat_buttons(ctx: &mut Ctx, out: &Surface) {
    let p = ip(ctx);
    if p._pStatPts > 0 && !crate::player::is_inspecting_player(ctx) {
        let buttons = ctx.panels.p_chr_buttons.clone().expect("pChrButtons");
        let chrbtn = ctx.control.chrbtn;
        let rows: [(i32, i32, CharacterAttribute, usize); 4] = [
            (p._pBaseStr, 157, CharacterAttribute::Strength, 1),
            (p._pBaseMag, 185, CharacterAttribute::Magic, 3),
            (p._pBaseDex, 214, CharacterAttribute::Dexterity, 5),
            (p._pBaseVit, 242, CharacterAttribute::Vitality, 7),
        ];
        for (base, y, attr, frame) in rows {
            if base < p.get_maximum_attribute_value(attr) {
                let pos = get_panel_position(ctx, UiPanels::Character, Point::new(137, y));
                clx_draw(out, (pos.x, pos.y), &buttons.get(if chrbtn[attr as usize] { frame + 1 } else { frame }));
            }
        }
    }
}

/// Original: `devilution::LoadCharPanel` (panels/charpanel.cpp).
// @port panels/charpanel.cpp|devilution::LoadCharPanel() sha=159cab528866
pub fn load_char_panel(ctx: &mut Ctx) {
    let background = load_clx(ctx, "data\\charbg.clx");
    let first = background.get(0);
    let mut owned = OwnedSurface::new(first.width() as i32, first.height() as i32);
    let out = owned.view();
    render_clx_sprite(&out, &first, (0, 0));
    drop(background);
    {
        let box_left = load_clx(ctx, "data\\boxleftend.clx");
        let box_middle = load_clx(ctx, "data\\boxmiddle.clx");
        let box_right = load_clx(ctx, "data\\boxrightend.clx");

        let is_small_font_tall = is_small_font_tall(ctx);
        ctx.panels.attribute_headers_y = if is_small_font_tall { 112 } else { 114 };
        ctx.panels.gold_header_y = if is_small_font_tall { 105 } else { 106 };

        for (i, entry) in PANEL_ENTRIES.iter().enumerate() {
            let position = entry_position(ctx, i);
            if entry.stat_display_func.is_some() {
                draw_panel_field(&out, position, entry.length, &box_left.get(0), &box_middle.get(0), &box_right.get(0));
            }
            draw_shadow_string(ctx, &out, entry, position);
        }
    }
    ctx.panels.char_panel = Some(surface_to_clx(&out, 1, None));
    drop(owned);
}

/// Original: `devilution::FreeCharPanel` (panels/charpanel.cpp).
// @port panels/charpanel.cpp|devilution::FreeCharPanel() sha=580759b6f96b
pub fn free_char_panel(ctx: &mut Ctx) {
    ctx.panels.char_panel = None;
}

/// Original: `devilution::DrawChr` (panels/charpanel.cpp).
// @port panels/charpanel.cpp|devilution::DrawChr(const Surface &out) sha=692e7ed425bb
pub fn draw_chr(ctx: &mut Ctx, out: &Surface) {
    let pos = get_panel_position(ctx, UiPanels::Character, Point::new(0, 0));
    let panel = ctx.panels.char_panel.as_ref().expect("Panel").get(0);
    render_clx_sprite(out, &panel, (pos.x, pos.y));
    for (i, entry) in PANEL_ENTRIES.iter().enumerate() {
        if let Some(f) = entry.stat_display_func {
            let tmp = f(ctx);
            let p = entry_position(ctx, i);
            draw_string(
                ctx,
                out,
                &tmp.text,
                Rect::new(p.x + pos.x, p.y + pos.y + PANEL_FIELD_PADDING_TOP, entry.length, PANEL_FIELD_INNER_HEIGHT),
                UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | tmp.style,
                tmp.spacing,
                -1,
            );
        }
    }
    draw_stat_buttons(ctx, out);
}
