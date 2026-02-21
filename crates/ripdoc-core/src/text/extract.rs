use crate::objects::Char;
use crate::page::TextExtractOptions;

/// Extract text from a page's characters.
///
/// Two modes:
/// - Simple (layout=false): concatenate text in reading order with line breaks
/// - Layout-preserving (layout=true): preserve spatial positioning using a character grid
pub fn extract_text(
    chars: &[Char],
    page_width: f64,
    page_height: f64,
    options: &TextExtractOptions,
) -> String {
    if chars.is_empty() {
        return String::new();
    }

    if options.layout {
        extract_text_layout(chars, page_width, page_height, options)
    } else {
        extract_text_simple(chars, options)
    }
}

/// Simple text extraction: group chars into lines, join with spaces and newlines.
fn extract_text_simple(chars: &[Char], options: &TextExtractOptions) -> String {
    if chars.is_empty() {
        return String::new();
    }

    // Sort by position: top-to-bottom, left-to-right
    let mut sorted: Vec<&Char> = chars.iter().collect();
    sorted.sort_by(|a, b| {
        let y_cmp = a.top.partial_cmp(&b.top).unwrap();
        if (a.top - b.top).abs() <= options.y_tolerance {
            a.x0.partial_cmp(&b.x0).unwrap()
        } else {
            y_cmp
        }
    });

    let mut result = String::new();
    let mut current_line_top = sorted[0].top;

    for ch in &sorted {
        if !options.keep_blank_chars && ch.text.trim().is_empty() && ch.text != " " {
            continue;
        }

        // Check if we're on a new line
        if (ch.top - current_line_top).abs() > options.y_tolerance {
            // Remove trailing spaces and add newline
            let trimmed_len = result.trim_end_matches(' ').len();
            result.truncate(trimmed_len);
            result.push('\n');
            current_line_top = ch.top;
        } else if !result.is_empty() && !result.ends_with('\n') {
            // Check if there's a gap between this char and the last (insert space)
            let last_x1 = get_last_x1(&result, &sorted, ch);
            if let Some(prev_x1) = last_x1 {
                let gap = ch.x0 - prev_x1;
                if gap > options.x_tolerance {
                    result.push(' ');
                }
            }
        }

        result.push_str(&ch.text);
    }

    // Trim trailing whitespace
    result.trim_end().to_string()
}

fn get_last_x1(_result: &str, sorted: &[&Char], current: &Char) -> Option<f64> {
    // Find the previous character that contributed to the result
    for ch in sorted.iter().rev() {
        if std::ptr::eq(*ch, current) {
            continue;
        }
        if (ch.top - current.top).abs() <= 3.0 && ch.x1 <= current.x0 + 1.0 {
            return Some(ch.x1);
        }
    }
    None
}

/// Layout-preserving text extraction using a character grid.
/// Maps each character to its approximate grid position based on coordinates.
fn extract_text_layout(
    chars: &[Char],
    page_width: f64,
    _page_height: f64,
    options: &TextExtractOptions,
) -> String {
    if chars.is_empty() {
        return String::new();
    }

    // Calculate grid dimensions
    let cols = (page_width / options.x_density).ceil() as usize;
    if cols == 0 {
        return String::new();
    }

    // Group characters by line (y-position)
    let mut lines: Vec<Vec<&Char>> = Vec::new();
    let mut sorted: Vec<&Char> = chars.iter().collect();
    sorted.sort_by(|a, b| a.top.partial_cmp(&b.top).unwrap());

    let mut current_line: Vec<&Char> = vec![sorted[0]];
    let mut current_top = sorted[0].top;

    for ch in &sorted[1..] {
        if (ch.top - current_top).abs() <= options.y_tolerance {
            current_line.push(ch);
        } else {
            if !current_line.is_empty() {
                lines.push(current_line);
            }
            current_line = vec![ch];
            current_top = ch.top;
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // Render each line onto a character grid
    let mut output_lines: Vec<String> = Vec::new();

    for line_chars in &lines {
        let mut grid = vec![' '; cols + 1];

        for ch in line_chars {
            let col = (ch.x0 / options.x_density).round() as usize;
            if col < grid.len() {
                for (i, c) in ch.text.chars().enumerate() {
                    if col + i < grid.len() {
                        grid[col + i] = c;
                    }
                }
            }
        }

        let line: String = grid.into_iter().collect();
        output_lines.push(line.trim_end().to_string());
    }

    // Remove trailing empty lines
    while output_lines.last().map_or(false, |l| l.is_empty()) {
        output_lines.pop();
    }

    output_lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_simple_extraction() {
        let chars = vec![
            make_char("H", 72.0, 80.0, 100.0),
            make_char("i", 80.0, 84.0, 100.0),
        ];
        let text = extract_text_simple(&chars, &TextExtractOptions::default());
        assert_eq!(text, "Hi");
    }

    #[test]
    fn test_multiline() {
        let chars = vec![
            make_char("A", 72.0, 80.0, 100.0),
            make_char("B", 72.0, 80.0, 120.0),
        ];
        let text = extract_text_simple(&chars, &TextExtractOptions::default());
        assert_eq!(text, "A\nB");
    }
}
