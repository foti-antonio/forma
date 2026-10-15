fn main() {
    #[cfg(feature = "desktop")]
    {
        let mut attributes = tauri_build::Attributes::new();
        // Tauri's default resource only reaches binaries, so Windows lib tests
        // otherwise load without Common Controls v6 and fail before test startup.
        // Use the MSVC linker for all artifacts, preserving the same dependency.
        if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
            && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
        {
            attributes = attributes
                .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
            let manifest = std::path::PathBuf::from(
                std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory"),
            )
            .join("windows-app-manifest.xml");
            println!("cargo:rerun-if-changed={}", manifest.display());
            println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
            println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
        }
        tauri_build::try_build(attributes).expect("Tauri build configuration");
    }
}
