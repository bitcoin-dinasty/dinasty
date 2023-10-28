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

use crate::{client_ext::ClientExt, core_connect::CoreConnect, Descriptor};

#[derive(thiserror::Error, Debug)]
pub enum LocktimeError {
    #[error(transparent)]
    CoreRpc(#[from] bitcoincore_rpc::Error),

    #[error(transparent)]
    Any(#[from] anyhow::Error),

    #[error(transparent)]
    Psbt(#[from] PsbtParseError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Coonversion(#[from] miniscript::descriptor::ConversionError),

    #[error(transparent)]
    Miniscript(#[from] miniscript::Error),

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
    from_wallet_name: &str,
    to_wallet_name: &str,
    locktime_future: i64,
) -> Result<Vec<PartiallySignedTransaction>, LocktimeError> {
    let client_from = core_connect.client_with_wallet(from_wallet_name)?;
    let client_to = core_connect.client_with_wallet(to_wallet_name)?;
    let client_to_descriptors = client_to.list_descriptors(to_wallet_name)?;
    let client_to_external_desc: Vec<_> = client_to_descriptors
        .iter()
        .filter(|e| !e.internal)
        .map(|e| e.desc.to_string())
        .collect();
    assert!(client_to_external_desc.len() == 1);
    let client_to_descriptor = client_to_external_desc.first().unwrap();
    let client_to_descriptor: Descriptor = client_to_descriptor.parse()?;
    let mut vec = vec![];
    for i in 0..1_000 {
        // TODO same range as core
        let derived = client_to_descriptor.at_derivation_index(i)?;
        vec.push(derived.address(core_connect.network)?);
    }

    let list_unspent = client_from.list_unspent(None, None, None, None, None)?;
    let list_unspent_outpoints: HashSet<_> = list_unspent
        .iter()
        .map(|u| OutPoint::new(u.txid, u.vout))
        .collect();

    let mut result = vec![];

    let mut heir_addresses = vec.iter();
    for unspent in list_unspent {
        let output_address = loop {
            // if the address is already mapped to a still unspent outpoint use another
            let current = heir_addresses
                .next()
                .ok_or(LocktimeError::NotEnoughAddresses)?;
            let info = client_from.get_address_info(current)?;
            let receiver_info = client_to.get_address_info(current)?;
            assert!(receiver_info.is_mine.unwrap_or(false));
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

        let blockchain_info = client_from.get_blockchain_info()?;
        if blockchain_info.initial_block_download == true {
            return Err(LocktimeError::StillIBD);
        }
        let locktime = blockchain_info.blocks as i64 + locktime_future;

        let psbt = client_from
            .wallet_create_funded_psbt(&[input], &outputs, Some(locktime), Some(options), None)
            .unwrap();
        let t = PartiallySignedTransaction::from_str(&psbt.psbt)?;

        let outpoint = OutPoint::new(unspent.txid, unspent.vout);
        client_from.set_label(output_address, &outpoint.to_string())?;

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

        let heir_wo_desc = "tr([01e0b4da/1']tpubD8GvnJ7jbLd3ZCmUUoTwDMpQ5N7sVv2HjW4sBgBss7zeEm8mPPSxDmDxYy4rxGZbQAcbRGwawzXMUpnLAnHcrNmZcqucy3qAyn7NZzKChpx/<0;1>/*)";

        commands::import(&core_connect, owner_wo_desc, "wo", false).unwrap();
        commands::import(&core_connect, owner_desc, "signer", true).unwrap();

        commands::import(&core_connect, heir_wo_desc, "heir", false).unwrap();

        let wo_client = core_connect.client_with_wallet("wo").unwrap();
        let signer_client = core_connect.client_with_wallet("signer").unwrap();

        let first = wo_client.get_new_bech32m_address(Network::Regtest).unwrap();
        let first_same = signer_client
            .get_new_bech32m_address(Network::Regtest)
            .unwrap();

        assert_eq!(first, first_same);

        node.client.generate_to_address(1, &first).unwrap();
        node.client.generate_to_address(100, &node_address).unwrap();

        let psbts = commands::locktime(&core_connect, "wo", "heir", 500).unwrap();

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
