//! Developer probe: decode a PCX through PcxToClx + ClxDraw and save it as PNG.
//! cargo run --example render_probe -- <data dir> <pcx name without extension> <out.png> [frames]
use diablo1_rs::ctx::Ctx;
use diablo1_rs::engine::render::clx_render::clx_draw;
use diablo1_rs::engine::surface::OwnedSurface;
use diablo1_rs::platform::{png, Platform};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut ctx = Ctx::new(Platform::headless_from_env());
    ctx.paths.set_base_path(&a[1]);
    diablo1_rs::init::load_core_archives(&mut ctx);
    diablo1_rs::init::load_game_archives(&mut ctx);
    let frames: i32 = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
    let mut pal = [[0u8; 3]; 256];
    let list = diablo1_rs::engine::load_sprites::load_pcx_sprite_list(&mut ctx, &a[2], frames, None, Some(&mut pal), true).expect("pcx");
    let sprite = list.get(0);
    let (w, h) = (sprite.width() as i32, sprite.height() as i32 * list.num_sprites() as i32);
    let mut surf = OwnedSurface::new(w, h);
    let view = surf.view();
    for (i, s) in list.iter().enumerate() {
        clx_draw(&view, (0, (i as i32 + 1) * s.height() as i32 - 1), &s);
    }
    let mut rgba = Vec::new();
    for &p in &surf.pixels {
        let c = pal[p as usize];
        rgba.extend_from_slice(&[c[0], c[1], c[2], 255]);
    }
    png::write_rgba(std::path::Path::new(&a[3]), w as u32, h as u32, &rgba).unwrap();
    println!("{} sprites {}x{}", list.num_sprites(), sprite.width(), sprite.height());
}
