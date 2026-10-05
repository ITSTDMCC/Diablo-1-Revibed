//! Ports of DevilutionX 1.5.3's test/scrollrt_test.cpp (tiles in view, tile offsets and rows
//! covered by the panel for several resolutions and zoom) and diablo_test.cpp.

use diablo1_rs::ctx::Ctx;
use diablo1_rs::platform::Platform;

fn setup(width: i32, height: i32, viewport_height: i32, zoom: bool) -> Ctx {
    let mut ctx = Ctx::new(Platform::headless_from_env());
    ctx.diablo.headless_mode = true; // test/main.cpp: HeadlessMode = true
    ctx.dx.gn_screen_width = width;
    ctx.dx.gn_screen_height = height;
    ctx.dx.gn_viewport_height = viewport_height;
    let _ = ctx.options.graphics.zoom.set_value(zoom);
    ctx
}

#[test]
fn calc_tiles_in_view() {
    use diablo1_rs::engine::render::scrollrt::tiles_in_view;
    assert_eq!(tiles_in_view(&setup(640, 480, 480 - 128, false)), (10, 11), "original");
    assert_eq!(tiles_in_view(&setup(640, 480, 480 - 128, true)), (5, 6), "original zoom");
    assert_eq!(tiles_in_view(&setup(960, 540, 540, false)), (15, 17), "960x540");
    assert_eq!(tiles_in_view(&setup(640, 512, 512 - 128, false)), (10, 12), "640x512");
    assert_eq!(tiles_in_view(&setup(768, 480, 480, true)), (6, 8), "768x480 zoom");
}

#[test]
fn calc_tile_offset() {
    use diablo1_rs::engine::render::scrollrt::calc_tile_offset;
    assert_eq!(calc_tile_offset(&setup(640, 480, 480 - 128, false)), (0, 0), "original");
    assert_eq!(calc_tile_offset(&setup(640, 480, 480 - 128, true)), (0, 8), "original zoom");
    assert_eq!(calc_tile_offset(&setup(960, 540, 540, false)), (0, 2), "960x540");
    assert_eq!(calc_tile_offset(&setup(853, 480, 480, false)), (21, 0), "853x480");
    assert_eq!(calc_tile_offset(&setup(768, 480, 480, true)), (0, 8), "768x480 zoom");
}

#[test]
fn calc_tiles_covered_by_panel() {
    use diablo1_rs::engine::render::scrollrt::rows_covered_by_panel;
    for (width, zoom, expected) in [(640, false, 0), (960, false, 4), (960, true, 2)] {
        let mut c = setup(width, 480, 480, zoom);
        diablo1_rs::control::calculate_panel_areas(&mut c);
        assert_eq!(rows_covered_by_panel(&c), expected, "width {width} zoom {zoom}");
    }
}

#[test]
fn diablo_pause_game_unpause() {
    let mut c = setup(640, 480, 352, false);
    c.init.gb_is_multiplayer = false;
    c.diablo.pause_mode = 1;
    diablo1_rs::diablo_game::diablo_pause_game(&mut c);
    assert_eq!(c.diablo.pause_mode, 0);
}
