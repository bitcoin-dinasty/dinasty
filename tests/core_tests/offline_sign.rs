use crate::{create_random_extended, setup};
use bitcoin::{bip32::ExtendedPubKey, secp256k1::Secp256k1, Amount, Network};
use bitcoind::bitcoincore_rpc::RpcApi;
use dinasty::client_ext::ClientExt;
use std::collections::HashMap;

/// two nodes. One synced with watch-only, the other offline with private keys
#[test]
fn offline_sign() {
    let (node, node_address, core_connect) = setup();
    let (offline_node, _node_address, core_connect_offline) = setup();
    let secp = Secp256k1::new();

    let xprv = create_random_extended(Network::Regtest);
    let xpub = ExtendedPubKey::from_priv(&secp, &xprv);

    // setup online watch-only wallet
    let wo_desc = format!("tr({xpub}/0/*)");
    println!("{wo_desc}");
    let wo_desc = node.client.add_checksum(&wo_desc).unwrap();
    let wo_desc_change = format!("tr({xpub}/1/*)");
    println!("{wo_desc_change}");

    let wo_desc_change = node.client.add_checksum(&wo_desc_change).unwrap();
    let wo_client = node
        .client
        .create_blank_wallet("alice", &core_connect, true, None)
        .unwrap();
    wo_client.import_descriptor(&wo_desc, false).unwrap();
    wo_client.import_descriptor(&wo_desc_change, true).unwrap();
    let first_wo = wo_client.get_new_bech32m_address(Network::Regtest).unwrap();

    // setup offline signer wallet
    let offline_desc = format!("tr({xprv}/0/*)");
    let offline_desc = offline_node.client.add_checksum(&offline_desc).unwrap();
    let offline_desc_change = format!("tr({xprv}/1/*)");
    let offline_desc_change = offline_node
        .client
        .add_checksum(&offline_desc_change)
        .unwrap();
    let offline_client = offline_node
        .client
        .create_blank_wallet("alice", &core_connect_offline, false, None)
        .unwrap();
    offline_client
        .import_descriptor(&offline_desc, false)
        .unwrap();
    offline_client
        .import_descriptor(&offline_desc_change, true)
        .unwrap();
    let first_offline = offline_client
        .get_new_bech32m_address(Network::Regtest)
        .unwrap();

    assert_eq!(first_offline, first_wo);

    node.client.generate_to_address(1, &first_wo).unwrap();
    node.client.generate_to_address(100, &node_address).unwrap();

    let mut outputs = HashMap::new();
    let sent_back = Amount::from_sat(100_000);
    outputs.insert(node_address.to_string(), sent_back);

    let psbt = wo_client
        .wallet_create_funded_psbt(&[], &outputs, None, None, Some(true))
        .unwrap();
    let fee = psbt.fee;
    println!("{}", psbt.psbt);

    let psbt = offline_client
        .wallet_process_psbt(&psbt.psbt, None, None, Some(true))
        .unwrap();
    println!("{}", psbt.psbt);

    let psbt = wo_client.finalize_psbt(&psbt.psbt, None).unwrap();
    assert!(psbt.complete);

    let result = wo_client
        .test_mempool_accept(&[psbt.hex.as_ref().unwrap()])
        .unwrap();
    assert!(result[0].allowed);

    wo_client
        .send_raw_transaction(psbt.hex.as_ref().unwrap())
        .unwrap();

    let balances = wo_client.get_balances().unwrap();

    assert_eq!(
        balances.mine.trusted,
        Amount::ONE_BTC * 50 - fee - sent_back
    );
}
