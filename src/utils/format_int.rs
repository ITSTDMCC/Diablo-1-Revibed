//! `Source/utils/format_int.cpp`

use crate::utils::language::tr;

/// Original: `devilution::FormatInteger` (utils/format_int.cpp).
// @port utils/format_int.cpp|devilution::FormatInteger(int n) sha=c8199fcee38f
pub fn format_integer(n: i32) -> String {
    const GROUP_SIZE: usize = 3;
    let buf = n.to_string();
    let len = buf.len();
    let prefix_len = if n < 0 { 1 } else { 0 };
    let num_len = len - prefix_len;
    if num_len <= GROUP_SIZE {
        return buf;
    }
    // TRANSLATORS: Thousands separator
    let separator = tr(",");
    let digits = &buf[prefix_len..];
    let mut out = String::with_capacity(len + separator.len() * (num_len - 1) / GROUP_SIZE);
    if n < 0 {
        out.push('-');
    }
    let mut mlen = num_len % GROUP_SIZE;
    if mlen == 0 {
        mlen = GROUP_SIZE;
    }
    out.push_str(&digits[..mlen]);
    let mut begin = mlen;
    while begin != num_len {
        out.push_str(&separator);
        out.push_str(&digits[begin..begin + GROUP_SIZE]);
        begin += GROUP_SIZE;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_thousands() {
        assert_eq!(format_integer(0), "0");
        assert_eq!(format_integer(999), "999");
        assert_eq!(format_integer(1000), "1,000");
        assert_eq!(format_integer(-1234567), "-1,234,567");
        assert_eq!(format_integer(i32::MIN), "-2,147,483,648");
    }
}
