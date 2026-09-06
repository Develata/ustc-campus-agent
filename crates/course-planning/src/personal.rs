//! Request-local planning from explicitly supplied course evidence. This module
//! neither imports public campus facts nor persists a private profile.
use crate::TimeSlot;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use time::{OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersonalCourseRequest {
    pub schema: String,
    pub consent_this_request: bool,
    pub courses: Vec<SuppliedCourse>,
    pub completed_courses: Vec<String>,
    pub interests: Vec<String>,
    /// Empty means no availability restriction; supplied windows are half-open.
    pub free_slots: Vec<TimeSlot>,
    pub min_credits_tenths: u16,
    pub max_credits_tenths: u16,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuppliedCourse {
    pub code: String,
    pub title: String,
    pub credits_tenths: u16,
    pub prerequisites: Vec<String>,
    pub tags: Vec<String>,
    pub meetings: Vec<CourseMeeting>,
    pub source_url: String,
    pub source_excerpt: String,
    pub observed_at: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CourseMeeting {
    pub starts_at: String,
    pub duration_minutes: u16,
}
#[derive(Clone, Serialize)]
pub struct CalendarSuggestion {
    pub title: String,
    pub scheduled_for: String,
    pub source_url: String,
    pub course_code: String,
    pub duration_minutes: u16,
}
#[derive(Serialize)]
pub struct PersonalCoursePlan {
    pub schema: &'static str,
    pub authority: &'static str,
    pub candidates: Vec<PersonalCourseCandidate>,
    pub exclusions: Vec<String>,
    pub warnings: Vec<String>,
}
#[derive(Serialize)]
pub struct PersonalCourseCandidate {
    pub course_codes: Vec<String>,
    pub total_credits_tenths: u16,
    pub reasons: Vec<String>,
    pub sources: Vec<SuppliedCourse>,
    pub calendar_suggestions: Vec<CalendarSuggestion>,
}
#[derive(Clone)]
struct Choice {
    indices: Vec<usize>,
    credits: u16,
    interest_score: usize,
}

pub fn plan_personal(request: &PersonalCourseRequest) -> Result<PersonalCoursePlan, String> {
    validate(request)?;
    let interests: Vec<_> = request
        .interests
        .iter()
        .map(|s| s.trim().to_lowercase())
        .collect();
    let completed: BTreeSet<_> = request.completed_courses.iter().collect();
    let mut eligible = Vec::new();
    let mut exclusions = Vec::new();
    for (index, course) in request.courses.iter().enumerate() {
        let reason = if completed.contains(&course.code) {
            Some("已修课程")
        } else if course.prerequisites.iter().any(|p| !completed.contains(p)) {
            Some("先修课程尚未完成")
        } else if !request.free_slots.is_empty() && course.meetings.is_empty() {
            Some("缺少上课时间，无法确认是否符合空闲时间")
        } else if course
            .meetings
            .iter()
            .any(|meeting| !fits_availability(meeting, &request.free_slots))
        {
            Some("上课时间超出所填空闲时段")
        } else {
            None
        };
        if let Some(reason) = reason {
            exclusions.push(format!("{} {}：{}", course.code, course.title, reason));
        } else {
            eligible.push(index);
        }
    }
    let mut choices = vec![Choice {
        indices: Vec::new(),
        credits: 0,
        interest_score: 0,
    }];
    for index in eligible {
        let course = &request.courses[index];
        let matched = matches_interests(course, &interests);
        let additions: Vec<_> = choices
            .iter()
            .filter_map(|choice| {
                let credits = choice.credits.checked_add(course.credits_tenths)?;
                if credits > request.max_credits_tenths
                    || choice
                        .indices
                        .iter()
                        .any(|i| courses_conflict(course, &request.courses[*i]))
                {
                    return None;
                }
                let mut next = choice.clone();
                next.indices.push(index);
                next.credits = credits;
                next.interest_score += matched.len();
                Some(next)
            })
            .collect();
        choices.extend(additions);
        choices.sort_by(|a, b| {
            b.interest_score
                .cmp(&a.interest_score)
                .then(b.credits.cmp(&a.credits))
                .then(a.indices.cmp(&b.indices))
        });
        choices.truncate(512);
    }
    let candidates = choices
        .into_iter()
        .filter(|c| !c.indices.is_empty() && c.credits >= request.min_credits_tenths)
        .take(3)
        .map(|choice| {
            let mut reasons = Vec::new();
            let mut calendar_suggestions = Vec::new();
            let mut sources = Vec::new();
            for i in &choice.indices {
                let course = &request.courses[*i];
                let matched = matches_interests(course, &interests);
                reasons.push(format!(
                    "{} {}：{}；按输入资料核对先修条件及课程间冲突{}。",
                    course.code,
                    course.title,
                    if matched.is_empty() {
                        "无已匹配兴趣，按可行性与学分排序".into()
                    } else {
                        format!("匹配兴趣 {}", matched.join("、"))
                    },
                    if request.free_slots.is_empty() {
                        "；你未限定空闲时间"
                    } else {
                        "，全部已知课时位于空闲时段"
                    }
                ));
                for meeting in &course.meetings {
                    calendar_suggestions.push(CalendarSuggestion {
                        title: format!("{} {}", course.code, course.title),
                        scheduled_for: meeting.starts_at.clone(),
                        source_url: course.source_url.clone(),
                        course_code: course.code.clone(),
                        duration_minutes: meeting.duration_minutes,
                    });
                }
                sources.push(course.clone());
            }
            PersonalCourseCandidate {
                course_codes: choice
                    .indices
                    .iter()
                    .map(|i| request.courses[*i].code.clone())
                    .collect(),
                total_credits_tenths: choice.credits,
                reasons,
                sources,
                calendar_suggestions,
            }
        })
        .collect::<Vec<_>>();
    let mut warnings = vec!["仅按本次用户提供的课程原文与时间规划；资料尚未成为经学校核验的课程目录，不保证选课名额或培养要求达成。".into(), "未读取 iCourse 评分；兴趣匹配使用你输入的标题与标签，未将模型知识作为课程事实。".into(), "候选采用有界搜索；未找到候选不代表数学上不存在可行方案。加入日历还需预览并由你确认，不会自动选课。".into()];
    if request.courses.iter().any(|c| c.meetings.is_empty()) {
        warnings.push("部分课程没有明确日期；不会为其推测日期或生成日历事项。".into());
    }
    Ok(PersonalCoursePlan {
        schema: "personal-course-plan/v1",
        authority: "user_supplied_evidence_not_official_verification",
        candidates,
        exclusions,
        warnings,
    })
}
fn matches_interests(course: &SuppliedCourse, interests: &[String]) -> Vec<String> {
    let text = format!("{} {}", course.title, course.tags.join(" ")).to_lowercase();
    interests
        .iter()
        .filter(|i| text.contains(i.as_str()))
        .cloned()
        .collect()
}
fn meeting_range(meeting: &CourseMeeting) -> Option<(i128, i128)> {
    let start = OffsetDateTime::parse(&meeting.starts_at, &Rfc3339)
        .ok()?
        .unix_timestamp_nanos();
    Some((
        start,
        start.checked_add(i128::from(meeting.duration_minutes) * 60 * 1_000_000_000)?,
    ))
}
fn courses_conflict(a: &SuppliedCourse, b: &SuppliedCourse) -> bool {
    a.meetings.iter().any(|x| {
        b.meetings
            .iter()
            .any(|y| match (meeting_range(x), meeting_range(y)) {
                (Some((a, b)), Some((c, d))) => a < d && c < b,
                _ => true,
            })
    })
}
fn fits_availability(meeting: &CourseMeeting, slots: &[TimeSlot]) -> bool {
    if slots.is_empty() {
        return true;
    }
    let Ok(start) = OffsetDateTime::parse(&meeting.starts_at, &Rfc3339) else {
        return false;
    };
    let Ok(offset) = UtcOffset::from_hms(8, 0, 0) else {
        return false;
    };
    let start = start.to_offset(offset);
    let second =
        u64::from(start.hour()) * 3600 + u64::from(start.minute()) * 60 + u64::from(start.second());
    let start_nanos = second * 1_000_000_000 + u64::from(start.nanosecond());
    let end_nanos = start_nanos + u64::from(meeting.duration_minutes) * 60 * 1_000_000_000;
    slots.iter().any(|s| {
        s.weekday == start.weekday().number_from_monday()
            && u64::from(s.start_minute) * 60 * 1_000_000_000 <= start_nanos
            && end_nanos <= u64::from(s.end_minute) * 60 * 1_000_000_000
    })
}
fn validate(request: &PersonalCourseRequest) -> Result<(), String> {
    if request.schema != "personal-course-request/v1" || !request.consent_this_request {
        return Err("course_request_consent_required".into());
    }
    if request.courses.is_empty()
        || request.courses.len() > 64
        || request.completed_courses.len() > 256
        || request.interests.len() > 32
        || request.free_slots.len() > 64
        || request.max_credits_tenths == 0
        || request.max_credits_tenths > 1000
        || request.min_credits_tenths > request.max_credits_tenths
        || request
            .interests
            .iter()
            .any(|i| i.trim().is_empty() || i.len() > 128)
        || request
            .completed_courses
            .iter()
            .any(|s| s.is_empty() || s.len() > 64)
    {
        return Err("course_request_out_of_bounds".into());
    }
    if request.free_slots.iter().any(|s| {
        !(1..=7).contains(&s.weekday) || s.start_minute >= s.end_minute || s.end_minute > 1440
    }) {
        return Err("course_availability_invalid".into());
    }
    let mut seen = BTreeSet::new();
    let mut total_meetings = 0;
    for c in &request.courses {
        total_meetings += c.meetings.len();
        if !seen.insert(&c.code)
            || c.code.trim().is_empty()
            || c.code.len() > 64
            || c.title.trim().is_empty()
            || c.title.len() > 200
            || c.credits_tenths == 0
            || c.credits_tenths > 1000
            || c.tags.len() > 32
            || c.tags.iter().any(|t| t.len() > 128)
            || c.prerequisites.len() > 64
            || c.prerequisites.iter().any(|p| p.is_empty() || p.len() > 64)
            || c.meetings.len() > 128
            || total_meetings > 512
            || c.source_excerpt.len() > 8192
            || !c.source_excerpt.contains(&c.code)
            || !c.source_excerpt.contains(&c.title)
            || OffsetDateTime::parse(&c.observed_at, &Rfc3339).is_err()
        {
            return Err("course_evidence_invalid".into());
        }
        ustc_campus_agent_core::source_registry::SourceUrl::parse(c.source_url.clone())
            .map_err(|_| "course_source_url_invalid")?;
        if c.meetings.iter().any(|m| {
            m.duration_minutes == 0 || m.duration_minutes > 720 || meeting_range(m).is_none()
        }) {
            return Err("course_meeting_invalid".into());
        }
        for (i, a) in c.meetings.iter().enumerate() {
            for b in &c.meetings[i + 1..] {
                if let (Some((x, y)), Some((z, w))) = (meeting_range(a), meeting_range(b))
                    && x < w
                    && z < y
                {
                    return Err("course_meeting_overlap".into());
                }
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> PersonalCourseRequest {
        PersonalCourseRequest {
            schema: "personal-course-request/v1".into(),
            consent_this_request: true,
            completed_courses: vec![],
            interests: vec!["数学".into()],
            min_credits_tenths: 10,
            max_credits_tenths: 40,
            free_slots: vec![TimeSlot {
                weekday: 1,
                start_minute: 480,
                end_minute: 720,
            }],
            courses: vec![SuppliedCourse {
                code: "TEST1".into(),
                title: "数学测试课".into(),
                credits_tenths: 30,
                prerequisites: vec![],
                tags: vec![],
                meetings: vec![CourseMeeting {
                    starts_at: "2026-09-07T09:00:00+08:00".into(),
                    duration_minutes: 90,
                }],
                source_url: "https://www.ustc.edu.cn/".into(),
                source_excerpt: "Controlled fixture TEST1 数学测试课".into(),
                observed_at: "2026-09-06T08:00:00Z".into(),
            }],
        }
    }
    #[test]
    fn interest_availability_and_exact_calendar_evidence() {
        let r = request();
        let plan = plan_personal(&r).expect("plan");
        assert_eq!(plan.candidates.len(), 1);
        assert_eq!(
            plan.candidates[0].calendar_suggestions[0].scheduled_for,
            r.courses[0].meetings[0].starts_at
        );
        assert!(plan.candidates[0].reasons[0].contains("匹配兴趣 数学"));
        let mut blocked = r;
        blocked.free_slots[0].end_minute = 540;
        assert!(
            plan_personal(&blocked)
                .expect("filtered")
                .candidates
                .is_empty()
        );
    }
    #[test]
    fn consent_missing_evidence_and_prerequisites_fail_closed() {
        let mut r = request();
        r.consent_this_request = false;
        assert!(plan_personal(&r).is_err());
        r.consent_this_request = true;
        r.courses[0].source_excerpt.clear();
        assert!(plan_personal(&r).is_err());
        r = request();
        r.courses[0].prerequisites.push("PRE1".into());
        assert!(plan_personal(&r).expect("filtered").candidates.is_empty());
    }
    #[test]
    fn overlapping_courses_never_share_candidate() {
        let mut r = request();
        let mut other = r.courses[0].clone();
        other.code = "TEST2".into();
        other.source_excerpt = "TEST2 数学测试课".into();
        r.courses.push(other);
        r.max_credits_tenths = 100;
        let plan = plan_personal(&r).expect("plan");
        assert!(plan.candidates.iter().all(|c| c.course_codes.len() == 1));
    }
}

#[cfg(test)]
mod exact_time_tests {
    use super::*;
    #[test]
    fn partial_minute_cannot_escape_free_time_or_overlap_checks() {
        let slots = vec![TimeSlot {
            weekday: 1,
            start_minute: 480,
            end_minute: 540,
        }];
        assert!(!fits_availability(
            &CourseMeeting {
                starts_at: "2026-09-07T08:00:00.001+08:00".into(),
                duration_minutes: 60
            },
            &slots
        ));
        assert!(fits_availability(
            &CourseMeeting {
                starts_at: "2026-09-07T08:00:00+08:00".into(),
                duration_minutes: 60
            },
            &slots
        ));
    }
}
