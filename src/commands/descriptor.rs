pub fn descriptor(xprv: crate::key_origin::XprvWithSource, public: bool) -> String {
    if public {
        format!("tr({xprv:#}/<0;1>/*)")
    } else {
        format!("tr({xprv}/<0;1>/*)")
    }
}
