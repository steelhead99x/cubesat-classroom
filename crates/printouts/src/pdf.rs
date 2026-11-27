//! A small PDF writer: one or more pages of text, lines and filled
//! rectangles in the base-14 fonts, with link annotations. Output is
//! deterministic (no dates or random IDs), so a rebuilt file only changes when
//! its content does.

use std::fmt::Write as _;

use crate::metrics::{HELVETICA, HELVETICA_BOLD};

/// The three faces a handout uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    /// Helvetica.
    Sans,
    /// Helvetica-Bold.
    Bold,
    /// Courier.
    Mono,
}

impl Face {
    fn resource(self) -> &'static str {
        match self {
            Face::Sans => "F1",
            Face::Bold => "F2",
            Face::Mono => "F3",
        }
    }
}

/// An RGB colour, each channel 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb(pub f64, pub f64, pub f64);

impl Rgb {
    /// From `0xRRGGBB`.
    #[must_use]
    pub fn hex(v: u32) -> Self {
        let c = |shift: u32| f64::from((v >> shift) & 0xff) / 255.0;
        Rgb(c(16), c(8), c(0))
    }
}

/// Map a character to its WinAnsi code, or `?` when it has none.
#[must_use]
pub fn win_ansi(ch: char) -> u8 {
    match ch {
        ' '..='~' => ch as u8,
        '\u{a0}'..='\u{ff}' => ch as u32 as u8,
        '€' => 0x80,
        '‚' => 0x82,
        '„' => 0x84,
        '…' => 0x85,
        '‘' => 0x91,
        '’' => 0x92,
        '“' => 0x93,
        '”' => 0x94,
        '•' => 0x95,
        '–' => 0x96,
        '—' => 0x97,
        '™' => 0x99,
        _ => b'?',
    }
}

/// Width of `text` in points.
#[must_use]
pub fn text_width(text: &str, face: Face, size: f64) -> f64 {
    let units: u32 = text
        .chars()
        .map(|ch| {
            let code = win_ansi(ch);
            match face {
                Face::Mono => 600,
                Face::Sans => u32::from(HELVETICA[usize::from(code.max(32) - 32)]),
                Face::Bold => u32::from(HELVETICA_BOLD[usize::from(code.max(32) - 32)]),
            }
        })
        .sum();
    f64::from(units) * size / 1000.0
}

fn pdf_string(text: &str) -> String {
    let mut out = String::from("(");
    for ch in text.chars() {
        match win_ansi(ch) {
            b'(' => out.push_str("\\("),
            b')' => out.push_str("\\)"),
            b'\\' => out.push_str("\\\\"),
            c @ 32..=126 => out.push(c as char),
            c => {
                let _ = write!(out, "\\{c:03o}");
            }
        }
    }
    out.push(')');
    out
}

/// Format a number for a content stream: up to 3 decimals, no trailing zeros.
fn n(v: f64) -> String {
    let text = format!("{v:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" {
        "0".into()
    } else {
        text.into()
    }
}

/// One page being drawn. Coordinates are points from the bottom-left corner.
#[derive(Default)]
pub struct Page {
    content: String,
    links: Vec<([f64; 4], String)>,
}

impl Page {
    /// Draw `text` with its baseline starting at (x, y).
    pub fn text(&mut self, x: f64, y: f64, text: &str, face: Face, size: f64, color: Rgb) {
        let _ = writeln!(
            self.content,
            "BT /{} {} Tf {} {} {} rg {} {} Td {} Tj ET",
            face.resource(),
            n(size),
            n(color.0),
            n(color.1),
            n(color.2),
            n(x),
            n(y),
            pdf_string(text)
        );
    }

    /// A straight line.
    pub fn line(&mut self, from: [f64; 2], to: [f64; 2], width: f64, color: Rgb) {
        let _ = writeln!(
            self.content,
            "{} {} {} RG {} w {} {} m {} {} l S",
            n(color.0),
            n(color.1),
            n(color.2),
            n(width),
            n(from[0]),
            n(from[1]),
            n(to[0]),
            n(to[1])
        );
    }

    /// A rectangle, filled and/or stroked.
    pub fn rect(&mut self, xywh: [f64; 4], fill: Option<Rgb>, stroke: Option<(Rgb, f64)>) {
        let [x, y, w, h] = xywh;
        if let Some(c) = fill {
            let _ = writeln!(
                self.content,
                "{} {} {} rg {} {} {} {} re f",
                n(c.0),
                n(c.1),
                n(c.2),
                n(x),
                n(y),
                n(w),
                n(h)
            );
        }
        if let Some((c, width)) = stroke {
            let _ = writeln!(
                self.content,
                "{} {} {} RG {} w {} {} {} {} re S",
                n(c.0),
                n(c.1),
                n(c.2),
                n(width),
                n(x),
                n(y),
                n(w),
                n(h)
            );
        }
    }

    /// A clickable area that opens `uri`. `rect` is [x0, y0, x1, y1].
    pub fn link(&mut self, rect: [f64; 4], uri: &str) {
        self.links.push((rect, uri.to_string()));
    }
}

/// Assemble pages into a PDF file.
#[must_use]
pub fn document(pages: &[Page], size: [f64; 2], title: &str, author: &str) -> Vec<u8> {
    let mut objects: Vec<String> = Vec::new();
    // 1 catalog, 2 pages, 3-5 fonts, 6 info, then per page: page, content, links.
    objects.push("<< /Type /Catalog /Pages 2 0 R >>".into());
    objects.push(String::new()); // pages, filled in below
    for base in ["Helvetica", "Helvetica-Bold", "Courier"] {
        objects.push(format!(
            "<< /Type /Font /Subtype /Type1 /BaseFont /{base} /Encoding /WinAnsiEncoding >>"
        ));
    }
    objects.push(format!(
        "<< /Title {} /Author {} /Creator (CubeSat Classroom printouts) /Producer (CubeSat Classroom printouts) >>",
        pdf_string(title),
        pdf_string(author)
    ));
    let mut kids = Vec::new();
    for page in pages {
        let page_id = objects.len() + 1;
        let content_id = page_id + 1;
        let link_ids: Vec<usize> = (0..page.links.len()).map(|i| content_id + 1 + i).collect();
        kids.push(format!("{page_id} 0 R"));
        let annots = if link_ids.is_empty() {
            String::new()
        } else {
            format!(
                " /Annots [{}]",
                link_ids
                    .iter()
                    .map(|id| format!("{id} 0 R"))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Resources << /Font << /F1 3 0 R /F2 4 0 R /F3 5 0 R >> >> /Contents {content_id} 0 R{annots} >>",
            n(size[0]),
            n(size[1])
        ));
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}endstream",
            page.content.len(),
            page.content
        ));
        for (rect, uri) in &page.links {
            objects.push(format!(
                "<< /Type /Annot /Subtype /Link /Rect [{} {} {} {}] /Border [0 0 0] /A << /S /URI /URI {} >> >>",
                n(rect[0]),
                n(rect[1]),
                n(rect[2]),
                n(rect[3]),
                pdf_string(uri)
            ));
        }
    }
    objects[1] = format!(
        "<< /Type /Pages /Kids [{}] /Count {} >>",
        kids.join(" "),
        pages.len()
    );

    let mut out = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    let mut tail = format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1);
    for off in offsets {
        let _ = writeln!(tail, "{off:010} 00000 n ");
    }
    let _ = write!(
        tail,
        "trailer\n<< /Size {} /Root 1 0 R /Info 6 0 R >>\nstartxref\n{xref}\n%%EOF\n",
        objects.len() + 1
    );
    out.extend_from_slice(tail.as_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_and_strings() {
        // "Hello" in Helvetica: H 722 e 556 l 222 l 222 o 556.
        assert!((text_width("Hello", Face::Sans, 10.0) - 22.78).abs() < 1e-9);
        assert!((text_width("abc", Face::Mono, 10.0) - 18.0).abs() < 1e-9);
        assert_eq!(pdf_string("a(b)’•"), "(a\\(b\\)\\222\\225)");
    }

    #[test]
    fn document_is_well_formed() {
        let mut page = Page::default();
        page.text(72.0, 700.0, "Hi", Face::Bold, 12.0, Rgb::hex(0x0013_284a));
        page.link([72.0, 690.0, 100.0, 712.0], "https://example.com");
        let pdf = document(&[page], [612.0, 792.0], "T", "A");
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.starts_with("%PDF-1.4"));
        assert!(text.contains("/URI (https://example.com)"));
        let xref: usize = text
            .rsplit("startxref\n")
            .next()
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert!(pdf[xref..].starts_with(b"xref"));
    }
}
