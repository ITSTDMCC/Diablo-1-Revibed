//! bzip2 decompression for MPQ sectors compressed with method 0x10 (devilutionx.mpq uses it).
//! DevilutionX links libbz2 through libmpq; the port carries a small decoder instead of a crate.

#[derive(Debug, PartialEq, Eq)]
pub struct Bzip2Error(pub &'static str);

type Result<T> = std::result::Result<T, Bzip2Error>;

struct Bits<'a> {
    data: &'a [u8],
    pos: usize,
    buf: u64,
    nbits: u32,
}

impl Bits<'_> {
    /// Reads `n` (<= 32) bits, most significant bit first.
    fn get(&mut self, n: u32) -> Result<u32> {
        while self.nbits < n {
            let b = *self.data.get(self.pos).ok_or(Bzip2Error("unexpected end of input"))?;
            self.pos += 1;
            self.buf = (self.buf << 8) | b as u64;
            self.nbits += 8;
        }
        self.nbits -= n;
        Ok(((self.buf >> self.nbits) & ((1u64 << n) - 1)) as u32)
    }

    fn bit(&mut self) -> Result<bool> {
        Ok(self.get(1)? == 1)
    }
}

/// Canonical Huffman table in the libbz2 layout (limit/base/perm).
struct Table {
    min_len: u32,
    max_len: u32,
    limit: [i32; 22],
    base: [i32; 22],
    perm: [u16; 258],
}

impl Table {
    fn new(lengths: &[u8]) -> Table {
        let min_len = *lengths.iter().min().unwrap() as u32;
        let max_len = *lengths.iter().max().unwrap() as u32;
        let mut perm = [0u16; 258];
        let mut pp = 0;
        for len in min_len..=max_len {
            for (s, &l) in lengths.iter().enumerate() {
                if l as u32 == len {
                    perm[pp] = s as u16;
                    pp += 1;
                }
            }
        }
        let mut count = [0i32; 23];
        for &l in lengths {
            count[l as usize + 1] += 1;
        }
        let mut base = [0i32; 22];
        for i in 1..22 {
            base[i] = base[i - 1] + count[i];
        }
        // base[i] now = number of symbols with length < i
        let mut limit = [0i32; 22];
        let mut vec = 0i32;
        let mut basev = [0i32; 22];
        for len in min_len..=max_len {
            let n = base[len as usize + 1] - base[len as usize];
            vec += n;
            limit[len as usize] = vec - 1;
            vec <<= 1;
        }
        for len in (min_len + 1)..=max_len {
            basev[len as usize] = ((limit[len as usize - 1] + 1) << 1) - base[len as usize];
        }
        basev[min_len as usize] = -base[min_len as usize];
        Table { min_len, max_len, limit, base: basev, perm }
    }

    fn decode(&self, bits: &mut Bits) -> Result<u16> {
        let mut len = self.min_len;
        let mut code = bits.get(len)? as i32;
        loop {
            if code <= self.limit[len as usize] {
                let idx = code - self.base[len as usize];
                return self.perm.get(idx as usize).copied().ok_or(Bzip2Error("bad huffman code"));
            }
            len += 1;
            if len > self.max_len {
                return Err(Bzip2Error("bad huffman code"));
            }
            code = (code << 1) | bits.get(1)? as i32;
        }
    }
}

const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
const END_MAGIC: u64 = 0x1772_4538_5090;

fn crc_table() -> [u32; 256] {
    let mut t = [0u32; 256];
    for (i, e) in t.iter_mut().enumerate() {
        let mut c = (i as u32) << 24;
        for _ in 0..8 {
            c = if c & 0x8000_0000 != 0 { (c << 1) ^ 0x04c1_1db7 } else { c << 1 };
        }
        *e = c;
    }
    t
}

/// Decompresses a complete bzip2 stream, appending to `out`.
pub fn decompress(data: &[u8], out: &mut Vec<u8>) -> Result<()> {
    let mut bits = Bits { data, pos: 0, buf: 0, nbits: 0 };
    if bits.get(24)? != 0x425a68 {
        return Err(Bzip2Error("bad stream header"));
    }
    let level = bits.get(8)?;
    if !(b'1' as u32..=b'9' as u32).contains(&level) {
        return Err(Bzip2Error("bad block size"));
    }
    let max_block = (level - b'0' as u32) as usize * 100_000;
    let crc_t = crc_table();
    let mut combined: u32 = 0;
    let mut tt: Vec<u32> = Vec::with_capacity(max_block);
    loop {
        let magic = ((bits.get(24)? as u64) << 24) | bits.get(24)? as u64;
        if magic == END_MAGIC {
            let stored = bits.get(32)?;
            if stored != combined {
                return Err(Bzip2Error("stream crc mismatch"));
            }
            return Ok(());
        }
        if magic != BLOCK_MAGIC {
            return Err(Bzip2Error("bad block magic"));
        }
        let block_crc = bits.get(32)?;
        if bits.bit()? {
            return Err(Bzip2Error("randomised blocks are not supported"));
        }
        let orig_ptr = bits.get(24)? as usize;

        // symbol map
        let used16 = bits.get(16)?;
        let mut seq_to_unseq = Vec::with_capacity(256);
        for i in 0..16 {
            if used16 & (0x8000 >> i) != 0 {
                let w = bits.get(16)?;
                for j in 0..16 {
                    if w & (0x8000 >> j) != 0 {
                        seq_to_unseq.push((i * 16 + j) as u8);
                    }
                }
            }
        }
        if seq_to_unseq.is_empty() {
            return Err(Bzip2Error("empty symbol map"));
        }
        let alpha_size = seq_to_unseq.len() + 2;

        let n_groups = bits.get(3)? as usize;
        if !(2..=6).contains(&n_groups) {
            return Err(Bzip2Error("bad group count"));
        }
        let n_selectors = bits.get(15)? as usize;
        if n_selectors == 0 {
            return Err(Bzip2Error("no selectors"));
        }
        let mut mtf_groups: Vec<u8> = (0..n_groups as u8).collect();
        let mut selectors = Vec::with_capacity(n_selectors);
        for _ in 0..n_selectors {
            let mut j = 0;
            while bits.bit()? {
                j += 1;
                if j >= n_groups {
                    return Err(Bzip2Error("bad selector"));
                }
            }
            let v = mtf_groups.remove(j);
            mtf_groups.insert(0, v);
            selectors.push(v);
        }

        let mut tables = Vec::with_capacity(n_groups);
        for _ in 0..n_groups {
            let mut lengths = vec![0u8; alpha_size];
            let mut cur = bits.get(5)? as i32;
            for l in lengths.iter_mut() {
                loop {
                    if !(1..=20).contains(&cur) {
                        return Err(Bzip2Error("bad code length"));
                    }
                    if !bits.bit()? {
                        break;
                    }
                    if bits.bit()? {
                        cur -= 1;
                    } else {
                        cur += 1;
                    }
                }
                *l = cur as u8;
            }
            tables.push(Table::new(&lengths));
        }

        // MTF/RLE2 decode into tt (low byte = symbol)
        let eob = (alpha_size - 1) as u16;
        let mut mtf: Vec<u8> = (0..=255u8).collect();
        let mut unzftab = [0u32; 256];
        tt.clear();
        let mut group_idx = 0usize;
        let mut group_left = 0;
        let mut run: u32 = 0;
        let mut run_weight: u32 = 1;
        loop {
            if group_left == 0 {
                if group_idx >= selectors.len() {
                    return Err(Bzip2Error("ran out of selectors"));
                }
                group_idx += 1;
                group_left = 50;
            }
            let table = &tables[selectors[group_idx - 1] as usize];
            group_left -= 1;
            let sym = table.decode(&mut bits)?;
            if sym <= 1 {
                // RUNA = 0, RUNB = 1
                run += run_weight << sym;
                run_weight <<= 1;
                if run > max_block as u32 {
                    return Err(Bzip2Error("run too long"));
                }
                continue;
            }
            if run > 0 {
                let b = seq_to_unseq[mtf[0] as usize];
                if tt.len() + run as usize > max_block {
                    return Err(Bzip2Error("block too long"));
                }
                unzftab[b as usize] += run;
                tt.extend(std::iter::repeat_n(b as u32, run as usize));
                run = 0;
                run_weight = 1;
            }
            if sym == eob {
                break;
            }
            let idx = (sym - 1) as usize;
            let v = mtf.remove(idx);
            mtf.insert(0, v);
            let b = seq_to_unseq[v as usize];
            if tt.len() >= max_block {
                return Err(Bzip2Error("block too long"));
            }
            unzftab[b as usize] += 1;
            tt.push(b as u32);
        }
        let n = tt.len();
        if orig_ptr >= n.max(1) {
            return Err(Bzip2Error("bad origPtr"));
        }

        // inverse BWT: build the next-pointer vector in the high bits of tt
        let mut cftab = [0u32; 257];
        for i in 0..256 {
            cftab[i + 1] = cftab[i] + unzftab[i];
        }
        for i in 0..n {
            let b = (tt[i] & 0xff) as usize;
            let dst = cftab[b] as usize;
            tt[dst] |= (i as u32) << 8;
            cftab[b] += 1;
        }

        // walk the chain, undoing the initial run-length encoding (RLE1)
        let mut crc: u32 = 0xffff_ffff;
        let mut pos = tt[orig_ptr] >> 8;
        let mut last: i32 = -1;
        let mut count = 0;
        let emit = |b: u8, out: &mut Vec<u8>, crc: &mut u32| {
            out.push(b);
            *crc = (*crc << 8) ^ crc_t[((*crc >> 24) as u8 ^ b) as usize];
        };
        let mut i = 0;
        while i < n {
            let entry = tt[pos as usize];
            let b = (entry & 0xff) as u8;
            pos = entry >> 8;
            i += 1;
            if count == 4 {
                for _ in 0..b {
                    emit(last as u8, out, &mut crc);
                }
                count = 0;
                last = -1;
                continue;
            }
            if b as i32 == last {
                count += 1;
            } else {
                last = b as i32;
                count = 1;
            }
            emit(b, out, &mut crc);
        }
        let crc = !crc;
        if crc != block_crc {
            return Err(Bzip2Error("block crc mismatch"));
        }
        combined = combined.rotate_left(1) ^ crc;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Python: `bz2.compress(b'hello hello hello hello aaaaaaaaaaaaaaaaaaaa banana\n' * 3, 9)`
    const VECTOR: [u8; 63] = [
        66, 90, 104, 57, 49, 65, 89, 38, 83, 89, 29, 228, 128, 120, 0, 0, 33, 113, 0, 0, 16, 64, 0, 64, 0, 50, 69, 160, 0, 84, 64, 12, 4, 80, 134, 37,
        14, 180, 194, 26, 210, 29, 101, 243, 46, 3, 8, 91, 128, 232, 54, 48, 11, 23, 114, 69, 56, 80, 144, 29, 228, 128, 120,
    ];

    #[test]
    fn decodes_reference_stream() {
        let mut out = Vec::new();
        decompress(&VECTOR, &mut out).unwrap();
        assert_eq!(out, b"hello hello hello hello aaaaaaaaaaaaaaaaaaaa banana\n".repeat(3));
    }

    #[test]
    fn rejects_garbage() {
        let mut out = Vec::new();
        assert!(decompress(b"not bzip2", &mut out).is_err());
    }
}
