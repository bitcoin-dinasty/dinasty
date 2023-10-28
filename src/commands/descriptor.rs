use super::{import::explode_descriptor, ImportError};

pub fn descriptor(
    xprv: crate::key_origin::XprvWithSource,
    public: bool,
) -> Result<String, ImportError> {
    let desc = if public {
        format!("tr({xprv:#}/<0;1>/*)")
    } else {
        format!("tr({xprv}/<0;1>/*)")
    };
    let _ = explode_descriptor(&desc, !public)?;

    Ok(desc)
}
