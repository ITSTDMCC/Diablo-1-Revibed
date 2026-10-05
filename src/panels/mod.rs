//! `Source/panels`

#[allow(unused_imports)]
use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;
pub mod charpanel;
pub mod info_box;
pub mod mainpanel;
pub mod spell_book;
pub mod spell_icons;
pub mod spell_list;

/// Globals of panels/mainpanel.cpp, panels/charpanel.cpp, panels/spell_icons.cpp and
/// panels/spell_book.cpp.
pub struct PanelsState {
    /// `PanelButtonDown` (mainpanel.cpp)
    pub panel_button_down: Option<ClxSpriteList>,
    /// `TalkButton` (mainpanel.cpp)
    pub talk_button: Option<ClxSpriteList>,
    /// `PanelButton` (mainpanel.cpp)
    pub(crate) panel_button: Option<ClxSpriteList>,
    /// `PanelButtonGrime` (mainpanel.cpp)
    pub(crate) panel_button_grime: Option<ClxSpriteList>,
    /// `PanelButtonDownGrime` (mainpanel.cpp)
    pub(crate) panel_button_down_grime: Option<ClxSpriteList>,
    /// `pChrButtons` (charpanel.cpp)
    pub p_chr_buttons: Option<ClxSpriteList>,
    /// `Panel` (charpanel.cpp)
    pub(crate) char_panel: Option<ClxSpriteList>,
    /// `panelEntries[i].position.y` for the dynamically placed headers (charpanel.cpp)
    pub(crate) attribute_headers_y: i32,
    pub(crate) gold_header_y: i32,
    /// `SmallSpellIcons` (spell_icons.cpp)
    pub(crate) small_spell_icons: Option<ClxSpriteList>,
    /// `LargeSpellIcons` (spell_icons.cpp)
    pub(crate) large_spell_icons: Option<ClxSpriteList>,
    /// `SplTransTbl` (spell_icons.cpp)
    pub(crate) spl_trans_tbl: [u8; 256],
    /// `pSBkBtnCel` (spell_book.cpp)
    pub(crate) p_s_bk_btn_cel: Option<ClxSpriteList>,
    /// `pSpellBkCel` (spell_book.cpp)
    pub(crate) p_spell_bk_cel: Option<ClxSpriteList>,
}

impl Default for PanelsState {
    fn default() -> Self {
        PanelsState {
            panel_button_down: None,
            talk_button: None,
            panel_button: None,
            panel_button_grime: None,
            panel_button_down_grime: None,
            p_chr_buttons: None,
            char_panel: None,
            attribute_headers_y: 0,
            gold_header_y: 0,
            small_spell_icons: None,
            large_spell_icons: None,
            spl_trans_tbl: [0; 256],
            p_s_bk_btn_cel: None,
            p_spell_bk_cel: None,
        }
    }
}
