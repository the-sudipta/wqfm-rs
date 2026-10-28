//! Embeds the wQFM-rs icon and version details in `wqfm-rs-gui.exe`
//! (Windows builds only).

fn main() {
    println!("cargo:rerun-if-changed=assets/wqfm-rs.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/wqfm-rs.ico");
        res.set("FileDescription", "wQFM-rs desktop app");
        res.set("ProductName", "wQFM-rs");
        res.set(
            "LegalCopyright",
            "Apache-2.0; wQFM is (c) its authors (see NOTICE)",
        );
        if let Err(e) = res.compile() {
            println!("cargo:warning=icon not embedded: {e}");
        }
    }
}
