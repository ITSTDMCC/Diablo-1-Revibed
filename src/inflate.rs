//! zlib/DEFLATE decompression (RFC 1950/1951) for MPQ sectors compressed with method 0x02.
//! DevilutionX links zlib for this; the port carries a small decoder instead of a crate.

#[derive(Debug, PartialEq, Eq)]
pub struct InflateError(pub &'static str);

struct Bits<'a> {
    data: &'a [u8],
    pos: usize,
    bit: u32,
    nbits: u32,
}

impl Bits<'_> {
    fn need(&mut self, n: u32) -> Result<(), InflateError> {
        while self.nbits < n {
            let b = *self.data.get(self.pos).ok_or(InflateError("unexpected end of input"))?;
            self.pos += 1;
            self.bit |= (b as u32) << self.nbits;
            self.nbits += 8;
        }
        Ok(())
    }

    fn get(&mut self, n: u32) -> Result<u32, InflateError> {
        if n == 0 {
            return Ok(0);
        }
        self.need(n)?;
        let v = self.bit & ((1u32 << n) - 1);
        self.bit >>= n;
        self.nbits -= n;
        Ok(v)
    }

    fn align(&mut self) {
        self.bit = 0;
        self.nbits = 0;
    }
}

/// Canonical Huffman decoding table: counts per length and symbols sorted by code.
struct Huffman {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

impl Huffman {
    fn new(lengths: &[u8]) -> Result<Huffman, InflateError> {
        let mut counts = [0u16; 16];
        for &l in lengths {
            counts[l as usize] += 1;
        }
        counts[0] = 0;
        let mut left: i32 = 1;
        for len in 1..16 {
            left <<= 1;
            left -= counts[len] as i32;
            if left < 0 {
                return Err(InflateError("over-subscribed code"));
            }
        }
        let mut offs = [0u16; 16];
        for len in 1..15 {
            offs[len + 1] = offs[len] + counts[len];
        }
        let mut symbols = vec![0u16; lengths.len()];
        for (sym, &l) in lengths.iter().enumerate() {
            if l != 0 {
                symbols[offs[l as usize] as usize] = sym as u16;
                offs[l as usize] += 1;
            }
        }
        Ok(Huffman { counts, symbols })
    }

    fn decode(&self, bits: &mut Bits) -> Result<u16, InflateError> {
        let (mut code, mut first, mut index) = (0i32, 0i32, 0i32);
        for len in 1..16 {
            code |= bits.get(1)? as i32;
            let count = self.counts[len] as i32;
            if code - count < first {
                return Ok(self.symbols[(index + (code - first)) as usize]);
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        Err(InflateError("bad code"))
    }
}

const LEN_BASE: [u16; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
const LEN_EXTRA: [u8; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145,
    8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];

fn codes(bits: &mut Bits, out: &mut Vec<u8>, lit: &Huffman, dist: &Huffman) -> Result<(), InflateError> {
    loop {
        let sym = lit.decode(bits)? as usize;
        if sym < 256 {
            out.push(sym as u8);
        } else if sym == 256 {
            return Ok(());
        } else {
            let s = sym - 257;
            if s >= 29 {
                return Err(InflateError("bad length symbol"));
            }
            let len = LEN_BASE[s] as usize + bits.get(LEN_EXTRA[s] as u32)? as usize;
            let d = dist.decode(bits)? as usize;
            if d >= 30 {
                return Err(InflateError("bad distance symbol"));
            }
            let distance = DIST_BASE[d] as usize + bits.get(DIST_EXTRA[d] as u32)? as usize;
            if distance > out.len() {
                return Err(InflateError("distance too far back"));
            }
            let start = out.len() - distance;
            for k in 0..len {
                out.push(out[start + k]);
            }
        }
    }
}

/// Decompresses a raw DEFLATE stream.
pub fn inflate_raw(data: &[u8], out: &mut Vec<u8>) -> Result<usize, InflateError> {
    let mut bits = Bits { data, pos: 0, bit: 0, nbits: 0 };
    loop {
        let last = bits.get(1)?;
        match bits.get(2)? {
            0 => {
                bits.align();
                let p = bits.pos;
                let hdr = data.get(p..p + 4).ok_or(InflateError("truncated stored block"))?;
                let len = u16::from_le_bytes([hdr[0], hdr[1]]) as usize;
                if u16::from_le_bytes([hdr[2], hdr[3]]) != !(len as u16) {
                    return Err(InflateError("stored block length check"));
                }
                out.extend_from_slice(data.get(p + 4..p + 4 + len).ok_or(InflateError("truncated stored block"))?);
                bits.pos = p + 4 + len;
            }
            1 => {
                let mut l = [0u8; 288];
                l[..144].fill(8);
                l[144..256].fill(9);
                l[256..280].fill(7);
                l[280..].fill(8);
                codes(&mut bits, out, &Huffman::new(&l)?, &Huffman::new(&[5u8; 30])?)?;
            }
            2 => {
                let nlen = bits.get(5)? as usize + 257;
                let ndist = bits.get(5)? as usize + 1;
                let ncode = bits.get(4)? as usize + 4;
                const ORDER: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
                let mut cl = [0u8; 19];
                for &o in ORDER.iter().take(ncode) {
                    cl[o] = bits.get(3)? as u8;
                }
                let clh = Huffman::new(&cl)?;
                let mut lengths = vec![0u8; nlen + ndist];
                let mut i = 0;
                while i < nlen + ndist {
                    let sym = clh.decode(&mut bits)?;
                    let (val, rep) = match sym {
                        0..=15 => (sym as u8, 1),
                        16 => {
                            if i == 0 {
                                return Err(InflateError("repeat with no previous length"));
                            }
                            (lengths[i - 1], 3 + bits.get(2)? as usize)
                        }
                        17 => (0, 3 + bits.get(3)? as usize),
                        _ => (0, 11 + bits.get(7)? as usize),
                    };
                    if i + rep > nlen + ndist {
                        return Err(InflateError("too many lengths"));
                    }
                    lengths[i..i + rep].fill(val);
                    i += rep;
                }
                codes(&mut bits, out, &Huffman::new(&lengths[..nlen])?, &Huffman::new(&lengths[nlen..])?)?;
            }
            _ => return Err(InflateError("invalid block type")),
        }
        if last == 1 {
            return Ok(bits.pos);
        }
    }
}

/// Decompresses a zlib stream (2-byte header, DEFLATE data, Adler-32 trailer, which is checked).
pub fn inflate_zlib(data: &[u8], out: &mut Vec<u8>) -> Result<(), InflateError> {
    if data.len() < 6 || data[0] & 0x0F != 8 || (u16::from_be_bytes([data[0], data[1]]) % 31) != 0 {
        return Err(InflateError("bad zlib header"));
    }
    if data[1] & 0x20 != 0 {
        return Err(InflateError("preset dictionary not supported"));
    }
    let start = out.len();
    let used = inflate_raw(&data[2..], out)?;
    let trailer = data.get(2 + used..2 + used + 4).ok_or(InflateError("missing adler32"))?;
    let (mut a, mut b) = (1u32, 0u32);
    for &x in &out[start..] {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    if (b << 16 | a) != u32::from_be_bytes(trailer.try_into().unwrap()) {
        return Err(InflateError("adler32 mismatch"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inflates_fixed_and_stored_blocks() {
        // zlib.compress(b"hello hello hello hello") (fixed Huffman)
        let z = [0x78, 0x9c, 0xcb, 0x48, 0xcd, 0xc9, 0xc9, 0x57, 0xc8, 0x40, 0x27, 0x01, 0x68, 0x03, 0x08, 0xb1];
        let mut out = Vec::new();
        inflate_zlib(&z, &mut out).unwrap();
        assert_eq!(out, b"hello hello hello hello");
        // zlib.compress(b"abc", 0) (stored)
        let s = [0x78, 0x01, 0x01, 0x03, 0x00, 0xfc, 0xff, 0x61, 0x62, 0x63, 0x02, 0x4d, 0x01, 0x27];
        out.clear();
        inflate_zlib(&s, &mut out).unwrap();
        assert_eq!(out, b"abc");
    }
}
