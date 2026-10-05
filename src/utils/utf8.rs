//! UTF-8 helpers (`Source/utils/utf8.cpp`). Replaced by Rust std (see port/replace_rules.csv);
//! the truncation rule of `CopyUtf8`, which decides what fits in fixed-size buffers, is kept.

/// `IsTrailUtf8CodeUnit`
pub fn is_trail_utf8_code_unit(b: u8) -> bool {
    (b & 0xC0) == 0x80
}

/// `CopyUtf8(dest, source, bytes)`: the source truncated to `bytes - 1` bytes at a code point
/// boundary (the C buffer keeps one byte for the terminator).
pub fn copy_utf8(source: &str, bytes: usize) -> String {
    let len = bytes.saturating_sub(1);
    if source.len() <= len {
        return source.to_string();
    }
    let b = source.as_bytes();
    let mut i = len;
    while i > 0 && is_trail_utf8_code_unit(b[i]) {
        i -= 1;
    }
    source[..i].to_string()
}
