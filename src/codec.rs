//! `Source/codec.cpp`: the save file block cipher (Diablo SHA-1 keyed XOR with a signature).

use crate::platform::log;
use crate::sha::{sha1_calculate, sha1_result, BlockSize, SHA1Context, SHA1HashSize};

const BLOCK_SIZE_BYTES: usize = BlockSize * 4;
const SIGNATURE_SIZE: usize = 8;

/// Original: `CodecInitKey` (codec.cpp).
// @port codec.cpp|devilution::CodecInitKey(const char *pszPassword) sha=47d084940bea
fn codec_init_key(password: &str) -> SHA1Context {
    // The password as a NUL-terminated C string (padded so 4-byte loads stay in bounds).
    let mut pass = password.as_bytes().to_vec();
    pass.extend_from_slice(&[0, 0, 0, 0]);
    let mut pw = [0u32; BlockSize];
    let mut j = 0usize;
    for value in pw.iter_mut() {
        if pass[j] == 0 {
            j = 0;
        }
        *value = u32::from_le_bytes([pass[j], pass[j + 1], pass[j + 2], pass[j + 3]]);
        j += 4;
    }
    let digest = {
        let mut context = SHA1Context::default();
        sha1_calculate(&mut context, &pw);
        sha1_result(&context)
    };
    let mut key: [u32; BlockSize] = [
        2908958655, 4146550480, 658981742, 1113311088, 3927878744, 679301322, 1760465731, 3305370375, 2269115995, 3928541685, 580724401, 2607446661,
        2233092279, 2416822349, 4106933702, 3046442503,
    ];
    for (i, k) in key.iter_mut().enumerate() {
        *k ^= digest[(i + 3) % SHA1HashSize];
    }
    let mut context = SHA1Context::default();
    sha1_calculate(&mut context, &key);
    context
}

fn load_block(src: &[u8]) -> [u32; BlockSize] {
    let mut buf = [0u32; BlockSize];
    for (i, w) in buf.iter_mut().enumerate() {
        *w = u32::from_le_bytes([src[i * 4], src[i * 4 + 1], src[i * 4 + 2], src[i * 4 + 3]]);
    }
    buf
}

fn store_block(dst: &mut [u8], buf: &[u32; BlockSize]) {
    for (i, w) in buf.iter().enumerate() {
        dst[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes());
    }
}

/// Original: `XorBlock` (codec.cpp).
// @port codec.cpp|devilution::XorBlock(const uint32_t *shaResult, uint32_t *out) sha=50551f525c22
fn xor_block(sha_result: &[u32; SHA1HashSize], out: &mut [u32; BlockSize]) {
    for (i, o) in out.iter_mut().enumerate() {
        *o ^= sha_result[i % SHA1HashSize];
    }
}

/// Original: `devilution::codec_decode` (codec.cpp). Decodes `data` in place and returns the
/// decoded length, or 0 when the data is invalid.
// @port codec.cpp|devilution::codec_decode(byte *pbSrcDst, std::size_t size, const char *pszPassword) sha=77c5464c5f99
pub fn codec_decode(data: &mut [u8], password: &str) -> usize {
    let mut context = codec_init_key(password);
    let mut size = data.len();
    if size <= SIGNATURE_SIZE {
        return 0;
    }
    size -= SIGNATURE_SIZE;
    if size % BlockSize != 0 {
        return 0;
    }
    let mut pos = 0;
    while pos < size {
        let mut buf = load_block(&data[pos..]);
        let dst = sha1_result(&context);
        xor_block(&dst, &mut buf);
        sha1_calculate(&mut context, &buf);
        store_block(&mut data[pos..], &buf);
        pos += BLOCK_SIZE_BYTES;
    }
    let checksum = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
    let error = data[pos + 4];
    let last_chunk_size = data[pos + 5];
    if error > 0 {
        return 0;
    }
    let dst = sha1_result(&context);
    if checksum != dst[0] {
        log::error!("Checksum mismatch signature={} vs calculated={}", checksum, dst[0]);
        return 0;
    }
    (size as i64 + last_chunk_size as i64 - BLOCK_SIZE_BYTES as i64) as usize
}

/// Original: `devilution::codec_get_encoded_len` (codec.cpp).
// @port codec.cpp|devilution::codec_get_encoded_len(std::size_t dwSrcBytes) sha=d9c83def25fb
pub fn codec_get_encoded_len(mut src_bytes: usize) -> usize {
    if src_bytes % BLOCK_SIZE_BYTES != 0 {
        src_bytes += BLOCK_SIZE_BYTES - (src_bytes % BLOCK_SIZE_BYTES);
    }
    src_bytes + SIGNATURE_SIZE
}

/// Original: `devilution::codec_encode` (codec.cpp). `data` holds `size` bytes of input and is
/// resized to `codec_get_encoded_len(size)`.
// @port codec.cpp|devilution::codec_encode(byte *pbSrcDst, std::size_t size, std::size_t size64, const char *pszPassword) sha=ee7602b06f84
pub fn codec_encode(data: &mut Vec<u8>, size: usize, password: &str) {
    let size64 = codec_get_encoded_len(size);
    data.resize(size64, 0);
    let mut context = codec_init_key(password);
    let mut remaining = size;
    let mut pos = 0;
    let mut last_chunk = 0;
    while remaining != 0 {
        let chunk = remaining.min(BLOCK_SIZE_BYTES);
        let mut bytes = [0u8; BLOCK_SIZE_BYTES];
        bytes[..chunk].copy_from_slice(&data[pos..pos + chunk]);
        let mut buf = load_block(&bytes);
        let dst = sha1_result(&context);
        sha1_calculate(&mut context, &buf);
        xor_block(&dst, &mut buf);
        store_block(&mut data[pos..], &buf);
        pos += BLOCK_SIZE_BYTES;
        last_chunk = chunk;
        remaining -= chunk;
    }
    let tmp = sha1_result(&context);
    data[pos..pos + 4].copy_from_slice(&tmp[0].to_le_bytes());
    data[pos + 4] = 0;
    data[pos + 5] = last_chunk as u8;
    data[pos + 6] = 0;
    data[pos + 7] = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_then_decode() {
        for len in [1usize, 63, 64, 65, 1266] {
            let original: Vec<u8> = (0..len).map(|i| (i * 7 + 3) as u8).collect();
            let mut data = original.clone();
            codec_encode(&mut data, len, "xrgyrkj1");
            assert_eq!(data.len(), codec_get_encoded_len(len));
            let mut copy = data.clone();
            let n = codec_decode(&mut copy, "xrgyrkj1");
            assert_eq!(n, len);
            assert_eq!(&copy[..n], &original[..]);
            assert_eq!(codec_decode(&mut data, "szqnlsk1"), 0);
        }
    }
}
