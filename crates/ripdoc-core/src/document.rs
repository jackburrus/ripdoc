use std::collections::HashMap;
use std::path::Path;

use lopdf::{Document as LopdfDocument, Object, ObjectId};

use crate::content_stream::ContentStreamInterpreter;
use crate::error::{Error, Result};
use crate::fonts::FontCache;
use crate::page::Page;

/// A PDF document opened for extraction.
pub struct Document {
    inner: LopdfDocument,
    page_ids: Vec<(u32, ObjectId)>,
    pages_cache: HashMap<usize, Page>,
    font_cache: FontCache,
}

impl Document {
    /// Open a PDF document from a file path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let inner = LopdfDocument::load(path).map_err(|e| Error::PdfParse(e.to_string()))?;
        Self::from_lopdf(inner)
    }

    /// Open a PDF document from bytes in memory.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let inner =
            LopdfDocument::load_mem(data).map_err(|e| Error::PdfParse(e.to_string()))?;
        Self::from_lopdf(inner)
    }

    fn from_lopdf(inner: LopdfDocument) -> Result<Self> {
        let mut page_ids: Vec<(u32, ObjectId)> = inner.get_pages().into_iter().collect();
        page_ids.sort_by_key(|(num, _)| *num);

        Ok(Self {
            inner,
            page_ids,
            pages_cache: HashMap::new(),
            font_cache: FontCache::new(),
        })
    }

    /// Get the number of pages.
    pub fn page_count(&self) -> usize {
        self.page_ids.len()
    }

    /// Get document metadata.
    pub fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();

        // Try to get info dict from trailer
        if let Ok(Object::Reference(info_ref)) = self.inner.trailer.get(b"Info") {
            if let Ok(Object::Dictionary(info)) = self.inner.get_object(*info_ref) {
                for (key, value) in info.iter() {
                    let key_str = String::from_utf8_lossy(key).to_string();
                    let val_str = match value {
                        Object::String(s, _) => String::from_utf8_lossy(s).to_string(),
                        Object::Name(n) => String::from_utf8_lossy(n).to_string(),
                        Object::Integer(n) => n.to_string(),
                        Object::Real(n) => n.to_string(),
                        _ => continue,
                    };
                    meta.insert(key_str, val_str);
                }
            }
        }

        meta
    }

    /// Extract a single page (1-indexed, like pdfplumber).
    pub fn page(&mut self, page_number: usize) -> Result<&Page> {
        if page_number == 0 || page_number > self.page_ids.len() {
            return Err(Error::PageNotFound(page_number));
        }

        if self.pages_cache.contains_key(&page_number) {
            return Ok(self.pages_cache.get(&page_number).unwrap());
        }

        let page = self.extract_page(page_number)?;
        self.pages_cache.insert(page_number, page);
        Ok(self.pages_cache.get(&page_number).unwrap())
    }

    /// Extract all pages.
    pub fn pages(&mut self) -> Result<Vec<&Page>> {
        let count = self.page_count();
        for i in 1..=count {
            if !self.pages_cache.contains_key(&i) {
                let page = self.extract_page(i)?;
                self.pages_cache.insert(i, page);
            }
        }
        let mut result: Vec<&Page> = Vec::new();
        for i in 1..=count {
            result.push(self.pages_cache.get(&i).unwrap());
        }
        Ok(result)
    }

    fn extract_page(&mut self, page_number: usize) -> Result<Page> {
        let idx = page_number - 1;
        let (_, page_id) = self.page_ids[idx];

        // Get page dimensions
        let (width, height) = self.get_page_dimensions(page_id)?;

        // Calculate doctop offset
        let doctop_offset: f64 = (0..idx)
            .map(|i| {
                let (_, pid) = self.page_ids[i];
                self.get_page_dimensions(pid)
                    .map(|(_, h)| h)
                    .unwrap_or(0.0)
            })
            .sum();

        // Get page resources
        let resources = self.get_page_resources(page_id);

        // Create interpreter and process
        let mut interpreter =
            ContentStreamInterpreter::new(&self.inner, height, doctop_offset, &mut self.font_cache);

        interpreter.process_page(page_id, resources.as_ref())?;

        let mut page = Page::new(page_number, width, height, doctop_offset);
        page.chars = interpreter.chars;
        page.lines = interpreter.lines;
        page.rects = interpreter.rects;
        page.curves = interpreter.curves;

        Ok(page)
    }

    fn get_page_dimensions(&self, page_id: ObjectId) -> Result<(f64, f64)> {
        let page_obj = self
            .inner
            .get_object(page_id)
            .map_err(|e| Error::PdfParse(e.to_string()))?;

        let page_dict = page_obj
            .as_dict()
            .map_err(|_| Error::PdfParse("Page is not a dictionary".into()))?;

        // Try MediaBox, then fall back to CropBox
        let media_box = self.find_media_box(page_dict)?;

        let width = media_box[2] - media_box[0];
        let height = media_box[3] - media_box[1];

        Ok((width.abs(), height.abs()))
    }

    fn find_media_box(&self, page_dict: &lopdf::Dictionary) -> Result<[f64; 4]> {
        // Look for MediaBox directly on page
        if let Ok(mb) = page_dict.get(b"MediaBox") {
            if let Some(bbox) = self.parse_rect_array(mb) {
                return Ok(bbox);
            }
        }

        // Look for CropBox
        if let Ok(cb) = page_dict.get(b"CropBox") {
            if let Some(bbox) = self.parse_rect_array(cb) {
                return Ok(bbox);
            }
        }

        // Follow Parent chain
        if let Ok(parent_ref) = page_dict.get(b"Parent") {
            if let Object::Reference(id) = parent_ref {
                if let Ok(parent_obj) = self.inner.get_object(*id) {
                    if let Ok(parent_dict) = parent_obj.as_dict() {
                        return self.find_media_box(parent_dict);
                    }
                }
            }
        }

        // Default to US Letter
        Ok([0.0, 0.0, 612.0, 792.0])
    }

    fn parse_rect_array(&self, obj: &Object) -> Option<[f64; 4]> {
        let arr = match obj {
            Object::Array(a) => a.clone(),
            Object::Reference(id) => self
                .inner
                .get_object(*id)
                .ok()
                .and_then(|o| o.as_array().ok())
                .cloned()?,
            _ => return None,
        };

        if arr.len() >= 4 {
            Some([
                obj_to_f64(&arr[0])?,
                obj_to_f64(&arr[1])?,
                obj_to_f64(&arr[2])?,
                obj_to_f64(&arr[3])?,
            ])
        } else {
            None
        }
    }

    fn get_page_resources(&self, page_id: ObjectId) -> Option<lopdf::Dictionary> {
        let page_obj = self.inner.get_object(page_id).ok()?;
        let page_dict = page_obj.as_dict().ok()?;

        match page_dict.get(b"Resources") {
            Ok(Object::Dictionary(d)) => Some(d.clone()),
            Ok(Object::Reference(id)) => self
                .inner
                .get_object(*id)
                .ok()
                .and_then(|o| o.as_dict().ok())
                .cloned(),
            _ => {
                // Try parent
                if let Ok(Object::Reference(parent_id)) = page_dict.get(b"Parent") {
                    if let Ok(parent_obj) = self.inner.get_object(*parent_id) {
                        if let Ok(parent_dict) = parent_obj.as_dict() {
                            match parent_dict.get(b"Resources") {
                                Ok(Object::Dictionary(d)) => return Some(d.clone()),
                                Ok(Object::Reference(id)) => {
                                    return self
                                        .inner
                                        .get_object(*id)
                                        .ok()
                                        .and_then(|o| o.as_dict().ok())
                                        .cloned();
                                }
                                _ => {}
                            }
                        }
                    }
                }
                None
            }
        }
    }
}

fn obj_to_f64(obj: &Object) -> Option<f64> {
    match obj {
        Object::Integer(n) => Some(*n as f64),
        Object::Real(n) => Some(*n as f64),
        _ => None,
    }
}
