use std::collections::HashMap;

/// Parse a ToUnicode CMap stream into a character code → Unicode string mapping.
///
/// CMap syntax we handle:
/// ```text
/// beginbfchar
/// <0003> <0020>
/// endbfchar
/// beginbfrange
/// <0013> <0017> <0030>
/// <001D> <0024> [<004A> <004B> <004C>]
/// endbfrange
/// ```
pub fn parse_to_unicode_cmap(cmap_text: &str, mapping: &mut HashMap<u32, String>) {
    let lines: Vec<&str> = cmap_text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.contains("beginbfchar") {
            i += 1;
            while i < lines.len() {
                let line = lines[i].trim();
                if line.contains("endbfchar") {
                    break;
                }
                if let Some((code, unicode)) = parse_bfchar_line(line) {
                    mapping.insert(code, unicode);
                }
                i += 1;
            }
        } else if line.contains("beginbfrange") {
            i += 1;
            while i < lines.len() {
                let line = lines[i].trim();
                if line.contains("endbfrange") {
                    break;
                }
                parse_bfrange_line(line, mapping);
                i += 1;
            }
        }

        i += 1;
    }
}

/// Parse a line like `<0003> <0020>` (code → unicode)
fn parse_bfchar_line(line: &str) -> Option<(u32, String)> {
    let parts: Vec<&str> = line.split('<').collect();
    if parts.len() < 3 {
        return None;
    }

    let code_str = parts[1].split('>').next()?;
    let unicode_str = parts[2].split('>').next()?;

    let code = u32::from_str_radix(code_str.trim(), 16).ok()?;
    let unicode = hex_to_unicode_string(unicode_str.trim())?;

    Some((code, unicode))
}

/// Parse a line like `<0013> <0017> <0030>` (range start, range end, unicode start)
/// or `<001D> <0024> [<004A> <004B> ...]` (range with explicit values)
fn parse_bfrange_line(line: &str, mapping: &mut HashMap<u32, String>) {
    let trimmed = line.trim();

    // Check for array form: <start> <end> [<v1> <v2> ...]
    if let Some(bracket_pos) = trimmed.find('[') {
        let before_bracket = &trimmed[..bracket_pos];
        let parts: Vec<&str> = before_bracket.split('<').collect();
        if parts.len() < 3 {
            return;
        }

        let start_str = parts[1].split('>').next().unwrap_or("");
        let start = match u32::from_str_radix(start_str.trim(), 16) {
            Ok(v) => v,
            Err(_) => return,
        };

        // Parse array values
        let array_str = &trimmed[bracket_pos..];
        let values: Vec<&str> = array_str.split('<').collect();
        for (i, val) in values.iter().enumerate() {
            if i == 0 {
                continue; // Skip the "[" part
            }
            let hex = val.split('>').next().unwrap_or("");
            if let Some(unicode) = hex_to_unicode_string(hex.trim()) {
                mapping.insert(start + (i as u32 - 1), unicode);
            }
        }
    } else {
        // Standard form: <start> <end> <unicode_start>
        let parts: Vec<&str> = trimmed.split('<').collect();
        if parts.len() < 4 {
            return;
        }

        let start_str = parts[1].split('>').next().unwrap_or("");
        let end_str = parts[2].split('>').next().unwrap_or("");
        let unicode_str = parts[3].split('>').next().unwrap_or("");

        let start = match u32::from_str_radix(start_str.trim(), 16) {
            Ok(v) => v,
            Err(_) => return,
        };
        let end = match u32::from_str_radix(end_str.trim(), 16) {
            Ok(v) => v,
            Err(_) => return,
        };
        let unicode_start = match u32::from_str_radix(unicode_str.trim(), 16) {
            Ok(v) => v,
            Err(_) => return,
        };

        for code in start..=end {
            let unicode_code = unicode_start + (code - start);
            if let Some(c) = char::from_u32(unicode_code) {
                mapping.insert(code, c.to_string());
            }
        }
    }
}

/// Convert hex string like "0041" or "00410042" to a Unicode string.
fn hex_to_unicode_string(hex: &str) -> Option<String> {
    if hex.is_empty() {
        return None;
    }

    let mut result = String::new();
    let hex = hex.trim();

    // Each Unicode character is 4 hex digits (2 bytes, UTF-16)
    let mut i = 0;
    while i + 4 <= hex.len() {
        let code = u32::from_str_radix(&hex[i..i + 4], 16).ok()?;

        // Handle UTF-16 surrogate pairs
        if (0xD800..=0xDBFF).contains(&code) {
            // High surrogate
            if i + 8 <= hex.len() {
                let low = u32::from_str_radix(&hex[i + 4..i + 8], 16).ok()?;
                if (0xDC00..=0xDFFF).contains(&low) {
                    let codepoint = 0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00);
                    result.push(char::from_u32(codepoint)?);
                    i += 8;
                    continue;
                }
            }
            return None;
        }

        result.push(char::from_u32(code)?);
        i += 4;
    }

    // Handle 2-digit hex (single byte)
    if i + 2 <= hex.len() && result.is_empty() {
        let code = u32::from_str_radix(&hex[i..i + 2], 16).ok()?;
        result.push(char::from_u32(code)?);
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bfchar() {
        let (code, unicode) = parse_bfchar_line("<0041> <0041>").unwrap();
        assert_eq!(code, 0x41);
        assert_eq!(unicode, "A");
    }

    #[test]
    fn test_parse_bfrange() {
        let mut mapping = HashMap::new();
        parse_bfrange_line("<0041> <0043> <0041>", &mut mapping);
        assert_eq!(mapping.get(&0x41), Some(&"A".to_string()));
        assert_eq!(mapping.get(&0x42), Some(&"B".to_string()));
        assert_eq!(mapping.get(&0x43), Some(&"C".to_string()));
    }

    #[test]
    fn test_parse_full_cmap() {
        let cmap = r#"
/CIDInit /ProcSet findresource begin
12 dict begin
begincmap
/CMapType 2 def
1 begincodespacerange
<0000> <FFFF>
endcodespacerange
2 beginbfchar
<0003> <0020>
<0011> <002E>
endbfchar
1 beginbfrange
<0013> <0017> <0030>
endbfrange
endcmap
"#;
        let mut mapping = HashMap::new();
        parse_to_unicode_cmap(cmap, &mut mapping);

        assert_eq!(mapping.get(&0x0003), Some(&" ".to_string()));
        assert_eq!(mapping.get(&0x0011), Some(&".".to_string()));
        assert_eq!(mapping.get(&0x0013), Some(&"0".to_string()));
        assert_eq!(mapping.get(&0x0014), Some(&"1".to_string()));
    }

    #[test]
    fn test_hex_to_unicode() {
        assert_eq!(hex_to_unicode_string("0041"), Some("A".to_string()));
        assert_eq!(hex_to_unicode_string("00410042"), Some("AB".to_string()));
    }
}
