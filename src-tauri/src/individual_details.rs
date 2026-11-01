//! Explicit optional individual observations. Every non-null field is manually
//! confirmed; screenshot resource counts never become player-wide budgets.
use crate::inventory::{Error, InventoryRepository, PokemonId, Result};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IndividualDetails {
    pub current_hp: Option<u32>,
    pub maximum_hp: Option<u32>,
    pub weight_kg: Option<f64>,
    pub height_m: Option<f64>,
    pub gender: Option<String>,
    pub catch_date: Option<String>,
    pub fast_move: Option<String>,
    pub charged_move: Option<String>,
    pub second_charged_move: Option<String>,
    pub level: Option<f64>,
    pub level_evidence: Option<String>,
    pub shiny: Option<bool>,
    pub shadow: Option<bool>,
    pub purified: Option<bool>,
    pub favorite: Option<bool>,
    pub protected: bool,
}

impl IndividualDetails {
    fn validate(&self) -> Result<()> {
        if self.current_hp.is_some_and(|v| v > 100000)
            || self.maximum_hp.is_some_and(|v| v == 0 || v > 100000)
            || matches!((self.current_hp,self.maximum_hp),(Some(a),Some(b)) if a>b)
            || [self.weight_kg, self.height_m]
                .iter()
                .flatten()
                .any(|v| !v.is_finite() || *v <= 0.0 || *v > 1_000_000.0)
            || self.level.is_some_and(|v| {
                !v.is_finite() || !(1.0..=51.0).contains(&v) || (v * 2.0).fract() != 0.0
            })
            || (self.level.is_some() && self.level_evidence.is_none())
            || self
                .gender
                .as_ref()
                .is_some_and(|v| !matches!(v.as_str(), "male" | "female" | "genderless"))
            || self.shadow == Some(true) && self.purified == Some(true)
            || [
                &self.fast_move,
                &self.charged_move,
                &self.second_charged_move,
                &self.level_evidence,
            ]
            .iter()
            .filter_map(|v| v.as_ref())
            .any(|v| !crate::reference::valid_text(v, 100))
        {
            return Err(Error::InvalidInput(
                "Invalid optional observation; check HP, units, level evidence and variants",
            ));
        }
        if let Some(date) = &self.catch_date {
            // SQLite's date parser also rejects malformed calendar dates.
            if date.len() != 10
                || date.as_bytes()[4] != b'-'
                || date.as_bytes()[7] != b'-'
                || !date
                    .bytes()
                    .enumerate()
                    .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
            {
                return Err(Error::InvalidInput("Catch date must be YYYY-MM-DD"));
            }
        }
        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailsView {
    pub values: IndividualDetails,
    pub source: &'static str,
    pub observation_revision: String,
    pub saved_at_revision: Option<String>,
    pub needs_reconfirmation: bool,
}

impl InventoryRepository {
    pub fn confirm_batch_import(
        &self,
        request_id: &str,
        input: &crate::inventory::PokemonInput,
        tags: &[crate::inventory::TagId],
    ) -> Result<(crate::inventory::Pokemon, Vec<crate::inventory::Tag>)> {
        PokemonId::parse(request_id)?;
        let mut ids = tags.iter().map(|id| id.as_i64()).collect::<Vec<_>>();
        ids.sort();
        ids.dedup();
        let input_digest=crate::reference_store::digest(&serde_json::to_vec(&serde_json::json!({"name":input.display_name,"cp":input.cp.map(|v|v.to_string()),"ivs":input.ivs,"tags":ids})).unwrap());
        let transaction = rusqlite::Transaction::new_unchecked(
            &self.connection,
            rusqlite::TransactionBehavior::Immediate,
        )?;
        let previous: Option<(String, String)> = self
            .connection
            .query_row(
                "SELECT pokemon_id,input_digest FROM pokemon_import_requests WHERE request_id=?1",
                [request_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        if let Some((id, digest)) = previous {
            if digest != input_digest {
                return Err(Error::InvalidInput("This import request already confirmed different facts. Reload the inventory before starting a new import."));
            }
            let id = PokemonId::parse(&id)?;
            let result = (self.get(&id)?, self.tags_for(&id)?);
            transaction.commit()?;
            return Ok(result);
        }
        let pokemon = self.create(input)?;
        for tag in tags {
            self.assign_tag(&pokemon.id, *tag)?;
        }
        self.connection.execute(
            "INSERT INTO pokemon_import_requests VALUES (?1,?2,?3)",
            params![request_id, pokemon.id.as_str(), input_digest],
        )?;
        let result = (pokemon.clone(), self.tags_for(&pokemon.id)?);
        transaction.commit()?;
        Ok(result)
    }

    pub fn confirm_tags(
        &self,
        id: &PokemonId,
        expected: &[String],
        desired: &[String],
    ) -> Result<()> {
        if expected.len() > 100 || desired.len() > 100 {
            return Err(Error::InvalidInput("Too many tags"));
        }
        let transaction = self.connection.unchecked_transaction()?;
        self.get(id)?;
        let mut current = self
            .tags_for(id)?
            .iter()
            .map(|tag| tag.id.as_i64().to_string())
            .collect::<Vec<_>>();
        current.sort();
        let mut expected = expected.to_vec();
        expected.sort();
        if current != expected {
            return Err(Error::ObservationConflict);
        }
        let desired = desired
            .iter()
            .map(|value| crate::inventory::TagId::parse(value))
            .collect::<Result<Vec<_>>>()?;
        for value in &current {
            self.remove_tag(id, crate::inventory::TagId::parse(value)?)?;
        }
        for tag in desired {
            self.assign_tag(id, tag)?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn individual_details(&self, id: &PokemonId) -> Result<DetailsView> {
        let pokemon = self.get(id)?;
        let stored: Option<(String, i64, bool)> = self
            .connection
            .query_row(
                "SELECT details_json,observation_revision,invalidated FROM pokemon_details WHERE pokemon_id=?1",
                [id.as_str()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        let (values, saved, invalidated) = match stored {
            Some((json, revision, invalidated)) => (
                serde_json::from_str(&json)
                    .map_err(|_| Error::InvalidInput("Invalid stored individual observations"))?,
                Some(revision),
                invalidated,
            ),
            None => (IndividualDetails::default(), None, false),
        };
        Ok(DetailsView {
            values,
            source: "manually_confirmed",
            observation_revision: pokemon.strategic_observation_revision.to_string(),
            saved_at_revision: saved.map(|v| v.to_string()),
            needs_reconfirmation: invalidated
                || saved.is_some_and(|v| v != pokemon.strategic_observation_revision),
        })
    }

    pub fn save_individual_details(
        &self,
        id: &PokemonId,
        expected_revision: i64,
        values: IndividualDetails,
    ) -> Result<DetailsView> {
        values.validate()?;
        if let Some(date) = &values.catch_date {
            let valid: bool =
                self.connection
                    .query_row("SELECT date(?1,'+0 days') IS ?1", [date], |r| r.get(0))?;
            if !valid {
                return Err(Error::InvalidInput(
                    "Catch date is not a valid calendar date",
                ));
            }
        }
        let transaction = self.connection.unchecked_transaction()?;
        let pokemon = self.get(id)?;
        if pokemon.strategic_observation_revision != expected_revision {
            return Err(Error::ObservationConflict);
        }
        let current = self.individual_details(id)?;
        if current.values != values || current.needs_reconfirmation {
            let revision = expected_revision
                .checked_add(1)
                .ok_or(Error::InvalidInput("Observation revision overflow"))?;
            self.connection.execute("UPDATE pokemon SET strategic_observation_revision=?2,updated_at=CURRENT_TIMESTAMP WHERE id=?1",params![id.as_str(),revision])?;
            self.connection.execute("INSERT INTO pokemon_details VALUES (?1,?2,?3,0) ON CONFLICT(pokemon_id) DO UPDATE SET details_json=excluded.details_json,observation_revision=excluded.observation_revision,invalidated=0",params![id.as_str(),serde_json::to_string(&values).unwrap(),revision])?;
        }
        transaction.commit()?;
        self.individual_details(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn batch_import_retry_survives_restart_without_duplicate_uuid() {
        let path = std::env::temp_dir().join(format!(
            "tcm-batch-retry-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let request = "00000000-0000-4000-8000-000000000012";
        let input = crate::inventory::PokemonInput {
            display_name: "Fictional".into(),
            cp: None,
            ivs: None,
        };
        let repo = InventoryRepository::open(&path).unwrap();
        let first = repo.confirm_batch_import(request, &input, &[]).unwrap().0;
        drop(repo);
        let repo = InventoryRepository::open(&path).unwrap();
        let retry = repo.confirm_batch_import(request, &input, &[]).unwrap().0;
        assert_eq!(first.id, retry.id);
        assert_eq!(repo.list().unwrap().len(), 1);
        let mut changed = input.clone();
        changed.cp = Some(123);
        assert!(repo.confirm_batch_import(request, &changed, &[]).is_err());
        assert_eq!(repo.get(&first.id).unwrap().cp, None);
        assert!(repo
            .confirm_batch_import(
                "00000000-0000-4000-8000-000000000013",
                &input,
                &[crate::inventory::TagId::parse("999999").unwrap()]
            )
            .is_err());
        assert_eq!(repo.list().unwrap().len(), 1);
        drop(repo);
        let _ = std::fs::remove_file(path);
    }
    #[test]
    fn history_only_power_up_requires_optional_evidence_reconfirmation() {
        let path = std::env::temp_dir().join(format!(
            "tcm-details-power-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let repo = InventoryRepository::open(&path).unwrap();
        let pokemon = repo
            .create(&crate::inventory::PokemonInput {
                display_name: "Fictional".into(),
                cp: Some(500),
                ivs: None,
            })
            .unwrap();
        repo.save_individual_details(
            &pokemon.id,
            0,
            IndividualDetails {
                level: Some(10.0),
                level_evidence: Some("Explicit user evidence".into()),
                ..Default::default()
            },
        )
        .unwrap();
        repo.confirm_power_up(
            &pokemon.id,
            crate::inventory::PowerUpConfirmation {
                request_id: "00000000-0000-4000-8000-000000000011",
                expected_revision: 1,
                cp: None,
                performed_in_game_confirmed: true,
            },
        )
        .unwrap();
        assert_eq!(
            repo.get(&pokemon.id)
                .unwrap()
                .strategic_observation_revision,
            1
        );
        assert!(
            repo.individual_details(&pokemon.id)
                .unwrap()
                .needs_reconfirmation
        );
        assert_eq!(
            repo.individual_details(&pokemon.id).unwrap().values.level,
            Some(10.0)
        );
        drop(repo);
        let _ = std::fs::remove_file(path);
    }
    #[test]
    fn uncertain_variants_and_optional_fields_stay_unknown_and_invalid_values_fail() {
        let mut details = IndividualDetails::default();
        assert_eq!(details.shiny, None);
        assert!(details.validate().is_ok());
        details.level = Some(20.0);
        assert!(details.validate().is_err());
        details.level_evidence = Some("Manually checked level".into());
        assert!(details.validate().is_ok());
        details.shadow = Some(true);
        details.purified = Some(true);
        assert!(details.validate().is_err());
        details.purified = None;
        details.current_hp = Some(99);
        details.maximum_hp = Some(50);
        assert!(details.validate().is_err());
    }

    #[test]
    fn confirmed_details_survive_restart_and_conflicts_preserve_facts_and_uuid() {
        let path = std::env::temp_dir().join(format!(
            "tcm-details-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let repo = InventoryRepository::open(&path).unwrap();
        let pokemon = repo
            .create(&crate::inventory::PokemonInput {
                display_name: "Original fictional individual".into(),
                cp: Some(500),
                ivs: None,
            })
            .unwrap();
        let details = IndividualDetails {
            current_hp: Some(47),
            maximum_hp: Some(47),
            shiny: None,
            favorite: Some(true),
            protected: true,
            catch_date: Some("2026-10-23".into()),
            ..Default::default()
        };
        let saved = repo
            .save_individual_details(&pokemon.id, 0, details.clone())
            .unwrap();
        assert_eq!(saved.observation_revision, "1");
        assert!(!saved.needs_reconfirmation);
        assert!(repo
            .save_individual_details(&pokemon.id, 0, IndividualDetails::default())
            .is_err());
        let noop = repo
            .save_individual_details(&pokemon.id, 1, details.clone())
            .unwrap();
        assert_eq!(noop.observation_revision, "1");
        let mut invalid = details.clone();
        invalid.catch_date = Some("2026-02-30".into());
        assert!(repo
            .save_individual_details(&pokemon.id, 1, invalid)
            .is_err());
        drop(repo);
        let repo = InventoryRepository::open(&path).unwrap();
        assert_eq!(
            repo.individual_details(&pokemon.id).unwrap().values,
            details
        );
        repo.correct_cp(&pokemon.id, Some(500), 1, Some(550))
            .unwrap();
        assert!(
            repo.individual_details(&pokemon.id)
                .unwrap()
                .needs_reconfirmation
        );
        assert_eq!(repo.list().unwrap().len(), 1);
        assert_eq!(repo.list().unwrap()[0].id, pokemon.id);
        drop(repo);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn user_applied_tags_are_atomic_and_reject_stale_assignments() {
        let path = std::env::temp_dir().join(format!(
            "tcm-tags-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let repo = InventoryRepository::open(&path).unwrap();
        let pokemon = repo
            .create(&crate::inventory::PokemonInput {
                display_name: "Fiction".into(),
                cp: None,
                ivs: None,
            })
            .unwrap();
        let tag = repo.create_custom_tag("Personal goal").unwrap();
        let ids = vec![tag.id.as_i64().to_string()];
        repo.confirm_tags(&pokemon.id, &[], &ids).unwrap();
        assert!(repo.confirm_tags(&pokemon.id, &[], &[]).is_err());
        assert!(repo
            .confirm_tags(&pokemon.id, &ids, &["999999".into()])
            .is_err());
        assert_eq!(repo.tags_for(&pokemon.id).unwrap()[0].id, tag.id);
        assert_eq!(
            repo.get(&pokemon.id)
                .unwrap()
                .strategic_observation_revision,
            0
        );
        drop(repo);
        let _ = std::fs::remove_file(path);
    }
}
