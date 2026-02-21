pub mod cmap;
pub mod encoding;
pub mod metrics;

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{Error, Result};

/// Resolved font information for character decoding and positioning.
#[derive(Debug, Clone)]
pub struct FontInfo {
    pub name: String,
    pub base_font: String,
    pub subtype: FontSubtype,
    /// Maps character codes to Unicode strings.
    pub to_unicode: HashMap<u32, String>,
    /// Maps character codes to glyph widths (in 1/1000 text space units).
    pub widths: HashMap<u32, f64>,
    /// Default width for characters not in the widths map.
    pub default_width: f64,
    /// First character code in the Widths array.
    pub first_char: u32,
    /// Encoding used by this font.
    pub encoding: encoding::Encoding,
    /// Whether this font is a CID font (Type0).
    pub is_cid: bool,
    /// Number of bytes per character code.
    pub bytes_per_char: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontSubtype {
    Type1,
    TrueType,
    Type3,
    Type0,
    CIDFontType0,
    CIDFontType2,
    MMType1,
    Unknown(String),
}

impl FontInfo {
    /// Decode a character code to a Unicode string.
    pub fn decode_char(&self, code: u32) -> String {
        // First try ToUnicode map (highest priority)
        if let Some(s) = self.to_unicode.get(&code) {
            return s.clone();
        }

        // Then try encoding-based lookup
        if let Some(c) = self.encoding.decode(code) {
            return c.to_string();
        }

        // Fallback: try standard Unicode mapping
        if code < 128 {
            if let Some(c) = char::from_u32(code) {
                return c.to_string();
            }
        }

        // Last resort: replacement character
        String::from('\u{FFFD}')
    }

    /// Get the width of a character code in 1/1000 text space units.
    pub fn char_width(&self, code: u32) -> f64 {
        self.widths.get(&code).copied().unwrap_or(self.default_width)
    }

    /// Decode a byte string into characters with their codes and unicode.
    pub fn decode_string(&self, bytes: &[u8]) -> Vec<(u32, String)> {
        if self.is_cid || self.bytes_per_char == 2 {
            // CID font: 2 bytes per character
            bytes
                .chunks(2)
                .map(|chunk| {
                    let code = if chunk.len() == 2 {
                        ((chunk[0] as u32) << 8) | (chunk[1] as u32)
                    } else {
                        chunk[0] as u32
                    };
                    let text = self.decode_char(code);
                    (code, text)
                })
                .collect()
        } else {
            // Simple font: 1 byte per character
            bytes
                .iter()
                .map(|&b| {
                    let code = b as u32;
                    let text = self.decode_char(code);
                    (code, text)
                })
                .collect()
        }
    }
}

impl Default for FontInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            base_font: String::new(),
            subtype: FontSubtype::Type1,
            to_unicode: HashMap::new(),
            widths: HashMap::new(),
            default_width: 1000.0,
            first_char: 0,
            encoding: encoding::Encoding::Standard,
            is_cid: false,
            bytes_per_char: 1,
        }
    }
}

/// Cache of resolved fonts for a document.
#[derive(Debug, Default)]
pub struct FontCache {
    fonts: HashMap<String, Arc<FontInfo>>,
}

impl FontCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, name: &str) -> Option<Arc<FontInfo>> {
        self.fonts.get(name).cloned()
    }

    pub fn insert(&mut self, name: String, font: FontInfo) {
        self.fonts.insert(name, Arc::new(font));
    }

    pub fn contains(&self, name: &str) -> bool {
        self.fonts.contains_key(name)
    }
}

/// Resolve a font dictionary from lopdf into FontInfo.
pub fn resolve_font(
    doc: &lopdf::Document,
    font_name: &str,
    font_dict: &lopdf::Dictionary,
) -> Result<FontInfo> {
    let mut info = FontInfo::default();
    info.name = font_name.to_string();

    // Get base font name
    if let Ok(base_font) = font_dict.get(b"BaseFont") {
        if let Ok(name) = base_font.as_name() {
            info.base_font = String::from_utf8_lossy(name).to_string();
        }
    }

    // Get subtype
    if let Ok(subtype) = font_dict.get(b"Subtype") {
        if let Ok(name) = subtype.as_name() {
            info.subtype = match name {
                b"Type1" => FontSubtype::Type1,
                b"TrueType" => FontSubtype::TrueType,
                b"Type3" => FontSubtype::Type3,
                b"Type0" => FontSubtype::Type0,
                b"CIDFontType0" => FontSubtype::CIDFontType0,
                b"CIDFontType2" => FontSubtype::CIDFontType2,
                b"MMType1" => FontSubtype::MMType1,
                other => FontSubtype::Unknown(String::from_utf8_lossy(other).to_string()),
            };
        }
    }

    // Handle Type0 (CID) fonts
    if info.subtype == FontSubtype::Type0 {
        info.is_cid = true;
        info.bytes_per_char = 2;
        resolve_type0_font(doc, font_dict, &mut info)?;
    } else {
        resolve_simple_font(doc, font_dict, &mut info)?;
    }

    // Parse ToUnicode CMap (works for all font types)
    if let Ok(to_unicode_obj) = font_dict.get(b"ToUnicode") {
        parse_to_unicode(doc, to_unicode_obj, &mut info)?;
    }

    Ok(info)
}

fn resolve_simple_font(
    doc: &lopdf::Document,
    font_dict: &lopdf::Dictionary,
    info: &mut FontInfo,
) -> Result<()> {
    // Get encoding
    if let Ok(enc_obj) = font_dict.get(b"Encoding") {
        match enc_obj {
            lopdf::Object::Name(name) => {
                info.encoding = encoding::Encoding::from_name(name);
            }
            lopdf::Object::Reference(id) => {
                if let Ok(obj) = doc.get_object(*id) {
                    if let Ok(name) = obj.as_name() {
                        info.encoding = encoding::Encoding::from_name(name);
                    } else if let Ok(dict) = obj.as_dict() {
                        parse_encoding_dict(dict, info)?;
                    }
                }
            }
            lopdf::Object::Dictionary(dict) => {
                parse_encoding_dict(dict, info)?;
            }
            _ => {}
        }
    } else {
        // Check if it's a standard font
        info.encoding = encoding::Encoding::from_base_font(&info.base_font);
    }

    // Get widths
    if let Ok(first_char) = font_dict.get(b"FirstChar") {
        info.first_char = first_char.as_i64().unwrap_or(0) as u32;
    }

    if let Ok(widths_obj) = font_dict.get(b"Widths") {
        let widths = match widths_obj {
            lopdf::Object::Reference(id) => {
                doc.get_object(*id)
                    .ok()
                    .and_then(|o| o.as_array().ok())
                    .cloned()
                    .unwrap_or_default()
            }
            lopdf::Object::Array(arr) => arr.clone(),
            _ => vec![],
        };

        for (i, w) in widths.iter().enumerate() {
            let width = match w {
                lopdf::Object::Integer(n) => *n as f64,
                lopdf::Object::Real(n) => *n as f64,
                lopdf::Object::Reference(id) => {
                    doc.get_object(*id)
                        .ok()
                        .map(|o| match o {
                            lopdf::Object::Integer(n) => *n as f64,
                            lopdf::Object::Real(n) => *n as f64,
                            _ => 0.0,
                        })
                        .unwrap_or(0.0)
                }
                _ => 0.0,
            };
            info.widths.insert(info.first_char + i as u32, width);
        }
    } else {
        // Use standard widths for known fonts
        metrics::load_standard_widths(&info.base_font, &mut info.widths);
    }

    // Get default width from font descriptor
    if let Ok(desc_obj) = font_dict.get(b"FontDescriptor") {
        let desc = match desc_obj {
            lopdf::Object::Reference(id) => doc.get_object(*id).ok().and_then(|o| o.as_dict().ok()),
            lopdf::Object::Dictionary(d) => Some(d),
            _ => None,
        };
        if let Some(desc) = desc {
            if let Ok(mw) = desc.get(b"MissingWidth") {
                info.default_width = match mw {
                    lopdf::Object::Integer(n) => *n as f64,
                    lopdf::Object::Real(n) => *n as f64,
                    _ => 1000.0,
                };
            }
        }
    }

    Ok(())
}

fn resolve_type0_font(
    doc: &lopdf::Document,
    font_dict: &lopdf::Dictionary,
    info: &mut FontInfo,
) -> Result<()> {
    // Get descendant fonts
    if let Ok(descendants) = font_dict.get(b"DescendantFonts") {
        let desc_array = match descendants {
            lopdf::Object::Array(arr) => arr.clone(),
            lopdf::Object::Reference(id) => {
                doc.get_object(*id)
                    .ok()
                    .and_then(|o| o.as_array().ok())
                    .cloned()
                    .unwrap_or_default()
            }
            _ => vec![],
        };

        if let Some(first) = desc_array.first() {
            let cid_font_dict = match first {
                lopdf::Object::Reference(id) => {
                    doc.get_object(*id).ok().and_then(|o| o.as_dict().ok())
                }
                lopdf::Object::Dictionary(d) => Some(d),
                _ => None,
            };

            if let Some(cid_dict) = cid_font_dict {
                // Get default width
                if let Ok(dw) = cid_dict.get(b"DW") {
                    info.default_width = match dw {
                        lopdf::Object::Integer(n) => *n as f64,
                        lopdf::Object::Real(n) => *n as f64,
                        _ => 1000.0,
                    };
                }

                // Get widths from /W array
                if let Ok(w_obj) = cid_dict.get(b"W") {
                    let w_array = match w_obj {
                        lopdf::Object::Array(arr) => arr.clone(),
                        lopdf::Object::Reference(id) => {
                            doc.get_object(*id)
                                .ok()
                                .and_then(|o| o.as_array().ok())
                                .cloned()
                                .unwrap_or_default()
                        }
                        _ => vec![],
                    };
                    parse_cid_widths(&w_array, &mut info.widths);
                }
            }
        }
    }

    Ok(())
}

/// Parse CID width array format:
/// [cid [w1 w2 ...]] or [cid_start cid_end w]
fn parse_cid_widths(w_array: &[lopdf::Object], widths: &mut HashMap<u32, f64>) {
    let mut i = 0;
    while i < w_array.len() {
        let start_cid = match &w_array[i] {
            lopdf::Object::Integer(n) => *n as u32,
            _ => {
                i += 1;
                continue;
            }
        };
        i += 1;

        if i >= w_array.len() {
            break;
        }

        match &w_array[i] {
            lopdf::Object::Array(arr) => {
                // [start_cid [w1 w2 w3 ...]]
                for (j, w) in arr.iter().enumerate() {
                    let width = match w {
                        lopdf::Object::Integer(n) => *n as f64,
                        lopdf::Object::Real(n) => *n as f64,
                        _ => continue,
                    };
                    widths.insert(start_cid + j as u32, width);
                }
                i += 1;
            }
            lopdf::Object::Integer(end_cid) => {
                // [start_cid end_cid width]
                let end_cid = *end_cid as u32;
                i += 1;
                if i < w_array.len() {
                    let width = match &w_array[i] {
                        lopdf::Object::Integer(n) => *n as f64,
                        lopdf::Object::Real(n) => *n as f64,
                        _ => {
                            i += 1;
                            continue;
                        }
                    };
                    for cid in start_cid..=end_cid {
                        widths.insert(cid, width);
                    }
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }
}

fn parse_encoding_dict(dict: &lopdf::Dictionary, info: &mut FontInfo) -> Result<()> {
    // Base encoding
    if let Ok(base_enc) = dict.get(b"BaseEncoding") {
        if let Ok(name) = base_enc.as_name() {
            info.encoding = encoding::Encoding::from_name(name);
        }
    }

    // Differences array for custom encoding
    if let Ok(diffs) = dict.get(b"Differences") {
        if let Ok(arr) = diffs.as_array() {
            let mut code = 0u32;
            let mut overrides = HashMap::new();

            for obj in arr {
                match obj {
                    lopdf::Object::Integer(n) => {
                        code = *n as u32;
                    }
                    lopdf::Object::Name(name) => {
                        let glyph_name = String::from_utf8_lossy(name).to_string();
                        if let Some(c) = encoding::glyph_name_to_char(&glyph_name) {
                            overrides.insert(code, c);
                        }
                        code += 1;
                    }
                    _ => {}
                }
            }

            if !overrides.is_empty() {
                info.encoding = encoding::Encoding::Custom {
                    base: Box::new(info.encoding.clone()),
                    overrides,
                };
            }
        }
    }

    Ok(())
}

fn parse_to_unicode(
    doc: &lopdf::Document,
    obj: &lopdf::Object,
    info: &mut FontInfo,
) -> Result<()> {
    let stream_data = match obj {
        lopdf::Object::Reference(id) => {
            let obj = doc.get_object(*id).map_err(|e| Error::Font(e.to_string()))?;
            match obj {
                lopdf::Object::Stream(stream) => {
                    let mut stream = stream.clone();
                    let _ = stream.decompress();
                    stream.content.clone()
                }
                _ => return Ok(()),
            }
        }
        lopdf::Object::Stream(stream) => {
            let mut stream = stream.clone();
            let _ = stream.decompress();
            stream.content.clone()
        }
        _ => return Ok(()),
    };

    let cmap_str = String::from_utf8_lossy(&stream_data);
    cmap::parse_to_unicode_cmap(&cmap_str, &mut info.to_unicode);

    Ok(())
}
