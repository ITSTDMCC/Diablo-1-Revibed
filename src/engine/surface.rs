//! `Source/engine/surface.hpp` / `surface.cpp`: 8-bit surfaces.
//!
//! As in the original, a `Surface` is a lightweight view (pointer + pitch + region) into pixel
//! memory owned elsewhere (`OwnedSurface`, the back buffer in `DxState`). Views are `Copy` and
//! may overlap, like the C++ ones. The owner must outlive every view made from it; the game
//! creates views per draw call from buffers that live in the game context.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Surface {
    pixels: *mut u8,
    pitch: i32,
    pub region: Rect,
}

impl Default for Surface {
    fn default() -> Self {
        Surface { pixels: std::ptr::null_mut(), pitch: 0, region: Rect::default() }
    }
}

impl Surface {
    /// A view of the whole of `owner`.
    pub fn of(owner: &mut OwnedSurface) -> Surface {
        Surface { pixels: owner.pixels.as_mut_ptr(), pitch: owner.pitch, region: Rect::new(0, 0, owner.w, owner.h) }
    }

    pub fn is_null(&self) -> bool {
        self.pixels.is_null()
    }

    pub fn w(&self) -> i32 {
        self.region.w
    }

    pub fn h(&self) -> i32 {
        self.region.h
    }

    pub fn pitch(&self) -> i32 {
        self.pitch
    }

    /// `at(x, y)`: pointer to the pixel (region-relative).
    pub fn at(&self, x: i32, y: i32) -> *mut u8 {
        // SAFETY: callers stay within the owner's buffer, as in the C++ code.
        unsafe { self.pixels.offset((self.region.x + x + self.pitch * (self.region.y + y)) as isize) }
    }

    pub fn get(&self, x: i32, y: i32) -> u8 {
        // SAFETY: see `at`.
        unsafe { *self.at(x, y) }
    }

    pub fn put(&self, x: i32, y: i32, v: u8) {
        // SAFETY: see `at`.
        unsafe { *self.at(x, y) = v }
    }

    /// The row `y` of the region as a slice.
    pub fn row(&self, y: i32) -> &mut [u8] {
        // SAFETY: the region lies within the owner's buffer.
        unsafe { std::slice::from_raw_parts_mut(self.at(0, y), self.region.w as usize) }
    }

    /// `SetPixel`: sets the pixel if it is in bounds.
    pub fn set_pixel(&self, x: i32, y: i32, col: u8) {
        if self.in_bounds(x, y) {
            self.put(x, y, col);
        }
    }

    /// `InBounds`
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.region.w && y < self.region.h
    }

    /// `subregion`
    pub fn subregion(&self, x: i32, y: i32, w: i32, h: i32) -> Surface {
        Surface { pixels: self.pixels, pitch: self.pitch, region: Rect::new(self.region.x + x, self.region.y + y, w, h) }
    }

    /// `subregionY`
    pub fn subregion_y(&self, y: i32, h: i32) -> Surface {
        let mut r = self.region;
        r.y += y;
        r.h = h;
        Surface { pixels: self.pixels, pitch: self.pitch, region: r }
    }

    /// `Clip`
    pub fn clip(&self, src_rect: &mut Rect, target: &mut (i32, i32)) {
        if target.0 < 0 {
            src_rect.x -= target.0;
            src_rect.w += target.0;
            target.0 = 0;
        }
        if target.1 < 0 {
            src_rect.y -= target.1;
            src_rect.h += target.1;
            target.1 = 0;
        }
        if target.0 + src_rect.w > self.region.w {
            src_rect.w = self.region.w - target.0;
        }
        if target.1 + src_rect.h > self.region.h {
            src_rect.h = self.region.h - target.1;
        }
    }

    /// Original: `Surface::BlitFrom` (engine/surface.cpp).
    // @port engine/surface.cpp|devilution::Surface::BlitFrom(const Surface &src, SDL_Rect srcRect, Point targetPosition) sha=407c7adb6ca8
    pub fn blit_from(&self, src: &Surface, mut src_rect: Rect, mut target: (i32, i32)) {
        self.clip(&mut src_rect, &mut target);
        if src_rect.w <= 0 || src_rect.h <= 0 {
            return;
        }
        for y in 0..src_rect.h {
            // SAFETY: clipped to both regions; std::ptr::copy handles overlap like memmove.
            unsafe { std::ptr::copy(src.at(src_rect.x, src_rect.y + y), self.at(target.0, target.1 + y), src_rect.w as usize) };
        }
    }

    /// Original: `Surface::BlitFromSkipColorIndexZero` (engine/surface.cpp).
    // @port engine/surface.cpp|devilution::Surface::BlitFromSkipColorIndexZero(const Surface &src, SDL_Rect srcRect, Point targetPosition) sha=b83dafdc0834
    pub fn blit_from_skip_color_index_zero(&self, src: &Surface, mut src_rect: Rect, mut target: (i32, i32)) {
        self.clip(&mut src_rect, &mut target);
        for y in 0..src_rect.h {
            for x in 0..src_rect.w {
                let v = src.get(src_rect.x + x, src_rect.y + y);
                if v != 0 {
                    self.put(target.0 + x, target.1 + y, v);
                }
            }
        }
    }
}

/// `OwnedSurface`: an 8-bit surface that owns its pixels.
#[derive(Clone, Debug, Default)]
pub struct OwnedSurface {
    pub w: i32,
    pub h: i32,
    pub pitch: i32,
    pub pixels: Vec<u8>,
}

impl OwnedSurface {
    /// `OwnedSurface(width, height)` (SDL_CreateRGBSurfaceWithFormat, INDEX8, zero-filled).
    pub fn new(w: i32, h: i32) -> OwnedSurface {
        OwnedSurface { w, h, pitch: w, pixels: vec![0; (w.max(0) * h.max(0)) as usize] }
    }

    pub fn view(&mut self) -> Surface {
        Surface::of(self)
    }
}
