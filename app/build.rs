fn main() {
    #[cfg(debug_assertions)]
    {
        println!("DEV BUILD");
        tauri_build::build();
    }

    #[cfg(all(not(debug_assertions), not(feature = "develop"), target_os = "windows"))]
    {
        use std::fs;
        let mut windows = tauri_build::WindowsAttributes::new();
        windows = windows.app_manifest(fs::read_to_string("windows-app.manifest").unwrap());
        let attributes = tauri_build::Attributes::new().windows_attributes(windows);

        tauri_build::try_build(attributes).expect("failed to run build script");
    }

    #[cfg(all(not(debug_assertions), target_os = "linux"))]
    {
        tauri_build::build();
    }
}
