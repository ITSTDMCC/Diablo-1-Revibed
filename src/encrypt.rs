//! MPQ hashing/encryption and PKWARE compression helpers (`Source/encrypt.cpp`).

use std::sync::OnceLock;

use crate::pkware;

/// The `hashtable` lambda in `encrypt.cpp`: 5 x 256 values from seed 0x00100001.
fn hashtable() -> &'static [[u32; 256]; 5] {
    static TABLE: OnceLock<[[u32; 256]; 5]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut seed: u32 = 0x0010_0001;
        let mut ret = [[0u32; 256]; 5];
        for i in 0..256 {
            for row in ret.iter_mut() {
                seed = (125 * seed + 3) % 0x2A_AAAB;
                let ch = seed & 0xFFFF;
                seed = (125 * seed + 3) % 0x2A_AAAB;
                row[i] = (ch << 16) | (seed & 0xFFFF);
            }
        }
        ret
    })
}

/// Original: `devilution::Decrypt` (encrypt.cpp:63). `size` is in bytes; trailing bytes that do
/// not fill a 32-bit word are left untouched, as in the original.
// @port encrypt.cpp|devilution::Decrypt(uint32_t *castBlock, uint32_t size, uint32_t key) sha=0ce2685b11b6
pub fn decrypt(block: &mut [u8], key: u32) {
    let table = hashtable();
    let mut seed: u32 = 0xEEEE_EEEE;
    let mut key = key;
    for word in block.chunks_exact_mut(4) {
        let mut t = u32::from_le_bytes(word.try_into().unwrap());
        seed = seed.wrapping_add(table[4][(key & 0xFF) as usize]);
        t ^= seed.wrapping_add(key);
        word.copy_from_slice(&t.to_le_bytes());
        seed = seed.wrapping_add(t).wrapping_add(seed << 5).wrapping_add(3);
        key = ((key << 0x15) ^ 0xFFE0_0000).wrapping_add(0x1111_1111) | (key >> 0x0B);
    }
}

/// Original: `devilution::Encrypt` (encrypt.cpp:77).
// @port encrypt.cpp|devilution::Encrypt(uint32_t *castBlock, uint32_t size, uint32_t key) sha=48f81c0c1dbb
pub fn encrypt(block: &mut [u8], key: u32) {
    let table = hashtable();
    let mut seed: u32 = 0xEEEE_EEEE;
    let mut key = key;
    for word in block.chunks_exact_mut(4) {
        let ch = u32::from_le_bytes(word.try_into().unwrap());
        seed = seed.wrapping_add(table[4][(key & 0xFF) as usize]);
        let t = ch ^ seed.wrapping_add(key);
        word.copy_from_slice(&t.to_le_bytes());
        seed = seed.wrapping_add(ch).wrapping_add(seed << 5).wrapping_add(3);
        key = ((key << 0x15) ^ 0xFFE0_0000).wrapping_add(0x1111_1111) | (key >> 0x0B);
    }
}

/// Original: `devilution::Hash` (encrypt.cpp:92). Characters are upper-cased with the C locale.
/// The original indexes the table with a signed char, so bytes >= 0x80 read outside the row;
/// file names in the game are ASCII, and such bytes are rejected here rather than reproduced.
// @port encrypt.cpp|devilution::Hash(const char *s, int type) sha=47cae5bcf82a
pub fn hash(s: &[u8], hash_type: usize) -> u32 {
    let table = hashtable();
    let mut seed1: u32 = 0x7FED_7FED;
    let mut seed2: u32 = 0xEEEE_EEEE;
    for &c in s.iter().take_while(|&&c| c != 0) {
        assert!(c < 0x80, "non-ASCII byte {c:#x} in hashed name");
        let ch = c.to_ascii_uppercase() as u32;
        seed1 = table[hash_type][ch as usize] ^ seed1.wrapping_add(seed2);
        seed2 = ch.wrapping_add(seed1).wrapping_add(seed2).wrapping_add(seed2 << 5).wrapping_add(3);
    }
    seed1
}

/// Original: `devilution::PkwareDecompress` (encrypt.cpp:134). Decompresses `buf[..recv_size]`
/// in place, writing at most `max_bytes` (the original's output buffer size) back into `buf`.
/// Returns the number of decompressed bytes.
// @port encrypt.cpp|devilution::PkwareDecompress(byte *inBuff, uint32_t recvSize, int maxBytes) sha=d415f0817835
pub fn pkware_decompress(buf: &mut Vec<u8>, recv_size: usize, max_bytes: usize) -> usize {
    let mut out = Vec::with_capacity(max_bytes);
    // explode's errors leave the output empty; the original copies destOffset (0) bytes then.
    let _ = pkware::explode(&buf[..recv_size], &mut out);
    // The original writes into a maxBytes buffer; more output would overflow it. Fail loudly.
    assert!(out.len() <= max_bytes, "PkwareDecompress overflow: {} > {max_bytes}", out.len());
    if buf.len() < out.len() {
        buf.resize(out.len(), 0);
    }
    buf[..out.len()].copy_from_slice(&out);
    out.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_keys_match_the_mpq_format() {
        // Well-known MPQ constants: keys of the hash and block tables.
        assert_eq!(hash(b"(hash table)", 3), 0xC3AF_3770);
        assert_eq!(hash(b"(block table)", 3), 0xEC83_B3A3);
    }

    #[test]
    fn encrypt_round_trips() {
        let original: Vec<u8> = (0..64u8).collect();
        let mut data = original.clone();
        encrypt(&mut data, 0x1234_5678);
        assert_ne!(data, original);
        decrypt(&mut data, 0x1234_5678);
        assert_eq!(data, original);
    }
}
