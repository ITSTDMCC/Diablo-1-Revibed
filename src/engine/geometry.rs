//! `Source/engine/{point,displacement,size,rectangle,direction,circle,world_tile}.hpp`:
//! the game's geometry types. The C++ templates are instantiated with `int` almost everywhere;
//! the port uses `i32` for all of them (`WorldTilePosition` values are converted where stored).

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Shl, Shr, Sub, SubAssign};

/// `Direction`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
#[repr(u8)]
pub enum Direction {
    #[default]
    South,
    SouthWest,
    West,
    NorthWest,
    North,
    NorthEast,
    East,
    SouthEast,
    NoDirection,
}

impl Direction {
    pub const ALL8: [Direction; 8] = [
        Direction::South,
        Direction::SouthWest,
        Direction::West,
        Direction::NorthWest,
        Direction::North,
        Direction::NorthEast,
        Direction::East,
        Direction::SouthEast,
    ];

    pub fn from_u8(v: u8) -> Direction {
        match v {
            0..=7 => Self::ALL8[v as usize],
            _ => Direction::NoDirection,
        }
    }

    pub fn idx(self) -> usize {
        self as usize
    }
}

/// Original: `devilution::Left` (engine/direction.hpp).
// @port engine/direction.hpp|devilution::Left(Direction facing) sha=e31c47b3769e
pub fn left(facing: Direction) -> Direction {
    Direction::from_u8((facing as u8 + 7) % 8)
}

/// Original: `devilution::Right` (engine/direction.hpp).
// @port engine/direction.hpp|devilution::Right(Direction facing) sha=781ce975ff97
pub fn right(facing: Direction) -> Direction {
    Direction::from_u8((facing as u8 + 1) % 8)
}

/// Original: `devilution::Opposite` (engine/direction.hpp).
// @port engine/direction.hpp|devilution::Opposite(Direction facing) sha=f44f0de2bc86
pub fn opposite(facing: Direction) -> Direction {
    Direction::from_u8((facing as u8 + 4) % 8)
}

/// Original: `devilution::DirectionToString` (engine/direction.cpp).
// @port engine/direction.cpp|devilution::DirectionToString(Direction direction) sha=73c5337c1c83
pub fn direction_to_string(direction: Direction) -> &'static str {
    match direction {
        Direction::South => "South",
        Direction::SouthWest => "SouthWest",
        Direction::West => "West",
        Direction::NorthWest => "NorthWest",
        Direction::North => "North",
        Direction::NorthEast => "NorthEast",
        Direction::East => "East",
        Direction::SouthEast => "SouthEast",
        Direction::NoDirection => "",
    }
}

/// `Displacement` (`DisplacementOf<int>`)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Displacement {
    pub delta_x: i32,
    pub delta_y: i32,
}

// @port engine/displacement.hpp|devilution::DisplacementOf::DisplacementOf(DisplacementOf<DisplacementDeltaT> other) sha=62eaee0d8fef
// @port engine/displacement.hpp|devilution::DisplacementOf::DisplacementOf(DeltaT deltaX, DeltaT deltaY) sha=e5381e573559
// @port engine/displacement.hpp|devilution::DisplacementOf::DisplacementOf(DeltaT delta) sha=eb1306b55874
// @port engine/displacement.hpp|devilution::DisplacementOf::DisplacementOf(const SizeOf<SizeT> &size) sha=0cdf4e54f27f
// @port engine/displacement.hpp|devilution::DisplacementOf::DisplacementOf(Direction direction) sha=a485f04a6c7b
// @port engine/displacement.hpp|devilution::DisplacementOf::operator==(const DisplacementOf<DisplacementDeltaT> &other) sha=0ddf0997cc11
// @port engine/displacement.hpp|devilution::DisplacementOf::operator!=(const DisplacementOf<DisplacementDeltaT> &other) sha=7305639fde16
// @port engine/displacement.hpp|devilution::DisplacementOf::magnitude() sha=7b24440306b4
// @port engine/displacement.hpp|devilution::DisplacementOf::worldToScreen() sha=3f313c4e08ed
// @port engine/displacement.hpp|devilution::DisplacementOf::screenToWorld() sha=99076b41d117
// @port engine/displacement.hpp|devilution::DisplacementOf::screenToMissile() sha=2bcb1825c89b
// @port engine/displacement.hpp|devilution::DisplacementOf::screenToLight() sha=dba6c006e0ac
// @port engine/displacement.hpp|devilution::DisplacementOf::worldToNormalScreen() sha=2660bdf8ae55
// @port engine/displacement.hpp|devilution::DisplacementOf::Rotate(int quadrants) sha=978b1882395d
// @port engine/displacement.hpp|devilution::DisplacementOf::flipX() sha=273691de0bc3
// @port engine/displacement.hpp|devilution::DisplacementOf::flipY() sha=89088b5ac648
// @port engine/displacement.hpp|devilution::DisplacementOf::flipXY() sha=5404d84aff91
// @port engine/displacement.hpp|devilution::DisplacementOf::fromDirection(Direction direction) sha=7296917db435
// @port engine/displacement.hpp|devilution::normalized() sha=51b051baeaf8
// @port engine/displacement.hpp|devilution::abs(DisplacementOf<DisplacementDeltaT> a) sha=4b63bd43e981
impl Displacement {
    pub const fn new(delta_x: i32, delta_y: i32) -> Displacement {
        Displacement { delta_x, delta_y }
    }

    pub const fn splat(delta: i32) -> Displacement {
        Displacement { delta_x: delta, delta_y: delta }
    }

    pub const fn from_size(size: Size) -> Displacement {
        Displacement { delta_x: size.width, delta_y: size.height }
    }

    pub const fn from_direction(direction: Direction) -> Displacement {
        match direction {
            Direction::South => Displacement::new(1, 1),
            Direction::SouthWest => Displacement::new(0, 1),
            Direction::West => Displacement::new(-1, 1),
            Direction::NorthWest => Displacement::new(-1, 0),
            Direction::North => Displacement::new(-1, -1),
            Direction::NorthEast => Displacement::new(0, -1),
            Direction::East => Displacement::new(1, -1),
            Direction::SouthEast => Displacement::new(1, 0),
            Direction::NoDirection => Displacement::new(0, 0),
        }
    }

    pub fn magnitude(self) -> f32 {
        ((self.delta_x * self.delta_x + self.delta_y * self.delta_y) as f32).sqrt()
    }

    pub const fn world_to_screen(self) -> Displacement {
        Displacement::new((self.delta_y - self.delta_x) * 32, (self.delta_y + self.delta_x) * -16)
    }

    pub const fn screen_to_world(self) -> Displacement {
        Displacement::new((2 * self.delta_y + self.delta_x) / -64, (2 * self.delta_y - self.delta_x) / -64)
    }

    pub const fn screen_to_missile(self) -> Displacement {
        let x_numerator = 2 * self.delta_y + self.delta_x;
        let y_numerator = 2 * self.delta_y - self.delta_x;
        let x_offset = if x_numerator >= 0 { 32 } else { -32 };
        let y_offset = if y_numerator >= 0 { 32 } else { -32 };
        Displacement::new((x_numerator + x_offset) / 64, (y_numerator + y_offset) / 64)
    }

    pub const fn screen_to_light(self) -> Displacement {
        Displacement::new((2 * self.delta_y + self.delta_x) / 8, (2 * self.delta_y - self.delta_x) / 8)
    }

    pub fn world_to_normal_screen(self) -> Displacement {
        let rotated = Displacement::new(self.delta_y - self.delta_x, -(self.delta_y + self.delta_x));
        let n = rotated.normalized();
        Displacement::new(n.delta_x, n.delta_y / 2)
    }

    /// 16 bit fixed point normalised displacement (magnitude ~1.0).
    pub fn normalized(self) -> Displacement {
        let magnitude = self.magnitude();
        let mut d = self << 16;
        d /= magnitude;
        d
    }

    pub const fn rotate(self, quadrants: i32) -> Displacement {
        const SINES: [i32; 4] = [0, 1, 0, -1];
        let q = (quadrants % 4 + 4) % 4;
        let sine = SINES[q as usize];
        let cosine = SINES[((q + 1) % 4) as usize];
        Displacement::new(self.delta_x * cosine - self.delta_y * sine, self.delta_x * sine + self.delta_y * cosine)
    }

    pub const fn flip_x(self) -> Displacement {
        Displacement::new(-self.delta_x, self.delta_y)
    }

    pub const fn flip_y(self) -> Displacement {
        Displacement::new(self.delta_x, -self.delta_y)
    }

    pub const fn flip_xy(self) -> Displacement {
        Displacement::new(-self.delta_x, -self.delta_y)
    }

    pub const fn abs(self) -> Displacement {
        Displacement::new(self.delta_x.abs(), self.delta_y.abs())
    }
}

impl From<Direction> for Displacement {
    fn from(d: Direction) -> Self {
        Displacement::from_direction(d)
    }
}

// @port engine/displacement.hpp|devilution::DisplacementOf::operator+=(DisplacementOf<DisplacementDeltaT> displacement) sha=76315021d246
// @port engine/displacement.hpp|devilution::operator+(DisplacementOf<DisplacementDeltaT> a, DisplacementOf<OtherDisplacementDeltaT> b) sha=b1469d56ffa9
impl AddAssign for Displacement {
    fn add_assign(&mut self, o: Displacement) {
        self.delta_x += o.delta_x;
        self.delta_y += o.delta_y;
    }
}
impl Add for Displacement {
    type Output = Displacement;
    fn add(mut self, o: Displacement) -> Displacement {
        self += o;
        self
    }
}

// @port engine/displacement.hpp|devilution::DisplacementOf::operator-=(DisplacementOf<DisplacementDeltaT> displacement) sha=1279af311fde
// @port engine/displacement.hpp|devilution::operator-(DisplacementOf<DisplacementDeltaT> a, DisplacementOf<OtherDisplacementDeltaT> b) sha=7dae3550a84a
impl SubAssign for Displacement {
    fn sub_assign(&mut self, o: Displacement) {
        self.delta_x -= o.delta_x;
        self.delta_y -= o.delta_y;
    }
}
impl Sub for Displacement {
    type Output = Displacement;
    fn sub(mut self, o: Displacement) -> Displacement {
        self -= o;
        self
    }
}

// @port engine/displacement.hpp|devilution::DisplacementOf::operator*=(const int factor) sha=710f8041b5bc
// @port engine/displacement.hpp|devilution::operator*(DisplacementOf<DisplacementDeltaT> a, const int factor) sha=58bcc5cb5533
impl MulAssign<i32> for Displacement {
    fn mul_assign(&mut self, f: i32) {
        self.delta_x *= f;
        self.delta_y *= f;
    }
}
impl Mul<i32> for Displacement {
    type Output = Displacement;
    fn mul(mut self, f: i32) -> Displacement {
        self *= f;
        self
    }
}

// @port engine/displacement.hpp|devilution::DisplacementOf::operator*=(const float factor) sha=ec7482756247
// @port engine/displacement.hpp|devilution::operator*(DisplacementOf<DisplacementDeltaT> a, const float factor) sha=e6a46ff34da3
impl MulAssign<f32> for Displacement {
    fn mul_assign(&mut self, f: f32) {
        self.delta_x = (self.delta_x as f32 * f) as i32;
        self.delta_y = (self.delta_y as f32 * f) as i32;
    }
}
impl Mul<f32> for Displacement {
    type Output = Displacement;
    fn mul(mut self, f: f32) -> Displacement {
        self *= f;
        self
    }
}

// @port engine/displacement.hpp|devilution::DisplacementOf::operator*=(const DisplacementOf<DeltaU> factor) sha=474a8a3f1666
// @port engine/displacement.hpp|devilution::operator*(DisplacementOf<DisplacementDeltaT> a, const DisplacementOf<DisplacementDeltaU> factor) sha=9ae4a8e6198d
impl MulAssign<Displacement> for Displacement {
    fn mul_assign(&mut self, f: Displacement) {
        self.delta_x *= f.delta_x;
        self.delta_y *= f.delta_y;
    }
}
impl Mul<Displacement> for Displacement {
    type Output = Displacement;
    fn mul(mut self, f: Displacement) -> Displacement {
        self *= f;
        self
    }
}

// @port engine/displacement.hpp|devilution::DisplacementOf::operator/=(const int factor) sha=079dea1ce04f
// @port engine/displacement.hpp|devilution::operator/(DisplacementOf<DisplacementDeltaT> a, const int factor) sha=fa7098591478
impl DivAssign<i32> for Displacement {
    fn div_assign(&mut self, f: i32) {
        self.delta_x /= f;
        self.delta_y /= f;
    }
}
impl Div<i32> for Displacement {
    type Output = Displacement;
    fn div(mut self, f: i32) -> Displacement {
        self /= f;
        self
    }
}

// @port engine/displacement.hpp|devilution::DisplacementOf::operator/=(const float factor) sha=7d06d25c723f
// @port engine/displacement.hpp|devilution::operator/(DisplacementOf<DisplacementDeltaT> a, const float factor) sha=e038be650a34
impl DivAssign<f32> for Displacement {
    fn div_assign(&mut self, f: f32) {
        self.delta_x = (self.delta_x as f32 / f) as i32;
        self.delta_y = (self.delta_y as f32 / f) as i32;
    }
}
impl Div<f32> for Displacement {
    type Output = Displacement;
    fn div(mut self, f: f32) -> Displacement {
        self /= f;
        self
    }
}

// @port engine/displacement.hpp|devilution::operator-(DisplacementOf<DisplacementDeltaT> a) sha=aeed4cd3f355
impl Neg for Displacement {
    type Output = Displacement;
    fn neg(self) -> Displacement {
        Displacement::new(-self.delta_x, -self.delta_y)
    }
}

// @port engine/displacement.hpp|devilution::operator<<(DisplacementOf<DisplacementDeltaT> a, unsigned factor) sha=62cf1306d061
impl Shl<u32> for Displacement {
    type Output = Displacement;
    fn shl(self, f: u32) -> Displacement {
        Displacement::new(self.delta_x << f, self.delta_y << f)
    }
}

// @port engine/displacement.hpp|devilution::operator>>(DisplacementOf<DisplacementDeltaT> a, unsigned factor) sha=b7013b536e83
impl Shr<u32> for Displacement {
    type Output = Displacement;
    fn shr(self, f: u32) -> Displacement {
        Displacement::new(self.delta_x >> f, self.delta_y >> f)
    }
}

/// `Point` (`PointOf<int>`; `WorldTilePosition` values are stored as `Point` too)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

// @port engine/point.hpp|devilution::PointOf::PointOf(PointOf<PointCoordT> other) sha=dd71085a2249
// @port engine/point.hpp|devilution::PointOf::PointOf(CoordT x, CoordT y) sha=a0ae070640cf
// @port engine/point.hpp|devilution::PointOf::operator==(const PointOf<PointCoordT> &other) sha=b70593b493ca
// @port engine/point.hpp|devilution::PointOf::operator!=(const PointOf<PointCoordT> &other) sha=ab008535e09b
// @port engine/point.hpp|devilution::PointOf::ApproxDistance(PointOf<PointCoordT> other) sha=f777b4be4799
// @port engine/point.hpp|devilution::PointOf::ExactDistance(PointOf<PointCoordT> other) sha=de1a020afb75
// @port engine/point.hpp|devilution::PointOf::ManhattanDistance(PointOf<PointCoordT> other) sha=9253a10ac9bb
// @port engine/point.hpp|devilution::PointOf::WalkingDistance(PointOf<PointCoordT> other) sha=25bdfd678dbf
// @port engine/point.hpp|devilution::PointOf::megaToWorld() sha=00295e45ac2d
// @port engine/point.hpp|devilution::PointOf::worldToMega() sha=240da08b359c
// @port engine/point.hpp|devilution::abs(PointOf<PointCoordT> a) sha=fe2bb244f1af
impl Point {
    pub const fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    pub fn approx_distance(self, other: Point) -> i32 {
        let offset = (self - other).abs();
        let (min, max) = if offset.delta_y < offset.delta_x { (offset.delta_y, offset.delta_x) } else { (offset.delta_x, offset.delta_y) };
        let mut approx = max * 1007 + min * 441;
        if max < min * 16 {
            approx -= max * 40;
        }
        (approx + 512) / 1024
    }

    pub fn exact_distance(self, other: Point) -> i32 {
        let v = self - other;
        ((v.delta_x as i64 * v.delta_x as i64 + v.delta_y as i64 * v.delta_y as i64) as f64).sqrt() as i32
    }

    pub const fn manhattan_distance(self, other: Point) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    pub const fn walking_distance(self, other: Point) -> i32 {
        let a = (self.x - other.x).abs();
        let b = (self.y - other.y).abs();
        if a > b { a } else { b }
    }

    pub const fn mega_to_world(self) -> Point {
        Point::new(16 + 2 * self.x, 16 + 2 * self.y)
    }

    pub const fn world_to_mega(self) -> Point {
        Point::new((self.x - 16) / 2, (self.y - 16) / 2)
    }

    pub const fn abs(self) -> Point {
        Point::new(self.x.abs(), self.y.abs())
    }
}

// @port engine/point.hpp|devilution::PointOf::operator+=(const DisplacementOf<DisplacementDeltaT> &displacement) sha=80abd8e618b6
// @port engine/point.hpp|devilution::operator+(PointOf<PointCoordT> a, DisplacementOf<DisplacementDeltaT> displacement) sha=1acf6b891aa0
impl AddAssign<Displacement> for Point {
    fn add_assign(&mut self, d: Displacement) {
        self.x += d.delta_x;
        self.y += d.delta_y;
    }
}
impl Add<Displacement> for Point {
    type Output = Point;
    fn add(mut self, d: Displacement) -> Point {
        self += d;
        self
    }
}

// @port engine/point.hpp|devilution::PointOf::operator+=(Direction direction) sha=bb1fbc6280f6
// @port engine/point.hpp|devilution::operator+(PointOf<PointCoordT> a, Direction direction) sha=b48953a93698
impl AddAssign<Direction> for Point {
    fn add_assign(&mut self, d: Direction) {
        *self += Displacement::from_direction(d);
    }
}
impl Add<Direction> for Point {
    type Output = Point;
    fn add(mut self, d: Direction) -> Point {
        self += d;
        self
    }
}

// @port engine/point.hpp|devilution::PointOf::operator-=(const DisplacementOf<DisplacementDeltaT> &displacement) sha=27c5e6bee195
// @port engine/point.hpp|devilution::operator-(PointOf<PointCoordT> a, DisplacementOf<DisplacementDeltaT> displacement) sha=016333a57ead
impl SubAssign<Displacement> for Point {
    fn sub_assign(&mut self, d: Displacement) {
        self.x -= d.delta_x;
        self.y -= d.delta_y;
    }
}
impl Sub<Displacement> for Point {
    type Output = Point;
    fn sub(mut self, d: Displacement) -> Point {
        self -= d;
        self
    }
}

// @port engine/point.hpp|devilution::operator-(PointOf<PointCoordT> a, PointOf<OtherPointCoordT> b) sha=508a1ecf4dfd
impl Sub<Point> for Point {
    type Output = Displacement;
    fn sub(self, b: Point) -> Displacement {
        Displacement::new(self.x - b.x, self.y - b.y)
    }
}

// @port engine/point.hpp|devilution::PointOf::operator*=(const float factor) sha=fdacf9ea79a4
// @port engine/point.hpp|devilution::operator*(PointOf<PointCoordT> a, const float factor) sha=fc195c899b39
impl MulAssign<f32> for Point {
    fn mul_assign(&mut self, f: f32) {
        self.x = (self.x as f32 * f) as i32;
        self.y = (self.y as f32 * f) as i32;
    }
}
impl Mul<f32> for Point {
    type Output = Point;
    fn mul(mut self, f: f32) -> Point {
        self *= f;
        self
    }
}

// @port engine/point.hpp|devilution::PointOf::operator*=(const int factor) sha=cb92f397489f
// @port engine/point.hpp|devilution::operator*(PointOf<PointCoordT> a, const int factor) sha=3cb9869d717e
impl MulAssign<i32> for Point {
    fn mul_assign(&mut self, f: i32) {
        self.x *= f;
        self.y *= f;
    }
}
impl Mul<i32> for Point {
    type Output = Point;
    fn mul(mut self, f: i32) -> Point {
        self *= f;
        self
    }
}

// @port engine/point.hpp|devilution::PointOf::operator-() sha=7a2cd2728c64
impl Neg for Point {
    type Output = Point;
    fn neg(self) -> Point {
        Point::new(-self.x, -self.y)
    }
}

/// `Size` (`SizeOf<int>`)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

// @port engine/size.hpp|devilution::SizeOf::SizeOf(SizeT width, SizeT height) sha=1d07d9bba240
// @port engine/size.hpp|devilution::SizeOf::SizeOf(SizeT size) sha=937f48530068
// @port engine/size.hpp|devilution::SizeOf::operator==(const SizeOf<SizeT> &other) sha=134a9aba4747
// @port engine/size.hpp|devilution::SizeOf::operator!=(const SizeOf<SizeT> &other) sha=431ab8c25764
// @port engine/size.hpp|devilution::SizeOf::operator+=(SizeT factor) sha=87bffdc07045
// @port engine/size.hpp|devilution::SizeOf::operator-=(SizeT factor) sha=331ab5381752
// @port engine/size.hpp|devilution::SizeOf::operator*=(SizeT factor) sha=9a7af38b45ab
// @port engine/size.hpp|devilution::SizeOf::operator*=(float factor) sha=dc082fb2463e
// @port engine/size.hpp|devilution::SizeOf::operator/=(SizeT factor) sha=1328b6ef69b8
// @port engine/size.hpp|devilution::SizeOf::operator+(SizeOf<SizeT> a, SizeT factor) sha=c854846e8a4e
// @port engine/size.hpp|devilution::SizeOf::operator-(SizeOf<SizeT> a, SizeT factor) sha=7a04787667f3
// @port engine/size.hpp|devilution::SizeOf::operator*(SizeOf<SizeT> a, SizeT factor) sha=a2c99f5a09fa
// @port engine/size.hpp|devilution::SizeOf::operator/(SizeOf<SizeT> a, SizeT factor) sha=7a2c26fe86a3
impl Size {
    pub const fn new(width: i32, height: i32) -> Size {
        Size { width, height }
    }

    pub const fn splat(size: i32) -> Size {
        Size { width: size, height: size }
    }
}

impl Add<i32> for Size {
    type Output = Size;
    fn add(self, f: i32) -> Size {
        Size::new(self.width + f, self.height + f)
    }
}
impl Sub<i32> for Size {
    type Output = Size;
    fn sub(self, f: i32) -> Size {
        Size::new(self.width - f, self.height - f)
    }
}
impl Mul<i32> for Size {
    type Output = Size;
    fn mul(self, f: i32) -> Size {
        Size::new(self.width * f, self.height * f)
    }
}
impl Mul<f32> for Size {
    type Output = Size;
    fn mul(self, f: f32) -> Size {
        Size::new((self.width as f32 * f) as i32, (self.height as f32 * f) as i32)
    }
}
impl Div<i32> for Size {
    type Output = Size;
    fn div(self, f: i32) -> Size {
        Size::new(self.width / f, self.height / f)
    }
}

/// `Rectangle` (`RectangleOf<int, int>`)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Rectangle {
    pub position: Point,
    pub size: Size,
}

// @port engine/rectangle.hpp|devilution::RectangleOf::RectangleOf(PointOf<CoordT> position, SizeOf<SizeT> size) sha=8eeed01e0497
// @port engine/rectangle.hpp|devilution::RectangleOf::RectangleOf(PointOf<CoordT> center, SizeT radius) sha=05994d361328
// @port engine/rectangle.hpp|devilution::RectangleOf::contains(PointOf<PointCoordT> point) sha=004066ea0158
// @port engine/rectangle.hpp|devilution::RectangleOf::contains(T x, T y) sha=6db56d92db58
// @port engine/rectangle.hpp|devilution::RectangleOf::Center() sha=7f9ebdaa8b3e
// @port engine/rectangle.hpp|devilution::RectangleOf::inset(DisplacementOf<SizeT> factor) sha=67fc783ef311
impl Rectangle {
    pub const fn new(position: Point, size: Size) -> Rectangle {
        Rectangle { position, size }
    }

    pub const fn from_center(center: Point, radius: i32) -> Rectangle {
        Rectangle { position: Point::new(center.x - radius, center.y - radius), size: Size::splat(2 * radius + 1) }
    }

    pub const fn contains(&self, p: Point) -> bool {
        self.contains_xy(p.x, p.y)
    }

    pub const fn contains_xy(&self, x: i32, y: i32) -> bool {
        x >= self.position.x && x < self.position.x + self.size.width && y >= self.position.y && y < self.position.y + self.size.height
    }

    pub const fn center(&self) -> Point {
        Point::new(self.position.x + self.size.width / 2, self.position.y + self.size.height / 2)
    }

    pub const fn inset(&self, factor: Displacement) -> Rectangle {
        Rectangle {
            position: Point::new(self.position.x + factor.delta_x, self.position.y + factor.delta_y),
            size: Size::new(self.size.width - factor.delta_x * 2, self.size.height - factor.delta_y * 2),
        }
    }
}

/// `Circle`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Circle {
    pub position: Point,
    pub radius: i32,
}

impl Circle {
    /// Original: `Circle::contains` (engine/circle.hpp).
    // @port engine/circle.hpp|devilution::Circle::contains(Point point) sha=4f0058bff999
    pub const fn contains(&self, point: Point) -> bool {
        let x = point.x - self.position.x;
        let y = point.y - self.position.y;
        x * x + y * y < self.radius * self.radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distances_match_the_original_formulas() {
        let a = Point::new(10, 10);
        assert_eq!(a.approx_distance(Point::new(13, 14)), 5);
        assert_eq!(a.walking_distance(Point::new(13, 14)), 4);
        assert_eq!(a.manhattan_distance(Point::new(13, 14)), 7);
        assert_eq!(a.exact_distance(Point::new(13, 14)), 5);
    }

    #[test]
    fn directions_and_screen_transforms() {
        assert_eq!(left(Direction::South), Direction::SouthEast);
        assert_eq!(right(Direction::SouthEast), Direction::South);
        assert_eq!(opposite(Direction::North), Direction::South);
        assert_eq!(Displacement::new(1, 0).world_to_screen(), Displacement::new(-32, -16));
        assert_eq!(Displacement::new(-32, -16).screen_to_world(), Displacement::new(1, 0));
    }
}

/// `PointsInRectangleRange`: row-major ({0,0}, {1,0}, ..., {0,1}, ...).
pub fn points_in_rectangle(region: Rectangle) -> impl DoubleEndedIterator<Item = Point> + Clone {
    let (o, w, h) = (region.position, region.size.width.max(0), region.size.height.max(0));
    (0..h).flat_map(move |y| (0..w).map(move |x| Point::new(o.x + x, o.y + y)))
}

/// `PointsInRectangleRangeColMajor`: column-major ({0,0}, {0,1}, ..., {1,0}, ...).
pub fn points_in_rectangle_col_major(region: Rectangle) -> impl DoubleEndedIterator<Item = Point> + Clone {
    let (o, w, h) = (region.position, region.size.width.max(0), region.size.height.max(0));
    (0..w).flat_map(move |x| (0..h).map(move |y| Point::new(o.x + x, o.y + y)))
}
