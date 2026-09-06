//! Bounded display-title policy; model prose never owns dates or conversation state.

pub(crate) mod provider;

const MAX_TOPIC_CHARS: usize = 24;

pub(crate) fn current_date() -> String {
    date_at(time::OffsetDateTime::now_utc())
}

fn date_at(instant: time::OffsetDateTime) -> String {
    let campus =
        instant.to_offset(time::UtcOffset::from_hms(8, 0, 0).expect("valid campus offset"));
    format!(
        "{:02}{:02}{:02}",
        campus.year().rem_euclid(100),
        u8::from(campus.month()),
        campus.day()
    )
}

pub(crate) fn valid_date(date: &str) -> bool {
    if date.len() != 6 || !date.bytes().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let year =
        2000 + i32::from(date.as_bytes()[0] - b'0') * 10 + i32::from(date.as_bytes()[1] - b'0');
    let month = (date.as_bytes()[2] - b'0') * 10 + date.as_bytes()[3] - b'0';
    let day = (date.as_bytes()[4] - b'0') * 10 + date.as_bytes()[5] - b'0';
    time::Month::try_from(month)
        .is_ok_and(|month| time::Date::from_calendar_date(year, month, day).is_ok())
}

fn forbidden(c: char) -> bool {
    c.is_control() || matches!(c, '|' | '\u{2028}' | '\u{2029}')
}

/// Model suggestions must already be a single short topic, not an arbitrary transcript.
pub(crate) fn normalize_topic(raw: &str) -> Option<String> {
    let topic = raw
        .trim()
        .trim_matches(['"', '\'', '“', '”', '「', '」'])
        .trim();
    if topic.is_empty() || topic.chars().count() > MAX_TOPIC_CHARS || topic.chars().any(forbidden) {
        return None;
    }
    Some(topic.to_owned())
}

/// Deterministic fallback accepts user prose, removes separators and bounds its display size.
pub(crate) fn dated_title(date: &str, topic: &str) -> String {
    let topic: String = topic
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .filter(|c| !forbidden(*c))
        .take(MAX_TOPIC_CHARS)
        .collect();
    let topic = topic.trim();
    format!("{date}|{}", if topic.is_empty() { "新对话" } else { topic })
}

pub(crate) fn valid_title(title: &str) -> bool {
    let Some((date, topic)) = title.split_once('|') else {
        return false;
    };
    valid_date(date)
        && !topic.is_empty()
        && topic == topic.trim()
        && topic.chars().count() <= MAX_TOPIC_CHARS
        && !topic.chars().any(forbidden)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversation_title_date_uses_campus_midnight_and_valid_calendar_dates() {
        // Build the exact boundary with the same existing time library, independent of host TZ.
        let midnight = time::Date::from_calendar_date(2026, time::Month::September, 20)
            .expect("date")
            .with_hms(16, 0, 0)
            .expect("time")
            .assume_utc();
        assert_eq!(date_at(midnight - time::Duration::seconds(1)), "260920");
        assert_eq!(date_at(midnight), "260921");
        for date in ["260921", "240229", "000229"] {
            assert!(valid_date(date));
        }
        for date in [
            "260229",
            "261301",
            "260931",
            "260900",
            "１２３４５６",
            "26092",
            "abcdef",
        ] {
            assert!(!valid_date(date));
        }
    }

    #[test]
    fn conversation_title_format_bounds_untrusted_topics() {
        assert_eq!(dated_title("260921", "启动日历"), "260921|启动日历");
        assert_eq!(
            dated_title("260921", "  日历|\n提醒\0  "),
            "260921|日历 提醒"
        );
        assert_eq!(dated_title("260921", "|\n\0"), "260921|新对话");
        assert_eq!(
            dated_title("260921", &"课".repeat(40)),
            format!("260921|{}", "课".repeat(24))
        );
        assert!(valid_title("260921|启动日历"));
        for title in [
            "260921|",
            "260931|日历",
            "260921|a|b",
            "260921|a\nb",
            "260921|日历 ",
        ] {
            assert!(!valid_title(title));
        }
        assert_eq!(normalize_topic(" “启动日历” "), Some("启动日历".into()));
        for topic in ["", "a\nb", "a|b", "a\u{2028}b"] {
            assert!(normalize_topic(topic).is_none());
        }
        assert!(normalize_topic(&"课".repeat(25)).is_none());
    }
}
