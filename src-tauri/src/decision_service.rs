//! Refresh derived individual advice after confirmed facts have been saved.
//! This command neither repeats a game confirmation nor applies suggested tags.
use crate::inventory::{Error, InventoryRepository, PokemonId, Result};
use crate::strategy::StrategyReport;
use crate::strategy_service::RoadmapView;
use serde::Serialize;

const MAX_REVIEW_ATTEMPTS: usize = 3;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndividualReview {
    pub context: String,
    pub report: StrategyReport,
    pub roadmap: RoadmapView,
    pub notice: Option<String>,
}

fn review_from_roadmap(id: &PokemonId, roadmap: RoadmapView) -> Result<IndividualReview> {
    if roadmap.stale || roadmap.context != roadmap.current_context {
        return Err(Error::ObservationConflict);
    }
    let report = roadmap.report.as_ref().ok_or(Error::InvalidInput(
        "The refreshed individual roadmap has no assessment",
    ))?;
    if report.pokemon_id != id.as_str() {
        return Err(Error::InvalidInput(
            "Individual assessment identity mismatch",
        ));
    }
    let notice = match &report.preparation {
        Some(plan)
            if plan.resources.values.selected_path_id.is_some()
                && plan.selected_path_id.is_none() =>
        {
            Some("Your previous route choice no longer matches the current evidence. Review the updated proposed plan before an in-game action.".into())
        }
        Some(plan) if plan.paths.is_empty() => Some(
            "More reference or observed evidence is needed for a preparation route. Your confirmed individual data is retained.".into(),
        ),
        _ => None,
    };
    Ok(IndividualReview {
        context: roadmap.context.clone(),
        report: report.clone(),
        roadmap,
        notice,
    })
}

impl InventoryRepository {
    /// Use after a successful fact confirmation, or when opening an individual.
    /// All action/observation transactions have already committed. An assessment
    /// failure must not turn a real saved action into a failed confirmation.
    ///
    /// The roadmap writer compares the captured context inside an immediate
    /// transaction. Read back the saved view afterwards to detect another
    /// instance changing inputs or generating a different roadmap in between.
    /// The result is consistent at that read snapshot; subsequent edits require
    /// another refresh. Automatic route proposals do not become manual choices.
    pub fn individual_decision(&self, id: &PokemonId) -> Result<IndividualReview> {
        for _ in 0..MAX_REVIEW_ATTEMPTS {
            let snapshot = self.individual_strategy_snapshot(id)?;
            if !snapshot
                .reports
                .iter()
                .any(|report| report.pokemon_id == id.as_str())
            {
                return Err(Error::NotFound("Pokemon"));
            }
            let generated = match self.generate_roadmap(id, &snapshot.context) {
                Ok(view) => view,
                Err(Error::ObservationConflict) => continue,
                Err(error) => return Err(error),
            };
            let current = self.roadmap(id)?;
            if current.context != generated.context {
                continue;
            }
            match review_from_roadmap(id, current) {
                Err(Error::ObservationConflict) => continue,
                result => return result,
            }
        }
        Err(Error::ObservationConflict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::individual_details::IndividualDetails;
    use crate::inventory::{
        AppraisalIvs, EvolutionConfirmation, PokemonInput, PowerUpConfirmation,
    };
    use crate::reference::Identity;

    struct Database(std::path::PathBuf);
    impl Database {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let name = format!(
                "individual-review-{}-{}-{}.sqlite3",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            );
            Self(std::env::temp_dir().join(name))
        }
    }
    impl Drop for Database {
        fn drop(&mut self) {
            for suffix in ["", "-wal", "-shm"] {
                let mut path = self.0.as_os_str().to_os_string();
                path.push(suffix);
                let _ = std::fs::remove_file(std::path::PathBuf::from(path));
            }
        }
    }
    fn setup() -> (Database, InventoryRepository, PokemonId) {
        let db = Database::new();
        let mut repo = InventoryRepository::open(&db.0).unwrap();
        let mut package = crate::gym::fictional_package();
        package.catalog["dataset"]["version"] = "individual-service-fixture-1".into();
        package.catalog["forms"][0]["evolutions"] = serde_json::json!([
            {"targetFormId":"dex-61002-normal","requirements":["Invented source relationship; confirm prerequisites in game"]}
        ]);
        repo.install_reference(package).unwrap();
        let individual = repo
            .create(&PokemonInput {
                display_name: "Sampleton".into(),
                cp: None,
                ivs: Some(AppraisalIvs {
                    attack: 15,
                    defense: 15,
                    hp: 15,
                }),
            })
            .unwrap();
        repo.correct_identity(
            &individual.id,
            None,
            Some(Identity {
                species: "Sampleton".into(),
                form: Some("Normal".into()),
            }),
        )
        .unwrap();
        let details = repo.individual_details(&individual.id).unwrap();
        repo.save_individual_details(
            &individual.id,
            details.observation_revision.parse().unwrap(),
            &details.evidence_token,
            IndividualDetails {
                level: Some(20.0),
                level_evidence: Some("Explicit fictional manual level".into()),
                fast_move: Some("Sample Tap".into()),
                charged_move: Some("Sample Slam".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let custom = repo.create_custom_tag("My training specimen").unwrap();
        repo.assign_tag(&individual.id, custom.id).unwrap();
        (db, repo, individual.id)
    }

    #[test]
    fn automatic_review_is_idempotent_and_preserves_manual_progress_after_reopen() {
        let (db, repo, id) = setup();
        let first = repo.individual_decision(&id).unwrap();
        assert_eq!(first.report.pokemon_id, id.as_str());
        assert!(!first.roadmap.stale);
        let step = first.report.steps.first().unwrap();
        let progressed = repo
            .update_roadmap_progress(
                &id,
                &first.context,
                first.roadmap.revision.parse().unwrap(),
                &step.id,
                "postponed",
            )
            .unwrap();
        let facts = repo.get(&id).unwrap();
        let tags = repo.tags_for(&id).unwrap();
        let resources = repo.preparation_resources(&id).unwrap();
        assert!(
            resources.values.selected_path_id.is_none(),
            "An automatic proposal must not be persisted as a manual route choice"
        );
        let expected = serde_json::to_vec(&repo.individual_decision(&id).unwrap()).unwrap();
        for _ in 0..4 {
            let reviewed = repo.individual_decision(&id).unwrap();
            assert_eq!(serde_json::to_vec(&reviewed).unwrap(), expected);
            assert_eq!(reviewed.roadmap.revision, progressed.revision);
            assert_eq!(
                reviewed.roadmap.progress.get(&step.id).map(String::as_str),
                Some("postponed")
            );
        }
        assert_eq!(repo.get(&id).unwrap(), facts);
        assert_eq!(repo.tags_for(&id).unwrap(), tags);
        assert_eq!(repo.preparation_resources(&id).unwrap(), resources);
        drop(repo);
        let reopened = InventoryRepository::open(&db.0).unwrap();
        assert_eq!(
            serde_json::to_vec(&reopened.individual_decision(&id).unwrap()).unwrap(),
            expected
        );
    }

    #[test]
    fn real_power_up_and_evolution_refresh_the_same_individual_without_inventing_facts() {
        let (_db, repo, id) = setup();
        let initial = repo.individual_decision(&id).unwrap();
        let tags = repo.tags_for(&id).unwrap();
        let resources = repo.preparation_resources(&id).unwrap();
        let pokemon = repo.get(&id).unwrap();
        repo.confirm_power_up(
            &id,
            PowerUpConfirmation {
                request_id: "20000000-0000-4000-8000-000000000021",
                expected_revision: pokemon.strategic_observation_revision,
                cp: Some(80),
                performed_in_game_confirmed: true,
            },
        )
        .unwrap();
        let after_power = repo.individual_decision(&id).unwrap();
        assert_ne!(after_power.context, initial.context);
        assert_eq!(after_power.report.pokemon_id, id.as_str());
        assert!(!after_power.roadmap.stale);
        assert_eq!(
            after_power.roadmap.previous.as_ref().unwrap().context,
            initial.context
        );
        assert_eq!(
            after_power
                .report
                .preparation
                .as_ref()
                .unwrap()
                .current_level,
            None
        );
        assert!(repo.individual_details(&id).unwrap().needs_reconfirmation);
        assert_eq!(repo.get(&id).unwrap().cp, Some(80));
        let options = repo.evolution_options(&id).unwrap();
        repo.confirm_evolution(
            &id,
            EvolutionConfirmation {
                request_id: "20000000-0000-4000-8000-000000000022",
                expected_revision: options
                    .expected_strategic_observation_revision
                    .parse()
                    .unwrap(),
                dataset_id: &options.dataset_id,
                dataset_version: &options.dataset_version,
                target_form_id: "dex-61002-normal",
            },
        )
        .unwrap();
        let after_evolution = repo.individual_decision(&id).unwrap();
        assert_ne!(after_evolution.context, after_power.context);
        assert_eq!(after_evolution.report.pokemon_id, id.as_str());
        assert_eq!(
            after_evolution.roadmap.previous.as_ref().unwrap().context,
            after_power.context
        );
        assert_eq!(
            after_evolution
                .report
                .preparation
                .as_ref()
                .unwrap()
                .current_level,
            None
        );
        let evolved = repo.get(&id).unwrap();
        assert_eq!(evolved.reference.species.unwrap().name, "Sampleprime");
        assert_eq!(evolved.cp, None);
        assert_eq!(evolved.ivs, pokemon.ivs);
        assert_eq!(repo.history(&id).unwrap().len(), 2);
        assert_eq!(repo.tags_for(&id).unwrap(), tags);
        assert_eq!(repo.preparation_resources(&id).unwrap(), resources);
        assert!(!after_evolution.roadmap.stale);
    }

    #[test]
    fn refreshing_preserves_a_valid_manual_route_choice() {
        let (_db, repo, id) = setup();
        let initial = repo.individual_decision(&id).unwrap();
        let route = initial
            .report
            .preparation
            .as_ref()
            .unwrap()
            .paths
            .iter()
            .find(|path| path.actions.iter().any(|action| action.kind == "use_now"))
            .unwrap()
            .id
            .clone();
        let choice = repo
            .select_preparation_path(&id, &initial.context, &route)
            .unwrap();
        let reviewed = repo.individual_decision(&id).unwrap();
        assert_ne!(reviewed.context, initial.context);
        assert_eq!(
            reviewed
                .report
                .preparation
                .as_ref()
                .unwrap()
                .selected_path_id
                .as_deref(),
            Some(route.as_str())
        );
        assert_eq!(repo.preparation_resources(&id).unwrap(), choice);
        let repeated = repo.individual_decision(&id).unwrap();
        assert_eq!(
            serde_json::to_vec(&reviewed).unwrap(),
            serde_json::to_vec(&repeated).unwrap()
        );
    }

    #[test]
    fn stale_readback_is_rejected_instead_of_returning_a_freshness_claim() {
        let (_db, repo, id) = setup();
        repo.individual_decision(&id).unwrap();
        let mut view = repo.roadmap(&id).unwrap();
        view.current_context = "different snapshot".into();
        assert!(matches!(
            review_from_roadmap(&id, view),
            Err(Error::ObservationConflict)
        ));
        let mut view = repo.roadmap(&id).unwrap();
        view.stale = true;
        assert!(matches!(
            review_from_roadmap(&id, view),
            Err(Error::ObservationConflict)
        ));
    }

    #[test]
    fn incomplete_import_receives_a_limited_review_without_fabricating_identity() {
        let db = Database::new();
        let repo = InventoryRepository::open(&db.0).unwrap();
        let created = repo
            .create(&PokemonInput {
                display_name: "An unconfirmed nickname".into(),
                cp: Some(321),
                ivs: None,
            })
            .unwrap();
        let reviewed = repo.individual_decision(&created.id).unwrap();
        assert_eq!(reviewed.report.pokemon_id, created.id.as_str());
        assert!(!reviewed.roadmap.stale);
        assert!(reviewed.notice.is_some());
        assert_eq!(repo.get(&created.id).unwrap(), created);
        assert!(repo.tags_for(&created.id).unwrap().is_empty());
        assert!(repo.history(&created.id).unwrap().is_empty());
    }

    #[test]
    fn assessment_failure_does_not_undo_an_already_confirmed_game_action() {
        let (_db, repo, id) = setup();
        repo.individual_decision(&id).unwrap();
        let observed = repo.get(&id).unwrap();
        repo.confirm_power_up(
            &id,
            PowerUpConfirmation {
                request_id: "20000000-0000-4000-8000-000000000023",
                expected_revision: observed.strategic_observation_revision,
                cp: Some(91),
                performed_in_game_confirmed: true,
            },
        )
        .unwrap();
        repo.connection
            .execute(
                "UPDATE pokemon_roadmaps SET report_json='{}' WHERE pokemon_id=?1",
                [id.as_str()],
            )
            .unwrap();
        assert!(repo.individual_decision(&id).is_err());
        assert_eq!(repo.get(&id).unwrap().cp, Some(91));
        assert_eq!(repo.history(&id).unwrap().len(), 1);
    }
}
