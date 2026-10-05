//! `Source/pack.cpp` / `pack.h`

#[allow(unused_imports)]
use crate::ctx::Ctx;

/// `sizeof(ItemPack)`: the packed (1-byte aligned) struct: u32 iSeed, u16 iCreateInfo, u16 idx,
/// u8 bId, bDur, bMDur, bCh, bMCh, u16 wValue, u32 dwBuff.
pub const ITEM_PACK_SIZE: usize = 4 + 2 + 2 + 5 + 2 + 4;
