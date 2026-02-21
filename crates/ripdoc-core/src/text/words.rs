use crate::objects::{Char, Word};

/// Group characters into words based on spatial proximity.
///
/// Characters are grouped into the same word if:
/// - They are within `x_tolerance` of each other horizontally
/// - They are within `y_tolerance` of each other vertically
///
/// This matches pdfplumber's word grouping behavior.
pub fn group_chars_to_words(chars: &[Char], x_tolerance: f64, y_tolerance: f64) -> Vec<Word> {
    if chars.is_empty() {
        return vec![];
    }

    // Sort chars by position: top-to-bottom, then left-to-right
    let mut sorted_chars: Vec<(usize, &Char)> = chars.iter().enumerate().collect();
    sorted_chars.sort_by(|a, b| {
        let y_cmp = a.1.top.partial_cmp(&b.1.top).unwrap();
        if y_cmp == std::cmp::Ordering::Equal {
            a.1.x0.partial_cmp(&b.1.x0).unwrap()
        } else {
            y_cmp
        }
    });

    let mut words: Vec<Word> = Vec::new();
    let mut current_word_chars: Vec<&Char> = vec![sorted_chars[0].1];

    for &(_, ch) in &sorted_chars[1..] {
        let last = *current_word_chars.last().unwrap();

        // Space characters are word separators
        let is_space = ch.text.trim().is_empty();

        // Check if this character continues the current word
        let same_line = (ch.top - last.top).abs() <= y_tolerance;
        let close_enough = (ch.x0 - last.x1).abs() <= x_tolerance;

        if same_line && close_enough && !is_space {
            current_word_chars.push(ch);
        } else {
            // Finish current word and start new one
            if let Some(word) = build_word(&current_word_chars) {
                words.push(word);
            }
            current_word_chars = vec![ch];
        }
    }

    // Don't forget the last word
    if let Some(word) = build_word(&current_word_chars) {
        words.push(word);
    }

    words
}

fn build_word(chars: &[&Char]) -> Option<Word> {
    if chars.is_empty() {
        return None;
    }

    // Filter out leading/trailing space characters
    let non_space_chars: Vec<&&Char> = chars
        .iter()
        .filter(|c| !c.text.trim().is_empty())
        .collect();

    if non_space_chars.is_empty() {
        return None;
    }

    let text: String = non_space_chars.iter().map(|c| c.text.as_str()).collect();

    // Skip whitespace-only words
    if text.trim().is_empty() {
        return None;
    }

    let x0 = non_space_chars.iter().map(|c| c.x0).fold(f64::MAX, f64::min);
    let x1 = non_space_chars.iter().map(|c| c.x1).fold(f64::MIN, f64::max);
    let top = non_space_chars.iter().map(|c| c.top).fold(f64::MAX, f64::min);
    let bottom = non_space_chars.iter().map(|c| c.bottom).fold(f64::MIN, f64::max);
    let doctop = non_space_chars.iter().map(|c| c.doctop).fold(f64::MAX, f64::min);

    Some(Word {
        text,
        x0,
        x1,
        top,
        bottom,
        doctop,
        upright: non_space_chars[0].upright,
        fontname: non_space_chars[0].fontname.clone(),
        size: non_space_chars[0].size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::Char;

    fn make_char(text: &str, x0: f64, x1: f64, top: f64) -> Char {
        Char {
            text: text.to_string(),
            fontname: "Helvetica".to_string(),
            size: 12.0,
            x0,
            x1,
            top,
            bottom: top + 12.0,
            doctop: top,
            matrix: [12.0, 0.0, 0.0, 12.0, x0, 780.0 - top],
            upright: true,
            stroking_color: std::sync::Arc::new(None),
            non_stroking_color: std::sync::Arc::new(None),
            adv: x1 - x0,
        }
    }

    #[test]
    fn test_word_grouping() {
        let chars = vec![
            make_char("H", 72.0, 80.0, 100.0),
            make_char("e", 80.0, 86.0, 100.0),
            make_char("l", 86.0, 89.0, 100.0),
            make_char("l", 89.0, 92.0, 100.0),
            make_char("o", 92.0, 98.0, 100.0),
            make_char(" ", 98.0, 101.0, 100.0),
            make_char("W", 105.0, 115.0, 100.0),
            make_char("o", 115.0, 121.0, 100.0),
            make_char("r", 121.0, 125.0, 100.0),
            make_char("l", 125.0, 128.0, 100.0),
            make_char("d", 128.0, 134.0, 100.0),
        ];

        let words = group_chars_to_words(&chars, 3.0, 3.0);
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].text, "Hello");
        assert_eq!(words[1].text, "World");
    }
}
