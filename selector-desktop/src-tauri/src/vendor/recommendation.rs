use std::cmp::Ordering;

use super::field_mapping::normalize_value;
use super::models::{
    MatchRuleResult, RecommendationCandidate, RecommendationRequest, RecommendationRequirement,
    VendorModelRecord,
};

pub fn recommend_models(
    request: &RecommendationRequest,
    models: Vec<VendorModelRecord>,
) -> Vec<RecommendationCandidate> {
    let mut candidates = models
        .into_iter()
        .map(|model| score_model(request, model))
        .filter(|candidate| !candidate.matched_rules.is_empty())
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.model.model_name.cmp(&right.model.model_name))
    });
    candidates.truncate(request.limit.unwrap_or(10).max(1));
    candidates
}

pub fn infer_component_type(module_id: &str) -> Option<String> {
    let lowered = module_id.to_lowercase();
    let component = if lowered.contains("timing-belt") {
        "同步轮同步带"
    } else if lowered.contains("ball-screw") {
        "滚珠丝杠"
    } else if lowered.contains("linear-guide") {
        "直线导轨"
    } else if lowered.contains("cylinder") {
        "气缸"
    } else if lowered.contains("vacuum") {
        "真空"
    } else if lowered.contains("valve") {
        "电磁阀"
    } else if lowered.contains("servo") || lowered.contains("stepper") {
        "伺服/步进电机"
    } else if lowered.contains("motor") {
        "普通电机"
    } else if lowered.contains("indexer") {
        "分割器"
    } else {
        return None;
    };
    Some(component.to_string())
}

fn score_model(request: &RecommendationRequest, model: VendorModelRecord) -> RecommendationCandidate {
    let mut matched_rules = Vec::new();
    let mut failed_rules = Vec::new();

    for requirement in &request.requirements {
        match find_candidate_value(requirement, &model) {
            Some((candidate_value, unit)) if candidate_value >= normalize_requirement(requirement).0 => {
                let required_value = normalize_requirement(requirement).0;
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
            Some((candidate_value, unit)) => {
                let required_value = normalize_requirement(requirement).0;
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
            None => {
                let (required_value, unit) = normalize_requirement(requirement);
                failed_rules.push(MatchRuleResult {
                    requirement_id: requirement.id.clone(),
                    label: requirement.label.clone(),
                    message: format!("型号库缺少 {} 对应参数", requirement.label),
                    required_value,
                    candidate_value: None,
                    unit,
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

fn find_candidate_value(
    requirement: &RecommendationRequirement,
    model: &VendorModelRecord,
) -> Option<(f64, String)> {
    let aliases = field_aliases(&requirement.id);
    aliases.iter().find_map(|alias| {
        model
            .normalized_parameters
            .get(alias)
            .map(|parameter| (parameter.value, parameter.unit.clone()))
    })
}

fn field_aliases(field: &str) -> Vec<String> {
    match field {
        "outputTorque" | "totalTorque" | "loadTorque" => {
            vec!["outputTorque", "totalTorque", "loadTorque", "ratedTorque"]
        }
        "requiredSpeed" | "motorSpeed" | "outputSpeed" => {
            vec!["requiredSpeed", "ratedSpeed", "speed", "outputSpeed"]
        }
        "power" | "ratedPower" => vec!["power", "ratedPower"],
        "load" | "loadMass" | "force" | "thrust" => vec!["load", "force", "thrust"],
        "stroke" => vec!["stroke"],
        "bore" => vec!["bore"],
        "vacuumPressure" => vec!["vacuumPressure"],
        "flowRate" => vec!["flowRate"],
        "dynamicLoad" => vec!["dynamicLoad"],
        "staticLoad" => vec!["staticLoad"],
        _ => return vec![field.to_string()],
    }
    .into_iter()
    .map(ToString::to_string)
    .collect()
}

fn normalize_requirement(requirement: &RecommendationRequirement) -> (f64, String) {
    normalize_value(requirement.value, &requirement.unit)
}
