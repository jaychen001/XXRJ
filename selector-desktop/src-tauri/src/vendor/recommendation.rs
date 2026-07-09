use std::cmp::Ordering;

use super::field_mapping::normalize_value;
use super::models::{
    MatchRuleResult, RecommendationCandidate, RecommendationRequest, RecommendationRequirement,
    VendorModelRecord,
};
use super::recommendation_aliases::field_aliases;

pub use super::recommendation_aliases::infer_component_type;

pub fn recommend_models(
    request: &RecommendationRequest,
    models: Vec<VendorModelRecord>,
) -> Vec<RecommendationCandidate> {
    if request.requirements.is_empty() {
        return Vec::new();
    }

    let mut candidates = models
        .into_iter()
        .map(|model| score_model(request, model))
        .filter(|candidate| {
            !candidate.matched_rules.is_empty() || !candidate.failed_rules.is_empty()
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.failed_rules.len().cmp(&right.failed_rules.len()))
            .then_with(|| left.model.model_name.cmp(&right.model.model_name))
    });
    candidates.truncate(request.limit.unwrap_or(10).max(1));
    candidates
}

fn score_model(
    request: &RecommendationRequest,
    model: VendorModelRecord,
) -> RecommendationCandidate {
    let mut matched_rules = Vec::new();
    let mut failed_rules = Vec::new();

    for requirement in &request.requirements {
        let (required_value, required_unit) = normalize_requirement(requirement);
        match find_candidate_value(requirement, &model, &required_unit) {
            CandidateLookup::Value {
                value: candidate_value,
                unit,
            } if candidate_value >= required_value => {
                matched_rules.push(MatchRuleResult {
                    requirement_id: requirement.id.clone(),
                    label: requirement.label.clone(),
                    message: format!(
                        "{} 满足：{:.3} {} >= {:.3} {}",
                        requirement.label, candidate_value, unit, required_value, unit
                    ),
                    required_value,
                    candidate_value: Some(candidate_value),
                    unit,
                });
            }
            CandidateLookup::Value {
                value: candidate_value,
                unit,
            } => {
                failed_rules.push(MatchRuleResult {
                    requirement_id: requirement.id.clone(),
                    label: requirement.label.clone(),
                    message: format!(
                        "{} 不足：{:.3} {} < {:.3} {}",
                        requirement.label, candidate_value, unit, required_value, unit
                    ),
                    required_value,
                    candidate_value: Some(candidate_value),
                    unit,
                });
            }
            CandidateLookup::UnitMismatch { candidate_unit } => {
                failed_rules.push(MatchRuleResult {
                    requirement_id: requirement.id.clone(),
                    label: requirement.label.clone(),
                    message: format!(
                        "{} 单位不一致：型号库为 {}，需求为 {}",
                        requirement.label,
                        display_unit(&candidate_unit),
                        display_unit(&required_unit)
                    ),
                    required_value,
                    candidate_value: None,
                    unit: required_unit,
                });
            }
            CandidateLookup::Missing => {
                failed_rules.push(MatchRuleResult {
                    requirement_id: requirement.id.clone(),
                    label: requirement.label.clone(),
                    message: format!("型号库缺少 {} 对应参数", requirement.label),
                    required_value,
                    candidate_value: None,
                    unit: required_unit,
                });
            }
        }
    }

    let total = request.requirements.len().max(1) as f64;
    let score = matched_rules.len() as f64 / total - failed_rules.len() as f64 * 0.05;
    RecommendationCandidate {
        model,
        score: score.max(0.0),
        matched_rules,
        failed_rules,
    }
}

enum CandidateLookup {
    Value { value: f64, unit: String },
    UnitMismatch { candidate_unit: String },
    Missing,
}

fn find_candidate_value(
    requirement: &RecommendationRequirement,
    model: &VendorModelRecord,
    required_unit: &str,
) -> CandidateLookup {
    let aliases = field_aliases(&requirement.id);
    let mut mismatched_unit = None;
    for alias in aliases {
        let Some(parameter) = model.normalized_parameters.get(&alias) else {
            continue;
        };
        let (value, unit) = normalize_value(parameter.value, &parameter.unit);
        if units_compatible(&unit, required_unit) {
            return CandidateLookup::Value { value, unit };
        }
        mismatched_unit.get_or_insert(unit);
    }
    mismatched_unit
        .map(|candidate_unit| CandidateLookup::UnitMismatch { candidate_unit })
        .unwrap_or(CandidateLookup::Missing)
}

fn normalize_requirement(requirement: &RecommendationRequirement) -> (f64, String) {
    normalize_value(requirement.value, &requirement.unit)
}

fn units_compatible(candidate_unit: &str, required_unit: &str) -> bool {
    let candidate = candidate_unit.trim();
    let required = required_unit.trim();
    !candidate.is_empty() && !required.is_empty() && candidate.eq_ignore_ascii_case(required)
}

fn display_unit(unit: &str) -> &str {
    if unit.trim().is_empty() {
        "未标单位"
    } else {
        unit
    }
}
