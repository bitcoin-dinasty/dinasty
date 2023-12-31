use bitcoin::{
    absolute, psbt::PartiallySignedTransaction, Address, Amount, Network, ScriptBuf, SignedAmount,
    Txid,
};
use std::{collections::HashSet, fmt::Display};

use crate::Descriptor;

#[derive(Debug, thiserror::Error)]
pub enum BalanceError {
    #[error("Missing witness UTXO in input {0}")]
    MissingWitnessUtxo(usize),

    #[error(transparent)]
    Coonversion(#[from] miniscript::descriptor::ConversionError),

    #[error(transparent)]
    Miniscript(#[from] miniscript::Error),
}

#[derive(Debug, PartialEq, Eq)]
pub struct PsbtDetail {
    i: usize,
    txid: Txid,
    lock_time: absolute::LockTime,
    descriptors: Vec<String>,
    outgoing: Amount,
    incoming: Amount,
    fee: Amount,

    inputs: Vec<String>,
    outputs: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct GroupDetail {
    details: Vec<PsbtDetail>,
    outgoing: Amount,
    incoming: Amount,
    fee: Amount,
}

pub fn psbt_details(
    psbts: &[PartiallySignedTransaction],
    descriptor: &[Descriptor],
    network: Network,
) -> Result<GroupDetail, BalanceError> {
    let mut group = GroupDetail::default();
    let my_scripts = MyScripts::new(descriptor, 1_000)?;

    for (i, psbt) in psbts.iter().enumerate() {
        let balance = psbt_detail(i, &psbt, &my_scripts, network)?;
        group.merge(balance);
    }

    Ok(group)
}

impl GroupDetail {
    pub fn merge(&mut self, other: PsbtDetail) {
        self.outgoing += other.outgoing;
        self.incoming += other.incoming;
        self.fee += other.fee;
        self.details.push(other);
    }
    fn net_balance(&self) -> SignedAmount {
        self.incoming.to_signed().unwrap() - self.outgoing.to_signed().unwrap()
    }
}
struct MyScripts {
    descriptors: Vec<String>,
    cache: HashSet<ScriptBuf>,
}

impl MyScripts {
    pub fn new(
        multi_descriptors: &[Descriptor],
        how_many_per_desc: u32,
    ) -> Result<Self, BalanceError> {
        let mut cache = HashSet::new();
        let mut descriptors = vec![];
        for multi_descriptor in multi_descriptors {
            for descriptor in multi_descriptor.clone().into_single_descriptors()? {
                descriptors.push(descriptor.to_string());
                for i in 0..how_many_per_desc {
                    let derived = descriptor.at_derivation_index(i)?;
                    cache.insert(derived.script_pubkey());
                }
            }
        }

        Ok(Self { descriptors, cache })
    }
    pub fn contains(&self, script_pubkey: &ScriptBuf) -> bool {
        self.cache.contains(script_pubkey)
    }
    pub fn descriptors(&self) -> &[String] {
        &self.descriptors
    }
}

impl Display for PsbtDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "tx{:>3}:               {}", self.i, self.txid)?;

        writeln!(f, "lockt:               {}", self.lock_time)?;

        for input in self.inputs.iter() {
            writeln!(f, "{}", input)?;
        }
        for output in self.outputs.iter() {
            writeln!(f, "{}", output)?;
        }
        writeln!(f, "{:<5}: {}", "fee", amount8(self.fee.to_sat()))?;

        Ok(())
    }
}

impl Display for GroupDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for detail in self.details.iter() {
            writeln!(f, "{}", detail)?;
        }

        // recap
        let total_tx = self.details.len();
        let net_balance = format!("{:>13.9}", self.net_balance().to_btc());

        writeln!(f, "#txs :               {total_tx}")?;
        writeln!(f, "fees : {}", amount8(self.fee.to_sat()))?;
        writeln!(f, "net  : {}", net_balance)
    }
}

fn psbt_detail(
    i: usize,
    psbt: &PartiallySignedTransaction,
    my_scripts: &MyScripts,
    network: Network,
) -> Result<PsbtDetail, BalanceError> {
    let mut sum_inputs = 0;
    let mut outgoing = 0;

    let mut sum_outputs = 0;
    let mut incoming = 0;

    let tx = psbt.clone().extract_tx();

    let mut inputs = vec![];
    let mut outputs = vec![];

    for (i, input) in psbt.inputs.iter().enumerate() {
        let tx_out = input
            .witness_utxo
            .as_ref()
            .ok_or(BalanceError::MissingWitnessUtxo(i))?;
        sum_inputs += tx_out.value;
        let mine = my_scripts.contains(&tx_out.script_pubkey);
        if mine {
            outgoing += tx_out.value;
        }

        let signed = !input.partial_sigs.is_empty();
        inputs.push(format!(
            "in{:>3}: {} {}{}{}",
            i,
            amount8(tx_out.value),
            tx.input[i].previous_output,
            if mine { " m" } else { "" },
            if signed { " s" } else { "" }
        ));
    }

    for (i, tx_out) in tx.output.iter().enumerate() {
        sum_outputs += tx_out.value;

        let mine = my_scripts.contains(&tx_out.script_pubkey);

        if mine {
            incoming += tx_out.value;
        }

        outputs.push(format!(
            "out{:>2}: {} {}{}",
            i,
            amount8(tx_out.value),
            Address::from_script(&tx_out.script_pubkey, network).unwrap(),
            if mine { " m" } else { "" }
        ));
    }

    Ok(PsbtDetail {
        i,
        txid: tx.txid(),
        lock_time: tx.lock_time,
        descriptors: my_scripts.descriptors().to_vec(),
        fee: Amount::from_sat(sum_inputs - sum_outputs),
        outgoing: Amount::from_sat(outgoing),
        incoming: Amount::from_sat(incoming),
        inputs,
        outputs,
    })
}

fn amount8(satoshi: u64) -> String {
    format!("{:>13.9}", Amount::from_sat(satoshi).to_btc())
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use bitcoin::{absolute, psbt::PartiallySignedTransaction, Amount, Txid};
    use details::psbt_details;

    use crate::{
        commands::details::{self, GroupDetail, MyScripts, PsbtDetail},
        Descriptor,
    };

    use super::psbt_detail;

    #[test]
    fn test_details() {
        // taken from offline_sign test
        let network = bitcoin::Network::Regtest;
        let desc = "tr(tpubD6NzVbkrYhZ4XUprtHTHAWupukJFpWBJBBU9pyp62LVMhxnpb1dqDouxv5m2MTTAuWzLvFQmtgWwzHCFTrVXi1HscGm1BZ2xuGDN5KL4zNF/<0;1>/*)";
        let psbt = "cHNidP8BAH0CAAAAAbviGkHAroGDRJdSJP00ADQjpUVeAuccJEzYdTLcwCCIAAAAAAD9////AqCGAQAAAAAAFgAUFyAkO/DEPdd2RR3+zerWr2TDcTXUZQQqAQAAACJRIPaIAcQ3QPw+rS9eJQF9YxL4tR2Fm1T7DBxNCKACNVE+AAAAAAABASsA8gUqAQAAACJRIBEw73shsIFvsT73iQkCiZ/nCc2UwmnpwiKINaibjv6fIRZdw89r9r2z5GhjsuC2NxCQTpzxWEkpPeaP3hBES3QJEw0A1lI6KgAAAAAAAAAAARcgXcPPa/a9s+RoY7LgtjcQkE6c8VhJKT3mj94QREt0CRMAAAEFIM+8YdC2bFRzq/jzakcz/g+hqbs4xYR/8M9ntkrXOsK5IQfPvGHQtmxUc6v482pHM/4Poam7OMWEf/DPZ7ZK1zrCuQ0A1lI6KgEAAAAAAAAAAA==";
        let desc: [Descriptor; 1] = [desc.parse().unwrap()];

        let psbt: PartiallySignedTransaction = psbt.parse().unwrap();
        let my_scripts = MyScripts::new(&desc, 1_000).unwrap();

        let balance = psbt_detail(0, &psbt, &my_scripts, network).unwrap();

        let txid = "981e91290b2f05d8b5e16d93d7ffe180595c16e19acbcb6e721399d9ae56bb45";
        let txid = Txid::from_str(txid).unwrap();
        assert_eq!(
            balance,
            PsbtDetail {
                i: 0,
                txid: txid.clone(),
                lock_time: absolute::LockTime::from_height(0).unwrap(),
                descriptors: vec!["tr(tpubD6NzVbkrYhZ4XUprtHTHAWupukJFpWBJBBU9pyp62LVMhxnpb1dqDouxv5m2MTTAuWzLvFQmtgWwzHCFTrVXi1HscGm1BZ2xuGDN5KL4zNF/0/*)#xq5v4q9d".to_string(), "tr(tpubD6NzVbkrYhZ4XUprtHTHAWupukJFpWBJBBU9pyp62LVMhxnpb1dqDouxv5m2MTTAuWzLvFQmtgWwzHCFTrVXi1HscGm1BZ2xuGDN5KL4zNF/1/*)#h53dg444".to_string()],
                outgoing: Amount::from_btc(50.0).unwrap(),
                incoming: Amount::from_btc(49.99898580).unwrap(),
                fee: Amount::from_sat(1420),
                inputs: vec!["in  0:  50.000000000 8820c0dc3275d84c241ce7025e45a523340034fd245297448381aec0411ae2bb:0 m".to_string()],
                outputs: vec!["out 0:   0.001000000 bcrt1qzuszgwlscs7awaj9rhlvm6kk4ajvxuf4qs9ue9".to_string(),"out 1:  49.998985800 bcrt1p76yqr3phgr7ratf0tcjszltrztut28v9nd20krquf5y2qq342ylqfv0qfu m".to_string()],
            }
        );

        let balances = psbt_details(&[psbt.clone()], &desc, network).unwrap();
        assert_eq!(
            balances,
            GroupDetail {
                outgoing: Amount::from_btc(50.0).unwrap(),
                incoming: Amount::from_btc(49.99898580).unwrap(),
                fee: Amount::from_sat(1420),

                details: vec![balance],
            }
        );

        let expected = r#"tx  0:               981e91290b2f05d8b5e16d93d7ffe180595c16e19acbcb6e721399d9ae56bb45
lockt:               0
in  0:  50.000000000 8820c0dc3275d84c241ce7025e45a523340034fd245297448381aec0411ae2bb:0 m
out 0:   0.001000000 bcrt1qzuszgwlscs7awaj9rhlvm6kk4ajvxuf4qs9ue9
out 1:  49.998985800 bcrt1p76yqr3phgr7ratf0tcjszltrztut28v9nd20krquf5y2qq342ylqfv0qfu m
fee  :   0.000014200

#txs :               1
fees :   0.000014200
net  :  -0.001014200
"#;
        let actual = balances.to_string();
        assert_eq!(balances.to_string(), expected, "\n{}\n{}", actual, expected);

        let balances2 = psbt_details(&[psbt.clone(), psbt.clone()], &desc, network).unwrap();
        let actual = balances2.to_string();
        let expected = r#"tx  0:               981e91290b2f05d8b5e16d93d7ffe180595c16e19acbcb6e721399d9ae56bb45
lockt:               0
in  0:  50.000000000 8820c0dc3275d84c241ce7025e45a523340034fd245297448381aec0411ae2bb:0 m
out 0:   0.001000000 bcrt1qzuszgwlscs7awaj9rhlvm6kk4ajvxuf4qs9ue9
out 1:  49.998985800 bcrt1p76yqr3phgr7ratf0tcjszltrztut28v9nd20krquf5y2qq342ylqfv0qfu m
fee  :   0.000014200

tx  1:               981e91290b2f05d8b5e16d93d7ffe180595c16e19acbcb6e721399d9ae56bb45
lockt:               0
in  0:  50.000000000 8820c0dc3275d84c241ce7025e45a523340034fd245297448381aec0411ae2bb:0 m
out 0:   0.001000000 bcrt1qzuszgwlscs7awaj9rhlvm6kk4ajvxuf4qs9ue9
out 1:  49.998985800 bcrt1p76yqr3phgr7ratf0tcjszltrztut28v9nd20krquf5y2qq342ylqfv0qfu m
fee  :   0.000014200

#txs :               2
fees :   0.000028400
net  :  -0.002028400
"#;
        assert_eq!(
            balances2.to_string(),
            expected,
            "\n{}\n{}",
            actual,
            expected
        );
    }

    #[test]
    fn test_many_derivations() {
        let desc0: &str = "tr(tpubD6NzVbkrYhZ4X2WmBwDRV6ADRP3PEo5ojs87nQ961SCKZ3MgWxuWUAzCcnzBYJAPGcnCbgn7oKeAyMvaVzWEYrhzK6n6QvTioRZ5SXTWgLi/0/*)";
        let desc0: Descriptor = desc0.parse().unwrap();
        let _ = MyScripts::new(&[desc0], 10_000).unwrap();
    }
}
