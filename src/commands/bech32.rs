use bech32::{Error, ToBase32, Variant};

pub fn bech32(
    hrp: &str,
    content: &[u8],
    lowercase: bool,
    with_checksum: bool,
) -> Result<String, Error> {
    let bech32_encoded = if with_checksum {
        bech32::encode(hrp, content.to_base32(), Variant::Bech32m)?
    } else {
        bech32::encode_without_checksum(hrp, content.to_base32())?
    };
    Ok(if lowercase {
        bech32_encoded
    } else {
        bech32_encoded.to_ascii_uppercase()
    })
}
