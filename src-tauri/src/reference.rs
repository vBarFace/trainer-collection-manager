//! Validated offline snapshots. This module never reads player data or uses a network.
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Dataset {
    pub dataset_id: String,
    pub version: String,
    pub source_url: String,
    pub upstream_revision: String,
    pub license: String,
    pub notice: String,
    pub retrieved_at: String,
    pub rights_evidence_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Species {
    pub id: String,
    pub dex_number: u16,
    pub name: String,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PokemonType {
    Normal,
    Fire,
    Water,
    Electric,
    Grass,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
    Dark,
    Steel,
    Fairy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaseStats {
    pub attack: u16,
    pub defense: u16,
    pub stamina: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Evolution {
    pub target_form_id: String,
    #[serde(deserialize_with = "Option::<Vec<String>>::deserialize")]
    pub requirements: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Form {
    pub id: String,
    pub species_id: String,
    pub name: String,
    pub aliases: Vec<String>,
    #[serde(deserialize_with = "Option::<Vec<PokemonType>>::deserialize")]
    pub types: Option<Vec<PokemonType>>,
    #[serde(deserialize_with = "Option::<BaseStats>::deserialize")]
    pub base_stats: Option<BaseStats>,
    #[serde(deserialize_with = "Option::<Vec<Evolution>>::deserialize")]
    pub evolutions: Option<Vec<Evolution>>,
}

/// Explicitly confirmed identity, separate from display labels and tentative OCR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub species: String,
    #[serde(deserialize_with = "Option::<String>::deserialize")]
    pub form: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    pub dataset_id: String,
    pub species_id: String,
    pub form_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionState {
    NotSet,
    Unavailable,
    UnknownSpecies,
    UnknownForm,
    Ambiguous,
    Resolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resolution {
    pub state: ResolutionState,
    pub identity: Option<Identity>,
    pub dataset: Dataset,
    pub species: Option<Species>,
    pub form: Option<Form>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    schema_version: u16,
    dataset: Dataset,
    species: Vec<Species>,
    forms: Vec<Form>,
    seen_versions: HashSet<(String, String)>,
    species_identities: HashMap<(String, String), u16>,
    dex_identities: HashMap<(String, u16), String>,
    form_identities: HashMap<(String, String), String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CatalogDocument {
    schema_version: u16,
    dataset: Dataset,
    species: Vec<Species>,
    forms: Vec<Form>,
}

pub type Result<T> = std::result::Result<T, &'static str>;

pub fn valid_text(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.chars().count() <= max
        && !value.chars().any(char::is_control)
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_:".contains(&b))
}

fn valid_aliases(name: &str, aliases: &[String]) -> bool {
    let mut names = HashSet::from([name.to_lowercase()]);
    valid_text(name, 100)
        && aliases
            .iter()
            .all(|a| valid_text(a, 100) && names.insert(a.to_lowercase()))
}

impl Identity {
    pub fn validate(&self) -> Result<()> {
        if valid_text(&self.species, 100) && self.form.as_ref().is_none_or(|v| valid_text(v, 100)) {
            Ok(())
        } else {
            Err("Species and form must be trimmed text of 1–100 characters without control characters")
        }
    }
}

impl Catalog {
    /// Only reviewed directed evolution edges between different species are
    /// actionable. Same-species form changes are outside this lifecycle scope.
    pub fn evolution_targets(&self, resolved: &Resolution) -> Vec<(Species, Form)> {
        if resolved.state != ResolutionState::Resolved {
            return vec![];
        }
        let Some(source) = resolved.form.as_ref() else {
            return vec![];
        };
        source
            .evolutions
            .iter()
            .flatten()
            .filter_map(|edge| {
                let form = self
                    .forms
                    .iter()
                    .find(|form| form.id == edge.target_form_id)?;
                if form.species_id == source.species_id {
                    return None;
                }
                let species = self
                    .species
                    .iter()
                    .find(|species| species.id == form.species_id)?;
                Some((species.clone(), form.clone()))
            })
            .collect()
    }

    pub fn bundled() -> Result<Self> {
        Self::load(include_bytes!("../reference/catalog.json"))
    }

    /// Parse/validate everything before making the snapshot available to callers.
    pub fn load(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 16 * 1024 * 1024 {
            return Err("Reference snapshot is too large");
        }
        let document: CatalogDocument =
            serde_json::from_slice(bytes).map_err(|_| "Malformed reference snapshot")?;
        let catalog = Self {
            schema_version: document.schema_version,
            dataset: document.dataset,
            species: document.species,
            forms: document.forms,
            seen_versions: HashSet::new(),
            species_identities: HashMap::new(),
            dex_identities: HashMap::new(),
            form_identities: HashMap::new(),
        };
        catalog.validate()?;
        let mut catalog = catalog;
        catalog.seen_versions.insert((
            catalog.dataset.dataset_id.clone(),
            catalog.dataset.version.clone(),
        ));
        catalog.remember_identities();
        Ok(catalog)
    }

    /// Versions are opaque pinned upstream identifiers, never lexically ordered.
    /// A version cannot silently acquire different content during this session.
    pub fn activate(&mut self, bytes: &[u8]) -> Result<()> {
        let mut next = Self::load(bytes)?;
        if self.dataset.dataset_id == next.dataset.dataset_id
            && self.dataset.version == next.dataset.version
        {
            if self.schema_version != next.schema_version
                || self.dataset != next.dataset
                || self.species != next.species
                || self.forms != next.forms
            {
                return Err("Reference version reused with different content");
            }
            return Ok(());
        }
        let key = (
            next.dataset.dataset_id.clone(),
            next.dataset.version.clone(),
        );
        // Prevent A -> B -> modified A without retaining entire old snapshots.
        // A rollback in this native session must use a fresh version identifier.
        if self.seen_versions.contains(&key) {
            return Err("Previously active reference version cannot be reactivated");
        }
        // Keep identity history even when a snapshot temporarily omits records.
        // Renames and metadata corrections are allowed; retargeting IDs requires
        // a separately reviewed mapping rather than rebinding saved inventory.
        let namespace = &next.dataset.dataset_id;
        for species in &next.species {
            if self
                .species_identities
                .get(&(namespace.clone(), species.id.clone()))
                .is_some_and(|dex| *dex != species.dex_number)
                || self
                    .dex_identities
                    .get(&(namespace.clone(), species.dex_number))
                    .is_some_and(|id| *id != species.id)
            {
                return Err(
                    "Reference species identifier changed its National Pokedex association",
                );
            }
        }
        for form in &next.forms {
            if self
                .form_identities
                .get(&(namespace.clone(), form.id.clone()))
                .is_some_and(|species_id| *species_id != form.species_id)
            {
                return Err("Reference form identifier changed its species association");
            }
        }
        next.seen_versions = self.seen_versions.clone();
        next.seen_versions.insert(key);
        next.species_identities = self.species_identities.clone();
        next.dex_identities = self.dex_identities.clone();
        next.form_identities = self.form_identities.clone();
        next.remember_identities();
        *self = next;
        Ok(())
    }

    fn remember_identities(&mut self) {
        let namespace = &self.dataset.dataset_id;
        for species in &self.species {
            self.species_identities
                .insert((namespace.clone(), species.id.clone()), species.dex_number);
            self.dex_identities
                .insert((namespace.clone(), species.dex_number), species.id.clone());
        }
        for form in &self.forms {
            self.form_identities.insert(
                (namespace.clone(), form.id.clone()),
                form.species_id.clone(),
            );
        }
    }

    fn validate(&self) -> Result<()> {
        let d = &self.dataset;
        if self.schema_version != 1 {
            return Err("Unsupported reference schema version");
        }
        if !valid_id(&d.dataset_id)
            || [
                &d.version,
                &d.upstream_revision,
                &d.license,
                &d.notice,
                &d.retrieved_at,
            ]
            .iter()
            .any(|v| !valid_text(v, 2000))
            || [&d.source_url, &d.rights_evidence_url]
                .iter()
                .any(|v| !valid_text(v, 2000) || !v.starts_with("https://"))
        {
            return Err("Missing reference provenance");
        }
        let mut species_ids = HashSet::new();
        let mut dex_numbers = HashSet::new();
        for s in &self.species {
            if !valid_id(&s.id)
                || !species_ids.insert(&s.id)
                || s.dex_number == 0
                || !dex_numbers.insert(s.dex_number)
                || !valid_aliases(&s.name, &s.aliases)
            {
                return Err("Invalid or duplicate species");
            }
        }
        let mut form_ids = HashSet::new();
        for f in &self.forms {
            if !valid_id(&f.id)
                || !form_ids.insert(&f.id)
                || !species_ids.contains(&f.species_id)
                || !valid_aliases(&f.name, &f.aliases)
                || f.types
                    .as_ref()
                    .is_some_and(|t| t.is_empty() || t.len() > 2 || (t.len() == 2 && t[0] == t[1]))
                || f.base_stats
                    .as_ref()
                    .is_some_and(|s| s.attack == 0 || s.defense == 0 || s.stamina == 0)
            {
                return Err("Invalid or duplicate form");
            }
        }
        for f in &self.forms {
            let mut targets = HashSet::new();
            for e in f.evolutions.iter().flatten() {
                if !form_ids.contains(&e.target_form_id)
                    || !targets.insert(&e.target_form_id)
                    || e.requirements
                        .as_ref()
                        .is_some_and(|r| r.iter().any(|v| !valid_text(v, 500)))
                {
                    return Err("Invalid evolution relationship");
                }
            }
        }
        // Kahn's algorithm rejects cycles without recursive parsing/stack growth.
        let mut indegree = vec![0usize; self.forms.len()];
        let indices: std::collections::HashMap<_, _> = self
            .forms
            .iter()
            .enumerate()
            .map(|(i, f)| (f.id.as_str(), i))
            .collect();
        for f in &self.forms {
            for e in f.evolutions.iter().flatten() {
                indegree[indices[e.target_form_id.as_str()]] += 1;
            }
        }
        let mut ready: Vec<_> = indegree
            .iter()
            .enumerate()
            .filter(|(_, n)| **n == 0)
            .map(|(i, _)| i)
            .collect();
        let mut visited = 0;
        while let Some(i) = ready.pop() {
            visited += 1;
            for e in self.forms[i].evolutions.iter().flatten() {
                let target = indices[e.target_form_id.as_str()];
                indegree[target] -= 1;
                if indegree[target] == 0 {
                    ready.push(target);
                }
            }
        }
        if visited != self.forms.len() {
            return Err("Cyclic evolution relationship");
        }
        Ok(())
    }

    pub fn resolve(&self, identity: Option<&Identity>, pin: Option<&Pin>) -> Resolution {
        let mut result = Resolution {
            state: ResolutionState::NotSet,
            identity: identity.cloned(),
            dataset: self.dataset.clone(),
            species: None,
            form: None,
        };
        let Some(identity) = identity else {
            return result;
        };
        if self.species.is_empty() || pin.is_some_and(|p| p.dataset_id != self.dataset.dataset_id) {
            result.state = ResolutionState::Unavailable;
            return result;
        }
        let species_key = pin.map_or(identity.species.as_str(), |p| p.species_id.as_str());
        let species: Vec<_> = if let Some(s) = self.species.iter().find(|s| s.id == species_key) {
            vec![s]
        } else if pin.is_some() {
            vec![]
        } else {
            self.species
                .iter()
                .filter(|s| name_matches(species_key, &s.name, &s.aliases))
                .collect()
        };
        result.state = ResolutionState::UnknownSpecies;
        if species.len() > 1 {
            result.state = ResolutionState::Ambiguous;
            return result;
        }
        let Some(s) = species.first() else {
            return result;
        };
        result.species = Some((*s).clone());
        result.state = ResolutionState::UnknownForm;
        // An absent form is unknown even when the snapshot contains only one form.
        let Some(form_key) = pin.map(|p| p.form_id.as_str()).or(identity.form.as_deref()) else {
            return result;
        };
        let forms: Vec<_> = if let Some(f) = self
            .forms
            .iter()
            .find(|f| f.id == form_key && f.species_id == s.id)
        {
            vec![f]
        } else if pin.is_some() {
            vec![]
        } else {
            self.forms
                .iter()
                .filter(|f| f.species_id == s.id && name_matches(form_key, &f.name, &f.aliases))
                .collect()
        };
        if forms.len() > 1 {
            result.state = ResolutionState::Ambiguous;
            return result;
        }
        if let Some(f) = forms.first() {
            result.form = Some((*f).clone());
            result.state = ResolutionState::Resolved;
        }
        result
    }
}

fn name_matches(query: &str, name: &str, aliases: &[String]) -> bool {
    let query = query.to_lowercase();
    query == name.to_lowercase() || aliases.iter().any(|a| a.to_lowercase() == query)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub const FIXTURE: &[u8] = include_bytes!("../tests/fixtures/reference-synthetic.json");
    pub fn identity(species: &str, form: Option<&str>) -> Identity {
        Identity {
            species: species.into(),
            form: form.map(String::from),
        }
    }
    pub fn document() -> serde_json::Value {
        serde_json::from_slice(FIXTURE).unwrap()
    }

    #[test]
    fn original_fixture_resolves_ids_names_aliases_and_preserves_unknowns() {
        let catalog = Catalog::load(FIXTURE).unwrap();
        for query in [
            identity("testling", Some("testling-normal")),
            identity("eSSai", Some("bASe")),
        ] {
            let resolved = catalog.resolve(Some(&query), None);
            assert_eq!(resolved.state, ResolutionState::Resolved);
            assert_eq!(resolved.identity.as_ref(), Some(&query));
            let form = resolved.form.unwrap();
            assert_eq!(form.types, Some(vec![PokemonType::Fire]));
            assert_eq!(form.base_stats.unwrap().stamina, 103);
            assert_eq!(
                form.evolutions.unwrap()[0].target_form_id,
                "testwing-normal"
            );
        }
        let dual = catalog
            .resolve(Some(&identity("Testwing", Some("Normal"))), None)
            .form
            .unwrap();
        assert_eq!(
            dual.types,
            Some(vec![PokemonType::Fire, PokemonType::Flying])
        );
        assert_eq!(dual.base_stats, None);
        assert_eq!(dual.evolutions, Some(vec![]));
        let missing = catalog
            .resolve(Some(&identity("Testling", Some("Alternate"))), None)
            .form
            .unwrap();
        assert_eq!(missing.types, None);
        assert_eq!(missing.evolutions, None);
    }

    #[test]
    fn no_guessing_from_ambiguous_unknown_or_missing_queries() {
        let catalog = Catalog::load(FIXTURE).unwrap();
        for (query, state) in [
            (
                identity("Shared", Some("Normal")),
                ResolutionState::Ambiguous,
            ),
            (
                identity("Testling", Some("Shared form")),
                ResolutionState::Ambiguous,
            ),
            (
                identity("Missing", Some("Normal")),
                ResolutionState::UnknownSpecies,
            ),
            (identity("Testwing", None), ResolutionState::UnknownForm),
            (
                identity("Testling", Some("Absent")),
                ResolutionState::UnknownForm,
            ),
        ] {
            let resolved = catalog.resolve(Some(&query), None);
            assert_eq!(resolved.state, state);
            assert!(resolved.form.is_none());
            assert_eq!(resolved.identity, Some(query));
        }
        assert_eq!(catalog.resolve(None, None).state, ResolutionState::NotSet);
        assert_eq!(
            Catalog::bundled()
                .unwrap()
                .resolve(Some(&identity("Testling", Some("Normal"))), None)
                .state,
            ResolutionState::Unavailable
        );
    }

    #[test]
    fn malformed_incompatible_and_inconsistent_snapshots_never_activate() {
        let mut active = Catalog::load(FIXTURE).unwrap();
        let original = active.clone();
        for bytes in [
            b"broken".as_slice(),
            b"{}",
            br#"{"schemaVersion":1,"schemaVersion":2}"#,
        ] {
            assert!(active.activate(bytes).is_err());
            assert_eq!(active, original);
        }
        let mut invalid = vec![];
        let mut v = document();
        v["schemaVersion"] = 2.into();
        invalid.push(v);
        let mut v = document();
        v["dataset"]["license"] = "".into();
        invalid.push(v);
        let mut v = document();
        v["species"][1]["id"] = "testling".into();
        invalid.push(v);
        let mut v = document();
        v["species"][1]["dexNumber"] = 60000.into();
        invalid.push(v);
        let mut v = document();
        v["forms"][1]["id"] = "testling-normal".into();
        invalid.push(v);
        let mut v = document();
        v["forms"][1]["speciesId"] = "missing".into();
        invalid.push(v);
        let mut v = document();
        v["forms"][0]["baseStats"]["attack"] = 0.into();
        invalid.push(v);
        let mut v = document();
        v["forms"][0]["types"] = serde_json::json!(["fire", "fire"]);
        invalid.push(v);
        let mut v = document();
        v["forms"][0]["types"] = serde_json::json!([]);
        invalid.push(v);
        let mut v = document();
        v["forms"][0]["types"] = serde_json::json!(["none"]);
        invalid.push(v);
        let mut v = document();
        v["forms"][0]["evolutions"][0]["targetFormId"] = "missing".into();
        invalid.push(v);
        let mut v = document();
        v["forms"][0]["evolutions"][0]["targetFormId"] = "testling-normal".into();
        invalid.push(v);
        let mut v = document();
        v["forms"][2]["evolutions"] =
            serde_json::json!([{"targetFormId":"testling-normal","requirements":null}]);
        invalid.push(v);
        let mut v = document();
        v["forms"][0].as_object_mut().unwrap().remove("baseStats");
        invalid.push(v);
        let mut v = document();
        v["species"][0]["stats"] = serde_json::json!({});
        invalid.push(v);
        for v in invalid {
            assert!(
                active.activate(&serde_json::to_vec(&v).unwrap()).is_err(),
                "{v}"
            );
            assert_eq!(active, original);
        }
    }

    #[test]
    fn versions_are_explicit_and_pins_survive_renames_without_rebinding() {
        let mut active = Catalog::load(FIXTURE).unwrap();
        active.activate(FIXTURE).unwrap();
        let original = active.clone();
        let mut next = document();
        next["species"][0]["name"] = "Renamed fictional species".into();
        assert!(active
            .activate(&serde_json::to_vec(&next).unwrap())
            .is_err());
        assert_eq!(active, original);
        next["dataset"]["version"] = "test-2".into();
        active
            .activate(&serde_json::to_vec(&next).unwrap())
            .unwrap();
        active
            .activate(&serde_json::to_vec(&next).unwrap())
            .unwrap();
        assert!(active.activate(FIXTURE).is_err());
        let mut reused = document();
        reused["species"][0]["name"] = "Reused version".into();
        assert!(active
            .activate(&serde_json::to_vec(&reused).unwrap())
            .is_err());
        let query = identity("Testling", Some("Normal"));
        let mut pin = Pin {
            dataset_id: "synthetic-test".into(),
            species_id: "testling".into(),
            form_id: "testling-normal".into(),
        };
        let resolved = active.resolve(Some(&query), Some(&pin));
        assert_eq!(resolved.state, ResolutionState::Resolved);
        assert_eq!(resolved.dataset.version, "test-2");
        pin.form_id = "removed".into();
        assert_eq!(
            active.resolve(Some(&query), Some(&pin)).state,
            ResolutionState::UnknownForm
        );
        pin.dataset_id = "another-provider".into();
        assert_eq!(
            active.resolve(Some(&query), Some(&pin)).state,
            ResolutionState::Unavailable
        );
    }

    #[test]
    fn identity_conflicts_are_rejected_even_after_removal_or_provider_switch() {
        let mut active = Catalog::load(FIXTURE).unwrap();
        let mut candidates = vec![];
        let mut changed_dex = document();
        changed_dex["species"][0]["dexNumber"] = 60002.into();
        candidates.push(changed_dex);
        let mut changed_id = document();
        changed_id["species"][0]["id"] = "replacement".into();
        for form in changed_id["forms"].as_array_mut().unwrap() {
            if form["speciesId"] == "testling" {
                form["speciesId"] = "replacement".into();
            }
        }
        candidates.push(changed_id);
        let mut moved_form = document();
        moved_form["forms"][1]["speciesId"] = "testwing".into();
        candidates.push(moved_form);

        for phase in 0..3 {
            for candidate in &candidates {
                let mut candidate = candidate.clone();
                candidate["dataset"]["version"] = format!("conflicting-{phase}").into();
                // Every candidate is structurally valid in isolation. Only
                // comparison with previously accepted identities rejects it.
                let bytes = serde_json::to_vec(&candidate).unwrap();
                Catalog::load(&bytes).unwrap();
                let before = active.clone();
                assert!(active.activate(&bytes).is_err());
                assert_eq!(active, before);
            }
            let mut next = document();
            if phase == 0 {
                next["dataset"]["version"] = "temporarily-empty".into();
                next["species"] = serde_json::json!([]);
                next["forms"] = serde_json::json!([]);
            } else {
                next["dataset"]["datasetId"] = "another-provider".into();
                next["dataset"]["version"] = format!("provider-{phase}").into();
                // Same normalized IDs are allowed in an independent namespace.
                next["species"][0]["dexNumber"] = 60002.into();
            }
            active
                .activate(&serde_json::to_vec(&next).unwrap())
                .unwrap();
        }
        let mut restored = document();
        restored["dataset"]["version"] = "restored-identities".into();
        active
            .activate(&serde_json::to_vec(&restored).unwrap())
            .unwrap();
        assert_eq!(
            active
                .resolve(Some(&identity("Essai", Some("Base"))), None)
                .state,
            ResolutionState::Resolved
        );
    }

    #[test]
    fn compatible_additions_renames_and_missing_metadata_keep_pinned_ids() {
        let mut active = Catalog::load(FIXTURE).unwrap();
        let query = identity("Essai", Some("Base"));
        let pin = Pin {
            dataset_id: "synthetic-test".into(),
            species_id: "testling".into(),
            form_id: "testling-normal".into(),
        };
        let mut next = document();
        next["dataset"]["version"] = "compatible-2".into();
        next["species"][0]["name"] = "Renamed fictional species".into();
        next["species"][0]["aliases"] = serde_json::json!([]);
        next["forms"][0]["name"] = "Renamed fictional form".into();
        next["forms"][0]["aliases"] = serde_json::json!([]);
        for field in ["types", "baseStats", "evolutions"] {
            next["forms"][0][field] = serde_json::Value::Null;
        }
        next["species"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "id": "testbud", "dexNumber": 60002, "name": "Testbud", "aliases": ["Essai"]
            }));
        next["forms"].as_array_mut().unwrap().push(serde_json::json!({
            "id": "testbud-normal", "speciesId": "testbud", "name": "Normal", "aliases": ["Base"],
            "types": null, "baseStats": null, "evolutions": []
        }));
        active
            .activate(&serde_json::to_vec(&next).unwrap())
            .unwrap();
        assert_eq!(
            active.resolve(Some(&query), None).species.unwrap().id,
            "testbud"
        );
        let resolved = active.resolve(Some(&query), Some(&pin));
        assert_eq!(resolved.state, ResolutionState::Resolved);
        assert_eq!(resolved.identity, Some(query));
        assert_eq!(resolved.species.unwrap().id, "testling");
        let form = resolved.form.unwrap();
        assert_eq!(form.id, "testling-normal");
        assert_eq!(form.types, None);
        assert_eq!(form.base_stats, None);
        assert_eq!(form.evolutions, None);
    }

    #[test]
    fn evaluation_consumes_available_metadata_without_claiming_rankings() {
        let catalog = Catalog::load(FIXTURE).unwrap();
        let resolved = catalog.resolve(Some(&identity("Testling", Some("Normal"))), None);
        let result = crate::evaluation::assess(None, &resolved).unwrap();
        assert!(result.reference.missing_data.is_empty());
        assert!(result.appraisal.total.is_none());
        assert!(result.appraisal.recommendations.is_empty());
        let missing = Catalog::bundled().unwrap().resolve(None, None);
        let result = crate::evaluation::assess(
            Some(crate::inventory::AppraisalIvs {
                attack: 15,
                defense: 15,
                hp: 15,
            }),
            &missing,
        )
        .unwrap();
        assert_eq!(result.reference.missing_data.len(), 4);
        assert_eq!(result.appraisal.total, Some(45));
        assert!(result.appraisal.recommendations.is_empty());
    }
}
