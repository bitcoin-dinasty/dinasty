use crate::{create_random_extended, setup};
use bitcoin::{
    address::NetworkUnchecked, bip32::ExtendedPubKey, psbt::PartiallySignedTransaction,
    secp256k1::Secp256k1, Address, Amount, Network,
};
use bitcoind::bitcoincore_rpc::{
    core_rpc_json::{CreateRawTransactionInput, WalletCreateFundedPsbtOptions},
    RpcApi,
};
use dinasty::client_ext::ClientExt;
use std::{collections::HashMap, str::FromStr};

/// A wallet creates a transactions with a future nlocktime, thus is available to be spend only
/// after `nlocktime` blocks have been passed.
#[test]
fn nlocktime() {
    let (node, node_address, core_connect) = setup();
    let secp = Secp256k1::new();

    let locktime = 500;
    let alice_xprv = create_random_extended(Network::Regtest);
    let alice_xpub = ExtendedPubKey::from_priv(&secp, &alice_xprv);
    let alice_wallet = format!("tr({alice_xprv}/0/*)");
    let alice_wallet = node.client.add_checksum(&alice_wallet).unwrap();
    let alice_wallet_change = format!("tr({alice_xprv}/1/*)");
    let alice_wallet_change = node.client.add_checksum(&alice_wallet_change).unwrap();
    let alice_client = node
        .client
        .create_blank_wallet("alice", &core_connect, false, None)
        .unwrap();
    alice_client
        .import_descriptor(&alice_wallet, false)
        .unwrap();
    alice_client
        .import_descriptor(&alice_wallet_change, true)
        .unwrap();
    let alice_address = alice_client
        .get_new_bech32m_address(Network::Regtest)
        .unwrap();

    let alice_wallet_wo = format!("tr({alice_xpub}/0/*)");
    let alice_wallet_wo = node.client.add_checksum(&alice_wallet_wo).unwrap();
    let alice_wallet_wo_change = format!("tr({alice_xpub}/1/*)");
    let alice_wallet_wo_change = node.client.add_checksum(&alice_wallet_wo_change).unwrap();
    let alice_wo_client = node
        .client
        .create_blank_wallet("alice_wo", &core_connect, true, None)
        .unwrap();
    alice_wo_client
        .import_descriptor(&alice_wallet_wo, false)
        .unwrap();
    alice_wo_client
        .import_descriptor(&alice_wallet_wo_change, true)
        .unwrap();
    let alice_wo_address = alice_wo_client
        .get_new_bech32m_address(Network::Regtest)
        .unwrap();

    assert_eq!(alice_address, alice_wo_address);

    node.client.generate_to_address(1, &alice_address).unwrap();
    node.client.generate_to_address(100, &node_address).unwrap(); // enough to be spendable, not enought for CSV

    let unspents = alice_wo_client
        .list_unspent(None, None, None, None, None)
        .unwrap();

    let unspent = &unspents[0];
    let input = CreateRawTransactionInput {
        txid: unspent.txid,
        vout: unspent.vout,
        sequence: None,
    };

    let mut outputs = HashMap::new();
    outputs.insert(node_address.to_string(), unspent.amount);

    let mut options = WalletCreateFundedPsbtOptions::default();
    options.subtract_fee_from_outputs = vec![0];
    let psbt = alice_wo_client
        .wallet_create_funded_psbt(&[input], &outputs, Some(locktime), Some(options), None)
        .unwrap();
    let t = PartiallySignedTransaction::from_str(&psbt.psbt).unwrap();
    println!("{:?}", &t);

    assert_eq!(psbt.psbt.len(), 296);

    let psbt = alice_client
        .wallet_process_psbt(&psbt.psbt, None, None, None)
        .unwrap();
    assert!(psbt.complete);
    assert_eq!(psbt.psbt.len(), 280);
    let t = PartiallySignedTransaction::from_str(&psbt.psbt).unwrap();
    println!("{:?}", &t);

    let psbt = alice_client.finalize_psbt(&psbt.psbt, None).unwrap();
    assert_eq!(psbt.hex.as_ref().unwrap().len(), 150);

    let non_owned_address: Address<NetworkUnchecked> =
        "bcrt1q6jaw5xdxwf2wvmymjsgnypmea3eaap8m0jsmfk9u5wt43nfpy9gspg6ze0"
            .parse()
            .unwrap();
    let non_owned_address = non_owned_address.assume_checked();

    while alice_client.get_blockchain_info().unwrap().blocks < locktime as u64 {
        if let Ok(result) = alice_client.test_mempool_accept(&[psbt.hex.as_ref().unwrap()]) {
            assert!(!result[0].allowed);
            assert_eq!(result[0].reject_reason, Some("non-final".to_string()));
            node.client
                .generate_to_address(100, &non_owned_address)
                .unwrap();
        }
    }

    let result = alice_client
        .test_mempool_accept(&[psbt.hex.as_ref().unwrap()])
        .unwrap();
    assert!(result[0].allowed);
    assert_eq!(result[0].fees.as_ref().unwrap().base, Amount::from_sat(990));

    let txid = alice_client
        .send_raw_transaction(psbt.hex.as_ref().unwrap())
        .unwrap();
    let tx = alice_client
        .get_transaction(&txid, None)
        .unwrap()
        .transaction()
        .unwrap();
    assert_eq!(tx.input.len(), 1);
    assert_eq!(tx.output.len(), 1);
}
