use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use bitcoin::{psbt::PartiallySignedTransaction, Amount, ScriptBuf, Txid};
use miniscript::{descriptor::ConversionError, Descriptor, DescriptorPublicKey};

#[derive(Debug, thiserror::Error)]
pub enum BalanceError {
    #[error("Todo")]
    Todo,

    #[error(transparent)]
    Coonversion(#[from] miniscript::descriptor::ConversionError),

    #[error("MergeSame")]
    MergeSame,

    #[error("SameTxidDifferentFee")]
    SameTxidDifferentFee,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Balance {
    txid: Txid,
    descriptor: String,
    outgoing: Amount,
    incoming: Amount,
    fee: Amount,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Balances {
    txids_descriptors: HashMap<Txid, Vec<String>>,
    outgoing: Amount,
    incoming: Amount,
    fee: Amount,
    // TODO: add external addresses paid?
}

impl Balances {
    pub fn merge(&mut self, other: Balance) -> Result<(), BalanceError> {
        match self.txids_descriptors.get_mut(&other.txid) {
            Some(descriptors) => {
                if descriptors.contains(&other.descriptor) {
                    return Err(BalanceError::MergeSame);
                }
                self.outgoing += other.outgoing;
                self.incoming += other.incoming;
                descriptors.push(other.descriptor);
            }
            None => {
                self.outgoing += other.outgoing;
                self.incoming += other.incoming;
                self.fee += other.fee;
                self.txids_descriptors
                    .insert(other.txid, vec![other.descriptor]);
            }
        }
        Ok(())
    }
    pub fn net_balance(&self) -> String {
        (self.incoming.to_signed().unwrap() - self.outgoing.to_signed().unwrap()).to_string()
    }

    pub fn verbose(&self) -> String {
        let mut result = String::new();
        let total_tx = self.txids_descriptors.len();
        let txids = self
            .txids_descriptors
            .keys()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let net_balance = self.net_balance();
        result.push_str(&format!("Transactions: {total_tx}\n"));
        result.push_str(&format!("txids: {txids}\n"));
        result.push_str(&format!("fees: {}\n", self.fee));
        result.push_str(&format!("net balance: {net_balance}\n"));
        result
    }
}
struct DescCache {
    descriptor: String,
    cache: HashSet<ScriptBuf>,
}

impl DescCache {
    pub fn new(
        descriptor: &Descriptor<DescriptorPublicKey>,
        how_many: u32,
    ) -> Result<Self, ConversionError> {
        let mut cache = HashSet::with_capacity(how_many as usize);
        for i in 0..how_many {
            let derived = descriptor.at_derivation_index(i)?;
            cache.insert(derived.script_pubkey());
        }
        let descriptor = descriptor.to_string();
        Ok(Self { descriptor, cache })
    }
    pub fn contains(&self, script_pubkey: &ScriptBuf) -> bool {
        self.cache.contains(script_pubkey)
    }
    pub fn descriptor(&self) -> String {
        self.descriptor.clone()
    }
}

impl Display for Balance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f) // TODO
    }
}

impl Display for Balances {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f) // TODO
    }
}

pub fn balances(
    psets: &[PartiallySignedTransaction],
    descriptors: &[Descriptor<DescriptorPublicKey>],
) -> Result<Balances, BalanceError> {
    let mut balances = Balances::default();
    for descriptor in descriptors {
        let desc_cache = DescCache::new(descriptor, 1_000)?;
        for pset in psets {
            let balance = balance_single(&pset, &desc_cache)?;
            balances.merge(balance)?;
        }
    }
    Ok(balances)
}

fn balance_single(
    psbt: &PartiallySignedTransaction,
    desc: &DescCache,
) -> Result<Balance, BalanceError> {
    let mut sum_inputs = 0;
    let mut outgoing = 0;

    let mut sum_outputs = 0;
    let mut incoming = 0;

    let tx = psbt.clone().extract_tx();

    for input in psbt.inputs.iter() {
        let tx_out = input.witness_utxo.as_ref().ok_or(BalanceError::Todo)?;
        sum_inputs += tx_out.value;
        if desc.contains(&tx_out.script_pubkey) {
            outgoing += tx_out.value;
            break;
        }
    }

    for tx_out in tx.output.iter() {
        sum_outputs += tx_out.value;

        if desc.contains(&tx_out.script_pubkey) {
            incoming += tx_out.value;
            break;
        }
    }

    Ok(Balance {
        txid: tx.txid(),
        descriptor: desc.descriptor(),
        fee: Amount::from_sat(sum_inputs - sum_outputs),
        outgoing: Amount::from_sat(outgoing),
        incoming: Amount::from_sat(incoming),
    })
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, str::FromStr};

    use bitcoin::{psbt::PartiallySignedTransaction, Amount, Txid};
    use miniscript::{Descriptor, DescriptorPublicKey};

    use crate::commands::balance::{balances, Balance, Balances, DescCache};

    use super::balance_single;

    // #[test]
    // fn test_balance() {
    //     let desc0: &str = "tr(tpubD6NzVbkrYhZ4X2WmBwDRV6ADRP3PEo5ojs87nQ961SCKZ3MgWxuWUAzCcnzBYJAPGcnCbgn7oKeAyMvaVzWEYrhzK6n6QvTioRZ5SXTWgLi/0/*)";
    //     let desc1 = "tr(tpubD6NzVbkrYhZ4X2WmBwDRV6ADRP3PEo5ojs87nQ961SCKZ3MgWxuWUAzCcnzBYJAPGcnCbgn7oKeAyMvaVzWEYrhzK6n6QvTioRZ5SXTWgLi/1/*)";
    //     let psbt = "cHNidP8BAH0CAAAAATo+J0zYwEal7/uaVNj6+Qg0jXJLGjJ4meeMoFJzYRMoAAAAAAD9////AqCGAQAAAAAAFgAUF2HTc5jrZjQt8vvjZDF4V4bRoq3UZQQqAQAAACJRIEsuu2OBSG4b2fJgLJP+ktSl5Orpn/24whb1Ys/koXGHAAAAAAABASsA8gUqAQAAACJRINmRakRtUofyuj572h5Tfr9igtxLXp8kj+zCG8rZL5FrIRZx8Gl8xlwR2it1EcysGbfpAghRJ6WuE1BeQtRrCGDVqA0AdTpWvwAAAAAAAAAAARcgcfBpfMZcEdordRHMrBm36QIIUSelrhNQXkLUawhg1agAAAEFICFpsB1A6mrjmH+4pmbJ4gfp9+AnrXrfE2z4N3ES7c1sIQchabAdQOpq45h/uKZmyeIH6ffgJ6163xNs+DdxEu3NbA0AdTpWvwEAAAAAAAAAAA==";
    //     let desc0: Descriptor<DescriptorPublicKey> = desc0.parse().unwrap();
    //     let desc1: Descriptor<DescriptorPublicKey> = desc1.parse().unwrap();
    //     let psbt: PartiallySignedTransaction = psbt.parse().unwrap();

    //     // let balance = balance_single(&psbt, &desc0).unwrap();
    //     // assert_eq!("", balance.to_string());
    // }

    #[test]
    fn test_balance() {
        // taken from offline_sign test
        let desc0 = "tr(tpubD6NzVbkrYhZ4XUprtHTHAWupukJFpWBJBBU9pyp62LVMhxnpb1dqDouxv5m2MTTAuWzLvFQmtgWwzHCFTrVXi1HscGm1BZ2xuGDN5KL4zNF/0/*)";
        let desc1 = "tr(tpubD6NzVbkrYhZ4XUprtHTHAWupukJFpWBJBBU9pyp62LVMhxnpb1dqDouxv5m2MTTAuWzLvFQmtgWwzHCFTrVXi1HscGm1BZ2xuGDN5KL4zNF/1/*)";
        let psbt = "cHNidP8BAH0CAAAAAbviGkHAroGDRJdSJP00ADQjpUVeAuccJEzYdTLcwCCIAAAAAAD9////AqCGAQAAAAAAFgAUFyAkO/DEPdd2RR3+zerWr2TDcTXUZQQqAQAAACJRIPaIAcQ3QPw+rS9eJQF9YxL4tR2Fm1T7DBxNCKACNVE+AAAAAAABASsA8gUqAQAAACJRIBEw73shsIFvsT73iQkCiZ/nCc2UwmnpwiKINaibjv6fIRZdw89r9r2z5GhjsuC2NxCQTpzxWEkpPeaP3hBES3QJEw0A1lI6KgAAAAAAAAAAARcgXcPPa/a9s+RoY7LgtjcQkE6c8VhJKT3mj94QREt0CRMAAAEFIM+8YdC2bFRzq/jzakcz/g+hqbs4xYR/8M9ntkrXOsK5IQfPvGHQtmxUc6v482pHM/4Poam7OMWEf/DPZ7ZK1zrCuQ0A1lI6KgEAAAAAAAAAAA==";
        let desc0: Descriptor<DescriptorPublicKey> = desc0.parse().unwrap();
        let desc1: Descriptor<DescriptorPublicKey> = desc1.parse().unwrap();

        let psbt: PartiallySignedTransaction = psbt.parse().unwrap();
        let set0 = DescCache::new(&desc0, 1_000).unwrap();
        let set1 = DescCache::new(&desc1, 1_000).unwrap();

        let balance0 = balance_single(&psbt, &set0).unwrap();
        let balance1 = balance_single(&psbt, &set1).unwrap();
        let txid = "981e91290b2f05d8b5e16d93d7ffe180595c16e19acbcb6e721399d9ae56bb45";
        let txid = Txid::from_str(txid).unwrap();
        assert_eq!(
            balance0,
            Balance {
                txid: txid.clone(),
                descriptor: desc0.to_string(),
                outgoing: Amount::from_btc(50.0).unwrap(),
                incoming: Amount::from_sat(0),
                fee: Amount::from_sat(1420)
            }
        );
        assert_eq!(
            balance1,
            Balance {
                txid,
                descriptor: desc1.to_string(),
                outgoing: Amount::from_sat(0),
                incoming: Amount::from_btc(49.99898580).unwrap(),
                fee: Amount::from_sat(1420)
            }
        );

        let balances = balances(&[psbt], &[desc0.clone(), desc1.clone()]).unwrap();
        let mut txids_descriptors = HashMap::new();
        txids_descriptors.insert(txid, vec![desc0.to_string(), desc1.to_string()]);
        assert_eq!(
            balances,
            Balances {
                outgoing: Amount::from_btc(50.0).unwrap(),
                incoming: Amount::from_btc(49.99898580).unwrap(),
                fee: Amount::from_sat(1420),
                txids_descriptors
            }
        );
    }

    #[test]
    fn test_many_derivations() {
        let desc0: &str = "tr(tpubD6NzVbkrYhZ4X2WmBwDRV6ADRP3PEo5ojs87nQ961SCKZ3MgWxuWUAzCcnzBYJAPGcnCbgn7oKeAyMvaVzWEYrhzK6n6QvTioRZ5SXTWgLi/0/*)";
        let desc0: Descriptor<DescriptorPublicKey> = desc0.parse().unwrap();
        let _set = DescCache::new(&desc0, 10_000);
    }
}
