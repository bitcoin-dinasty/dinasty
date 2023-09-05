use bitcoin::{bip32::ExtendedPubKey, secp256k1::Secp256k1, Network};
use bitcoind::bitcoincore_rpc::RpcApi;
use dinasty::client_ext::ClientExt;

use crate::{create_random_extended, setup};

/// Two owners creates a taproot shared wallet, one can spend only with taproot keyspend, the other
/// only with the scriptspend
#[test]
fn taproot_keyspend_and_scriptspend() {
    let (node, node_address, core_connect) = setup();
    let secp = Secp256k1::new();

    let alice_prv = create_random_extended(Network::Regtest);
    let alice_pub = ExtendedPubKey::from_priv(&secp, &alice_prv);
    let bob_prv = create_random_extended(Network::Regtest);
    let bob_pub = ExtendedPubKey::from_priv(&secp, &bob_prv);

    let alice_wallet = format!("tr({alice_prv}/*,pk({bob_pub}))");
    let alice_wallet = node.client.add_checksum(&alice_wallet).unwrap();
    let bob_wallet = format!("tr({alice_pub}/*,pk({bob_prv}))");
    let bob_wallet = node.client.add_checksum(&bob_wallet).unwrap();

    assert_ne!(alice_wallet, bob_wallet);

    let alice_client = node
        .client
        .create_blank_wallet("alice", &core_connect, false, None)
        .unwrap();
    let bob_client = node
        .client
        .create_blank_wallet("bob", &core_connect, false, None)
        .unwrap();

    alice_client
        .import_descriptor(&alice_wallet, false)
        .unwrap();

    let first = alice_client
        .get_new_bech32m_address(Network::Regtest)
        .unwrap();
    let second = alice_client
        .get_new_bech32m_address(Network::Regtest)
        .unwrap();

    bob_client.import_descriptor(&bob_wallet, false).unwrap();

    let first_same = bob_client
        .get_new_bech32m_address(Network::Regtest)
        .unwrap();
    let second_same = bob_client
        .get_new_bech32m_address(Network::Regtest)
        .unwrap();

    assert_eq!(first, first_same);
    assert_eq!(second, second_same);

    node.client.generate_to_address(1, &first).unwrap();
    node.client.generate_to_address(100, &node_address).unwrap();

    // using send all otherwise without internal descriptor would fail creating a change address
    let txid = alice_client.send_all(&second);

    let tx = alice_client.get_transaction(&txid, None).unwrap();
    assert_eq!(
        tx.transaction().unwrap().input[0].witness.to_vec().len(),
        1,
        "keyspend has one element in witness"
    );

    assert_eq!(tx.transaction().unwrap().weight().to_wu() / 4, 111);

    node.client.generate_to_address(1, &second).unwrap();
    node.client.generate_to_address(100, &node_address).unwrap();

    let txid = bob_client.send_all(&node_address);
    let tx = bob_client.get_transaction(&txid, None).unwrap();
    assert_eq!(
        tx.transaction().unwrap().input[0].witness.to_vec().len(),
        3,
        "scriptspend has 3 elements in witness"
    );
}
