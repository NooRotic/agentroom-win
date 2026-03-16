use std::path::PathBuf;

fn main() {
    // Emit AGENTROOM_CASS_BIN_DEFAULT so cass_bin() can find the submodule binary
    // without requiring the user to set CASS_BIN manually.
    // CARGO_MANIFEST_DIR is src-tauri/, so ../search-backend/cass/ is the submodule root.
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    let cass_suffix = if cfg!(windows) { "cass.exe" } else { "cass" };
    let cass_bin = PathBuf::from(&manifest_dir)
        .join("../search-backend/cass/target/release")
        .join(cass_suffix);

    // Normalise separators without requiring the path to exist yet.
    let cass_bin_str = cass_bin
        .to_string_lossy()
        .replace('\\', "/");

    println!("cargo:rustc-env=AGENTROOM_CASS_BIN_DEFAULT={cass_bin_str}");

    tauri_build::build()
}
