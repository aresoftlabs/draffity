//! AI validators (Épica G). Analytical passes over a chapter that surface
//! consistency / craft issues. Three are model-backed (character, voice, plot)
//! and two are local heuristics (repetition, style) — the local ones need no
//! network and are fast, the AI ones build a JSON-constrained prompt and parse
//! the response.
//!
//! Everything except the orchestrator's HTTP call is a **pure function**
//! (prompt builders, JSON parser, local detectors, codex coverage), so the
//! logic is unit-tested without a network. The orchestrator (`OpenRouterValidators`)
//! just wires the model in. See `backlog-v4.md` G-01..G-08.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::services::ai::{AIService, ChatMessage, CompletionRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

impl Severity {
    fn parse(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "critical" | "crítico" | "critico" => Severity::Critical,
            "info" => Severity::Info,
            _ => Severity::Warning,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidatorKind {
    Character,
    Voice,
    Repetition,
    Plot,
    Style,
}

impl ValidatorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ValidatorKind::Character => "character",
            ValidatorKind::Voice => "voice",
            ValidatorKind::Repetition => "repetition",
            ValidatorKind::Plot => "plot",
            ValidatorKind::Style => "style",
        }
    }

    pub fn parse(s: &str) -> AppResult<Self> {
        match s {
            "character" => Ok(ValidatorKind::Character),
            "voice" => Ok(ValidatorKind::Voice),
            "repetition" => Ok(ValidatorKind::Repetition),
            "plot" => Ok(ValidatorKind::Plot),
            "style" => Ok(ValidatorKind::Style),
            other => Err(AppError::Invariant(format!(
                "validador desconocido: {other}"
            ))),
        }
    }

    /// Local heuristics need no model; AI validators do.
    fn is_local(self) -> bool {
        matches!(self, ValidatorKind::Repetition | ValidatorKind::Style)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub validator: String,
    pub severity: Severity,
    pub title: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

/// Owned inputs for a validation run, assembled by the command layer.
#[derive(Debug, Clone, Default)]
pub struct ValidationInput {
    /// Plain text of the chapter/document being validated.
    pub text: String,
    /// Rendered codex block (characters/places/…), for character + plot.
    pub codex_block: String,
    /// Sample of prior chapters, for the voice/tone anchor.
    pub anchor_text: String,
}

pub trait AIValidatorService: Send + Sync {
    /// Run one validator and return its findings. Local validators work
    /// regardless of tier/key; AI validators require `available()`.
    fn run(&self, kind: ValidatorKind, input: &ValidationInput) -> AppResult<Vec<Finding>>;
    /// Whether the AI-backed validators can run (premium + key).
    fn available(&self) -> bool;
}

/// Summarise findings for storage / the UI header, e.g. "1 crítico · 3 advertencias".
pub fn summarize(findings: &[Finding]) -> String {
    let mut crit = 0;
    let mut warn = 0;
    let mut info = 0;
    for f in findings {
        match f.severity {
            Severity::Critical => crit += 1,
            Severity::Warning => warn += 1,
            Severity::Info => info += 1,
        }
    }
    if findings.is_empty() {
        return "sin hallazgos".to_string();
    }
    let mut parts = Vec::new();
    if crit > 0 {
        parts.push(format!("{crit} crítico{}", plural(crit)));
    }
    if warn > 0 {
        parts.push(format!("{warn} advertencia{}", plural(warn)));
    }
    if info > 0 {
        parts.push(format!("{info} info"));
    }
    parts.join(" · ")
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

// ---------------------------------------------------------------------------
// Prompt builders for AI validators (pure)
// ---------------------------------------------------------------------------

const JSON_CONTRACT: &str = "Devolvé EXCLUSIVAMENTE un array JSON (sin texto alrededor, sin ```), con esta forma: \
[{\"severity\":\"critical|warning|info\",\"title\":\"breve\",\"detail\":\"explicación\",\"excerpt\":\"fragmento exacto del texto\",\"suggestion\":\"arreglo propuesto\"}]. \
Si no encontrás problemas, devolvé exactamente [].";

/// Build the chat messages for an AI validator. `Repetition`/`Style` are local
/// and don't go through here (returns an error if asked).
pub fn messages_for(kind: ValidatorKind, input: &ValidationInput) -> AppResult<Vec<ChatMessage>> {
    let system = match kind {
        ValidatorKind::Character => format!(
            "Eres un editor de continuidad narrativa. Te doy el catálogo (codex) de entidades del proyecto \
y un fragmento del manuscrito. Detectá contradicciones entre el texto y el codex (apariencia, rasgos, \
motivaciones) o entre menciones dentro del propio texto. {JSON_CONTRACT}\n\n=== Codex ===\n{}",
            input.codex_block
        ),
        ValidatorKind::Voice => format!(
            "Eres un editor de estilo. Te doy fragmentos de capítulos anteriores como REFERENCIA de voz \
narrativa y un capítulo NUEVO. Detectá desvíos de voz, punto de vista, tono o tiempo verbal del capítulo \
nuevo respecto de la referencia. {JSON_CONTRACT}\n\n=== Referencia (capítulos anteriores) ===\n{}",
            input.anchor_text
        ),
        ValidatorKind::Plot => format!(
            "Eres un editor de estructura. Te doy el codex del proyecto y un fragmento del manuscrito. \
Detectá problemas de coherencia de trama y temporalidad: causalidad rota, saltos o contradicciones \
temporales, hilos abandonados y eventos establecidos que se ignoran. {JSON_CONTRACT}\n\n=== Codex ===\n{}",
            input.codex_block
        ),
        ValidatorKind::Repetition | ValidatorKind::Style => {
            return Err(AppError::Invariant(
                "este validador es local, no usa prompt".into(),
            ))
        }
    };
    Ok(vec![
        ChatMessage::system(system),
        ChatMessage::user(input.text.clone()),
    ])
}

// ---------------------------------------------------------------------------
// JSON parsing of AI findings (pure, tolerant)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct RawFinding {
    #[serde(default)]
    severity: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    detail: Option<String>,
    #[serde(default)]
    excerpt: Option<String>,
    #[serde(default)]
    suggestion: Option<String>,
}

/// Parse the model's JSON array into findings. Tolerant: extracts the first
/// `[ … ]` block (models sometimes wrap it in prose or fences), skips entries
/// with neither title nor detail, and defaults a missing severity to Warning.
pub fn parse_findings(validator: ValidatorKind, raw: &str) -> Vec<Finding> {
    let Some(slice) = extract_json_array(raw) else {
        return Vec::new();
    };
    let parsed: Vec<RawFinding> = serde_json::from_str(slice).unwrap_or_default();
    parsed
        .into_iter()
        .filter_map(|r| {
            let title = r.title.unwrap_or_default().trim().to_string();
            let detail = r.detail.unwrap_or_default().trim().to_string();
            if title.is_empty() && detail.is_empty() {
                return None;
            }
            Some(Finding {
                validator: validator.as_str().to_string(),
                severity: r
                    .severity
                    .as_deref()
                    .map(Severity::parse)
                    .unwrap_or(Severity::Warning),
                title: if title.is_empty() {
                    detail.clone()
                } else {
                    title
                },
                detail,
                excerpt: r.excerpt.filter(|s| !s.trim().is_empty()),
                suggestion: r.suggestion.filter(|s| !s.trim().is_empty()),
            })
        })
        .collect()
}

fn extract_json_array(raw: &str) -> Option<&str> {
    let start = raw.find('[')?;
    let end = raw.rfind(']')?;
    if end > start {
        Some(&raw[start..=end])
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Local heuristic validators (pure)
// ---------------------------------------------------------------------------

const REPEAT_MIN_COUNT: usize = 3;
const LONG_SENTENCE_WORDS: usize = 40;
const MAX_LOCAL_FINDINGS: usize = 12;

fn is_stopword(w: &str) -> bool {
    matches!(
        w,
        // Spanish
        "el" | "la" | "los" | "las" | "un" | "una" | "unos" | "unas" | "de" | "del" | "y" | "o"
            | "que" | "en" | "a" | "se" | "su" | "sus" | "con" | "por" | "para" | "lo" | "le"
            | "les" | "al" | "es" | "no" | "más" | "como" | "pero"
            // English
            | "the" | "an" | "of" | "and" | "or" | "to" | "in" | "is" | "it" | "that"
            | "for" | "with" | "as" | "was" | "were" | "be" | "on" | "at" | "by" | "he" | "she"
            | "they"
    )
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect()
}

/// Detect repeated phrases (bigrams + trigrams) that appear often enough to
/// read as a tic. A gram is reported only if it carries at least one content
/// word (not all stopwords). Local, no model.
pub fn detect_repetitions(text: &str) -> Vec<Finding> {
    let tokens = tokenize(text);
    if tokens.len() < 2 {
        return Vec::new();
    }
    let mut counts: HashMap<String, usize> = HashMap::new();
    for n in [3usize, 2usize] {
        if tokens.len() < n {
            continue;
        }
        for window in tokens.windows(n) {
            if window.iter().all(|w| is_stopword(w) || w.len() < 2) {
                continue;
            }
            *counts.entry(window.join(" ")).or_insert(0) += 1;
        }
    }
    let mut hits: Vec<(String, usize)> = counts
        .into_iter()
        .filter(|(phrase, c)| *c >= REPEAT_MIN_COUNT && phrase.contains(' '))
        .collect();
    // Most-repeated first; longer phrases win ties (more specific).
    hits.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.len().cmp(&a.0.len())));
    hits.truncate(MAX_LOCAL_FINDINGS);
    hits.into_iter()
        .map(|(phrase, count)| Finding {
            validator: ValidatorKind::Repetition.as_str().to_string(),
            severity: Severity::Info,
            title: format!("Frase repetida: «{phrase}»"),
            detail: format!("Aparece {count} veces. Considerá variar la redacción."),
            excerpt: Some(phrase),
            suggestion: None,
        })
        .collect()
}

/// Detect style smells: passive-voice constructions, adverb pile-ups
/// (`-mente` / `-ly`) and very long sentences. Local, no model.
pub fn detect_style_anomalies(text: &str) -> Vec<Finding> {
    let mut out = Vec::new();

    for sentence in split_sentences(text) {
        let words: Vec<&str> = sentence.split_whitespace().collect();
        if words.len() > LONG_SENTENCE_WORDS {
            out.push(Finding {
                validator: ValidatorKind::Style.as_str().to_string(),
                severity: Severity::Info,
                title: format!("Oración muy larga ({} palabras)", words.len()),
                detail: "Las oraciones largas pueden cansar al lector; evaluá dividirla.".into(),
                excerpt: Some(truncate(sentence, 160)),
                suggestion: None,
            });
        }
        let adverbs = words
            .iter()
            .filter(|w| {
                let lw = w.trim_matches(|c: char| !c.is_alphabetic()).to_lowercase();
                lw.len() > 5 && (lw.ends_with("mente") || lw.ends_with("ly"))
            })
            .count();
        if adverbs >= 3 {
            out.push(Finding {
                validator: ValidatorKind::Style.as_str().to_string(),
                severity: Severity::Info,
                title: format!("Acumulación de adverbios ({adverbs})"),
                detail: "Varios adverbios en una oración suelen debilitar la prosa.".into(),
                excerpt: Some(truncate(sentence, 160)),
                suggestion: None,
            });
        }
        if looks_passive(sentence) {
            out.push(Finding {
                validator: ValidatorKind::Style.as_str().to_string(),
                severity: Severity::Info,
                title: "Posible voz pasiva".into(),
                detail: "La voz activa suele ser más directa.".into(),
                excerpt: Some(truncate(sentence, 160)),
                suggestion: None,
            });
        }
        if out.len() >= MAX_LOCAL_FINDINGS {
            break;
        }
    }
    out.truncate(MAX_LOCAL_FINDINGS);
    out
}

fn split_sentences(text: &str) -> Vec<&str> {
    text.split(['.', '!', '?', '\n'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Heuristic passive voice: a copular/auxiliary verb followed (within a couple
/// of tokens) by a past participle. Spanish (`ser` + regular `-ado/-ido`) +
/// English (`be` + `-ed/-en`). Conservative on purpose — irregular Spanish
/// participles (escrito, hecho, visto) are missed to keep false positives low,
/// since this is an `info` hint, not a model call.
fn looks_passive(sentence: &str) -> bool {
    let words: Vec<String> = sentence
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()).to_lowercase())
        .collect();
    let aux = [
        "es", "son", "fue", "fueron", "era", "eran", "será", "serán", "sido", "is", "are", "was",
        "were", "been", "be",
    ];
    for (i, w) in words.iter().enumerate() {
        if aux.contains(&w.as_str()) {
            for next in words.iter().skip(i + 1).take(2) {
                if next.len() > 4
                    && (next.ends_with("ado")
                        || next.ends_with("ido")
                        || next.ends_with("ada")
                        || next.ends_with("idas")
                        || next.ends_with("ed")
                        || next.ends_with("en"))
                {
                    return true;
                }
            }
        }
    }
    false
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let t: String = s.chars().take(max).collect();
        format!("{t}…")
    } else {
        s.to_string()
    }
}

// ---------------------------------------------------------------------------
// Codex coverage pre-check (G-03, pure)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CoverageReport {
    /// Recurring capitalised names found in the text.
    pub candidates: usize,
    /// How many are covered by a codex entry.
    pub covered: usize,
}

impl CoverageReport {
    pub fn ratio(&self) -> f32 {
        if self.candidates == 0 {
            1.0
        } else {
            self.covered as f32 / self.candidates as f32
        }
    }
    /// Sparse enough that character/plot validation will be unreliable.
    pub fn is_sparse(&self) -> bool {
        self.candidates >= 3 && self.ratio() < 0.5
    }
}

/// Estimate how much of the text's apparent cast is described in the codex.
/// Heuristic: recurring capitalised tokens are candidate names; a candidate is
/// "covered" if a codex name matches it (case-insensitive, either contains).
pub fn codex_coverage(text: &str, codex_names: &[String]) -> CoverageReport {
    let mut freq: HashMap<String, usize> = HashMap::new();
    for raw in text.split_whitespace() {
        let w = raw.trim_matches(|c: char| !c.is_alphanumeric());
        if w.chars().count() < 3 {
            continue;
        }
        let mut chars = w.chars();
        let first = chars.next().unwrap();
        if first.is_uppercase() && w.chars().all(|c| c.is_alphabetic()) {
            *freq.entry(w.to_lowercase()).or_insert(0) += 1;
        }
    }
    let names_lc: Vec<String> = codex_names.iter().map(|n| n.to_lowercase()).collect();
    let mut candidates = 0;
    let mut covered = 0;
    for (cand, count) in &freq {
        if *count < 2 {
            continue;
        }
        candidates += 1;
        if names_lc
            .iter()
            .any(|n| n == cand || n.contains(cand.as_str()) || cand.contains(n.as_str()))
        {
            covered += 1;
        }
    }
    CoverageReport {
        candidates,
        covered,
    }
}

// ---------------------------------------------------------------------------
// Orchestrator
// ---------------------------------------------------------------------------

pub struct OpenRouterValidators {
    ai: Arc<dyn AIService>,
}

impl OpenRouterValidators {
    pub fn new(ai: Arc<dyn AIService>) -> Self {
        Self { ai }
    }
}

impl AIValidatorService for OpenRouterValidators {
    fn available(&self) -> bool {
        self.ai.available()
    }

    fn run(&self, kind: ValidatorKind, input: &ValidationInput) -> AppResult<Vec<Finding>> {
        if kind.is_local() {
            return Ok(match kind {
                ValidatorKind::Repetition => detect_repetitions(&input.text),
                ValidatorKind::Style => detect_style_anomalies(&input.text),
                _ => unreachable!(),
            });
        }
        if !self.ai.available() {
            return Err(AppError::Unsupported(
                "las funciones de IA no están activas".into(),
            ));
        }
        let messages = messages_for(kind, input)?;
        let resp = self.ai.complete(CompletionRequest {
            messages,
            model: None,
            temperature: Some(0.2),
            max_tokens: Some(1500),
        })?;
        Ok(parse_findings(kind, &resp.text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ai::CompletionResponse;

    #[test]
    fn parse_findings_extracts_array_from_prose_and_fences() {
        let raw = "Claro, aquí están:\n```json\n[{\"severity\":\"critical\",\"title\":\"Ojos\",\"detail\":\"dice verdes pero el codex azules\",\"excerpt\":\"ojos verdes\"}]\n```";
        let f = parse_findings(ValidatorKind::Character, raw);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].severity, Severity::Critical);
        assert_eq!(f[0].validator, "character");
        assert_eq!(f[0].excerpt.as_deref(), Some("ojos verdes"));
    }

    #[test]
    fn parse_findings_empty_array_and_garbage() {
        assert!(parse_findings(ValidatorKind::Voice, "[]").is_empty());
        assert!(parse_findings(ValidatorKind::Voice, "no hay json aquí").is_empty());
    }

    #[test]
    fn parse_findings_skips_entries_without_title_or_detail() {
        let raw = r#"[{"severity":"info"},{"title":"ok","detail":"x"}]"#;
        assert_eq!(parse_findings(ValidatorKind::Plot, raw).len(), 1);
    }

    #[test]
    fn repetition_flags_overused_phrase() {
        let text =
            "—dijo ella con calma. —dijo ella mirando al mar. Más tarde —dijo ella otra vez.";
        let f = detect_repetitions(text);
        assert!(
            f.iter().any(|x| x.excerpt.as_deref() == Some("dijo ella")),
            "expected 'dijo ella' repetition, got {f:?}"
        );
    }

    #[test]
    fn repetition_ignores_pure_stopword_grams() {
        let text = "de la de la de la de la de la";
        assert!(detect_repetitions(text).is_empty());
    }

    #[test]
    fn style_flags_long_sentence_and_passive() {
        let long = "palabra ".repeat(45);
        let f = detect_style_anomalies(&long);
        assert!(f.iter().any(|x| x.title.contains("muy larga")));

        // Conservative heuristic: regular participles (-ado/-ido/…). Irregulars
        // like "escrita" are a documented miss — info-level, precision over recall.
        let passive = detect_style_anomalies("El puente fue construido por ingenieros.");
        assert!(passive.iter().any(|x| x.title.contains("pasiva")));
    }

    #[test]
    fn coverage_flags_sparse_codex() {
        let text = "Aragorn miró a Boromir. Aragorn habló. Boromir asintió. Legolas disparó. Legolas corrió.";
        // Codex only has Aragorn → Boromir/Legolas uncovered.
        let cov = codex_coverage(text, &["Aragorn".to_string()]);
        assert!(cov.candidates >= 3, "candidates: {}", cov.candidates);
        assert!(cov.covered >= 1);
        assert!(cov.is_sparse(), "ratio {}", cov.ratio());
    }

    #[test]
    fn coverage_not_sparse_when_well_described() {
        let text = "Aragorn miró a Boromir. Aragorn habló. Boromir asintió.";
        let cov = codex_coverage(text, &["Aragorn".to_string(), "Boromir".to_string()]);
        assert!(!cov.is_sparse());
    }

    #[test]
    fn summarize_counts_by_severity() {
        let findings = vec![
            Finding {
                validator: "character".into(),
                severity: Severity::Critical,
                title: "a".into(),
                detail: "a".into(),
                excerpt: None,
                suggestion: None,
            },
            Finding {
                validator: "character".into(),
                severity: Severity::Warning,
                title: "b".into(),
                detail: "b".into(),
                excerpt: None,
                suggestion: None,
            },
        ];
        assert_eq!(summarize(&findings), "1 crítico · 1 advertencia");
        assert_eq!(summarize(&[]), "sin hallazgos");
    }

    // --- orchestrator with a stub AIService ---
    struct StubAI {
        available: bool,
        reply: String,
    }
    impl AIService for StubAI {
        fn available(&self) -> bool {
            self.available
        }
        fn complete(&self, _req: CompletionRequest) -> AppResult<CompletionResponse> {
            Ok(CompletionResponse {
                text: self.reply.clone(),
                usage: None,
            })
        }
    }

    #[test]
    fn orchestrator_runs_local_without_ai() {
        let svc = OpenRouterValidators::new(Arc::new(StubAI {
            available: false,
            reply: String::new(),
        }));
        // Local validators run even when AI is unavailable.
        let f = svc
            .run(
                ValidatorKind::Repetition,
                &ValidationInput {
                    text: "dijo ella. dijo ella. dijo ella.".into(),
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(!f.is_empty());
    }

    #[test]
    fn orchestrator_ai_validator_requires_availability() {
        let svc = OpenRouterValidators::new(Arc::new(StubAI {
            available: false,
            reply: "[]".into(),
        }));
        assert!(svc
            .run(ValidatorKind::Character, &ValidationInput::default())
            .is_err());
    }

    #[test]
    fn orchestrator_parses_ai_findings() {
        let svc = OpenRouterValidators::new(Arc::new(StubAI {
            available: true,
            reply: r#"[{"severity":"warning","title":"POV","detail":"cambia a primera persona"}]"#
                .into(),
        }));
        let f = svc
            .run(ValidatorKind::Voice, &ValidationInput::default())
            .unwrap();
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].validator, "voice");
        assert_eq!(f[0].severity, Severity::Warning);
    }
}
