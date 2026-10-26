//! Offline appraisal only. No species, CP, meta or user-preference assumptions.
use crate::inventory::{AppraisalIvs, Error, Result};
use serde::Serialize;

pub const RULESET_VERSION: &str = "appraisal-1";
pub const LIMITATIONS: &[&str] = &[
    "Appraisal percentage describes the sum of confirmed IVs out of 45. It is not a PvP ranking, PvE strength score, overall quality score or transfer criterion.",
    "PvP, PvE, gym, Rocket, evolution, power-up and transfer analysis have not been performed. Species, moves, level, game data and user goals are not assessed.",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationState {
    Pending,
    Evaluated,
    NeedsData,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationScope {
    Appraisal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IvField {
    Attack,
    Defense,
    Hp,
}

/// Vocabulary for proposed classifications, never assigned by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum StandardClassification {
    #[serde(rename = "EVOLVE")]
    Evolve,
    #[serde(rename = "POWER UP")]
    PowerUp,
    #[serde(rename = "ATTACKER")]
    Attacker,
    #[serde(rename = "GYM DEF")]
    GymDefense,
    #[serde(rename = "ROCKET")]
    Rocket,
    #[serde(rename = "PVP 1500")]
    Pvp1500,
    #[serde(rename = "PVP 2500")]
    Pvp2500,
    #[serde(rename = "TRANSFER")]
    Transfer,
    #[serde(rename = "SAVE/SPECIAL")]
    SaveSpecial,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ObservedFact {
    pub field: IvField,
    pub value: u8,
}

/// The current engine emits no recommendations. A classification is a proposal,
/// separate from inventory tag assignments, with explicit evidence and limits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recommendation {
    pub code: String,
    pub proposed_classification: Option<StandardClassification>,
    pub explanation: String,
    pub observed_facts: Vec<ObservedFact>,
    pub ruleset_version: String,
    pub limitations: Vec<String>,
    pub missing_data: Vec<IvField>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingCode {
    PerfectAppraisal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub code: FindingCode,
    pub explanation: &'static str,
    pub observed_facts: Vec<ObservedFact>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppraisalResult {
    pub ivs: Option<AppraisalIvs>,
    pub total: Option<u8>,
    pub appraisal_percentage: Option<f64>,
    pub perfect: Option<bool>,
    pub findings: Vec<Finding>,
    pub recommendations: Vec<Recommendation>,
    pub missing_data: Vec<IvField>,
    pub limitations: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Evaluation {
    pub scope: EvaluationScope,
    pub state: EvaluationState,
    pub current_ruleset_version: &'static str,
    pub ruleset_version: Option<String>,
    pub evaluated_at: Option<String>,
    /// Only current results are returned; stale inputs/rules are not reconstructed.
    pub result: Option<AppraisalResult>,
}

pub fn appraise(ivs: Option<AppraisalIvs>) -> Result<AppraisalResult> {
    if ivs.is_some_and(|v| v.attack > 15 || v.defense > 15 || v.hp > 15) {
        return Err(Error::InvalidInput(
            "Each appraisal IV must be between 0 and 15",
        ));
    }
    let total = ivs.map(|v| v.attack + v.defense + v.hp);
    let perfect = total.map(|v| v == 45);
    let findings = if perfect == Some(true) {
        vec![Finding {
            code: FindingCode::PerfectAppraisal,
            explanation: "Perfect 15/15/15 appraisal. This noteworthy IV finding does not imply universal competitive superiority or recommend an action.",
            observed_facts: vec![
                ObservedFact { field: IvField::Attack, value: 15 },
                ObservedFact { field: IvField::Defense, value: 15 },
                ObservedFact { field: IvField::Hp, value: 15 },
            ],
        }]
    } else {
        vec![]
    };
    Ok(AppraisalResult {
        ivs,
        total,
        appraisal_percentage: total.map(|v| f64::from(v) * 100.0 / 45.0),
        perfect,
        findings,
        recommendations: vec![],
        missing_data: if ivs.is_none() {
            vec![IvField::Attack, IvField::Defense, IvField::Hp]
        } else {
            vec![]
        },
        limitations: LIMITATIONS,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_is_insufficient_not_zero() {
        let result = appraise(None).unwrap();
        assert_eq!(result.ivs, None);
        assert_eq!(result.total, None);
        assert_eq!(result.appraisal_percentage, None);
        assert_eq!(result.perfect, None);
        assert_eq!(
            result.missing_data,
            vec![IvField::Attack, IvField::Defense, IvField::Hp]
        );
        assert!(result.findings.is_empty());
        assert!(result.recommendations.is_empty());
    }

    #[test]
    fn all_iv_combinations_are_deterministic_descriptive_and_never_recommend() {
        for attack in 0..=15 {
            for defense in 0..=15 {
                for hp in 0..=15 {
                    let ivs = Some(AppraisalIvs {
                        attack,
                        defense,
                        hp,
                    });
                    let result = appraise(ivs).unwrap();
                    assert_eq!(result, appraise(ivs).unwrap());
                    assert_eq!(result.ivs, ivs);
                    assert_eq!(result.total, Some(attack + defense + hp));
                    assert_eq!(
                        result.appraisal_percentage,
                        Some(f64::from(attack + defense + hp) * 100.0 / 45.0)
                    );
                    let perfect = attack == 15 && defense == 15 && hp == 15;
                    assert_eq!(result.perfect, Some(perfect));
                    assert_eq!(result.findings.len(), usize::from(perfect));
                    assert!(result.recommendations.is_empty());
                    assert!(result.missing_data.is_empty());
                }
            }
        }
        for ivs in [
            AppraisalIvs {
                attack: 16,
                defense: 0,
                hp: 0,
            },
            AppraisalIvs {
                attack: 0,
                defense: 16,
                hp: 0,
            },
            AppraisalIvs {
                attack: 0,
                defense: 0,
                hp: 255,
            },
        ] {
            assert!(appraise(Some(ivs)).is_err());
        }
    }
}
