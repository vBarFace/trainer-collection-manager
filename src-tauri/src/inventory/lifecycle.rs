use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvolutionTarget {
    pub form_id: String,
    pub species: String,
    pub form: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvolutionOptions {
    pub expected_strategic_observation_revision: String,
    pub dataset_id: String,
    pub dataset_version: String,
    pub targets: Vec<EvolutionTarget>,
    pub unavailable_reason: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationSource {
    Screenshot,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObservationPatch<T> {
    pub value: T,
    pub source: ObservationSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationSnapshot {
    pub identity: Option<Identity>,
    pub species_id: Option<String>,
    pub form_id: Option<String>,
    pub cp: Option<String>,
    pub ivs: Option<AppraisalIvs>,
}
impl From<&Pokemon> for ObservationSnapshot {
    fn from(pokemon: &Pokemon) -> Self {
        Self {
            identity: pokemon.reference.identity.clone(),
            species_id: pokemon
                .reference
                .species
                .as_ref()
                .map(|species| species.id.clone()),
            form_id: pokemon.reference.form.as_ref().map(|form| form.id.clone()),
            cp: pokemon.cp.map(|cp| cp.to_string()),
            ivs: pokemon.ivs,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleEvent {
    pub request_id: String,
    pub pokemon_id: String,
    pub kind: String,
    pub confirmed_at: String,
    pub before: ObservationSnapshot,
    pub after: ObservationSnapshot,
    pub cp_source: Option<ObservationSource>,
    pub ivs_source: Option<ObservationSource>,
    pub observed_identity: Option<Identity>,
    pub same_individual_confirmed: bool,
    pub dataset_id: String,
    pub dataset_version: String,
}

pub struct EvolutionConfirmation<'a> {
    pub request_id: &'a str,
    pub expected_revision: i64,
    pub dataset_id: &'a str,
    pub dataset_version: &'a str,
    pub target_form_id: &'a str,
}

pub struct ReobservationConfirmation<'a> {
    pub request_id: &'a str,
    pub expected_revision: i64,
    pub cp: Option<ObservationPatch<i64>>,
    pub ivs: Option<ObservationPatch<AppraisalIvs>>,
    pub observed_identity: Option<Identity>,
    pub same_individual_confirmed: bool,
}

impl InventoryRepository {
    pub fn evolution_options(&self, id: &PokemonId) -> Result<EvolutionOptions> {
        let pokemon = self.get(id)?;
        let targets: Vec<_> = self
            .catalog
            .evolution_targets(&pokemon.reference)
            .into_iter()
            .map(|(species, form)| EvolutionTarget {
                form_id: form.id,
                species: species.name,
                form: form.name,
            })
            .collect();
        let unavailable_reason = if pokemon.reference.state != ResolutionState::Resolved {
            Some("Confirm a species and form supported by the local reference catalog before evolving. Reference data may be unavailable.")
        } else if pokemon
            .reference
            .form
            .as_ref()
            .is_none_or(|form| form.evolutions.is_none())
        {
            Some("Evolution relationships are unknown in the local reference catalog.")
        } else if targets.is_empty() {
            Some("The local reference catalog contains no supported permanent species evolution for this form.")
        } else {
            None
        };
        Ok(EvolutionOptions {
            expected_strategic_observation_revision: pokemon
                .strategic_observation_revision
                .to_string(),
            dataset_id: pokemon.reference.dataset.dataset_id,
            dataset_version: pokemon.reference.dataset.version,
            targets,
            unavailable_reason,
        })
    }

    fn check_confirmation(
        &self,
        id: &PokemonId,
        request_id: &str,
        expected_revision: i64,
    ) -> Result<Pokemon> {
        PokemonId::parse(request_id)?;
        if expected_revision < 0 {
            return Err(Error::InvalidInput(
                "Expected a nonnegative observation revision",
            ));
        }
        let existing = self.get(id)?;
        let repeated: bool = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM pokemon_lifecycle_events WHERE request_id = ?1)",
            [request_id],
            |row| row.get(0),
        )?;
        if repeated {
            return Err(Error::AlreadyConfirmed);
        }
        if existing.strategic_observation_revision != expected_revision {
            return Err(Error::ObservationConflict);
        }
        Ok(existing)
    }

    fn append_event(&self, event: &LifecycleEvent) -> Result<()> {
        let json = serde_json::to_string(event)
            .map_err(|_| Error::InvalidInput("Unable to encode confirmed lifecycle history"))?;
        self.connection.execute("INSERT INTO pokemon_lifecycle_events (request_id, pokemon_id, kind, event_json) VALUES (?1, ?2, ?3, ?4)", params![event.request_id, event.pokemon_id, event.kind, json])?;
        Ok(())
    }

    pub fn confirm_evolution(
        &self,
        id: &PokemonId,
        confirmation: EvolutionConfirmation<'_>,
    ) -> Result<(Pokemon, Vec<Tag>)> {
        let transaction = rusqlite::Transaction::new_unchecked(
            &self.connection,
            rusqlite::TransactionBehavior::Immediate,
        )?;
        let before =
            self.check_confirmation(id, confirmation.request_id, confirmation.expected_revision)?;
        if before.reference.dataset.dataset_id != confirmation.dataset_id
            || before.reference.dataset.version != confirmation.dataset_version
        {
            return Err(Error::ObservationConflict);
        }
        let (species, form) = self.catalog.evolution_targets(&before.reference).into_iter().find(|(_, form)| form.id == confirmation.target_form_id)
            .ok_or(Error::InvalidInput("Select an explicitly supported permanent evolution target from the local reference catalog"))?;
        // CP belongs to the former species; IV appraisal and user metadata survive.
        // Existing schema has no moves, observed HP or level to copy or fabricate.
        self.connection.execute(
            "UPDATE pokemon SET cp = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            [id.as_str()],
        )?;
        self.connection.execute("UPDATE pokemon_reference_identity SET species_query = ?2, form_query = ?3, dataset_id = ?4, species_id = ?5, form_id = ?6, confirmed_at = CURRENT_TIMESTAMP WHERE pokemon_id = ?1", params![id.as_str(), species.name, form.name, confirmation.dataset_id, species.id, form.id])?;
        // Migration 6 observes query text, while a confirmed evolution also changes
        // pinned species identity. Equal labels must not revive a pre-evolution draft.
        if self.get(id)?.strategic_observation_revision == before.strategic_observation_revision {
            if self.connection.execute(
                "UPDATE pokemon SET strategic_observation_revision = strategic_observation_revision + 1 WHERE id = ?1 AND strategic_observation_revision < 9223372036854775807",
                [id.as_str()],
            )? == 0 {
                return Err(Error::InvalidInput("The observation revision is exhausted; the evolution was not saved"));
            }
        }
        let after = self.get(id)?;
        self.append_event(&LifecycleEvent {
            request_id: confirmation.request_id.into(),
            pokemon_id: id.as_str().into(),
            kind: "evolution".into(),
            confirmed_at: String::new(),
            before: (&before).into(),
            after: (&after).into(),
            cp_source: None,
            ivs_source: None,
            observed_identity: None,
            same_individual_confirmed: true,
            dataset_id: confirmation.dataset_id.into(),
            dataset_version: confirmation.dataset_version.into(),
        })?;
        let tags = self.tags_for(id)?;
        transaction.commit()?;
        Ok((after, tags))
    }

    pub fn reobserve(
        &self,
        id: &PokemonId,
        confirmation: ReobservationConfirmation<'_>,
    ) -> Result<(Pokemon, Vec<Tag>)> {
        if !confirmation.same_individual_confirmed {
            return Err(Error::InvalidInput(
                "Confirm that the screenshot shows this individual and its current species/form",
            ));
        }
        if confirmation.cp.is_none() && confirmation.ivs.is_none() {
            return Err(Error::InvalidInput(
                "Select at least one reviewed observation to update",
            ));
        }
        if confirmation
            .cp
            .as_ref()
            .is_some_and(|patch| patch.value <= 0)
        {
            return Err(Error::InvalidInput("Observed CP must be positive"));
        }
        if let Some(patch) = &confirmation.ivs {
            validate_ivs(Some(patch.value))?;
        }
        if let Some(identity) = &confirmation.observed_identity {
            identity.validate().map_err(Error::InvalidInput)?;
        }
        let transaction = rusqlite::Transaction::new_unchecked(
            &self.connection,
            rusqlite::TransactionBehavior::Immediate,
        )?;
        let before =
            self.check_confirmation(id, confirmation.request_id, confirmation.expected_revision)?;
        if before.reference.identity != confirmation.observed_identity {
            return Err(Error::InvalidInput("The reviewed screenshot species/form differs from the selected individual. Resolve the mismatch before updating observations; a screenshot cannot confirm an evolution."));
        }
        let cp = confirmation
            .cp
            .as_ref()
            .map(|patch| patch.value)
            .or(before.cp);
        let ivs = confirmation
            .ivs
            .as_ref()
            .map(|patch| patch.value)
            .or(before.ivs);
        if cp != before.cp || ivs != before.ivs {
            self.connection.execute("UPDATE pokemon SET cp = ?2, iv_attack = ?3, iv_defense = ?4, iv_hp = ?5, updated_at = CURRENT_TIMESTAMP WHERE id = ?1", params![id.as_str(), cp, ivs.map(|ivs| ivs.attack), ivs.map(|ivs| ivs.defense), ivs.map(|ivs| ivs.hp)])?;
        }
        let after = self.get(id)?;
        self.append_event(&LifecycleEvent {
            request_id: confirmation.request_id.into(),
            pokemon_id: id.as_str().into(),
            kind: "reobservation".into(),
            confirmed_at: String::new(),
            before: (&before).into(),
            after: (&after).into(),
            cp_source: confirmation.cp.as_ref().map(|patch| patch.source),
            ivs_source: confirmation.ivs.as_ref().map(|patch| patch.source),
            observed_identity: confirmation.observed_identity,
            same_individual_confirmed: true,
            dataset_id: before.reference.dataset.dataset_id,
            dataset_version: before.reference.dataset.version,
        })?;
        let tags = self.tags_for(id)?;
        transaction.commit()?;
        Ok((after, tags))
    }

    pub fn history(&self, id: &PokemonId) -> Result<Vec<LifecycleEvent>> {
        self.get(id)?;
        let mut statement = self.connection.prepare("SELECT event_json, confirmed_at FROM pokemon_lifecycle_events WHERE pokemon_id = ?1 ORDER BY rowid DESC")?;
        let events = statement
            .query_map([id.as_str()], |row| {
                let json: String = row.get(0)?;
                let mut event: LifecycleEvent =
                    serde_json::from_str(&json).map_err(|_| rusqlite::Error::InvalidQuery)?;
                event.confirmed_at = row.get(1)?;
                Ok(event)
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(events)
    }
}

#[cfg(test)]
mod tests;
