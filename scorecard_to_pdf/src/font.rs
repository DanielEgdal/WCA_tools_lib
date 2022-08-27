use font_kit::loaders::default::Font;
use printpdf::{IndirectFontRef, PdfDocumentReference};
use std::sync::Arc;

pub type FontWidth = Font;
pub type FontPDF = IndirectFontRef;

const NORMAL: &[u8] = include_bytes!("../dependencies/Helvetica.ttf");
const BOLD: &[u8] = include_bytes!("../dependencies/Helvetica-Bold.ttf");

pub fn load_fonts(doc: &PdfDocumentReference, weight: &str) -> (FontWidth, FontPDF) {
    let bytes = match weight {
        "normal" => NORMAL,
        "bold" => BOLD,
        _ => NORMAL
    };
    let font_width = Font::from_bytes(Arc::new(bytes.to_vec()), 0).unwrap();
    let font = doc.add_external_font(bytes).unwrap();

    (font_width, font)
}
