use std::{collections::HashMap, str::FromStr};

use bitcoin::{
    psbt::{PartiallySignedTransaction, PsbtParseError},
    OutPoint,
};
use bitcoind::bitcoincore_rpc::{
    self,
    core_rpc_json::{CreateRawTransactionInput, WalletCreateFundedPsbtOptions},
    RpcApi,
};

use crate::{
    client_ext::ClientExt,
    core_connect::{self, CoreConnect},
};

#[derive(thiserror::Error, Debug)]
pub enum RefreshError {
    #[error(transparent)]
    CoreRpc(#[from] bitcoincore_rpc::Error),

    #[error(transparent)]
    CoreConnect(#[from] core_connect::ConnectError),

    #[error(transparent)]
    Psbt(#[from] PsbtParseError),
}

pub fn refresh(
    core_connect: &CoreConnect,
    wallet_name: &str,
    older_than_blocks: u32,
) -> Result<Vec<PartiallySignedTransaction>, RefreshError> {
    let client = core_connect.client_with_wallet(wallet_name)?;

    let unspent = client.list_unspent(None, None, None, None, None)?;
    let mut result = vec![];

    for old_unspent in unspent
        .iter()
        .filter(|u| u.confirmations > older_than_blocks)
    {
        let input = CreateRawTransactionInput {
            txid: old_unspent.txid,
            vout: old_unspent.vout,
            sequence: None,
        };
        let my_address = client
            .get_new_bech32m_address(core_connect.network)?
            .to_string();

        let mut outputs = HashMap::new();
        outputs.insert(my_address.clone(), old_unspent.amount);

        let options = WalletCreateFundedPsbtOptions {
            subtract_fee_from_outputs: vec![0],
            ..Default::default()
        };

        let psbt = client
            .wallet_create_funded_psbt(&[input], &outputs, None, Some(options), None)
            .unwrap();
        let t = PartiallySignedTransaction::from_str(&psbt.psbt)?;
        let signed = !t.inputs[0].partial_sigs.is_empty();

        let outpoint = OutPoint::new(old_unspent.txid, old_unspent.vout);
        let fee = psbt.fee;
        let amount = old_unspent.amount;
        let confirmations = old_unspent.confirmations;
        log::info!(
            "{outpoint} conf:{confirmations} signed:{signed} {amount} -> {my_address} fee:{fee}"
        );

        result.push(t);
    }

    Ok(result)
}

#[cfg(test)]
mod test {

    use bitcoin::{Address, Network};
    use bitcoind::bitcoincore_rpc::RpcApi;

    use crate::{
        client_ext::ClientExt,
        commands::{self, refresh},
        test_util::TestNode,
    };

    #[test]
    fn test_refresh() {
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
        node.client.generate_to_address(10, &node_address).unwrap();
        node.client
            .generate_to_address(
                1,
                &wo_client.get_new_bech32m_address(Network::Regtest).unwrap(),
            )
            .unwrap();
        node.client.generate_to_address(100, &node_address).unwrap();

        let mut res = refresh(&core_connect, "wo", 105).unwrap();
        assert_eq!(res.len(), 1);
        let tx = res.pop().unwrap().extract_tx();

        let output_addr =
            Address::from_script(&tx.output[0].script_pubkey, Network::Regtest).unwrap();
        let info = wo_client.get_address_info(&output_addr).unwrap();
        assert_eq!(info.is_mine, Some(true));
    }
}
