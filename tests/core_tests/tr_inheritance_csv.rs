use std::collections::HashMap;

use bitcoin::{
    address::NetworkUnchecked, bip32::ExtendedPubKey, secp256k1::Secp256k1, Address, Amount,
    Network,
};
use bitcoind::bitcoincore_rpc::{core_rpc_json::CreateRawTransactionInput, RpcApi};
use dinasty::client_ext::ClientExt;

use crate::{create_random_extended, setup};

/// Two keys, Owner and Heir
/// The Owner can spend with their key, the Heir can spend with their key only after `block_wait`
/// blocks have passed after the coin has been created.
/// requires bitcoin compiled with https://github.com/bitcoin/bitcoin/pull/27255 because it uses tapscript
#[ignore]
#[test]
fn tr_inheritance() {
    let (node, node_address, core_connect) = setup();
    let secp = Secp256k1::new();
    let block_wait = 1000;

    assert_eq!(
        node.client.get_network_info().unwrap().version,
        259900,
        "requires tapscript"
    );

    // setup Owner and Heir key
    let owner_xprv = create_random_extended(Network::Regtest);
    let owner_xpub = ExtendedPubKey::from_priv(&secp, &owner_xprv);
    let heir_xprv = create_random_extended(Network::Regtest);
    let heir_xpub = ExtendedPubKey::from_priv(&secp, &heir_xprv);

    // setup Owner wallet
    let owner_desc =
        format!("tr({owner_xprv}/0/*,and_v(v:pk({heir_xpub}/0/*),older({block_wait})))");
    let owner_desc = node.client.add_checksum(&owner_desc).unwrap();
    let owner_desc_change =
        format!("tr({owner_xprv}/1/*,and_v(v:pk({heir_xpub}/1/*),older({block_wait})))");
    let owner_desc_change = node.client.add_checksum(&owner_desc_change).unwrap();
    let owner_client = node
        .client
        .create_blank_wallet("owner", &core_connect, false, None)
        .unwrap();
    owner_client.import_descriptor(&owner_desc, false).unwrap();
    owner_client
        .import_descriptor(&owner_desc_change, true)
        .unwrap();
    let first_owner = owner_client
        .get_new_bech32m_address(Network::Regtest)
        .unwrap();

    // setup Heir wallet
    let heir_desc =
        format!("tr({owner_xpub}/0/*,and_v(v:pk({heir_xprv}/0/*),older({block_wait})))");
    let heir_desc = node.client.add_checksum(&heir_desc).unwrap();
    let heir_desc_change =
        format!("tr({owner_xpub}/1/*,and_v(v:pk({heir_xprv}/1/*),older({block_wait})))");
    let heir_desc_change = node.client.add_checksum(&heir_desc_change).unwrap();
    let heir_client = node
        .client
        .create_blank_wallet("heir", &core_connect, false, None)
        .unwrap();
    heir_client.import_descriptor(&heir_desc, false).unwrap();
    heir_client
        .import_descriptor(&heir_desc_change, true)
        .unwrap();
    let first_heir = heir_client
        .get_new_bech32m_address(Network::Regtest)
        .unwrap();

    assert_eq!(first_owner, first_heir);
    let first = first_owner;

    node.client.generate_to_address(1, &first).unwrap();
    node.client.generate_to_address(100, &node_address).unwrap();

    // owner spending
    let mut outputs = HashMap::new();
    let sent_back = Amount::from_sat(100_000);
    outputs.insert(node_address.to_string(), sent_back);

    let psbt = owner_client
        .wallet_create_funded_psbt(&[], &outputs, None, None, None)
        .unwrap();
    let fee = psbt.fee;

    let psbt = owner_client
        .wallet_process_psbt(&psbt.psbt, None, None, None)
        .unwrap();

    let psbt = owner_client.finalize_psbt(&psbt.psbt, None).unwrap();
    assert!(psbt.complete);

    let result = owner_client
        .test_mempool_accept(&[psbt.hex.as_ref().unwrap()])
        .unwrap();
    assert!(result[0].allowed);

    owner_client
        .send_raw_transaction(psbt.hex.as_ref().unwrap())
        .unwrap();

    let balances = owner_client.get_balances().unwrap();

    assert_eq!(
        balances.mine.trusted,
        Amount::ONE_BTC * 50 - fee - sent_back
    );
    node.client.generate_to_address(1, &node_address).unwrap();

    // heir spending
    let mut outputs = HashMap::new();
    let sent_back = Amount::from_sat(100_001);
    outputs.insert(node_address.to_string(), sent_back);

    let unspents = heir_client
        .list_unspent(None, None, None, None, None)
        .unwrap();

    let unspent = &unspents[0];
    let input = CreateRawTransactionInput {
        txid: unspent.txid,
        vout: unspent.vout,
        sequence: Some(block_wait),
    };

    let psbt = heir_client
        .wallet_create_funded_psbt(&[input], &outputs, None, None, None)
        .unwrap();
    let fee = psbt.fee;

    let psbt = heir_client
        .wallet_process_psbt(&psbt.psbt, None, None, None)
        .unwrap();

    let psbt = heir_client.finalize_psbt(&psbt.psbt, None).unwrap();
    assert!(psbt.complete);

    let non_owned_address: Address<NetworkUnchecked> =
        "bcrt1q6jaw5xdxwf2wvmymjsgnypmea3eaap8m0jsmfk9u5wt43nfpy9gspg6ze0"
            .parse()
            .unwrap();
    let non_owned_address = non_owned_address.assume_checked();
    let mut remaining = block_wait as i64 - 1;
    while remaining > 0 {
        if let Ok(result) = heir_client.test_mempool_accept(&[psbt.hex.as_ref().unwrap()]) {
            assert!(!result[0].allowed);
            assert_eq!(result[0].reject_reason, Some("non-BIP68-final".to_string()));

            if let Ok(_) = node.client.generate_to_address(100, &non_owned_address) {
                remaining -= 100;
            } // make the CSV expire
        }
    }

    let result = heir_client
        .test_mempool_accept(&[psbt.hex.as_ref().unwrap()])
        .unwrap();
    assert!(result[0].allowed);

    heir_client
        .send_raw_transaction(psbt.hex.as_ref().unwrap())
        .unwrap();

    let new_balances = heir_client.get_balances().unwrap();

    assert_eq!(
        new_balances.mine.trusted,
        balances.mine.trusted - fee - sent_back
    );
}
