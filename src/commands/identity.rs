use super::Seed;
use age::x25519::Identity;
use bitcoin::{
    bech32::{self, ToBase32, Variant},
    hashes::{sha256d, Hash},
};
use std::str::FromStr;

const SECRET_KEY_PREFIX: &str = "age-secret-key-";

#[derive(thiserror::Error, Debug)]
pub enum IdentityError {
    #[error(transparent)]
    Bech32(#[from] bech32::Error),

    #[error(transparent)]
    Bip32(#[from] bitcoin::bip32::Error),

    #[error("Identity parsing error {0}")]
    Identity(String),
}

pub fn identity(seed: &Seed) -> Result<Identity, IdentityError> {
    let mnemonic = seed.mnemonic();
    let hash = sha256d::Hash::hash(mnemonic.to_string().as_bytes())
        .as_byte_array()
        .to_vec();

    let encoded = bech32::encode(SECRET_KEY_PREFIX, hash.to_base32(), Variant::Bech32)?;

    let identity =
        Identity::from_str(&encoded).map_err(|e| IdentityError::Identity(e.to_string()))?;

    Ok(identity)
}

#[cfg(test)]
mod test {
    use super::identity;
    use crate::commands::Seed;
    use age::secrecy::ExposeSecret;
    use std::str::FromStr;

    #[test]
    fn test_identity() {
        let seed = Seed::from_str(
            "alter trial legal chuckle wear mansion sweet invest shy cabin autumn ribbon",
        )
        .unwrap();

        assert_eq!(seed.fingerprint().unwrap().to_string(), "8335dcdb");

        let id = identity(&seed).unwrap();

        assert_eq!(
            id.to_string().expose_secret(),
            "AGE-SECRET-KEY-1TZTJ6VGVHFE2703MUX0RW0TS9A4GPPPSFJKXFRW863NQYT7E295SWL07QS"
        );

        assert_eq!(
            id.to_public().to_string(),
            "age1qqly9jy2g3gfykzdnnegrjg3zpcsd086ckcllzlppaqw59puxy7q4a4asf"
        );
    }
}
