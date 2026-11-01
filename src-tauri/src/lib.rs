mod commands;
mod database;
pub mod evaluation;
mod extraction;
pub mod inventory;
mod ocr_regions;
pub mod reference;
pub mod reference_store;
mod screenshot;
pub mod source_adapter;
pub mod utility;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tauri::{Manager, State};

#[derive(Clone)]
struct NativeInventory(Arc<Mutex<inventory::InventoryRepository>>);
static PICKER_OPEN: AtomicBool = AtomicBool::new(false);

#[tauri::command]
async fn inventory_snapshot(
    state: State<'_, NativeInventory>,
) -> Result<commands::Snapshot, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        commands::snapshot(&repository)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn confirm_inventory_import(
    state: State<'_, NativeInventory>,
    input: commands::ImportInput,
) -> Result<commands::RecordView, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        commands::confirm(&repository, input)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn evaluate_pokemon(
    state: State<'_, NativeInventory>,
    id: String,
) -> Result<commands::RecordView, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        commands::evaluate(&repository, &id)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn correct_pokemon_cp(
    state: State<'_, NativeInventory>,
    id: String,
    input: commands::CorrectCpInput,
) -> Result<commands::RecordView, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        commands::correct_cp(&repository, &id, input)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn correct_pokemon_ivs(
    state: State<'_, NativeInventory>,
    id: String,
    input: commands::CorrectIvsInput,
) -> Result<commands::RecordView, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        commands::correct_ivs(&repository, &id, input)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn correct_pokemon_identity(
    state: State<'_, NativeInventory>,
    id: String,
    input: commands::CorrectIdentityInput,
) -> Result<commands::RecordView, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        commands::correct_identity(&repository, &id, input)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn select_screenshot(
    window: tauri::WebviewWindow,
) -> Result<Option<screenshot::Preview>, screenshot::Error> {
    if PICKER_OPEN.swap(true, Ordering::AcqRel) {
        return Err(screenshot::Error {
            code: "picker_busy",
            message: "A screenshot picker is already open.",
        });
    }
    tauri::async_runtime::spawn_blocking(move || {
        struct Release;
        impl Drop for Release {
            fn drop(&mut self) {
                PICKER_OPEN.store(false, Ordering::Release);
            }
        }
        let _release = Release;
        screenshot::select(&window)
    })
    .await
    .map_err(|_| screenshot::Error {
        code: "image_error",
        message: "Unable to review this image. Try another PNG or JPEG.",
    })?
}

#[tauri::command]
async fn evolution_options(
    state: State<'_, NativeInventory>,
    id: String,
) -> Result<inventory::EvolutionOptions, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        commands::evolution_options(&repository, &id)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn confirm_evolution(
    state: State<'_, NativeInventory>,
    id: String,
    input: commands::EvolutionInput,
) -> Result<commands::RecordView, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        commands::confirm_evolution(&repository, &id, input)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn reobserve_pokemon(
    state: State<'_, NativeInventory>,
    id: String,
    input: commands::ReobserveInput,
) -> Result<commands::RecordView, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        commands::reobserve(&repository, &id, input)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn confirm_power_up(
    state: State<'_, NativeInventory>,
    id: String,
    input: commands::PowerUpInput,
) -> Result<commands::RecordView, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        commands::confirm_power_up(&repository, &id, input)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn pokemon_history(
    state: State<'_, NativeInventory>,
    id: String,
) -> Result<Vec<inventory::LifecycleEvent>, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        commands::history(&repository, &id)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn reference_status(
    state: State<'_, NativeInventory>,
) -> Result<reference_store::ReferenceStatus, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?
            .reference_status()
            .map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn import_reference(
    window: tauri::WebviewWindow,
    state: State<'_, NativeInventory>,
    provider: String,
    revision: String,
) -> Result<Option<reference_store::ReferenceStatus>, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        use std::io::Read;
        if !reference::valid_text(&revision, 100) {
            return Err(commands::CommandError {
                code: "invalid_input",
                message: "Supply an upstream revision or dated local source identifier.",
            });
        }
        let Some(path) = rfd::FileDialog::new()
            .set_parent(&window)
            .set_title("Import local reference JSON")
            .add_filter("Reference JSON", &["json"])
            .pick_file()
        else {
            return Ok(None);
        };
        let file = std::fs::File::open(path).map_err(|_| commands::CommandError {
            code: "reference_read",
            message: "Unable to read the selected reference file.",
        })?;
        if !file
            .metadata()
            .map_err(|_| commands::CommandError::storage())?
            .is_file()
        {
            return Err(commands::CommandError::storage());
        }
        let mut bytes = Vec::new();
        file.take(reference_store::MAX_SOURCE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| commands::CommandError::storage())?;
        if bytes.len() as u64 > reference_store::MAX_SOURCE_BYTES {
            return Err(commands::CommandError {
                code: "reference_limit",
                message: "Reference source exceeds the 64 MiB limit.",
            });
        }
        let mut repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        let date: String = repository
            .connection
            .query_row("SELECT date('now')", [], |r| r.get(0))
            .map_err(|_| commands::CommandError::storage())?;
        let source_digest = reference_store::digest(&bytes);
        if repository.local_package().ok().flatten().is_some_and(|active| {
            active.provider == provider
                && active.source_sha256 == source_digest
                && active.catalog["dataset"]["upstreamRevision"].as_str() == Some(&revision)
        }) {
            return repository.reference_status().map(Some).map_err(Into::into);
        }
        let normalized =
            source_adapter::normalize(&provider, &bytes, &revision, &date).map_err(|message| {
                commands::CommandError {
                    code: "reference_invalid",
                    message,
                }
            })?;
        repository
            .install_reference(reference_store::LocalPackage {
                catalog: normalized.catalog,
                cp_multipliers: normalized.cp_multipliers,
                coverage: normalized.coverage,
                limitations: normalized.limitations,
                source_sha256: reference_store::digest(&bytes),
                provider,
            })
            .map(Some)
            .map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn rollback_reference(
    state: State<'_, NativeInventory>,
) -> Result<reference_store::ReferenceStatus, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?
            .rollback_reference()
            .map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let path = app
                .path()
                .app_data_dir()
                .map_err(|_| "Unable to resolve local application storage")?
                .join(database::DATABASE_FILE_NAME);
            let repository = inventory::InventoryRepository::open(&path)
                .map_err(|_| "Unable to initialize local inventory storage")?;
            app.manage(NativeInventory(Arc::new(Mutex::new(repository))));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            inventory_snapshot,
            confirm_inventory_import,
            evaluate_pokemon,
            correct_pokemon_ivs,
            correct_pokemon_cp,
            correct_pokemon_identity,
            select_screenshot,
            evolution_options,
            confirm_evolution,
            confirm_power_up,
            reobserve_pokemon,
            pokemon_history,
            reference_status,
            import_reference,
            rollback_reference
        ])
        .run(tauri::generate_context!())
        .expect("error while running Trainer Collection Manager");
}
