fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        res.set("FileDescription", "Steam Workshop Utility");
        res.set("ProductName", "s7forge");
        res.set("CompanyName", "Burak Kartal");
        res.set("FileVersion", "0.1.1");
        res.set("ProductVersion", "0.1.1");
        res.set("LegalCopyright", "MIT License - Open Source Software");
        res.set("OriginalFilename", "s7forge.exe");
        res.set("InternalName", "s7forge");
        res.compile().expect("Failed to compile Windows resources");
    }
}
