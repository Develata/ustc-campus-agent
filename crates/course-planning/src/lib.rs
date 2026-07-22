//! Deterministic, fixture-driven Course Planning core.
//!
//! The planner deliberately excludes model calls, network access, databases, and
//! enrollment side effects. It turns a validated, provenance-bearing fixture into
//! bounded candidate plans whose hard constraints are recomputed before output.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use ustc_campus_agent_core::SourceAuthority;

/// Current fixture contract implemented by this crate.
pub const FIXTURE_SCHEMA_VERSION: &str = "course-planning/v0";

/// A complete, synthetic-or-approved input to the deterministic planner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoursePlanningFixture {
    /// Fixture schema version.
    pub schema_version: String,
    /// Revision identifying the complete imported fixture.
    pub source_revision: String,
    /// Sources referenced by requirements, offerings, and community signals.
    pub sources: Vec<SourceDescriptor>,
    /// User-owned academic snapshot and planning preferences.
    pub profile: UserAcademicSnapshot,
    /// Requirement groups that every returned candidate must satisfy.
    pub requirements: Vec<RequirementGroup>,
    /// Raw offerings; lower-authority duplicate facts are resolved deterministically.
    pub courses: Vec<CourseOffering>,
    /// Optional subjective signals that may affect soft ranking only.
    #[serde(default)]
    pub community_signals: Vec<CommunitySignal>,
}

/// Provenance metadata for one source revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDescriptor {
    /// Stable source id referenced by fixture objects.
    pub id: String,
    /// Authority used for deterministic conflict resolution.
    pub authority: SourceAuthority,
    /// Source-local revision or snapshot hash label.
    pub revision: String,
    /// RFC 3339 retrieval/import timestamp supplied by the fixture producer.
    pub retrieved_at: String,
    /// Term/date range in which the facts apply, when the source supplies one.
    #[serde(default)]
    pub effective_time: Option<String>,
    /// Whether the producer has marked this revision stale.
    #[serde(default)]
    pub stale: bool,
    /// Human-readable provenance note.
    pub note: String,
}

/// User-owned planning inputs. These are not campus-source facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserAcademicSnapshot {
    /// Courses completed before the planned term.
    pub completed_courses: Vec<String>,
    /// Minimum requested credits.
    pub min_credits: u16,
    /// Maximum requested credits.
    pub max_credits: u16,
    /// Per-course soft preference weights. Negative values are allowed.
    #[serde(default)]
    pub preference_weights: BTreeMap<String, i32>,
}

/// One curriculum requirement group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequirementGroup {
    /// Stable requirement id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Minimum credits selected from `eligible_courses`.
    pub min_credits: u16,
    /// Courses that may cover this requirement.
    pub eligible_courses: Vec<String>,
    /// Source containing this requirement fact.
    pub source_id: String,
}

/// Identity status for a normalized course code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityStatus {
    /// Course code and revision are resolved.
    Verified,
    /// Cross-source alias conflict is unresolved; the planner must not guess.
    UnresolvedAlias,
}

/// One course offering fact from one source revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CourseOffering {
    /// Normalized course code.
    pub code: String,
    /// Course title supplied by the source.
    pub title: String,
    /// Integer credit units used by the initial fixture contract.
    pub credits: u16,
    /// Prerequisites that must already appear in `completed_courses`.
    pub prerequisites: Vec<String>,
    /// Weekly meeting intervals. An explicit empty list means asynchronous/no fixed slot.
    pub slots: Vec<TimeSlot>,
    /// Source containing this offering fact.
    pub source_id: String,
    /// Whether the course is available in the fixture's planned term.
    pub available: bool,
    /// Whether cross-source identity resolution is complete.
    pub identity_status: IdentityStatus,
}

/// A half-open weekly meeting interval `[start_minute, end_minute)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeSlot {
    /// ISO-like weekday number in `1..=7`.
    pub weekday: u8,
    /// Minutes after midnight.
    pub start_minute: u16,
    /// Minutes after midnight.
    pub end_minute: u16,
}

impl TimeSlot {
    fn conflicts(self, other: Self) -> bool {
        self.weekday == other.weekday
            && self.start_minute < other.end_minute
            && other.start_minute < self.end_minute
    }
}

/// Subjective link-out signal. It can rank candidates but cannot alter hard facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommunitySignal {
    /// Course receiving the signal.
    pub course_code: String,
    /// Community or mirror source id.
    pub source_id: String,
    /// Bounded score in `0..=100`.
    pub score: u16,
    /// Link-out URL; content is not embedded in the fixture.
    pub link: String,
}

/// Search bounds for deterministic beam planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanningConfig {
    /// Maximum number of returned candidates.
    pub max_results: usize,
    /// Maximum number of partial states retained per course.
    pub beam_width: usize,
}

impl Default for PlanningConfig {
    fn default() -> Self {
        Self {
            max_results: 3,
            beam_width: 1_024,
        }
    }
}

/// Planner output suitable for CLI JSON serialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanResult {
    /// Output schema version.
    pub schema_version: String,
    /// Input fixture revision.
    pub source_revision: String,
    /// Ranked feasible candidates.
    pub candidates: Vec<PlanCandidate>,
    /// Sum of candidate hard-constraint violations; successful output must be zero.
    pub hard_constraint_violations: usize,
    /// Deterministic conflict, stale-source, and eligibility warnings.
    pub warnings: Vec<String>,
}

/// One feasible course plan candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanCandidate {
    /// Selected normalized course codes.
    pub course_codes: Vec<String>,
    /// Total selected credits.
    pub total_credits: u16,
    /// Covered credits by requirement id.
    pub requirement_credits: BTreeMap<String, u16>,
    /// Soft preference/community score. It never changes hard feasibility.
    pub soft_score: i64,
    /// Recomputed hard-constraint violations; successful candidates have none.
    pub hard_constraint_violations: Vec<String>,
    /// Concise deterministic rationale.
    pub rationale: Vec<String>,
    /// Fact-level provenance for selected offerings and requirements.
    pub provenance: Vec<FactProvenance>,
}

/// Conflict state recorded for a material output fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictStatus {
    /// No competing fact was present in the fixture.
    NoKnownConflict,
    /// A higher-authority fact deterministically displaced lower-authority records.
    ResolvedByAuthority,
    /// Multiple highest-authority sources supplied equivalent facts.
    EquivalentSources,
}

/// Evidence attached to one material output fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactProvenance {
    /// Material fact label, for example `course:MATH2001`.
    pub fact: String,
    /// Stable source id.
    pub source_id: String,
    /// Source-local revision.
    pub revision: String,
    /// Source authority.
    pub authority: SourceAuthority,
    /// Retrieval/import timestamp from the fixture.
    pub retrieved_at: String,
    /// Term/date range in which the fact applies, when available.
    pub effective_time: Option<String>,
    /// How competing source records were handled.
    pub conflict_status: ConflictStatus,
}

/// Errors returned before a trustworthy plan can be produced.
#[derive(Debug)]
pub enum PlanningError {
    /// Fixture file could not be read.
    ReadFixture { path: PathBuf, source: io::Error },
    /// Fixture JSON could not be decoded.
    DecodeFixture {
        path: PathBuf,
        source: serde_json::Error,
    },
    /// Fixture violates one or more fail-closed contract checks.
    InvalidFixture { problems: Vec<String> },
    /// No plan satisfies all hard constraints within the bounded search.
    NoFeasiblePlan,
}

impl fmt::Display for PlanningError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadFixture { path, source } => {
                write!(
                    formatter,
                    "failed to read fixture {}: {source}",
                    path.display()
                )
            }
            Self::DecodeFixture { path, source } => {
                write!(
                    formatter,
                    "failed to decode fixture {}: {source}",
                    path.display()
                )
            }
            Self::InvalidFixture { problems } => {
                write!(
                    formatter,
                    "invalid Course Planning fixture: {}",
                    problems.join("; ")
                )
            }
            Self::NoFeasiblePlan => write!(formatter, "no feasible plan within bounded search"),
        }
    }
}

impl Error for PlanningError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadFixture { source, .. } => Some(source),
            Self::DecodeFixture { source, .. } => Some(source),
            Self::InvalidFixture { .. } | Self::NoFeasiblePlan => None,
        }
    }
}

/// Reads and validates a fixture from disk.
pub fn load_fixture(path: impl AsRef<Path>) -> Result<CoursePlanningFixture, PlanningError> {
    let path = path.as_ref();
    let text = fs::read_to_string(path).map_err(|source| PlanningError::ReadFixture {
        path: path.to_path_buf(),
        source,
    })?;
    let fixture = serde_json::from_str(&text).map_err(|source| PlanningError::DecodeFixture {
        path: path.to_path_buf(),
        source,
    })?;
    validate_fixture(&fixture)?;
    Ok(fixture)
}

/// Validates a fixture without running the planner.
pub fn validate_fixture(fixture: &CoursePlanningFixture) -> Result<(), PlanningError> {
    let mut problems = Vec::new();
    if fixture.schema_version != FIXTURE_SCHEMA_VERSION {
        problems.push(format!(
            "schema_version must be {FIXTURE_SCHEMA_VERSION}, got {}",
            fixture.schema_version
        ));
    }
    if fixture.source_revision.trim().is_empty() {
        problems.push("source_revision must not be empty".to_owned());
    }
    if fixture.profile.min_credits > fixture.profile.max_credits {
        problems.push("profile min_credits exceeds max_credits".to_owned());
    }
    if fixture.profile.max_credits == 0 {
        problems.push("profile max_credits must be positive".to_owned());
    }

    let mut sources = BTreeMap::new();
    for source in &fixture.sources {
        if source.id.trim().is_empty() {
            problems.push("source id must not be empty".to_owned());
        } else if sources.insert(source.id.as_str(), source).is_some() {
            problems.push(format!("duplicate source id: {}", source.id));
        }
        if source.revision.trim().is_empty() {
            problems.push(format!("source {} has empty revision", source.id));
        }
        if source.retrieved_at.trim().is_empty() {
            problems.push(format!("source {} has empty retrieved_at", source.id));
        } else if OffsetDateTime::parse(&source.retrieved_at, &Rfc3339).is_err() {
            problems.push(format!(
                "source {} has invalid RFC 3339 retrieved_at",
                source.id
            ));
        }
    }

    let mut requirement_ids = BTreeSet::new();
    for requirement in &fixture.requirements {
        if !requirement_ids.insert(requirement.id.as_str()) {
            problems.push(format!("duplicate requirement id: {}", requirement.id));
        }
        if requirement.min_credits == 0 {
            problems.push(format!(
                "requirement {} has zero min_credits",
                requirement.id
            ));
        }
        match sources.get(requirement.source_id.as_str()) {
            Some(source)
                if source.authority >= SourceAuthority::ReviewedOfficialSource && !source.stale => {
            }
            Some(source) if source.stale => problems.push(format!(
                "requirement {} uses stale source {}",
                requirement.id, requirement.source_id
            )),
            Some(_) => problems.push(format!(
                "requirement {} uses non-official source {}",
                requirement.id, requirement.source_id
            )),
            None => problems.push(format!(
                "requirement {} references unknown source {}",
                requirement.id, requirement.source_id
            )),
        }
    }

    for course in &fixture.courses {
        if course.code.trim().is_empty() {
            problems.push("course code must not be empty".to_owned());
        }
        if course.credits == 0 {
            problems.push(format!("course {} has zero credits", course.code));
        }
        match sources.get(course.source_id.as_str()) {
            Some(source) if source.authority > SourceAuthority::CommunitySignal => {}
            Some(_) => problems.push(format!(
                "course {} uses non-authoritative source {}",
                course.code, course.source_id
            )),
            None => problems.push(format!(
                "course {} references unknown source {}",
                course.code, course.source_id
            )),
        }
        for slot in &course.slots {
            if !(1..=7).contains(&slot.weekday)
                || slot.start_minute >= slot.end_minute
                || slot.end_minute > 1_440
            {
                problems.push(format!("course {} has invalid meeting slot", course.code));
            }
        }
    }

    let course_codes: BTreeSet<&str> = fixture
        .courses
        .iter()
        .map(|course| course.code.as_str())
        .collect();
    for requirement in &fixture.requirements {
        for code in &requirement.eligible_courses {
            if !course_codes.contains(code.as_str()) {
                problems.push(format!(
                    "requirement {} references unknown course {}",
                    requirement.id, code
                ));
            }
        }
    }
    for code in fixture.profile.preference_weights.keys() {
        if !course_codes.contains(code.as_str()) {
            problems.push(format!("preference references unknown course {code}"));
        }
    }

    for signal in &fixture.community_signals {
        if !course_codes.contains(signal.course_code.as_str()) {
            problems.push(format!(
                "community signal references unknown course {}",
                signal.course_code
            ));
        }
        if signal.score > 100 {
            problems.push(format!(
                "community signal for {} exceeds 100",
                signal.course_code
            ));
        }
        if !signal.link.starts_with("https://icourse.club/") {
            problems.push(format!(
                "community signal for {} is not an iCourse link-out",
                signal.course_code
            ));
        }
        match sources.get(signal.source_id.as_str()) {
            Some(source)
                if matches!(
                    source.authority,
                    SourceAuthority::CommunitySignal | SourceAuthority::ICourseMirror
                ) => {}
            Some(_) => problems.push(format!(
                "community signal for {} uses non-community source {}",
                signal.course_code, signal.source_id
            )),
            None => problems.push(format!(
                "community signal for {} references unknown source {}",
                signal.course_code, signal.source_id
            )),
        }
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(PlanningError::InvalidFixture { problems })
    }
}

/// Produces deterministic feasible candidates from a validated fixture.
pub fn plan_fixture(
    fixture: &CoursePlanningFixture,
    config: PlanningConfig,
) -> Result<PlanResult, PlanningError> {
    validate_fixture(fixture)?;
    if config.max_results == 0 || config.beam_width == 0 {
        return Err(PlanningError::InvalidFixture {
            problems: vec!["planning bounds must be positive".to_owned()],
        });
    }

    let sources: BTreeMap<&str, &SourceDescriptor> = fixture
        .sources
        .iter()
        .map(|source| (source.id.as_str(), source))
        .collect();
    let (mut courses, mut warnings) = resolve_offerings(fixture, &sources);
    let completed: BTreeSet<&str> = fixture
        .profile
        .completed_courses
        .iter()
        .map(String::as_str)
        .collect();

    courses.retain(|resolved| {
        let course = &resolved.offering;
        if resolved.source.stale {
            warnings.push(format!(
                "course {} excluded because source {} is stale",
                course.code, resolved.source.id
            ));
            return false;
        }
        if !course.available {
            warnings.push(format!(
                "course {} excluded because it is unavailable",
                course.code
            ));
            return false;
        }
        if course.identity_status == IdentityStatus::UnresolvedAlias {
            warnings.push(format!(
                "course {} excluded because its alias identity is unresolved",
                course.code
            ));
            return false;
        }
        let missing: Vec<&str> = course
            .prerequisites
            .iter()
            .map(String::as_str)
            .filter(|code| !completed.contains(code))
            .collect();
        if !missing.is_empty() {
            warnings.push(format!(
                "course {} excluded because prerequisites are incomplete: {}",
                course.code,
                missing.join(",")
            ));
            return false;
        }
        true
    });
    courses.sort_by(|left, right| left.offering.code.cmp(&right.offering.code));

    let (signal_scores, signal_warnings) = community_scores(fixture, &sources);
    warnings.extend(signal_warnings);
    let mut states = vec![SearchState::default()];
    for (index, resolved) in courses.iter().enumerate() {
        let mut expanded = Vec::with_capacity(states.len().saturating_mul(2));
        for state in &states {
            expanded.push(state.clone());
            if let Some(included) = state.with_course(
                index,
                resolved,
                fixture,
                signal_scores.get(resolved.offering.code.as_str()).copied(),
            ) {
                expanded.push(included);
            }
        }
        expanded.sort_by(|left, right| {
            right
                .coverage_priority(&fixture.requirements)
                .cmp(&left.coverage_priority(&fixture.requirements))
                .then_with(|| right.soft_score.cmp(&left.soft_score))
                .then_with(|| right.credits.cmp(&left.credits))
                .then_with(|| left.selected.cmp(&right.selected))
        });
        expanded.dedup_by(|left, right| left.selected == right.selected);
        expanded.truncate(config.beam_width);
        states = expanded;
    }

    let mut candidates = Vec::new();
    for state in states {
        if state.credits < fixture.profile.min_credits
            || state.credits > fixture.profile.max_credits
            || !requirements_satisfied(&state.coverage, &fixture.requirements)
        {
            continue;
        }
        let violations = candidate_violations(&state, &courses, fixture);
        if !violations.is_empty() {
            continue;
        }
        candidates.push(build_candidate(
            &state, &courses, fixture, &sources, violations,
        ));
    }

    candidates.sort_by(|left, right| {
        right
            .soft_score
            .cmp(&left.soft_score)
            .then_with(|| right.total_credits.cmp(&left.total_credits))
            .then_with(|| left.course_codes.cmp(&right.course_codes))
    });
    candidates.dedup_by(|left, right| left.course_codes == right.course_codes);
    candidates.truncate(config.max_results);
    if candidates.is_empty() {
        return Err(PlanningError::NoFeasiblePlan);
    }
    let hard_constraint_violations = candidates
        .iter()
        .map(|candidate| candidate.hard_constraint_violations.len())
        .sum();
    warnings.sort();
    warnings.dedup();

    Ok(PlanResult {
        schema_version: "course-plan-result/v0".to_owned(),
        source_revision: fixture.source_revision.clone(),
        candidates,
        hard_constraint_violations,
        warnings,
    })
}

#[derive(Debug, Clone)]
struct ResolvedCourse<'a> {
    offering: &'a CourseOffering,
    source: &'a SourceDescriptor,
    conflict_status: ConflictStatus,
}

fn resolve_offerings<'a>(
    fixture: &'a CoursePlanningFixture,
    sources: &BTreeMap<&'a str, &'a SourceDescriptor>,
) -> (Vec<ResolvedCourse<'a>>, Vec<String>) {
    let mut grouped: BTreeMap<&str, Vec<ResolvedCourse<'a>>> = BTreeMap::new();
    let mut warnings = Vec::new();

    for offering in &fixture.courses {
        let Some(source) = sources.get(offering.source_id.as_str()).copied() else {
            continue;
        };
        grouped
            .entry(offering.code.as_str())
            .or_default()
            .push(ResolvedCourse {
                offering,
                source,
                conflict_status: ConflictStatus::NoKnownConflict,
            });
    }

    let mut resolved = Vec::new();
    for (code, mut records) in grouped {
        records.sort_by(|left, right| {
            right
                .source
                .authority
                .cmp(&left.source.authority)
                .then_with(|| left.source.id.cmp(&right.source.id))
                .then_with(|| left.source.revision.cmp(&right.source.revision))
        });
        let Some(canonical) = records.first().cloned() else {
            continue;
        };
        let highest_authority = canonical.source.authority;
        let highest: Vec<&ResolvedCourse<'_>> = records
            .iter()
            .filter(|record| record.source.authority == highest_authority)
            .collect();
        if highest
            .iter()
            .any(|record| !same_material_fact(canonical.offering, record.offering))
        {
            let source_ids: Vec<&str> = highest
                .iter()
                .map(|record| record.source.id.as_str())
                .collect();
            warnings.push(format!(
                "course {code} excluded because highest-authority sources conflict: {}",
                source_ids.join(",")
            ));
            continue;
        }

        let lower: Vec<&ResolvedCourse<'_>> = records
            .iter()
            .filter(|record| record.source.authority < highest_authority)
            .collect();
        for record in &lower {
            warnings.push(format!(
                "course {code}: lower-authority source {} ignored in favor of {}",
                record.source.id, canonical.source.id
            ));
        }
        if highest.len() > 1 {
            warnings.push(format!(
                "course {code}: equivalent highest-authority facts resolved to source {}",
                canonical.source.id
            ));
        }

        let conflict_status = if lower.is_empty() {
            if highest.len() > 1 {
                ConflictStatus::EquivalentSources
            } else {
                ConflictStatus::NoKnownConflict
            }
        } else {
            ConflictStatus::ResolvedByAuthority
        };
        resolved.push(ResolvedCourse {
            conflict_status,
            ..canonical
        });
    }

    (resolved, warnings)
}

fn same_material_fact(left: &CourseOffering, right: &CourseOffering) -> bool {
    left.code == right.code
        && left.title == right.title
        && left.credits == right.credits
        && left.prerequisites == right.prerequisites
        && left.slots == right.slots
        && left.available == right.available
        && left.identity_status == right.identity_status
}

fn community_scores(
    fixture: &CoursePlanningFixture,
    sources: &BTreeMap<&str, &SourceDescriptor>,
) -> (BTreeMap<String, i64>, Vec<String>) {
    let mut scores: BTreeMap<String, Vec<u16>> = BTreeMap::new();
    let mut warnings = Vec::new();
    for signal in &fixture.community_signals {
        let Some(source) = sources.get(signal.source_id.as_str()) else {
            continue;
        };
        if source.stale {
            warnings.push(format!(
                "community signal for {} ignored because source {} is stale",
                signal.course_code, source.id
            ));
            continue;
        }
        scores
            .entry(signal.course_code.clone())
            .or_default()
            .push(signal.score);
    }
    let scores = scores
        .into_iter()
        .map(|(code, values)| {
            let sum: u64 = values.iter().map(|value| u64::from(*value)).sum();
            let average = sum / u64::try_from(values.len()).unwrap_or(1);
            let centered = i64::try_from(average).unwrap_or(50) - 50;
            (code, centered / 5)
        })
        .collect();
    (scores, warnings)
}

#[derive(Debug, Clone, Default)]
struct SearchState {
    selected: Vec<usize>,
    credits: u16,
    slots: Vec<TimeSlot>,
    coverage: BTreeMap<String, u16>,
    soft_score: i64,
}

impl SearchState {
    fn with_course(
        &self,
        index: usize,
        resolved: &ResolvedCourse<'_>,
        fixture: &CoursePlanningFixture,
        community_score: Option<i64>,
    ) -> Option<Self> {
        let course = resolved.offering;
        let credits = self.credits.checked_add(course.credits)?;
        if credits > fixture.profile.max_credits
            || course
                .slots
                .iter()
                .any(|slot| self.slots.iter().any(|existing| slot.conflicts(*existing)))
        {
            return None;
        }

        let mut next = self.clone();
        next.selected.push(index);
        next.credits = credits;
        next.slots.extend(course.slots.iter().copied());
        for requirement in &fixture.requirements {
            if requirement
                .eligible_courses
                .iter()
                .any(|code| code == &course.code)
            {
                *next.coverage.entry(requirement.id.clone()).or_default() += course.credits;
            }
        }
        next.soft_score += i64::from(
            fixture
                .profile
                .preference_weights
                .get(&course.code)
                .copied()
                .unwrap_or_default(),
        );
        next.soft_score += community_score.unwrap_or_default();
        Some(next)
    }

    fn coverage_priority(&self, requirements: &[RequirementGroup]) -> u64 {
        requirements
            .iter()
            .map(|requirement| {
                u64::from(
                    self.coverage
                        .get(&requirement.id)
                        .copied()
                        .unwrap_or_default()
                        .min(requirement.min_credits),
                )
            })
            .sum()
    }
}

fn requirements_satisfied(
    coverage: &BTreeMap<String, u16>,
    requirements: &[RequirementGroup],
) -> bool {
    requirements.iter().all(|requirement| {
        coverage.get(&requirement.id).copied().unwrap_or_default() >= requirement.min_credits
    })
}

fn candidate_violations(
    state: &SearchState,
    courses: &[ResolvedCourse<'_>],
    fixture: &CoursePlanningFixture,
) -> Vec<String> {
    let mut violations = Vec::new();
    let completed: BTreeSet<&str> = fixture
        .profile
        .completed_courses
        .iter()
        .map(String::as_str)
        .collect();
    let selected: Vec<&CourseOffering> = state
        .selected
        .iter()
        .filter_map(|index| courses.get(*index).map(|resolved| resolved.offering))
        .collect();
    let credits: u16 = selected.iter().map(|course| course.credits).sum();
    if credits != state.credits {
        violations.push("planner state credit total drifted from selected courses".to_owned());
    }
    if credits < fixture.profile.min_credits || credits > fixture.profile.max_credits {
        violations.push(format!("total credits {credits} outside requested bounds"));
    }
    for course in &selected {
        let missing: Vec<&str> = course
            .prerequisites
            .iter()
            .map(String::as_str)
            .filter(|code| !completed.contains(code))
            .collect();
        if !missing.is_empty() {
            violations.push(format!(
                "course {} has incomplete prerequisites: {}",
                course.code,
                missing.join(",")
            ));
        }
    }
    for left in 0..selected.len() {
        for right in (left + 1)..selected.len() {
            if selected[left].slots.iter().any(|left_slot| {
                selected[right]
                    .slots
                    .iter()
                    .any(|right_slot| left_slot.conflicts(*right_slot))
            }) {
                violations.push(format!(
                    "courses {} and {} have a time conflict",
                    selected[left].code, selected[right].code
                ));
            }
        }
    }
    let mut coverage = BTreeMap::new();
    for course in &selected {
        for requirement in &fixture.requirements {
            if requirement
                .eligible_courses
                .iter()
                .any(|code| code == &course.code)
            {
                *coverage.entry(requirement.id.clone()).or_default() += course.credits;
            }
        }
    }
    if coverage != state.coverage {
        violations
            .push("planner state requirement coverage drifted from selected courses".to_owned());
    }
    if !requirements_satisfied(&coverage, &fixture.requirements) {
        violations.push("one or more requirement groups are under-covered".to_owned());
    }
    violations
}

fn build_candidate(
    state: &SearchState,
    courses: &[ResolvedCourse<'_>],
    fixture: &CoursePlanningFixture,
    sources: &BTreeMap<&str, &SourceDescriptor>,
    hard_constraint_violations: Vec<String>,
) -> PlanCandidate {
    let selected: Vec<&ResolvedCourse<'_>> = state
        .selected
        .iter()
        .filter_map(|index| courses.get(*index))
        .collect();
    let course_codes: Vec<String> = selected
        .iter()
        .map(|resolved| resolved.offering.code.clone())
        .collect();
    let mut provenance = Vec::new();
    for resolved in &selected {
        provenance.push(FactProvenance {
            fact: format!("course:{}", resolved.offering.code),
            source_id: resolved.source.id.clone(),
            revision: resolved.source.revision.clone(),
            authority: resolved.source.authority,
            retrieved_at: resolved.source.retrieved_at.clone(),
            effective_time: resolved.source.effective_time.clone(),
            conflict_status: resolved.conflict_status,
        });
    }
    for requirement in &fixture.requirements {
        if let Some(source) = sources.get(requirement.source_id.as_str()) {
            provenance.push(FactProvenance {
                fact: format!("requirement:{}", requirement.id),
                source_id: source.id.clone(),
                revision: source.revision.clone(),
                authority: source.authority,
                retrieved_at: source.retrieved_at.clone(),
                effective_time: source.effective_time.clone(),
                conflict_status: ConflictStatus::NoKnownConflict,
            });
        }
    }
    let selected_codes: BTreeSet<&str> = selected
        .iter()
        .map(|resolved| resolved.offering.code.as_str())
        .collect();
    for signal in &fixture.community_signals {
        if !selected_codes.contains(signal.course_code.as_str()) {
            continue;
        }
        if let Some(source) = sources.get(signal.source_id.as_str()) {
            if source.stale {
                continue;
            }
            provenance.push(FactProvenance {
                fact: format!(
                    "community-signal:{}:{}",
                    signal.course_code, signal.source_id
                ),
                source_id: source.id.clone(),
                revision: source.revision.clone(),
                authority: source.authority,
                retrieved_at: source.retrieved_at.clone(),
                effective_time: source.effective_time.clone(),
                conflict_status: ConflictStatus::NoKnownConflict,
            });
        }
    }
    provenance.sort_by(|left, right| left.fact.cmp(&right.fact));
    provenance.dedup_by(|left, right| left.fact == right.fact);

    let rationale = vec![
        format!(
            "selected {} courses within {}..={} credits",
            course_codes.len(),
            fixture.profile.min_credits,
            fixture.profile.max_credits
        ),
        "all requirement groups meet their minimum credit coverage".to_owned(),
        "non-stale community signals affect soft ranking only".to_owned(),
    ];

    PlanCandidate {
        course_codes,
        total_credits: state.credits,
        requirement_credits: state.coverage.clone(),
        soft_score: state.soft_score,
        hard_constraint_violations,
        rationale,
        provenance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../../market/fixtures/course-planning/minimal-v0.json");

    fn fixture() -> CoursePlanningFixture {
        let decoded = serde_json::from_str(FIXTURE);
        let Ok(fixture) = decoded else {
            panic!("minimal-v0 fixture must decode");
        };
        fixture
    }

    #[test]
    fn minimal_fixture_produces_multiple_zero_violation_candidates() {
        let fixture = fixture();
        let result = plan_fixture(&fixture, PlanningConfig::default());
        let Ok(result) = result else {
            panic!("minimal-v0 fixture must produce a plan");
        };
        assert!(result.candidates.len() >= 2);
        assert_eq!(result.hard_constraint_violations, 0);
        assert!(result.candidates.iter().all(|candidate| {
            candidate.hard_constraint_violations.is_empty()
                && candidate.total_credits >= fixture.profile.min_credits
                && candidate.total_credits <= fixture.profile.max_credits
        }));
        assert!(result.candidates.iter().all(|candidate| {
            !candidate
                .course_codes
                .iter()
                .any(|code| code == "MATH2006" || code == "CS2002" || code == "CS2004")
        }));
    }

    #[test]
    fn unknown_constraint_fields_fail_closed() {
        let prerequisite_typo = FIXTURE.replacen(
            "\"prerequisites\": [\"MATH1001\"]",
            "\"prerequisities\": [\"UNCOMPLETED999\"]",
            1,
        );
        let meeting_typo = FIXTURE.replacen("\"slots\": [", "\"meeting_slots\": [", 1);
        let missing_prerequisites =
            FIXTURE.replacen("      \"prerequisites\": [\"MATH1001\"],\n", "", 1);
        let missing_slots = FIXTURE.replacen(
            "      \"slots\": [{\"weekday\": 1, \"start_minute\": 480, \"end_minute\": 570}],\n",
            "",
            1,
        );
        assert!(serde_json::from_str::<CoursePlanningFixture>(&prerequisite_typo).is_err());
        assert!(serde_json::from_str::<CoursePlanningFixture>(&meeting_typo).is_err());
        assert!(serde_json::from_str::<CoursePlanningFixture>(&missing_prerequisites).is_err());
        assert!(serde_json::from_str::<CoursePlanningFixture>(&missing_slots).is_err());
    }

    #[test]
    fn official_source_wins_over_lower_authority_duplicate() {
        let fixture = fixture();
        let sources: BTreeMap<&str, &SourceDescriptor> = fixture
            .sources
            .iter()
            .map(|source| (source.id.as_str(), source))
            .collect();
        let (courses, warnings) = resolve_offerings(&fixture, &sources);
        let resolved = courses
            .iter()
            .find(|course| course.offering.code == "MATH2001");
        let Some(resolved) = resolved else {
            panic!("MATH2001 must resolve");
        };
        assert_eq!(resolved.source.id, "official-catalog-synthetic");
        assert_eq!(resolved.offering.credits, 4);
        assert!(warnings.iter().any(|warning| {
            warning.contains("MATH2001") && warning.contains("lower-authority")
        }));
    }

    #[test]
    fn lower_authority_conflicts_cannot_hide_later_official_fact() {
        let mut fixture = fixture();
        let official = fixture
            .courses
            .iter()
            .find(|course| {
                course.code == "MATH2001" && course.source_id == "official-catalog-synthetic"
            })
            .cloned();
        let mirror = fixture
            .courses
            .iter()
            .find(|course| {
                course.code == "MATH2001" && course.source_id == "icourse-mirror-synthetic"
            })
            .cloned();
        let (Some(official), Some(mirror)) = (official, mirror) else {
            panic!("canonical MATH2001 facts must exist");
        };
        let mut conflicting_mirror = mirror.clone();
        conflicting_mirror.credits = mirror.credits + 1;
        fixture.courses.retain(|course| course.code != "MATH2001");
        fixture.courses.insert(0, official);
        fixture.courses.insert(0, conflicting_mirror);
        fixture.courses.insert(0, mirror);

        let sources: BTreeMap<&str, &SourceDescriptor> = fixture
            .sources
            .iter()
            .map(|source| (source.id.as_str(), source))
            .collect();
        let (courses, _) = resolve_offerings(&fixture, &sources);
        let resolved = courses
            .iter()
            .find(|course| course.offering.code == "MATH2001");
        let Some(resolved) = resolved else {
            panic!("official MATH2001 must survive lower-authority conflicts");
        };
        assert_eq!(resolved.source.id, "official-catalog-synthetic");
        assert_eq!(resolved.offering.credits, 4);
        assert_eq!(
            resolved.conflict_status,
            ConflictStatus::ResolvedByAuthority
        );
    }

    #[test]
    fn unresolved_alias_is_excluded_instead_of_guessed() {
        let fixture = fixture();
        let result = plan_fixture(&fixture, PlanningConfig::default());
        let Ok(result) = result else {
            panic!("canonical fixture must plan");
        };
        assert!(
            result
                .candidates
                .iter()
                .all(|candidate| !candidate.course_codes.iter().any(|code| code == "PE2001"))
        );
        assert!(
            result
                .warnings
                .iter()
                .any(|warning| warning.contains("PE2001") && warning.contains("unresolved"))
        );
    }

    #[test]
    fn planning_is_deterministic() {
        let fixture = fixture();
        let first = plan_fixture(&fixture, PlanningConfig::default());
        let second = plan_fixture(&fixture, PlanningConfig::default());
        let (Ok(first), Ok(second)) = (first, second) else {
            panic!("minimal-v0 fixture must plan twice");
        };
        assert_eq!(first, second);

        let mut reordered = fixture;
        reordered.sources.reverse();
        reordered.courses.reverse();
        reordered.community_signals.reverse();
        let reordered = plan_fixture(&reordered, PlanningConfig::default());
        let Ok(reordered) = reordered else {
            panic!("reordered fact set must remain feasible");
        };
        assert_eq!(first, reordered);
    }

    #[test]
    fn material_output_contains_course_requirement_and_community_provenance() {
        let fixture = fixture();
        let result = plan_fixture(&fixture, PlanningConfig::default());
        let Ok(result) = result else {
            panic!("minimal-v0 fixture must produce a plan");
        };
        let Some(candidate) = result.candidates.first() else {
            panic!("planner must return a candidate");
        };
        assert!(candidate.provenance.iter().all(|evidence| {
            !evidence.source_id.is_empty()
                && !evidence.revision.is_empty()
                && !evidence.retrieved_at.is_empty()
                && evidence.effective_time.as_deref() == Some("2026-fall")
        }));
        assert!(
            candidate
                .provenance
                .iter()
                .any(|evidence| evidence.fact.starts_with("course:"))
        );
        assert!(
            candidate
                .provenance
                .iter()
                .any(|evidence| evidence.fact.starts_with("requirement:"))
        );
        assert!(
            candidate
                .provenance
                .iter()
                .any(|evidence| evidence.fact.starts_with("community-signal:"))
        );
        let math_evidence = result
            .candidates
            .iter()
            .flat_map(|candidate| candidate.provenance.iter())
            .find(|evidence| evidence.fact == "course:MATH2001");
        let Some(math_evidence) = math_evidence else {
            panic!("at least one candidate must carry MATH2001 provenance");
        };
        assert_eq!(
            math_evidence.conflict_status,
            ConflictStatus::ResolvedByAuthority
        );
    }

    #[test]
    fn stale_missing_or_conflicting_offerings_fail_closed() {
        let mut stale = fixture();
        stale.sources.push(SourceDescriptor {
            id: "stale-department-synthetic".to_owned(),
            authority: SourceAuthority::ReviewedOfficialSource,
            revision: "synthetic-stale-v0".to_owned(),
            retrieved_at: "2026-07-22T00:00:00Z".to_owned(),
            effective_time: Some("2026-fall".to_owned()),
            stale: true,
            note: "Synthetic stale source for fail-closed testing.".to_owned(),
        });
        let course = stale
            .courses
            .iter_mut()
            .find(|course| course.code == "CS2005");
        let Some(course) = course else {
            panic!("CS2005 must exist");
        };
        course.source_id = "stale-department-synthetic".to_owned();
        let planned = plan_fixture(&stale, PlanningConfig::default());
        let Ok(planned) = planned else {
            panic!("other fixture courses must remain feasible");
        };
        assert!(
            planned
                .candidates
                .iter()
                .all(|candidate| !candidate.course_codes.iter().any(|code| code == "CS2005"))
        );
        assert!(
            planned
                .warnings
                .iter()
                .any(|warning| warning.contains("CS2005") && warning.contains("stale"))
        );

        let mut missing = fixture();
        let Some(course) = missing.courses.first_mut() else {
            panic!("fixture must contain a course");
        };
        course.source_id = "missing-source".to_owned();
        assert!(matches!(
            validate_fixture(&missing),
            Err(PlanningError::InvalidFixture { .. })
        ));

        let mut conflicting = fixture();
        let official = conflicting
            .courses
            .iter()
            .find(|course| {
                course.code == "MATH2001" && course.source_id == "official-catalog-synthetic"
            })
            .cloned();
        let Some(mut official_conflict) = official else {
            panic!("official MATH2001 must exist");
        };
        official_conflict.credits += 1;
        conflicting.courses.push(official_conflict);
        let sources: BTreeMap<&str, &SourceDescriptor> = conflicting
            .sources
            .iter()
            .map(|source| (source.id.as_str(), source))
            .collect();
        let (resolved, warnings) = resolve_offerings(&conflicting, &sources);
        assert!(
            resolved
                .iter()
                .all(|course| course.offering.code != "MATH2001")
        );
        assert!(warnings.iter().any(|warning| {
            warning.contains("MATH2001") && warning.contains("highest-authority sources conflict")
        }));
    }

    #[test]
    fn stale_requirement_source_fails_closed() {
        let mut fixture = fixture();
        let source = fixture
            .sources
            .iter_mut()
            .find(|source| source.id == "official-catalog-synthetic");
        let Some(source) = source else {
            panic!("official fixture source must exist");
        };
        source.stale = true;
        let result = validate_fixture(&fixture);
        assert!(matches!(result, Err(PlanningError::InvalidFixture { .. })));
    }

    #[test]
    fn community_source_cannot_author_course_facts() {
        let mut fixture = fixture();
        let course = fixture
            .courses
            .iter_mut()
            .find(|course| course.source_id == "official-catalog-synthetic");
        let Some(course) = course else {
            panic!("official fixture course must exist");
        };
        course.source_id = "icourse-linkout".to_owned();
        let result = validate_fixture(&fixture);
        assert!(matches!(result, Err(PlanningError::InvalidFixture { .. })));
    }

    #[test]
    fn community_source_cannot_author_requirements() {
        let mut fixture = fixture();
        let Some(requirement) = fixture.requirements.first_mut() else {
            panic!("fixture must contain a requirement");
        };
        requirement.source_id = "icourse-linkout".to_owned();
        let result = validate_fixture(&fixture);
        assert!(matches!(result, Err(PlanningError::InvalidFixture { .. })));
    }

    #[test]
    fn invalid_retrieval_timestamp_fails_closed() {
        let mut fixture = fixture();
        let Some(source) = fixture.sources.first_mut() else {
            panic!("fixture must contain a source");
        };
        source.retrieved_at = "not-a-time".to_owned();
        assert!(matches!(
            validate_fixture(&fixture),
            Err(PlanningError::InvalidFixture { .. })
        ));
    }

    #[test]
    fn large_preference_weights_do_not_overflow_soft_score() {
        let mut fixture = fixture();
        let codes: BTreeSet<String> = fixture
            .courses
            .iter()
            .map(|course| course.code.clone())
            .collect();
        fixture.profile.preference_weights =
            codes.into_iter().map(|code| (code, i32::MAX)).collect();
        let result = plan_fixture(&fixture, PlanningConfig::default());
        let Ok(result) = result else {
            panic!("large but valid preference weights must remain plannable");
        };
        let Some(candidate) = result.candidates.first() else {
            panic!("planner must return a candidate");
        };
        assert!(candidate.soft_score > i64::from(i32::MAX));
    }

    #[test]
    fn stale_community_signals_are_warned_and_excluded_from_provenance() {
        let mut fixture = fixture();
        let source = fixture
            .sources
            .iter_mut()
            .find(|source| source.id == "icourse-linkout");
        let Some(source) = source else {
            panic!("community fixture source must exist");
        };
        source.stale = true;
        let result = plan_fixture(&fixture, PlanningConfig::default());
        let Ok(result) = result else {
            panic!("stale soft signals must not block hard planning");
        };
        assert!(
            result.warnings.iter().any(|warning| {
                warning.contains("community signal") && warning.contains("stale")
            })
        );
        assert!(result.candidates.iter().all(|candidate| {
            candidate
                .provenance
                .iter()
                .all(|evidence| !evidence.fact.starts_with("community-signal:"))
        }));
    }

    #[test]
    fn invalid_credit_bounds_fail_closed() {
        let mut fixture = fixture();
        fixture.profile.min_credits = 13;
        fixture.profile.max_credits = 12;
        let result = validate_fixture(&fixture);
        assert!(matches!(result, Err(PlanningError::InvalidFixture { .. })));
    }
}
