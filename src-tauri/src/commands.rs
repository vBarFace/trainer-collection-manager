//! Narrow renderer contracts. Decimal strings preserve the entire SQLite i64 range.
use crate::inventory::{
    self, AppraisalIvs, InventoryRepository, Pokemon, PokemonInput, Tag, TagId, TagScope,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: &'static str,
    pub message: &'static str,
}

impl CommandError {
    pub fn storage() -> Self {
        Self {
            code: "storage",
            message: "Local inventory is unavailable. Your existing data and any open review are retained; try again.",
        }
    }
}

impl From<inventory::Error> for CommandError {
    fn from(error: inventory::Error) -> Self {
        match error {
            inventory::Error::InvalidInput(message) => Self { code: "invalid_input", message },
            inventory::Error::Conflict => Self { code: "conflict", message: "The saved IVs changed while you were editing. Reload the inventory before correcting them again." },
            inventory::Error::NotFound("Tag") => Self { code: "missing_tag", message: "A selected tag no longer exists. Cancel and reload the inventory to review tags." },
            inventory::Error::NotFound(_) => Self { code: "missing_pokemon", message: "This Pokémon no longer exists. Reload the inventory." },
            _ => Self::storage(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ivs {
    attack: u8,
    defense: u8,
    hp: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportInput {
    display_name: String,
    cp: Option<String>,
    ivs: Option<Ivs>,
    tag_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CorrectIvsInput {
    // These nullable fields must be explicit: omission must never clear data.
    #[serde(deserialize_with = "Option::<Ivs>::deserialize")]
    expected_ivs: Option<Ivs>,
    #[serde(deserialize_with = "Option::<Ivs>::deserialize")]
    ivs: Option<Ivs>,
}

impl From<Ivs> for AppraisalIvs {
    fn from(ivs: Ivs) -> Self {
        Self {
            attack: ivs.attack,
            defense: ivs.defense,
            hp: ivs.hp,
        }
    }
}

fn parse_cp(value: Option<&str>) -> Result<Option<i64>, CommandError> {
    value
        .map(|value| {
            if value.is_empty() || value.len() > 19 || !value.bytes().all(|c| c.is_ascii_digit()) {
                return Err(CommandError {
                    code: "invalid_input",
                    message: "CP must be a whole number from 1 to 9223372036854775807, or unknown.",
                });
            }
            value
                .parse::<i64>()
                .ok()
                .filter(|cp| *cp > 0)
                .ok_or(CommandError {
                    code: "invalid_input",
                    message: "CP must be a whole number from 1 to 9223372036854775807, or unknown.",
                })
        })
        .transpose()
}

#[derive(Debug, Serialize)]
pub struct TagView {
    id: String,
    name: String,
    scope: &'static str,
}
impl From<Tag> for TagView {
    fn from(tag: Tag) -> Self {
        Self {
            id: tag.id.as_i64().to_string(),
            name: tag.name,
            scope: match tag.scope {
                TagScope::Standard => "standard",
                TagScope::Local => "local",
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordView {
    id: String,
    display_name: String,
    cp: Option<String>,
    ivs: Option<Ivs>,
    tags: Vec<TagView>,
    evaluation: crate::evaluation::Evaluation,
}
impl From<(Pokemon, Vec<Tag>)> for RecordView {
    fn from((pokemon, tags): (Pokemon, Vec<Tag>)) -> Self {
        Self {
            id: pokemon.id.as_str().into(),
            display_name: pokemon.display_name,
            cp: pokemon.cp.map(|cp| cp.to_string()),
            ivs: pokemon.ivs.map(|v| Ivs {
                attack: v.attack,
                defense: v.defense,
                hp: v.hp,
            }),
            tags: tags.into_iter().map(Into::into).collect(),
            evaluation: pokemon.evaluation,
        }
    }
}

#[derive(Serialize)]
pub struct Snapshot {
    records: Vec<RecordView>,
    tags: Vec<TagView>,
}

pub fn snapshot(repository: &InventoryRepository) -> Result<Snapshot, CommandError> {
    let (records, tags) = repository.snapshot()?;
    Ok(Snapshot {
        records: records.into_iter().map(Into::into).collect(),
        tags: tags.into_iter().map(Into::into).collect(),
    })
}

pub fn confirm(
    repository: &InventoryRepository,
    input: ImportInput,
) -> Result<RecordView, CommandError> {
    let pokemon = PokemonInput {
        display_name: input.display_name,
        cp: parse_cp(input.cp.as_deref())?,
        ivs: input.ivs.map(|v| AppraisalIvs {
            attack: v.attack,
            defense: v.defense,
            hp: v.hp,
        }),
    };
    let tags = input
        .tag_ids
        .iter()
        .map(|id| TagId::parse(id))
        .collect::<inventory::Result<Vec<_>>>()?;
    Ok(repository.create_with_tags(&pokemon, &tags)?.into())
}

pub fn evaluate(repository: &InventoryRepository, id: &str) -> Result<RecordView, CommandError> {
    let id = inventory::PokemonId::parse(id)?;
    Ok(repository.evaluate(&id)?.into())
}

pub fn correct_ivs(
    repository: &InventoryRepository,
    id: &str,
    input: CorrectIvsInput,
) -> Result<RecordView, CommandError> {
    let id = inventory::PokemonId::parse(id)?;
    Ok(repository
        .correct_ivs(
            &id,
            input.expected_ivs.map(Into::into),
            input.ivs.map(Into::into),
        )?
        .into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cp_preserves_sqlite_integer_precision() {
        for cp in [
            "1",
            "9007199254740991",
            "9007199254740992",
            "9007199254740993",
            "9223372036854775807",
        ] {
            let parsed = parse_cp(Some(cp)).unwrap().unwrap();
            assert_eq!(parsed.to_string(), cp);
        }
        assert_eq!(parse_cp(None).unwrap(), None);
        for cp in [
            "",
            "0",
            "-1",
            "1.5",
            "1e2",
            " 1",
            "+1",
            "9223372036854775808",
            "9999999999999999999999",
        ] {
            assert_eq!(parse_cp(Some(cp)).unwrap_err().code, "invalid_input");
        }
    }

    #[test]
    fn iv_correction_contract_requires_explicit_groups_and_rejects_unrelated_fields() {
        assert!(serde_json::from_value::<CorrectIvsInput>(
            serde_json::json!({"expectedIvs":null,"ivs":null})
        )
        .is_ok());
        for value in [
            serde_json::json!({}),
            serde_json::json!({"expectedIvs":null}),
            serde_json::json!({"ivs":null}),
            serde_json::json!({"expectedIvs":null,"ivs":{"attack":15}}),
            serde_json::json!({"expectedIvs":null,"ivs":{"attack":1.5,"defense":0,"hp":0}}),
            serde_json::json!({"expectedIvs":null,"ivs":{"attack":-1,"defense":0,"hp":0}}),
        ] {
            assert!(serde_json::from_value::<CorrectIvsInput>(value).is_err());
        }
        for field in [
            "displayName",
            "cp",
            "tagIds",
            "path",
            "extraction",
            "evaluation",
        ] {
            let mut value = serde_json::json!({"expectedIvs":null,"ivs":null});
            value[field] = serde_json::json!("unrelated data");
            assert!(serde_json::from_value::<CorrectIvsInput>(value).is_err());
        }
    }

    #[test]
    fn iv_correction_command_returns_current_record_and_safe_errors() {
        let path = std::env::temp_dir().join(format!(
            "tcm-correct-command-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let repo = InventoryRepository::open(&path).unwrap();
        let original = confirm(&repo, serde_json::from_value(serde_json::json!({"displayName":"Saved","cp":"9223372036854775807","ivs":null,"tagIds":["9"]})).unwrap()).unwrap();
        evaluate(&repo, &original.id).unwrap();
        let request =
            serde_json::json!({"expectedIvs":null,"ivs":{"attack":0,"defense":14,"hp":15}});
        let saved = correct_ivs(
            &repo,
            &original.id,
            serde_json::from_value(request.clone()).unwrap(),
        )
        .unwrap();
        assert_eq!(saved.id, original.id);
        assert_eq!(saved.cp, original.cp);
        assert_eq!(saved.ivs.unwrap().attack, 0);
        assert_eq!(
            serde_json::to_value(saved.tags).unwrap(),
            serde_json::to_value(original.tags).unwrap()
        );
        assert_eq!(
            saved.evaluation.state,
            crate::evaluation::EvaluationState::Stale
        );
        assert_eq!(
            correct_ivs(
                &repo,
                &original.id,
                serde_json::from_value(request.clone()).unwrap()
            )
            .unwrap_err()
            .code,
            "conflict"
        );
        let invalid =
            serde_json::json!({"expectedIvs":null,"ivs":{"attack":16,"defense":0,"hp":0}});
        assert_eq!(
            correct_ivs(
                &repo,
                &original.id,
                serde_json::from_value(invalid).unwrap()
            )
            .unwrap_err()
            .code,
            "invalid_input"
        );
        assert_eq!(
            correct_ivs(
                &repo,
                "malformed",
                serde_json::from_value(request.clone()).unwrap()
            )
            .unwrap_err()
            .code,
            "invalid_input"
        );
        repo.delete(&inventory::PokemonId::parse(&original.id).unwrap())
            .unwrap();
        assert_eq!(
            correct_ivs(
                &repo,
                &original.id,
                serde_json::from_value(request).unwrap()
            )
            .unwrap_err()
            .code,
            "missing_pokemon"
        );
        drop(repo);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn evaluation_contract_uses_confirmed_data_only_and_safe_errors() {
        let path = std::env::temp_dir().join(format!(
            "tcm-evaluate-command-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let repo = InventoryRepository::open(&path).unwrap();
        let input = serde_json::from_value(serde_json::json!({
            "displayName": "Confirmed", "cp": null,
            "ivs": {"attack":15,"defense":15,"hp":15}, "tagIds":["9"]
        }))
        .unwrap();
        let saved = confirm(&repo, input).unwrap();
        assert_eq!(
            saved.evaluation.state,
            crate::evaluation::EvaluationState::Pending
        );
        let evaluated = evaluate(&repo, &saved.id).unwrap();
        let json = serde_json::to_value(&evaluated).unwrap();
        assert_eq!(json["evaluation"]["scope"], "appraisal");
        assert_eq!(json["evaluation"]["state"], "evaluated");
        assert_eq!(json["evaluation"]["rulesetVersion"], "appraisal-1");
        assert_eq!(json["evaluation"]["result"]["total"], 45);
        assert_eq!(json["evaluation"]["result"]["appraisalPercentage"], 100.0);
        assert_eq!(
            json["evaluation"]["result"]["findings"][0]["code"],
            "perfect_appraisal"
        );
        assert_eq!(
            json["evaluation"]["result"]["recommendations"],
            serde_json::json!([])
        );
        assert_eq!(json["tags"], serde_json::to_value(&saved.tags).unwrap());
        assert_eq!(
            evaluate(&repo, "malformed").unwrap_err().code,
            "invalid_input"
        );
        let id = inventory::PokemonId::parse(&saved.id).unwrap();
        repo.delete(&id).unwrap();
        assert_eq!(
            evaluate(&repo, &saved.id).unwrap_err().code,
            "missing_pokemon"
        );
        drop(repo);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn typed_input_rejects_partial_fractional_and_unexpected_fields() {
        for value in [
            serde_json::json!({"attack": 15}),
            serde_json::json!({"attack": 15, "defense": 15}),
            serde_json::json!({"attack": 1.5, "defense": 15, "hp": 15}),
            serde_json::json!({"attack": -1, "defense": 15, "hp": 15}),
        ] {
            assert!(serde_json::from_value::<Ivs>(value).is_err());
        }
        let request =
            serde_json::json!({"displayName":"Reviewed", "cp":null, "ivs":null, "tagIds":[]});
        assert!(serde_json::from_value::<ImportInput>(request.clone()).is_ok());
        let mut request_with_path = request.clone();
        request_with_path["path"] = serde_json::json!("private.png");
        assert!(serde_json::from_value::<ImportInput>(request_with_path).is_err());
        let mut numeric_cp = request;
        numeric_cp["cp"] = serde_json::json!(9007199254740992_i64);
        assert!(serde_json::from_value::<ImportInput>(numeric_cp).is_err());
    }

    #[test]
    fn confirmed_contract_survives_reopen_with_exact_cp_and_tags() {
        let path = std::env::temp_dir().join(format!(
            "tcm-confirm-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let repo = InventoryRepository::open(&path).unwrap();
        let special = repo
            .tags()
            .unwrap()
            .into_iter()
            .find(|tag| tag.name == "SAVE/SPECIAL")
            .unwrap();
        let input = serde_json::from_value(serde_json::json!({
            "displayName": "Manual review", "cp": "9223372036854775807",
            "ivs": {"attack":0,"defense":14,"hp":15}, "tagIds":[special.id.as_i64().to_string()]
        }))
        .unwrap();
        let saved = confirm(&repo, input).unwrap();
        assert_eq!(saved.cp.as_deref(), Some("9223372036854775807"));
        assert_eq!(saved.tags[0].name, "SAVE/SPECIAL");
        drop(repo);
        let repo = InventoryRepository::open(&path).unwrap();
        let loaded = snapshot(&repo).unwrap();
        assert_eq!(loaded.records.len(), 1);
        assert_eq!(loaded.records[0].id, saved.id);
        assert_eq!(loaded.records[0].cp, saved.cp);
        assert_eq!(loaded.records[0].ivs.as_ref().unwrap().attack, 0);
        assert_eq!(loaded.records[0].tags[0].name, "SAVE/SPECIAL");
        drop(repo);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn storage_errors_never_expose_details() {
        let error = CommandError::from(inventory::Error::Storage(
            rusqlite::Error::InvalidParameterName("private path or SQL".into()),
        ));
        assert_eq!(error.code, "storage");
        assert!(!error.message.contains("private"));
    }

    #[test]
    fn suggestions_are_not_persistence_and_only_confirmed_corrections_survive_reopen() {
        let path = std::env::temp_dir().join(format!(
            "tcm-extraction-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let repo = InventoryRepository::open(&path).unwrap();
        let suggestions =
            crate::extraction::extract(&crate::extraction::tests::fixture(1, [0, 14, 15]));
        assert_eq!(suggestions.attack.value, Some(0));
        // Extraction has no repository argument or write side effects.
        assert!(snapshot(&repo).unwrap().records.is_empty());
        drop(repo);
        let repo = InventoryRepository::open(&path).unwrap();
        assert!(snapshot(&repo).unwrap().records.is_empty());
        let request = serde_json::json!({
            "displayName": "Corrected review", "cp": null,
            "ivs": {"attack":1,"defense":14,"hp":15}, "tagIds":[]
        });
        // Raw image/suggestion payloads cannot be smuggled into persistence.
        for field in ["png", "extraction", "filename", "path"] {
            let mut invalid = request.clone();
            invalid[field] = serde_json::json!("not persisted");
            assert!(serde_json::from_value::<ImportInput>(invalid).is_err());
        }
        confirm(&repo, serde_json::from_value(request).unwrap()).unwrap();
        drop(repo);
        let repo = InventoryRepository::open(&path).unwrap();
        let snapshot = snapshot(&repo).unwrap();
        assert_eq!(snapshot.records.len(), 1);
        assert_eq!(snapshot.records[0].ivs.as_ref().unwrap().attack, 1);
        assert!(snapshot.records[0].cp.is_none());
        assert!(snapshot.records[0].tags.is_empty());
        drop(repo);
        std::fs::remove_file(path).unwrap();
    }
}
