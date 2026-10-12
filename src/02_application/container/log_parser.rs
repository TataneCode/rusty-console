use crate::application::container::dto::LogLineDto;
use chrono::{DateTime, Utc};

pub fn parse_log_lines(raw: &str) -> Vec<LogLineDto> {
    raw.lines()
        .filter(|line| !line.is_empty())
        .map(parse_log_line)
        .collect()
}

fn parse_log_line(line: &str) -> LogLineDto {
    match line.split_once(' ') {
        Some((timestamp, message)) => match DateTime::parse_from_rfc3339(timestamp) {
            Ok(parsed) => LogLineDto {
                timestamp: Some(parsed.with_timezone(&Utc)),
                message: message.to_string(),
            },
            Err(_) => LogLineDto {
                timestamp: None,
                message: line.to_string(),
            },
        },
        None => LogLineDto {
            timestamp: None,
            message: line.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use chrono::Timelike;

    #[test]
    fn test_parse_line_with_timestamp() {
        let lines = parse_log_lines("2026-10-26T10:30:00.123456789Z line1");
        assert_eq!(lines.len(), 1);
        let expected = Utc
            .with_ymd_and_hms(2026, 10, 26, 10, 30, 0)
            .unwrap()
            .with_nanosecond(123456789)
            .unwrap();
        assert_eq!(lines[0].timestamp, Some(expected));
        assert_eq!(lines[0].message, "line1");
    }

    #[test]
    fn test_parse_line_without_timestamp() {
        let lines = parse_log_lines("plain message");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].timestamp, None);
        assert_eq!(lines[0].message, "plain message");
    }

    #[test]
    fn test_parse_mixed_lines() {
        let raw = "2026-10-26T10:30:00Z first\nsecond\n2026-10-26T10:30:01Z third";
        let lines = parse_log_lines(raw);
        assert_eq!(lines.len(), 3);
        assert!(lines[0].timestamp.is_some());
        assert_eq!(lines[1].timestamp, None);
        assert_eq!(lines[1].message, "second");
        assert!(lines[2].timestamp.is_some());
        assert_eq!(lines[2].message, "third");
    }

    #[test]
    fn test_parse_skips_empty_lines() {
        let lines = parse_log_lines("\n\nline1\n\n");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].message, "line1");
    }

    #[test]
    fn test_parse_empty_input() {
        assert!(parse_log_lines("").is_empty());
    }

    #[test]
    fn test_parse_timestamp_with_offset() {
        let lines = parse_log_lines("2026-10-26T12:30:00+02:00 hello");
        assert_eq!(lines.len(), 1);
        let timestamp = lines[0].timestamp.unwrap();
        assert_eq!(
            timestamp,
            Utc.with_ymd_and_hms(2026, 10, 26, 10, 30, 0).unwrap()
        );
        assert_eq!(lines[0].message, "hello");
    }

    #[test]
    fn test_parse_message_containing_spaces() {
        let lines = parse_log_lines("2026-10-26T10:30:00Z a message with spaces");
        assert_eq!(lines[0].message, "a message with spaces");
    }
}
