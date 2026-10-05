//! `Source/DiabloUI/ui_item.h`: UI elements.
//!
//! The C++ class hierarchy (UiItemBase and its subclasses, selected by `UiType`) becomes one
//! struct with a `UiKind` enum. Items are shared (`UiItemRef`) because the original keeps raw
//! pointers to them in the global item list (`gUiItems`) while the dialog owns them.

use std::cell::RefCell;
use std::rc::Rc;

use crate::ctx::Ctx;
use crate::engine::clx_sprite::{ClxSprite, ClxSpriteList};
use crate::engine::render::text_render::{DrawStringFormatArg, UiFlags};
use crate::engine::surface::Rect;

/// `UiType`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiType {
    Text,
    ArtText,
    ArtTextButton,
    ImageClx,
    ImageAnimatedClx,
    Button,
    List,
    Scrollbar,
    Edit,
}

pub type Callback = fn(&mut Ctx);

/// Text that may be a fixed string or one the dialog updates between frames
/// (`UiArtText(const char **ptext, ...)`).
#[derive(Clone, Debug)]
pub enum UiText {
    Fixed(String),
    Dynamic(Rc<RefCell<String>>),
}

impl UiText {
    pub fn get(&self) -> String {
        match self {
            UiText::Fixed(s) => s.clone(),
            UiText::Dynamic(s) => s.borrow().clone(),
        }
    }
}

/// `UiListItem`
#[derive(Clone, Debug)]
pub struct UiListItem {
    pub m_text: String,
    pub args: Vec<DrawStringFormatArg>,
    pub m_value: i32,
    pub ui_flags: UiFlags,
}

impl UiListItem {
    /// `UiListItem(string_view text = "", int value = 0, UiFlags uiFlags = UiFlags::None)`
    pub fn new(text: &str, value: i32, ui_flags: UiFlags) -> UiListItemRef {
        Rc::new(RefCell::new(UiListItem { m_text: text.to_string(), args: Vec::new(), m_value: value, ui_flags }))
    }

    /// `UiListItem(string_view text, std::vector<DrawStringFormatArg> &args, int value, UiFlags)`
    pub fn with_args(text: &str, args: Vec<DrawStringFormatArg>, value: i32, ui_flags: UiFlags) -> UiListItemRef {
        Rc::new(RefCell::new(UiListItem { m_text: text.to_string(), args, m_value: value, ui_flags }))
    }
}

pub type UiListItemRef = Rc<RefCell<UiListItem>>;

/// `UiList`
#[derive(Clone, Debug)]
pub struct UiList {
    pub viewport_size: usize,
    pub m_x: i32,
    pub m_y: i32,
    pub m_width: i32,
    pub m_height: i32,
    pub m_vec_items: Vec<UiListItemRef>,
    spacing: i32,
    pressed_item_index: usize,
}

impl UiList {
    /// `itemRect`
    // @port DiabloUI/ui_item.h|devilution::UiList::itemRect(int i) sha=da617a1e082a
    pub fn item_rect(&self, i: i32) -> Rect {
        Rect::new(self.m_x, self.m_y + self.m_height * i, self.m_width, self.m_height)
    }

    /// `indexAt`
    // @port DiabloUI/ui_item.h|devilution::UiList::indexAt(Sint16 y) sha=8ef7b12e3bb3
    pub fn index_at(&self, list_rect: Rect, y: i32) -> usize {
        debug_assert!(y >= list_rect.y);
        let index = ((y - list_rect.y) / self.m_height) as usize;
        debug_assert!(index < self.m_vec_items.len());
        index
    }

    /// `GetItem`
    // @port DiabloUI/ui_item.h|devilution::UiList::GetItem(std::size_t i) sha=a38777be6818
    pub fn get_item(&self, i: usize) -> UiListItemRef {
        self.m_vec_items[i].clone()
    }

    pub fn get_spacing(&self) -> i32 {
        self.spacing
    }

    pub fn is_pressed(&self, index: usize) -> bool {
        self.pressed_item_index == index
    }

    pub fn press(&mut self, index: usize) {
        self.pressed_item_index = index;
    }

    pub fn release(&mut self) {
        self.pressed_item_index = usize::MAX;
    }
}

/// The subclass-specific data.
#[derive(Clone, Debug)]
pub enum UiKind {
    /// `UiText`
    Text { text: String },
    /// `UiArtText`
    ArtText { text: UiText, spacing: i32, line_height: i32 },
    /// `UiArtTextButton`
    ArtTextButton { text: String, action: Callback },
    /// `UiImageClx`
    ImageClx { sprite: ClxSprite },
    /// `UiImageAnimatedClx`
    ImageAnimatedClx { list: ClxSpriteList },
    /// `UiButton`
    Button { text: String, action: Callback, pressed: bool },
    /// `UiList`
    List(UiList),
    /// `UiScrollbar`
    Scrollbar { bg: ClxSprite, thumb: ClxSprite, arrow: ClxSpriteList },
    /// `UiEdit`: `m_value` is the dialog's text buffer.
    Edit { hint: String, value: Rc<RefCell<String>>, max_length: usize, allow_empty: bool },
}

/// `UiItemBase`
#[derive(Clone, Debug)]
pub struct UiItem {
    pub kind: UiKind,
    pub m_rect: Rect,
    ui_flags: UiFlags,
}

pub type UiItemRef = Rc<RefCell<UiItem>>;

impl UiItem {
    fn new(kind: UiKind, rect: Rect, flags: UiFlags) -> UiItemRef {
        Rc::new(RefCell::new(UiItem { kind, m_rect: rect, ui_flags: flags }))
    }

    /// `UiText(string_view, SDL_Rect, UiFlags = ColorDialogWhite)`
    pub fn text(text: &str, rect: Rect, flags: UiFlags) -> UiItemRef {
        Self::new(UiKind::Text { text: text.to_string() }, rect, flags)
    }

    /// `UiArtText(const char *text, ...)`: defaults spacing 1, line height -1.
    pub fn art_text(text: &str, rect: Rect, flags: UiFlags, spacing: i32, line_height: i32) -> UiItemRef {
        Self::new(UiKind::ArtText { text: UiText::Fixed(text.to_string()), spacing, line_height }, rect, flags)
    }

    /// `UiArtText(const char **ptext, ...)`
    pub fn art_text_dynamic(text: Rc<RefCell<String>>, rect: Rect, flags: UiFlags, spacing: i32, line_height: i32) -> UiItemRef {
        Self::new(UiKind::ArtText { text: UiText::Dynamic(text), spacing, line_height }, rect, flags)
    }

    /// `UiArtTextButton`
    pub fn art_text_button(text: &str, action: Callback, rect: Rect, flags: UiFlags) -> UiItemRef {
        Self::new(UiKind::ArtTextButton { text: text.to_string(), action }, rect, flags)
    }

    /// `UiImageClx`
    pub fn image_clx(sprite: ClxSprite, rect: Rect, flags: UiFlags) -> UiItemRef {
        Self::new(UiKind::ImageClx { sprite }, rect, flags)
    }

    /// `UiImageAnimatedClx`
    pub fn image_animated_clx(list: ClxSpriteList, rect: Rect, flags: UiFlags) -> UiItemRef {
        Self::new(UiKind::ImageAnimatedClx { list }, rect, flags)
    }

    /// `UiButton`
    pub fn button(text: &str, action: Callback, rect: Rect, flags: UiFlags) -> UiItemRef {
        Self::new(UiKind::Button { text: text.to_string(), action, pressed: false }, rect, flags)
    }

    /// `UiList(const vUiListItem &, size_t viewportMaxSize, x, y, item_width, item_height, UiFlags, int spacing = 1)`
    #[allow(clippy::too_many_arguments)]
    pub fn list(items: &[UiListItemRef], viewport_max_size: usize, x: i32, y: i32, item_width: i32, item_height: i32, flags: UiFlags, spacing: i32) -> UiItemRef {
        let viewport_size = viewport_max_size.min(items.len());
        let list = UiList {
            viewport_size,
            m_x: x,
            m_y: y,
            m_width: item_width,
            m_height: item_height,
            m_vec_items: items.to_vec(),
            spacing,
            pressed_item_index: usize::MAX,
        };
        Self::new(UiKind::List(list), Rect::new(x, y, item_width, item_height * viewport_size as i32), flags)
    }

    /// `UiScrollbar`
    pub fn scrollbar(bg: ClxSprite, thumb: ClxSprite, arrow: ClxSpriteList, rect: Rect, flags: UiFlags) -> UiItemRef {
        Self::new(UiKind::Scrollbar { bg, thumb, arrow }, rect, flags)
    }

    /// `UiEdit`
    pub fn edit(hint: &str, value: Rc<RefCell<String>>, max_length: usize, allow_empty: bool, rect: Rect, flags: UiFlags) -> UiItemRef {
        Self::new(UiKind::Edit { hint: hint.to_string(), value, max_length, allow_empty }, rect, flags)
    }

    /// `GetType`
    // @port DiabloUI/ui_item.h|devilution::UiItemBase::GetType() sha=ba540d613497
    pub fn get_type(&self) -> UiType {
        match &self.kind {
            UiKind::Text { .. } => UiType::Text,
            UiKind::ArtText { .. } => UiType::ArtText,
            UiKind::ArtTextButton { .. } => UiType::ArtTextButton,
            UiKind::ImageClx { .. } => UiType::ImageClx,
            UiKind::ImageAnimatedClx { .. } => UiType::ImageAnimatedClx,
            UiKind::Button { .. } => UiType::Button,
            UiKind::List(_) => UiType::List,
            UiKind::Scrollbar { .. } => UiType::Scrollbar,
            UiKind::Edit { .. } => UiType::Edit,
        }
    }

    // @port DiabloUI/ui_item.h|devilution::UiItemBase::IsType(UiType testType) sha=c778fdcdc880
    pub fn is_type(&self, t: UiType) -> bool {
        self.get_type() == t
    }

    // @port DiabloUI/ui_item.h|devilution::UiItemBase::GetFlags() sha=ff62de79dfc4
    pub fn get_flags(&self) -> UiFlags {
        self.ui_flags
    }

    pub fn set_flags(&mut self, flags: UiFlags) {
        self.ui_flags = flags;
    }

    // @port DiabloUI/ui_item.h|devilution::UiItemBase::IsHidden() sha=469f53f6c29a
    pub fn is_hidden(&self) -> bool {
        self.ui_flags.has(UiFlags::ELEMENT_HIDDEN)
    }

    // @port DiabloUI/ui_item.h|devilution::UiItemBase::IsNotInteractive() sha=4fc43306f689
    pub fn is_not_interactive(&self) -> bool {
        self.ui_flags.has(UiFlags::ELEMENT_HIDDEN | UiFlags::ELEMENT_DISABLED)
    }

    // @port DiabloUI/ui_item.h|devilution::UiItemBase::Hide() sha=fcaa70ac2a36
    pub fn hide(&mut self) {
        self.ui_flags |= UiFlags::ELEMENT_HIDDEN;
    }

    // @port DiabloUI/ui_item.h|devilution::UiItemBase::Show() sha=4ad2e2d2d643
    pub fn show(&mut self) {
        self.ui_flags &= !UiFlags::ELEMENT_HIDDEN;
    }

    /// `isCentered` of the image items.
    pub fn is_centered(&self) -> bool {
        self.ui_flags.has(UiFlags::ALIGN_CENTER)
    }

    pub fn as_list(&self) -> &UiList {
        match &self.kind {
            UiKind::List(l) => l,
            _ => panic!("not a UiList"),
        }
    }

    pub fn as_list_mut(&mut self) -> &mut UiList {
        match &mut self.kind {
            UiKind::List(l) => l,
            _ => panic!("not a UiList"),
        }
    }
}
