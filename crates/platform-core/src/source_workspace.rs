//! M60 bounded local source observations. Never mints a published artifact or
//! advances the accepted baseline of the production source pipeline.
use crate::source_registry::{SourceId, SourceUrl};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedSource {
    pub source_id: String,
    pub title: String,
    pub url: String,
    pub reviewer: String,
    pub permission_evidence: String,
    pub review_evidence: String,
    pub minimum_interval_seconds: u64,
}
impl ReviewedSource {
    pub fn validate(&self) -> Result<(), String> {
        SourceId::parse(self.source_id.clone()).map_err(|_| "invalid_source_id")?;
        let url = SourceUrl::parse(self.url.clone()).map_err(|_| "invalid_source_url")?;
        let host = url
            .as_str()
            .strip_prefix("https://")
            .and_then(|s| s.split('/').next())
            .ok_or("invalid_source_url")?;
        if !(host == "ustc.edu.cn" || host.ends_with(".ustc.edu.cn")) || host.contains("icourse") {
            return Err("source_not_official_public_host".into());
        }
        if self.title.trim().is_empty()
            || self.title.len() > 256
            || self.reviewer.trim().is_empty()
            || self.permission_evidence.trim().is_empty()
            || self.review_evidence.trim().is_empty()
            || self.minimum_interval_seconds < 60
            || self.minimum_interval_seconds > 604800
            || [
                &self.reviewer,
                &self.permission_evidence,
                &self.review_evidence,
            ]
            .iter()
            .any(|v| v.len() > 2048)
        {
            return Err("source_review_evidence_required".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceObservation {
    pub source_id: String,
    pub revision_id: String,
    pub url: String,
    pub title: String,
    pub observed_at_unix: u64,
    pub origin: String,
    pub raw_sha256: String,
    pub text: String,
    pub last_modified: Option<String>,
    pub prior_revision_id: Option<String>,
    pub added_lines: Vec<String>,
    pub removed_lines: Vec<String>,
    pub review: Option<ObservationReview>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationReview {
    pub reviewer: String,
    pub evidence: String,
    pub reviewed_at_unix: u64,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceWorkspace {
    pub schema: String,
    observations: Vec<SourceObservation>,
    last_attempts: std::collections::BTreeMap<String, u64>,
}
#[derive(Serialize)]
pub struct SourceSearchResult {
    pub schema: &'static str,
    pub authority: &'static str,
    pub query: String,
    pub results: Vec<SourceObservation>,
    pub warning: &'static str,
}
impl SourceWorkspace {
    pub fn empty() -> Self {
        Self {
            schema: "source-workspace/v1".into(),
            ..Self::default()
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != "source-workspace/v1"
            || self.observations.len() > 256
            || self.last_attempts.len() > 64
        {
            return Err("invalid_source_workspace".into());
        }
        let mut ids = BTreeSet::new();
        let mut latest: std::collections::BTreeMap<&str, &SourceObservation> =
            std::collections::BTreeMap::new();
        for (source, time) in &self.last_attempts {
            SourceId::parse(source.clone()).map_err(|_| "invalid_source_attempt")?;
            if *time == 0 {
                return Err("invalid_source_attempt".into());
            }
        }
        for item in &self.observations {
            if !ids.insert(&item.revision_id)
                || item.text.trim().is_empty()
                || item.text.len() > 131072
                || item.raw_sha256.len() != 64
                || !item
                    .raw_sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                || item.title.trim().is_empty()
                || item.title.len() > 256
                || item.observed_at_unix == 0
                || !matches!(item.origin.as_str(), "https_fetch" | "operator_text_import")
                || item.last_modified.as_ref().is_some_and(|v| v.len() > 256)
            {
                return Err("invalid_source_observation".into());
            }
            SourceId::parse(item.source_id.clone()).map_err(|_| "invalid_source_observation")?;
            SourceUrl::parse(item.url.clone()).map_err(|_| "invalid_source_observation")?;
            let expected = format!(
                "observation:{:x}",
                Sha256::digest(
                    serde_json::to_vec(&(
                        &item.source_id,
                        &item.url,
                        item.observed_at_unix,
                        &item.raw_sha256,
                        &item.text
                    ))
                    .map_err(|_| "source_encode_failed")?
                )
            );
            if item.revision_id != expected {
                return Err("source_revision_binding_invalid".into());
            }
            let prior = latest.get(item.source_id.as_str());
            if item.prior_revision_id.as_deref() != prior.map(|p| p.revision_id.as_str())
                || prior.is_some_and(|p| p.observed_at_unix > item.observed_at_unix)
            {
                return Err("source_revision_chain_invalid".into());
            }
            let old: BTreeSet<&str> = prior.map(|p| p.text.lines().collect()).unwrap_or_default();
            let new: BTreeSet<&str> = item.text.lines().collect();
            if item.added_lines
                != new
                    .difference(&old)
                    .take(100)
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>()
                || item.removed_lines
                    != old
                        .difference(&new)
                        .take(100)
                        .map(|s| (*s).to_owned())
                        .collect::<Vec<_>>()
            {
                return Err("source_revision_diff_invalid".into());
            }
            if item.review.as_ref().is_some_and(|r| {
                r.reviewer.trim().is_empty()
                    || r.reviewer.len() > 256
                    || r.evidence.trim().is_empty()
                    || r.evidence.len() > 2048
                    || r.reviewed_at_unix < item.observed_at_unix
            }) {
                return Err("source_revision_review_invalid".into());
            }
            latest.insert(&item.source_id, item);
        }
        Ok(())
    }
    pub fn reserve_attempt(&mut self, source: &ReviewedSource, now: u64) -> Result<(), String> {
        source.validate()?;
        if self.observations.len() >= 256 {
            return Err("source_workspace_capacity".into());
        }
        if let Some(last) = self.last_attempts.get(&source.source_id)
            && (now < *last || now - last < source.minimum_interval_seconds)
        {
            return Err("source_rate_limited".into());
        }
        self.last_attempts.insert(source.source_id.clone(), now);
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub fn observe(
        &mut self,
        source: &ReviewedSource,
        now: u64,
        raw: &[u8],
        text: String,
        last_modified: Option<String>,
        origin: &str,
    ) -> Result<SourceObservation, String> {
        source.validate()?;
        if now == 0
            || raw.is_empty()
            || raw.len() > 1048576
            || text.trim().is_empty()
            || text.len() > 131072
            || !matches!(origin, "https_fetch" | "operator_text_import")
            || last_modified.as_ref().is_some_and(|v| v.len() > 256)
        {
            return Err("invalid_source_observation".into());
        }
        if self
            .observations
            .iter()
            .any(|o| o.source_id == source.source_id && o.url != source.url)
        {
            return Err("source_identity_url_rebinding_forbidden".into());
        }
        let raw_sha256 = format!("{:x}", Sha256::digest(raw));
        if let Some(prior) = self
            .observations
            .iter()
            .rev()
            .find(|o| o.source_id == source.source_id)
            && prior.raw_sha256 == raw_sha256
            && prior.text == text
            && prior.url == source.url
        {
            return Ok(prior.clone());
        }
        if self.observations.len() >= 256 {
            return Err("source_workspace_capacity".into());
        }
        let prior = self
            .observations
            .iter()
            .rev()
            .find(|o| o.source_id == source.source_id);
        if prior.is_some_and(|p| p.observed_at_unix > now) {
            return Err("source_clock_regression".into());
        }
        let old: BTreeSet<&str> = prior.map(|p| p.text.lines().collect()).unwrap_or_default();
        let new: BTreeSet<&str> = text.lines().collect();
        let revision_id = format!(
            "observation:{:x}",
            Sha256::digest(
                serde_json::to_vec(&(&source.source_id, &source.url, now, &raw_sha256, &text))
                    .map_err(|_| "source_encode_failed")?
            )
        );
        let value = SourceObservation {
            source_id: source.source_id.clone(),
            revision_id,
            url: source.url.clone(),
            title: source.title.clone(),
            observed_at_unix: now,
            origin: origin.into(),
            raw_sha256,
            added_lines: new
                .difference(&old)
                .take(100)
                .map(|s| (*s).to_owned())
                .collect(),
            removed_lines: old
                .difference(&new)
                .take(100)
                .map(|s| (*s).to_owned())
                .collect(),
            prior_revision_id: prior.map(|p| p.revision_id.clone()),
            text,
            last_modified,
            review: None,
        };
        self.observations.push(value.clone());
        Ok(value)
    }
    pub fn review(
        &mut self,
        revision: &str,
        reviewer: &str,
        evidence: &str,
        now: u64,
    ) -> Result<SourceObservation, String> {
        if reviewer.trim().is_empty()
            || evidence.trim().is_empty()
            || reviewer.len() > 256
            || evidence.len() > 2048
        {
            return Err("review_evidence_required".into());
        }
        let item = self
            .observations
            .iter_mut()
            .find(|o| o.revision_id == revision)
            .ok_or("source_revision_not_found")?;
        if let Some(review) = &item.review {
            if review.reviewer == reviewer && review.evidence == evidence {
                return Ok(item.clone());
            }
            return Err("source_review_conflict".into());
        }
        if now < item.observed_at_unix {
            return Err("source_clock_regression".into());
        }
        item.review = Some(ObservationReview {
            reviewer: reviewer.into(),
            evidence: evidence.into(),
            reviewed_at_unix: now,
        });
        Ok(item.clone())
    }
    pub fn search(
        &self,
        query: &str,
        source_id: Option<&str>,
    ) -> Result<SourceSearchResult, String> {
        self.search_allowed(query, source_id, None)
    }
    pub fn search_allowed(
        &self,
        query: &str,
        source_id: Option<&str>,
        allowed: Option<&[ReviewedSource]>,
    ) -> Result<SourceSearchResult, String> {
        if query.len() > 512 {
            return Err("source_query_too_long".into());
        }
        let terms: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
        let mut seen = BTreeSet::new();
        let results = self
            .observations
            .iter()
            .rev()
            .filter(|o| {
                allowed.is_none_or(|sources| {
                    sources
                        .iter()
                        .any(|s| s.source_id == o.source_id && s.url == o.url)
                })
            })
            .filter(|o| seen.insert(o.source_id.clone()))
            .filter(|o| source_id.is_none_or(|id| id == o.source_id))
            .filter(|o| {
                let haystack = format!("{} {}", o.title, o.text).to_lowercase();
                terms.iter().all(|t| haystack.contains(t))
            })
            .take(12)
            .cloned()
            .collect();
        Ok(SourceSearchResult {
            schema: "source-search/v1",
            authority: "m60_local_observation_not_published",
            query: query.into(),
            results,
            warning: "内容来自标注链接的本地观察；未审阅内容须回原文核对。获取时间不等于发布时间或生效时间；搜索未命中不代表学校无此规定。",
        })
    }
    pub fn history(&self, source_id: &str) -> Vec<SourceObservation> {
        self.observations
            .iter()
            .filter(|o| o.source_id == source_id)
            .rev()
            .cloned()
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn source() -> ReviewedSource {
        ReviewedSource {
            source_id: "test-official".into(),
            title: "Controlled source".into(),
            url: "https://www.ustc.edu.cn/".into(),
            reviewer: "controlled-reviewer".into(),
            permission_evidence: "controlled-permission".into(),
            review_evidence: "controlled-review".into(),
            minimum_interval_seconds: 60,
        }
    }
    #[test]
    fn immutable_observations_diff_review_and_rate_are_separate() {
        let mut workspace = SourceWorkspace::empty();
        let source = source();
        workspace.reserve_attempt(&source, 100).expect("reserve");
        assert!(workspace.reserve_attempt(&source, 120).is_err());
        let a = workspace
            .observe(
                &source,
                100,
                b"before",
                "line one\nold".into(),
                None,
                "operator_text_import",
            )
            .expect("first");
        let b = workspace
            .observe(
                &source,
                200,
                b"after",
                "line one\nnew".into(),
                None,
                "operator_text_import",
            )
            .expect("second");
        assert_eq!(b.prior_revision_id, Some(a.revision_id.clone()));
        assert_eq!(b.added_lines, vec!["new"]);
        assert_eq!(b.removed_lines, vec!["old"]);
        assert!(b.review.is_none());
        assert_eq!(
            workspace.search("old", None).expect("search").results.len(),
            0
        );
        workspace
            .review(&b.revision_id, "operator", "read-original", 201)
            .expect("review");
        assert!(workspace.history(&source.source_id)[0].review.is_some());
        let replay = workspace
            .observe(
                &source,
                300,
                b"after",
                "line one\nnew".into(),
                None,
                "operator_text_import",
            )
            .expect("replay");
        assert_eq!(replay.revision_id, b.revision_id);
        let restored: SourceWorkspace =
            serde_json::from_slice(&serde_json::to_vec(&workspace).expect("serialize"))
                .expect("restore");
        restored.validate().expect("validate");
    }
    #[test]
    fn missing_permission_and_unofficial_sources_fail_closed() {
        let mut s = source();
        s.permission_evidence.clear();
        assert!(s.validate().is_err());
        s = source();
        s.url = "https://icourse.club/".into();
        assert!(s.validate().is_err());
        s.url = "https://ustc.edu.cn.evil.test/".into();
        assert!(s.validate().is_err());
    }
}
