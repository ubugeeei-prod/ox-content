use crate::index::{Field, SearchDocument};
use crate::scope::{
    SearchQuery, SearchQueryFilter, get_search_document_scopes, matches_search_scopes,
};
use crate::tokenizer::tokenize_query;
use serde::{Deserialize, Serialize};

pub(super) const PHRASE_MATCH_BOOST: f64 = 3.0;

/// Search result metadata for rich result cards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultMetadata {
    /// Hierarchical scopes derived from the document id or URL.
    pub scopes: Vec<String>,

    /// Best section context, usually a matching heading.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,

    /// Matched locale when a locale filter is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    /// Matched version when a version filter is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Active query filters that shaped this result set.
    pub filters: Vec<SearchQueryFilter>,
}

/// Stable author-facing ranking details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultRanking {
    /// Final relevance score.
    pub score: f64,

    /// Document fields that contributed to the score.
    pub fields: Vec<String>,

    /// Human-readable, stable ranking reasons.
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum MatchKind {
    Term,
    Prefix,
    Fuzzy,
}

impl MatchKind {
    fn label(self) -> &'static str {
        match self {
            Self::Term => "term",
            Self::Prefix => "prefix",
            Self::Fuzzy => "fuzzy",
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct SearchCandidate {
    pub(super) score: f64,
    pub(super) matches: Vec<String>,
    fields: Vec<Field>,
    reasons: Vec<String>,
}

impl SearchCandidate {
    pub(super) fn add_term_match(&mut self, term: &str, field: Field, kind: MatchKind) {
        push_unique_string(&mut self.matches, term.to_owned());
        push_unique_field(&mut self.fields, field);
        push_unique_string(
            &mut self.reasons,
            format!("{} {} match: {term}", field_label(field), kind.label()),
        );
    }

    pub(super) fn add_phrase_match(&mut self, phrase: &str, field: Field) {
        push_unique_string(&mut self.matches, phrase.to_owned());
        push_unique_field(&mut self.fields, field);
        push_unique_string(
            &mut self.reasons,
            format!("{} phrase match: {phrase}", field_label(field)),
        );
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SearchConstraints<'a> {
    scopes: &'a [String],
    locale: Option<&'a str>,
    version: Option<&'a str>,
}

impl<'a> SearchConstraints<'a> {
    pub(super) fn from_query(query: &'a SearchQuery) -> Self {
        Self {
            scopes: &query.scopes,
            locale: query.first_filter_value(&["locale"]),
            version: query.first_filter_value(&["version"]),
        }
    }

    pub(super) fn has_refinements(self) -> bool {
        !self.scopes.is_empty() || self.locale.is_some() || self.version.is_some()
    }

    pub(super) fn matches(self, doc: &SearchDocument) -> bool {
        matches_search_scopes(&doc.id, &doc.url, self.scopes)
            && self.locale.is_none_or(|locale| doc_matches_locale(doc, locale, self.version))
            && self.version.is_none_or(|version| doc_matches_version(doc, version))
    }
}

pub(super) fn term_tokens(query: &SearchQuery) -> Vec<String> {
    query.terms.iter().flat_map(|term| tokenize_query(term)).collect()
}

pub(super) fn phrase_tokens(query: &SearchQuery) -> Vec<String> {
    query.phrases.iter().flat_map(|phrase| tokenize_query(phrase)).collect()
}

pub(super) fn phrase_fields(doc: &SearchDocument, phrase: &str) -> Vec<Field> {
    let mut fields = Vec::new();
    if contains_query_phrase(&doc.title, phrase) {
        fields.push(Field::Title);
    }
    if doc.headings.iter().any(|heading| contains_query_phrase(heading, phrase)) {
        fields.push(Field::Heading);
    }
    if contains_query_phrase(&doc.body, phrase) {
        fields.push(Field::Body);
    }
    if doc.code.iter().any(|code| contains_query_phrase(code, phrase)) {
        fields.push(Field::Code);
    }
    fields
}

pub(super) fn result_metadata(
    doc: &SearchDocument,
    matches: &[String],
    constraints: SearchConstraints<'_>,
    filters: Vec<SearchQueryFilter>,
) -> SearchResultMetadata {
    SearchResultMetadata {
        scopes: get_search_document_scopes(&doc.id, &doc.url),
        section: matching_section(doc, matches),
        locale: constraints.locale.and_then(|locale| {
            doc_matches_locale(doc, locale, constraints.version).then(|| locale.to_string())
        }),
        version: constraints
            .version
            .and_then(|version| doc_matches_version(doc, version).then(|| version.to_string())),
        filters,
    }
}

pub(super) fn result_ranking(
    score: f64,
    candidate: SearchCandidate,
    query: &SearchQuery,
) -> SearchResultRanking {
    let mut fields =
        candidate.fields.into_iter().map(field_label).map(String::from).collect::<Vec<_>>();
    fields.sort_by_key(|field| field_order(field));
    fields.dedup();

    let mut reasons = candidate.reasons;
    for scope in &query.scopes {
        push_unique_string(&mut reasons, format!("scope filter: {scope}"));
    }
    for filter in &query.filters {
        push_unique_string(&mut reasons, format!("{} filter: {}", filter.name, filter.value));
    }
    reasons.sort();
    reasons.dedup();

    SearchResultRanking { score, fields, reasons }
}

pub(super) fn result_aria_label(
    doc: &SearchDocument,
    metadata: &SearchResultMetadata,
    ranking: &SearchResultRanking,
) -> String {
    let mut parts = vec![if doc.title.is_empty() {
        "Untitled search result".to_string()
    } else {
        doc.title.clone()
    }];
    if let Some(section) = &metadata.section {
        parts.push(format!("section {section}"));
    }
    if let Some(locale) = &metadata.locale {
        parts.push(format!("language {locale}"));
    }
    if let Some(version) = &metadata.version {
        parts.push(format!("version {version}"));
    }
    if !ranking.reasons.is_empty() {
        parts.push(format!("{} ranking reasons", ranking.reasons.len()));
    }
    parts.join(", ")
}

fn matching_section(doc: &SearchDocument, matches: &[String]) -> Option<String> {
    for heading in &doc.headings {
        if matches.iter().any(|term| contains_query_phrase(heading, term)) {
            return Some(heading.clone());
        }
    }

    doc.headings.first().filter(|heading| !heading.is_empty()).cloned()
}

fn contains_query_phrase(value: &str, phrase: &str) -> bool {
    value.to_lowercase().contains(phrase)
}

fn doc_matches_locale(doc: &SearchDocument, locale: &str, version: Option<&str>) -> bool {
    let mut source = normalized_doc_source(doc);
    if let Some(version) = version {
        source = strip_version_segment(&source, version);
    }
    first_segment(&source).is_some_and(|segment| segment == locale)
}

fn doc_matches_version(doc: &SearchDocument, version: &str) -> bool {
    let source = normalized_doc_source(doc);
    source == version
        || source.starts_with(&format!("{version}/"))
        || source.split('/').any(|segment| segment == version)
}

fn strip_version_segment(source: &str, version: &str) -> String {
    if source == version {
        return String::new();
    }
    if let Some(stripped) = source.strip_prefix(&format!("{version}/")) {
        return stripped.to_string();
    }

    let needle = format!("/{version}/");
    if let Some(position) = source.find(&needle) {
        return source[position + needle.len()..].to_string();
    }

    source.to_string()
}

fn normalized_doc_source(doc: &SearchDocument) -> String {
    let source = if doc.id.is_empty() { &doc.url } else { &doc.id };
    source.trim_start_matches('/').to_lowercase()
}

fn first_segment(source: &str) -> Option<&str> {
    source.split('/').find(|segment| !segment.is_empty())
}

fn field_label(field: Field) -> &'static str {
    match field {
        Field::Title => "title",
        Field::Heading => "heading",
        Field::Body => "body",
        Field::Code => "code",
    }
}

fn field_order(field: &str) -> u8 {
    match field {
        "title" => 0,
        "heading" => 1,
        "body" => 2,
        "code" => 3,
        _ => 4,
    }
}

fn push_unique_string(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn push_unique_field(values: &mut Vec<Field>, value: Field) {
    if !values.contains(&value) {
        values.push(value);
    }
}
