pub mod backup;
mod batch;
pub mod combat_reference;
mod commands;
mod database;
pub mod decision;
pub mod decision_service;
pub mod evaluation;
mod extraction;
pub mod gym;
#[cfg(test)]
mod gym_integration_tests;
pub mod individual_details;
pub mod integrated_planning;
pub mod inventory;
#[cfg(test)]
mod mvp_smoke_tests;
mod ocr_regions;
pub mod preparation;
pub mod preparation_reference;
pub mod preparation_resources;
#[cfg(test)]
mod preparation_tests;
pub mod reference;
pub mod reference_store;
pub mod roles;
mod screenshot;
pub mod source_adapter;
pub mod source_files;
pub mod strategy;
pub mod strategy_service;
pub mod trainer_combat_reference;
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
        if !reference::valid_text(&revision, 100) {
            return Err(commands::CommandError {
                code: "invalid_input",
                message: "Supply an upstream revision or dated local source identifier.",
            });
        }
        let picker = rfd::FileDialog::new()
            .set_parent(&window)
            .set_title("Import local reference JSON")
            .add_filter("Reference JSON", &["json"]);
        let paths = if provider == "pogoapi" {
            picker.pick_files()
        } else {
            picker.pick_file().map(|path| vec![path])
        };
        let Some(paths) = paths else {
            return Ok(None);
        };
        let date: String = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?
            .connection
            .query_row("SELECT date('now')", [], |r| r.get(0))
            .map_err(|_| commands::CommandError::storage())?;
        let input = source_files::read(&provider, &paths, &revision, &date).map_err(|message| {
            commands::CommandError {
                code: "reference_read",
                message,
            }
        })?;
        let bytes = input.bytes;
        let mut repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        let source_digest = reference_store::digest(&bytes);
        if repository
            .local_package()
            .ok()
            .flatten()
            .is_some_and(|active| {
                repository.reference_error.is_none()
                    && active.validate().is_ok()
                    && active.provider == provider
                    && active.source_sha256 == source_digest
                    && active.catalog["dataset"]["upstreamRevision"].as_str() == Some(&revision)
            })
        {
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
                combat: normalized.combat,
                preparation: normalized.preparation,
                trainer_combat: normalized.trainer_combat,
                coverage: normalized.coverage,
                limitations: normalized.limitations,
                source_sha256: reference_store::digest(&bytes),
                provider,
                artifacts: input.artifacts,
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

#[tauri::command]
async fn select_screenshot_batch(
    window: tauri::WebviewWindow,
    state: State<'_, Arc<Mutex<batch::Batch>>>,
) -> Result<Option<batch::Selection>, screenshot::Error> {
    let state = state.inner().clone();
    if PICKER_OPEN.swap(true, Ordering::AcqRel) {
        return Err(screenshot::Error {
            code: "picker_busy",
            message: "An image picker is already open.",
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
        state
            .lock()
            .map_err(|_| screenshot::Error {
                code: "batch_error",
                message: "Temporary batch unavailable.",
            })?
            .select(&window)
    })
    .await
    .map_err(|_| screenshot::Error {
        code: "batch_error",
        message: "Unable to select screenshots.",
    })?
}

#[tauri::command]
async fn screenshot_batch_preview(
    state: State<'_, Arc<Mutex<batch::Batch>>>,
    session_id: String,
    index: usize,
) -> Result<screenshot::Preview, screenshot::Error> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .lock()
            .map_err(|_| screenshot::Error {
                code: "batch_error",
                message: "Temporary batch unavailable.",
            })?
            .preview(&session_id, index)
    })
    .await
    .map_err(|_| screenshot::Error {
        code: "batch_error",
        message: "Unable to preview this screenshot.",
    })?
}

#[tauri::command]
async fn release_screenshot_batch(
    state: State<'_, Arc<Mutex<batch::Batch>>>,
    session_id: String,
) -> Result<(), screenshot::Error> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .lock()
            .map_err(|_| screenshot::Error {
                code: "batch_error",
                message: "Temporary batch unavailable.",
            })?
            .release(&session_id);
        Ok(())
    })
    .await
    .map_err(|_| screenshot::Error {
        code: "batch_error",
        message: "Unable to release this screenshot queue.",
    })?
}

#[tauri::command]
async fn individual_details(
    state: State<'_, NativeInventory>,
    id: String,
) -> Result<individual_details::DetailsView, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        let id = inventory::PokemonId::parse(&id)?;
        repository.individual_details(&id).map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn save_individual_details(
    state: State<'_, NativeInventory>,
    id: String,
    expected_revision: String,
    expected_evidence_token: String,
    values: individual_details::IndividualDetails,
) -> Result<individual_details::DetailsView, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        let id = inventory::PokemonId::parse(&id)?;
        let revision = expected_revision
            .parse::<i64>()
            .map_err(|_| commands::CommandError {
                code: "invalid_input",
                message: "Invalid observation revision.",
            })?;
        repository
            .save_individual_details(&id, revision, &expected_evidence_token, values)
            .map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn create_custom_tag(
    state: State<'_, NativeInventory>,
    name: String,
) -> Result<commands::TagView, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?
            .create_custom_tag(&name)
            .map(Into::into)
            .map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn confirm_pokemon_tags(
    state: State<'_, NativeInventory>,
    id: String,
    expected: Vec<String>,
    desired: Vec<String>,
) -> Result<(), commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        let id = inventory::PokemonId::parse(&id)?;
        repository
            .confirm_tags(&id, &expected, &desired)
            .map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn confirm_batch_import(
    state: State<'_, NativeInventory>,
    request_id: String,
    input: commands::ImportInput,
) -> Result<commands::RecordView, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        commands::confirm_batch(&repository, &request_id, input)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

fn revision_arg(value: &str) -> Result<i64, commands::CommandError> {
    value
        .parse::<i64>()
        .ok()
        .filter(|v| *v >= 0 && v.to_string() == value)
        .ok_or(commands::CommandError {
            code: "invalid_input",
            message: "Invalid nonnegative revision.",
        })
}

#[tauri::command]
async fn collection_strategy(
    state: State<'_, NativeInventory>,
) -> Result<strategy_service::CollectionStrategy, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?
            .collection_strategy()
            .map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn pokemon_roadmap(
    state: State<'_, NativeInventory>,
    id: String,
) -> Result<strategy_service::RoadmapView, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        let id = inventory::PokemonId::parse(&id)?;
        repository.roadmap(&id).map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn individual_decision(
    state: State<'_, NativeInventory>,
    id: String,
) -> Result<decision_service::IndividualReview, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?
            .individual_decision(&inventory::PokemonId::parse(&id)?)
            .map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn generate_roadmap(
    state: State<'_, NativeInventory>,
    id: String,
    expected_context: String,
) -> Result<strategy_service::RoadmapView, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        let id = inventory::PokemonId::parse(&id)?;
        repository
            .generate_roadmap(&id, &expected_context)
            .map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn update_roadmap_progress(
    state: State<'_, NativeInventory>,
    id: String,
    expected_context: String,
    expected_revision: String,
    step_id: String,
    status: String,
) -> Result<strategy_service::RoadmapView, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repository = state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?;
        let id = inventory::PokemonId::parse(&id)?;
        repository
            .update_roadmap_progress(
                &id,
                &expected_context,
                revision_arg(&expected_revision)?,
                &step_id,
                &status,
            )
            .map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn save_strategy_preferences(
    state: State<'_, NativeInventory>,
    expected_revision: String,
    goal: String,
    stardust_budget: Option<u32>,
) -> Result<strategy_service::GoalContext, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?
            .save_strategy_preferences(revision_arg(&expected_revision)?, &goal, stardust_budget)
            .map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn preparation_resources(
    state: State<'_, NativeInventory>,
    id: String,
) -> Result<preparation_resources::ResourceBudget, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?
            .preparation_resources(&inventory::PokemonId::parse(&id)?)
            .map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn select_preparation_path(
    state: State<'_, NativeInventory>,
    id: String,
    expected_context: String,
    path_id: String,
) -> Result<preparation_resources::ResourceBudget, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?
            .select_preparation_path(
                &inventory::PokemonId::parse(&id)?,
                &expected_context,
                &path_id,
            )
            .map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn save_preparation_resources(
    state: State<'_, NativeInventory>,
    id: String,
    expected_revision: String,
    values: preparation_resources::ResourceValues,
) -> Result<preparation_resources::ResourceBudget, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?
            .save_preparation_resources(
                &inventory::PokemonId::parse(&id)?,
                revision_arg(&expected_revision)?,
                values,
            )
            .map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn clear_preparation_route(
    state: State<'_, NativeInventory>,
    id: String,
    expected_context: String,
) -> Result<preparation_resources::ResourceBudget, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .0
            .lock()
            .map_err(|_| commands::CommandError::storage())?
            .clear_preparation_route(&inventory::PokemonId::parse(&id)?, &expected_context)
            .map_err(Into::into)
    })
    .await
    .map_err(|_| commands::CommandError::storage())?
}

#[tauri::command]
async fn export_structured_backup(
    window: tauri::WebviewWindow,
    state: State<'_, NativeInventory>,
) -> Result<bool, commands::CommandError> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move||{
        let Some(path)=rfd::FileDialog::new().set_parent(&window).set_title("Export confirmed collection data").set_file_name("trainer-collection-backup.json").add_filter("Structured JSON backup",&["json"]).save_file() else {return Ok(false)};
        let bytes=state.0.lock().map_err(|_|commands::CommandError::storage())?.structured_backup()?;
        backup::write_atomic(&path,&bytes).map_err(|_|commands::CommandError{code:"backup_write",message:"Unable to write the backup. Existing target data is retained; choose a writable local location."})?;
        Ok(true)
    }).await.map_err(|_|commands::CommandError::storage())?
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
            app.manage(Arc::new(Mutex::new(batch::Batch::default())));
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
            rollback_reference,
            select_screenshot_batch,
            screenshot_batch_preview,
            release_screenshot_batch,
            individual_details,
            save_individual_details,
            create_custom_tag,
            confirm_pokemon_tags,
            confirm_batch_import,
            collection_strategy,
            individual_decision,
            pokemon_roadmap,
            generate_roadmap,
            update_roadmap_progress,
            save_strategy_preferences,
            preparation_resources,
            save_preparation_resources,
            select_preparation_path,
            clear_preparation_route,
            export_structured_backup
        ])
        .run(tauri::generate_context!())
        .expect("error while running Trainer Collection Manager");
}
