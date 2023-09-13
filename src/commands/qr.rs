pub fn qr(content: &str, max_chars: u16) -> Result<String, qr_code::types::QrError> {
    let splitted_content = content
        .chars()
        .collect::<Vec<char>>()
        .chunks(max_chars as usize)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<String>>();
    let mut result = String::new();

    //TODO split qr code with structured append anyway??
    for piece in splitted_content {
        let qr = qr_code::QrCode::new(piece.as_bytes())?;
        result.push_str(&qr.to_string(true, 4));
        result.push_str("\n\n\n");
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
