use crate::client_ext::ClientExt;
use crate::core_connect::CoreConnect;
use bitcoin::secp256k1::Secp256k1;
use bitcoind::bitcoincore_rpc;
use bitcoind::bitcoincore_rpc::jsonrpc::serde_json;

const MULTIPATH: &str = "<0;1>";

#[derive(thiserror::Error, Debug)]
pub enum ImportError {
    #[error(transparent)]
    CoreRpc(#[from] bitcoincore_rpc::Error),

    #[error(transparent)]
    Any(#[from] anyhow::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Miniscript(#[from] miniscript::Error),

    #[error("Given descriptor isn't multipat, it doesn't contain <0;1>")]
    DescriptorIsntMultipath,

    #[error("With private key flags used but descriptors don't contain private keys")]
    WithPrivateKeyFlagButPublicDescriptor,

    #[error("Without private key flags used but descriptors contain private keys")]
    WithoutPrivateKeyFlagButSecretDescriptor,

    #[error("Without private key flags used but descriptors contain private keys")]
    CannotImport,
}

/// Import descriptors into bitcoin core, note we are explicitly using string because multipath secret descriptor are not supported
pub fn import(
    core_connect: &CoreConnect,
    desc: &str,
    wallet_name: &str,
    with_private_keys: bool,
) -> Result<String, ImportError> {
    let client = core_connect.client()?;

    let ExplodedDesc { internal, external } = explode_descriptor(desc, with_private_keys)?;
    let internal = client.add_checksum(&internal)?;
    let external = client.add_checksum(&external)?;

    let client = client.create_blank_wallet(
        wallet_name,
        core_connect,
        !with_private_keys,
        with_private_keys.then(|| desc),
    )?;
    if with_private_keys {
        client.wallet_passphrase(&desc)
    }

    let r1 = client.import_descriptor(&external, false)?;
    let r2 = client.import_descriptor(&internal, true)?;

    if !r1.success || !r2.success {
        return Err(ImportError::CannotImport);
    }

    Ok("ok".to_string())
}

pub(crate) struct ExplodedDesc {
    /// change /1/
    internal: String,

    /// /0/
    external: String,
}
/// from a descriptor with the "<0;1>" notation creates 2 descriptors, one with 0 and the other with 1
/// This shouldn't be used outside of this module, here is used because multipath secret descriptor
/// aren't supported in rust-miniscript
pub(crate) fn explode_descriptor(
    desc: &str,
    with_private_keys: bool,
) -> Result<ExplodedDesc, ImportError> {
    if !desc.contains(MULTIPATH) {
        return Err(ImportError::DescriptorIsntMultipath);
    }
    let (external, internal) = (desc.replace(MULTIPATH, "0"), desc.replace(MULTIPATH, "1"));
    let secp = Secp256k1::new();
    let (_, key_map) = miniscript::Descriptor::parse_descriptor(&secp, &external)?; // I can check only external or internal since they are the same but the change index

    if with_private_keys && key_map.is_empty() {
        return Err(ImportError::WithPrivateKeyFlagButPublicDescriptor);
    }
    if !with_private_keys && !key_map.is_empty() {
        return Err(ImportError::WithoutPrivateKeyFlagButSecretDescriptor);
    }

    Ok(ExplodedDesc { internal, external })
}

#[cfg(test)]
mod test {
    use crate::{
        commands::{self},
        test_util::TestNode,
        Descriptor,
    };
    use bitcoind::bitcoincore_rpc::RpcApi;

    #[test]
    fn test_import() {
        let TestNode {
            node, core_connect, ..
        } = crate::test_util::setup_node();
        let desc = "tr([8335dcdb/48'/1'/0'/2']tprv8ifUoGVh57yDBkyW2sS6kMNv7ewZVLmSLp1RSgZw4H5AhMP6AtxJB1P842vZcvdu9giYEfWDa6NX5nCGaaUVK5boJt1AeA8fFKv2u87Ua3g/<0;1>/*)";

        let _ = commands::import(&core_connect, desc, "1", true).unwrap();
        let _ = commands::import(&core_connect, desc, "2", false).unwrap_err();

        let desc: &str = "tr([8335dcdb/48'/1'/0'/2']tpubDFMWwgXwDVet5E1HvX6h9m32ggTVefxLv7cCjCcEUYsZXqdroHmtMVzzE9RcbwgWa5rCXnZqFXxtKvH7JB5JkTgsNdYdgc1nWJFXHj26ux1/<0;1>/*)";
        let _ = commands::import(&core_connect, desc, "3", false).unwrap();
        let _ = commands::import(&core_connect, desc, "4", true).unwrap_err();

        let wallets = node.client.list_wallets().unwrap();

        assert!(wallets.contains(&"1".to_owned()));
        assert!(!wallets.contains(&"2".to_owned()));
        assert!(wallets.contains(&"3".to_owned()));
        assert!(!wallets.contains(&"4".to_owned()));
    }

    #[test]
    fn test_multipath() {
        let desc: &str = "tr([8335dcdb/48'/1'/0'/2']tpubDFMWwgXwDVet5E1HvX6h9m32ggTVefxLv7cCjCcEUYsZXqdroHmtMVzzE9RcbwgWa5rCXnZqFXxtKvH7JB5JkTgsNdYdgc1nWJFXHj26ux1/<0;1>/*)";

        let desc: Descriptor = desc.parse().unwrap();

        let descriptors = desc.into_single_descriptors().unwrap();

        assert_eq!(descriptors.len(), 2);
        assert_eq!(descriptors[0].to_string(), "tr([8335dcdb/48'/1'/0'/2']tpubDFMWwgXwDVet5E1HvX6h9m32ggTVefxLv7cCjCcEUYsZXqdroHmtMVzzE9RcbwgWa5rCXnZqFXxtKvH7JB5JkTgsNdYdgc1nWJFXHj26ux1/0/*)#0ppqg948");
        assert_eq!(descriptors[1].to_string(), "tr([8335dcdb/48'/1'/0'/2']tpubDFMWwgXwDVet5E1HvX6h9m32ggTVefxLv7cCjCcEUYsZXqdroHmtMVzzE9RcbwgWa5rCXnZqFXxtKvH7JB5JkTgsNdYdgc1nWJFXHj26ux1/1/*)#74yp4s9l");
    }
}
