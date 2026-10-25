mod commands;
mod database;
pub mod inventory;
mod screenshot;

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
            select_screenshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running Trainer Collection Manager");
}
