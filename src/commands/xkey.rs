use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::{
    bip32::{self, DerivationPath, ExtendedPrivKey},
    secp256k1::Secp256k1,
    Network,
};
use codex32::Codex32String;

use crate::error::Codex32ErrorWrapper;

#[derive(thiserror::Error, Debug)]
pub enum KeyError {
    #[error(transparent)]
    Bip39(#[from] bip39::Error),

    #[error(transparent)]
    Bip32(#[from] bip32::Error),

    #[error(transparent)]
    Codex32(#[from] Codex32ErrorWrapper), // wrapping is needed becose codex32::Error doesn't support Display
}

pub fn key(
    mnemonic_or_codex32: &str,
    path: &DerivationPath,
    network: Network,
) -> Result<String, KeyError> {
    let secp = Secp256k1::new();
    let mnemonic = match Mnemonic::from_str(mnemonic_or_codex32) {
        Ok(mnemonic) => mnemonic,
        Err(_) => {
            let codex32 = Codex32String::from_string(mnemonic_or_codex32.to_string())
                .map_err(Codex32ErrorWrapper)?;
            let entropy = codex32.parts().data();
            Mnemonic::from_entropy(&entropy)?
        }
    };
    let xprv = ExtendedPrivKey::new_master(network, &mnemonic.to_seed(""))?;
    let fingerprint = xprv.fingerprint(&secp);
    let derived = xprv.derive_priv(&secp, path)?;
    let path = format!("{path}");
    let path = &path[1..];
    let result = format!("[{fingerprint}{path}]{derived}");
    Ok(result)
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use bitcoin::{
        bip32::{DerivationPath, ExtendedPubKey},
        secp256k1::Secp256k1,
        Network,
    };

    use crate::key_origin::XprvWithSource;

    use super::key;

    #[test]
    fn test_liana() {
        let mnemonic =
            "alter trial legal chuckle wear mansion sweet invest shy cabin autumn ribbon";
        let expected = "[8335dcdb/48'/1'/0'/2']tprv8ifUoGVh57yDBkyW2sS6kMNv7ewZVLmSLp1RSgZw4H5AhMP6AtxJB1P842vZcvdu9giYEfWDa6NX5nCGaaUVK5boJt1AeA8fFKv2u87Ua3g";

        let path = DerivationPath::from_str("m/48h/1h/0h/2h").unwrap();
        let result = key(mnemonic, &path, Network::Testnet).unwrap();
        assert_eq!(result, expected);

        let expected_xpub ="tpubDFMWwgXwDVet5E1HvX6h9m32ggTVefxLv7cCjCcEUYsZXqdroHmtMVzzE9RcbwgWa5rCXnZqFXxtKvH7JB5JkTgsNdYdgc1nWJFXHj26ux1";
        let key_source = XprvWithSource::from_str(expected).unwrap();
        assert_eq!("8335dcdb", key_source.fingerprint().to_string());
        assert_eq!("m/48'/1'/0'/2'", key_source.path().to_string());

        let secp = Secp256k1::new();
        let xpub = ExtendedPubKey::from_priv(&secp, key_source.key());
        assert_eq!(format!("{}", xpub), expected_xpub);
    }
}
