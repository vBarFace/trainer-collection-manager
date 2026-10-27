use super::*;
use serde_json::{json, Value};

const INDIVIDUAL: &str = "a1111111-1111-4111-8111-111111111111";
const OTHER: &str = "b2222222-2222-4222-8222-222222222222";
const FIXTURES: &str = include_str!("../../../tests/fixtures/utility/acceptance.json");

fn fixture(name: &str) -> Value {
    let fixtures: Value = serde_json::from_str(FIXTURES).unwrap();
    fixtures["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == name)
        .unwrap()["assessment"]
        .clone()
}

fn load(value: &Value) -> Result<Assessment> {
    Assessment::load(&serde_json::to_vec(value).unwrap(), INDIVIDUAL)
}

fn rejected(name: &str, change: impl FnOnce(&mut Value), expected: &str) {
    let mut value = fixture(name);
    change(&mut value);
    let error = load(&value)
        .expect_err("invalid contract was accepted")
        .to_string();
    assert!(
        error.contains(expected),
        "expected {expected:?}, got {error}"
    );
}

#[test]
fn shared_fictional_acceptance_cases_round_trip_through_native_contract() {
    let fixtures: Value = serde_json::from_str(FIXTURES).unwrap();
    assert!(fixtures["notice"].as_str().unwrap().contains("fictional"));
    assert_eq!(fixtures["cases"].as_array().unwrap().len(), 9);
    for case in fixtures["cases"].as_array().unwrap() {
        let assessment =
            load(&case["assessment"]).unwrap_or_else(|error| panic!("{}: {error}", case["id"]));
        let serialized = serde_json::to_vec(&assessment).unwrap();
        assert_eq!(
            Assessment::load(&serialized, INDIVIDUAL).unwrap(),
            assessment
        );
    }
}

#[test]
fn potential_role_does_not_assert_current_usefulness_or_block_first_evolution() {
    let a = load(&fixture("evolution-first")).unwrap();
    assert_eq!(a.current_roles, Some(vec![]));
    assert_eq!(
        a.potential_roles.as_ref().unwrap()[0].readiness,
        Readiness::PreparationRequired
    );
    assert_eq!(a.suggested_action.unwrap().kind, ActionKind::Evolve);
    assert!(!a.paths[0].steps[1].prerequisites.is_empty());
    rejected(
        "evolution-first",
        |a| a["currentRoles"] = a["potentialRoles"].clone(),
        "current and potential readiness",
    );
    rejected(
        "evolution-first",
        |a| a["potentialRoles"] = Value::Null,
        "supported potential role",
    );
}

#[test]
fn use_now_and_optional_investment_remain_separate_choices() {
    let a = load(&fixture("usable-now-optional-investment")).unwrap();
    assert_eq!(a.paths.len(), 2);
    assert_eq!(a.selected_path_id, "use-now");
    assert!(a.suggested_action.is_none());
    assert_eq!(
        a.suggested_summary_tag,
        Some(SummaryTag::Role {
            tag: Role::Attacker
        })
    );
    assert!(a.paths[0].estimated_costs.as_ref().unwrap().is_empty());
    assert!(!a.paths[1].steps[0].required);
    rejected(
        "usable-now-optional-investment",
        |a| a["currentRoles"] = json!([]),
        "supported current role",
    );
}

#[test]
fn costs_are_per_alternative_not_summed_and_zero_budget_still_allows_use_now() {
    let mut value = fixture("usable-now-optional-investment");
    value["constraints"]["budget"][0]["amount"] = json!("9");
    let mut other = value["paths"][1].clone();
    other["id"] = json!("another-investment");
    other["steps"][0]["id"] = json!("other-power");
    other["steps"][1]["id"] = json!("other-use");
    other["steps"][1]["prerequisites"] = json!(["other-power"]);
    value["paths"].as_array_mut().unwrap().push(other);
    // Each investment individually costs 9 against 9; their sum is irrelevant.
    load(&value).unwrap();
    let zero = load(&fixture("zero-budget")).unwrap();
    assert_eq!(
        zero.constraints.budget.unwrap()[0].amount.as_deref(),
        Some("0")
    );
    assert_eq!(zero.paths[1].affordability, Affordability::OverBudget);
    assert_eq!(zero.selected_path_id, "use-now");
}

#[test]
fn missing_evidence_preserves_null_versus_known_empty() {
    let missing = load(&fixture("missing-evidence")).unwrap();
    assert!(missing.current_roles.is_none());
    assert!(missing.potential_roles.is_none());
    assert!(missing.constraints.budget.is_none());
    assert!(missing.paths[0].estimated_costs.is_none());
    assert!(missing.paths[0].steps[0].conditions.is_none());
    assert!(missing.provenance.evaluated.datasets.is_none());
    assert!(!missing.missing_evidence.is_empty());
    let unknown = load(&fixture("unknown-budget")).unwrap();
    assert!(unknown.constraints.budget.unwrap()[0].amount.is_none());
    assert!(unknown.paths[0].estimated_costs.as_ref().unwrap()[0]
        .amount
        .is_none());
}

#[test]
fn required_move_change_has_an_action_without_an_invented_default_tag() {
    let a = load(&fixture("required-move-change")).unwrap();
    assert_eq!(a.suggested_action.unwrap().kind, ActionKind::ChangeMoves);
    assert!(a.suggested_summary_tag.is_none());
    // Exact values greater than JS's safe integer remain strings across serde.
    assert_eq!(
        a.paths[0].estimated_costs.as_ref().unwrap()[0]
            .amount
            .as_deref(),
        Some("9007199254740993")
    );
    assert_eq!(
        a.constraints.budget.unwrap()[0].amount.as_deref(),
        Some("9223372036854775807")
    );
}

#[test]
fn stale_reports_preserve_history_without_verifying_actions_or_spending() {
    let a = load(&fixture("stale-context-report")).unwrap();
    assert_eq!(a.validity, Validity::Stale);
    assert!(a.suggested_action.is_none() && a.suggested_summary_tag.is_none());
    assert_ne!(a.provenance.evaluated, a.provenance.current);
    let report = serde_json::to_value(&a.action_reports[0]).unwrap();
    assert!(report.get("completed").is_none());
    assert!(report.get("confirmedObservations").is_none());
    assert!(report.get("spent").is_none());
    rejected(
        "stale-context-report",
        |a| a["actionReports"][0]["spent"] = json!([]),
        "unknown field",
    );
    // A report alone cannot move a current assessment on to its next step.
    rejected(
        "evolution-first",
        |a| {
            a["actionReports"] = json!([{
                "pathId":"prepare", "stepId":"prepare-evolve",
                "observationRevision":"0", "constraintRevision":"0"
            }]);
            a["suggestedAction"]["kind"] = json!("change_moves");
            a["suggestedAction"]["stepId"] = json!("prepare-moves");
            a["suggestedSummaryTag"] = Value::Null;
        },
        "first step",
    );
}

#[test]
fn rejects_wrong_individual_and_goal_associations() {
    rejected(
        "evolution-first",
        |a| a["pokemonId"] = json!("not-a-uuid"),
        "canonical lowercase UUID",
    );
    rejected(
        "evolution-first",
        |a| a["pokemonId"] = json!(OTHER),
        "individual association mismatch",
    );
    rejected(
        "evolution-first",
        |a| a["paths"][0]["pokemonId"] = json!(OTHER),
        "path individual association",
    );
    rejected(
        "evolution-first",
        |a| a["paths"][0]["goalId"] = json!("different"),
        "goal association",
    );
    rejected(
        "evolution-first",
        |a| a["potentialRoles"][0]["contextId"] = json!("different"),
        "context differs",
    );
    let bytes = serde_json::to_vec(&fixture("evolution-first")).unwrap();
    assert!(Assessment::load(&bytes, "invalid")
        .unwrap_err()
        .to_string()
        .contains("expectedPokemonId"));
}

#[test]
fn rejects_duplicate_or_missing_paths_and_steps() {
    rejected(
        "evolution-first",
        |a| {
            let duplicate = a["paths"][0].clone();
            a["paths"].as_array_mut().unwrap().push(duplicate);
        },
        "duplicate alternative",
    );
    rejected(
        "evolution-first",
        |a| a["paths"][0]["steps"][1]["id"] = json!("prepare-evolve"),
        "duplicate plan-local step",
    );
    rejected(
        "usable-now-optional-investment",
        |a| a["paths"][1]["steps"][0]["id"] = json!("now-use"),
        "duplicate plan-local step",
    );
    rejected(
        "evolution-first",
        |a| a["selectedPathId"] = json!("absent"),
        "selected alternative does not exist",
    );
    rejected(
        "evolution-first",
        |a| a["paths"] = json!([]),
        "at least one selected path",
    );
}

#[test]
fn rejects_missing_forward_self_cross_path_and_cyclic_prerequisites() {
    for prerequisite in ["absent", "prepare-moves", "prepare-evolve"] {
        rejected(
            "evolution-first",
            |a| a["paths"][0]["steps"][0]["prerequisites"] = json!([prerequisite]),
            "earlier step",
        );
    }
    rejected(
        "evolution-first",
        |a| {
            a["paths"][0]["steps"][0]["prerequisites"] = json!(["prepare-moves"]);
            a["paths"][0]["steps"][1]["prerequisites"] = json!(["prepare-evolve"]);
        },
        "cyclic",
    );
    rejected(
        "usable-now-optional-investment",
        |a| a["paths"][1]["steps"][0]["prerequisites"] = json!(["now-use"]),
        "earlier step",
    );
    rejected(
        "evolution-first",
        |a| {
            a["paths"][0]["steps"][1]["prerequisites"] = json!(["prepare-evolve", "prepare-evolve"])
        },
        "duplicate prerequisite",
    );
}

#[test]
fn rejects_kind_mismatches_and_unknown_vocabulary() {
    rejected(
        "evolution-first",
        |a| a["suggestedSummaryTag"] = json!({"kind":"role","tag":"EVOLVE"}),
        "unknown variant",
    );
    rejected(
        "evolution-first",
        |a| a["suggestedSummaryTag"] = json!({"kind":"action","tag":"ATTACKER"}),
        "unknown variant",
    );
    rejected(
        "evolution-first",
        |a| a["suggestedSummaryTag"]["tag"] = json!("EVOLUTION"),
        "unknown variant",
    );
    rejected(
        "evolution-first",
        |a| a["suggestedAction"]["kind"] = json!("power_up"),
        "first step",
    );
    rejected(
        "evolution-first",
        |a| a["paths"][0]["steps"][3]["action"]["role"] = json!("GYM DEF"),
        "use role differs",
    );
    rejected(
        "evolution-first",
        |a| a["suggestedSummaryTag"]["tag"] = json!("POWER UP"),
        "summary tag must match",
    );
    rejected(
        "required-move-change",
        |a| a["suggestedSummaryTag"] = json!({"kind":"action","tag":"POWER UP"}),
        "move changes have no default tag",
    );
}

#[test]
fn rejects_unknown_cost_affordability_and_contradictory_budgets() {
    for costs in [
        Value::Null,
        json!([{"resource":"paper_tokens","amount":null}]),
    ] {
        rejected(
            "power-up-first",
            |a| a["paths"][0]["estimatedCosts"] = costs,
            "unknown",
        );
    }
    rejected(
        "power-up-first",
        |a| a["constraints"]["budget"] = Value::Null,
        "known budget",
    );
    rejected(
        "power-up-first",
        |a| a["constraints"]["budget"][0]["amount"] = json!("0"),
        "contradicts",
    );
    rejected(
        "power-up-first",
        |a| a["paths"][0]["affordability"] = json!("unknown"),
        "known affordability",
    );
    rejected(
        "power-up-first",
        |a| a["paths"][0]["affordability"] = json!("over_budget"),
        "contradicts",
    );
    rejected(
        "zero-budget",
        |a| a["paths"][0]["estimatedCosts"] = json!([{"resource":"paper_tokens","amount":"1"}]),
        "known zero investment",
    );
}

#[test]
fn rejects_current_investment_with_missing_data_conditions_or_protection_conflict() {
    for allowed in [json!(false), Value::Null] {
        rejected(
            "evolution-first",
            |a| a["constraints"]["evolutionAllowed"] = allowed,
            "evolution permission",
        );
    }
    rejected(
        "power-up-first",
        |a| a["missingEvidence"] = json!(["mechanics"]),
        "missing required evidence",
    );
    for conditions in [
        Value::Null,
        json!([{"description":"Fictional unmet condition","state":"unmet"}]),
        json!([{"description":"Fictional unknown condition","state":"unknown"}]),
    ] {
        rejected(
            "power-up-first",
            |a| a["paths"][0]["steps"][0]["conditions"] = conditions,
            "unmet or unknown conditions",
        );
    }
    rejected(
        "evolution-first",
        |a| a["unsupportedRoles"] = json!(["ATTACKER"]),
        "unsupported role",
    );
}

#[test]
fn rejects_use_now_summary_when_required_evidence_or_conditions_are_missing() {
    rejected(
        "usable-now-optional-investment",
        |a| {
            a["missingEvidence"] = json!(["identity"]);
        },
        "missing required evidence",
    );
    for conditions in [
        Value::Null,
        json!([{"description":"Fictional unmet condition","state":"unmet"}]),
        json!([{"description":"Fictional unknown condition","state":"unknown"}]),
    ] {
        rejected(
            "usable-now-optional-investment",
            |a| {
                a["paths"][0]["steps"][0]["conditions"] = conditions;
            },
            "unmet or unknown conditions",
        );
    }
}

#[test]
fn rejects_suggestions_on_non_current_assessments_and_version_mismatches() {
    for name in ["protection-conflict", "stale-context-report"] {
        rejected(
            name,
            |a| {
                a["suggestedAction"] =
                    json!({"pathId":"prepare","stepId":"prepare-evolve","kind":"evolve"})
            },
            "withhold",
        );
        rejected(
            name,
            |a| a["suggestedSummaryTag"] = json!({"kind":"action","tag":"EVOLVE"}),
            "withhold",
        );
    }
    rejected(
        "evolution-first",
        |a| {
            a["validity"] = json!("invalid");
            a["validityReasons"] = json!(["Fictional contract revoked"]);
        },
        "withhold",
    );
    rejected(
        "evolution-first",
        |a| a["provenance"]["current"]["observationRevision"] = json!("42"),
        "changed context",
    );
    rejected(
        "evolution-first",
        |a| a["provenance"]["current"]["rulesetVersion"] = json!("fictional-rules-2"),
        "changed context",
    );
    rejected(
        "evolution-first",
        |a| a["provenance"]["current"]["datasets"][0]["version"] = json!("fictional-2"),
        "changed context",
    );
    rejected(
        "stale-context-report",
        |a| a["validityReasons"] = json!([]),
        "needs a reason",
    );
}

#[test]
fn rejects_unsupported_versions_malformed_revisions_and_resource_integers() {
    rejected(
        "evolution-first",
        |a| a["contractVersion"] = json!("utility-2"),
        "unsupported utility contract",
    );
    for value in ["", "-1", "01", "1.5", "1e3", " 1", "9223372036854775808"] {
        rejected(
            "evolution-first",
            |a| a["provenance"]["evaluated"]["observationRevision"] = json!(value),
            "observationRevision",
        );
        rejected(
            "evolution-first",
            |a| a["constraints"]["budget"][0]["amount"] = json!(value),
            "constraints.budget",
        );
    }
    rejected(
        "evolution-first",
        |a| a["provenance"]["evaluated"]["rulesetVersion"] = json!("bad version"),
        "rulesetVersion",
    );
    rejected(
        "evolution-first",
        |a| a["provenance"]["evaluated"]["datasets"][0]["sha256"] = json!("not-a-digest"),
        "sha256",
    );
    rejected(
        "evolution-first",
        |a| a["provenance"]["evaluated"]["model"] = json!({"mode":"trained"}),
        "unknown variant",
    );
    rejected(
        "evolution-first",
        |a| a["provenance"]["evaluated"]["observationRevision"] = json!(1),
        "invalid type",
    );
}

#[test]
fn deterministic_only_model_rejects_extra_fields_instead_of_losing_provenance() {
    rejected(
        "evolution-first",
        |a| {
            a["provenance"]["evaluated"]["model"]["modelVersion"] = json!("unreviewed");
        },
        "unknown field",
    );
}

#[test]
fn enum_values_require_the_renderer_string_shape_not_serde_object_aliases() {
    rejected(
        "evolution-first",
        |a| {
            a["goal"]["role"] = json!({"ATTACKER":null});
        },
        "invalid type",
    );
    rejected(
        "evolution-first",
        |a| {
            a["paths"][0]["kind"] = json!({"prepare":null});
        },
        "invalid type",
    );
}

#[test]
fn actionable_evolution_targets_need_a_matching_dataset_provenance_namespace() {
    rejected(
        "evolution-first",
        |a| {
            a["paths"][0]["steps"][0]["action"]["target"]["datasetId"] = json!("unrecorded-data");
        },
        "recorded dataset provenance",
    );
    rejected(
        "evolution-first",
        |a| {
            a["provenance"]["evaluated"]["datasets"] = Value::Null;
            a["provenance"]["current"]["datasets"] = Value::Null;
        },
        "recorded dataset provenance",
    );
}

#[test]
fn rejects_invalid_numeric_measurements_and_mismatched_target_context() {
    for (min, max) in [(-1.0, 2.0), (3.0, 2.0), (0.0, 1_000_001.0)] {
        rejected(
            "power-up-first",
            |a| {
                a["paths"][0]["steps"][0]["action"]["target"]["minimum"] = json!(min);
                a["paths"][0]["steps"][0]["action"]["target"]["maximum"] = json!(max);
            },
            "measurement range",
        );
    }
    let mut a = load(&fixture("power-up-first")).unwrap();
    if let StepAction::PowerUp { target } = &mut a.paths[0].steps[0].action {
        target.maximum = f64::INFINITY;
    }
    assert!(a
        .validate(INDIVIDUAL)
        .unwrap_err()
        .to_string()
        .contains("finite"));
    rejected(
        "power-up-first",
        |a| a["paths"][0]["steps"][0]["action"]["target"]["contextId"] = json!("wrong"),
        "target context differs",
    );
}

#[test]
fn rejects_unbounded_lists_text_and_documents() {
    rejected(
        "evolution-first",
        |a| a["currentUsefulness"] = json!("x".repeat(501)),
        "500 characters",
    );
    rejected(
        "evolution-first",
        |a| a["goal"]["id"] = json!("x".repeat(65)),
        "1–64",
    );
    rejected(
        "evolution-first",
        |a| a["limitations"] = json!(vec!["fictional"; MAX_ITEMS + 1]),
        "at most 16",
    );
    rejected(
        "evolution-first",
        |a| a["paths"] = json!(vec![a["paths"][0].clone(); MAX_PATHS + 1]),
        "at most 4",
    );
    rejected(
        "evolution-first",
        |a| a["paths"][0]["steps"] = json!(vec![a["paths"][0]["steps"][0].clone(); MAX_STEPS + 1]),
        "at most 8",
    );
    assert!(
        Assessment::load(&vec![b' '; MAX_DOCUMENT_BYTES + 1], INDIVIDUAL)
            .unwrap_err()
            .to_string()
            .contains("128 KiB")
    );
}

#[test]
fn rejects_missing_explicit_nulls_unknown_fields_and_malformed_json() {
    for field in [
        "currentRoles",
        "potentialRoles",
        "suggestedAction",
        "suggestedSummaryTag",
    ] {
        rejected(
            "evolution-first",
            |a| {
                a.as_object_mut().unwrap().remove(field);
            },
            "missing field",
        );
    }
    rejected(
        "evolution-first",
        |a| a["assignedTags"] = json!(["EVOLVE"]),
        "unknown field",
    );
    rejected(
        "evolution-first",
        |a| {
            a["paths"][0]
                .as_object_mut()
                .unwrap()
                .remove("estimatedCosts");
        },
        "missing field",
    );
    rejected(
        "evolution-first",
        |a| {
            a["paths"][0]["steps"][0]
                .as_object_mut()
                .unwrap()
                .remove("conditions");
        },
        "missing field",
    );
    assert!(Assessment::load(b"{", INDIVIDUAL)
        .unwrap_err()
        .to_string()
        .contains("invalid utility JSON"));
}

#[test]
fn rejects_dangling_reports_duplicate_resources_and_contradictory_path_kinds() {
    rejected(
        "stale-context-report",
        |a| a["actionReports"][0]["stepId"] = json!("absent"),
        "existing path and step",
    );
    rejected(
        "power-up-first",
        |a| {
            let cost = a["paths"][0]["estimatedCosts"][0].clone();
            a["paths"][0]["estimatedCosts"]
                .as_array_mut()
                .unwrap()
                .push(cost);
        },
        "duplicate resource",
    );
    rejected(
        "power-up-first",
        |a| a["paths"][0]["kind"] = json!("use_now"),
        "only role use",
    );
    rejected(
        "power-up-first",
        |a| a["paths"][0]["kind"] = json!("wait"),
        "evidence guidance only",
    );
    rejected(
        "power-up-first",
        |a| a["paths"][0]["steps"][1]["required"] = json!(false),
        "required role use",
    );
}
