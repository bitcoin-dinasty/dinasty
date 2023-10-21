use std::str::FromStr;

use bitcoin::psbt::{PartiallySignedTransaction, PsbtParseError};
use bitcoind::bitcoincore_rpc::{self, RpcApi};

use crate::{client_ext::ClientExt, core_connect::CoreConnect};

#[derive(thiserror::Error, Debug)]
pub enum SignError {
    #[error(transparent)]
    CoreRpc(#[from] bitcoincore_rpc::Error),

    #[error(transparent)]
    Psbt(#[from] PsbtParseError),

    #[error(transparent)]
    Any(#[from] anyhow::Error),
}

pub fn sign(
    core_connect: &CoreConnect,
    descriptor: &str,
    wallet_name: &str,
    psbts: &[PartiallySignedTransaction],
) -> Result<Vec<PartiallySignedTransaction>, SignError> {
    let client = core_connect.client_with_wallet(wallet_name)?;
    client.wallet_passphrase(descriptor);

    let mut results = vec![];
    for psbt in psbts {
        let result = client.wallet_process_psbt(&psbt.to_string(), None, None, None)?;
        let signed_psbt = PartiallySignedTransaction::from_str(&result.psbt)?;

        let changed = psbt != &signed_psbt;
        log::info!("changed:{changed}");

        results.push(signed_psbt);
    }
    client.wallet_lock();
    Ok(results)
}

#[cfg(test)]
mod test {

    use bitcoin::Network;
    use bitcoind::bitcoincore_rpc::RpcApi;

    use crate::{
        client_ext::ClientExt,
        commands::{self, refresh, sign},
        test_util::TestNode,
    };

    #[test]
    fn test_sign() {
        let TestNode {
            node,
            node_address,
            core_connect,
            ..
        } = crate::test_util::setup_node();

        let xprv_desc = "tr([8335dcdb/48'/1'/0'/2']tprv8ifUoGVh57yDBkyW2sS6kMNv7ewZVLmSLp1RSgZw4H5AhMP6AtxJB1P842vZcvdu9giYEfWDa6NX5nCGaaUVK5boJt1AeA8fFKv2u87Ua3g/<0;1>/*)";
        let xpub_desc = "tr([8335dcdb/48'/1'/0'/2']tpubDFMWwgXwDVet5E1HvX6h9m32ggTVefxLv7cCjCcEUYsZXqdroHmtMVzzE9RcbwgWa5rCXnZqFXxtKvH7JB5JkTgsNdYdgc1nWJFXHj26ux1/<0;1>/*)";

        commands::import(&core_connect, xpub_desc, "wo", false).unwrap();
        commands::import(&core_connect, xprv_desc, "signer", true).unwrap();

        let wo_client = core_connect.client_with_wallet("wo").unwrap();
        let signer_client = core_connect.client_with_wallet("signer").unwrap();

        let first = wo_client.get_new_bech32m_address(Network::Regtest).unwrap();
        let first_same = signer_client
            .get_new_bech32m_address(Network::Regtest)
            .unwrap();

        assert_eq!(first, first_same);

        node.client.generate_to_address(1, &first).unwrap();
        node.client.generate_to_address(100, &node_address).unwrap();

        let psbts = refresh(&core_connect, "wo", 5).unwrap();
        assert_eq!(psbts.len(), 1);
        let tx = psbts[0].clone().extract_tx();

        let result = node.client.test_mempool_accept(&[&tx]).unwrap();
        assert!(!result[0].allowed);

        let signed_psbts = sign(&core_connect, xprv_desc, "signer", &psbts).unwrap();

        let tx = signed_psbts[0].clone().extract_tx();

        let result = node.client.test_mempool_accept(&[&tx]).unwrap();
        assert!(result[0].allowed);
    }
}
