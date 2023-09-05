use std::{fmt::Display, str::FromStr};

use bitcoin::{
    bip32::{self, DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint, KeySource},
    hashes::hex,
    secp256k1::Secp256k1,
};

#[derive(thiserror::Error, Debug)]
pub enum KeyOriginError {
    #[error("Invalid extended private key with source({0})")]
    InvalidXprvWithSource(String),

    #[error(transparent)]
    Bip32(#[from] bip32::Error),

    #[error(transparent)]
    Hex(#[from] hex::Error),
}

pub struct XprvWithSource {
    /// Source of the key
    source: KeySource,
    /// The extended private key, like tprv8ifUoGVh57yDBkyW2sS6kMNv7ewZVLmSLp1RSgZw4H5AhMP6AtxJB1P842vZcvdu9giYEfWDa6NX5nCGaaUVK5boJt1AeA8fFKv2u87Ua3g
    key: ExtendedPrivKey,
}

impl XprvWithSource {
    pub fn fingerprint(&self) -> &Fingerprint {
        &self.source.0
    }
    pub fn path(&self) -> &DerivationPath {
        &self.source.1
    }
    pub fn key(&self) -> &ExtendedPrivKey {
        &self.key
    }
}

impl Display for XprvWithSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        let path = format!("{}", self.path());
        let path_with_fingerprint = path.replace('m', &self.fingerprint().to_string());
        f.write_str(&path_with_fingerprint)?;

        f.write_str("]")?;
        if f.alternate() {
            let secp = Secp256k1::new();
            let key = ExtendedPubKey::from_priv(&secp, self.key());
            f.write_fmt(format_args!("{}", key))
        } else {
            f.write_fmt(format_args!("{}", self.key))
        }
    }
}

impl FromStr for XprvWithSource {
    type Err = KeyOriginError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pieces: Vec<_> = s.split(']').collect();
        let source = pieces
            .first()
            .ok_or(KeyOriginError::InvalidXprvWithSource(s.to_string()))?;
        let key = pieces
            .get(1)
            .ok_or(KeyOriginError::InvalidXprvWithSource(s.to_string()))?;
        let key = ExtendedPrivKey::from_str(key)?;

        let sub_pieces: Vec<_> = source.split('/').collect();
        let fingerprint = sub_pieces
            .first()
            .ok_or(KeyOriginError::InvalidXprvWithSource(s.to_string()))?;
        let fingerprint = fingerprint
            .get(1..)
            .ok_or(KeyOriginError::InvalidXprvWithSource(s.to_string()))?;
        let fingerprint = Fingerprint::from_str(fingerprint)?;
        let path = if !sub_pieces.is_empty() {
            format!("m/{}", sub_pieces[1..].join("/"))
        } else {
            "m".to_string()
        };
        let path = DerivationPath::from_str(&path)?;

        Ok(XprvWithSource {
            source: (fingerprint, path),
            key,
        })
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::XprvWithSource;

    #[test]
    fn key_with_source() {
        let expected = "[8335dcdb/48'/1'/0'/2']tprv8ifUoGVh57yDBkyW2sS6kMNv7ewZVLmSLp1RSgZw4H5AhMP6AtxJB1P842vZcvdu9giYEfWDa6NX5nCGaaUVK5boJt1AeA8fFKv2u87Ua3g";

        let key_with_source = XprvWithSource::from_str(expected).unwrap();

        assert_eq!(expected, format!("{}", key_with_source));

        let expected_xpub = "[8335dcdb/48'/1'/0'/2']tpubDFMWwgXwDVet5E1HvX6h9m32ggTVefxLv7cCjCcEUYsZXqdroHmtMVzzE9RcbwgWa5rCXnZqFXxtKvH7JB5JkTgsNdYdgc1nWJFXHj26ux1";

        assert_eq!(expected_xpub, format!("{:#}", key_with_source));
    }
}
