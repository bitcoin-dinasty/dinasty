use std::str::FromStr;

use bitcoin::{
    bip32::{DerivationPath, ExtendedPubKey},
    secp256k1::Secp256k1,
    Network,
};

use super::{import::explode_descriptor, ImportError, Seed};

#[derive(thiserror::Error, Debug)]
pub enum DescriptorError {
    #[error(transparent)]
    Bip32(#[from] bitcoin::bip32::Error),

    #[error(transparent)]
    Import(#[from] ImportError),
}

pub fn descriptor(
    seed: Seed,
    network: Network,
    account: u16,
    public: bool,
) -> Result<String, DescriptorError> {
    let coin_type = match network {
        Network::Bitcoin => 0,
        _ => 1,
    };

    let path_str = format!("86h/{coin_type}h/{account}h");

    let bip86_path = DerivationPath::from_str(&format!("m/{path_str}"))
        .expect("coin type [0,1] and account (u16) cannot exceed valid values");

    let fingerprint = seed.fingerprint()?;

    let secp = Secp256k1::new();
    let xprv = seed.xprv(network)?.derive_priv(&secp, &bip86_path)?;

    let xkey = if public {
        ExtendedPubKey::from_priv(&secp, &xprv).to_string()
    } else {
        xprv.to_string()
    };

    let desc = format!("tr([{fingerprint}/{path_str}]{xkey}/<0;1>/*)");

    let _ = explode_descriptor(&desc, !public)?;

    Ok(desc)
}

#[cfg(test)]
mod test {
    use crate::commands::Seed;
    use std::str::FromStr;

    #[test]
    fn test_liana() {
        let mnemonic =
            "alter trial legal chuckle wear mansion sweet invest shy cabin autumn ribbon";
        let seed = Seed::from_str(mnemonic).unwrap();
        assert_eq!(seed.fingerprint().unwrap().to_string(), "8335dcdb");

        // let expected = "[8335dcdb/48'/1'/0'/2']tprv8ifUoGVh57yDBkyW2sS6kMNv7ewZVLmSLp1RSgZw4H5AhMP6AtxJB1P842vZcvdu9giYEfWDa6NX5nCGaaUVK5boJt1AeA8fFKv2u87Ua3g";

        // let path = DerivationPath::from_str("m/48h/1h/0h/2h").unwrap();
        // let result = key(mnemonic, &path, Network::Testnet);
        // assert_eq!(result, expected);

        // let expected_xpub ="tpubDFMWwgXwDVet5E1HvX6h9m32ggTVefxLv7cCjCcEUYsZXqdroHmtMVzzE9RcbwgWa5rCXnZqFXxtKvH7JB5JkTgsNdYdgc1nWJFXHj26ux1";
        // let key_source = XprvWithSource::from_str(expected).unwrap();
        // assert_eq!("8335dcdb", key_source.fingerprint().to_string());
        // assert_eq!("m/48'/1'/0'/2'", key_source.path().to_string());

        // let secp = Secp256k1::new();
        // let xpub = ExtendedPubKey::from_priv(&secp, key_source.key());
        // assert_eq!(format!("{}", xpub), expected_xpub);
    }

    // pub fn key(mnemonic_or_codex32: &str, path: &DerivationPath, network: Network) -> String {
    //     let secp = Secp256k1::new();
    //     let mnemonic = match Mnemonic::from_str(mnemonic_or_codex32) {
    //         Ok(mnemonic) => mnemonic,
    //         Err(_) => {
    //             let codex32 = Codex32String::from_string(mnemonic_or_codex32.to_string())
    //                 .map_err(Codex32ErrorWrapper)
    //                 .unwrap();
    //             let entropy = codex32.parts().data();
    //             Mnemonic::from_entropy(&entropy).unwrap()
    //         }
    //     };
    //     let xprv = ExtendedPrivKey::new_master(network, &mnemonic.to_seed("")).unwrap();
    //     let fingerprint = xprv.fingerprint(&secp);
    //     let derived = xprv.derive_priv(&secp, path).unwrap();
    //     let path = format!("{path}");
    //     let path = &path[1..];
    //     let result = format!("[{fingerprint}{path}]{derived}");
    //     result
    // }
}
