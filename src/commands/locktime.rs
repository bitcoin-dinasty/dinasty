use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use bitcoin::{
    psbt::{PartiallySignedTransaction, PsbtParseError},
    Network, OutPoint,
};
use bitcoind::bitcoincore_rpc::{
    self,
    core_rpc_json::{
        CreateRawTransactionInput, GetAddressInfoResultLabel, WalletCreateFundedPsbtOptions,
    },
    RpcApi,
};

use crate::core_connect::{self, CoreConnect};

#[derive(thiserror::Error, Debug)]
pub enum LocktimeError {
    #[error(transparent)]
    CoreRpc(#[from] bitcoincore_rpc::Error),

    #[error(transparent)]
    CoreConnect(#[from] core_connect::ConnectError),

    #[error(transparent)]
    Psbt(#[from] PsbtParseError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Invalid network: descriptor:{descriptor} expected:{expected}")]
    DerivedAddressInvalidNetwork {
        descriptor: Network,
        expected: Network,
    },

    #[error("There are not enough generated heir addresses")]
    NotEnoughAddresses,

    #[error("The node is still downloading blocks (IBD)")]
    StillIBD,
}

pub fn locktime(
    core_connect: &CoreConnect,
    wallet_name: &str,
    heir_wo_descriptor: &str,
    locktime_future: i64,
) -> Result<Vec<PartiallySignedTransaction>, LocktimeError> {
    let client = core_connect.client_with_wallet(wallet_name)?;

    client.get_descriptor_info(heir_wo_descriptor)?; // validate heir descriptor

    let heir_addresses_unchecked: Vec<_> =
        client.derive_addresses(heir_wo_descriptor, Some([0, 1000]))?;
    let mut heir_addresses = vec![];

    for a in heir_addresses_unchecked {
        if a.is_valid_for_network(core_connect.network) {
            heir_addresses.push(a.assume_checked())
        } else {
            return Err(LocktimeError::DerivedAddressInvalidNetwork {
                descriptor: a.network,
                expected: core_connect.network,
            });
        }
    }
    let mut heir_addresses = heir_addresses.iter();

    let list_unspent = client.list_unspent(None, None, None, None, None)?;
    let list_unspent_outpoints: HashSet<_> = list_unspent
        .iter()
        .map(|u| OutPoint::new(u.txid, u.vout))
        .collect();

    let mut result = vec![];

    for unspent in list_unspent {
        let output_address = loop {
            // if the address is already mapped to a still unspent outpoint use another
            let current = heir_addresses
                .next()
                .ok_or(LocktimeError::NotEnoughAddresses)?;
            let info = client.get_address_info(current)?;
            if let Some(GetAddressInfoResultLabel::Simple(label)) = info.labels.first() {
                if let Ok(outpoint) = OutPoint::from_str(label) {
                    if list_unspent_outpoints.contains(&outpoint) {
                        continue;
                    }
                }
            }
            break current;
        };

        let input = CreateRawTransactionInput {
            txid: unspent.txid,
            vout: unspent.vout,
            sequence: None,
        };

        let mut outputs = HashMap::new();
        let amount = unspent.amount;
        outputs.insert(output_address.to_string(), amount);

        let options = WalletCreateFundedPsbtOptions {
            subtract_fee_from_outputs: vec![0],
            ..Default::default()
        };

        let blockchain_info = client.get_blockchain_info()?;
        if blockchain_info.initial_block_download == true {
            return Err(LocktimeError::StillIBD);
        }
        let locktime = blockchain_info.blocks as i64 + locktime_future;

        let psbt = client
            .wallet_create_funded_psbt(&[input], &outputs, Some(locktime), Some(options), None)
            .unwrap();
        let t = PartiallySignedTransaction::from_str(&psbt.psbt)?;

        let outpoint = OutPoint::new(unspent.txid, unspent.vout);
        client.set_label(output_address, &outpoint.to_string())?;

        let fee = psbt.fee;
        log::info!("{outpoint} {amount} -> {output_address} fee:{fee} locktime:{locktime_future}");

        result.push(t);
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use crate::{client_ext::ClientExt, commands, test_util::TestNode};
    use bitcoin::Network;
    use bitcoind::bitcoincore_rpc::RpcApi;

    #[test]
    fn test_locktime() {
        let TestNode {
            node,
            node_address,
            core_connect,
            ..
        } = crate::test_util::setup_node();
        let owner_desc = "tr([8335dcdb/48'/1'/0'/2']tprv8ifUoGVh57yDBkyW2sS6kMNv7ewZVLmSLp1RSgZw4H5AhMP6AtxJB1P842vZcvdu9giYEfWDa6NX5nCGaaUVK5boJt1AeA8fFKv2u87Ua3g/<0;1>/*)";

        let owner_wo_desc = "tr([8335dcdb/48'/1'/0'/2']tpubDFMWwgXwDVet5E1HvX6h9m32ggTVefxLv7cCjCcEUYsZXqdroHmtMVzzE9RcbwgWa5rCXnZqFXxtKvH7JB5JkTgsNdYdgc1nWJFXHj26ux1/<0;1>/*)";

        let heir_wo_desc = "tr([01e0b4da/1']tpubD8GvnJ7jbLd3ZCmUUoTwDMpQ5N7sVv2HjW4sBgBss7zeEm8mPPSxDmDxYy4rxGZbQAcbRGwawzXMUpnLAnHcrNmZcqucy3qAyn7NZzKChpx/0/*)";
        let heir_wo_desc = node.client.add_checksum(heir_wo_desc).unwrap();

        commands::import(&core_connect, owner_wo_desc, "wo", false).unwrap();
        commands::import(&core_connect, owner_desc, "signer", true).unwrap();

        let wo_client = core_connect.client_with_wallet("wo").unwrap();
        let signer_client = core_connect.client_with_wallet("signer").unwrap();

        let first = wo_client.get_new_bech32m_address(Network::Regtest).unwrap();
        let first_same = signer_client
            .get_new_bech32m_address(Network::Regtest)
            .unwrap();

        assert_eq!(first, first_same);

        node.client.generate_to_address(1, &first).unwrap();
        node.client.generate_to_address(100, &node_address).unwrap();

        let psbts = commands::locktime(&core_connect, "wo", &heir_wo_desc, 500).unwrap();

        node.client.generate_to_address(450, &node_address).unwrap();

        let signed_psbts = commands::sign(&core_connect, owner_desc, "signer", &psbts).unwrap();
        let accepted = node
            .client
            .test_mempool_accept_psbts(&signed_psbts)
            .unwrap();
        assert!(
            accepted.into_iter().all(|t| !t),
            "even if signed, locktime isn't expired yet"
        );

        node.client.generate_to_address(50, &node_address).unwrap();
        let accepted = node.client.test_mempool_accept_psbts(&psbts).unwrap();
        assert!(
            accepted.into_iter().all(|t| !t),
            "locktime expired but not signed"
        );

        let signed_psbts = commands::sign(&core_connect, owner_desc, "signer", &psbts).unwrap();
        let accepted = node
            .client
            .test_mempool_accept_psbts(&signed_psbts)
            .unwrap();
        assert!(
            accepted.into_iter().all(|t| t),
            "signed and locktime expired"
        );
    }
}
