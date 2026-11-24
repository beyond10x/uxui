//! Base64 and SHA-1, the two codecs the WebSocket handshake needs, on their published test vectors.

use ux_browser::{base64, sha1};

#[test]
fn base64_round_trips_the_rfc_4648_vectors() {
    let vectors = [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ];
    for (plain, coded) in vectors {
        assert_eq!(base64::encode(plain.as_bytes()), coded);
        assert_eq!(base64::decode(coded).unwrap(), plain.as_bytes());
    }
}

#[test]
fn base64_refuses_a_character_outside_the_alphabet() {
    assert!(base64::decode("Zm9v*mFy").is_err());
}

#[test]
fn sha1_matches_the_fips_180_vectors() {
    let hex = |d: [u8; 20]| d.iter().map(|b| format!("{b:02x}")).collect::<String>();
    assert_eq!(
        hex(sha1::digest(b"abc")),
        "a9993e364706816aba3e25717850c26c9cd0d89d"
    );
    assert_eq!(
        hex(sha1::digest(b"")),
        "da39a3ee5e6b4b0d3255bfef95601890afd80709"
    );
    assert_eq!(
        hex(sha1::digest(
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
        )),
        "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
    );
}
