//! User-selected reference files only. Normalized data lives in the per-user
//! SQLite database; activation and rollback are transactions, never bundle edits.
use crate::inventory::{Error, InventoryRepository, Result};
use crate::reference::Catalog;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const MAX_SOURCE_BYTES: u64 = 64 * 1024 * 1024;

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalPackage {
    pub catalog: Value,
    pub cp_multipliers: Vec<(f64, f64)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combat: Option<crate::combat_reference::CombatReference>,
    pub coverage: Vec<String>,
    pub limitations: Vec<String>,
    pub source_sha256: String,
    pub provider: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<crate::source_files::SourceArtifact>,
}

impl LocalPackage {
    pub fn validate(&self) -> Result<Catalog> {
        let catalog = Catalog::load(
            &serde_json::to_vec(&self.catalog)
                .map_err(|_| Error::InvalidInput("Invalid catalog"))?,
        )
        .map_err(Error::InvalidInput)?;
        if self.source_sha256.len() != 64
            || !self.source_sha256.bytes().all(|b| b.is_ascii_hexdigit())
            || self.cp_multipliers.len() > 200
            || self.coverage.len() > 100
            || self.limitations.len() > 100
        {
            return Err(Error::InvalidInput(
                "Invalid source provenance or mechanics size",
            ));
        }
        let mut previous = (0.0, 0.0);
        if let Some(combat) = &self.combat {
            combat.validate().map_err(Error::InvalidInput)?;
            if self.provider != "pogoapi"
                || self.catalog["dataset"]["datasetId"].as_str() != Some("local-pogoapi-normal-1")
                || combat.pools.iter().any(|pool| {
                    !self.catalog["forms"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|form| form["id"].as_str() == Some(&pool.form_id))
                })
            {
                return Err(Error::InvalidInput(
                    "Combat pools must belong to the active PoGoAPI catalog",
                ));
            }
        }
        if self.artifacts.len() > 7
            || self.artifacts.iter().any(|artifact| {
                !crate::reference::valid_text(&artifact.kind, 100)
                    || !crate::reference::valid_text(&artifact.source_url, 2000)
                    || !artifact.source_url.starts_with("https://")
                    || artifact.sha256.len() != 64
                    || !artifact.sha256.bytes().all(|b| b.is_ascii_hexdigit())
                    || !crate::reference::valid_text(&artifact.upstream_revision, 100)
                    || !crate::reference::valid_text(&artifact.retrieved_at, 10)
            })
        {
            return Err(Error::InvalidInput("Invalid source artifact provenance"));
        }
        for &(level, multiplier) in &self.cp_multipliers {
            if !level.is_finite()
                || !multiplier.is_finite()
                || !(1.0..=51.0).contains(&level)
                || (level * 2.0).fract() != 0.0
                || !(0.0..1.0).contains(&multiplier)
                || level <= previous.0
                || multiplier <= previous.1
            {
                return Err(Error::InvalidInput(
                    "Invalid or unordered CP multiplier table",
                ));
            }
            previous = (level, multiplier);
        }
        Ok(catalog)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceStatus {
    pub active: Option<LocalPackage>,
    pub can_rollback: bool,
    pub error: Option<&'static str>,
}

impl InventoryRepository {
    pub(crate) fn local_package(&self) -> Result<Option<LocalPackage>> {
        let text: Option<String> = self
            .connection
            .query_row(
                "SELECT package_json FROM local_reference_slots WHERE slot='active'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        text.map(|text| {
            serde_json::from_str(&text)
                .map_err(|_| Error::InvalidInput("Stored reference package is corrupted"))
        })
        .transpose()
    }

    pub fn reference_status(&self) -> Result<ReferenceStatus> {
        Ok(ReferenceStatus {
            active: self
                .local_package()
                .ok()
                .flatten()
                .filter(|p| p.validate().is_ok()),
            error: self.reference_error,
            can_rollback: self.connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM local_reference_slots WHERE slot='previous')",
                [],
                |r| r.get(0),
            )?,
        })
    }

    pub(crate) fn restore_local_reference(&mut self) -> Result<()> {
        if let Some(package) = self.local_package()? {
            self.catalog = package.validate()?;
        }
        Ok(())
    }

    pub fn install_reference(&mut self, package: LocalPackage) -> Result<ReferenceStatus> {
        let catalog = package.validate()?;
        let data = &package.catalog["dataset"];
        let namespace = data["datasetId"]
            .as_str()
            .ok_or(Error::InvalidInput("Missing namespace"))?;
        let version = data["version"]
            .as_str()
            .ok_or(Error::InvalidInput("Missing version"))?;
        let serialized = serde_json::to_string(&package)
            .map_err(|_| Error::InvalidInput("Invalid reference package"))?;
        // Acquisition time is provenance, not immutable source content. Reusing
        // identical bytes on another day must not look like version tampering.
        let mut content =
            serde_json::to_value(&package).map_err(|_| Error::InvalidInput("Invalid package"))?;
        content["catalog"]["dataset"]["retrievedAt"] = Value::Null;
        content.as_object_mut().unwrap().remove("artifacts");
        let package_digest = digest(&serde_json::to_vec(&content).unwrap());
        let transaction = self.connection.unchecked_transaction()?;
        let seen: Option<String> = transaction
            .query_row(
                "SELECT digest FROM local_reference_versions WHERE dataset_id=?1 AND version=?2",
                params![namespace, version],
                |r| r.get(0),
            )
            .optional()?;
        if seen.as_ref().is_some_and(|old| old != &package_digest) {
            return Err(Error::InvalidInput(
                "Reference version was previously used with different content",
            ));
        }
        if seen.is_none()
            && transaction.query_row("SELECT count(*) FROM local_reference_versions", [], |r| {
                r.get::<_, i64>(0)
            })? >= 4096
        {
            return Err(Error::InvalidInput(
                "Reference version ledger is full; export your data before maintenance",
            ));
        }
        // Cumulative identity continuity survives restart, removal and rollback.
        for species in package.catalog["species"]
            .as_array()
            .ok_or(Error::InvalidInput("Missing species"))?
        {
            let id = species["id"].as_str().unwrap();
            let dex = species["dexNumber"].as_i64().unwrap();
            let old: Option<i64> = transaction
                .query_row(
                    "SELECT dex FROM local_reference_species WHERE dataset_id=?1 AND species_id=?2",
                    params![namespace, id],
                    |r| r.get(0),
                )
                .optional()?;
            let old_id: Option<String> = transaction
                .query_row(
                    "SELECT species_id FROM local_reference_species WHERE dataset_id=?1 AND dex=?2",
                    params![namespace, dex],
                    |r| r.get(0),
                )
                .optional()?;
            if old.is_some_and(|value| value != dex) || old_id.is_some_and(|value| value != id) {
                return Err(Error::InvalidInput(
                    "Species identity changed; explicit mapping review is required",
                ));
            }
            transaction.execute(
                "INSERT OR IGNORE INTO local_reference_species VALUES (?1,?2,?3)",
                params![namespace, id, dex],
            )?;
        }
        for form in package.catalog["forms"]
            .as_array()
            .ok_or(Error::InvalidInput("Missing forms"))?
        {
            let id = form["id"].as_str().unwrap();
            let species = form["speciesId"].as_str().unwrap();
            let old:Option<String> = transaction.query_row("SELECT species_id FROM local_reference_forms WHERE dataset_id=?1 AND form_id=?2",params![namespace,id],|r|r.get(0)).optional()?;
            if old.is_some_and(|value| value != species) {
                return Err(Error::InvalidInput(
                    "Form identity changed; explicit mapping review is required",
                ));
            }
            transaction.execute(
                "INSERT OR IGNORE INTO local_reference_forms VALUES (?1,?2,?3)",
                params![namespace, id, species],
            )?;
        }
        transaction.execute(
            "INSERT OR IGNORE INTO local_reference_versions VALUES (?1,?2,?3)",
            params![namespace, version, package_digest],
        )?;
        let previous: Option<String> = transaction
            .query_row(
                "SELECT package_json FROM local_reference_slots WHERE slot='active'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        if previous.as_ref() != Some(&serialized) {
            if let Some(previous) = previous {
                transaction.execute(
                    "INSERT OR REPLACE INTO local_reference_slots VALUES ('previous',?1)",
                    [previous],
                )?;
            }
            transaction.execute(
                "INSERT OR REPLACE INTO local_reference_slots VALUES ('active',?1)",
                [serialized],
            )?;
        }
        transaction.commit()?;
        self.catalog = catalog;
        self.reference_error = None;
        self.reference_status()
    }

    pub fn rollback_reference(&mut self) -> Result<ReferenceStatus> {
        let previous: Option<String> = self
            .connection
            .query_row(
                "SELECT package_json FROM local_reference_slots WHERE slot='previous'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        let package: LocalPackage = serde_json::from_str(
            &previous.ok_or(Error::InvalidInput("No previous reference package"))?,
        )
        .map_err(|_| Error::InvalidInput("Previous reference package is corrupted"))?;
        self.install_reference(package)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn package() -> LocalPackage {
        LocalPackage {
            catalog: serde_json::from_slice(include_bytes!(
                "../tests/fixtures/reference-synthetic.json"
            ))
            .unwrap(),
            cp_multipliers: vec![(1.0, 0.1), (2.0, 0.2)],
            combat: None,
            coverage: vec![],
            limitations: vec![],
            source_sha256: digest(b"fictional original source"),
            provider: "fictional".into(),
            artifacts: vec![],
        }
    }
    #[test]
    fn atomic_activation_rollback_restart_and_immutable_versions() {
        let path = std::env::temp_dir().join(format!(
            "tcm-reference-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut repository = InventoryRepository::open(&path).unwrap();
        let first = package();
        repository.install_reference(first.clone()).unwrap();
        let mut second = first.clone();
        second.catalog["dataset"]["version"] = Value::String("second".into());
        repository.install_reference(second.clone()).unwrap();
        drop(repository);
        let mut repository = InventoryRepository::open(&path).unwrap();
        assert_eq!(
            repository
                .reference_status()
                .unwrap()
                .active
                .unwrap()
                .catalog,
            second.catalog
        );
        repository.rollback_reference().unwrap();
        assert_eq!(
            repository
                .reference_status()
                .unwrap()
                .active
                .unwrap()
                .catalog,
            first.catalog
        );
        let mut corrupted = second.clone();
        corrupted.catalog["forms"][0]["baseStats"]["attack"] = Value::from(999);
        assert!(repository.install_reference(corrupted).is_err());
        let mut remapped = second;
        remapped.catalog["dataset"]["version"] = Value::String("third".into());
        remapped.catalog["species"][0]["dexNumber"] = Value::from(9999);
        assert!(repository.install_reference(remapped).is_err());
        assert_eq!(
            repository
                .reference_status()
                .unwrap()
                .active
                .unwrap()
                .catalog,
            first.catalog
        );
        drop(repository);
        let _ = std::fs::remove_file(path);
    }
    #[test]
    fn rejects_corrupt_catalog_and_nonfinite_or_duplicate_multipliers() {
        let mut value = package();
        value.cp_multipliers.push((2.0, 0.3));
        assert!(value.validate().is_err());
        let mut value = package();
        value.catalog["schemaVersion"] = Value::from(999);
        assert!(value.validate().is_err());
        let mut value = package();
        value.cp_multipliers[0].1 = f64::NAN;
        assert!(value.validate().is_err());
    }

    #[test]
    fn reimport_date_is_not_content_and_corrupt_active_can_be_recovered() {
        let path = std::env::temp_dir().join(format!(
            "tcm-recovery-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut repo = InventoryRepository::open(&path).unwrap();
        let first = package();
        repo.install_reference(first.clone()).unwrap();
        let mut second = first.clone();
        second.catalog["dataset"]["version"] = Value::String("second".into());
        repo.install_reference(second).unwrap();
        let mut later = first.clone();
        later.catalog["dataset"]["retrievedAt"] = Value::String("2026-11-02".into());
        repo.install_reference(later).unwrap();
        repo.connection
            .execute(
                "UPDATE local_reference_slots SET package_json='{}' WHERE slot='active'",
                [],
            )
            .unwrap();
        drop(repo);
        let mut repo = InventoryRepository::open(&path).unwrap();
        assert!(repo.reference_status().unwrap().error.is_some());
        assert!(repo.reference_status().unwrap().active.is_none());
        assert!(repo.snapshot().is_ok());
        repo.rollback_reference().unwrap();
        assert!(repo.reference_status().unwrap().error.is_none());
        assert_eq!(
            repo.reference_status().unwrap().active.unwrap().catalog["dataset"]["version"],
            "second"
        );
        drop(repo);
        let _ = std::fs::remove_file(path);
    }
}
