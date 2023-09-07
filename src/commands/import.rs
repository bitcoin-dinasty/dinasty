use crate::client_ext::ClientExt;
use crate::core_connect::{self, CoreConnect};
use bitcoind::bitcoincore_rpc;
use bitcoind::bitcoincore_rpc::jsonrpc::serde_json;

const MULTIPATH: &str = "<0;1>";

#[derive(thiserror::Error, Debug)]
pub enum ImportError {
    #[error(transparent)]
    CoreRpc(#[from] bitcoincore_rpc::Error),

    #[error(transparent)]
    CoreConnect(#[from] core_connect::ConnectError),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

pub fn import(
    core_connect: &CoreConnect,
    desc: &str,
    wallet_name: &str,
    with_private_keys: bool,
) -> Result<String, ImportError> {
    let client = core_connect.client()?;

    let client = client.create_blank_wallet(
        wallet_name,
        core_connect,
        !with_private_keys,
        with_private_keys.then(|| desc),
    )?;
    if with_private_keys {
        client.wallet_passphrase(&desc)
    }

    let (external, internal) = explode_descriptor(desc);
    let external = client.add_checksum(&external)?;
    let internal = client.add_checksum(&internal)?;

    let mut r1 = client.import_descriptor(&external, false)?;
    let r2 = client.import_descriptor(&internal, true)?;

    r1.extend(r2);
    Ok(serde_json::to_string(&r1)?)
}

/// from a descriptor with the "<0;1>" notation creates 2 descriptors, one with 0 and the other with 1
pub fn explode_descriptor(desc: &str) -> (String, String) {
    (desc.replace(MULTIPATH, "0"), desc.replace(MULTIPATH, "1"))
}

#[cfg(test)]
mod test {
    use super::{explode_descriptor, MULTIPATH};
    use crate::{commands, test_util::TestNode};
    use bitcoind::bitcoincore_rpc::RpcApi;

    #[test]
    fn test_explode_descriptor() {
        let descriptors = explode_descriptor(MULTIPATH);
        assert_eq!(&descriptors.0, "0");
        assert_eq!(&descriptors.1, "1");

        let mut multi = MULTIPATH.to_string();
        multi.push_str(MULTIPATH);

        let descriptors = explode_descriptor(&multi);
        assert_eq!(&descriptors.0, "00");
        assert_eq!(&descriptors.1, "11");
    }

    #[test]
    fn test_import() {
        let TestNode {
            node, core_connect, ..
        } = crate::test_util::setup_node();
        let desc = "tr([8335dcdb/48'/1'/0'/2']tprv8ifUoGVh57yDBkyW2sS6kMNv7ewZVLmSLp1RSgZw4H5AhMP6AtxJB1P842vZcvdu9giYEfWDa6NX5nCGaaUVK5boJt1AeA8fFKv2u87Ua3g/<0;1>/*)";

        let _ = commands::import(&core_connect, desc, "wallet_name", false).unwrap();

        assert!(node
            .client
            .list_wallets()
            .unwrap()
            .iter()
            .any(|f| f == "wallet_name"));
    }
}
