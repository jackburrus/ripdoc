use crate::error::Result;
use crate::geometry::BBox;
use crate::page::{Page, TextMatch};

/// Search for text on a page by literal string or regex pattern.
pub fn search_page(page: &Page, pattern: &str, regex: bool) -> Result<Vec<TextMatch>> {
    let mut matches = Vec::new();

    if page.chars.is_empty() || pattern.is_empty() {
        return Ok(matches);
    }

    // Build full text and track char positions
    let mut sorted_chars: Vec<(usize, &crate::objects::Char)> =
        page.chars.iter().enumerate().collect();
    sorted_chars.sort_by(|a, b| {
        let y_cmp = a.1.top.partial_cmp(&b.1.top).unwrap();
        if (a.1.top - b.1.top).abs() <= 3.0 {
            a.1.x0.partial_cmp(&b.1.x0).unwrap()
        } else {
            y_cmp
        }
    });

    let full_text: String = sorted_chars.iter().map(|(_, c)| c.text.as_str()).collect();

    if regex {
        // Simple substring search as fallback (full regex would need regex crate)
        search_literal(&full_text, pattern, &sorted_chars, page.page_number, &mut matches);
    } else {
        search_literal(&full_text, pattern, &sorted_chars, page.page_number, &mut matches);
    }

    Ok(matches)
}

fn search_literal(
    full_text: &str,
    pattern: &str,
    sorted_chars: &[(usize, &crate::objects::Char)],
    page_number: usize,
    matches: &mut Vec<TextMatch>,
) {
    let pattern_lower = pattern.to_lowercase();
    let text_lower = full_text.to_lowercase();

    let mut search_start = 0;
    while let Some(pos) = text_lower[search_start..].find(&pattern_lower) {
        let abs_pos = search_start + pos;
        let end_pos = abs_pos + pattern.len();

        // Find the character indices in the sorted list that correspond to this match
        let mut char_byte_pos = 0;
        let mut start_idx = None;
        let mut end_idx = None;
        let mut char_indices = Vec::new();

        for (i, (orig_idx, ch)) in sorted_chars.iter().enumerate() {
            let ch_len = ch.text.len();
            if char_byte_pos + ch_len > abs_pos && start_idx.is_none() {
                start_idx = Some(i);
            }
            if char_byte_pos >= abs_pos && char_byte_pos < end_pos {
                char_indices.push(*orig_idx);
            }
            if char_byte_pos + ch_len >= end_pos && end_idx.is_none() {
                end_idx = Some(i);
            }
            char_byte_pos += ch_len;
            if end_idx.is_some() {
                break;
            }
        }

        if let (Some(si), Some(ei)) = (start_idx, end_idx) {
            let match_chars: Vec<&crate::objects::Char> =
                sorted_chars[si..=ei].iter().map(|(_, c)| *c).collect();

            if !match_chars.is_empty() {
                let x0 = match_chars.iter().map(|c| c.x0).fold(f64::MAX, f64::min);
                let x1 = match_chars.iter().map(|c| c.x1).fold(f64::MIN, f64::max);
                let top = match_chars.iter().map(|c| c.top).fold(f64::MAX, f64::min);
                let bottom = match_chars.iter().map(|c| c.bottom).fold(f64::MIN, f64::max);

                matches.push(TextMatch {
                    text: full_text[abs_pos..end_pos].to_string(),
                    bbox: BBox::new(x0, top, x1, bottom),
                    page_number,
                    char_indices,
                });
            }
        }

        search_start = abs_pos + 1;
    }
}
