use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct QualityAssessment {
    pub information_quality: f32,
    pub usability: f32,
    pub flags: Vec<String>,
    pub publishable: bool,
}

pub fn assess_quality(title: &str, body: &str) -> QualityAssessment {
    let trimmed = body.trim();
    let word_count = trimmed.split_whitespace().count();
    let non_empty_lines: Vec<&str> = trimmed
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    let unique_terms = unique_terms(trimmed);
    let has_structure = trimmed.contains("\n#")
        || trimmed.starts_with('#')
        || trimmed.contains("\n- ")
        || trimmed.contains("\n1. ")
        || trimmed.contains("\n|")
        || trimmed.contains("## ");
    let has_links =
        trimmed.contains("http://") || trimmed.contains("https://") || trimmed.contains("](");
    let sentence_count =
        trimmed.matches('.').count() + trimmed.matches('?').count() + trimmed.matches('!').count();

    let lower_title = title.trim().to_lowercase();
    let lower_body = trimmed.to_lowercase();
    let generic_title = matches!(
        lower_title.as_str(),
        "errors" | "error" | "notes" | "note" | "todo" | "tbd" | "misc" | "untitled"
    );
    let hard_placeholder = generic_title
        || lower_body == "# errors"
        || lower_body == "errors"
        || lower_body.contains("todo")
        || lower_body.contains("tbd")
        || lower_body.contains("placeholder");

    let mut info = 0.0f32;
    let mut usability = 0.0f32;
    let mut flags = Vec::new();

    info += match word_count {
        0..=24 => 0.02,
        25..=59 => 0.10,
        60..=119 => 0.18,
        120..=249 => 0.28,
        _ => 0.36,
    };
    info += match unique_terms.len() {
        0..=9 => 0.02,
        10..=19 => 0.08,
        20..=39 => 0.15,
        40..=59 => 0.22,
        _ => 0.28,
    };
    info += match non_empty_lines.len() {
        0..=2 => 0.02,
        3..=5 => 0.08,
        6..=9 => 0.12,
        _ => 0.16,
    };
    if has_structure {
        info += 0.08;
    }
    if has_links {
        info += 0.05;
    }

    usability += if title.trim().len() >= 8 { 0.16 } else { 0.04 };
    usability += match non_empty_lines.len() {
        0..=2 => 0.04,
        3..=5 => 0.10,
        6..=9 => 0.16,
        _ => 0.20,
    };
    usability += if has_structure { 0.28 } else { 0.05 };
    usability += match sentence_count {
        0..=1 => 0.04,
        2..=4 => 0.12,
        _ => 0.18,
    };
    usability += if has_links { 0.08 } else { 0.0 };

    if word_count < 40 {
        flags.push("low_word_count".to_string());
        info -= 0.10;
    }
    if non_empty_lines.len() < 3 {
        flags.push("thin_structure".to_string());
        usability -= 0.10;
    }
    if unique_terms.len() < 12 {
        flags.push("low_unique_term_count".to_string());
        info -= 0.10;
    }
    if generic_title {
        flags.push("generic_title".to_string());
        usability -= 0.20;
    }
    if hard_placeholder {
        flags.push("placeholder_pattern".to_string());
        info -= 0.35;
        usability -= 0.35;
    }

    let information_quality = info.clamp(0.0, 1.0);
    let usability = usability.clamp(0.0, 1.0);
    let publishable = !hard_placeholder && information_quality >= 0.45 && usability >= 0.45;

    QualityAssessment {
        information_quality,
        usability,
        flags,
        publishable,
    }
}

fn unique_terms(text: &str) -> HashSet<String> {
    text.split_whitespace()
        .map(|word| {
            word.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|word| word.len() >= 4)
        .collect()
}
