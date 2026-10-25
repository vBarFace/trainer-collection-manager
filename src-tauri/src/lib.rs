mod database;

use std::path::PathBuf;
use tauri::{Manager, Runtime};

fn database_path<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|directory| directory.join(database::DATABASE_FILE_NAME))
        .map_err(|error| format!("Unable to resolve the local application data directory: {error}"))
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let path = database_path(&app.handle())?;
            database::initialize(&path).map_err(|error| {
                format!(
                    "Unable to initialize the local database at {}: {error}",
                    path.display()
                )
            })?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Trainer Collection Manager");
}
