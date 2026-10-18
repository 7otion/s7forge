fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        res.set("FileDescription", "Steam Workshop Utility");
        res.set("ProductName", "s7forge");
        res.set("CompanyName", "Burak Kartal");
        let version = std::env::var("CARGO_PKG_VERSION").unwrap();
        res.set("FileVersion", &version);
        res.set("ProductVersion", &version);
        res.set("LegalCopyright", "MIT License - Open Source Software");
        res.set("OriginalFilename", "s7forge.exe");
        res.set("InternalName", "s7forge");
        res.compile().expect("Failed to compile Windows resources");
    }
}
