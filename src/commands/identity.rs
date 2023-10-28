use std::str::FromStr;

use age::x25519::Identity;
use bitcoin::{
    bech32::{self, ToBase32, Variant},
    hashes::{sha256, Hash},
    Network,
};

use crate::{xpub_compatibility, IncompatibleNetwork};

use super::Seed;

const SECRET_KEY_PREFIX: &str = "age-secret-key-";

#[derive(thiserror::Error, Debug)]
pub enum IdentityError {
    #[error(transparent)]
    Bech32(#[from] bech32::Error),

    #[error(transparent)]
    Bip32(#[from] bitcoin::bip32::Error),

    #[error("Identity parsing error {0}")]
    Identity(String),

    #[error(transparent)]
    IncompatibleNetwork(#[from] IncompatibleNetwork),
}

pub fn identity(seed: &Seed, network: Network) -> Result<Identity, IdentityError> {
    let xprv = seed.xprv(network)?;
    xpub_compatibility(network, xprv.network)?;
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

        let id = identity(&seed, bitcoin::Network::Regtest).unwrap();

        assert_eq!(
            id.to_string().expose_secret(),
            "AGE-SECRET-KEY-15QHD8RZNETCHKLMUVF4PT66N38Z9WDMK92DCQKJ78K4NZQ6823CQAUVPE5"
        );

        assert_eq!(
            id.to_public().to_string(),
            "age1kjaz342hey4e9r2zj5xqrju4hapzayqn05zplcur9vkp9ycmsp5qlcjxek"
        );
    }
}
