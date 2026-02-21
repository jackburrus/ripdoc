use std::collections::HashMap;

/// Font encoding types.
#[derive(Debug, Clone)]
pub enum Encoding {
    Standard,
    MacRoman,
    WinAnsi,
    PDFDoc,
    MacExpert,
    Identity,
    Custom {
        base: Box<Encoding>,
        overrides: HashMap<u32, char>,
    },
}

impl Encoding {
    pub fn from_name(name: &[u8]) -> Self {
        match name {
            b"WinAnsiEncoding" => Encoding::WinAnsi,
            b"MacRomanEncoding" => Encoding::MacRoman,
            b"StandardEncoding" => Encoding::Standard,
            b"PDFDocEncoding" => Encoding::PDFDoc,
            b"MacExpertEncoding" => Encoding::MacExpert,
            b"Identity-H" | b"Identity-V" => Encoding::Identity,
            _ => Encoding::Standard,
        }
    }

    pub fn from_base_font(base_font: &str) -> Self {
        if base_font.contains("Symbol") {
            Encoding::Standard
        } else if base_font.contains("ZapfDingbats") {
            Encoding::Standard
        } else {
            Encoding::WinAnsi
        }
    }

    pub fn decode(&self, code: u32) -> Option<char> {
        match self {
            Encoding::Custom { base, overrides } => {
                overrides.get(&code).copied().or_else(|| base.decode(code))
            }
            Encoding::WinAnsi => win_ansi_decode(code),
            Encoding::MacRoman => mac_roman_decode(code),
            Encoding::Standard => standard_decode(code),
            Encoding::PDFDoc => pdf_doc_decode(code),
            Encoding::Identity => char::from_u32(code),
            Encoding::MacExpert => standard_decode(code),
        }
    }
}

fn win_ansi_decode(code: u32) -> Option<char> {
    if code < 128 {
        return char::from_u32(code);
    }
    // Windows-1252 upper range
    let c = match code {
        128 => '\u{20AC}', // Euro sign
        130 => '\u{201A}', // Single low-9 quotation mark
        131 => '\u{0192}', // Latin small letter f with hook
        132 => '\u{201E}', // Double low-9 quotation mark
        133 => '\u{2026}', // Horizontal ellipsis
        134 => '\u{2020}', // Dagger
        135 => '\u{2021}', // Double dagger
        136 => '\u{02C6}', // Modifier letter circumflex accent
        137 => '\u{2030}', // Per mille sign
        138 => '\u{0160}', // Latin capital letter S with caron
        139 => '\u{2039}', // Single left-pointing angle quotation mark
        140 => '\u{0152}', // Latin capital ligature OE
        142 => '\u{017D}', // Latin capital letter Z with caron
        145 => '\u{2018}', // Left single quotation mark
        146 => '\u{2019}', // Right single quotation mark
        147 => '\u{201C}', // Left double quotation mark
        148 => '\u{201D}', // Right double quotation mark
        149 => '\u{2022}', // Bullet
        150 => '\u{2013}', // En dash
        151 => '\u{2014}', // Em dash
        152 => '\u{02DC}', // Small tilde
        153 => '\u{2122}', // Trade mark sign
        154 => '\u{0161}', // Latin small letter s with caron
        155 => '\u{203A}', // Single right-pointing angle quotation mark
        156 => '\u{0153}', // Latin small ligature oe
        158 => '\u{017E}', // Latin small letter z with caron
        159 => '\u{0178}', // Latin capital letter Y with diaeresis
        160..=255 => char::from_u32(code)?,
        _ => return None,
    };
    Some(c)
}

fn mac_roman_decode(code: u32) -> Option<char> {
    if code < 128 {
        return char::from_u32(code);
    }
    // MacRoman high bytes
    static MAC_ROMAN_HIGH: [u16; 128] = [
        0x00C4, 0x00C5, 0x00C7, 0x00C9, 0x00D1, 0x00D6, 0x00DC, 0x00E1,
        0x00E0, 0x00E2, 0x00E4, 0x00E3, 0x00E5, 0x00E7, 0x00E9, 0x00E8,
        0x00EA, 0x00EB, 0x00ED, 0x00EC, 0x00EE, 0x00EF, 0x00F1, 0x00F3,
        0x00F2, 0x00F4, 0x00F6, 0x00F5, 0x00FA, 0x00F9, 0x00FB, 0x00FC,
        0x2020, 0x00B0, 0x00A2, 0x00A3, 0x00A7, 0x2022, 0x00B6, 0x00DF,
        0x00AE, 0x00A9, 0x2122, 0x00B4, 0x00A8, 0x2260, 0x00C6, 0x00D8,
        0x221E, 0x00B1, 0x2264, 0x2265, 0x00A5, 0x00B5, 0x2202, 0x2211,
        0x220F, 0x03C0, 0x222B, 0x00AA, 0x00BA, 0x2126, 0x00E6, 0x00F8,
        0x00BF, 0x00A1, 0x00AC, 0x221A, 0x0192, 0x2248, 0x2206, 0x00AB,
        0x00BB, 0x2026, 0x00A0, 0x00C0, 0x00C3, 0x00D5, 0x0152, 0x0153,
        0x2013, 0x2014, 0x201C, 0x201D, 0x2018, 0x2019, 0x00F7, 0x25CA,
        0x00FF, 0x0178, 0x2044, 0x20AC, 0x2039, 0x203A, 0xFB01, 0xFB02,
        0x2021, 0x00B7, 0x201A, 0x201E, 0x2030, 0x00C2, 0x00CA, 0x00C1,
        0x00CB, 0x00C8, 0x00CD, 0x00CE, 0x00CF, 0x00CC, 0x00D3, 0x00D4,
        0xF8FF, 0x00D2, 0x00DA, 0x00DB, 0x00D9, 0x0131, 0x02C6, 0x02DC,
        0x00AF, 0x02D8, 0x02D9, 0x02DA, 0x00B8, 0x02DD, 0x02DB, 0x02C7,
    ];
    if code >= 128 && code < 256 {
        char::from_u32(MAC_ROMAN_HIGH[(code - 128) as usize] as u32)
    } else {
        None
    }
}

fn standard_decode(code: u32) -> Option<char> {
    // StandardEncoding — for codes < 128, it's mostly ASCII
    if code < 128 {
        return char::from_u32(code);
    }
    // For higher codes, use common PostScript standard encoding mappings
    let c = match code {
        0xA1 => '\u{00A1}', // exclamdown
        0xA2 => '\u{00A2}', // cent
        0xA3 => '\u{00A3}', // sterling
        0xA4 => '\u{2044}', // fraction
        0xA5 => '\u{00A5}', // yen
        0xA6 => '\u{0192}', // florin
        0xA7 => '\u{00A7}', // section
        0xA8 => '\u{00A4}', // currency
        0xA9 => '\u{0027}', // quotesingle
        0xAA => '\u{201C}', // quotedblleft
        0xAB => '\u{00AB}', // guillemotleft
        0xAC => '\u{2039}', // guilsinglleft
        0xAD => '\u{203A}', // guilsinglright
        0xAE => '\u{FB01}', // fi
        0xAF => '\u{FB02}', // fl
        0xB1 => '\u{2013}', // endash
        0xB2 => '\u{2020}', // dagger
        0xB3 => '\u{2021}', // daggerdbl
        0xB4 => '\u{00B7}', // periodcentered
        0xB7 => '\u{2022}', // bullet
        0xB8 => '\u{201A}', // quotesinglbase
        0xB9 => '\u{201E}', // quotedblbase
        0xBA => '\u{201D}', // quotedblright
        0xBB => '\u{00BB}', // guillemotright
        0xBC => '\u{2026}', // ellipsis
        0xBD => '\u{2030}', // perthousand
        0xC1 => '\u{0060}', // grave
        0xC2 => '\u{00B4}', // acute
        0xC3 => '\u{02C6}', // circumflex
        0xC4 => '\u{02DC}', // tilde
        0xC5 => '\u{00AF}', // macron
        0xC6 => '\u{02D8}', // breve
        0xC7 => '\u{02D9}', // dotaccent
        0xC8 => '\u{00A8}', // dieresis
        0xCA => '\u{02DA}', // ring
        0xCB => '\u{00B8}', // cedilla
        0xCD => '\u{02DD}', // hungarumlaut
        0xCE => '\u{02DB}', // ogonek
        0xCF => '\u{02C7}', // caron
        0xD0 => '\u{2014}', // emdash
        0xE1 => '\u{00C6}', // AE
        0xE3 => '\u{00AA}', // ordfeminine
        0xE8 => '\u{0141}', // Lslash
        0xE9 => '\u{00D8}', // Oslash
        0xEA => '\u{0152}', // OE
        0xEB => '\u{00BA}', // ordmasculine
        0xF1 => '\u{00E6}', // ae
        0xF5 => '\u{0131}', // dotlessi
        0xF8 => '\u{0142}', // lslash
        0xF9 => '\u{00F8}', // oslash
        0xFA => '\u{0153}', // oe
        0xFB => '\u{00DF}', // germandbls
        _ => return char::from_u32(code),
    };
    Some(c)
}

fn pdf_doc_decode(code: u32) -> Option<char> {
    // PDFDocEncoding is similar to Latin-1 with some exceptions
    match code {
        0x80 => Some('\u{2022}'),
        0x81 => Some('\u{2020}'),
        0x82 => Some('\u{2021}'),
        0x83 => Some('\u{2026}'),
        0x84 => Some('\u{2014}'),
        0x85 => Some('\u{2013}'),
        0x86 => Some('\u{0192}'),
        0x87 => Some('\u{2044}'),
        0x88 => Some('\u{2039}'),
        0x89 => Some('\u{203A}'),
        0x8A => Some('\u{2212}'),
        0x8B => Some('\u{2030}'),
        0x8C => Some('\u{201E}'),
        0x8D => Some('\u{201C}'),
        0x8E => Some('\u{201D}'),
        0x8F => Some('\u{2018}'),
        0x90 => Some('\u{2019}'),
        0x91 => Some('\u{201A}'),
        0x92 => Some('\u{2122}'),
        0x93 => Some('\u{FB01}'),
        0x94 => Some('\u{FB02}'),
        0x95 => Some('\u{0141}'),
        0x96 => Some('\u{0152}'),
        0x97 => Some('\u{0160}'),
        0x98 => Some('\u{0178}'),
        0x99 => Some('\u{017D}'),
        0x9A => Some('\u{0131}'),
        0x9B => Some('\u{0142}'),
        0x9C => Some('\u{0153}'),
        0x9D => Some('\u{0161}'),
        0x9E => Some('\u{017E}'),
        0xAD => Some('\u{00AD}'),
        _ => char::from_u32(code),
    }
}

/// Map Adobe glyph names to Unicode characters.
pub fn glyph_name_to_char(name: &str) -> Option<char> {
    // Check for uniXXXX format
    if name.starts_with("uni") && name.len() == 7 {
        if let Ok(code) = u32::from_str_radix(&name[3..], 16) {
            return char::from_u32(code);
        }
    }

    // Common glyph name mappings
    match name {
        "space" => Some(' '),
        "exclam" => Some('!'),
        "quotedbl" => Some('"'),
        "numbersign" => Some('#'),
        "dollar" => Some('$'),
        "percent" => Some('%'),
        "ampersand" => Some('&'),
        "quotesingle" => Some('\''),
        "parenleft" => Some('('),
        "parenright" => Some(')'),
        "asterisk" => Some('*'),
        "plus" => Some('+'),
        "comma" => Some(','),
        "hyphen" | "minus" => Some('-'),
        "period" => Some('.'),
        "slash" => Some('/'),
        "zero" => Some('0'),
        "one" => Some('1'),
        "two" => Some('2'),
        "three" => Some('3'),
        "four" => Some('4'),
        "five" => Some('5'),
        "six" => Some('6'),
        "seven" => Some('7'),
        "eight" => Some('8'),
        "nine" => Some('9'),
        "colon" => Some(':'),
        "semicolon" => Some(';'),
        "less" => Some('<'),
        "equal" => Some('='),
        "greater" => Some('>'),
        "question" => Some('?'),
        "at" => Some('@'),
        "bracketleft" => Some('['),
        "backslash" => Some('\\'),
        "bracketright" => Some(']'),
        "asciicircum" => Some('^'),
        "underscore" => Some('_'),
        "grave" => Some('`'),
        "braceleft" => Some('{'),
        "bar" => Some('|'),
        "braceright" => Some('}'),
        "asciitilde" => Some('~'),
        "bullet" => Some('\u{2022}'),
        "endash" => Some('\u{2013}'),
        "emdash" => Some('\u{2014}'),
        "quotedblleft" => Some('\u{201C}'),
        "quotedblright" => Some('\u{201D}'),
        "quoteleft" => Some('\u{2018}'),
        "quoteright" => Some('\u{2019}'),
        "fi" => Some('\u{FB01}'),
        "fl" => Some('\u{FB02}'),
        "ellipsis" => Some('\u{2026}'),
        "dagger" => Some('\u{2020}'),
        "daggerdbl" => Some('\u{2021}'),
        "circumflex" => Some('\u{02C6}'),
        "tilde" => Some('\u{02DC}'),
        "degree" => Some('\u{00B0}'),
        "cent" => Some('\u{00A2}'),
        "sterling" => Some('\u{00A3}'),
        "yen" => Some('\u{00A5}'),
        "Euro" => Some('\u{20AC}'),
        "copyright" => Some('\u{00A9}'),
        "registered" => Some('\u{00AE}'),
        "trademark" => Some('\u{2122}'),
        "section" => Some('\u{00A7}'),
        "paragraph" => Some('\u{00B6}'),
        "germandbls" => Some('\u{00DF}'),
        // Latin accented characters
        "Agrave" => Some('\u{00C0}'),
        "Aacute" => Some('\u{00C1}'),
        "Acircumflex" => Some('\u{00C2}'),
        "Atilde" => Some('\u{00C3}'),
        "Adieresis" => Some('\u{00C4}'),
        "Aring" => Some('\u{00C5}'),
        "AE" => Some('\u{00C6}'),
        "Ccedilla" => Some('\u{00C7}'),
        "Egrave" => Some('\u{00C8}'),
        "Eacute" => Some('\u{00C9}'),
        "Ecircumflex" => Some('\u{00CA}'),
        "Edieresis" => Some('\u{00CB}'),
        "Igrave" => Some('\u{00CC}'),
        "Iacute" => Some('\u{00CD}'),
        "Icircumflex" => Some('\u{00CE}'),
        "Idieresis" => Some('\u{00CF}'),
        "Ntilde" => Some('\u{00D1}'),
        "Ograve" => Some('\u{00D2}'),
        "Oacute" => Some('\u{00D3}'),
        "Ocircumflex" => Some('\u{00D4}'),
        "Otilde" => Some('\u{00D5}'),
        "Odieresis" => Some('\u{00D6}'),
        "Oslash" => Some('\u{00D8}'),
        "Ugrave" => Some('\u{00D9}'),
        "Uacute" => Some('\u{00DA}'),
        "Ucircumflex" => Some('\u{00DB}'),
        "Udieresis" => Some('\u{00DC}'),
        "Yacute" => Some('\u{00DD}'),
        "agrave" => Some('\u{00E0}'),
        "aacute" => Some('\u{00E1}'),
        "acircumflex" => Some('\u{00E2}'),
        "atilde" => Some('\u{00E3}'),
        "adieresis" => Some('\u{00E4}'),
        "aring" => Some('\u{00E5}'),
        "ae" => Some('\u{00E6}'),
        "ccedilla" => Some('\u{00E7}'),
        "egrave" => Some('\u{00E8}'),
        "eacute" => Some('\u{00E9}'),
        "ecircumflex" => Some('\u{00EA}'),
        "edieresis" => Some('\u{00EB}'),
        "igrave" => Some('\u{00EC}'),
        "iacute" => Some('\u{00ED}'),
        "icircumflex" => Some('\u{00EE}'),
        "idieresis" => Some('\u{00EF}'),
        "ntilde" => Some('\u{00F1}'),
        "ograve" => Some('\u{00F2}'),
        "oacute" => Some('\u{00F3}'),
        "ocircumflex" => Some('\u{00F4}'),
        "otilde" => Some('\u{00F5}'),
        "odieresis" => Some('\u{00F6}'),
        "oslash" => Some('\u{00F8}'),
        "ugrave" => Some('\u{00F9}'),
        "uacute" => Some('\u{00FA}'),
        "ucircumflex" => Some('\u{00FB}'),
        "udieresis" => Some('\u{00FC}'),
        "yacute" => Some('\u{00FD}'),
        "ydieresis" => Some('\u{00FF}'),
        // Single-character names (A-Z, a-z)
        s if s.len() == 1 => s.chars().next(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win_ansi() {
        assert_eq!(win_ansi_decode(65), Some('A'));
        assert_eq!(win_ansi_decode(128), Some('\u{20AC}')); // Euro
        assert_eq!(win_ansi_decode(147), Some('\u{201C}')); // Left double quote
    }

    #[test]
    fn test_glyph_name() {
        assert_eq!(glyph_name_to_char("space"), Some(' '));
        assert_eq!(glyph_name_to_char("fi"), Some('\u{FB01}'));
        assert_eq!(glyph_name_to_char("uni0041"), Some('A'));
    }
}
