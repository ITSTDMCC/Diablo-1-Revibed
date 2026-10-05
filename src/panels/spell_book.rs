//! `Source/panels/spell_book.cpp`: the spell book panel.

use crate::control::{get_panel_position, SIDE_PANEL_SIZE};
use crate::ctx::Ctx;
use crate::engine::geometry::{Displacement, Point, Rectangle, Size};
use crate::engine::load_sprites::load_cel;
use crate::engine::render::clx_render::clx_draw;
use crate::engine::render::text_render::{draw_string, UiFlags};
use crate::engine::surface::{Rect, Surface};
use crate::enums::*;
use crate::items::get_spell_data;
use crate::panels::spell_icons::*;
use crate::spells::{get_spell_bitmask, is_valid_spell};
use crate::utils::language::{ngettext, pgettext, tr};

const SPELL_BOOK_PAGES: usize = 6;
const SPELL_BOOK_PAGE_ENTRIES: usize = 7;

/// `SpellPages`: maps from spellbook page number and position to SpellID.
const SPELL_PAGES: [[SpellID; SPELL_BOOK_PAGE_ENTRIES]; SPELL_BOOK_PAGES] = [
    [SpellID::Null, SpellID::Firebolt, SpellID::ChargedBolt, SpellID::HolyBolt, SpellID::Healing, SpellID::HealOther, SpellID::Inferno],
    [SpellID::Resurrect, SpellID::FireWall, SpellID::Telekinesis, SpellID::Lightning, SpellID::TownPortal, SpellID::Flash, SpellID::StoneCurse],
    [SpellID::Phasing, SpellID::ManaShield, SpellID::Elemental, SpellID::Fireball, SpellID::FlameWave, SpellID::ChainLightning, SpellID::Guardian],
    [SpellID::Nova, SpellID::Golem, SpellID::Teleport, SpellID::Apocalypse, SpellID::BoneSpirit, SpellID::BloodStar, SpellID::Etherealize],
    [SpellID::LightningWall, SpellID::Immolation, SpellID::Warp, SpellID::Reflect, SpellID::Berserk, SpellID::RingOfFire, SpellID::Search],
    [SpellID::Invalid, SpellID::Invalid, SpellID::Invalid, SpellID::Invalid, SpellID::Invalid, SpellID::Invalid, SpellID::Invalid],
];

fn inspect(ctx: &Ctx) -> usize {
    ctx.players.InspectPlayer.expect("InspectPlayer")
}

/// Original: `GetSpellFromSpellPage` (panels/spell_book.cpp).
// @port panels/spell_book.cpp|devilution::GetSpellFromSpellPage(size_t page, size_t entry) sha=10c569a5e8a1
fn get_spell_from_spell_page(ctx: &Ctx, page: usize, entry: usize) -> SpellID {
    assert!(page <= SPELL_BOOK_PAGES && entry <= SPELL_BOOK_PAGE_ENTRIES);
    if page == 0 && entry == 0 {
        return match ctx.players.Players[inspect(ctx)]._pClass {
            HeroClass::Warrior => SpellID::ItemRepair,
            HeroClass::Rogue => SpellID::TrapDisarm,
            HeroClass::Sorcerer => SpellID::StaffRecharge,
            HeroClass::Monk => SpellID::Search,
            HeroClass::Bard => SpellID::Identify,
            HeroClass::Barbarian => SpellID::Rage,
        };
    }
    SPELL_PAGES[page][entry]
}

/// `SpellBookDescription`
const SPELL_BOOK_DESCRIPTION: Size = Size::new(250, 43);
const SPELL_BOOK_DESCRIPTION_PADDING_HORIZONTAL: i32 = 2;

/// Original: `PrintSBookStr` (panels/spell_book.cpp).
// @port panels/spell_book.cpp|devilution::PrintSBookStr(const Surface &out, Point position, string_view text, UiFlags flags = UiFlags::None) sha=fcf7841cdac3
fn print_s_book_str(ctx: &mut Ctx, out: &Surface, position: Point, text: &str, flags: UiFlags) {
    let p = get_panel_position(ctx, UiPanels::Spell, position + Displacement::new(SPLICONLENGTH, 0));
    let r = Rectangle::new(p, SPELL_BOOK_DESCRIPTION).inset(Displacement::new(SPELL_BOOK_DESCRIPTION_PADDING_HORIZONTAL, 0));
    draw_string(ctx, out, text, Rect::new(r.position.x, r.position.y, r.size.width, r.size.height), UiFlags::COLOR_WHITE | flags, 1, -1);
}

/// Original: `GetSBookTrans` (panels/spell_book.cpp).
// @port panels/spell_book.cpp|devilution::GetSBookTrans(SpellID ii, bool townok) sha=880c7b259a0d
fn get_s_book_trans(ctx: &Ctx, ii: SpellID, townok: bool) -> SpellType {
    let pi = inspect(ctx);
    let player = &ctx.players.Players[pi];
    if player._pClass == HeroClass::Monk && ii == SpellID::Search {
        return SpellType::Skill;
    }
    let mut st = SpellType::Spell;
    if (player._pISpells & get_spell_bitmask(ii)) != 0 {
        st = SpellType::Charges;
    }
    if (player._pAblSpells & get_spell_bitmask(ii)) != 0 {
        st = SpellType::Skill;
    }
    if st == SpellType::Spell {
        if crate::spells::check_spell(ctx, pi, ii, st, true) != SpellCheckResult::Success {
            st = SpellType::Invalid;
        }
        if player.get_spell_level(ii) == 0 {
            st = SpellType::Invalid;
        }
    }
    if townok && ctx.gendung.leveltype == crate::levels::gendung::DungeonType::Town && st != SpellType::Invalid && !get_spell_data(ii).is_allowed_in_town() {
        st = SpellType::Invalid;
    }
    st
}

fn fmt1(fmt: &str, arg: &str) -> String {
    match (fmt.find('{'), fmt.find('}')) {
        (Some(a), Some(b)) if b > a => format!("{}{}{}", &fmt[..a], arg, &fmt[b + 1..]),
        _ => fmt.to_string(),
    }
}

/// Original: `devilution::InitSpellBook` (panels/spell_book.cpp).
// @port panels/spell_book.cpp|devilution::InitSpellBook() sha=f31de84ef90d
pub fn init_spell_book(ctx: &mut Ctx) {
    ctx.panels.p_spell_bk_cel = Some(load_cel(ctx, "data\\spellbk", SIDE_PANEL_SIZE.0 as u16));
    let w = if ctx.init.gb_is_hellfire { 61 } else { 76 };
    ctx.panels.p_s_bk_btn_cel = Some(load_cel(ctx, "data\\spellbkb", w));
    load_small_spell_icons(ctx);
}

/// Original: `devilution::FreeSpellBook` (panels/spell_book.cpp).
// @port panels/spell_book.cpp|devilution::FreeSpellBook() sha=cbcb08afe7d8
pub fn free_spell_book(ctx: &mut Ctx) {
    free_small_spell_icons(ctx);
    ctx.panels.p_s_bk_btn_cel = None;
    ctx.panels.p_spell_bk_cel = None;
}

/// Original: `devilution::DrawSpellBook` (panels/spell_book.cpp).
// @port panels/spell_book.cpp|devilution::DrawSpellBook(const Surface &out) sha=01bd3cac2b22
pub fn draw_spell_book(ctx: &mut Ctx, out: &Surface) {
    let p = get_panel_position(ctx, UiPanels::Spell, Point::new(0, 351));
    clx_draw(out, (p.x, p.y), &ctx.panels.p_spell_bk_cel.as_ref().expect("pSpellBkCel").get(0));
    let sbooktab = ctx.control.sbooktab;
    let btn = ctx.panels.p_s_bk_btn_cel.clone().expect("pSBkBtnCel");
    if ctx.init.gb_is_hellfire && sbooktab < 5 {
        let p = get_panel_position(ctx, UiPanels::Spell, Point::new(61 * sbooktab + 7, 348));
        clx_draw(out, (p.x, p.y), &btn.get(sbooktab as usize));
    } else {
        // BUGFIX: rendering of page 3 and page 4 buttons are both off-by-one pixel (fixed).
        let mut sx = 76 * sbooktab + 7;
        if sbooktab == 2 || sbooktab == 3 {
            sx += 1;
        }
        let p = get_panel_position(ctx, UiPanels::Spell, Point::new(sx, 348));
        clx_draw(out, (p.x, p.y), &btn.get(sbooktab as usize));
    }
    let pi = inspect(ctx);
    let spl = {
        let player = &ctx.players.Players[pi];
        player._pMemSpells | player._pISpells | player._pAblSpells
    };
    const LINE_HEIGHT: i32 = 18;
    let mut yp = 12;
    const TEXT_PADDING_TOP: i32 = 7;
    for page_entry in 0..SPELL_BOOK_PAGE_ENTRIES {
        let sn = get_spell_from_spell_page(ctx, sbooktab as usize, page_entry);
        if is_valid_spell(ctx, sn) && (spl & get_spell_bitmask(sn)) != 0 {
            let st = get_s_book_trans(ctx, sn, true);
            set_spell_trans(ctx, st);
            let spell_cell_position = get_panel_position(ctx, UiPanels::Spell, Point::new(11, yp + SPELL_BOOK_DESCRIPTION.height));
            draw_small_spell_icon(ctx, out, spell_cell_position, sn);
            let (r_spell, r_type) = (ctx.players.Players[pi]._pRSpell, ctx.players.Players[pi]._pRSplType);
            if sn == r_spell && st == r_type && !crate::player::is_inspecting_player(ctx) {
                set_spell_trans(ctx, SpellType::Skill);
                draw_small_spell_icon_border(ctx, out, spell_cell_position);
            }

            let line0 = Point::new(0, yp + TEXT_PADDING_TOP);
            let line1 = Point::new(0, yp + TEXT_PADDING_TOP + LINE_HEIGHT);
            print_s_book_str(ctx, out, line0, &pgettext("spell", get_spell_data(sn).sNameText), UiFlags::NONE);
            match get_s_book_trans(ctx, sn, false) {
                SpellType::Skill => print_s_book_str(ctx, out, line1, &tr("Skill"), UiFlags::NONE),
                SpellType::Charges => {
                    let charges = ctx.players.Players[pi].InvBody[INVLOC_HAND_LEFT as usize]._iCharges;
                    print_s_book_str(ctx, out, line1, &fmt1(&ngettext("Staff ({:d} charge)", "Staff ({:d} charges)", charges), &charges.to_string()), UiFlags::NONE);
                }
                _ => {
                    let mana = crate::spells::get_mana_amount(ctx, &ctx.players.Players[pi], sn) >> 6;
                    let lvl = ctx.players.Players[pi].get_spell_level(sn);
                    print_s_book_str(ctx, out, line0, &fmt1(&pgettext("spellbook", "Level {:d}"), &lvl.to_string()), UiFlags::ALIGN_RIGHT);
                    if lvl == 0 {
                        print_s_book_str(ctx, out, line1, &tr("Unusable"), UiFlags::ALIGN_RIGHT);
                    } else {
                        if sn != SpellID::BoneSpirit {
                            let (min, max) = crate::missiles::get_damage_amt(ctx, sn);
                            if min != -1 {
                                let f = if sn == SpellID::Healing || sn == SpellID::HealOther { tr("Heals: {:d} - {:d}") } else { tr("Damage: {:d} - {:d}") };
                                print_s_book_str(ctx, out, line1, &fmt1(&fmt1(&f, &min.to_string()), &max.to_string()), UiFlags::ALIGN_RIGHT);
                            }
                        } else {
                            print_s_book_str(ctx, out, line1, &tr("Dmg: 1/3 target hp"), UiFlags::ALIGN_RIGHT);
                        }
                        print_s_book_str(ctx, out, line1, &fmt1(&pgettext("spellbook", "Mana: {:d}"), &mana.to_string()), UiFlags::NONE);
                    }
                }
            }
        }
        yp += SPELL_BOOK_DESCRIPTION.height;
    }
}

/// Original: `devilution::CheckSBook` (panels/spell_book.cpp).
// @port panels/spell_book.cpp|devilution::CheckSBook() sha=6bf6c8f0c440
pub fn check_s_book(ctx: &mut Ctx) {
    // Icons are drawn in a column near the left side of the panel and aligned with the spell book description entries
    // Spell icons/buttons are 37x38 pixels, laid out from 11,18 with a 5 pixel margin between each icon. This is close
    // enough to the height of the space given to spell descriptions that we can reuse that value and subtract the
    // padding from the end of the area.
    let mouse = Point::new(ctx.diablo.mouse_position.0, ctx.diablo.mouse_position.1);
    let icon_area = Rectangle::new(get_panel_position(ctx, UiPanels::Spell, Point::new(11, 18)), Size::new(37, SPELL_BOOK_DESCRIPTION.height * 7 - 5));
    if icon_area.contains(mouse) && !crate::player::is_inspecting_player(ctx) {
        let sn = get_spell_from_spell_page(ctx, ctx.control.sbooktab as usize, ((mouse.y - icon_area.position.y) / SPELL_BOOK_DESCRIPTION.height) as usize);
        let pi = inspect(ctx);
        let player = &ctx.players.Players[pi];
        let spl = player._pMemSpells | player._pISpells | player._pAblSpells;
        if is_valid_spell(ctx, sn) && (spl & get_spell_bitmask(sn)) != 0 {
            let mut st = SpellType::Spell;
            if (player._pISpells & get_spell_bitmask(sn)) != 0 {
                st = SpellType::Charges;
            }
            if (player._pAblSpells & get_spell_bitmask(sn)) != 0 {
                st = SpellType::Skill;
            }
            let player = &mut ctx.players.Players[pi];
            player._pRSpell = sn;
            player._pRSplType = st;
            crate::engine::backbuffer_state::redraw_everything(ctx);
        }
        return;
    }
    // The width of the panel excluding the border is 305 pixels. This does not cleanly divide by 4 meaning Diablo tabs
    // end up with an extra pixel somewhere around the buttons. Vanilla Diablo had the buttons left-aligned, devilutionX
    // instead justifies the buttons and puts the gap between buttons 2/3. See DrawSpellBook
    let tab_width = if ctx.init.gb_is_hellfire { 61 } else { 76 };
    // Tabs are drawn in a row near the bottom of the panel
    let tab_area = Rectangle::new(get_panel_position(ctx, UiPanels::Spell, Point::new(7, 320)), Size::new(305, 29));
    if tab_area.contains(mouse) {
        let mut hit_column = mouse.x - tab_area.position.x;
        // Clicking on the gutter currently activates tab 3. Could make it do nothing by checking for == here and return early.
        if !ctx.init.gb_is_hellfire && hit_column > tab_width * 2 {
            // Subtract 1 pixel to account for the gutter between buttons 2/3
            hit_column -= 1;
        }
        ctx.control.sbooktab = hit_column / tab_width;
    }
}
