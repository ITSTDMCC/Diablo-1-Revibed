//! `Source/capture.cpp`: the screenshot function (PrintScreen), saving a PCX file.

use std::fs::File;
use std::io::Write;

use crate::ctx::Ctx;
use crate::engine::dx::Color;
use crate::engine::surface::Surface;
use crate::platform::log;

/// Original: `CaptureHdr` (capture.cpp): the 128-byte PCX header.
// @port capture.cpp|devilution::CaptureHdr(int16_t width, int16_t height, FILE *out) sha=378650b3fa79
fn capture_hdr(width: i16, height: i16, out: &mut File) -> bool {
    let mut buffer = [0u8; 128];
    buffer[0] = 10; // Manufacturer
    buffer[1] = 5; // Version
    buffer[2] = 1; // Encoding
    buffer[3] = 8; // BitsPerPixel
    buffer[8..10].copy_from_slice(&(width - 1).to_le_bytes()); // Xmax
    buffer[10..12].copy_from_slice(&(height - 1).to_le_bytes()); // Ymax
    buffer[12..14].copy_from_slice(&width.to_le_bytes()); // HDpi
    buffer[14..16].copy_from_slice(&height.to_le_bytes()); // VDpi
    buffer[65] = 1; // NPlanes
    buffer[66..68].copy_from_slice(&width.to_le_bytes()); // BytesPerLine
    out.write_all(&buffer).is_ok()
}

/// Original: `CapturePal` (capture.cpp).
// @port capture.cpp|devilution::CapturePal(SDL_Color *palette, FILE *out) sha=2788246b1063
fn capture_pal(palette: &[Color], out: &mut File) -> bool {
    let mut pcx_palette = [0u8; 1 + 256 * 3];
    pcx_palette[0] = 12;
    for i in 0..256 {
        pcx_palette[1 + 3 * i..1 + 3 * i + 3].copy_from_slice(&palette[i]);
    }
    out.write_all(&pcx_palette).is_ok()
}

/// Original: `CaptureEnc` (capture.cpp): RLE-compress one row of `width` pixels into `dst`.
// @port capture.cpp|devilution::CaptureEnc(uint8_t *src, uint8_t *dst, int width) sha=0f44057e05de
fn capture_enc(src: &[u8], dst: &mut Vec<u8>, mut width: i32) {
    let mut s = 0usize;
    loop {
        let rle_pixel = src[s];
        s += 1;
        let mut rle_length: u8 = 1;

        width -= 1;

        // The original reads `*src` one past the row end when width reaches 0 (then breaks).
        while s < src.len() && rle_pixel == src[s] {
            if rle_length >= 63 {
                break;
            }
            if width == 0 {
                break;
            }
            rle_length += 1;

            width -= 1;
            s += 1;
        }

        if rle_length > 1 || rle_pixel > 0xBF {
            dst.push(rle_length | 0xC0);
        }

        dst.push(rle_pixel);
        if width <= 0 {
            break;
        }
    }
}

/// Original: `CapturePix` (capture.cpp).
// @port capture.cpp|devilution::CapturePix(const Surface &buf, FILE *out) sha=3fe7639f8467
fn capture_pix(buf: &Surface, out: &mut File) -> bool {
    let width = buf.w();
    let mut p_buffer = Vec::with_capacity(2 * width as usize);
    for y in 0..buf.h() {
        // SAFETY: row `y` of the surface holds `width` pixels.
        let row = unsafe { std::slice::from_raw_parts(buf.at(0, y), width as usize) };
        p_buffer.clear();
        capture_enc(row, &mut p_buffer, width);
        if out.write_all(&p_buffer).is_err() {
            return false;
        }
    }
    true
}

/// Original: `CaptureFile` (capture.cpp).
// @port capture.cpp|devilution::CaptureFile(std::string *dstPath) sha=5b00f2d27bc6
fn capture_file(ctx: &mut Ctx) -> Option<(File, String)> {
    let tt = ctx.platform.time();
    let filename = match crate::platform::win32::localtime(tt) {
        Some((y, mo, d, h, mi, s)) => format!("Screenshot from {y:04}-{mo:02}-{d:02} {h:02}-{mi:02}-{s:02}"),
        None => "Screenshot".to_string(),
    };
    let pref = ctx.paths.pref_path().to_string();
    let mut dst_path = format!("{pref}{filename}.pcx");
    let mut i = 0;
    while std::path::Path::new(&dst_path).exists() {
        i += 1;
        dst_path = format!("{pref}{filename}-{i}.pcx");
    }
    File::create(&dst_path).ok().map(|f| (f, dst_path))
}

/// Original: `RedPalette` (capture.cpp): make the palette red and apply it to the screen.
// @port capture.cpp|devilution::RedPalette() sha=97fb02fde448
fn red_palette(ctx: &mut Ctx) {
    for c in ctx.dx.pal.system_palette.iter_mut() {
        c[1] = 0;
        c[2] = 0;
    }
    crate::engine::palette::palette_update(ctx, 0, 256);
    crate::engine::dx::blt_fast(ctx, None, None);
    crate::engine::dx::render_present(ctx);
}

/// Original: `devilution::CaptureScreen` (capture.cpp).
// @port capture.cpp|devilution::CaptureScreen() sha=b31b41701256
pub fn capture_screen(ctx: &mut Ctx) {
    let Some((mut out_stream, file_name)) = capture_file(ctx) else {
        return;
    };
    crate::engine::render::scrollrt::draw_and_blit(ctx);
    let palette = crate::engine::dx::palette_get_entries(ctx, 256);
    red_palette(ctx);

    let buf = crate::engine::dx::global_back_buffer(ctx);
    let mut success = capture_hdr(buf.w() as i16, buf.h() as i16, &mut out_stream);
    if success {
        success = capture_pix(&buf, &mut out_stream);
    }
    if success {
        success = capture_pal(&palette, &mut out_stream);
    }
    drop(out_stream);

    if !success {
        log::info!("Failed to save screenshot at {}", file_name);
        let _ = std::fs::remove_file(&file_name);
    } else {
        log::info!("Screenshot saved at {}", file_name);
    }
    ctx.platform.delay(300);
    ctx.dx.pal.system_palette[..256].copy_from_slice(&palette);
    crate::engine::palette::palette_update(ctx, 0, 256);
    crate::engine::backbuffer_state::redraw_everything(ctx);
}
