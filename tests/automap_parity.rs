//! Ports of DevilutionX 1.5.3's test/automap_test.cpp and drlg_common_test.cpp. The C++ automap
//! tests share global state in file order; each Rust test starts from `InitAutomapOnce` (and
//! `StartAutomap` where the C++ sequence had already run it).

use diablo1_rs::automap::*;
use diablo1_rs::ctx::Ctx;
use diablo1_rs::engine::geometry::{points_in_rectangle, points_in_rectangle_col_major, Point, Rectangle, Size};
use diablo1_rs::platform::Platform;

fn setup() -> Ctx {
    let mut ctx = Ctx::new(Platform::headless_from_env());
    ctx.diablo.headless_mode = true; // test/main.cpp: HeadlessMode = true
    init_automap_once(&mut ctx);
    ctx
}

fn lines(c: &Ctx) -> [i32; 5] {
    [am_line(c, 64), am_line(c, 32), am_line(c, 16), am_line(c, 8), am_line(c, 4)]
}

#[test]
fn init_automap() {
    let c = setup();
    assert!(!c.automap.AutomapActive);
    assert_eq!(c.automap.AutoMapScale, 50);
    assert_eq!(lines(&c), [32, 16, 8, 4, 2]);
}

#[test]
fn start_automap_resets_offset() {
    let mut c = setup();
    start_automap(&mut c);
    assert_eq!((c.automap.AutomapOffset.delta_x, c.automap.AutomapOffset.delta_y), (0, 0));
    assert!(c.automap.AutomapActive);
}

fn moved(f: fn(&mut Ctx)) -> (i32, i32) {
    let mut c = setup();
    c.automap.AutomapOffset.delta_x = 1;
    c.automap.AutomapOffset.delta_y = 1;
    f(&mut c);
    (c.automap.AutomapOffset.delta_x, c.automap.AutomapOffset.delta_y)
}

#[test]
fn automap_scrolling() {
    assert_eq!(moved(automap_up), (0, 0), "up");
    assert_eq!(moved(automap_down), (2, 2), "down");
    assert_eq!(moved(automap_left), (0, 2), "left");
    assert_eq!(moved(automap_right), (2, 0), "right");
}

fn zoomed(scale: i32, steps: &[fn(&mut Ctx)]) -> Ctx {
    let mut c = setup();
    c.automap.AutoMapScale = scale;
    for f in steps {
        f(&mut c);
    }
    c
}

#[test]
fn automap_zoom() {
    let c = zoomed(50, &[automap_zoom_in]);
    assert_eq!((c.automap.AutoMapScale, lines(&c)), (55, [35, 17, 8, 4, 2]), "zoom in");
    let c = zoomed(195, &[automap_zoom_in, automap_zoom_in]);
    assert_eq!((c.automap.AutoMapScale, lines(&c)), (200, [128, 64, 32, 16, 8]), "zoom in max");
    let c = zoomed(200, &[automap_zoom_out]);
    assert_eq!((c.automap.AutoMapScale, lines(&c)), (195, [124, 62, 31, 15, 7]), "zoom out");
    let c = zoomed(55, &[automap_zoom_out, automap_zoom_out]);
    assert_eq!((c.automap.AutoMapScale, lines(&c)), (50, [32, 16, 8, 4, 2]), "zoom out min");
}

#[test]
fn automap_zoom_reset_restores_defaults() {
    let mut c = setup();
    c.automap.AutoMapScale = 50;
    c.automap.AutomapOffset.delta_x = 1;
    c.automap.AutomapOffset.delta_y = 1;
    automap_zoom_reset(&mut c);
    assert_eq!((c.automap.AutomapOffset.delta_x, c.automap.AutomapOffset.delta_y), (0, 0));
    assert_eq!((c.automap.AutoMapScale, lines(&c)), (50, [32, 16, 8, 4, 2]));
}

// ---------------------------------------------------------------------------------------------
// drlg_common_test.cpp

#[test]
fn rectangle_range_iterator() {
    let top_left = Rectangle::from_center(Point::new(1, 1), 1);
    let bottom_right = Rectangle::from_center(Point::new(3, 3), 1);
    // Dungeon generation depends on the iteration order remaining unchanged

    let mut region = [[0; 5]; 5];
    let mut counter = 0;
    for p in points_in_rectangle(top_left) {
        counter += 1;
        region[p.x as usize][p.y as usize] = counter;
    }
    assert_eq!(counter, 9);
    assert_eq!(region[2][2], 9);
    assert_eq!((region[0][0], region[1][0], region[2][0], region[0][2]), (1, 2, 3, 7), "Default order should be row-major");
    assert_eq!((region[0][3], region[3][0]), (0, 0), "Iterator should not return out of bounds points");

    let mut region = [[0; 5]; 5];
    let mut counter = 0;
    for p in points_in_rectangle(bottom_right).rev() {
        counter += 1;
        region[p.x as usize][p.y as usize] = counter;
    }
    assert_eq!((region[4][4], region[2][4], region[4][2], region[2][2]), (1, 3, 7, 9), "Reverse iterators are required");

    let mut region = [[0; 5]; 5];
    let mut counter = 0;
    for p in points_in_rectangle_col_major(top_left) {
        counter += 1;
        region[p.x as usize][p.y as usize] = counter;
    }
    assert_eq!(counter, 9);
    assert_eq!(region[2][2], 9);
    assert_eq!((region[0][0], region[0][1], region[0][2], region[2][0]), (1, 2, 3, 7), "col-major iterator must use col-major order");
    assert_eq!((region[0][3], region[3][0]), (0, 0), "Iterator should not return out of bounds points");

    let mut region = [[0; 5]; 5];
    let mut counter = 0;
    for p in points_in_rectangle_col_major(bottom_right).rev() {
        counter += 1;
        region[p.x as usize][p.y as usize] = counter;
    }
    assert_eq!((region[4][4], region[4][2], region[2][4], region[2][2]), (1, 3, 7, 9), "Reverse iterators are required");
}

#[test]
fn theme_room_size() {
    let mut c = setup();
    c.gendung.dungeon = [[0; 40]; 40];
    // GetSizeForThemeRoom() (test overload) = GetSizeForThemeRoom(0, { 0, 0 }, 5, 10)
    let size = |c: &Ctx| diablo1_rs::levels::gendung::get_size_for_theme_room(c, 0, Point::new(0, 0), 5, 10);
    let mut set = |c: &mut Ctx, x: usize, y: usize, v: u8| c.gendung.dungeon[x][y] = v;

    assert_eq!(size(&c), Some(Size::new(8, 8)), "All floor theme area should be 8x8");
    set(&mut c, 9, 9, 1);
    assert_eq!(size(&c), Some(Size::new(7, 7)), "Corners shrink the chosen dimensions");
    set(&mut c, 9, 5, 1);
    assert_eq!(size(&c), Some(Size::new(7, 3)), "Minimum dimensions are determined by corners outside the min area");
    set(&mut c, 9, 4, 1);
    assert_eq!(size(&c), Some(Size::new(7, 8)), "Walls below the min size let larger opposing dimensions get picked");
    set(&mut c, 9, 5, 0);
    set(&mut c, 9, 4, 0);
    set(&mut c, 9, 9, 0);

    // Time for some unusual cases
    set(&mut c, 7, 2, 1);
    set(&mut c, 5, 9, 1);
    assert_eq!(size(&c), Some(Size::new(5, 7)), "Search space terminates at width 8 due to the wall being in the first three rows");
    set(&mut c, 6, 4, 1);
    assert_eq!(size(&c), Some(Size::new(4, 7)), "Smallest width now defined by row 5, height still extends due to minSize");
    set(&mut c, 6, 4, 0);
    set(&mut c, 5, 9, 0);
    set(&mut c, 7, 2, 0);

    set(&mut c, 7, 0, 1);
    set(&mut c, 6, 6, 1);
    set(&mut c, 8, 5, 1);
    assert_eq!(size(&c), Some(Size::new(4, 4)), "Search is terminated by the 0 width row 7, inset corner gives a larger height than otherwise expected");
}
