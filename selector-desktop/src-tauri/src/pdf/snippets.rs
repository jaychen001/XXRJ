pub(crate) fn excerpt_for_terms(text: &str, terms: &[&str], max_chars: usize) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let position = terms
        .iter()
        .filter_map(|term| text.find(term))
        .min()
        .map(|byte_index| text[..byte_index].chars().count())
        .unwrap_or(0);
    let start = position.saturating_sub(80);
    let end = usize::min(chars.len(), start + max_chars);
    chars[start..end].iter().collect::<String>()
}
