pub fn descriptor(
    xprv: crate::key_origin::XprvWithSource,
    public: bool,
    only_external: bool,
) -> String {
    match (public, only_external) {
        (true, true) => format!("tr({xprv:#}/0/*)"),
        (true, false) => format!("tr({xprv:#}/<0;1>/*)"),
        (false, true) => format!("tr({xprv}/0/*)"),
        (false, false) => format!("tr({xprv}/<0;1>/*)"),
    }
}
