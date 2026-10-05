//! Ports of DevilutionX 1.5.3's utility tests: test/cursor_test.cpp, format_int_test.cpp,
//! math_test.cpp and rectangle_test.cpp. The rectangle tests in C++ exercise mixed integer widths
//! (`RectangleOf<uint8_t>` against `PointOf<int8_t>`); the port's geometry is `i32` only, so the
//! same points are checked against `i32` rectangles.

use diablo1_rs::ctx::Ctx;
use diablo1_rs::engine::geometry::{Displacement, Point, Rectangle, Size};
use diablo1_rs::platform::Platform;

#[test]
fn cursor_new_cursor() {
    let mut ctx = Ctx::new(Platform::headless_from_env());
    ctx.diablo.headless_mode = true; // test/main.cpp: HeadlessMode = true
    // Note: There is no need to initialize the cursor sprites to test this.
    diablo1_rs::cursor::new_cursor(&mut ctx, diablo1_rs::cursor::CURSOR_HOURGLASS);
    assert_eq!(ctx.cursor.pcurs, diablo1_rs::cursor::CURSOR_HOURGLASS);
}

#[test]
fn format_integer() {
    use diablo1_rs::utils::format_int::format_integer;
    for (n, s) in [(1, "1"), (12, "12"), (123, "123"), (1234, "1,234"), (1234567, "1,234,567")] {
        assert_eq!(format_integer(n), s);
        assert_eq!(format_integer(-n), format!("-{s}"));
    }
}

fn d(x: i32, y: i32) -> Displacement {
    Displacement::new(x, y)
}

#[test]
fn math_world_screen_transformation() {
    let offset = d(5, 2);
    // Diablo renders tiles with the world origin translated to the top left of the screen
    assert_eq!(offset.world_to_screen(), d(-96, -112));
    // Transformation should be reversable (as long as it's not truncating)
    assert_eq!(offset.world_to_screen().screen_to_world(), offset);
    // Tiles with y >= x will still have a negative y coordinate in screen space
    assert_eq!(d(2, 5).world_to_screen(), d(96, -112));
    // Selecting a tile on the edge of the world with the default origin
    let cursor_position = d(342, -150);
    assert_eq!(cursor_position.screen_to_world(), d(0, 10));
    // Screen > World transforms lose information, so cannot be reversed exactly using ints
    assert_eq!(cursor_position.screen_to_world().world_to_screen(), d(320, -160));
}

/// `EXPECT_FLOAT_EQ`: within 4 ULPs.
fn float_eq(a: f32, b: f32) -> bool {
    (a.to_bits() as i64 - b.to_bits() as i64).abs() <= 4
}

#[test]
fn math_normalize_displacement() {
    // Normalizing displacements transforms the value into 16 bit fixed point representations
    assert!(float_eq(d(5, 0).magnitude(), 5.0));
    assert_eq!(d(5, 0).normalized(), d(1 << 16, 0));
    assert!(float_eq(d(3, 4).magnitude(), 5.0));
    assert_eq!(d(3, 4).normalized(), d(39321, 52428));
    assert!(float_eq(d(-5, 2).magnitude(), 5.385_164_7));
    assert_eq!(d(-5, 2).normalized(), d(-60848, 24339));
}

#[test]
fn math_missile_transformation() {
    // starting with a Displacement 2 world units West results in a vector pointing left of screen
    assert_eq!(d(2, -2).world_to_normal_screen(), d(-65536, 0));
    assert_eq!(d(4, -4).world_to_normal_screen(), d(-65536, 0));
    // Because of the isometric projection the y axis gets squashed
    assert_eq!(d(8, 1).world_to_normal_screen(), d(-40235, -25865));
    assert_eq!(d(8, 0).world_to_normal_screen(), d(-46340, -23170));
}

#[test]
fn rectangle_contains() {
    let rect = Rectangle::new(Point::new(0, 0), Size::new(10, 20));
    assert!(rect.contains(Point::new(9, 9)));
    assert!(!rect.contains(Point::new(-1, -1)));
    assert!(!rect.contains(Point::new(257, 257)));

    let rect = Rectangle::new(Point::new(0, 0), Size::new(255, 255));
    assert!(rect.contains(Point::new(5, 5)));
    assert!(!rect.contains(Point::new(-1, -1)));
    assert!(!rect.contains(Point::new(-2, -2)));

    let rect = Rectangle::new(Point::new(-10, -10), Size::new(127, 127));
    assert!(rect.contains(Point::new(0, 0)));
    assert!(!rect.contains(Point::new(255, 255)));
}
