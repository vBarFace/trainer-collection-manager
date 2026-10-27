fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "inventory_snapshot",
            "confirm_inventory_import",
            "evaluate_pokemon",
            "correct_pokemon_ivs",
            "correct_pokemon_cp",
            "correct_pokemon_identity",
            "select_screenshot",
            "evolution_options",
            "confirm_evolution",
            "confirm_power_up",
            "reobserve_pokemon",
            "pokemon_history",
        ]),
    ))
    .expect("Unable to build application permissions");
}
