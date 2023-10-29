use qr_code::QrCode;

/// Max bytes encodable in a structured append qr code, given Qr code version as array index
const MAX_BYTES: [usize; 33] = [
    0, 15, 30, 51, 76, 104, 132, 152, 190, 228, 269, 319, 365, 423, 456, 518, 584, 642, 716, 790,
    856, 927, 1001, 1089, 1169, 1271, 1365, 1463, 1526, 1626, 1730, 1838, 1950,
];

pub fn qr(
    content: &str,
    version: i16,
    border: u8,
    empty_lines: u8,
    label: Option<String>,
) -> Result<String, qr_code::types::QrError> {
    let mut result = String::new();
    let empty_lines = "\n".repeat(empty_lines as usize);
    let label = label.as_deref().unwrap_or("");

    let valid_bech32 = bech32::decode_without_checksum(content).is_ok();
    let mut max_chars = MAX_BYTES[version as usize];
    if valid_bech32 && content.chars().all(|c| c.is_ascii_uppercase()) {
        // in this case the Alphanumeric encoding save space
        max_chars = max_chars + max_chars / 2;
    }
    let splitted_data = content
        .chars()
        .collect::<Vec<char>>()
        .chunks(max_chars)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<String>>();
    let len = splitted_data.len();
    for (i, data) in splitted_data.iter().enumerate() {
        let qr = QrCode::new(data)?;
        print_qr(i, &qr, border, &mut result, &empty_lines, len, label);
    }

    Ok(result)
}

fn print_qr(
    i: usize,
    qr: &QrCode,
    border: u8,
    result: &mut String,
    empty_lines: &String,
    len: usize,
    label: &str,
) {
    let version = match qr.version() {
        qr_code::Version::Normal(x) => x,
        qr_code::Version::Micro(x) => -x,
    };
    let number = format!("{} ({}/{len}) v{:?}\n", label, i + 1, version);
    let qr_width_with_border = qr.width() + border as usize * 2;
    let spaces = " ".repeat((qr_width_with_border.saturating_sub(number.len())) / 2);

    result.push_str(&spaces);
    result.push_str(&number);

    result.push_str(&qr.to_string(true, border));
    result.push_str(empty_lines);
}

#[cfg(test)]
mod test {
    use bech32::{ToBase32, Variant};
    use bitcoin::psbt::PartiallySignedTransaction;
    use std::str::FromStr;

    #[test]
    fn qr() {
        let psbt_str = crate::test_util::psbt_base64();
        assert_eq!(psbt_str.len(), 452);
        let qr_code = qr_code::QrCode::new(psbt_str.as_bytes()).unwrap();
        assert_eq!(qr_code.to_vec().len(), 6561); // base64

        let psbt = PartiallySignedTransaction::from_str(&psbt_str).unwrap();
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
