//! `Source/engine/palette.cpp`: palettes, gamma, fades, colour cycling, transparency tables.

use crate::ctx::Ctx;
use crate::engine::dx::Color;

pub struct PaletteState {
    pub logical_palette: [Color; 256],
    pub system_palette: [Color; 256],
    pub orig_palette: [Color; 256],
    /// `paletteTransparencyLookup[256][256]`
    pub palette_transparency_lookup: Box<[[u8; 256]; 256]>,
    /// `paletteTransparencyLookupBlack16[65536]`
    pub palette_transparency_lookup_black16: Box<[u16; 65536]>,
    /// `sgbFadedIn`
    sgb_faded_in: bool,
    /// statics of palette_update_crypt / palette_update_hive
    delay_lava: bool,
    hive_delay: u8,
}

impl Default for PaletteState {
    fn default() -> Self {
        PaletteState {
            logical_palette: [[0; 3]; 256],
            system_palette: [[0; 3]; 256],
            orig_palette: [[0; 3]; 256],
            palette_transparency_lookup: Box::new([[0; 256]; 256]),
            palette_transparency_lookup_black16: vec![0u16; 65536].into_boxed_slice().try_into().unwrap(),
            sgb_faded_in: true,
            delay_lava: false,
            hive_delay: 0,
        }
    }
}

/// Original: `LoadGamma` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::LoadGamma() sha=65d31e4e781d
fn load_gamma(ctx: &mut Ctx) {
    let gamma_value = ctx.options.graphics.gamma_correction.get().clamp(30, 100);
    let _ = ctx.options.graphics.gamma_correction.set_value(gamma_value - gamma_value % 5);
}

/// Original: `FindBestMatchForColor` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::FindBestMatchForColor(std::array<SDL_Color, 256> &palette, SDL_Color color, int skipFrom, int skipTo) sha=c26a1cf4bf0f
fn find_best_match_for_color(palette: &[Color; 256], color: Color, skip_from: i32, skip_to: i32) -> u8 {
    let mut best = 0u8;
    let mut best_diff = u32::MAX;
    for i in 0..256 {
        if (i as i32) >= skip_from && (i as i32) <= skip_to {
            continue;
        }
        let dr = palette[i][0] as i32 - color[0] as i32;
        let dg = palette[i][1] as i32 - color[1] as i32;
        let db = palette[i][2] as i32 - color[2] as i32;
        let diff = (dr * dr + dg * dg + db * db) as u32;
        if best_diff > diff {
            best = i as u8;
            best_diff = diff;
        }
    }
    best
}

/// Original: `GenerateBlendedLookupTable` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::GenerateBlendedLookupTable(std::array<SDL_Color, 256> &palette, int skipFrom, int skipTo, int toUpdate = 256) sha=4196b5e85491
fn generate_blended_lookup_table(pal: &mut PaletteState, palette: &[Color; 256], skip_from: i32, skip_to: i32, to_update: i32) {
    let lut = &mut pal.palette_transparency_lookup;
    for i in 0..256usize {
        for j in 0..256usize {
            if i == j {
                lut[i][j] = j as u8;
                continue;
            }
            if i > j {
                lut[i][j] = lut[j][i];
                continue;
            }
            if i as i32 > to_update && j as i32 > to_update {
                continue;
            }
            let blended = [
                ((palette[i][0] as i32 + palette[j][0] as i32) / 2) as u8,
                ((palette[i][1] as i32 + palette[j][1] as i32) / 2) as u8,
                ((palette[i][2] as i32 + palette[j][2] as i32) / 2) as u8,
            ];
            lut[i][j] = find_best_match_for_color(palette, blended, skip_from, skip_to);
        }
    }
    for i in 0..256usize {
        for j in 0..256usize {
            let index = i | (j << 8);
            pal.palette_transparency_lookup_black16[index] = lut[0][i] as u16 | ((lut[0][j] as u16) << 8);
        }
    }
}

/// Original: `CycleColors` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::CycleColors(int from, int to) sha=34fadb223a20
fn cycle_colors(pal: &mut PaletteState, from: usize, to: usize) {
    pal.system_palette[from..=to].rotate_left(1);
    for row in pal.palette_transparency_lookup.iter_mut() {
        row[from..=to].rotate_left(1);
    }
    pal.palette_transparency_lookup[from..=to].rotate_left(1);
}

/// Original: `CycleColorsReverse` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::CycleColorsReverse(int from, int to) sha=6b655b4ff51c
fn cycle_colors_reverse(pal: &mut PaletteState, from: usize, to: usize) {
    pal.system_palette[from..=to].rotate_right(1);
    for row in pal.palette_transparency_lookup.iter_mut() {
        row[from..=to].rotate_right(1);
    }
    pal.palette_transparency_lookup[from..=to].rotate_right(1);
}

/// Original: `devilution::palette_update` (engine/palette.cpp): copies the system palette
/// into the SDL palette the back buffer uses. Defaults in the original: `(0, 256)`.
// @port engine/palette.cpp|devilution::palette_update(int first, int ncolor) sha=5572fdbe9c85
pub fn palette_update(ctx: &mut Ctx, first: usize, ncolor: usize) {
    if ctx.diablo.headless_mode {
        return;
    }
    let sys = ctx.dx.pal.system_palette;
    let p = ctx.dx.palette.as_mut().expect("Palette");
    p[first..first + ncolor].copy_from_slice(&sys[first..first + ncolor]);
    ctx.dx.pal_surface_palette_version += 1;
}

/// Original: `devilution::ApplyGamma` (engine/palette.cpp). `which` selects the destination
/// and source arrays of the palette state (the original passes references to them).
// @port engine/palette.cpp|devilution::ApplyGamma(std::array<SDL_Color, 256> &dst, const std::array<SDL_Color, 256> &src, int n) sha=3f26087c54ab
pub fn apply_gamma(ctx: &mut Ctx, dst: PaletteRef, src: PaletteRef, n: usize) {
    let g = ctx.options.graphics.gamma_correction.get() as f64 / 100.0;
    let src_pal = *ctx.dx.pal.get(src);
    let dst_pal = ctx.dx.pal.get_mut(dst);
    for i in 0..n {
        for c in 0..3 {
            dst_pal[i][c] = ((src_pal[i][c] as f64 / 256.0).powf(g) * 256.0) as u8;
        }
    }
    crate::engine::backbuffer_state::redraw_everything(ctx);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteRef {
    Logical,
    System,
    Orig,
}

impl PaletteState {
    fn get(&self, r: PaletteRef) -> &[Color; 256] {
        match r {
            PaletteRef::Logical => &self.logical_palette,
            PaletteRef::System => &self.system_palette,
            PaletteRef::Orig => &self.orig_palette,
        }
    }
    fn get_mut(&mut self, r: PaletteRef) -> &mut [Color; 256] {
        match r {
            PaletteRef::Logical => &mut self.logical_palette,
            PaletteRef::System => &mut self.system_palette,
            PaletteRef::Orig => &mut self.orig_palette,
        }
    }
}

/// Original: `devilution::palette_init` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::palette_init() sha=a67a65dc120c
pub fn palette_init(ctx: &mut Ctx) {
    load_gamma(ctx);
    ctx.dx.pal.system_palette = ctx.dx.pal.orig_palette;
    crate::engine::dx::init_palette(ctx);
}

/// Original: `devilution::LoadPalette` (engine/palette.cpp). `blend` defaults to true.
// @port engine/palette.cpp|devilution::LoadPalette(const char *pszFileName, bool blend) sha=a42e6ad47763
pub fn load_palette(ctx: &mut Ctx, file_name: &str, blend: bool) {
    if ctx.diablo.headless_mode {
        return;
    }
    let mut pal_data = [0u8; 256 * 3];
    crate::engine::load_file::load_file_in_mem_exact(ctx, file_name, &mut pal_data);
    for i in 0..256 {
        ctx.dx.pal.orig_palette[i] = [pal_data[i * 3], pal_data[i * 3 + 1], pal_data[i * 3 + 2]];
    }
    if blend {
        let orig = ctx.dx.pal.orig_palette;
        let lt = crate::levels::gendung::leveltype(ctx);
        use crate::levels::gendung::DungeonType::*;
        if lt == Caves || lt == Crypt {
            generate_blended_lookup_table(&mut ctx.dx.pal, &orig, 1, 31, 256);
        } else if lt == Nest {
            generate_blended_lookup_table(&mut ctx.dx.pal, &orig, 1, 15, 256);
        } else {
            generate_blended_lookup_table(&mut ctx.dx.pal, &orig, -1, -1, 256);
        }
    }
}

/// Original: `devilution::LoadRndLvlPal` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::LoadRndLvlPal(dungeon_type l) sha=63a0d3c6425d
pub fn load_rnd_lvl_pal(ctx: &mut Ctx, l: crate::levels::gendung::DungeonType) {
    use crate::levels::gendung::DungeonType::*;
    if ctx.diablo.headless_mode {
        return;
    }
    if l == Town {
        load_palette(ctx, "levels\\towndata\\town.pal", true);
        return;
    }
    let mut rv = ctx.rng.generate_rnd(4) + 1;
    if l == Crypt {
        load_palette(ctx, "nlevels\\l5data\\l5base.pal", true);
        return;
    }
    let name = if l == Nest {
        if !ctx.options.graphics.alternate_nest_art.get() {
            rv += 1;
        }
        format!("nlevels\\l{0}data\\l{0}base{1}.pal", 6, rv)
    } else {
        format!("levels\\l{0}data\\l{0}_{1}.pal", l as i32, rv)
    };
    load_palette(ctx, &name, true);
}

/// Original: `devilution::IncreaseGamma` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::IncreaseGamma() sha=2c5e5512d1c7
pub fn increase_gamma(ctx: &mut Ctx) {
    let gamma_value = ctx.options.graphics.gamma_correction.get();
    if gamma_value < 100 {
        let _ = ctx.options.graphics.gamma_correction.set_value((gamma_value + 5).min(100));
        apply_gamma(ctx, PaletteRef::System, PaletteRef::Logical, 256);
        palette_update(ctx, 0, 256);
    }
}

/// Original: `devilution::DecreaseGamma` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::DecreaseGamma() sha=a7cce553c06b
pub fn decrease_gamma(ctx: &mut Ctx) {
    let gamma_value = ctx.options.graphics.gamma_correction.get();
    if gamma_value > 30 {
        let _ = ctx.options.graphics.gamma_correction.set_value((gamma_value - 5).max(30));
        apply_gamma(ctx, PaletteRef::System, PaletteRef::Logical, 256);
        palette_update(ctx, 0, 256);
    }
}

/// Original: `devilution::UpdateGamma` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::UpdateGamma(int gamma) sha=78660f0b1a28
pub fn update_gamma(ctx: &mut Ctx, gamma: i32) -> i32 {
    if gamma > 0 {
        let _ = ctx.options.graphics.gamma_correction.set_value(130 - gamma);
        apply_gamma(ctx, PaletteRef::System, PaletteRef::Logical, 256);
        palette_update(ctx, 0, 256);
    }
    130 - ctx.options.graphics.gamma_correction.get()
}

/// Original: `devilution::SetFadeLevel` (engine/palette.cpp). `update_hardware_cursor`
/// defaults to true.
// @port engine/palette.cpp|devilution::SetFadeLevel(int fadeval, bool updateHardwareCursor) sha=154ca80386ff
pub fn set_fade_level(ctx: &mut Ctx, fadeval: i32, update_hardware_cursor: bool) {
    if ctx.diablo.headless_mode {
        return;
    }
    for i in 0..256 {
        for c in 0..3 {
            ctx.dx.pal.system_palette[i][c] = ((fadeval * ctx.dx.pal.logical_palette[i][c] as i32) / 256) as u8;
        }
    }
    palette_update(ctx, 0, 256);
    if update_hardware_cursor && crate::hwcursor::is_hardware_cursor(ctx) {
        crate::hwcursor::reinitialize_hardware_cursor(ctx);
    }
}

/// Original: `devilution::BlackPalette` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::BlackPalette() sha=d232021b6891
pub fn black_palette(ctx: &mut Ctx) {
    set_fade_level(ctx, 0, false);
}

/// Original: `devilution::PaletteFadeIn` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::PaletteFadeIn(int fr) sha=515c2f45ae60
pub fn palette_fade_in(ctx: &mut Ctx, fr: i32) {
    if ctx.diablo.headless_mode {
        return;
    }
    let mut fr = fr;
    if crate::engine::demomode::is_running(ctx) {
        fr = 0;
    }
    apply_gamma(ctx, PaletteRef::Logical, PaletteRef::Orig, 256);
    if fr > 0 {
        let tc = ctx.platform.ticks();
        let fr = (fr * 3) as u32;
        let mut prev_fade_value = 255u32;
        let mut i = 0u32;
        while i < 256 {
            if i != prev_fade_value {
                set_fade_level(ctx, i as i32, i != 0);
                prev_fade_value = i;
            }
            crate::engine::dx::blt_fast(ctx, None, None);
            crate::engine::dx::render_present(ctx);
            i = fr.wrapping_mul(ctx.platform.ticks().wrapping_sub(tc)) / 50;
        }
        set_fade_level(ctx, 256, true);
    } else {
        set_fade_level(ctx, 256, true);
        crate::engine::dx::blt_fast(ctx, None, None);
        crate::engine::dx::render_present(ctx);
    }
    ctx.dx.pal.logical_palette = ctx.dx.pal.orig_palette;
    ctx.dx.pal.sgb_faded_in = true;
}

/// Original: `devilution::PaletteFadeOut` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::PaletteFadeOut(int fr) sha=7a575fc25575
pub fn palette_fade_out(ctx: &mut Ctx, fr: i32) {
    if !ctx.dx.pal.sgb_faded_in || ctx.diablo.headless_mode {
        return;
    }
    let mut fr = fr;
    if crate::engine::demomode::is_running(ctx) {
        fr = 0;
    }
    if fr > 0 {
        let tc = ctx.platform.ticks();
        let fr = (fr * 3) as u32;
        let mut prev_fade_value = 0u32;
        let mut i = 0u32;
        while i < 256 {
            if i != prev_fade_value {
                set_fade_level(ctx, 256 - i as i32, true);
                prev_fade_value = i;
            }
            crate::engine::dx::blt_fast(ctx, None, None);
            crate::engine::dx::render_present(ctx);
            i = fr.wrapping_mul(ctx.platform.ticks().wrapping_sub(tc)) / 50;
        }
        set_fade_level(ctx, 0, true);
    } else {
        set_fade_level(ctx, 0, true);
        crate::engine::dx::blt_fast(ctx, None, None);
        crate::engine::dx::render_present(ctx);
    }
    ctx.dx.pal.sgb_faded_in = false;
}

/// Original: `devilution::palette_update_caves` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::palette_update_caves() sha=a931e2e9b04c
pub fn palette_update_caves(ctx: &mut Ctx) {
    cycle_colors(&mut ctx.dx.pal, 1, 31);
    palette_update(ctx, 0, 31);
}

/// Original: `devilution::palette_update_crypt` (engine/palette.cpp). As in the original, the
/// lava branch resets `delayLava` to false inside the block (it still alternates through the
/// final toggle).
// @port engine/palette.cpp|devilution::palette_update_crypt() sha=78f9513bbac1
pub fn palette_update_crypt(ctx: &mut Ctx) {
    if !ctx.dx.pal.delay_lava {
        cycle_colors_reverse(&mut ctx.dx.pal, 1, 15);
        ctx.dx.pal.delay_lava = false;
    }
    cycle_colors_reverse(&mut ctx.dx.pal, 16, 31);
    palette_update(ctx, 0, 31);
    ctx.dx.pal.delay_lava = !ctx.dx.pal.delay_lava;
}

/// Original: `devilution::palette_update_hive` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::palette_update_hive() sha=77ff695dcd54
pub fn palette_update_hive(ctx: &mut Ctx) {
    if ctx.dx.pal.hive_delay != 2 {
        ctx.dx.pal.hive_delay += 1;
        return;
    }
    cycle_colors_reverse(&mut ctx.dx.pal, 1, 8);
    cycle_colors_reverse(&mut ctx.dx.pal, 9, 15);
    palette_update(ctx, 0, 15);
    ctx.dx.pal.hive_delay = 0;
}

/// Original: `devilution::palette_update_quest_palette` (engine/palette.cpp).
// @port engine/palette.cpp|devilution::palette_update_quest_palette(int n) sha=a8841a3bbd85
pub fn palette_update_quest_palette(ctx: &mut Ctx, n: i32) {
    let i = (32 - n) as usize;
    ctx.dx.pal.logical_palette[i] = ctx.dx.pal.orig_palette[i];
    apply_gamma(ctx, PaletteRef::System, PaletteRef::Logical, 32);
    palette_update(ctx, 0, 31);
    let logical = ctx.dx.pal.logical_palette;
    for j in 0..256usize {
        if i == j {
            ctx.dx.pal.palette_transparency_lookup[i][j] = j as u8;
            continue;
        }
        let blended = [
            ((logical[i][0] as i32 + logical[j][0] as i32) / 2) as u8,
            ((logical[i][1] as i32 + logical[j][1] as i32) / 2) as u8,
            ((logical[i][2] as i32 + logical[j][2] as i32) / 2) as u8,
        ];
        let best = find_best_match_for_color(&logical, blended, 1, 31);
        ctx.dx.pal.palette_transparency_lookup[i][j] = best;
        ctx.dx.pal.palette_transparency_lookup[j][i] = best;
    }
}
