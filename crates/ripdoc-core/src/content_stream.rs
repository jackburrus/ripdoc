use lopdf::content::{Content, Operation};
use lopdf::{Document, Object, ObjectId};

use crate::error::{Error, Result};
use std::sync::Arc;

use crate::fonts::{FontCache, FontInfo};
use crate::geometry::ctm::Matrix;
use crate::objects::*;

/// Interprets a PDF content stream and extracts positioned characters,
/// lines, rectangles, and curves.
pub struct ContentStreamInterpreter<'a> {
    doc: &'a Document,
    page_height: f64,
    /// Accumulated doctop offset from previous pages.
    doctop_offset: f64,
    fonts: &'a mut FontCache,

    // Graphics state stack
    graphics_stack: Vec<GraphicsState>,
    gs: GraphicsState,

    // Text state
    ts: TextState,
    in_text: bool,

    // Path construction
    path: Vec<PathSegment>,
    current_point: Option<(f64, f64)>,

    // Extracted objects
    pub chars: Vec<Char>,
    pub lines: Vec<Line>,
    pub rects: Vec<Rect>,
    pub curves: Vec<Curve>,
}

#[derive(Debug, Clone)]
enum PathSegment {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    CurveTo(f64, f64, f64, f64, f64, f64),
    ClosePath,
    Rect(f64, f64, f64, f64),
}

impl<'a> ContentStreamInterpreter<'a> {
    pub fn new(
        doc: &'a Document,
        page_height: f64,
        doctop_offset: f64,
        fonts: &'a mut FontCache,
    ) -> Self {
        Self {
            doc,
            page_height,
            doctop_offset,
            fonts,
            graphics_stack: Vec::new(),
            gs: GraphicsState::default(),
            ts: TextState::default(),
            in_text: false,
            path: Vec::new(),
            current_point: None,
            chars: Vec::new(),
            lines: Vec::new(),
            rects: Vec::new(),
            curves: Vec::new(),
        }
    }

    /// Process a page's content stream(s).
    pub fn process_page(
        &mut self,
        page_id: ObjectId,
        page_resources: Option<&lopdf::Dictionary>,
    ) -> Result<()> {
        // Load fonts from page resources
        if let Some(resources) = page_resources {
            self.load_fonts(resources)?;
        }

        // Get page content
        let content_data = self
            .doc
            .get_page_content(page_id)
            .map_err(|e| Error::ContentStream(format!("Failed to get page content: {}", e)))?;

        let content = Content::decode(&content_data)
            .map_err(|e| Error::ContentStream(format!("Failed to decode content stream: {}", e)))?;

        for op in &content.operations {
            self.process_operation(op, page_resources)?;
        }

        Ok(())
    }

    fn load_fonts(&mut self, resources: &lopdf::Dictionary) -> Result<()> {
        let font_dict = match resources.get(b"Font") {
            Ok(obj) => match obj {
                Object::Dictionary(d) => Some(d.clone()),
                Object::Reference(id) => self
                    .doc
                    .get_object(*id)
                    .ok()
                    .and_then(|o| o.as_dict().ok())
                    .cloned(),
                _ => None,
            },
            Err(_) => None,
        };

        if let Some(font_dict) = font_dict {
            for (name, obj) in font_dict.iter() {
                let font_name = String::from_utf8_lossy(name).to_string();
                if self.fonts.contains(&font_name) {
                    continue;
                }

                let font_obj_dict = match obj {
                    Object::Reference(id) => self
                        .doc
                        .get_object(*id)
                        .ok()
                        .and_then(|o| o.as_dict().ok()),
                    Object::Dictionary(d) => Some(d),
                    _ => None,
                };

                if let Some(fd) = font_obj_dict {
                    match crate::fonts::resolve_font(self.doc, &font_name, fd) {
                        Ok(font_info) => {
                            self.fonts.insert(font_name, font_info);
                        }
                        Err(e) => {
                            log::warn!("Failed to resolve font {}: {}", font_name, e);
                            // Insert a default font so we don't try again
                            self.fonts.insert(
                                font_name.clone(),
                                FontInfo {
                                    name: font_name,
                                    ..FontInfo::default()
                                },
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn process_operation(
        &mut self,
        op: &Operation,
        page_resources: Option<&lopdf::Dictionary>,
    ) -> Result<()> {
        match op.operator.as_ref() {
            // Graphics state
            "q" => self.op_save_state(),
            "Q" => self.op_restore_state(),
            "cm" => self.op_concat_matrix(&op.operands),
            "w" => self.op_set_line_width(&op.operands),
            "J" => self.op_set_line_cap(&op.operands),
            "j" => self.op_set_line_join(&op.operands),
            "M" => self.op_set_miter_limit(&op.operands),
            "d" => self.op_set_dash(&op.operands),
            "gs" => self.op_set_graphics_state(&op.operands, page_resources),

            // Color operators
            "CS" => self.op_set_stroking_colorspace(&op.operands),
            "cs" => self.op_set_nonstroking_colorspace(&op.operands),
            "SC" | "SCN" => self.op_set_stroking_color(&op.operands),
            "sc" | "scn" => self.op_set_nonstroking_color(&op.operands),
            "G" => self.op_set_gray_stroke(&op.operands),
            "g" => self.op_set_gray_fill(&op.operands),
            "RG" => self.op_set_rgb_stroke(&op.operands),
            "rg" => self.op_set_rgb_fill(&op.operands),
            "K" => self.op_set_cmyk_stroke(&op.operands),
            "k" => self.op_set_cmyk_fill(&op.operands),

            // Text state
            "BT" => self.op_begin_text(),
            "ET" => self.op_end_text(),
            "Tf" => self.op_set_font(&op.operands),
            "Tc" => self.op_set_char_spacing(&op.operands),
            "Tw" => self.op_set_word_spacing(&op.operands),
            "Tz" => self.op_set_horizontal_scaling(&op.operands),
            "TL" => self.op_set_leading(&op.operands),
            "Tr" => self.op_set_render_mode(&op.operands),
            "Ts" => self.op_set_rise(&op.operands),

            // Text positioning
            "Td" => self.op_text_move(&op.operands),
            "TD" => self.op_text_move_set_leading(&op.operands),
            "Tm" => self.op_set_text_matrix(&op.operands),
            "T*" => self.op_text_next_line(),

            // Text showing
            "Tj" => self.op_show_text(&op.operands),
            "TJ" => self.op_show_text_adjusted(&op.operands),
            "'" => self.op_move_show_text(&op.operands),
            "\"" => self.op_move_set_show_text(&op.operands),

            // Path construction
            "m" => self.op_path_move(&op.operands),
            "l" => self.op_path_line(&op.operands),
            "c" => self.op_path_curve(&op.operands),
            "v" => self.op_path_curve_v(&op.operands),
            "y" => self.op_path_curve_y(&op.operands),
            "h" => self.op_path_close(),
            "re" => self.op_path_rect(&op.operands),

            // Path painting
            "S" => self.op_stroke(),
            "s" => {
                self.op_path_close();
                self.op_stroke();
            }
            "f" | "F" => self.op_fill(),
            "f*" => self.op_fill(),
            "B" | "B*" => {
                self.op_fill();
                self.op_stroke();
            }
            "b" | "b*" => {
                self.op_path_close();
                self.op_fill();
                self.op_stroke();
            }
            "n" => self.op_end_path(),

            // XObject (Form)
            "Do" => self.op_do_xobject(&op.operands, page_resources)?,

            _ => {} // Ignore unknown operators
        }

        Ok(())
    }

    // === Graphics State Operators ===

    fn op_save_state(&mut self) {
        self.graphics_stack.push(self.gs.clone());
    }

    fn op_restore_state(&mut self) {
        if let Some(gs) = self.graphics_stack.pop() {
            self.gs = gs;
        }
    }

    fn op_concat_matrix(&mut self, operands: &[Object]) {
        if operands.len() >= 6 {
            let m = Matrix::new(
                get_number(&operands[0]),
                get_number(&operands[1]),
                get_number(&operands[2]),
                get_number(&operands[3]),
                get_number(&operands[4]),
                get_number(&operands[5]),
            );
            self.gs.ctm = m.multiply(&self.gs.ctm);
        }
    }

    fn op_set_line_width(&mut self, operands: &[Object]) {
        if let Some(w) = operands.first() {
            self.gs.line_width = get_number(w);
        }
    }

    fn op_set_line_cap(&mut self, operands: &[Object]) {
        if let Some(j) = operands.first() {
            self.gs.line_cap = get_number(j) as i32;
        }
    }

    fn op_set_line_join(&mut self, operands: &[Object]) {
        if let Some(j) = operands.first() {
            self.gs.line_join = get_number(j) as i32;
        }
    }

    fn op_set_miter_limit(&mut self, operands: &[Object]) {
        if let Some(m) = operands.first() {
            self.gs.miter_limit = get_number(m);
        }
    }

    fn op_set_dash(&mut self, operands: &[Object]) {
        if operands.len() >= 2 {
            if let Ok(arr) = operands[0].as_array() {
                self.gs.dash_pattern = arr.iter().map(get_number).collect();
            }
            self.gs.dash_phase = get_number(&operands[1]);
        }
    }

    fn op_set_graphics_state(
        &mut self,
        operands: &[Object],
        page_resources: Option<&lopdf::Dictionary>,
    ) {
        if let (Some(name_obj), Some(resources)) = (operands.first(), page_resources) {
            let gs_name = match name_obj {
                Object::Name(n) => n.clone(),
                _ => return,
            };

            // Look up ExtGState in resources
            let ext_gstate = match resources.get(b"ExtGState") {
                Ok(Object::Dictionary(d)) => Some(d),
                Ok(Object::Reference(id)) => self
                    .doc
                    .get_object(*id)
                    .ok()
                    .and_then(|o| o.as_dict().ok()),
                _ => None,
            };

            if let Some(ext_gs) = ext_gstate {
                let gs_dict = match ext_gs.get(&gs_name) {
                    Ok(Object::Dictionary(d)) => Some(d),
                    Ok(Object::Reference(id)) => self
                        .doc
                        .get_object(*id)
                        .ok()
                        .and_then(|o| o.as_dict().ok()),
                    _ => None,
                };

                if let Some(gs) = gs_dict {
                    if let Ok(lw) = gs.get(b"LW") {
                        self.gs.line_width = get_number(lw);
                    }
                    if let Ok(lc) = gs.get(b"LC") {
                        self.gs.line_cap = get_number(lc) as i32;
                    }
                    if let Ok(lj) = gs.get(b"LJ") {
                        self.gs.line_join = get_number(lj) as i32;
                    }
                    // Font from graphics state
                    if let Ok(font) = gs.get(b"Font") {
                        if let Ok(arr) = font.as_array() {
                            if arr.len() >= 2 {
                                if let Ok(name) = arr[0].as_name() {
                                    self.ts.font_name =
                                        String::from_utf8_lossy(name).to_string();
                                }
                                self.ts.font_size = get_number(&arr[1]);
                            }
                        }
                    }
                }
            }
        }
    }

    // === Color Operators ===

    fn op_set_stroking_colorspace(&mut self, operands: &[Object]) {
        if let Some(Object::Name(name)) = operands.first() {
            self.gs.stroking_colorspace = String::from_utf8_lossy(name).to_string();
        }
    }

    fn op_set_nonstroking_colorspace(&mut self, operands: &[Object]) {
        if let Some(Object::Name(name)) = operands.first() {
            self.gs.non_stroking_colorspace = String::from_utf8_lossy(name).to_string();
        }
    }

    fn op_set_stroking_color(&mut self, operands: &[Object]) {
        self.gs.stroking_color = Arc::new(parse_color(operands, &self.gs.stroking_colorspace));
    }

    fn op_set_nonstroking_color(&mut self, operands: &[Object]) {
        self.gs.non_stroking_color = Arc::new(parse_color(operands, &self.gs.non_stroking_colorspace));
    }

    fn op_set_gray_stroke(&mut self, operands: &[Object]) {
        if let Some(g) = operands.first() {
            self.gs.stroking_color = Arc::new(Some(Color::Gray(get_number(g))));
            self.gs.stroking_colorspace = "DeviceGray".into();
        }
    }

    fn op_set_gray_fill(&mut self, operands: &[Object]) {
        if let Some(g) = operands.first() {
            self.gs.non_stroking_color = Arc::new(Some(Color::Gray(get_number(g))));
            self.gs.non_stroking_colorspace = "DeviceGray".into();
        }
    }

    fn op_set_rgb_stroke(&mut self, operands: &[Object]) {
        if operands.len() >= 3 {
            self.gs.stroking_color = Arc::new(Some(Color::RGB(
                get_number(&operands[0]),
                get_number(&operands[1]),
                get_number(&operands[2]),
            )));
            self.gs.stroking_colorspace = "DeviceRGB".into();
        }
    }

    fn op_set_rgb_fill(&mut self, operands: &[Object]) {
        if operands.len() >= 3 {
            self.gs.non_stroking_color = Arc::new(Some(Color::RGB(
                get_number(&operands[0]),
                get_number(&operands[1]),
                get_number(&operands[2]),
            )));
            self.gs.non_stroking_colorspace = "DeviceRGB".into();
        }
    }

    fn op_set_cmyk_stroke(&mut self, operands: &[Object]) {
        if operands.len() >= 4 {
            self.gs.stroking_color = Arc::new(Some(Color::CMYK(
                get_number(&operands[0]),
                get_number(&operands[1]),
                get_number(&operands[2]),
                get_number(&operands[3]),
            )));
            self.gs.stroking_colorspace = "DeviceCMYK".into();
        }
    }

    fn op_set_cmyk_fill(&mut self, operands: &[Object]) {
        if operands.len() >= 4 {
            self.gs.non_stroking_color = Arc::new(Some(Color::CMYK(
                get_number(&operands[0]),
                get_number(&operands[1]),
                get_number(&operands[2]),
                get_number(&operands[3]),
            )));
            self.gs.non_stroking_colorspace = "DeviceCMYK".into();
        }
    }

    // === Text State Operators ===

    fn op_begin_text(&mut self) {
        self.in_text = true;
        self.ts.text_matrix = Matrix::identity();
        self.ts.text_line_matrix = Matrix::identity();
    }

    fn op_end_text(&mut self) {
        self.in_text = false;
    }

    fn op_set_font(&mut self, operands: &[Object]) {
        if operands.len() >= 2 {
            if let Ok(name) = operands[0].as_name() {
                self.ts.font_name = String::from_utf8_lossy(name).to_string();
            }
            self.ts.font_size = get_number(&operands[1]);
        }
    }

    fn op_set_char_spacing(&mut self, operands: &[Object]) {
        if let Some(s) = operands.first() {
            self.ts.char_spacing = get_number(s);
        }
    }

    fn op_set_word_spacing(&mut self, operands: &[Object]) {
        if let Some(s) = operands.first() {
            self.ts.word_spacing = get_number(s);
        }
    }

    fn op_set_horizontal_scaling(&mut self, operands: &[Object]) {
        if let Some(s) = operands.first() {
            self.ts.horizontal_scaling = get_number(s);
        }
    }

    fn op_set_leading(&mut self, operands: &[Object]) {
        if let Some(l) = operands.first() {
            self.ts.leading = get_number(l);
        }
    }

    fn op_set_render_mode(&mut self, operands: &[Object]) {
        if let Some(r) = operands.first() {
            self.ts.render_mode = get_number(r) as i32;
        }
    }

    fn op_set_rise(&mut self, operands: &[Object]) {
        if let Some(r) = operands.first() {
            self.ts.rise = get_number(r);
        }
    }

    // === Text Positioning Operators ===

    fn op_text_move(&mut self, operands: &[Object]) {
        if operands.len() >= 2 {
            let tx = get_number(&operands[0]);
            let ty = get_number(&operands[1]);
            let m = Matrix::translate(tx, ty);
            self.ts.text_line_matrix = m.multiply(&self.ts.text_line_matrix);
            self.ts.text_matrix = self.ts.text_line_matrix;
        }
    }

    fn op_text_move_set_leading(&mut self, operands: &[Object]) {
        if operands.len() >= 2 {
            let ty = get_number(&operands[1]);
            self.ts.leading = -ty;
            self.op_text_move(operands);
        }
    }

    fn op_set_text_matrix(&mut self, operands: &[Object]) {
        if operands.len() >= 6 {
            let m = Matrix::new(
                get_number(&operands[0]),
                get_number(&operands[1]),
                get_number(&operands[2]),
                get_number(&operands[3]),
                get_number(&operands[4]),
                get_number(&operands[5]),
            );
            self.ts.text_matrix = m;
            self.ts.text_line_matrix = m;
        }
    }

    fn op_text_next_line(&mut self) {
        let m = Matrix::translate(0.0, -self.ts.leading);
        self.ts.text_line_matrix = m.multiply(&self.ts.text_line_matrix);
        self.ts.text_matrix = self.ts.text_line_matrix;
    }

    // === Text Showing Operators ===

    fn op_show_text(&mut self, operands: &[Object]) {
        if let Some(obj) = operands.first() {
            let bytes = match obj {
                Object::String(s, _) => s.clone(),
                _ => return,
            };
            self.render_text(&bytes);
        }
    }

    fn op_show_text_adjusted(&mut self, operands: &[Object]) {
        if let Some(obj) = operands.first() {
            let array = match obj {
                Object::Array(arr) => arr,
                _ => return,
            };

            for item in array {
                match item {
                    Object::String(s, _) => {
                        self.render_text(s);
                    }
                    Object::Integer(n) => {
                        // Adjust position: negative number = move right, positive = move left
                        let adjustment = *n as f64;
                        self.adjust_text_position(adjustment);
                    }
                    Object::Real(n) => {
                        self.adjust_text_position(*n as f64);
                    }
                    _ => {}
                }
            }
        }
    }

    fn op_move_show_text(&mut self, operands: &[Object]) {
        self.op_text_next_line();
        self.op_show_text(operands);
    }

    fn op_move_set_show_text(&mut self, operands: &[Object]) {
        if operands.len() >= 3 {
            self.ts.word_spacing = get_number(&operands[0]);
            self.ts.char_spacing = get_number(&operands[1]);
            self.op_text_next_line();
            self.op_show_text(&operands[2..]);
        }
    }

    /// Adjust text position for TJ kerning adjustments.
    fn adjust_text_position(&mut self, adjustment: f64) {
        // In PDF, positive adjustment moves LEFT (reduces space)
        // tx = -(adjustment / 1000) * fontSize * horizontalScaling/100
        let tx = -(adjustment / 1000.0) * self.ts.font_size * (self.ts.horizontal_scaling / 100.0);
        let translate = Matrix::translate(tx, 0.0);
        self.ts.text_matrix = translate.multiply(&self.ts.text_matrix);
    }

    /// Render text bytes using current font and text state.
    fn render_text(&mut self, bytes: &[u8]) {
        let font_info: Arc<FontInfo> = match self.fonts.get(&self.ts.font_name) {
            Some(f) => f,
            None => Arc::new(FontInfo::default()),
        };

        let decoded = font_info.decode_string(bytes);
        let h_scale = self.ts.horizontal_scaling / 100.0;

        for (code, text) in decoded {
            // Compute the text rendering matrix: Trm = Tm × CTM
            // But first apply font size and horizontal scaling
            let font_matrix = Matrix::new(
                self.ts.font_size * h_scale,
                0.0,
                0.0,
                self.ts.font_size,
                0.0,
                self.ts.rise,
            );
            let trm = font_matrix.multiply(&self.ts.text_matrix).multiply(&self.gs.ctm);

            // Character position in user space
            let (x, y) = (trm.e, trm.f);

            // Character width in text space
            let w0 = font_info.char_width(code) / 1000.0;

            // Actual displacement in user space
            let tx = (w0 * self.ts.font_size + self.ts.char_spacing) * h_scale;

            // Add word spacing for space characters
            let tx = if text == " " {
                tx + self.ts.word_spacing * h_scale
            } else {
                tx
            };

            // Convert PDF coordinates (origin at bottom-left) to pdfplumber coordinates (origin at top-left).
            // In PDF, y is the text baseline. Glyphs extend above the baseline by ~font_size.
            // So in top-left coords: bottom ≈ baseline, top ≈ baseline - font_size.
            let effective_size = trm.font_size();
            let bottom = self.page_height - y;
            let top = bottom - effective_size;
            let x0 = x;
            // Character width in user space coordinates
            let char_width_user = w0 * effective_size * h_scale;
            let x1 = x0 + char_width_user;

            let upright = trm.is_upright();

            let ch = Char {
                text: text.clone(),
                fontname: font_info.base_font.clone(),
                size: effective_size,
                x0,
                x1,
                top,
                bottom,
                doctop: top + self.doctop_offset,
                matrix: trm.as_array(),
                upright,
                stroking_color: self.gs.stroking_color.clone(),
                non_stroking_color: self.gs.non_stroking_color.clone(),
                adv: tx,
            };

            self.chars.push(ch);

            // Advance text position
            let advance = Matrix::translate(tx, 0.0);
            self.ts.text_matrix = advance.multiply(&self.ts.text_matrix);
        }
    }

    // === Path Construction Operators ===

    fn op_path_move(&mut self, operands: &[Object]) {
        if operands.len() >= 2 {
            let x = get_number(&operands[0]);
            let y = get_number(&operands[1]);
            self.path.push(PathSegment::MoveTo(x, y));
            self.current_point = Some((x, y));
        }
    }

    fn op_path_line(&mut self, operands: &[Object]) {
        if operands.len() >= 2 {
            let x = get_number(&operands[0]);
            let y = get_number(&operands[1]);
            self.path.push(PathSegment::LineTo(x, y));
            self.current_point = Some((x, y));
        }
    }

    fn op_path_curve(&mut self, operands: &[Object]) {
        if operands.len() >= 6 {
            let x1 = get_number(&operands[0]);
            let y1 = get_number(&operands[1]);
            let x2 = get_number(&operands[2]);
            let y2 = get_number(&operands[3]);
            let x3 = get_number(&operands[4]);
            let y3 = get_number(&operands[5]);
            self.path
                .push(PathSegment::CurveTo(x1, y1, x2, y2, x3, y3));
            self.current_point = Some((x3, y3));
        }
    }

    fn op_path_curve_v(&mut self, operands: &[Object]) {
        if operands.len() >= 4 {
            if let Some((cx, cy)) = self.current_point {
                let x2 = get_number(&operands[0]);
                let y2 = get_number(&operands[1]);
                let x3 = get_number(&operands[2]);
                let y3 = get_number(&operands[3]);
                self.path
                    .push(PathSegment::CurveTo(cx, cy, x2, y2, x3, y3));
                self.current_point = Some((x3, y3));
            }
        }
    }

    fn op_path_curve_y(&mut self, operands: &[Object]) {
        if operands.len() >= 4 {
            let x1 = get_number(&operands[0]);
            let y1 = get_number(&operands[1]);
            let x3 = get_number(&operands[2]);
            let y3 = get_number(&operands[3]);
            self.path
                .push(PathSegment::CurveTo(x1, y1, x3, y3, x3, y3));
            self.current_point = Some((x3, y3));
        }
    }

    fn op_path_close(&mut self) {
        self.path.push(PathSegment::ClosePath);
    }

    fn op_path_rect(&mut self, operands: &[Object]) {
        if operands.len() >= 4 {
            let x = get_number(&operands[0]);
            let y = get_number(&operands[1]);
            let w = get_number(&operands[2]);
            let h = get_number(&operands[3]);
            self.path.push(PathSegment::Rect(x, y, w, h));
        }
    }

    // === Path Painting Operators ===

    fn op_stroke(&mut self) {
        self.extract_path_objects(true, false);
        self.path.clear();
        self.current_point = None;
    }

    fn op_fill(&mut self) {
        self.extract_path_objects(false, true);
        self.path.clear();
        self.current_point = None;
    }

    fn op_end_path(&mut self) {
        self.path.clear();
        self.current_point = None;
    }

    /// Extract geometric objects from the current path.
    fn extract_path_objects(&mut self, stroke: bool, fill: bool) {
        let path = std::mem::take(&mut self.path);
        let ctm = self.gs.ctm;
        let mut current = (0.0f64, 0.0f64);
        let mut subpath_start = (0.0f64, 0.0f64);

        for segment in &path {
            match segment {
                PathSegment::MoveTo(x, y) => {
                    let (tx, ty) = ctm.transform_point(*x, *y);
                    current = (tx, ty);
                    subpath_start = current;
                }
                PathSegment::LineTo(x, y) => {
                    let (tx, ty) = ctm.transform_point(*x, *y);
                    if stroke {
                        self.add_line(current.0, current.1, tx, ty);
                    }
                    current = (tx, ty);
                }
                PathSegment::CurveTo(x1, y1, x2, y2, x3, y3) => {
                    let (tx1, ty1) = ctm.transform_point(*x1, *y1);
                    let (tx2, ty2) = ctm.transform_point(*x2, *y2);
                    let (tx3, ty3) = ctm.transform_point(*x3, *y3);

                    let curve = Curve {
                        points: vec![
                            self.to_page_coords(current.0, current.1),
                            self.to_page_coords(tx1, ty1),
                            self.to_page_coords(tx2, ty2),
                            self.to_page_coords(tx3, ty3),
                        ],
                        width: self.gs.line_width,
                        stroking_color: if stroke {
                            self.gs.stroking_color.clone()
                        } else {
                            Arc::new(None)
                        },
                        non_stroking_color: if fill {
                            self.gs.non_stroking_color.clone()
                        } else {
                            Arc::new(None)
                        },
                    };
                    self.curves.push(curve);
                    current = (tx3, ty3);
                }
                PathSegment::ClosePath => {
                    if stroke
                        && (current.0 - subpath_start.0).abs() > 1e-6
                        && (current.1 - subpath_start.1).abs() > 1e-6
                    {
                        self.add_line(current.0, current.1, subpath_start.0, subpath_start.1);
                    }
                    current = subpath_start;
                }
                PathSegment::Rect(x, y, w, h) => {
                    let (x0, y0) = ctm.transform_point(*x, *y);
                    let (x1, y1) = ctm.transform_point(*x + *w, *y + *h);

                    let top = self.page_height - y0.max(y1);
                    let bottom = self.page_height - y0.min(y1);
                    let left = x0.min(x1);
                    let right = x0.max(x1);

                    let rect = Rect {
                        x0: left,
                        top,
                        x1: right,
                        bottom,
                        width: (right - left).abs(),
                        height: (bottom - top).abs(),
                        linewidth: self.gs.line_width,
                        stroking_color: if stroke {
                            self.gs.stroking_color.clone()
                        } else {
                            Arc::new(None)
                        },
                        non_stroking_color: if fill {
                            self.gs.non_stroking_color.clone()
                        } else {
                            Arc::new(None)
                        },
                    };
                    self.rects.push(rect);
                }
            }
        }
    }

    fn add_line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        let (px0, py0) = self.to_page_coords(x0, y0);
        let (px1, py1) = self.to_page_coords(x1, y1);

        let top = py0.min(py1);
        let bottom = py0.max(py1);

        let line = Line {
            x0: px0,
            y0: py0,
            x1: px1,
            y1: py1,
            top,
            bottom,
            width: self.gs.line_width,
            stroking_color: self.gs.stroking_color.clone(),
            non_stroking_color: self.gs.non_stroking_color.clone(),
        };
        self.lines.push(line);
    }

    fn to_page_coords(&self, x: f64, y: f64) -> (f64, f64) {
        (x, self.page_height - y)
    }

    // === XObject Operator ===

    fn op_do_xobject(
        &mut self,
        operands: &[Object],
        page_resources: Option<&lopdf::Dictionary>,
    ) -> Result<()> {
        let name = match operands.first() {
            Some(Object::Name(n)) => n.clone(),
            _ => return Ok(()),
        };

        let xobject_dict = match page_resources {
            Some(r) => match r.get(b"XObject") {
                Ok(Object::Dictionary(d)) => Some(d),
                Ok(Object::Reference(id)) => self
                    .doc
                    .get_object(*id)
                    .ok()
                    .and_then(|o| o.as_dict().ok()),
                _ => None,
            },
            None => None,
        };

        let xobject_ref = match xobject_dict {
            Some(d) => match d.get(&name) {
                Ok(Object::Reference(id)) => Some(*id),
                _ => None,
            },
            None => None,
        };

        if let Some(xobj_id) = xobject_ref {
            if let Ok(Object::Stream(stream)) = self.doc.get_object(xobj_id) {
                let subtype = stream
                    .dict
                    .get(b"Subtype")
                    .ok()
                    .and_then(|o| o.as_name().ok())
                    .unwrap_or(b"");

                if subtype == b"Form" {
                    // Process Form XObject: save state, apply matrix, process content, restore
                    self.op_save_state();

                    // Apply form matrix if present
                    if let Ok(matrix_obj) = stream.dict.get(b"Matrix") {
                        if let Ok(arr) = matrix_obj.as_array() {
                            if arr.len() >= 6 {
                                let m = Matrix::new(
                                    get_number(&arr[0]),
                                    get_number(&arr[1]),
                                    get_number(&arr[2]),
                                    get_number(&arr[3]),
                                    get_number(&arr[4]),
                                    get_number(&arr[5]),
                                );
                                self.gs.ctm = m.multiply(&self.gs.ctm);
                            }
                        }
                    }

                    // Load resources from form XObject
                    let form_resources = stream
                        .dict
                        .get(b"Resources")
                        .ok()
                        .and_then(|o| match o {
                            Object::Dictionary(d) => Some(d.clone()),
                            Object::Reference(id) => self
                                .doc
                                .get_object(*id)
                                .ok()
                                .and_then(|o| o.as_dict().ok())
                                .cloned(),
                            _ => None,
                        });

                    let form_res = form_resources.as_ref().or(page_resources);
                    if let Some(r) = form_res {
                        let _ = self.load_fonts(r);
                    }

                    // Decode and process form content
                    let mut stream_clone = stream.clone();
                    let _ = stream_clone.decompress();
                    let content_bytes = &stream_clone.content;

                    if let Ok(content) = Content::decode(content_bytes) {
                        for op in &content.operations {
                            let _ = self.process_operation(op, form_res);
                        }
                    }

                    self.op_restore_state();
                }
            }
        }

        Ok(())
    }
}

/// Parse color from operands based on current colorspace.
fn parse_color(operands: &[Object], colorspace: &str) -> Option<Color> {
    match colorspace {
        "DeviceGray" | "CalGray" if !operands.is_empty() => {
            Some(Color::Gray(get_number(&operands[0])))
        }
        "DeviceRGB" | "CalRGB" if operands.len() >= 3 => Some(Color::RGB(
            get_number(&operands[0]),
            get_number(&operands[1]),
            get_number(&operands[2]),
        )),
        "DeviceCMYK" if operands.len() >= 4 => Some(Color::CMYK(
            get_number(&operands[0]),
            get_number(&operands[1]),
            get_number(&operands[2]),
            get_number(&operands[3]),
        )),
        _ => {
            // For unknown colorspaces, try to guess from operand count
            match operands.len() {
                1 => Some(Color::Gray(get_number(&operands[0]))),
                3 => Some(Color::RGB(
                    get_number(&operands[0]),
                    get_number(&operands[1]),
                    get_number(&operands[2]),
                )),
                4 => Some(Color::CMYK(
                    get_number(&operands[0]),
                    get_number(&operands[1]),
                    get_number(&operands[2]),
                    get_number(&operands[3]),
                )),
                _ => None,
            }
        }
    }
}

/// Extract a number (f64) from a PDF Object.
fn get_number(obj: &Object) -> f64 {
    match obj {
        Object::Integer(n) => *n as f64,
        Object::Real(n) => *n as f64,
        _ => 0.0,
    }
}
