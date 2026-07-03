use crate::knowledge::models::NewParameterCandidate;

use super::snippets::excerpt_for_terms;
use super::text_extract::PdfPageText;

const PARAMETER_KEYWORDS: [&str; 5] = ["摩擦系数", "效率", "负载率", "重力加速度", "安全系数"];

pub(crate) fn build_parameter_candidates(pages: &[PdfPageText]) -> Vec<NewParameterCandidate> {
    PARAMETER_KEYWORDS
        .iter()
        .filter_map(|keyword| {
            let page = pages.iter().find(|page| page.text.contains(keyword))?;
            let excerpt = excerpt_for_terms(&page.text, &[*keyword], 180);
            let value = extract_candidate_value(keyword, &excerpt)
                .unwrap_or_else(|| "需人工确认".to_string());
            Some(NewParameterCandidate {
                id: format!("root-pdf-{}", keyword),
                name: (*keyword).to_string(),
                value,
                unit: extract_unit(&excerpt),
                scenario: excerpt,
                source_page: Some(format!("P{}", page.page)),
            })
        })
        .collect()
}

fn extract_candidate_value(keyword: &str, excerpt: &str) -> Option<String> {
    if keyword == "重力加速度" {
        return ["9.81", "9.8"]
            .iter()
            .find(|value| excerpt.contains(**value))
            .map(|value| (*value).to_string());
    }

    let (_, tail) = excerpt.split_once(keyword)?;
    tail.split(|character: char| {
        character.is_whitespace() || "，。；;：:()（）".contains(character)
    })
    .map(clean_numeric_token)
    .find(|token| is_reasonable_parameter_value(token))
}

fn clean_numeric_token(token: &str) -> String {
    token
        .trim_matches(|character: char| {
            !character.is_ascii_digit() && character != '.' && character != '%' && character != '-'
        })
        .to_string()
}

fn is_reasonable_parameter_value(token: &str) -> bool {
    if token.is_empty() || !token.chars().any(|character| character.is_ascii_digit()) {
        return false;
    }

    token.contains('.')
        || token.contains('%')
        || token.parse::<f64>().is_ok_and(|value| value <= 10.0)
}

fn extract_unit(value: &str) -> Option<String> {
    ["%", "m/s2", "m/s²", "N", "Nm"]
        .iter()
        .find(|unit| value.contains(**unit))
        .map(|unit| (*unit).to_string())
}
