//! The libsodium functions DevilutionX's packet encryption uses (`PACKET_ENCRYPTION`), written
//! from their specifications so the port needs no crypto dependency:
//!
//! - `crypto_pwhash` with `crypto_pwhash_ALG_ARGON2ID13`: Argon2id v1.3 (RFC 9106), one lane,
//!   built on BLAKE2b (RFC 7693);
//! - `crypto_secretbox_easy` / `crypto_secretbox_open_easy`: XSalsa20-Poly1305 (NaCl secretbox),
//!   the 16-byte MAC first, then the ciphertext;
//! - `randombytes_buf`: the operating system's random number generator.
//!
//! Checked against the published test vectors in `tests/net_parity.rs`.

// ---------------------------------------------------------------------------------------------
// BLAKE2b (RFC 7693)

const BLAKE2B_IV: [u64; 8] = [
    0x6a09e667f3bcc908,
    0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b,
    0xa54ff53a5f1d36f1,
    0x510e527fade682d1,
    0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b,
    0x5be0cd19137e2179,
];

const SIGMA: [[usize; 16]; 12] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
];

/// Incremental BLAKE2b without a key.
pub struct Blake2b {
    h: [u64; 8],
    t: u128,
    buf: [u8; 128],
    buf_len: usize,
    out_len: usize,
}

impl Blake2b {
    pub fn new(out_len: usize) -> Blake2b {
        assert!((1..=64).contains(&out_len));
        let mut h = BLAKE2B_IV;
        h[0] ^= 0x0101_0000 ^ out_len as u64;
        Blake2b { h, t: 0, buf: [0; 128], buf_len: 0, out_len }
    }

    fn compress(&mut self, last: bool) {
        let mut m = [0u64; 16];
        for (i, w) in m.iter_mut().enumerate() {
            *w = u64::from_le_bytes(self.buf[i * 8..i * 8 + 8].try_into().unwrap());
        }
        let mut v = [0u64; 16];
        v[..8].copy_from_slice(&self.h);
        v[8..].copy_from_slice(&BLAKE2B_IV);
        v[12] ^= self.t as u64;
        v[13] ^= (self.t >> 64) as u64;
        if last {
            v[14] = !v[14];
        }
        fn g(v: &mut [u64; 16], a: usize, b: usize, c: usize, d: usize, x: u64, y: u64) {
            v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
            v[d] = (v[d] ^ v[a]).rotate_right(32);
            v[c] = v[c].wrapping_add(v[d]);
            v[b] = (v[b] ^ v[c]).rotate_right(24);
            v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
            v[d] = (v[d] ^ v[a]).rotate_right(16);
            v[c] = v[c].wrapping_add(v[d]);
            v[b] = (v[b] ^ v[c]).rotate_right(63);
        }
        for s in SIGMA.iter() {
            g(&mut v, 0, 4, 8, 12, m[s[0]], m[s[1]]);
            g(&mut v, 1, 5, 9, 13, m[s[2]], m[s[3]]);
            g(&mut v, 2, 6, 10, 14, m[s[4]], m[s[5]]);
            g(&mut v, 3, 7, 11, 15, m[s[6]], m[s[7]]);
            g(&mut v, 0, 5, 10, 15, m[s[8]], m[s[9]]);
            g(&mut v, 1, 6, 11, 12, m[s[10]], m[s[11]]);
            g(&mut v, 2, 7, 8, 13, m[s[12]], m[s[13]]);
            g(&mut v, 3, 4, 9, 14, m[s[14]], m[s[15]]);
        }
        for i in 0..8 {
            self.h[i] ^= v[i] ^ v[i + 8];
        }
    }

    pub fn update(&mut self, mut data: &[u8]) {
        while !data.is_empty() {
            if self.buf_len == 128 {
                self.t += 128;
                self.compress(false);
                self.buf_len = 0;
            }
            let n = (128 - self.buf_len).min(data.len());
            self.buf[self.buf_len..self.buf_len + n].copy_from_slice(&data[..n]);
            self.buf_len += n;
            data = &data[n..];
        }
    }

    pub fn finalize(mut self) -> Vec<u8> {
        self.t += self.buf_len as u128;
        self.buf[self.buf_len..].fill(0);
        self.compress(true);
        let mut out = Vec::with_capacity(64);
        for w in self.h {
            out.extend_from_slice(&w.to_le_bytes());
        }
        out.truncate(self.out_len);
        out
    }
}

/// BLAKE2b of `data` with an `out_len`-byte digest.
pub fn blake2b(out_len: usize, data: &[u8]) -> Vec<u8> {
    let mut h = Blake2b::new(out_len);
    h.update(data);
    h.finalize()
}

/// Argon2's variable-length hash H' (RFC 9106 section 3.3).
fn blake2b_long(out_len: usize, parts: &[&[u8]]) -> Vec<u8> {
    let len_prefix = (out_len as u32).to_le_bytes();
    if out_len <= 64 {
        let mut h = Blake2b::new(out_len);
        h.update(&len_prefix);
        for p in parts {
            h.update(p);
        }
        return h.finalize();
    }
    let mut h = Blake2b::new(64);
    h.update(&len_prefix);
    for p in parts {
        h.update(p);
    }
    let mut v = h.finalize();
    let mut out = Vec::with_capacity(out_len);
    out.extend_from_slice(&v[..32]);
    let r = out_len.div_ceil(32) - 2;
    for _ in 1..r {
        v = blake2b(64, &v);
        out.extend_from_slice(&v[..32]);
    }
    let last = blake2b(out_len - 32 * r, &v);
    out.extend_from_slice(&last);
    out
}

// ---------------------------------------------------------------------------------------------
// Argon2 (RFC 9106)

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Argon2Type {
    D = 0,
    I = 1,
    Id = 2,
}

const BLOCK_WORDS: usize = 128;
type Block = [u64; BLOCK_WORDS];
const SYNC_POINTS: usize = 4;

fn blamka(x: u64, y: u64) -> u64 {
    x.wrapping_add(y).wrapping_add(2u64.wrapping_mul(x & 0xFFFF_FFFF).wrapping_mul(y & 0xFFFF_FFFF))
}

fn permute(v: &mut Block, idx: [usize; 16]) {
    fn g(v: &mut Block, a: usize, b: usize, c: usize, d: usize) {
        v[a] = blamka(v[a], v[b]);
        v[d] = (v[d] ^ v[a]).rotate_right(32);
        v[c] = blamka(v[c], v[d]);
        v[b] = (v[b] ^ v[c]).rotate_right(24);
        v[a] = blamka(v[a], v[b]);
        v[d] = (v[d] ^ v[a]).rotate_right(16);
        v[c] = blamka(v[c], v[d]);
        v[b] = (v[b] ^ v[c]).rotate_right(63);
    }
    let i = idx;
    g(v, i[0], i[4], i[8], i[12]);
    g(v, i[1], i[5], i[9], i[13]);
    g(v, i[2], i[6], i[10], i[14]);
    g(v, i[3], i[7], i[11], i[15]);
    g(v, i[0], i[5], i[10], i[15]);
    g(v, i[1], i[6], i[11], i[12]);
    g(v, i[2], i[7], i[8], i[13]);
    g(v, i[3], i[4], i[9], i[14]);
}

/// `fill_block`: next = G(prev, ref) (xor-ed into next when `with_xor`).
fn fill_block(prev: &Block, reference: &Block, next: &mut Block, with_xor: bool) {
    let mut r = [0u64; BLOCK_WORDS];
    for k in 0..BLOCK_WORDS {
        r[k] = prev[k] ^ reference[k];
    }
    let mut tmp = r;
    if with_xor {
        for k in 0..BLOCK_WORDS {
            tmp[k] ^= next[k];
        }
    }
    for i in 0..8 {
        let b = 16 * i;
        permute(&mut r, [b, b + 1, b + 2, b + 3, b + 4, b + 5, b + 6, b + 7, b + 8, b + 9, b + 10, b + 11, b + 12, b + 13, b + 14, b + 15]);
    }
    for i in 0..8 {
        let b = 2 * i;
        permute(
            &mut r,
            [b, b + 1, b + 16, b + 17, b + 32, b + 33, b + 48, b + 49, b + 64, b + 65, b + 80, b + 81, b + 96, b + 97, b + 112, b + 113],
        );
    }
    for k in 0..BLOCK_WORDS {
        next[k] = tmp[k] ^ r[k];
    }
}

fn block_from_bytes(b: &[u8]) -> Block {
    let mut out = [0u64; BLOCK_WORDS];
    for (k, w) in out.iter_mut().enumerate() {
        *w = u64::from_le_bytes(b[k * 8..k * 8 + 8].try_into().unwrap());
    }
    out
}

/// Argon2 version 0x13. `m_kib` is the memory cost in KiB, `t` the passes, `p` the lanes.
#[allow(clippy::too_many_arguments)]
pub fn argon2(ty: Argon2Type, password: &[u8], salt: &[u8], secret: &[u8], ad: &[u8], t: u32, m_kib: u32, p: u32, out_len: usize) -> Vec<u8> {
    let lanes = p as usize;
    let mut h0 = Blake2b::new(64);
    for v in [p, out_len as u32, m_kib, t, 0x13, ty as u32] {
        h0.update(&v.to_le_bytes());
    }
    for part in [password, salt, secret, ad] {
        h0.update(&(part.len() as u32).to_le_bytes());
        h0.update(part);
    }
    let h0 = h0.finalize();

    let memory_blocks = (m_kib as usize).max(2 * SYNC_POINTS * lanes);
    let segment_length = memory_blocks / (lanes * SYNC_POINTS);
    let lane_length = segment_length * SYNC_POINTS;
    let total_blocks = lane_length * lanes;
    let mut mem: Vec<Block> = vec![[0u64; BLOCK_WORDS]; total_blocks];

    for l in 0..lanes {
        for i in 0..2u32 {
            let b = blake2b_long(1024, &[&h0, &i.to_le_bytes(), &(l as u32).to_le_bytes()]);
            mem[l * lane_length + i as usize] = block_from_bytes(&b);
        }
    }

    for pass in 0..t as usize {
        for slice in 0..SYNC_POINTS {
            for lane in 0..lanes {
                let data_independent = ty == Argon2Type::I || (ty == Argon2Type::Id && pass == 0 && slice < SYNC_POINTS / 2);
                let zero_block: Block = [0; BLOCK_WORDS];
                let mut input_block: Block = [0; BLOCK_WORDS];
                let mut address_block: Block = [0; BLOCK_WORDS];
                if data_independent {
                    input_block[0] = pass as u64;
                    input_block[1] = lane as u64;
                    input_block[2] = slice as u64;
                    input_block[3] = total_blocks as u64;
                    input_block[4] = t as u64;
                    input_block[5] = ty as u64;
                }
                let next_addresses = |input_block: &mut Block, address_block: &mut Block| {
                    input_block[6] += 1;
                    fill_block(&zero_block, input_block, address_block, false);
                    let a = *address_block;
                    fill_block(&zero_block, &a, address_block, false);
                };
                let mut starting_index = 0;
                if pass == 0 && slice == 0 {
                    starting_index = 2;
                    if data_independent {
                        next_addresses(&mut input_block, &mut address_block);
                    }
                }
                let mut curr_offset = lane * lane_length + slice * segment_length + starting_index;
                let mut prev_offset = if curr_offset % lane_length == 0 { curr_offset + lane_length - 1 } else { curr_offset - 1 };
                for i in starting_index..segment_length {
                    if curr_offset % lane_length == 1 {
                        prev_offset = curr_offset - 1;
                    }
                    let pseudo_rand = if data_independent {
                        if i % BLOCK_WORDS == 0 {
                            next_addresses(&mut input_block, &mut address_block);
                        }
                        address_block[i % BLOCK_WORDS]
                    } else {
                        mem[prev_offset][0]
                    };
                    let mut ref_lane = ((pseudo_rand >> 32) % lanes as u64) as usize;
                    if pass == 0 && slice == 0 {
                        ref_lane = lane;
                    }
                    let same_lane = ref_lane == lane;
                    // index_alpha
                    let reference_area_size: u64 = if pass == 0 {
                        if slice == 0 {
                            (i - 1) as u64
                        } else if same_lane {
                            (slice * segment_length + i - 1) as u64
                        } else {
                            (slice * segment_length) as u64 - if i == 0 { 1 } else { 0 }
                        }
                    } else if same_lane {
                        (lane_length - segment_length + i - 1) as u64
                    } else {
                        (lane_length - segment_length) as u64 - if i == 0 { 1 } else { 0 }
                    };
                    let mut relative_position = pseudo_rand & 0xFFFF_FFFF;
                    relative_position = (relative_position * relative_position) >> 32;
                    relative_position = reference_area_size - 1 - ((reference_area_size * relative_position) >> 32);
                    let start_position = if pass != 0 && slice != SYNC_POINTS - 1 { (slice + 1) * segment_length } else { 0 };
                    let ref_index = (start_position + relative_position as usize) % lane_length;
                    let ref_offset = ref_lane * lane_length + ref_index;

                    let prev = mem[prev_offset];
                    let reference = mem[ref_offset];
                    fill_block(&prev, &reference, &mut mem[curr_offset], pass != 0);
                    curr_offset += 1;
                    prev_offset += 1;
                }
            }
        }
    }

    let mut c = mem[lane_length - 1];
    for l in 1..lanes {
        let last = &mem[l * lane_length + lane_length - 1];
        for k in 0..BLOCK_WORDS {
            c[k] ^= last[k];
        }
    }
    let mut c_bytes = Vec::with_capacity(1024);
    for w in c {
        c_bytes.extend_from_slice(&w.to_le_bytes());
    }
    blake2b_long(out_len, &[&c_bytes])
}

/// `crypto_pwhash(out, outlen, passwd, passwdlen, salt, opslimit, memlimit, crypto_pwhash_ALG_ARGON2ID13)`
pub fn crypto_pwhash_argon2id(out_len: usize, passwd: &[u8], salt: &[u8; 16], opslimit: u64, memlimit: usize) -> Vec<u8> {
    argon2(Argon2Type::Id, passwd, salt, &[], &[], opslimit as u32, (memlimit / 1024) as u32, 1, out_len)
}

/// `crypto_pwhash_argon2id_OPSLIMIT_MIN`, `crypto_pwhash_argon2id_MEMLIMIT_MIN`
pub const PWHASH_ARGON2ID_OPSLIMIT_MIN: u64 = 1;
pub const PWHASH_ARGON2ID_MEMLIMIT_MIN: usize = 8192;
/// `crypto_pwhash_argon2id_PASSWD_MAX` (the original clamps the password to it)
pub const PWHASH_ARGON2ID_PASSWD_MAX: usize = 4_294_967_295;

// ---------------------------------------------------------------------------------------------
// Salsa20 / XSalsa20 / Poly1305 (NaCl secretbox)

const SIGMA_WORDS: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];

fn salsa20_rounds(x: &mut [u32; 16]) {
    fn qr(x: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
        x[b] ^= x[a].wrapping_add(x[d]).rotate_left(7);
        x[c] ^= x[b].wrapping_add(x[a]).rotate_left(9);
        x[d] ^= x[c].wrapping_add(x[b]).rotate_left(13);
        x[a] ^= x[d].wrapping_add(x[c]).rotate_left(18);
    }
    for _ in 0..10 {
        // columns
        qr(x, 0, 4, 8, 12);
        qr(x, 5, 9, 13, 1);
        qr(x, 10, 14, 2, 6);
        qr(x, 15, 3, 7, 11);
        // rows
        qr(x, 0, 1, 2, 3);
        qr(x, 5, 6, 7, 4);
        qr(x, 10, 11, 8, 9);
        qr(x, 15, 12, 13, 14);
    }
}

fn salsa_state(key: &[u8; 32], input: &[u8; 16]) -> [u32; 16] {
    let k = |i: usize| u32::from_le_bytes(key[i * 4..i * 4 + 4].try_into().unwrap());
    let n = |i: usize| u32::from_le_bytes(input[i * 4..i * 4 + 4].try_into().unwrap());
    [
        SIGMA_WORDS[0],
        k(0),
        k(1),
        k(2),
        k(3),
        SIGMA_WORDS[1],
        n(0),
        n(1),
        n(2),
        n(3),
        SIGMA_WORDS[2],
        k(4),
        k(5),
        k(6),
        k(7),
        SIGMA_WORDS[3],
    ]
}

/// `crypto_core_hsalsa20`
pub fn hsalsa20(key: &[u8; 32], input: &[u8; 16]) -> [u8; 32] {
    let mut x = salsa_state(key, input);
    salsa20_rounds(&mut x);
    let mut out = [0u8; 32];
    for (j, &w) in [x[0], x[5], x[10], x[15], x[6], x[7], x[8], x[9]].iter().enumerate() {
        out[j * 4..j * 4 + 4].copy_from_slice(&w.to_le_bytes());
    }
    out
}

/// XORs `data` with the XSalsa20 keystream, starting at keystream byte `offset`.
fn xsalsa20_xor(key: &[u8; 32], nonce: &[u8; 24], offset: usize, data: &mut [u8]) {
    let subkey = hsalsa20(key, nonce[..16].try_into().unwrap());
    let mut pos = 0usize;
    while pos < data.len() {
        let stream_pos = offset + pos;
        let counter = (stream_pos / 64) as u64;
        let mut input = [0u8; 16];
        input[..8].copy_from_slice(&nonce[16..24]);
        input[8..].copy_from_slice(&counter.to_le_bytes());
        let state = salsa_state(&subkey, &input);
        let mut x = state;
        salsa20_rounds(&mut x);
        let mut block = [0u8; 64];
        for j in 0..16 {
            block[j * 4..j * 4 + 4].copy_from_slice(&x[j].wrapping_add(state[j]).to_le_bytes());
        }
        let start = stream_pos % 64;
        let n = (64 - start).min(data.len() - pos);
        for j in 0..n {
            data[pos + j] ^= block[start + j];
        }
        pos += n;
    }
}

/// `crypto_onetimeauth_poly1305` (poly1305-donna, 26-bit limbs)
pub fn poly1305(msg: &[u8], key: &[u8; 32]) -> [u8; 16] {
    let le = |b: &[u8]| u32::from_le_bytes(b[..4].try_into().unwrap());
    let r0 = le(&key[0..]) & 0x3ff_ffff;
    let r1 = (le(&key[3..]) >> 2) & 0x3ff_ff03;
    let r2 = (le(&key[6..]) >> 4) & 0x3ff_c0ff;
    let r3 = (le(&key[9..]) >> 6) & 0x3f0_3fff;
    let r4 = (le(&key[12..]) >> 8) & 0x00f_ffff;
    let (s1, s2, s3, s4) = (r1 * 5, r2 * 5, r3 * 5, r4 * 5);
    let (mut h0, mut h1, mut h2, mut h3, mut h4) = (0u32, 0u32, 0u32, 0u32, 0u32);
    for chunk in msg.chunks(16) {
        let mut block = [0u8; 17];
        block[..chunk.len()].copy_from_slice(chunk);
        let hibit: u32 = if chunk.len() == 16 { 1 << 24 } else {
            block[chunk.len()] = 1;
            0
        };
        h0 = h0.wrapping_add(le(&block[0..]) & 0x3ff_ffff);
        h1 = h1.wrapping_add((le(&block[3..]) >> 2) & 0x3ff_ffff);
        h2 = h2.wrapping_add((le(&block[6..]) >> 4) & 0x3ff_ffff);
        h3 = h3.wrapping_add((le(&block[9..]) >> 6) & 0x3ff_ffff);
        h4 = h4.wrapping_add((le(&block[12..]) >> 8) | hibit);
        let m = |a: u32, b: u32| a as u64 * b as u64;
        let d0 = m(h0, r0) + m(h1, s4) + m(h2, s3) + m(h3, s2) + m(h4, s1);
        let mut d1 = m(h0, r1) + m(h1, r0) + m(h2, s4) + m(h3, s3) + m(h4, s2);
        let mut d2 = m(h0, r2) + m(h1, r1) + m(h2, r0) + m(h3, s4) + m(h4, s3);
        let mut d3 = m(h0, r3) + m(h1, r2) + m(h2, r1) + m(h3, r0) + m(h4, s4);
        let mut d4 = m(h0, r4) + m(h1, r3) + m(h2, r2) + m(h3, r1) + m(h4, r0);
        let mut c = (d0 >> 26) as u32;
        h0 = d0 as u32 & 0x3ff_ffff;
        d1 += c as u64;
        c = (d1 >> 26) as u32;
        h1 = d1 as u32 & 0x3ff_ffff;
        d2 += c as u64;
        c = (d2 >> 26) as u32;
        h2 = d2 as u32 & 0x3ff_ffff;
        d3 += c as u64;
        c = (d3 >> 26) as u32;
        h3 = d3 as u32 & 0x3ff_ffff;
        d4 += c as u64;
        c = (d4 >> 26) as u32;
        h4 = d4 as u32 & 0x3ff_ffff;
        h0 = h0.wrapping_add(c * 5);
        c = h0 >> 26;
        h0 &= 0x3ff_ffff;
        h1 = h1.wrapping_add(c);
    }
    // full carry
    let mut c = h1 >> 26;
    h1 &= 0x3ff_ffff;
    h2 = h2.wrapping_add(c);
    c = h2 >> 26;
    h2 &= 0x3ff_ffff;
    h3 = h3.wrapping_add(c);
    c = h3 >> 26;
    h3 &= 0x3ff_ffff;
    h4 = h4.wrapping_add(c);
    c = h4 >> 26;
    h4 &= 0x3ff_ffff;
    h0 = h0.wrapping_add(c * 5);
    c = h0 >> 26;
    h0 &= 0x3ff_ffff;
    h1 = h1.wrapping_add(c);
    // h - p
    let mut g0 = h0.wrapping_add(5);
    c = g0 >> 26;
    g0 &= 0x3ff_ffff;
    let mut g1 = h1.wrapping_add(c);
    c = g1 >> 26;
    g1 &= 0x3ff_ffff;
    let mut g2 = h2.wrapping_add(c);
    c = g2 >> 26;
    g2 &= 0x3ff_ffff;
    let mut g3 = h3.wrapping_add(c);
    c = g3 >> 26;
    g3 &= 0x3ff_ffff;
    let g4 = h4.wrapping_add(c).wrapping_sub(1 << 26);
    let mask = (g4 >> 31).wrapping_sub(1);
    let nmask = !mask;
    h0 = (h0 & nmask) | (g0 & mask);
    h1 = (h1 & nmask) | (g1 & mask);
    h2 = (h2 & nmask) | (g2 & mask);
    h3 = (h3 & nmask) | (g3 & mask);
    h4 = (h4 & nmask) | (g4 & mask);
    // h = h % 2^128
    let h0w = h0 | (h1 << 26);
    let h1w = (h1 >> 6) | (h2 << 20);
    let h2w = (h2 >> 12) | (h3 << 14);
    let h3w = (h3 >> 18) | (h4 << 8);
    // mac = (h + pad) % 2^128
    let mut f = h0w as u64 + le(&key[16..]) as u64;
    let o0 = f as u32;
    f = h1w as u64 + le(&key[20..]) as u64 + (f >> 32);
    let o1 = f as u32;
    f = h2w as u64 + le(&key[24..]) as u64 + (f >> 32);
    let o2 = f as u32;
    f = h3w as u64 + le(&key[28..]) as u64 + (f >> 32);
    let o3 = f as u32;
    let mut out = [0u8; 16];
    for (j, w) in [o0, o1, o2, o3].iter().enumerate() {
        out[j * 4..j * 4 + 4].copy_from_slice(&w.to_le_bytes());
    }
    out
}

pub const SECRETBOX_KEYBYTES: usize = 32;
pub const SECRETBOX_NONCEBYTES: usize = 24;
pub const SECRETBOX_MACBYTES: usize = 16;

fn poly_key(key: &[u8; 32], nonce: &[u8; 24]) -> [u8; 32] {
    let mut k = [0u8; 32];
    xsalsa20_xor(key, nonce, 0, &mut k);
    k
}

/// `crypto_secretbox_easy`: returns MAC || ciphertext.
pub fn secretbox_easy(msg: &[u8], nonce: &[u8; 24], key: &[u8; 32]) -> Vec<u8> {
    let mut c = msg.to_vec();
    xsalsa20_xor(key, nonce, 32, &mut c);
    let mac = poly1305(&c, &poly_key(key, nonce));
    let mut out = Vec::with_capacity(16 + c.len());
    out.extend_from_slice(&mac);
    out.extend_from_slice(&c);
    out
}

/// `crypto_secretbox_open_easy`: `None` when the MAC does not verify.
pub fn secretbox_open_easy(boxed: &[u8], nonce: &[u8; 24], key: &[u8; 32]) -> Option<Vec<u8>> {
    if boxed.len() < SECRETBOX_MACBYTES {
        return None;
    }
    let (mac, c) = boxed.split_at(SECRETBOX_MACBYTES);
    let expected = poly1305(c, &poly_key(key, nonce));
    // constant-time comparison
    if mac.iter().zip(expected.iter()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) != 0 {
        return None;
    }
    let mut m = c.to_vec();
    xsalsa20_xor(key, nonce, 32, &mut m);
    Some(m)
}

// ---------------------------------------------------------------------------------------------
// randombytes_buf

#[cfg(windows)]
#[link(name = "advapi32")]
unsafe extern "system" {
    /// `RtlGenRandom`
    #[link_name = "SystemFunction036"]
    fn RtlGenRandom(buffer: *mut u8, length: u32) -> u8;
}

/// `randombytes_buf`
pub fn randombytes_buf(buf: &mut [u8]) {
    #[cfg(windows)]
    {
        // SAFETY: the buffer is valid for `len` bytes.
        let ok = unsafe { RtlGenRandom(buf.as_mut_ptr(), buf.len() as u32) };
        assert!(ok != 0, "RtlGenRandom failed");
    }
    #[cfg(not(windows))]
    {
        use std::io::Read;
        std::fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(buf)).expect("/dev/urandom");
    }
}
