use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageRecord {
    pub id: String,
    pub chapter: String,
    pub implementation_shape: String,
    pub status: String,
    pub source_page_range: Option<String>,
    pub catalog_page: Option<String>,
    pub catalog_excerpt: String,
    pub knowledge_entry_count: i64,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSearchRecord {
    pub id: String,
    pub title: String,
    pub content: String,
    pub page: Option<String>,
    pub tags: Vec<String>,
    pub source_title: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterCandidateRecord {
    pub id: String,
    pub name: String,
    pub value: String,
    pub unit: Option<String>,
    pub scenario: String,
    pub source_page: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct NewKnowledgeEntry {
    pub id: String,
    pub page: Option<String>,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NewCoverageRecord {
    pub id: String,
    pub chapter: String,
    pub implementation_shape: String,
    pub status: String,
    pub source_page_range: Option<String>,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct NewCatalogItem {
    pub id: String,
    pub chapter: String,
    pub catalog_page: Option<String>,
    pub matched_pages: Option<String>,
    pub excerpt: String,
}

#[derive(Debug, Clone)]
pub struct NewParameterCandidate {
    pub id: String,
    pub name: String,
    pub value: String,
    pub unit: Option<String>,
    pub scenario: String,
    pub source_page: Option<String>,
}
