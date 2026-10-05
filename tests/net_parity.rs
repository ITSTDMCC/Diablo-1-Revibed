//! The network layer: the libsodium primitives DevilutionX's packet encryption uses, checked
//! against published test vectors (RFC 7693, RFC 8439, RFC 9106 and NaCl's own tests), and the
//! dvlnet packet/frame formats.

use diablo1_rs::dvlnet::crypto::*;

fn hex(s: &str) -> Vec<u8> {
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

#[test]
fn blake2b_rfc7693() {
    // RFC 7693 appendix A: BLAKE2b-512("abc")
    assert_eq!(
        blake2b(64, b"abc"),
        hex("ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923")
    );
    assert_eq!(
        blake2b(64, b""),
        hex("786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce")
    );
}

#[test]
fn poly1305_rfc8439() {
    // RFC 8439 section 2.5.2
    let key: [u8; 32] = hex("85d6be7857556d337f4452fe42d506a80103808afb0db2fd4abff6af4149f51b").try_into().unwrap();
    assert_eq!(poly1305(b"Cryptographic Forum Research Group", &key).to_vec(), hex("a8061dc1305136c6c22b8baf0c0127a9"));
}

#[test]
fn argon2_rfc9106() {
    // RFC 9106 section 5: t=3, m=32 KiB, p=4, 32-byte tag
    let (pw, salt, secret, ad) = ([1u8; 32], [2u8; 16], [3u8; 8], [4u8; 12]);
    let run = |ty| argon2(ty, &pw, &salt, &secret, &ad, 3, 32, 4, 32);
    assert_eq!(run(Argon2Type::D), hex("512b391b6f1162975371d30919734294f868e3be3984f3c1a13a4db9fabe4acb"), "Argon2d");
    assert_eq!(run(Argon2Type::I), hex("c814d9d1dc7f37aa13f0d77f2494bda1c8de6b016dd388d29952a4c4672b6ce8"), "Argon2i");
    assert_eq!(run(Argon2Type::Id), hex("0d640df58d78766c08c037a34a8b53c9d01ef0452d75b65eb52520e96b01e659"), "Argon2id");
}

#[test]
fn hsalsa20_nacl_core1() {
    // NaCl tests/core1.c: HSalsa20 of the shared secret with a zero input gives "firstkey"
    let shared: [u8; 32] = hex("4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742").try_into().unwrap();
    assert_eq!(hsalsa20(&shared, &[0; 16]).to_vec(), hex("1b27556473e985d462cd51197a9a46c76009549eac6474f206c4ee0844f68389"));
}

#[test]
fn secretbox_nacl() {
    // NaCl tests/secretbox.c
    let key: [u8; 32] = hex("1b27556473e985d462cd51197a9a46c76009549eac6474f206c4ee0844f68389").try_into().unwrap();
    let nonce: [u8; 24] = hex("69696ee955b62b73cd62bda875fc73d68219e0036b7a0b37").try_into().unwrap();
    let m = hex(
        "be075fc53c81f2d5cf141316ebeb0c7b5228c52a4c62cbd44b66849b64244ffce5ecbaaf33bd751a1ac728d45e6c61296cdc3c01233561f41db66cce314adb310e3be8250c46f06dceea3a7fa1348057e2f6556ad6b1318a024a838f21af1fde048977eb48f59ffd4924ca1c60902e52f0a089bc76897040e082f937763848645e0705",
    );
    let c = hex(
        "f3ffc7703f9400e52a7dfb4b3d3305d98e993b9f48681273c29650ba32fc76ce48332ea7164d96a4476fb8c531a1186ac0dfc17c98dce87b4da7f011ec48c97271d2c20f9b928fe2270d6fb863d51738b48eeee314a7cc8ab932164548e526ae90224368517acfeabd6bb3732bc0e9da99832b61ca01b6de56244a9e88d5f9b37973f622a43d14a6599b1f654cb45a74e355a5",
    );
    assert_eq!(secretbox_easy(&m, &nonce, &key), c);
    assert_eq!(secretbox_open_easy(&c, &nonce, &key), Some(m.clone()));
    let mut forged = c.clone();
    forged[20] ^= 1;
    assert_eq!(secretbox_open_easy(&forged, &nonce, &key), None);
}
