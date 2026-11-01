//! A bounded, temporary picker-authorized list. No path crosses IPC or enters SQLite.
use crate::screenshot::{self, Error, Preview};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

pub const MAX_BATCH: usize = 500;
static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);

#[derive(Default)]
pub struct Batch {
    session: Option<(String, Vec<PathBuf>)>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    pub session_id: String,
    pub count: usize,
}

fn stale() -> Error {
    Error {
        code: "batch_expired",
        message:
            "This temporary screenshot queue is no longer available. Select the screenshots again.",
    }
}

impl Batch {
    pub fn select(&mut self, parent: &tauri::WebviewWindow) -> Result<Option<Selection>, Error> {
        let paths = rfd::FileDialog::new()
            .set_parent(parent)
            .set_title("Select up to 500 screenshots for review")
            .add_filter("PNG or JPEG screenshots", &["png", "jpg", "jpeg"])
            .pick_files();
        paths.map(|paths| self.replace(paths)).transpose()
    }

    fn replace(&mut self, paths: Vec<PathBuf>) -> Result<Selection, Error> {
        if paths.is_empty() || paths.len() > MAX_BATCH {
            return Err(Error {
                code: "batch_limit",
                message: "Select between 1 and 500 screenshots per batch.",
            });
        }
        let session_id = NEXT_SESSION.fetch_add(1, Ordering::Relaxed).to_string();
        let count = paths.len();
        self.session = Some((session_id.clone(), paths));
        Ok(Selection { session_id, count })
    }

    pub fn preview(&self, session_id: &str, index: usize) -> Result<Preview, Error> {
        let (active, paths) = self.session.as_ref().ok_or_else(stale)?;
        if active != session_id {
            return Err(stale());
        }
        screenshot::read_selected(paths.get(index).ok_or_else(stale)?)
    }

    pub fn release(&mut self, session_id: &str) {
        if self
            .session
            .as_ref()
            .is_some_and(|(active, _)| active == session_id)
        {
            self.session = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_sessions_reject_stale_indices_and_release_only_the_selected_queue() {
        let mut batch = Batch::default();
        assert_eq!(batch.replace(vec![]).unwrap_err().code, "batch_limit");
        assert_eq!(
            batch
                .replace(vec![PathBuf::new(); MAX_BATCH + 1])
                .unwrap_err()
                .code,
            "batch_limit"
        );
        let first = batch
            .replace(vec![PathBuf::from("private-first.png")])
            .unwrap();
        let second = batch
            .replace(vec![PathBuf::from("private-second.png")])
            .unwrap();
        assert_eq!(
            batch.preview(&first.session_id, 0).unwrap_err().code,
            "batch_expired"
        );
        assert_eq!(
            batch.preview(&second.session_id, 1).unwrap_err().code,
            "batch_expired"
        );
        batch.release(&first.session_id);
        assert_eq!(
            batch.preview(&second.session_id, 0).unwrap_err().code,
            "image_read"
        );
        batch.release(&second.session_id);
        assert_eq!(
            batch.preview(&second.session_id, 0).unwrap_err().code,
            "batch_expired"
        );
    }

    #[test]
    fn processes_only_one_file_and_preserves_the_original() {
        use image::{DynamicImage, ImageFormat};
        let path = std::env::temp_dir().join(format!(
            "tcm-batch-{}-{}.png",
            std::process::id(),
            NEXT_SESSION.fetch_add(1, Ordering::Relaxed)
        ));
        let mut encoded = std::io::Cursor::new(Vec::new());
        DynamicImage::new_rgb8(12, 24)
            .write_to(&mut encoded, ImageFormat::Png)
            .unwrap();
        let bytes = encoded.into_inner();
        std::fs::write(&path, &bytes).unwrap();
        let mut batch = Batch::default();
        let selected = batch
            .replace(vec![path.clone(), PathBuf::from("missing.png")])
            .unwrap();
        assert_eq!(selected.count, 2);
        assert_eq!(batch.preview(&selected.session_id, 0).unwrap().height, 24);
        assert_eq!(
            batch.preview(&selected.session_id, 1).unwrap_err().code,
            "image_read"
        );
        batch.release(&selected.session_id);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        std::fs::remove_file(path).unwrap();
    }
}
