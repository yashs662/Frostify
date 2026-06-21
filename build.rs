fn main() {
    // Embed the Windows executable icon. No-op on other platforms.
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/logo/icon.ico");
        if let Err(e) = res.compile() {
            println!("cargo:warning=failed to embed exe icon: {e}");
        }
    }
    println!("cargo:rerun-if-changed=assets/logo/icon.ico");
    println!("cargo:rerun-if-changed=build.rs");
}
