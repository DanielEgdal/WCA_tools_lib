use font_kit::font::Font;
use printpdf::{PdfDocumentReference, IndirectFontRef, PdfDocument, Point, Mm, PdfPageIndex, PdfLayerIndex, Line, PdfLayerReference, LineDashPattern, Color, Greyscale};

pub struct ScorecardGenerator<'a> {
    document: PdfDocumentReference,
    normal_font: IndirectFontRef,
    normal_font_width: Font,
    bold_font: IndirectFontRef,
    bold_font_width: Font,
    offset_x: f64,
    offset_y: f64,
    page: usize,
    pages: Vec<(PdfPageIndex, PdfLayerIndex)>,
    competition_name: &'a str,
}

pub enum Weight {
    Normal,
    Bold,
}

pub enum Alignment {
    Left,
    Center,
    Right,
}

impl<'a> ScorecardGenerator<'a> {
    pub fn new(competition_name: &'a str) -> ScorecardGenerator<'a> {
        let doc = PdfDocument::empty(competition_name);
        let (normal_font_width, normal_font) = crate::font::load_fonts(&doc, "normal");
        let (bold_font_width, bold_font) = crate::font::load_fonts(&doc, "bold");
        ScorecardGenerator { 
            document: doc, 
            normal_font, 
            normal_font_width, 
            bold_font, 
            bold_font_width, 
            offset_x: 0.0, 
            offset_y: 0.0, 
            page: 0, 
            pages: vec![],
            competition_name }
    }

    pub fn set_page(&mut self, page: usize) {
        while self.pages.len() <= page {
            let (page, layer) = self.document.add_page(Mm(210.0), Mm(297.0), "");
            self.pages.push((page, layer));

            let current_layer = self.document.get_page(page).get_layer(layer);
            let points1 = vec![(Point::new(Mm(105.0), Mm(0.0)), false),
                            (Point::new(Mm(105.0), Mm(297.0)), false)];
            let points2 = vec![(Point::new(Mm(0.0), Mm(99.0)), false),
                            (Point::new(Mm(210.0), Mm(99.0)), false)];
            let points3 = vec![(Point::new(Mm(0.0), Mm(198.0)), false),
                            (Point::new(Mm(210.0), Mm(198.0)), false)];
            let line1 = line_from_points(points1);
            let line2 = line_from_points(points2);
            let line3 = line_from_points(points3);
            let width = Some(5);
            let gap = Some(10);
            let dash_pattern = LineDashPattern::new(0, width, gap, width, gap, width, gap);
            let outline_color = Color::Greyscale(Greyscale::new(0.0, None));
            current_layer.set_overprint_stroke(true);
            current_layer.set_line_dash_pattern(dash_pattern);
            current_layer.set_outline_color(outline_color);
            current_layer.set_outline_thickness(0.5);
            current_layer.add_shape(line1);
            current_layer.add_shape(line2);
            current_layer.add_shape(line3);
            
            let dash_pattern = LineDashPattern::new(0, None, None, None, None, None, None);
            current_layer.set_line_dash_pattern(dash_pattern);
        }
        self.page = page;
    }

    pub fn set_position(&mut self, position: usize) {
        (self.offset_x, self.offset_y) = match position {
            0 => (0.0, 297.0),
            1 => (105.0, 297.0),
            2 => (0.0, 198.0),
            3 => (105.0, 198.0),
            4 => (0.0, 99.0),
            5 => (105.0, 99.0),
            _ => unreachable!()
        }
    }

    pub fn get_current_layer(&self) -> PdfLayerReference {
        let (page, layer) = self.pages[self.page];
        self.document.get_page(page).get_layer(layer)
    }

    pub fn draw_square(&mut self, x: f64, y: f64, width: f64, height: f64) {
        let (x1, y1) = (self.offset_x, self.offset_y);
        let points = vec![(Point::new(Mm(x + x1), Mm(y1 - y)), false),
            (Point::new(Mm(x + x1 + width), Mm(y1 - y)), false),
            (Point::new(Mm(x + x1 + width), Mm(y1 - y - height)), false),
            (Point::new(Mm(x + x1), Mm(y1 - y - height)), false)];
        let square = Line {
            points,
            is_closed: true,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        };
        let current_layer = self.get_current_layer();
        current_layer.add_shape(square);
    }

    pub fn write_multi_text(&mut self, mut x: f64, y: f64, font_size: f64, alignemnt: Alignment, strings: &[(&str, Weight)]) {
        let width_of_string = strings.iter().map(|(string, weight)| match weight {
            Weight::Normal => get_width_of_string(&self.normal_font_width, string, font_size),
            Weight::Bold => get_width_of_string(&self.bold_font_width, string, font_size),
        }).sum();
        x -= match alignemnt {
            Alignment::Left => 0.0,
            Alignment::Right => width_of_string,
            Alignment::Center => width_of_string / 2.0,
        };
        let current_layer = self.get_current_layer();
        current_layer.begin_text_section();
        current_layer.set_text_cursor(Mm(x + self.offset_x), Mm(self.offset_y - y));
        current_layer.set_line_height(12.0);
        for (string, weight) in strings {
            let font = match weight {
                Weight::Normal => &self.normal_font,
                Weight::Bold => &self.bold_font,
            };
            current_layer.set_font(font, font_size);
            current_layer.write_text(*string, font);
        }
        current_layer.end_text_section();

    }

    pub fn write(&mut self, string: &str, x: f64, y: f64, font_size: f64, alignemnt: Alignment, weight: Weight) {
        self.write_multi_text(x, y, font_size, alignemnt, &[(string, weight)]);
    }

    pub fn get_competition_name(&self) -> &str {
        &self.competition_name
    }

    pub fn doc(self) -> PdfDocumentReference {
        self.document
    }

    pub fn get_width_of_string(&self, string: &str, font_size: f64, weight: Weight) -> f64 {
        match weight {
            Weight::Normal => get_width_of_string(&self.normal_font_width, string, font_size),
            Weight::Bold => get_width_of_string(&self.bold_font_width, string, font_size),
        }
    }
}

pub fn get_width_of_string(font: &Font, string: &str, font_size: f64) -> f64 {
    let upem = font.metrics().units_per_em;
    let mut width = 0.0;
    for char in string.chars() {
        if !char.is_whitespace() {
            if let Some(id) = font.glyph_for_char(char) {
                let glyph_width = font.advance(id).unwrap().x();
                width += glyph_width
            }
        } else {
            width += upem as f32 / 4.0;
        }
    }
    (width as f64 / (upem as f64 / font_size)) / 2.83
}

fn line_from_points(points: Vec<(Point, bool)>) -> Line {
    Line {
        points,
        is_closed: false,
        has_fill: false,
        has_stroke: true,
        is_clipping_path: false,
    }
}