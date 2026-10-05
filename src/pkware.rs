//! PKWARE Data Compression Library "explode" (decompression), ported from
//! `3rdParty/PKWare/explode.cpp` (Ladislav Zezula's reimplementation used by DevilutionX 1.5.3).
//! Used for imploded MPQ files and, through `PkwareDecompress`, for save data.
//!
//! The decoder keeps the original's observable behaviour: output is produced in 0x1000-byte
//! blocks through a 0x2204-byte ring buffer, and running out of input mid-stream stops decoding
//! after flushing what was decoded (the original then returns `CMP_ABORT`, which callers ignore).

const CMP_BINARY: u8 = 0;
const CMP_ASCII: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplodeError {
    /// `CMP_BAD_DATA`: four bytes or fewer of input.
    BadData,
    /// `CMP_INVALID_DICTSIZE`
    InvalidDictSize,
    /// `CMP_INVALID_MODE`
    InvalidMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplodeStatus {
    /// `CMP_NO_ERROR`: the end-of-stream literal was reached.
    Ok,
    /// `CMP_ABORT`: input ran out or a distance was invalid; output holds what was decoded.
    Abort,
}

static DIST_BITS: [u8; 0x40] = [
    0x02, 0x04, 0x04, 0x05, 0x05, 0x05, 0x05, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06,
    0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07,
    0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07,
    0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
];

static DIST_CODE: [u8; 0x40] = [
    0x03, 0x0D, 0x05, 0x19, 0x09, 0x11, 0x01, 0x3E, 0x1E, 0x2E, 0x0E, 0x36, 0x16, 0x26, 0x06, 0x3A,
    0x1A, 0x2A, 0x0A, 0x32, 0x12, 0x22, 0x42, 0x02, 0x7C, 0x3C, 0x5C, 0x1C, 0x6C, 0x2C, 0x4C, 0x0C,
    0x74, 0x34, 0x54, 0x14, 0x64, 0x24, 0x44, 0x04, 0x78, 0x38, 0x58, 0x18, 0x68, 0x28, 0x48, 0x08,
    0xF0, 0x70, 0xB0, 0x30, 0xD0, 0x50, 0x90, 0x10, 0xE0, 0x60, 0xA0, 0x20, 0xC0, 0x40, 0x80, 0x00,
];

pub(crate) static EX_LEN_BITS: [u8; 0x10] =
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];

pub(crate) static LEN_BASE: [u16; 0x10] = [
    0x0000, 0x0001, 0x0002, 0x0003, 0x0004, 0x0005, 0x0006, 0x0007, 0x0008, 0x000A, 0x000E, 0x0016, 0x0026,
    0x0046, 0x0086, 0x0106,
];

pub(crate) static LEN_BITS: [u8; 0x10] =
    [0x03, 0x02, 0x03, 0x03, 0x04, 0x04, 0x04, 0x05, 0x05, 0x05, 0x05, 0x06, 0x06, 0x06, 0x07, 0x07];

pub(crate) static LEN_CODE: [u8; 0x10] =
    [0x05, 0x03, 0x01, 0x06, 0x0A, 0x02, 0x0C, 0x14, 0x04, 0x18, 0x08, 0x30, 0x10, 0x20, 0x40, 0x00];

pub(crate) static CH_BITS_ASC: [u8; 0x100] = [
    0x0B, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x08, 0x07, 0x0C, 0x0C, 0x07, 0x0C, 0x0C,
    0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0D, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
    0x04, 0x0A, 0x08, 0x0C, 0x0A, 0x0C, 0x0A, 0x08, 0x07, 0x07, 0x08, 0x09, 0x07, 0x06, 0x07, 0x08,
    0x07, 0x06, 0x07, 0x07, 0x07, 0x07, 0x08, 0x07, 0x07, 0x08, 0x08, 0x0C, 0x0B, 0x07, 0x09, 0x0B,
    0x0C, 0x06, 0x07, 0x06, 0x06, 0x05, 0x07, 0x08, 0x08, 0x06, 0x0B, 0x09, 0x06, 0x07, 0x06, 0x06,
    0x07, 0x0B, 0x06, 0x06, 0x06, 0x07, 0x09, 0x08, 0x09, 0x09, 0x0B, 0x08, 0x0B, 0x09, 0x0C, 0x08,
    0x0C, 0x05, 0x06, 0x06, 0x06, 0x05, 0x06, 0x06, 0x06, 0x05, 0x0B, 0x07, 0x05, 0x06, 0x05, 0x05,
    0x06, 0x0A, 0x05, 0x05, 0x05, 0x05, 0x08, 0x07, 0x08, 0x08, 0x0A, 0x0B, 0x0B, 0x0C, 0x0C, 0x0C,
    0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D,
    0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D,
    0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D,
    0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
    0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
    0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
    0x0D, 0x0C, 0x0D, 0x0D, 0x0D, 0x0C, 0x0D, 0x0D, 0x0D, 0x0C, 0x0D, 0x0D, 0x0D, 0x0D, 0x0C, 0x0D,
    0x0D, 0x0D, 0x0C, 0x0C, 0x0C, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D,
];

pub(crate) static CH_CODE_ASC: [u16; 0x100] = [
    0x0490, 0x0FE0, 0x07E0, 0x0BE0, 0x03E0, 0x0DE0, 0x05E0, 0x09E0, 0x01E0, 0x00B8, 0x0062, 0x0EE0, 0x06E0,
    0x0022, 0x0AE0, 0x02E0, 0x0CE0, 0x04E0, 0x08E0, 0x00E0, 0x0F60, 0x0760, 0x0B60, 0x0360, 0x0D60, 0x0560,
    0x1240, 0x0960, 0x0160, 0x0E60, 0x0660, 0x0A60, 0x000F, 0x0250, 0x0038, 0x0260, 0x0050, 0x0C60, 0x0390,
    0x00D8, 0x0042, 0x0002, 0x0058, 0x01B0, 0x007C, 0x0029, 0x003C, 0x0098, 0x005C, 0x0009, 0x001C, 0x006C,
    0x002C, 0x004C, 0x0018, 0x000C, 0x0074, 0x00E8, 0x0068, 0x0460, 0x0090, 0x0034, 0x00B0, 0x0710, 0x0860,
    0x0031, 0x0054, 0x0011, 0x0021, 0x0017, 0x0014, 0x00A8, 0x0028, 0x0001, 0x0310, 0x0130, 0x003E, 0x0064,
    0x001E, 0x002E, 0x0024, 0x0510, 0x000E, 0x0036, 0x0016, 0x0044, 0x0030, 0x00C8, 0x01D0, 0x00D0, 0x0110,
    0x0048, 0x0610, 0x0150, 0x0060, 0x0088, 0x0FA0, 0x0007, 0x0026, 0x0006, 0x003A, 0x001B, 0x001A, 0x002A,
    0x000A, 0x000B, 0x0210, 0x0004, 0x0013, 0x0032, 0x0003, 0x001D, 0x0012, 0x0190, 0x000D, 0x0015, 0x0005,
    0x0019, 0x0008, 0x0078, 0x00F0, 0x0070, 0x0290, 0x0410, 0x0010, 0x07A0, 0x0BA0, 0x03A0, 0x0240, 0x1C40,
    0x0C40, 0x1440, 0x0440, 0x1840, 0x0840, 0x1040, 0x0040, 0x1F80, 0x0F80, 0x1780, 0x0780, 0x1B80, 0x0B80,
    0x1380, 0x0380, 0x1D80, 0x0D80, 0x1580, 0x0580, 0x1980, 0x0980, 0x1180, 0x0180, 0x1E80, 0x0E80, 0x1680,
    0x0680, 0x1A80, 0x0A80, 0x1280, 0x0280, 0x1C80, 0x0C80, 0x1480, 0x0480, 0x1880, 0x0880, 0x1080, 0x0080,
    0x1F00, 0x0F00, 0x1700, 0x0700, 0x1B00, 0x0B00, 0x1300, 0x0DA0, 0x05A0, 0x09A0, 0x01A0, 0x0EA0, 0x06A0,
    0x0AA0, 0x02A0, 0x0CA0, 0x04A0, 0x08A0, 0x00A0, 0x0F20, 0x0720, 0x0B20, 0x0320, 0x0D20, 0x0520, 0x0920,
    0x0120, 0x0E20, 0x0620, 0x0A20, 0x0220, 0x0C20, 0x0420, 0x0820, 0x0020, 0x0FC0, 0x07C0, 0x0BC0, 0x03C0,
    0x0DC0, 0x05C0, 0x09C0, 0x01C0, 0x0EC0, 0x06C0, 0x0AC0, 0x02C0, 0x0CC0, 0x04C0, 0x08C0, 0x00C0, 0x0F40,
    0x0740, 0x0B40, 0x0340, 0x0300, 0x0D40, 0x1D00, 0x0D00, 0x1500, 0x0540, 0x0500, 0x1900, 0x0900, 0x0940,
    0x1100, 0x0100, 0x1E00, 0x0E00, 0x0140, 0x1600, 0x0600, 0x1A00, 0x0E40, 0x0640, 0x0A40, 0x0A00, 0x1200,
    0x0200, 0x1C00, 0x0C00, 0x1400, 0x0400, 0x1800, 0x0800, 0x1000, 0x0000,
];

/// `GenDecodeTabs`
fn gen_decode_tabs(positions: &mut [u8; 0x100], start_indexes: &[u8], length_bits: &[u8]) {
    for (i, (&start, &bits)) in start_indexes.iter().zip(length_bits).enumerate() {
        let length = 1usize << bits;
        let mut index = start as usize;
        while index < 0x100 {
            positions[index] = i as u8;
            index += length;
        }
    }
}

struct Work<'a> {
    ctype: u8,
    dsize_bits: u32,
    dsize_mask: u32,
    bit_buff: u32,
    extra_bits: u32,
    input: &'a [u8],
    in_pos: usize,
    out_buff: Box<[u8; 0x2204]>,
    output_pos: usize,
    dist_pos_codes: [u8; 0x100],
    length_codes: [u8; 0x100],
    offs_2c34: [u8; 0x100],
    offs_2d34: [u8; 0x100],
    offs_2e34: [u8; 0x80],
    offs_2eb4: [u8; 0x100],
    ch_bits_asc: [u8; 0x100],
}

impl Work<'_> {
    /// `GenAscTabs`
    fn gen_asc_tabs(&mut self) {
        for count in (0..=0xFFusize).rev() {
            let code = CH_CODE_ASC[count] as u32;
            let mut bits_asc = self.ch_bits_asc[count] as u32;
            if bits_asc <= 8 {
                let add = 1u32 << bits_asc;
                let mut acc = code;
                loop {
                    self.offs_2c34[acc as usize] = count as u8;
                    acc += add;
                    if acc >= 0x100 {
                        break;
                    }
                }
            } else if code & 0xFF != 0 {
                self.offs_2c34[(code & 0xFF) as usize] = 0xFF;
                if code & 0x3F != 0 {
                    bits_asc -= 4;
                    self.ch_bits_asc[count] = bits_asc as u8;
                    let add = 1u32 << bits_asc;
                    let mut acc = code >> 4;
                    loop {
                        self.offs_2d34[acc as usize] = count as u8;
                        acc += add;
                        if acc >= 0x100 {
                            break;
                        }
                    }
                } else {
                    bits_asc -= 6;
                    self.ch_bits_asc[count] = bits_asc as u8;
                    let add = 1u32 << bits_asc;
                    let mut acc = code >> 6;
                    loop {
                        self.offs_2e34[acc as usize] = count as u8;
                        acc += add;
                        if acc >= 0x80 {
                            break;
                        }
                    }
                }
            } else {
                bits_asc -= 8;
                self.ch_bits_asc[count] = bits_asc as u8;
                let add = 1u32 << bits_asc;
                let mut acc = code >> 8;
                loop {
                    self.offs_2eb4[acc as usize] = count as u8;
                    acc += add;
                    if acc >= 0x100 {
                        break;
                    }
                }
            }
        }
    }

    /// `WasteBits`: returns false at end of input (`PKDCL_STREAM_END`).
    fn waste_bits(&mut self, n_bits: u32) -> bool {
        if n_bits <= self.extra_bits {
            self.extra_bits -= n_bits;
            self.bit_buff >>= n_bits;
            return true;
        }
        self.bit_buff >>= self.extra_bits;
        if self.in_pos == self.input.len() {
            return false;
        }
        self.bit_buff |= (self.input[self.in_pos] as u32) << 8;
        self.in_pos += 1;
        self.bit_buff >>= n_bits - self.extra_bits;
        self.extra_bits = (self.extra_bits + 8) - n_bits;
        true
    }

    /// `DecodeLit`: 0x000-0x0FF byte, 0x100-0x304 repetition of length-0xFE bytes, 0x305 end, 0x306 error.
    fn decode_lit(&mut self) -> u32 {
        if self.bit_buff & 1 != 0 {
            if !self.waste_bits(1) {
                return 0x306;
            }
            let mut length_code = self.length_codes[(self.bit_buff & 0xFF) as usize] as u32;
            if !self.waste_bits(LEN_BITS[length_code as usize] as u32) {
                return 0x306;
            }
            let extra_length_bits = EX_LEN_BITS[length_code as usize] as u32;
            if extra_length_bits != 0 {
                let extra_length = self.bit_buff & ((1 << extra_length_bits) - 1);
                if !self.waste_bits(extra_length_bits) && length_code + extra_length != 0x10E {
                    return 0x306;
                }
                length_code = LEN_BASE[length_code as usize] as u32 + extra_length;
            }
            return length_code + 0x100;
        }
        if !self.waste_bits(1) {
            return 0x306;
        }
        if self.ctype == CMP_BINARY {
            let byte = self.bit_buff & 0xFF;
            if !self.waste_bits(8) {
                return 0x306;
            }
            return byte;
        }
        let value;
        if self.bit_buff & 0xFF != 0 {
            let mut v = self.offs_2c34[(self.bit_buff & 0xFF) as usize] as u32;
            if v == 0xFF {
                if self.bit_buff & 0x3F != 0 {
                    if !self.waste_bits(4) {
                        return 0x306;
                    }
                    v = self.offs_2d34[(self.bit_buff & 0xFF) as usize] as u32;
                } else {
                    if !self.waste_bits(6) {
                        return 0x306;
                    }
                    v = self.offs_2e34[(self.bit_buff & 0x7F) as usize] as u32;
                }
            }
            value = v;
        } else {
            if !self.waste_bits(8) {
                return 0x306;
            }
            value = self.offs_2eb4[(self.bit_buff & 0xFF) as usize] as u32;
        }
        if self.waste_bits(self.ch_bits_asc[value as usize] as u32) { value } else { 0x306 }
    }

    /// `DecodeDist`: 0 on end of input.
    fn decode_dist(&mut self, rep_length: u32) -> u32 {
        let dist_pos_code = self.dist_pos_codes[(self.bit_buff & 0xFF) as usize] as u32;
        let dist_pos_bits = DIST_BITS[dist_pos_code as usize] as u32;
        if !self.waste_bits(dist_pos_bits) {
            return 0;
        }
        let distance;
        if rep_length == 2 {
            distance = (dist_pos_code << 2) | (self.bit_buff & 0x03);
            if !self.waste_bits(2) {
                return 0;
            }
        } else {
            distance = (dist_pos_code << self.dsize_bits) | (self.bit_buff & self.dsize_mask);
            if !self.waste_bits(self.dsize_bits) {
                return 0;
            }
        }
        distance + 1
    }

    /// `Expand`
    fn expand(&mut self, out: &mut Vec<u8>) -> u32 {
        self.output_pos = 0x1000;
        let mut result;
        loop {
            let next_literal = self.decode_lit();
            result = next_literal;
            if next_literal >= 0x305 {
                break;
            }
            if next_literal >= 0x100 {
                let rep_length = (next_literal - 0xFE) as usize;
                let minus_dist = self.decode_dist(rep_length as u32) as usize;
                if minus_dist == 0 {
                    result = 0x306;
                    break;
                }
                // Byte-by-byte copy: the source may overlap the target (runs). A distance reaching
                // before the start of the buffer reads whatever the ring buffer holds, as in C;
                // here that is out of range only for corrupt data, which we treat as an abort.
                let target = self.output_pos;
                let Some(source) = target.checked_sub(minus_dist) else {
                    result = 0x306;
                    break;
                };
                for k in 0..rep_length {
                    self.out_buff[target + k] = self.out_buff[source + k];
                }
                self.output_pos += rep_length;
            } else {
                self.out_buff[self.output_pos] = next_literal as u8;
                self.output_pos += 1;
            }
            if self.output_pos >= 0x2000 {
                out.extend_from_slice(&self.out_buff[0x1000..0x2000]);
                self.out_buff.copy_within(0x1000..self.output_pos, 0);
                self.output_pos -= 0x1000;
            }
        }
        out.extend_from_slice(&self.out_buff[0x1000..self.output_pos]);
        result
    }
}

/// `explode`: decompresses `input` and appends the result to `out`.
pub fn explode(input: &[u8], out: &mut Vec<u8>) -> Result<ExplodeStatus, ExplodeError> {
    // The original reads at most 0x800 bytes per call into its input buffer; reading the whole
    // slice at once yields the same bit stream. Only the "> 4 bytes" check sees the first chunk.
    if input.len() <= 4 {
        return Err(ExplodeError::BadData);
    }
    let mut w = Work {
        ctype: input[0],
        dsize_bits: input[1] as u32,
        dsize_mask: 0,
        bit_buff: input[2] as u32,
        extra_bits: 0,
        input,
        in_pos: 3,
        out_buff: Box::new([0; 0x2204]),
        output_pos: 0,
        dist_pos_codes: [0; 0x100],
        length_codes: [0; 0x100],
        offs_2c34: [0; 0x100],
        offs_2d34: [0; 0x100],
        offs_2e34: [0; 0x80],
        offs_2eb4: [0; 0x100],
        ch_bits_asc: [0; 0x100],
    };
    if !(4..=6).contains(&w.dsize_bits) {
        return Err(ExplodeError::InvalidDictSize);
    }
    w.dsize_mask = 0xFFFF >> (0x10 - w.dsize_bits);
    if w.ctype != CMP_BINARY {
        if w.ctype != CMP_ASCII {
            return Err(ExplodeError::InvalidMode);
        }
        w.ch_bits_asc = CH_BITS_ASC;
        w.gen_asc_tabs();
    }
    gen_decode_tabs(&mut w.length_codes, &LEN_CODE, &LEN_BITS);
    gen_decode_tabs(&mut w.dist_pos_codes, &DIST_CODE, &DIST_BITS);
    Ok(if w.expand(out) != 0x306 { ExplodeStatus::Ok } else { ExplodeStatus::Abort })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The worked example from the PKWARE DCL format notes ("AIAIAIAIAIAIA", binary, 1 KiB dictionary).
    #[test]
    fn explodes_known_stream() {
        let input = [0x00, 0x04, 0x82, 0x24, 0x25, 0x8F, 0x80, 0x7F];
        let mut out = Vec::new();
        assert_eq!(explode(&input, &mut out), Ok(ExplodeStatus::Ok));
        assert_eq!(out, b"AIAIAIAIAIAIA");
    }

    #[test]
    fn rejects_bad_headers() {
        let mut out = Vec::new();
        assert_eq!(explode(&[0, 4, 0, 0], &mut out), Err(ExplodeError::BadData));
        assert_eq!(explode(&[0, 7, 0, 0, 0], &mut out), Err(ExplodeError::InvalidDictSize));
        assert_eq!(explode(&[2, 4, 0, 0, 0], &mut out), Err(ExplodeError::InvalidMode));
    }
}
