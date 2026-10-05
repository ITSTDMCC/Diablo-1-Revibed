//! Minimal PNG writer for test screenshots (stored DEFLATE blocks; no compression dependency).

use std::io::Write;
use std::path::Path;

fn crc32(chunks: &[&[u8]]) -> u32 {
    let mut table = [0u32; 256];
    for (n, t) in table.iter_mut().enumerate() {
        let mut c = n as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *t = c;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for chunk in chunks {
        for &b in *chunk {
            crc = table[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8);
        }
    }
    crc ^ 0xFFFF_FFFF
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    out.extend_from_slice(&crc32(&[kind, data]).to_be_bytes());
}

/// Encodes an RGBA8 image as PNG bytes.
pub fn encode_rgba(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let mut raw = Vec::with_capacity((width as usize * 4 + 1) * height as usize);
    for row in rgba.chunks_exact(width as usize * 4) {
        raw.push(0); // filter: none
        raw.extend_from_slice(row);
    }
    let mut z = vec![0x78, 0x01];
    let mut blocks = raw.chunks(0xFFFF).peekable();
    while let Some(b) = blocks.next() {
        z.push(if blocks.peek().is_none() { 1 } else { 0 });
        z.extend_from_slice(&(b.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(b.len() as u16)).to_le_bytes());
        z.extend_from_slice(b);
    }
    let (mut a, mut s) = (1u32, 0u32);
    for &x in &raw {
        a = (a + x as u32) % 65521;
        s = (s + a) % 65521;
    }
    z.extend_from_slice(&((s << 16) | a).to_be_bytes());
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}

pub fn write_rgba(path: &Path, width: u32, height: u32, rgba: &[u8]) -> std::io::Result<()> {
    std::fs::File::create(path)?.write_all(&encode_rgba(width, height, rgba))
}

#[cfg(test)]
mod tests {
    #[test]
    fn png_round_trips_through_our_inflate() {
        let rgba: Vec<u8> = (0..4 * 3 * 2).map(|i| i as u8).collect();
        let png = super::encode_rgba(3, 2, &rgba);
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        // IDAT payload starts after signature(8) + IHDR chunk (25) + length/type (8)
        let len = u32::from_be_bytes(png[33..37].try_into().unwrap()) as usize;
        let mut out = Vec::new();
        crate::inflate::inflate_zlib(&png[41..41 + len], &mut out).unwrap();
        assert_eq!(out.len(), (3 * 4 + 1) * 2);
        assert_eq!(&out[1..13], &rgba[..12]);
    }
}
