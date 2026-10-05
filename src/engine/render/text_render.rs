//! `Source/engine/render/text_render.cpp` and `DiabloUI/ui_flags.hpp`: bitmap fonts (one CLX
//! list of 256 glyphs per font size and Unicode row) and string drawing.

use std::collections::HashMap;

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::render::clx_render::{clx_draw, clx_draw_outline_skip_color_zero, clx_draw_trn};
use crate::engine::surface::{Rect, Surface};
use crate::platform::log;

/// `UiFlags` (bit flags)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct UiFlags(pub u32);

impl UiFlags {
    pub const NONE: UiFlags = UiFlags(0);
    pub const FONT_SIZE_12: UiFlags = UiFlags(1 << 0);
    pub const FONT_SIZE_24: UiFlags = UiFlags(1 << 1);
    pub const FONT_SIZE_30: UiFlags = UiFlags(1 << 2);
    pub const FONT_SIZE_42: UiFlags = UiFlags(1 << 3);
    pub const FONT_SIZE_46: UiFlags = UiFlags(1 << 4);
    pub const FONT_SIZE_DIALOG: UiFlags = UiFlags(1 << 5);
    pub const COLOR_UI_GOLD: UiFlags = UiFlags(1 << 6);
    pub const COLOR_UI_SILVER: UiFlags = UiFlags(1 << 7);
    pub const COLOR_UI_GOLD_DARK: UiFlags = UiFlags(1 << 8);
    pub const COLOR_UI_SILVER_DARK: UiFlags = UiFlags(1 << 9);
    pub const COLOR_DIALOG_WHITE: UiFlags = UiFlags(1 << 10);
    pub const COLOR_YELLOW: UiFlags = UiFlags(1 << 11);
    pub const COLOR_GOLD: UiFlags = UiFlags(1 << 12);
    pub const COLOR_BLACK: UiFlags = UiFlags(1 << 13);
    pub const COLOR_WHITE: UiFlags = UiFlags(1 << 14);
    pub const COLOR_WHITEGOLD: UiFlags = UiFlags(1 << 15);
    pub const COLOR_RED: UiFlags = UiFlags(1 << 16);
    pub const COLOR_BLUE: UiFlags = UiFlags(1 << 17);
    pub const COLOR_ORANGE: UiFlags = UiFlags(1 << 18);
    pub const COLOR_BUTTONFACE: UiFlags = UiFlags(1 << 19);
    pub const COLOR_BUTTONPUSHED: UiFlags = UiFlags(1 << 20);
    pub const ALIGN_CENTER: UiFlags = UiFlags(1 << 21);
    pub const ALIGN_RIGHT: UiFlags = UiFlags(1 << 22);
    pub const VERTICAL_CENTER: UiFlags = UiFlags(1 << 23);
    pub const KERNING_FIT_SPACING: UiFlags = UiFlags(1 << 24);
    pub const ELEMENT_DISABLED: UiFlags = UiFlags(1 << 25);
    pub const ELEMENT_HIDDEN: UiFlags = UiFlags(1 << 26);
    pub const PENTA_CURSOR: UiFlags = UiFlags(1 << 27);
    pub const TEXT_CURSOR: UiFlags = UiFlags(1 << 28);
    pub const OUTLINED: UiFlags = UiFlags(1 << 29);
    pub const NEEDS_NEXT_ELEMENT: UiFlags = UiFlags(1 << 30);

    /// `HasAnyOf`
    pub fn has(self, other: UiFlags) -> bool {
        self.0 & other.0 != 0
    }
}

impl std::ops::BitOr for UiFlags {
    type Output = UiFlags;
    fn bitor(self, rhs: UiFlags) -> UiFlags {
        UiFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for UiFlags {
    fn bitor_assign(&mut self, rhs: UiFlags) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for UiFlags {
    type Output = UiFlags;
    fn bitand(self, rhs: UiFlags) -> UiFlags {
        UiFlags(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for UiFlags {
    fn bitand_assign(&mut self, rhs: UiFlags) {
        self.0 &= rhs.0;
    }
}

impl std::ops::Not for UiFlags {
    type Output = UiFlags;
    fn not(self) -> UiFlags {
        UiFlags(!self.0)
    }
}

/// `GameFontTables`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum GameFontTables {
    GameFont12,
    GameFont24,
    GameFont30,
    GameFont42,
    GameFont46,
    FontSizeDialog,
}

/// `text_color`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TextColor {
    UiGold,
    UiSilver,
    UiGoldDark,
    UiSilverDark,
    DialogWhite,
    Yellow,
    Gold,
    Black,
    White,
    Whitegold,
    Red,
    Blue,
    Orange,
    Buttonface,
    Buttonpushed,
}

const ZWSP: char = '\u{200B}';
const FONT_SIZES: [i32; 6] = [12, 24, 30, 42, 46, 22];
const LINE_HEIGHTS: [i32; 6] = [12, 26, 38, 42, 50, 22];
const SMALL_FONT_TALL_LINE_HEIGHT: i32 = 16;
const BASE_LINE_OFFSET: [i32; 6] = [-3, -2, -3, -6, -7, 3];
const COLOR_TRANSLATIONS: [Option<&str>; 15] = [
    Some("fonts\\goldui.trn"),
    Some("fonts\\grayui.trn"),
    Some("fonts\\golduis.trn"),
    Some("fonts\\grayuis.trn"),
    None,
    Some("fonts\\yellow.trn"),
    None,
    Some("fonts\\black.trn"),
    Some("fonts\\white.trn"),
    Some("fonts\\whitegold.trn"),
    Some("fonts\\red.trn"),
    Some("fonts\\blue.trn"),
    Some("fonts\\orange.trn"),
    Some("fonts\\buttonface.trn"),
    Some("fonts\\buttonpushed.trn"),
];

#[derive(Default)]
pub struct TextRenderState {
    /// `pSPentSpn2Cels`
    pub p_s_pent_spn2_cels: Option<ClxSpriteList>,
    /// `Fonts`
    fonts: HashMap<u32, Option<ClxSpriteList>>,
    /// `ColorTranslationsData`
    color_translations_data: [Option<Box<[u8; 256]>>; 15],
}

/// `DrawStringFormatArg`
#[derive(Clone, Debug)]
pub struct DrawStringFormatArg {
    value: FormatValue,
    flags: UiFlags,
    formatted: Option<String>,
}

#[derive(Clone, Debug)]
enum FormatValue {
    Str(String),
    Int(i32),
}

impl DrawStringFormatArg {
    pub fn string(value: &str, flags: UiFlags) -> Self {
        DrawStringFormatArg { value: FormatValue::Str(value.to_string()), flags, formatted: None }
    }
    pub fn int(value: i32, flags: UiFlags) -> Self {
        DrawStringFormatArg { value: FormatValue::Int(value), flags, formatted: None }
    }
    fn is_string(&self) -> bool {
        matches!(self.value, FormatValue::Str(_))
    }
    /// `GetFormatted`
    pub fn get_formatted(&self) -> &str {
        match &self.value {
            FormatValue::Str(s) => s,
            FormatValue::Int(_) => self.formatted.as_deref().unwrap_or(""),
        }
    }
    fn has_formatted(&self) -> bool {
        self.is_string() || self.formatted.as_ref().is_some_and(|s| !s.is_empty())
    }
    pub fn get_flags(&self) -> UiFlags {
        self.flags
    }
}

/// `fmt::format(fmt, int)` for the integer specs the game uses: `{}`, `{:d}`, `{:+d}`,
/// `{:0Nd}`, `{:N}`.
fn format_int(spec: &str, v: i32) -> String {
    let inner = spec.trim_start_matches('{').trim_end_matches('}');
    let inner = inner.strip_prefix(':').unwrap_or("");
    let plus = inner.starts_with('+');
    let inner = inner.trim_start_matches('+').trim_end_matches('d');
    let zero = inner.starts_with('0');
    let width: usize = inner.trim_start_matches('0').parse().unwrap_or(0);
    let s = if plus && v >= 0 { format!("+{v}") } else { v.to_string() };
    if s.len() >= width {
        s
    } else if zero {
        let (sign, digits) = if s.starts_with('-') || s.starts_with('+') { s.split_at(1) } else { ("", s.as_str()) };
        format!("{sign}{}{digits}", "0".repeat(width - s.len()))
    } else {
        format!("{}{s}", " ".repeat(width - s.len()))
    }
}

/// `GetSizeFromFlags`
// @port engine/render/text_render.cpp|devilution::GetSizeFromFlags(UiFlags flags) sha=16ef9e35d281
fn get_size_from_flags(flags: UiFlags) -> GameFontTables {
    if flags.has(UiFlags::FONT_SIZE_24) {
        GameFontTables::GameFont24
    } else if flags.has(UiFlags::FONT_SIZE_30) {
        GameFontTables::GameFont30
    } else if flags.has(UiFlags::FONT_SIZE_42) {
        GameFontTables::GameFont42
    } else if flags.has(UiFlags::FONT_SIZE_46) {
        GameFontTables::GameFont46
    } else if flags.has(UiFlags::FONT_SIZE_DIALOG) {
        GameFontTables::FontSizeDialog
    } else {
        GameFontTables::GameFont12
    }
}

/// `GetColorFromFlags`
// @port engine/render/text_render.cpp|devilution::GetColorFromFlags(UiFlags flags) sha=06d7fca14410
fn get_color_from_flags(flags: UiFlags) -> TextColor {
    use TextColor::*;
    for (f, c) in [
        (UiFlags::COLOR_WHITE, White),
        (UiFlags::COLOR_BLUE, Blue),
        (UiFlags::COLOR_ORANGE, Orange),
        (UiFlags::COLOR_RED, Red),
        (UiFlags::COLOR_BLACK, Black),
        (UiFlags::COLOR_GOLD, Gold),
        (UiFlags::COLOR_UI_GOLD, UiGold),
        (UiFlags::COLOR_UI_SILVER, UiSilver),
        (UiFlags::COLOR_UI_GOLD_DARK, UiGoldDark),
        (UiFlags::COLOR_UI_SILVER_DARK, UiSilverDark),
        (UiFlags::COLOR_DIALOG_WHITE, DialogWhite),
        (UiFlags::COLOR_YELLOW, Yellow),
        (UiFlags::COLOR_BUTTONFACE, Buttonface),
        (UiFlags::COLOR_BUTTONPUSHED, Buttonpushed),
    ] {
        if flags.has(f) {
            return c;
        }
    }
    Whitegold
}

/// `GetUnicodeRow`
// @port engine/render/text_render.cpp|devilution::GetUnicodeRow(char32_t codePoint) sha=06b69873c065
fn get_unicode_row(c: char) -> u16 {
    (c as u32 >> 8) as u16
}

/// `IsCJK`
// @port engine/render/text_render.cpp|devilution::IsCJK(uint16_t row) sha=f4429a2e61db
fn is_cjk(row: u16) -> bool {
    (0x30..=0x9f).contains(&row)
}

/// `IsHangul`
// @port engine/render/text_render.cpp|devilution::IsHangul(uint16_t row) sha=4ec6cbb3f00e
fn is_hangul(row: u16) -> bool {
    (0xac..=0xd7).contains(&row)
}

/// `IsSmallFontTallRow`
// @port engine/render/text_render.cpp|devilution::IsSmallFontTallRow(uint16_t row) sha=d4b72f371a94
fn is_small_font_tall_row(row: u16) -> bool {
    is_cjk(row) || is_hangul(row)
}

/// `GetFontPath`
// @port engine/render/text_render.cpp|devilution::GetFontPath(GameFontTables size, uint16_t row, string_view ext, char *out) sha=c79e35b67172
fn get_font_path(size: GameFontTables, row: u16, ext: &str) -> String {
    format!("fonts\\{}-{:02x}{}", FONT_SIZES[size as usize], row, ext)
}

/// `GetFontId`
// @port engine/render/text_render.cpp|devilution::GetFontId(GameFontTables size, uint16_t row) sha=7ec33e5b881a
fn get_font_id(size: GameFontTables, row: u16) -> u32 {
    ((size as u32) << 16) | row as u32
}

/// `LoadFont`
// @port engine/render/text_render.cpp|devilution::LoadFont(GameFontTables size, text_color color, uint16_t row) sha=dc0dae314048
fn load_font(ctx: &mut Ctx, size: GameFontTables, color: TextColor, row: u16) -> Option<ClxSpriteList> {
    let ci = color as usize;
    if let Some(path) = COLOR_TRANSLATIONS[ci] {
        if ctx.text_render.color_translations_data[ci].is_none() {
            let mut t = Box::new([0u8; 256]);
            crate::engine::load_file::load_file_in_mem_exact(ctx, path, &mut t[..]);
            ctx.text_render.color_translations_data[ci] = Some(t);
        }
    }
    let font_id = get_font_id(size, row);
    if let Some(f) = ctx.text_render.fonts.get(&font_id) {
        return f.clone();
    }
    let path = get_font_path(size, row, ".clx");
    let mut font = crate::engine::load_sprites::load_optional_clx(ctx, &path);
    if font.is_none() {
        // An old devilutionx.mpq or fonts.mpq with PCX fonts.
        let pcx = get_font_path(size, row, "");
        font = crate::engine::load_sprites::load_pcx_sprite_list(ctx, &pcx, 256, Some(1), None, true);
    }
    if font.is_none() {
        log::error!("Error loading font: {}", path);
    }
    ctx.text_render.fonts.insert(font_id, font.clone());
    font
}

/// `CurrentFont`
#[derive(Default)]
struct CurrentFont {
    sprite: Option<ClxSpriteList>,
    has_attempted_load: bool,
    current_unicode_row: u32,
}

impl CurrentFont {
    /// `CurrentFont::load`
    // @port engine/render/text_render.cpp|devilution::CurrentFont::load(GameFontTables size, text_color color, char32_t next) sha=5bcf1fa7353f
    fn load(&mut self, ctx: &mut Ctx, size: GameFontTables, color: TextColor, next: char) -> bool {
        let row = get_unicode_row(next) as u32;
        if row == self.current_unicode_row && self.has_attempted_load {
            return true;
        }
        self.sprite = load_font(ctx, size, color, row as u16);
        self.has_attempted_load = true;
        self.current_unicode_row = row;
        self.sprite.is_some()
    }

    /// `CurrentFont::clear`
    // @port engine/render/text_render.cpp|devilution::CurrentFont::clear() sha=ccde7c236b54
    fn clear(&mut self) {
        self.has_attempted_load = false;
    }

    fn sprite(&self) -> &ClxSpriteList {
        // As in the original, a failed load of the current row is not re-checked here
        // (`load` returned true for a row already attempted).
        self.sprite.as_ref().expect("font")
    }
}

/// `DrawFont`
// @port engine/render/text_render.cpp|devilution::DrawFont(const Surface &out, Point position, const ClxSpriteList font, text_color color, int frame, bool outline) sha=e01aab6c0fa4
fn draw_font(ctx: &Ctx, out: &Surface, position: (i32, i32), font: &ClxSpriteList, color: TextColor, frame: usize, outline: bool) {
    let glyph = font.get(frame);
    if outline {
        clx_draw_outline_skip_color_zero(out, 0, (position.0, position.1 + glyph.height() as i32 - 1), &glyph);
    }
    match &ctx.text_render.color_translations_data[color as usize] {
        Some(trn) => render_clx_sprite_with_trn(out, &glyph, position, trn),
        None => render_clx_sprite(out, &glyph, position),
    }
}

/// `RenderClxSprite` (engine/render/clx_render.hpp): `position` is the top-left corner.
// @port engine/render/clx_render.hpp|devilution::RenderClxSprite(const Surface &out, ClxSprite clx, Point position) sha=4380d385014d
pub fn render_clx_sprite(out: &Surface, clx: &crate::engine::clx_sprite::ClxSprite, position: (i32, i32)) {
    clx_draw(out, (position.0, position.1 + clx.height() as i32 - 1), clx);
}

/// `RenderClxSpriteWithTRN` (engine/render/clx_render.hpp)
// @port engine/render/clx_render.hpp|devilution::RenderClxSpriteWithTRN(const Surface &out, ClxSprite clx, Point position, const uint8_t *trn) sha=12f1f3844d94
pub fn render_clx_sprite_with_trn(out: &Surface, clx: &crate::engine::clx_sprite::ClxSprite, position: (i32, i32), trn: &[u8; 256]) {
    clx_draw_trn(out, (position.0, position.1 + clx.height() as i32 - 1), clx, trn);
}

/// `IsWhitespace`
// @port engine/render/text_render.cpp|devilution::IsWhitespace(char32_t c) sha=2ecffc1828be
fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\u{3000}' || c == ZWSP
}

/// `IsFullWidthPunct`
// @port engine/render/text_render.cpp|devilution::IsFullWidthPunct(char32_t c) sha=1c2d2ca850dd
fn is_full_width_punct(c: char) -> bool {
    matches!(c, '，' | '、' | '。' | '？' | '！')
}

/// `IsBreakAllowed`
// @port engine/render/text_render.cpp|devilution::IsBreakAllowed(char32_t codepoint, char32_t nextCodepoint) sha=82f817a70092
fn is_break_allowed(c: char, next: char) -> bool {
    is_full_width_punct(c) && !is_full_width_punct(next)
}

/// `CountNewlines`
// @port engine/render/text_render.cpp|devilution::CountNewlines(string_view fmt, const DrawStringFormatArg *args, std::size_t argsLen) sha=bc5dc071a1c8
fn count_newlines(fmt: &str, args: &[DrawStringFormatArg]) -> usize {
    let mut result = fmt.matches('\n').count();
    for a in args {
        if a.is_string() {
            result += a.get_formatted().matches('\n').count();
        }
    }
    result
}

/// `FmtArgParser`
struct FmtArgParser {
    fmt: String,
    next: usize,
}

impl FmtArgParser {
    /// `FmtArgParser::operator()`: if `rest` starts with a `{...}` argument, formats it, consumes
    /// it from `rest` and returns its index.
    // @port engine/render/text_render.cpp|devilution::FmtArgParser::operator()() sha=310621332799
    fn parse(&mut self, rest: &mut &str, args: &mut [DrawStringFormatArg]) -> Option<usize> {
        if !rest.starts_with('{') {
            return None;
        }
        let Some(closing) = rest[1..].find('}').map(|p| p + 1) else {
            log::error!("Unclosed format argument: {}", self.fmt);
            return None;
        };
        let bytes = rest.as_bytes();
        let (result, fmt_len, positional) = if closing == 2 && bytes[1].is_ascii_digit() {
            ((bytes[1] - b'0') as usize, 3usize, true)
        } else {
            let r = self.next;
            self.next += 1;
            (r, closing + 1, false)
        };
        if result >= args.len() {
            log::error!("Not enough format arguments, {} given for: {}", args.len(), self.fmt);
            return None;
        }
        if !args[result].has_formatted() {
            let spec = if positional { "{}".to_string() } else { rest[..fmt_len].to_string() };
            if let FormatValue::Int(v) = args[result].value {
                args[result].formatted = Some(format_int(&spec, v));
            }
        }
        *rest = &rest[fmt_len..];
        Some(result)
    }

    fn offset(&self) -> usize {
        self.next
    }
}

/// `ContainsSmallFontTallCodepoints`
// @port engine/render/text_render.cpp|devilution::ContainsSmallFontTallCodepoints(string_view text) sha=96cd409065f2
fn contains_small_font_tall_codepoints(text: &str) -> bool {
    text.chars().any(|c| c != ZWSP && is_small_font_tall_row(get_unicode_row(c)))
}

/// `GetLineHeight(string_view fmt, DrawStringFormatArg *args, ...)`
// @port engine/render/text_render.cpp|devilution::GetLineHeight(string_view fmt, DrawStringFormatArg *args, std::size_t argsLen, GameFontTables fontIndex) sha=2739343e601a
fn get_line_height_fmt(ctx: &Ctx, fmt: &str, args: &mut [DrawStringFormatArg], font_index: GameFontTables) -> i32 {
    if font_index == GameFontTables::GameFont12 && crate::utils::language::is_small_font_tall(ctx) {
        let mut prev = '\0';
        let mut parser = FmtArgParser { fmt: fmt.to_string(), next: 0 };
        let mut rest = fmt;
        while !rest.is_empty() {
            if (prev == '{' || prev == '}') && rest.starts_with(prev) {
                rest = &rest[1..];
                continue;
            }
            if let Some(pos) = parser.parse(&mut rest, args) {
                if contains_small_font_tall_codepoints(args[pos].get_formatted()) {
                    return SMALL_FONT_TALL_LINE_HEIGHT;
                }
                prev = '\0';
                continue;
            }
            let next = rest.chars().next().unwrap();
            rest = &rest[next.len_utf8()..];
            if next == ZWSP {
                prev = next;
                continue;
            }
            if is_small_font_tall_row(get_unicode_row(next)) {
                return SMALL_FONT_TALL_LINE_HEIGHT;
            }
        }
    }
    LINE_HEIGHTS[font_index as usize]
}

/// `ClipSurface`
// @port engine/render/text_render.cpp|devilution::ClipSurface(const Surface &out, Rectangle rect) sha=bb7fc239923b
fn clip_surface(out: &Surface, rect: Rect) -> Surface {
    if rect.h == 0 {
        return out.subregion(0, 0, (rect.x + rect.w).min(out.w()), out.h());
    }
    out.subregion(0, 0, (rect.x + rect.w).min(out.w()), (rect.y + rect.h).min(out.h()))
}

/// `MaybeWrap`
// @port engine/render/text_render.cpp|devilution::MaybeWrap(Point &characterPosition, int characterWidth, int rightMargin, int initialX, int lineHeight) sha=e69cc4096035
fn maybe_wrap(pos: &mut (i32, i32), character_width: i32, right_margin: i32, initial_x: i32, line_height: i32) {
    if pos.0 + character_width > right_margin {
        pos.0 = initial_x;
        pos.1 += line_height;
    }
}

/// `GetLineStartX`
// @port engine/render/text_render.cpp|devilution::GetLineStartX(UiFlags flags, const Rectangle &rect, int lineWidth) sha=7e2475e0af63
fn get_line_start_x(flags: UiFlags, rect: Rect, line_width: i32) -> i32 {
    if flags.has(UiFlags::ALIGN_CENTER) {
        return rect.x + (rect.w - line_width) / 2;
    }
    if flags.has(UiFlags::ALIGN_RIGHT) {
        return rect.x + rect.w - line_width;
    }
    rect.x
}

/// Loads the font for `next`, falling back to '?', as the drawing loops do.
fn load_or_question_mark(ctx: &mut Ctx, font: &mut CurrentFont, size: GameFontTables, color: TextColor, next: char) -> char {
    if font.load(ctx, size, color, next) {
        return next;
    }
    if !font.load(ctx, size, color, '?') {
        crate::appfat::app_fatal(ctx, "Missing fonts");
    }
    '?'
}

/// `DoDrawString`: returns the number of bytes drawn.
// @port engine/render/text_render.cpp|devilution::DoDrawString(const Surface &out, string_view text, Rectangle rect, Point &characterPosition, int spacing, int lineHeight, int lineWidth, int rightMargin, int bottomMargin, UiFlags flags, GameFontTables size, text_color color, bool outline) sha=0c8915e7cdd2
#[allow(clippy::too_many_arguments)]
fn do_draw_string(
    ctx: &mut Ctx,
    out: &Surface,
    text: &str,
    rect: Rect,
    pos: &mut (i32, i32),
    spacing: i32,
    line_height: i32,
    mut line_width: i32,
    right_margin: i32,
    bottom_margin: i32,
    flags: UiFlags,
    size: GameFontTables,
    color: TextColor,
    outline: bool,
) -> usize {
    let mut font = CurrentFont::default();
    let mut remaining = text;
    while let Some(next0) = remaining.chars().next() {
        if next0 == '\0' {
            break;
        }
        let cp_len = next0.len_utf8();
        if next0 == ZWSP {
            remaining = &remaining[cp_len..];
            continue;
        }
        let next = load_or_question_mark(ctx, &mut font, size, color, next0);
        let frame = (next as u32 & 0xFF) as usize;
        let width = font.sprite().get(frame).width() as i32;
        if next == '\n' || pos.0 + width > right_margin {
            let next_line_y = pos.1 + line_height;
            if next_line_y >= bottom_margin {
                break;
            }
            pos.1 = next_line_y;
            if flags.has(UiFlags::ALIGN_CENTER | UiFlags::ALIGN_RIGHT) {
                line_width = width;
                if remaining.len() > cp_len {
                    line_width += spacing + get_line_width(ctx, &remaining[cp_len..], size, spacing, None);
                }
            }
            pos.0 = get_line_start_x(flags, rect, line_width);
            if next == '\n' {
                remaining = &remaining[cp_len..];
                continue;
            }
        }
        let f = font.sprite().clone();
        draw_font(ctx, out, *pos, &f, color, frame, outline);
        pos.0 += width + spacing;
        remaining = &remaining[cp_len..];
    }
    text.len() - remaining.len()
}

/// Original: `devilution::LoadSmallSelectionSpinner` (engine/render/text_render.cpp).
// @port engine/render/text_render.cpp|devilution::LoadSmallSelectionSpinner() sha=337bdd528ebc
pub fn load_small_selection_spinner(ctx: &mut Ctx) {
    ctx.text_render.p_s_pent_spn2_cels = Some(crate::engine::load_sprites::load_cel(ctx, "data\\pentspn2", 12));
}

/// Original: `devilution::UnloadFonts` (engine/render/text_render.cpp).
// @port engine/render/text_render.cpp|devilution::UnloadFonts() sha=808bc27a9721
pub fn unload_fonts(ctx: &mut Ctx) {
    ctx.text_render.fonts.clear();
}

/// Original: `devilution::GetLineWidth(string_view text, ...)` (engine/render/text_render.cpp).
/// Defaults in the original: size GameFont12, spacing 1.
// @port engine/render/text_render.cpp|devilution::GetLineWidth(string_view text, GameFontTables size, int spacing, int *charactersInLine) sha=ed9b2ba63719
pub fn get_line_width(ctx: &mut Ctx, text: &str, size: GameFontTables, spacing: i32, characters_in_line: Option<&mut i32>) -> i32 {
    let mut line_width = 0;
    let mut font = CurrentFont::default();
    let mut codepoints = 0;
    for next in text.chars() {
        if next == ZWSP {
            continue;
        }
        if next == '\n' {
            break;
        }
        let next = load_or_question_mark(ctx, &mut font, size, TextColor::DialogWhite, next);
        let frame = (next as u32 & 0xFF) as usize;
        line_width += font.sprite().get(frame).width() as i32 + spacing;
        codepoints += 1;
    }
    if let Some(c) = characters_in_line {
        *c = codepoints;
    }
    if line_width != 0 { line_width - spacing } else { 0 }
}

/// Original: `devilution::GetLineWidth(string_view fmt, DrawStringFormatArg *args, ...)`.
// @port engine/render/text_render.cpp|devilution::GetLineWidth(string_view fmt, DrawStringFormatArg *args, std::size_t argsLen, size_t argsOffset, GameFontTables size, int spacing, int *charactersInLine) sha=240b61c0e713
pub fn get_line_width_fmt(
    ctx: &mut Ctx,
    fmt: &str,
    args: &mut [DrawStringFormatArg],
    args_offset: usize,
    size: GameFontTables,
    spacing: i32,
    characters_in_line: Option<&mut i32>,
) -> i32 {
    let mut line_width = 0;
    let mut font = CurrentFont::default();
    let mut codepoints = 0;
    let mut prev = '\0';
    let mut parser = FmtArgParser { fmt: fmt.to_string(), next: args_offset };
    let mut rest = fmt;
    while !rest.is_empty() {
        if (prev == '{' || prev == '}') && rest.starts_with(prev) {
            rest = &rest[1..];
            continue;
        }
        if let Some(pos) = parser.parse(&mut rest, args) {
            let mut arg_cps = 0;
            let s = args[pos].get_formatted().to_string();
            line_width += get_line_width(ctx, &s, size, spacing, Some(&mut arg_cps));
            codepoints += arg_cps;
            prev = '\0';
            continue;
        }
        let next = rest.chars().next().unwrap();
        rest = &rest[next.len_utf8()..];
        if next == ZWSP {
            prev = next;
            continue;
        }
        if next == '\n' {
            break;
        }
        let next = load_or_question_mark(ctx, &mut font, size, TextColor::DialogWhite, next);
        let frame = (next as u32 & 0xFF) as usize;
        line_width += font.sprite().get(frame).width() as i32 + spacing;
        codepoints += 1;
        prev = next;
    }
    if let Some(c) = characters_in_line {
        *c = codepoints;
    }
    if line_width != 0 { line_width - spacing } else { 0 }
}

/// Original: `devilution::GetLineHeight(string_view text, GameFontTables fontIndex)`.
// @port engine/render/text_render.cpp|devilution::GetLineHeight(string_view text, GameFontTables fontIndex) sha=ab44d5de6635
pub fn get_line_height(ctx: &Ctx, text: &str, font_index: GameFontTables) -> i32 {
    if font_index == GameFontTables::GameFont12 && crate::utils::language::is_small_font_tall(ctx) && contains_small_font_tall_codepoints(text) {
        return SMALL_FONT_TALL_LINE_HEIGHT;
    }
    LINE_HEIGHTS[font_index as usize]
}

/// Original: `devilution::AdjustSpacingToFitHorizontally` (engine/render/text_render.cpp).
// @port engine/render/text_render.cpp|devilution::AdjustSpacingToFitHorizontally(int &lineWidth, int maxSpacing, int charactersInLine, int availableWidth) sha=15cfb422760f
pub fn adjust_spacing_to_fit_horizontally(line_width: &mut i32, max_spacing: i32, characters_in_line: i32, available_width: i32) -> i32 {
    if *line_width <= available_width || characters_in_line < 2 {
        return max_spacing;
    }
    let overhang = *line_width - available_width;
    let spacing_redux = (overhang + characters_in_line - 2) / (characters_in_line - 1);
    *line_width -= spacing_redux * (characters_in_line - 1);
    max_spacing - spacing_redux
}

/// Original: `devilution::WordWrapString` (engine/render/text_render.cpp). Defaults in the
/// original: size GameFont12, spacing 1.
// @port engine/render/text_render.cpp|devilution::WordWrapString(string_view text, unsigned width, GameFontTables size, int spacing) sha=daf9c49065ea
pub fn word_wrap_string(ctx: &mut Ctx, text: &str, width: u32, size: GameFontTables, spacing: i32) -> String {
    let mut output = String::new();
    if text.is_empty() || text.starts_with('\0') {
        return output;
    }
    let mut processed_end = 0usize;
    let mut last_breakable_pos: Option<usize> = None;
    let mut last_breakable_len = 0usize;
    let mut last_breakable_keep = false;
    let mut line_width: i32 = 0;
    let mut font = CurrentFont::default();
    let mut pos = 0usize; // start of `remaining`
    let mut next_codepoint = text.chars().next().unwrap();
    loop {
        let mut codepoint = next_codepoint;
        let codepoint_len = codepoint.len_utf8();
        pos += codepoint_len;
        next_codepoint = text[pos..].chars().next().unwrap_or('\0');
        if codepoint == '\n' {
            last_breakable_pos = None;
            line_width = 0;
            output.push_str(&text[processed_end..pos]);
            processed_end = pos;
        } else {
            if codepoint != ZWSP {
                let frame = (codepoint as u32 & 0xFF) as usize;
                codepoint = load_or_question_mark(ctx, &mut font, size, TextColor::DialogWhite, codepoint);
                line_width += font.sprite().get(frame).width() as i32 + spacing;
            }
            let ws = is_whitespace(codepoint);
            if ws || is_break_allowed(codepoint, next_codepoint) {
                last_breakable_pos = Some(pos - codepoint_len);
                last_breakable_len = codepoint_len;
                last_breakable_keep = !ws;
            } else if (line_width - spacing) as u32 > width {
                if let Some(lbp) = last_breakable_pos {
                    let mut end = lbp;
                    if last_breakable_keep {
                        end += last_breakable_len;
                    }
                    output.push_str(&text[processed_end..end]);
                    output.push('\n');
                    pos = lbp + last_breakable_len;
                    processed_end = pos;
                    last_breakable_pos = None;
                    line_width = 0;
                    next_codepoint = text[pos..].chars().next().unwrap_or('\0');
                }
            }
        }
        if pos >= text.len() || text[pos..].starts_with('\0') {
            break;
        }
    }
    output.push_str(&text[processed_end..pos.min(text.len())]);
    output
}

/// Original: `devilution::DrawString(const Surface &, string_view, const Rectangle &, UiFlags, int, int)`
/// (engine/render/text_render.cpp). Defaults in the original: flags None, spacing 1, line height -1.
// @port engine/render/text_render.cpp|devilution::DrawString(const Surface &out, string_view text, const Rectangle &rect, UiFlags flags, int spacing, int lineHeight) sha=ffe5c534949a
pub fn draw_string(ctx: &mut Ctx, out: &Surface, text: &str, rect: Rect, flags: UiFlags, spacing: i32, line_height: i32) -> u32 {
    let size = get_size_from_flags(flags);
    let color = get_color_from_flags(flags);
    let mut characters_in_line = 0;
    let mut line_width = 0;
    let mut spacing = spacing;
    let mut line_height = line_height;
    if flags.has(UiFlags::ALIGN_CENTER | UiFlags::ALIGN_RIGHT | UiFlags::KERNING_FIT_SPACING) {
        line_width = get_line_width(ctx, text, size, spacing, Some(&mut characters_in_line));
    }
    let max_spacing = spacing;
    if flags.has(UiFlags::KERNING_FIT_SPACING) {
        spacing = adjust_spacing_to_fit_horizontally(&mut line_width, max_spacing, characters_in_line, rect.w);
    }
    let mut pos = (get_line_start_x(flags, rect, line_width), rect.y);
    let initial_x = pos.0;
    let right_margin = rect.x + rect.w;
    let bottom_margin = if rect.h != 0 { (rect.y + rect.h + BASE_LINE_OFFSET[size as usize]).min(out.h()) } else { out.h() };
    if line_height == -1 {
        line_height = get_line_height(ctx, text, size);
    }
    if flags.has(UiFlags::VERTICAL_CENTER) {
        let text_height = (text.matches('\n').count() as i32 + 1) * line_height;
        pos.1 += 0.max((rect.h - text_height) / 2);
    }
    pos.1 += BASE_LINE_OFFSET[size as usize];
    let outlined = flags.has(UiFlags::OUTLINED);
    let clipped = clip_surface(out, rect);
    let bytes_drawn = do_draw_string(ctx, &clipped, text, rect, &mut pos, spacing, line_height, line_width, right_margin, bottom_margin, flags, size, color, outlined);
    draw_cursors(ctx, &clipped, flags, &mut pos, right_margin, initial_x, line_height, size, color, outlined);
    bytes_drawn as u32
}

/// The PentaCursor / TextCursor tails of DrawString and DrawStringWithColors.
#[allow(clippy::too_many_arguments)]
fn draw_cursors(
    ctx: &mut Ctx,
    clipped: &Surface,
    flags: UiFlags,
    pos: &mut (i32, i32),
    right_margin: i32,
    initial_x: i32,
    line_height: i32,
    size: GameFontTables,
    color: TextColor,
    outlined: bool,
) {
    if flags.has(UiFlags::PENTA_CURSOR) {
        let cels = ctx.text_render.p_s_pent_spn2_cels.clone().expect("pSPentSpn2Cels");
        let sprite = cels.get(pent_spn2_spin(ctx) as usize);
        maybe_wrap(pos, sprite.width() as i32, right_margin, initial_x, line_height);
        clx_draw(clipped, (pos.0, pos.1 + line_height - BASE_LINE_OFFSET[size as usize]), &sprite);
    } else if flags.has(UiFlags::TEXT_CURSOR) && crate::engine::get_animation_frame(ctx, 2, 500) != 0 {
        maybe_wrap(pos, 2, right_margin, initial_x, line_height);
        if let Some(base_font) = load_font(ctx, size, color, 0) {
            draw_font(ctx, clipped, *pos, &base_font, color, b'|' as usize, outlined);
        }
    }
}

/// Original: `devilution::DrawString(const Surface &, string_view, const Point &, ...)` (text_render.hpp).
// @port engine/render/text_render.hpp|devilution::DrawString(const Surface &out, string_view text, const Point &position, UiFlags flags = UiFlags::None, int spacing = 1, int lineHeight = -1) sha=d82146c1d261
pub fn draw_string_at(ctx: &mut Ctx, out: &Surface, text: &str, position: (i32, i32), flags: UiFlags, spacing: i32, line_height: i32) -> u32 {
    draw_string(ctx, out, text, Rect::new(position.0, position.1, out.w() - position.0, 0), flags, spacing, line_height)
}

/// Original: `devilution::DrawStringWithColors` (engine/render/text_render.cpp).
// @port engine/render/text_render.cpp|devilution::DrawStringWithColors(const Surface &out, string_view fmt, DrawStringFormatArg *args, std::size_t argsLen, const Rectangle &rect, UiFlags flags, int spacing, int lineHeight) sha=a2a49d77af5a
pub fn draw_string_with_colors(ctx: &mut Ctx, out: &Surface, fmt: &str, args: &mut [DrawStringFormatArg], rect: Rect, flags: UiFlags, spacing: i32, line_height: i32) {
    let size = get_size_from_flags(flags);
    let color = get_color_from_flags(flags);
    let mut characters_in_line = 0;
    let mut line_width = 0;
    let mut spacing = spacing;
    let mut line_height = line_height;
    if flags.has(UiFlags::ALIGN_CENTER | UiFlags::ALIGN_RIGHT | UiFlags::KERNING_FIT_SPACING) {
        line_width = get_line_width_fmt(ctx, fmt, args, 0, size, spacing, Some(&mut characters_in_line));
    }
    let max_spacing = spacing;
    if flags.has(UiFlags::KERNING_FIT_SPACING) {
        spacing = adjust_spacing_to_fit_horizontally(&mut line_width, max_spacing, characters_in_line, rect.w);
    }
    let mut pos = (get_line_start_x(flags, rect, line_width), rect.y);
    let initial_x = pos.0;
    let right_margin = rect.x + rect.w;
    let bottom_margin = if rect.h != 0 { (rect.y + rect.h + BASE_LINE_OFFSET[size as usize]).min(out.h()) } else { out.h() };
    if line_height == -1 {
        line_height = get_line_height_fmt(ctx, fmt, args, size);
    }
    if flags.has(UiFlags::VERTICAL_CENTER) {
        let text_height = (count_newlines(fmt, args) as i32 + 1) * line_height;
        pos.1 += 0.max((rect.h - text_height) / 2);
    }
    pos.1 += BASE_LINE_OFFSET[size as usize];
    let outlined = flags.has(UiFlags::OUTLINED);
    let clipped = clip_surface(out, rect);
    let mut font = CurrentFont::default();
    let mut prev = '\0';
    let mut remaining = fmt;
    let mut parser = FmtArgParser { fmt: fmt.to_string(), next: 0 };
    while let Some(next0) = remaining.chars().next() {
        if next0 == '\0' {
            break;
        }
        let cp_len = next0.len_utf8();
        if ((prev == '{' || prev == '}') && prev == next0) || next0 == ZWSP {
            remaining = &remaining[cp_len..];
            prev = next0;
            continue;
        }
        if let Some(pos_arg) = parser.parse(&mut remaining, args) {
            let s = args[pos_arg].get_formatted().to_string();
            let c = get_color_from_flags(args[pos_arg].get_flags());
            do_draw_string(ctx, &clipped, &s, rect, &mut pos, spacing, line_height, line_width, right_margin, bottom_margin, flags, size, c, outlined);
            prev = '\0';
            font.clear();
            continue;
        }
        let next = load_or_question_mark(ctx, &mut font, size, color, next0);
        let frame = (next as u32 & 0xFF) as usize;
        let width = font.sprite().get(frame).width() as i32;
        if next == '\n' || pos.0 + width > right_margin {
            let next_line_y = pos.1 + line_height;
            if next_line_y >= bottom_margin {
                break;
            }
            pos.1 = next_line_y;
            if flags.has(UiFlags::ALIGN_CENTER | UiFlags::ALIGN_RIGHT) {
                line_width = width;
                if remaining.len() > cp_len {
                    let off = parser.offset();
                    line_width += spacing + get_line_width_fmt(ctx, &remaining[cp_len..], args, off, size, spacing, None);
                }
            }
            pos.0 = get_line_start_x(flags, rect, line_width);
            if next == '\n' {
                remaining = &remaining[cp_len..];
                prev = next;
                continue;
            }
        }
        let f = font.sprite().clone();
        draw_font(ctx, &clipped, pos, &f, color, frame, outlined);
        pos.0 += width + spacing;
        remaining = &remaining[cp_len..];
        prev = next;
    }
    draw_cursors(ctx, &clipped, flags, &mut pos, right_margin, initial_x, line_height, size, color, outlined);
}

/// Original: `devilution::PentSpn2Spin` (engine/render/text_render.cpp).
// @port engine/render/text_render.cpp|devilution::PentSpn2Spin() sha=de637fff1e1c
pub fn pent_spn2_spin(ctx: &Ctx) -> u8 {
    ((ctx.platform.ticks() / 50) % 8) as u8
}

/// Original: `devilution::HaveExtraFonts` (init.h).
// @port init.h|devilution::HaveExtraFonts() sha=0c5708d81b77
pub fn have_extra_fonts(ctx: &Ctx) -> bool {
    ctx.init.archives.font_mpq.is_some()
}

#[cfg(test)]
mod tests {
    #[test]
    fn int_specs() {
        assert_eq!(super::format_int("{}", 5), "5");
        assert_eq!(super::format_int("{:+d}", 5), "+5");
        assert_eq!(super::format_int("{:02d}", 5), "05");
        assert_eq!(super::format_int("{:3}", -5), " -5");
    }
}
