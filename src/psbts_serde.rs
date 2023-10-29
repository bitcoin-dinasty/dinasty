use bitcoin::{
    consensus::{encode, Decodable, Encodable, ReadExt},
    psbt::{self, PartiallySignedTransaction},
    VarInt,
};

const MAGIC: &[u8] = "psbts".as_bytes();
const SEPARATOR: u8 = 0xff;

pub fn serialize(psbts: &[PartiallySignedTransaction]) -> Vec<u8> {
    let mut result = vec![];
    result.extend(MAGIC);
    result.push(SEPARATOR);

    VarInt(psbts.len() as u64)
        .consensus_encode(&mut result)
        .expect("OOM");
    for psbt in psbts {
        result.extend(psbt.serialize());
    }

    result
}

#[derive(thiserror::Error, Debug)]
pub enum DecodeError {
    #[error(transparent)]
    ConsensusDecode(#[from] encode::Error),

    #[error(transparent)]
    PsbtDecode(#[from] psbt::Error),

    #[error("WrongMagic {0:?}")]
    WrongMagic([u8; 5]),

    #[error("WrongSeparator")]
    WrongSeparator,
}

pub fn deserialize(bytes: &[u8]) -> Result<Vec<PartiallySignedTransaction>, DecodeError> {
    let mut result = vec![];
    let mut bytes = bytes;
    let mut magic = [0u8; 5];
    bytes.read_slice(&mut magic)?;
    if magic != MAGIC {
        return Err(DecodeError::WrongMagic(magic));
    }
    let sep = bytes.read_u8()?;
    if sep != SEPARATOR {
        return Err(DecodeError::WrongSeparator);
    }

    let len = VarInt::consensus_decode(&mut bytes)?;
    let mut bytes_read = 0;
    for i in 0..len.0 {
        eprintln!("{i}");
        let bytes = &bytes[bytes_read..];

        let psbt = PartiallySignedTransaction::deserialize(bytes)?;
        bytes_read += psbt.serialize().len();

        result.push(psbt);
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use bitcoin::{
        consensus::{Decodable, ReadExt},
        psbt::PartiallySignedTransaction,
    };

    use crate::{psbts_serde::MAGIC, test_util::psbt_base64};

    #[test]
    fn psbts_roundtrip() {
        let psbt_str = psbt_base64();
        let psbt: PartiallySignedTransaction = psbt_str.parse().unwrap();
        let psbts = vec![psbt.clone(), psbt];
        let psbts_ser = super::serialize(&psbts);
        let psbts_back = super::deserialize(&psbts_ser).unwrap();
        assert_eq!(psbts, psbts_back);
    }

    #[test]
    fn test_advance() {
        let data = vec![0u8, 0];
        let mut data = &data[..];
        bitcoin::VarInt::consensus_decode(&mut data).unwrap();
        assert_eq!(data, &[0u8]);

        let mut data = MAGIC.to_vec();
        data.push(0);
        let mut data = &data[..];
        let mut buffer = [0u8; 5];
        data.read_slice(&mut buffer).unwrap();
        assert_eq!(data, &[0u8]);
    }
}
