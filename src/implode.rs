//! PKWARE Data Compression Library "implode" (compression), ported from
//! `3rdParty/PKWare/implode.cpp` (Ladislav Zezula's reimplementation used by DevilutionX 1.5.3).
//! Used for save files (`PkwareCompress`).
//!
//! The work structure (`TCmpStruct`) is reproduced field by field, including the original's reads
//! past the end of `work_buff`: those land in `phash_offs` (the next field), and they can change
//! which repetition is chosen, so the compressed bytes depend on them. `wb()` emulates that.

const MAX_REP_LENGTH: u32 = 0x204;

pub const CMP_BINARY: u32 = 0;
pub const CMP_ASCII: u32 = 1;

const WORK_BUFF_SIZE: usize = 0x2204;
const PHASH_OFFS_SIZE: usize = 0x2204;

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

static EX_LEN_BITS: [u8; 0x10] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];

static LEN_BITS: [u8; 0x10] = [0x03, 0x02, 0x03, 0x03, 0x04, 0x04, 0x04, 0x05, 0x05, 0x05, 0x05, 0x06, 0x06, 0x06, 0x07, 0x07];

static LEN_CODE: [u8; 0x10] = [0x05, 0x03, 0x01, 0x06, 0x0A, 0x02, 0x0C, 0x14, 0x04, 0x18, 0x08, 0x30, 0x10, 0x20, 0x40, 0x00];

static CH_BITS_ASC: [u8; 0x100] = [
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

static CH_CODE_ASC: [u16; 0x100] = [
    0x0490, 0x0FE0, 0x07E0, 0x0BE0, 0x03E0, 0x0DE0, 0x05E0, 0x09E0, 0x01E0, 0x00B8, 0x0062, 0x0EE0, 0x06E0, 0x0022, 0x0AE0, 0x02E0,
    0x0CE0, 0x04E0, 0x08E0, 0x00E0, 0x0F60, 0x0760, 0x0B60, 0x0360, 0x0D60, 0x0560, 0x1240, 0x0960, 0x0160, 0x0E60, 0x0660, 0x0A60,
    0x000F, 0x0250, 0x0038, 0x0260, 0x0050, 0x0C60, 0x0390, 0x00D8, 0x0042, 0x0002, 0x0058, 0x01B0, 0x007C, 0x0029, 0x003C, 0x0098,
    0x005C, 0x0009, 0x001C, 0x006C, 0x002C, 0x004C, 0x0018, 0x000C, 0x0074, 0x00E8, 0x0068, 0x0460, 0x0090, 0x0034, 0x00B0, 0x0710,
    0x0860, 0x0031, 0x0054, 0x0011, 0x0021, 0x0017, 0x0014, 0x00A8, 0x0028, 0x0001, 0x0310, 0x0130, 0x003E, 0x0064, 0x001E, 0x002E,
    0x0024, 0x0510, 0x000E, 0x0036, 0x0016, 0x0044, 0x0030, 0x00C8, 0x01D0, 0x00D0, 0x0110, 0x0048, 0x0610, 0x0150, 0x0060, 0x0088,
    0x0FA0, 0x0007, 0x0026, 0x0006, 0x003A, 0x001B, 0x001A, 0x002A, 0x000A, 0x000B, 0x0210, 0x0004, 0x0013, 0x0032, 0x0003, 0x001D,
    0x0012, 0x0190, 0x000D, 0x0015, 0x0005, 0x0019, 0x0008, 0x0078, 0x00F0, 0x0070, 0x0290, 0x0410, 0x0010, 0x07A0, 0x0BA0, 0x03A0,
    0x0240, 0x1C40, 0x0C40, 0x1440, 0x0440, 0x1840, 0x0840, 0x1040, 0x0040, 0x1F80, 0x0F80, 0x1780, 0x0780, 0x1B80, 0x0B80, 0x1380,
    0x0380, 0x1D80, 0x0D80, 0x1580, 0x0580, 0x1980, 0x0980, 0x1180, 0x0180, 0x1E80, 0x0E80, 0x1680, 0x0680, 0x1A80, 0x0A80, 0x1280,
    0x0280, 0x1C80, 0x0C80, 0x1480, 0x0480, 0x1880, 0x0880, 0x1080, 0x0080, 0x1F00, 0x0F00, 0x1700, 0x0700, 0x1B00, 0x0B00, 0x1300,
    0x0DA0, 0x05A0, 0x09A0, 0x01A0, 0x0EA0, 0x06A0, 0x0AA0, 0x02A0, 0x0CA0, 0x04A0, 0x08A0, 0x00A0, 0x0F20, 0x0720, 0x0B20, 0x0320,
    0x0D20, 0x0520, 0x0920, 0x0120, 0x0E20, 0x0620, 0x0A20, 0x0220, 0x0C20, 0x0420, 0x0820, 0x0020, 0x0FC0, 0x07C0, 0x0BC0, 0x03C0,
    0x0DC0, 0x05C0, 0x09C0, 0x01C0, 0x0EC0, 0x06C0, 0x0AC0, 0x02C0, 0x0CC0, 0x04C0, 0x08C0, 0x00C0, 0x0F40, 0x0740, 0x0B40, 0x0340,
    0x0300, 0x0D40, 0x1D00, 0x0D00, 0x1500, 0x0540, 0x0500, 0x1900, 0x0900, 0x0940, 0x1100, 0x0100, 0x1E00, 0x0E00, 0x0140, 0x1600,
    0x0600, 0x1A00, 0x0E40, 0x0640, 0x0A40, 0x0A00, 0x1200, 0x0200, 0x1C00, 0x0C00, 0x1400, 0x0400, 0x1800, 0x0800, 0x1000, 0x0000,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplodeError {
    /// `CMP_INVALID_DICTSIZE`
    InvalidDictSize,
    /// `CMP_INVALID_MODE`
    InvalidMode,
}

/// `TCmpStruct`
struct CmpStruct<'a> {
    distance: u32,
    out_bytes: u32,
    out_bits: u32,
    dsize_bits: u32,
    dsize_mask: u32,
    ctype: u32,
    dsize_bytes: u32,
    dist_bits: [u8; 0x40],
    dist_codes: [u8; 0x40],
    n_ch_bits: [u8; 0x306],
    n_ch_codes: [u16; 0x306],
    offs09bc: [u16; 0x204],
    phash_to_index: [u16; 0x900],
    out_buff: [u8; 0x802],
    work_buff: Box<[u8; WORK_BUFF_SIZE]>,
    phash_offs: Box<[u16; PHASH_OFFS_SIZE]>,
    /// `TDataInfo` of `PkwareCompress`: the source and the output.
    src: &'a [u8],
    src_offset: usize,
    dest: Vec<u8>,
}

impl CmpStruct<'_> {
    /// A byte of `work_buff`, continuing into `phash_offs` like the C struct layout.
    fn wb(&self, i: usize) -> u8 {
        if i < WORK_BUFF_SIZE {
            self.work_buff[i]
        } else {
            let j = i - WORK_BUFF_SIZE;
            let w = self.phash_offs.get(j / 2).copied().unwrap_or(0);
            if j % 2 == 0 {
                w as u8
            } else {
                (w >> 8) as u8
            }
        }
    }

    fn byte_pair_hash(&self, p: usize) -> usize {
        (self.wb(p) as usize * 4) + (self.wb(p + 1) as usize * 5)
    }

    /// `PkwareBufferRead`
    fn read_buf(&mut self, dst: usize, size: u32) -> u32 {
        let remaining = self.src.len() - self.src_offset;
        let s_size = (size as usize).min(remaining);
        self.work_buff[dst..dst + s_size].copy_from_slice(&self.src[self.src_offset..self.src_offset + s_size]);
        self.src_offset += s_size;
        s_size as u32
    }

    /// Original: `SortBuffer` (implode.cpp).
    fn sort_buffer(&mut self, buffer_begin: usize, mut buffer_end: usize) {
        self.phash_to_index = [0; 0x900];
        for p in buffer_begin..buffer_end {
            let h = self.byte_pair_hash(p);
            self.phash_to_index[h] = self.phash_to_index[h].wrapping_add(1);
        }
        let mut total_sum: u16 = 0;
        for v in self.phash_to_index.iter_mut() {
            total_sum = total_sum.wrapping_add(*v);
            *v = total_sum;
        }
        // for(buffer_end--; buffer_end >= buffer_begin; buffer_end--)
        while buffer_end > buffer_begin {
            buffer_end -= 1;
            let byte_pair_hash = self.byte_pair_hash(buffer_end);
            let byte_pair_offs = buffer_end as u16;
            self.phash_to_index[byte_pair_hash] = self.phash_to_index[byte_pair_hash].wrapping_sub(1);
            self.phash_offs[self.phash_to_index[byte_pair_hash] as usize] = byte_pair_offs;
        }
    }

    /// Original: `FlushBuf` (implode.cpp).
    fn flush_buf(&mut self) {
        self.dest.extend_from_slice(&self.out_buff[..0x800]);
        let save_ch1 = self.out_buff[0x800];
        let save_ch2 = self.out_buff[self.out_bytes as usize];
        self.out_bytes -= 0x800;
        self.out_buff = [0; 0x802];
        if self.out_bytes != 0 {
            self.out_buff[0] = save_ch1;
        }
        if self.out_bits != 0 {
            self.out_buff[self.out_bytes as usize] = save_ch2;
        }
    }

    /// Original: `OutputBits` (implode.cpp).
    fn output_bits(&mut self, mut nbits: u32, mut bit_buff: u32) {
        if nbits > 8 {
            self.output_bits(8, bit_buff);
            bit_buff >>= 8;
            nbits -= 8;
        }
        let out_bits = self.out_bits;
        self.out_buff[self.out_bytes as usize] |= (bit_buff << out_bits) as u8;
        self.out_bits += nbits;
        if self.out_bits > 8 {
            self.out_bytes += 1;
            bit_buff >>= 8 - out_bits;
            self.out_buff[self.out_bytes as usize] = bit_buff as u8;
            self.out_bits &= 7;
        } else {
            self.out_bits &= 7;
            if self.out_bits == 0 {
                self.out_bytes += 1;
            }
        }
        if self.out_bytes >= 0x800 {
            self.flush_buf();
        }
    }

    fn phash_pos(&self, index: usize) -> usize {
        self.phash_offs[index] as usize
    }

    /// The `offs09BC` update loop of `FindRep`.
    fn extend_offs09bc(&mut self, input_data: usize, offs_in_rep: &mut u16, di_val: &mut u16, limit: u32) {
        while (*offs_in_rep as u32) < limit {
            if self.wb(input_data + *offs_in_rep as usize) != self.wb(input_data + *di_val as usize) {
                *di_val = self.offs09bc[*di_val as usize];
                if *di_val != 0xFFFF {
                    continue;
                }
            }
            *offs_in_rep += 1;
            *di_val = di_val.wrapping_add(1);
            self.offs09bc[*offs_in_rep as usize] = *di_val;
        }
    }

    /// Original: `FindRep` (implode.cpp). Positions are offsets into `work_buff`.
    fn find_rep(&mut self, input_data: usize) -> u32 {
        let mut rep_length: u32 = 1;
        let mut equal_byte_count: u32 = 0;

        let h = self.byte_pair_hash(input_data);
        let min_phash_offs = (input_data as u32).wrapping_sub(self.dsize_bytes).wrapping_add(1) as u16;
        let mut phash_offs_index = self.phash_to_index[h];

        if self.phash_offs[phash_offs_index as usize] < min_phash_offs {
            while self.phash_offs[phash_offs_index as usize] < min_phash_offs {
                phash_offs_index = phash_offs_index.wrapping_add(1);
            }
            self.phash_to_index[h] = phash_offs_index;
        }

        let mut prev_repetition = self.phash_pos(phash_offs_index as usize);
        let repetition_limit = input_data - 1;

        if prev_repetition >= repetition_limit {
            return 0;
        }

        let mut input_data_ptr = input_data;
        loop {
            if self.wb(input_data_ptr) == self.wb(prev_repetition)
                && self.wb(input_data_ptr + rep_length as usize - 1) == self.wb(prev_repetition + rep_length as usize - 1)
            {
                prev_repetition += 1;
                input_data_ptr += 1;
                equal_byte_count = 2;
                while equal_byte_count < MAX_REP_LENGTH {
                    prev_repetition += 1;
                    input_data_ptr += 1;
                    if self.wb(prev_repetition) != self.wb(input_data_ptr) {
                        break;
                    }
                    equal_byte_count += 1;
                }
                input_data_ptr = input_data;
                if equal_byte_count >= rep_length {
                    self.distance = (input_data as i64 - prev_repetition as i64 + equal_byte_count as i64 - 1) as u32;
                    rep_length = equal_byte_count;
                    if rep_length > 10 {
                        break;
                    }
                }
            }
            phash_offs_index = phash_offs_index.wrapping_add(1);
            prev_repetition = self.phash_pos(phash_offs_index as usize);
            if prev_repetition >= repetition_limit {
                return if rep_length >= 2 { rep_length } else { 0 };
            }
        }

        if equal_byte_count == MAX_REP_LENGTH {
            self.distance = self.distance.wrapping_sub(1);
            return equal_byte_count;
        }

        if self.phash_pos(phash_offs_index as usize + 1) >= repetition_limit {
            return rep_length;
        }

        self.offs09bc[0] = 0xFFFF;
        self.offs09bc[1] = 0x0000;
        let mut di_val: u16 = 0;
        let mut offs_in_rep: u16 = 1;
        self.extend_offs09bc(input_data, &mut offs_in_rep, &mut di_val, rep_length);

        prev_repetition = self.phash_pos(phash_offs_index as usize);
        let mut prev_rep_end = prev_repetition + rep_length as usize;
        let mut rep_length2 = rep_length;

        loop {
            rep_length2 = self.offs09bc[rep_length2 as usize] as u32;
            if rep_length2 == 0xFFFF {
                rep_length2 = 0;
            }

            loop {
                phash_offs_index = phash_offs_index.wrapping_add(1);
                prev_repetition = self.phash_pos(phash_offs_index as usize);
                if prev_repetition >= repetition_limit {
                    return rep_length;
                }
                if prev_repetition + rep_length2 as usize >= prev_rep_end {
                    break;
                }
            }

            let pre_last_byte = self.wb(input_data + rep_length as usize - 2);
            if pre_last_byte == self.wb(prev_repetition + rep_length as usize - 2) {
                if prev_repetition + rep_length2 as usize != prev_rep_end {
                    prev_rep_end = prev_repetition;
                    rep_length2 = 0;
                }
            } else {
                loop {
                    phash_offs_index = phash_offs_index.wrapping_add(1);
                    prev_repetition = self.phash_pos(phash_offs_index as usize);
                    if prev_repetition >= repetition_limit {
                        return rep_length;
                    }
                    if !(self.wb(prev_repetition + rep_length as usize - 2) != pre_last_byte || self.wb(prev_repetition) != self.wb(input_data)) {
                        break;
                    }
                }
                prev_rep_end = prev_repetition + 2;
                rep_length2 = 2;
            }

            while self.wb(prev_rep_end) == self.wb(input_data + rep_length2 as usize) {
                rep_length2 += 1;
                if rep_length2 >= 0x204 {
                    break;
                }
                prev_rep_end += 1;
            }

            if rep_length2 >= rep_length {
                self.distance = (input_data as i64 - prev_repetition as i64 - 1) as u32;
                rep_length = rep_length2;
                if rep_length == 0x204 {
                    return rep_length;
                }
                self.extend_offs09bc(input_data, &mut offs_in_rep, &mut di_val, rep_length2);
            }
        }
    }

    fn output_literal(&mut self, ch: usize) {
        let (bits, code) = (self.n_ch_bits[ch] as u32, self.n_ch_codes[ch] as u32);
        self.output_bits(bits, code);
    }

    /// Original: `WriteCmpData` (implode.cpp).
    fn write_cmp_data(&mut self) {
        let mut input_data = (self.dsize_bytes + 0x204) as usize;
        let mut input_data_ended = false;
        let mut phase = 0;

        self.out_buff[0] = self.ctype as u8;
        self.out_buff[1] = self.dsize_bits as u8;
        self.out_bytes = 2;
        for b in self.out_buff[2..].iter_mut() {
            *b = 0;
        }
        self.out_bits = 0;

        'outer: while !input_data_ended {
            let mut bytes_to_load: u32 = 0x1000;
            let mut total_loaded: u32 = 0;
            while bytes_to_load != 0 {
                let dst = (self.dsize_bytes + 0x204 + total_loaded) as usize;
                let bytes_loaded = self.read_buf(dst, bytes_to_load);
                if bytes_loaded == 0 {
                    if total_loaded == 0 && phase == 0 {
                        break 'outer;
                    }
                    input_data_ended = true;
                    break;
                }
                bytes_to_load -= bytes_loaded;
                total_loaded += bytes_loaded;
            }

            let mut input_data_end = (self.dsize_bytes + total_loaded) as usize;
            if input_data_ended {
                input_data_end += 0x204;
            }

            match phase {
                0 => {
                    self.sort_buffer(input_data, input_data_end + 1);
                    phase += 1;
                    if self.dsize_bytes != 0x1000 {
                        phase += 1;
                    }
                }
                1 => {
                    self.sort_buffer(input_data - self.dsize_bytes as usize + 0x204, input_data_end + 1);
                    phase += 1;
                }
                _ => {
                    self.sort_buffer(input_data - self.dsize_bytes as usize, input_data_end + 1);
                }
            }

            while input_data < input_data_end {
                let mut rep_length = self.find_rep(input_data);
                let mut flushed_repetition = false;
                while rep_length != 0 {
                    if rep_length == 2 && self.distance >= 0x100 {
                        break;
                    }
                    let mut flush = false;
                    if input_data_ended && input_data + rep_length as usize > input_data_end {
                        rep_length = (input_data_end - input_data) as u32;
                        if rep_length < 2 {
                            break;
                        }
                        if rep_length == 2 && self.distance >= 0x100 {
                            break;
                        }
                        flush = true;
                    }
                    if !flush && (rep_length >= 8 || input_data + 1 >= input_data_end) {
                        flush = true;
                    }
                    if !flush {
                        let save_rep_length = rep_length;
                        let save_distance = self.distance;
                        rep_length = self.find_rep(input_data + 1);
                        if rep_length > save_rep_length && (rep_length > save_rep_length + 1 || save_distance > 0x80) {
                            let ch = self.work_buff[input_data] as usize;
                            self.output_literal(ch);
                            input_data += 1;
                            continue;
                        }
                        rep_length = save_rep_length;
                        self.distance = save_distance;
                    }

                    // __FlushRepetition:
                    self.output_literal(rep_length as usize + 0xFE);
                    if rep_length == 2 {
                        let d = (self.distance >> 2) as usize;
                        let (b, c) = (self.dist_bits[d] as u32, self.dist_codes[d] as u32);
                        self.output_bits(b, c);
                        self.output_bits(2, self.distance & 3);
                    } else {
                        let d = (self.distance >> self.dsize_bits) as usize;
                        let (b, c) = (self.dist_bits[d] as u32, self.dist_codes[d] as u32);
                        self.output_bits(b, c);
                        self.output_bits(self.dsize_bits, self.dsize_mask & self.distance);
                    }
                    input_data += rep_length as usize;
                    flushed_repetition = true;
                    break;
                }
                if !flushed_repetition {
                    let ch = self.wb(input_data) as usize;
                    self.output_literal(ch);
                    input_data += 1;
                }
            }

            if !input_data_ended {
                input_data -= 0x1000;
                let n = (self.dsize_bytes + 0x204) as usize;
                self.work_buff.copy_within(0x1000..0x1000 + n, 0);
            }
        }

        // __Exit:
        self.output_literal(0x305);
        if self.out_bits != 0 {
            self.out_bytes += 1;
        }
        let n = self.out_bytes as usize;
        self.dest.extend_from_slice(&self.out_buff[..n]);
    }
}

/// Original: `implode` (3rdParty/PKWare/implode.cpp), reading all of `src` and returning the
/// compressed stream. The work buffer starts zeroed, as `PkwareCompress` allocates it.
pub fn implode(src: &[u8], ctype: u32, dsize: u32) -> Result<Vec<u8>, ImplodeError> {
    let mut w = CmpStruct {
        distance: 0,
        out_bytes: 0,
        out_bits: 0,
        dsize_bits: 4,
        dsize_mask: 0x0F,
        ctype,
        dsize_bytes: dsize,
        dist_bits: [0; 0x40],
        dist_codes: [0; 0x40],
        n_ch_bits: [0; 0x306],
        n_ch_codes: [0; 0x306],
        offs09bc: [0; 0x204],
        phash_to_index: [0; 0x900],
        out_buff: [0; 0x802],
        work_buff: Box::new([0; WORK_BUFF_SIZE]),
        phash_offs: Box::new([0; PHASH_OFFS_SIZE]),
        src,
        src_offset: 0,
        dest: Vec::new(),
    };

    match dsize {
        4096 => {
            w.dsize_bits += 2;
            w.dsize_mask |= 0x20 | 0x10;
        }
        2048 => {
            w.dsize_bits += 1;
            w.dsize_mask |= 0x10;
        }
        1024 => {}
        _ => return Err(ImplodeError::InvalidDictSize),
    }

    let mut n_count: usize;
    match ctype {
        CMP_BINARY => {
            let mut n_ch_code: u32 = 0;
            n_count = 0;
            while n_count < 0x100 {
                w.n_ch_bits[n_count] = 9;
                w.n_ch_codes[n_count] = n_ch_code as u16;
                n_ch_code = (n_ch_code & 0x0000_FFFF) + 2;
                n_count += 1;
            }
        }
        CMP_ASCII => {
            n_count = 0;
            while n_count < 0x100 {
                w.n_ch_bits[n_count] = CH_BITS_ASC[n_count] + 1;
                w.n_ch_codes[n_count] = CH_CODE_ASC[n_count].wrapping_mul(2);
                n_count += 1;
            }
        }
        _ => return Err(ImplodeError::InvalidMode),
    }

    for i in 0..0x10 {
        for n_count2 in 0..(1u32 << EX_LEN_BITS[i]) {
            w.n_ch_bits[n_count] = EX_LEN_BITS[i] + LEN_BITS[i] + 1;
            w.n_ch_codes[n_count] = ((n_count2 << (LEN_BITS[i] + 1)) | ((LEN_CODE[i] as u32 & 0xFFFF_00FF) * 2) | 1) as u16;
            n_count += 1;
        }
    }

    w.dist_codes = DIST_CODE;
    w.dist_bits = DIST_BITS;
    w.write_cmp_data();
    Ok(w.dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(data: &[u8], ctype: u32, dsize: u32) {
        let packed = implode(data, ctype, dsize).unwrap();
        if packed.len() <= 4 {
            // explode rejects streams of 4 bytes or less (CMP_BAD_DATA)
            return;
        }
        let mut out = Vec::new();
        crate::pkware::explode(&packed, &mut out).unwrap();
        assert_eq!(out, data, "ctype {ctype} dsize {dsize} len {}", data.len());
    }

    #[test]
    fn round_trips_through_explode() {
        let mut seed = 7u32;
        let mut noisy = Vec::new();
        for _ in 0..20000 {
            seed = seed.wrapping_mul(214013).wrapping_add(2531011);
            noisy.push(((seed >> 16) % 7) as u8 + b'a');
        }
        let text: Vec<u8> = b"The Butcher's cleaver rests here. ".iter().cycle().take(9000).copied().collect();
        let zeros = vec![0u8; 5000];
        for data in [&b""[..], b"a", b"ab", &noisy[..], &text[..], &zeros[..], &noisy[..4096], &noisy[..4095], &noisy[..4097]] {
            for dsize in [1024, 2048, 4096] {
                round_trip(data, CMP_BINARY, dsize);
                round_trip(data, CMP_ASCII, dsize);
            }
        }
    }

    #[test]
    fn compresses_repetitive_data() {
        let zeros = vec![0u8; 5000];
        let packed = implode(&zeros, CMP_BINARY, 4096).unwrap();
        assert!(packed.len() < 100, "{}", packed.len());
    }
}
