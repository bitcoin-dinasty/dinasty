use std::str::FromStr;

use age::x25519::Identity;
use bitcoin::{
    bech32::{self, ToBase32, Variant},
    hashes::{sha256, Hash},
    Network,
};

use crate::{xpub_compatibility, IncompatibleNetwork};

const SECRET_KEY_PREFIX: &str = "age-secret-key-";

#[derive(thiserror::Error, Debug)]
pub enum IdentityError {
    #[error(transparent)]
    Bech32(#[from] bech32::Error),

    #[error("Identity parsing error {0}")]
    Identity(String),

    #[error(transparent)]
    IncompatibleNetwork(#[from] IncompatibleNetwork),
}

pub fn identity(
    xprv: &crate::key_origin::XprvWithSource,
    network: Network,
) -> Result<Identity, IdentityError> {
    xpub_compatibility(network, xprv.key().network)?;
    let hash = sha256::Hash::hash(xprv.to_string().as_bytes())
        .as_byte_array()
        .to_vec();

    let encoded = bech32::encode(SECRET_KEY_PREFIX, hash.to_base32(), Variant::Bech32)?;

    let identity =
        Identity::from_str(&encoded).map_err(|e| IdentityError::Identity(e.to_string()))?;

    Ok(identity)
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use age::secrecy::ExposeSecret;

    use super::identity;
    use crate::key_origin::XprvWithSource;

    #[test]
    fn test_identity() {
        let expected = "[8335dcdb/48'/1'/0'/2']tprv8ifUoGVh57yDBkyW2sS6kMNv7ewZVLmSLp1RSgZw4H5AhMP6AtxJB1P842vZcvdu9giYEfWDa6NX5nCGaaUVK5boJt1AeA8fFKv2u87Ua3g";

        let key_with_source = XprvWithSource::from_str(expected).unwrap();

        let id = identity(&key_with_source, bitcoin::Network::Regtest).unwrap();

        assert_eq!(
            id.to_string().expose_secret(),
            "AGE-SECRET-KEY-1CR8RT9HR03CGFMHPZETZCS4MF9R7QQ0K259CEKEZZQ3A075RDPMSNAA4YL"
        );

        assert_eq!(
            id.to_public().to_string(),
            "age1f7ukcx02m6mwfatanmzz75l56cm08890ehgxv9hvfyfpvj3avc2sdlxwn6"
        );
    }
}
