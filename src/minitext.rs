//! `Source/minitext.cpp`: scrolling dialog text (quest and towner speeches).

use crate::ctx::Ctx;
use crate::effects_data::*;
use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::render::text_render::{draw_string, word_wrap_string, GameFontTables, UiFlags};
use crate::engine::surface::{Rect, Surface};
use crate::enums::*;
use crate::tables::playerdat::herosounds;
use crate::tables::textdat::Speeches;
use crate::utils::language::tr;

/// Pixels for a line of text and the empty space under it.
const LINE_HEIGHT: i32 = 38;

/// Globals of minitext.cpp.
#[derive(Default)]
pub struct MinitextState {
    /// `qtextflag`
    pub qtextflag: bool,
    /// `qtextSpd`: vertical speed of the scrolling text in ms/px
    qtext_spd: i32,
    /// `ScrollStart`
    scroll_start: u32,
    /// `pTextBoxCels`
    p_text_box_cels: Option<ClxSpriteList>,
    /// `TextLines`
    text_lines: Vec<String>,
}

/// `qtextflag`
pub fn qtextflag(ctx: &Ctx) -> bool {
    ctx.minitext.qtextflag
}

/// `qtextflag = v`
pub fn set_qtextflag(ctx: &mut Ctx, v: bool) {
    ctx.minitext.qtextflag = v;
}

/// Original: `LoadText` (minitext.cpp).
// @port minitext.cpp|devilution::LoadText(string_view text) sha=6c79083c9f2e
fn load_text(ctx: &mut Ctx, text: &str) {
    ctx.minitext.text_lines.clear();
    let paragraphs = word_wrap_string(ctx, text, 543, GameFontTables::GameFont30, 1);
    ctx.minitext.text_lines = paragraphs.split('\n').map(str::to_string).collect();
}

/// Original: `CalculateTextSpeed` (minitext.cpp): ms/px so the text matches the audio.
// @port minitext.cpp|devilution::CalculateTextSpeed(int nSFX) sha=a2a2c985b7cb
fn calculate_text_speed(ctx: &mut Ctx, n_sfx: i32) -> u32 {
    let num_lines = ctx.minitext.text_lines.len() as i32;
    let sfx_frames = crate::effects::get_sfx_length(ctx, n_sfx) as u32;
    assert!(sfx_frames != 0);
    let mut text_height = (LINE_HEIGHT * num_lines) as u32;
    text_height += (LINE_HEIGHT * 5) as u32; // adjust so when speaker is done two line are left
    assert!(text_height != 0);
    sfx_frames / text_height
}

/// Original: `CalculateTextPosition` (minitext.cpp).
// @port minitext.cpp|devilution::CalculateTextPosition() sha=d024c39ef7ee
fn calculate_text_position(ctx: &mut Ctx) -> i32 {
    let curr_time = ctx.platform.ticks();
    let y = (curr_time.wrapping_sub(ctx.minitext.scroll_start) / ctx.minitext.qtext_spd as u32) as i32 - 260;
    let text_height = LINE_HEIGHT * ctx.minitext.text_lines.len() as i32;
    if y >= text_height {
        ctx.minitext.qtextflag = false;
    }
    y
}

/// Original: `DrawQTextContent` (minitext.cpp).
// @port minitext.cpp|devilution::DrawQTextContent(const Surface &out) sha=84f35e76d7b3
fn draw_q_text_content(ctx: &mut Ctx, out: &Surface) {
    let y = calculate_text_position(ctx);
    let sx = crate::utils::display::get_ui_rectangle(ctx).x + 48;
    let sy = -(y % LINE_HEIGHT);
    let skip_lines = (y / LINE_HEIGHT) as u32;
    for i in 0..8 {
        let line_number = skip_lines.wrapping_add(i as u32) as usize;
        if line_number >= ctx.minitext.text_lines.len() {
            continue;
        }
        let line = ctx.minitext.text_lines[line_number].clone();
        if line.is_empty() {
            continue;
        }
        draw_string(ctx, out, &line, Rect::new(sx, sy + i * LINE_HEIGHT, 543, LINE_HEIGHT), UiFlags::FONT_SIZE_30 | UiFlags::COLOR_GOLD, 1, -1);
    }
}

/// Original: `devilution::FreeQuestText` (minitext.cpp).
// @port minitext.cpp|devilution::FreeQuestText() sha=4d1371d26244
pub fn free_quest_text(ctx: &mut Ctx) {
    ctx.minitext.p_text_box_cels = None;
}

/// Original: `devilution::InitQuestText` (minitext.cpp).
// @port minitext.cpp|devilution::InitQuestText() sha=495ac0067b5b
pub fn init_quest_text(ctx: &mut Ctx) {
    ctx.minitext.p_text_box_cels = Some(crate::engine::load_sprites::load_cel(ctx, "data\\textbox", 591));
}

/// Original: `devilution::InitQTextMsg` (minitext.cpp).
// @port minitext.cpp|devilution::InitQTextMsg(_speech_id m) sha=1624fb3f1f95
pub fn init_q_text_msg(ctx: &mut Ctx, m: _speech_id) {
    let speech = Speeches[m as usize];
    let mut sfxnr = speech.sfxnr;
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let class_sounds = &herosounds[ctx.players.Players[me]._pClass as usize];
    sfxnr = match sfxnr {
        PS_WARR1 => class_sounds[HeroSpeech::ChamberOfBoneLore as usize],
        PS_WARR10 => class_sounds[HeroSpeech::ValorLore as usize],
        PS_WARR11 => class_sounds[HeroSpeech::HallsOfTheBlindLore as usize],
        PS_WARR12 => class_sounds[HeroSpeech::WarlordOfBloodLore as usize],
        PS_WARR54 => class_sounds[HeroSpeech::InSpirituSanctum as usize],
        PS_WARR55 => class_sounds[HeroSpeech::PraedictumOtium as usize],
        PS_WARR56 => class_sounds[HeroSpeech::EfficioObitusUtInimicus as usize],
        _ => sfxnr,
    };
    if speech.scrlltxt {
        ctx.quests.QuestLogIsOpen = false;
        load_text(ctx, &tr(speech.txtstr));
        ctx.minitext.qtextflag = true;
        ctx.minitext.qtext_spd = calculate_text_speed(ctx, sfxnr as i32) as i32;
        ctx.minitext.scroll_start = ctx.platform.ticks();
    }
    crate::effects::play_sfx(ctx, sfxnr);
}

/// Original: `devilution::DrawQTextBack` (minitext.cpp).
// @port minitext.cpp|devilution::DrawQTextBack(const Surface &out) sha=e27f21818cb6
pub fn draw_q_text_back(ctx: &mut Ctx, out: &Surface) {
    let ui = crate::utils::display::get_ui_rectangle(ctx);
    let cel = ctx.minitext.p_text_box_cels.as_ref().expect("pTextBoxCels").get(0);
    crate::engine::render::clx_render::clx_draw(out, (ui.x + 24, ui.y + 327), &cel);
    crate::engine::draw_half_transparent_rect_to(ctx, out, ui.x + 27, ui.y + 28, 585, 297);
}

/// Original: `devilution::DrawQText` (minitext.cpp).
// @port minitext.cpp|devilution::DrawQText(const Surface &out) sha=f8e4a4e7baac
pub fn draw_q_text(ctx: &mut Ctx, out: &Surface) {
    draw_q_text_back(ctx, out);
    let ui = crate::utils::display::get_ui_rectangle(ctx);
    draw_q_text_content(ctx, &out.subregion_y(ui.y + 49, 260));
}
