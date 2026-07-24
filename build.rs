fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        // blank version fields make av heuristics twitchy, so fill them in.
        // file/product version come from the cargo version automatically.
        res.set("ProductName", "GBFRER Summon Drop Picker");
        res.set("FileDescription", "GBFRER Summon Drop Picker");
        res.set("CompanyName", "Evoyn");
        res.set("LegalCopyright", "Copyright (c) 2026 Evoyn. MIT licensed.");
        res.set("InternalName", "GBFRER Summon Picker");
        res.set("OriginalFilename", "GBFRER Summon Picker.exe");
        if let Err(e) = res.compile() {
            println!("cargo:warning=could not embed icon: {e}");
        }
    }
    println!("cargo:rerun-if-changed=assets/icon.ico");
}
