//! Build script for qubit-setup installer
//! Embeds Windows icon and version metadata when compiling for Windows targets.

fn main() {
    println!("cargo:rerun-if-changed=resources/icon.ico");

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("resources/icon.ico");
        res.set("ProductName", "QubitCore & QubitDash");
        res.set("FileDescription", "QubitCore Setup & Bootstrapper");
        res.set("LegalCopyright", "Copyright (c) 2026 Kvbi213");
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=Failed to compile Windows resources: {}", e);
        }
    }
}
