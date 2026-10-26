fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "inventory_snapshot",
            "confirm_inventory_import",
            "evaluate_pokemon",
            "select_screenshot",
        ]),
    ))
    .expect("Unable to build application permissions");
}
