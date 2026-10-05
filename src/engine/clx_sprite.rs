//! `Source/engine/clx_sprite.hpp`: CLX sprite lists and sheets.
//!
//! The C++ views hold raw pointers into buffers owned by `Owned*` objects. Here the buffers
//! are reference-counted (`Rc<[u8]>`) and the views carry an offset, so a view can be kept or
//! passed around without borrowing the game context. Byte layout and accessors are unchanged:
//! list = u32 count, u32 offsets[count + 1]; sprite = 10-byte header (u16 pixel data offset,
//! u16 width, u16 height, 4 unused) + CLX pixel commands; sheet = u32 list offsets.

use std::rc::Rc;

fn le16(d: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([d[o], d[o + 1]])
}

fn le32(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}

/// `ClxSprite`
#[derive(Clone, Debug)]
pub struct ClxSprite {
    data: Rc<[u8]>,
    start: usize,
    size: usize,
}

impl ClxSprite {
    const HEADER_SIZE: usize = 10;

    // @port engine/clx_sprite.hpp|devilution::ClxSprite::width() sha=9971524140fa
    pub fn width(&self) -> u16 {
        le16(&self.data, self.start + 2)
    }

    // @port engine/clx_sprite.hpp|devilution::ClxSprite::height() sha=c73fc21a7668
    pub fn height(&self) -> u16 {
        le16(&self.data, self.start + 4)
    }

    /// `pixelData()`
    // @port engine/clx_sprite.hpp|devilution::ClxSprite::pixelData() sha=29ea235b47b0
    pub fn pixel_data(&self) -> &[u8] {
        &self.data[self.start + Self::HEADER_SIZE..self.start + self.size]
    }

    /// `pixelDataSize()`
    // @port engine/clx_sprite.hpp|devilution::ClxSprite::pixelDataSize() sha=df87cb98a532
    pub fn pixel_data_size(&self) -> usize {
        self.size - Self::HEADER_SIZE
    }
}

impl PartialEq for ClxSprite {
    /// Identity, as in the original (pointer comparison).
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data) && self.start == other.start
    }
}

/// `ClxSpriteList` (also `OwnedClxSpriteList`: the owner is the shared buffer)
#[derive(Clone, Debug)]
pub struct ClxSpriteList {
    data: Rc<[u8]>,
    start: usize,
}

pub type OwnedClxSpriteList = ClxSpriteList;

impl ClxSpriteList {
    /// `OwnedClxSpriteList(std::unique_ptr<uint8_t[]> &&data)`
    pub fn from_vec(data: Vec<u8>) -> ClxSpriteList {
        ClxSpriteList { data: data.into(), start: 0 }
    }

    fn at(data: Rc<[u8]>, start: usize) -> ClxSpriteList {
        ClxSpriteList { data, start }
    }

    /// `numSprites()`
    // @port engine/clx_sprite.hpp|devilution::ClxSpriteList::numSprites() sha=721c348da60a
    pub fn num_sprites(&self) -> u32 {
        le32(&self.data, self.start)
    }

    /// `spriteOffset(i)`
    // @port engine/clx_sprite.hpp|devilution::ClxSpriteList::spriteOffset(size_t spriteIndex) sha=39ad03afd3a5
    pub fn sprite_offset(&self, i: usize) -> u32 {
        le32(&self.data, self.start + 4 + i * 4)
    }

    /// `operator[]`
    pub fn get(&self, i: usize) -> ClxSprite {
        assert!(i < self.num_sprites() as usize);
        let begin = self.sprite_offset(i) as usize;
        let end = self.sprite_offset(i + 1) as usize;
        ClxSprite { data: self.data.clone(), start: self.start + begin, size: end - begin }
    }

    /// `nextSpriteSheetOffsetOrFileSize()`
    // @port engine/clx_sprite.hpp|devilution::ClxSpriteList::nextSpriteSheetOffsetOrFileSize() sha=1fe677ac40f1
    pub fn next_sprite_sheet_offset_or_file_size(&self) -> u32 {
        le32(&self.data, self.start + 4 + self.num_sprites() as usize * 4)
    }

    /// `clone()`: an independent copy of the list's bytes.
    pub fn deep_clone(&self) -> ClxSpriteList {
        let size = self.next_sprite_sheet_offset_or_file_size() as usize;
        ClxSpriteList::from_vec(self.data[self.start..self.start + size].to_vec())
    }

    pub fn iter(&self) -> impl Iterator<Item = ClxSprite> + '_ {
        (0..self.num_sprites() as usize).map(move |i| self.get(i))
    }

    /// `data()`
    pub fn data(&self) -> &[u8] {
        &self.data[self.start..]
    }

    /// Mutable bytes of the whole underlying buffer (copied first if shared), for
    /// `ClxApplyTrans`, which the original applies in place right after loading.
    pub fn data_mut(&mut self) -> &mut [u8] {
        if Rc::get_mut(&mut self.data).is_none() {
            self.data = Rc::from(self.data.to_vec());
        }
        &mut Rc::get_mut(&mut self.data).unwrap()[self.start..]
    }
}

/// `ClxSpriteSheet` / `OwnedClxSpriteSheet`
#[derive(Clone, Debug)]
pub struct ClxSpriteSheet {
    data: Rc<[u8]>,
    num_lists: u16,
}

pub type OwnedClxSpriteSheet = ClxSpriteSheet;

impl ClxSpriteSheet {
    pub fn from_vec(data: Vec<u8>, num_lists: u16) -> ClxSpriteSheet {
        assert!(num_lists > 0);
        ClxSpriteSheet { data: data.into(), num_lists }
    }

    /// `numLists()`
    // @port engine/clx_sprite.hpp|devilution::ClxSpriteSheet::numLists() sha=11323540d596
    pub fn num_lists(&self) -> u16 {
        self.num_lists
    }

    /// `sheetOffset(i)`
    // @port engine/clx_sprite.hpp|devilution::ClxSpriteSheet::sheetOffset(size_t sheetIndex) sha=0a2174759a2b
    pub fn sheet_offset(&self, i: usize) -> u32 {
        assert!(i < self.num_lists as usize);
        le32(&self.data, 4 * i)
    }

    /// `operator[]`
    pub fn get(&self, i: usize) -> ClxSpriteList {
        ClxSpriteList::at(self.data.clone(), self.sheet_offset(i) as usize)
    }

    pub fn iter(&self) -> impl Iterator<Item = ClxSpriteList> + '_ {
        (0..self.num_lists as usize).map(move |i| self.get(i))
    }

    /// See `ClxSpriteList::data_mut`.
    pub fn data_mut(&mut self) -> &mut [u8] {
        if Rc::get_mut(&mut self.data).is_none() {
            self.data = Rc::from(self.data.to_vec());
        }
        Rc::get_mut(&mut self.data).unwrap()
    }
}

/// `GetNumListsFromClxListOrSheetBuffer` (engine/clx_sprite.hpp)
// @port engine/clx_sprite.hpp|devilution::GetNumListsFromClxListOrSheetBuffer(const uint8_t *data, size_t size) sha=6bc6dc37a384
pub fn get_num_lists_from_clx_list_or_sheet_buffer(data: &[u8], size: usize) -> u16 {
    let maybe_num_frames = le32(data, 0) as usize;
    if le32(data, maybe_num_frames * 4 + 4) as usize != size {
        return (maybe_num_frames / 4) as u16;
    }
    0
}

/// `ClxSpriteListOrSheet` / `OwnedClxSpriteListOrSheet`
#[derive(Clone, Debug)]
pub enum ClxSpriteListOrSheet {
    List(ClxSpriteList),
    Sheet(ClxSpriteSheet),
}

impl ClxSpriteListOrSheet {
    /// `OwnedClxSpriteListOrSheet::FromBuffer`
    // @port engine/clx_sprite.hpp|devilution::OwnedClxSpriteListOrSheet::FromBuffer(std::unique_ptr<uint8_t[]> &&data, size_t size) sha=c5cc4da94898
    pub fn from_buffer(data: Vec<u8>) -> ClxSpriteListOrSheet {
        let n = get_num_lists_from_clx_list_or_sheet_buffer(&data, data.len());
        if n == 0 { ClxSpriteListOrSheet::List(ClxSpriteList::from_vec(data)) } else { ClxSpriteListOrSheet::Sheet(ClxSpriteSheet::from_vec(data, n)) }
    }

    pub fn is_sheet(&self) -> bool {
        matches!(self, ClxSpriteListOrSheet::Sheet(_))
    }

    pub fn list(&self) -> &ClxSpriteList {
        match self {
            ClxSpriteListOrSheet::List(l) => l,
            ClxSpriteListOrSheet::Sheet(_) => panic!("not a list"),
        }
    }

    pub fn sheet(&self) -> &ClxSpriteSheet {
        match self {
            ClxSpriteListOrSheet::Sheet(s) => s,
            ClxSpriteListOrSheet::List(_) => panic!("not a sheet"),
        }
    }
}
