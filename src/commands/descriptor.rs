use bitcoin::secp256k1::Secp256k1;

use super::{import::explode_descriptor, ImportError};

pub fn descriptor(
    xprv: crate::key_origin::XprvWithSource,
    public: bool,
    only_external: bool,
) -> Result<String, ImportError> {
    let desc = match (public, only_external) {
        (true, true) => format!("tr({xprv:#}/0/*)"),
        (true, false) => format!("tr({xprv:#}/<0;1>/*)"),
        (false, true) => format!("tr({xprv}/0/*)"),
        (false, false) => format!("tr({xprv}/<0;1>/*)"),
    };
    if only_external {
        let _ = miniscript::Descriptor::parse_descriptor(&Secp256k1::new(), &desc)?;
    } else {
        let _ = explode_descriptor(&desc, !public)?;
    }

    Ok(desc)
}
