use qr_code::structured::SplittedQr;

pub fn qr(content: &str, version: i16) -> Result<String, qr_code::types::QrError> {
    let splitted = SplittedQr::new(content.as_bytes().to_vec(), version)?;

    let mut result = String::new();
    let splitted = splitted.split()?;
    let border = 4;

    let len = splitted.len();
    for (i, qr) in splitted.iter().enumerate() {
        let number = format!("({}/{len})\n", i + 1);
        let spaces = " ".repeat((qr.width() + border * 2 - number.len()) / 2);

        result.push_str(&spaces);
        result.push_str(&number);

        result.push_str(&qr.to_string(true, border as u8));
        result.push_str("\n\n\n\n");
    }
    Ok(result)
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use bech32::{ToBase32, Variant};
    use bitcoin::psbt::PartiallySignedTransaction;

    #[test]
    fn qr() {
        let psbt_str = include_str!("../../test_data/psbt_base64");
        assert_eq!(psbt_str.len(), 452);
        let qr_code = qr_code::QrCode::new(psbt_str.as_bytes()).unwrap();
        assert_eq!(qr_code.to_vec().len(), 6561); // base64

        let psbt = PartiallySignedTransaction::from_str(psbt_str).unwrap();
        let psbt_bytes = psbt.serialize();
        let psbt_bech32 = bech32::encode("psbt", psbt_bytes.to_base32(), Variant::Bech32m)
            .unwrap()
            .to_ascii_uppercase();
        assert_eq!(psbt_bech32.len(), 554);

        let qr_code = qr_code::QrCode::new(psbt_bech32.as_bytes()).unwrap();
        assert_eq!(qr_code.to_vec().len(), 5929); // uppercase bech32 10% improvement over base64, is it enough to justify a switch in standard encoding? Probably not

        let qr_code =
            qr_code::QrCode::new(psbt.serialize_hex().to_ascii_uppercase().as_bytes()).unwrap();
        assert_eq!(qr_code.to_vec().len(), 7225); // uppercase hex worse than base64

        let bc_base64 = "cHNidP8BAHECAAAAAUKAmWVFG/zLmhHQZ8Q8bQKCoNxxiurIcNZVhafCoLCyAAAAAAD9////AjksAAAAAAAAFgAUvDLbTPtQXj/JTiGj7HTRrvz8ASwiwQAAAAAAABYAFOq0zUDTkmdWzFLl+RydPI47QA4ReCIMAE8BBIiyHgPG2cycgAAAADKRxb1j69WGKYT3nQrjs2zcdlE3UiHFNYyGd3vysjMcAiVPCY671cdIpViKxJr/88kFWlty0jqxfwY8PfU3Tbu3EMh3/URUAACAAAAAgAAAAIAAAQEfN/cAAAAAAAAWABSUBvAKoN3uZMfqYvmU5pVYAoCj5QEDBAEAAAAiBgNa5FjqiQq3akaVNyAsPdho05JgUpccQlhll1wyKD/3FhjId/1EVAAAgAAAAIAAAACAAAAAAAEAAAAAACICAkjtpvsDgJ+za5zGXZjCZhj5fNVw8Uqc+AgQItHhEYiTGMh3/URUAACAAAAAgAAAAIABAAAAAAAAAAA=";
        let qr_code1 = qr_code::QrCode::new(bc_base64.as_bytes()).unwrap();
        assert_eq!(qr_code1.to_vec().len(), 7921); // base64

        let bc_ur = "ur:crypto-psbt/hkadlbjojkidjyzmadaejsaoaeaeaeadfwlanlihfecwztsbnybytiiossfnjnaolfnbuojslewdspjotbgolpossanbpfpraeaeaeaeaezczmzmzmaoesdwaeaeaeaeaeaecmaebbrfeyuygszogdhyfhsoglclotwpjyttplztztaddwcpseaeaeaeaeaeaecmaebbwdqzsnfztemoiohfsfgmvwytcentfnmnfrfzbabykscpbnaegwadaaloprckaxswtasfnslaaeaeaeeymeskryiawmtllndtlrylntbkvlqdjzuokogyemgmclskeclklnktkgwzpreoceaodagwasmnrktlstfdonhdlessnyzmwfsoahhthpjptdftpalbamfnfsykemgtrkrlbespktzcfyghaeaelaaeaeaelaaeaeaelaaeadadctemylaeaeaeaeaeaecmaebbmwamwtbknbutwyiestwdidytmwvamdhdaolaotvwadaxaaadaeaeaecpamaxhtvehdwdldbkrlimfgmdemcxdwfstpistemohngmmscefwhdihmshheydefhylcmcsspktzcfyghaeaelaaeaeaelaaeaeaelaaeaeaeaeadaeaeaeaeaecpaoaofdweolzoaxlaneqdjensswhlmksaiycsytketljowngensyaaybecpttvybylomucsspktzcfyghaeaelaaeaeaelaaeaeaelaadaeaeaeaeaeaeaeaecfktctrf";
        let qr_code2 = qr_code::QrCode::new(bc_ur.to_ascii_uppercase().as_bytes()).unwrap();
        assert_eq!(qr_code2.to_vec().len(), 7921); // blockchain commons
    }
}
