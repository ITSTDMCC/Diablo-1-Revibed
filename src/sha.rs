//! `Source/sha.cpp`: Diablo's variant of SHA-1 (arithmetic right shifts in the rotation), used by
//! the save file codec. Not a real SHA-1, so it is ported rather than replaced.

pub const BlockSize: usize = 16;
pub const SHA1HashSize: usize = 5;

/// `SHA1Context`
#[derive(Clone, Copy, Debug)]
pub struct SHA1Context {
    pub state: [u32; SHA1HashSize],
    pub buffer: [u32; BlockSize],
}

impl Default for SHA1Context {
    fn default() -> Self {
        SHA1Context { state: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0], buffer: [0; BlockSize] }
    }
}

/// Original: `SHA1CircularShift` (sha.cpp).
// @port sha.cpp|devilution::SHA1CircularShift(uint32_t word, size_t bits) sha=fffc0b54d0f3
fn sha1_circular_shift(word: u32, bits: u32) -> u32 {
    if (word & (1 << 31)) != 0 {
        return (0xFFFF_FFFFu32 << bits) | (word >> (32 - bits));
    }
    (word << bits) | (word >> (32 - bits))
}

/// Original: `SHA1ProcessMessageBlock` (sha.cpp).
// @port sha.cpp|devilution::SHA1ProcessMessageBlock(SHA1Context *context) sha=ad4faa1aab6a
fn sha1_process_message_block(context: &mut SHA1Context) {
    let mut w = [0u32; 80];
    w[..BlockSize].copy_from_slice(&context.buffer);
    for i in 16..80 {
        w[i] = w[i - 16] ^ w[i - 14] ^ w[i - 8] ^ w[i - 3];
    }
    let [mut a, mut b, mut c, mut d, mut e] = context.state;
    for (i, &wi) in w.iter().enumerate() {
        let (f, k) = match i {
            0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
            20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
            40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
            _ => (b ^ c ^ d, 0xCA62C1D6),
        };
        let temp = sha1_circular_shift(a, 5).wrapping_add(f).wrapping_add(e).wrapping_add(wi).wrapping_add(k);
        e = d;
        d = c;
        c = sha1_circular_shift(b, 30);
        b = a;
        a = temp;
    }
    context.state[0] = context.state[0].wrapping_add(a);
    context.state[1] = context.state[1].wrapping_add(b);
    context.state[2] = context.state[2].wrapping_add(c);
    context.state[3] = context.state[3].wrapping_add(d);
    context.state[4] = context.state[4].wrapping_add(e);
}

/// Original: `devilution::SHA1Result` (sha.cpp).
// @port sha.cpp|devilution::SHA1Result(SHA1Context &context, uint32_t messageDigest[SHA1HashSize]) sha=b2105cfbba08
pub fn sha1_result(context: &SHA1Context) -> [u32; SHA1HashSize] {
    context.state
}

/// Original: `devilution::SHA1Calculate` (sha.cpp).
// @port sha.cpp|devilution::SHA1Calculate(SHA1Context &context, const uint32_t data[BlockSize]) sha=e6f4466118a9
pub fn sha1_calculate(context: &mut SHA1Context, data: &[u32; BlockSize]) {
    context.buffer = *data;
    sha1_process_message_block(context);
}
