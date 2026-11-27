//! Flowing text into a page: styled runs, word wrap, bullets, boxes and the
//! spacing between blocks.

use crate::pdf::{Face, Page, Rgb, text_width};

/// A piece of text with one style.
#[derive(Clone, Debug)]
pub struct Run {
    /// The text.
    pub text: String,
    /// Typeface.
    pub face: Face,
    /// Size in points; `None` uses the paragraph size.
    pub size: Option<f64>,
    /// Colour; `None` uses the paragraph colour.
    pub color: Option<Rgb>,
    /// A link target. Linked runs are underlined.
    pub link: Option<String>,
}

/// Styled text built up run by run.
#[derive(Clone, Debug, Default)]
pub struct Rich(pub Vec<Run>);

impl Rich {
    /// Empty text.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn push(
        mut self,
        text: &str,
        face: Face,
        size: Option<f64>,
        color: Option<Rgb>,
        link: Option<&str>,
    ) -> Self {
        self.0.push(Run {
            text: text.to_string(),
            face,
            size,
            color,
            link: link.map(str::to_string),
        });
        self
    }

    /// Plain text.
    #[must_use]
    pub fn t(self, text: &str) -> Self {
        self.push(text, Face::Sans, None, None, None)
    }

    /// Bold text.
    #[must_use]
    pub fn b(self, text: &str) -> Self {
        self.push(text, Face::Bold, None, None, None)
    }

    /// Inline code: monospace at 8.5 pt.
    #[must_use]
    pub fn code(self, text: &str) -> Self {
        self.push(text, Face::Mono, Some(8.5), None, None)
    }

    /// An underlined link.
    #[must_use]
    pub fn a(self, text: &str, href: &str, color: Rgb) -> Self {
        self.push(text, Face::Sans, None, Some(color), Some(href))
    }

    /// Text in `face` at the paragraph size.
    #[must_use]
    pub fn face(self, text: &str, face: Face) -> Self {
        self.push(text, face, None, None, None)
    }
}

/// Paragraph style.
#[derive(Clone, Copy, Debug)]
pub struct Style {
    /// Face for unstyled runs (`Rich::t` text uses Sans; this replaces it).
    pub face: Face,
    /// Size in points.
    pub size: f64,
    /// Baseline-to-baseline distance.
    pub leading: f64,
    /// Text colour.
    pub color: Rgb,
    /// Space above, dropped at the top of the page.
    pub space_before: f64,
    /// Space below.
    pub space_after: f64,
}

/// One placed fragment of a line.
#[derive(Clone, Debug)]
struct Piece {
    x: f64,
    text: String,
    face: Face,
    size: f64,
    color: Rgb,
    link: Option<String>,
}

/// Break `rich` into lines no wider than `width`. Returns the pieces of each line.
fn wrap(rich: &Rich, style: &Style, width: f64) -> Vec<Vec<Piece>> {
    // Words are runs of non-space text; a word may span several runs.
    struct Seg {
        text: String,
        face: Face,
        size: f64,
        color: Rgb,
        link: Option<String>,
    }
    let mut words: Vec<Vec<Seg>> = vec![Vec::new()];
    for run in &rich.0 {
        let face = if run.face == Face::Sans {
            style.face
        } else {
            run.face
        };
        let size = run.size.unwrap_or(style.size);
        let color = run.color.unwrap_or(style.color);
        for (i, part) in run.text.split(' ').enumerate() {
            if i > 0 && !words.last().is_some_and(Vec::is_empty) {
                words.push(Vec::new());
            }
            if !part.is_empty() {
                words.last_mut().expect("a word").push(Seg {
                    text: part.to_string(),
                    face,
                    size,
                    color,
                    link: run.link.clone(),
                });
            }
        }
    }
    words.retain(|w| !w.is_empty());
    let word_width = |w: &Vec<Seg>| {
        w.iter()
            .map(|s| text_width(&s.text, s.face, s.size))
            .sum::<f64>()
    };
    let space = |s: &Seg| text_width(" ", s.face, s.size);

    let mut lines: Vec<Vec<Piece>> = Vec::new();
    let mut line: Vec<Piece> = Vec::new();
    let mut x = 0.0;
    for word in &words {
        let w = word_width(word);
        let gap = if line.is_empty() {
            0.0
        } else {
            space(&word[0])
        };
        if !line.is_empty() && x + gap + w > width + 1e-6 {
            lines.push(std::mem::take(&mut line));
            x = 0.0;
        }
        if !line.is_empty() {
            // A space between two words of the same style joins them into one
            // piece, so a link's underline continues across it.
            let prev = line.last().expect("piece");
            let first = &word[0];
            let same = prev.face == first.face
                && (prev.size - first.size).abs() < 1e-9
                && prev.color == first.color
                && prev.link == first.link;
            let sp = text_width(" ", prev.face, prev.size);
            if same {
                line.last_mut().expect("piece").text.push(' ');
            }
            x += sp;
        }
        for seg in word {
            let piece_w = text_width(&seg.text, seg.face, seg.size);
            match line.last_mut() {
                Some(last)
                    if last.face == seg.face
                        && (last.size - seg.size).abs() < 1e-9
                        && last.color == seg.color
                        && last.link == seg.link
                        && (last.x + text_width(&last.text, last.face, last.size) - x).abs()
                            < 1e-6 =>
                {
                    last.text.push_str(&seg.text);
                }
                _ => line.push(Piece {
                    x,
                    text: seg.text.clone(),
                    face: seg.face,
                    size: seg.size,
                    color: seg.color,
                    link: seg.link.clone(),
                }),
            }
            x += piece_w;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// A block on the page.
pub enum Block {
    /// A paragraph, optionally indented with a bullet, optionally in a box.
    Para {
        /// Text style.
        style: Style,
        /// Text.
        text: Rich,
        /// Left indent of the text.
        indent: f64,
        /// A bullet drawn in the indent: (glyph, size).
        bullet: Option<(String, f64)>,
        /// A shaded box behind the paragraph: (fill, border, border width, padding).
        boxed: Option<(Rgb, Rgb, f64, f64)>,
    },
    /// Fixed vertical space.
    Space(f64),
    /// A one-cell table: shaded, bordered, with a thick accent rule on the left.
    Callout {
        /// Text style.
        style: Style,
        /// Text.
        text: Rich,
        /// Box width.
        width: f64,
        /// Fill colour.
        fill: Rgb,
        /// Border colour and width.
        border: (Rgb, f64),
        /// Left accent colour and width.
        accent: (Rgb, f64),
        /// Padding: left, right, top, bottom.
        pad: [f64; 4],
    },
}

/// Draw pieces of one line with the baseline at `y`.
fn draw_line(page: &mut Page, pieces: &[Piece], x0: f64, y: f64) {
    for p in pieces {
        page.text(x0 + p.x, y, &p.text, p.face, p.size, p.color);
        if let Some(href) = &p.link {
            let w = text_width(&p.text, p.face, p.size);
            page.line([x0 + p.x, y - 1.2], [x0 + p.x + w, y - 1.2], 0.6, p.color);
            page.link([x0 + p.x, y - 2.5, x0 + p.x + w, y + p.size], href);
        }
    }
}

/// Flow `blocks` down a frame from `top` within [left, left + width].
/// Returns the y where the content ended.
pub fn flow(page: &mut Page, blocks: &[Block], left: f64, top: f64, width: f64) -> f64 {
    let mut y = top;
    let mut first = true;
    for block in blocks {
        match block {
            Block::Space(h) => y -= h,
            Block::Para {
                style,
                text,
                indent,
                bullet,
                boxed,
            } => {
                if !first {
                    y -= style.space_before;
                }
                let pad = boxed.map_or(0.0, |b| b.3);
                // A box's padding sits outside the text column, above and below
                // the lines and out into the margins, as ReportLab draws it.
                y -= pad;
                let lines = wrap(text, style, width - indent);
                let height = lines.len() as f64 * style.leading;
                if let Some((fill, border, bw, pad)) = boxed {
                    page.rect(
                        [
                            left + indent - pad,
                            y - height - pad,
                            width - indent + 2.0 * pad,
                            height + 2.0 * pad,
                        ],
                        Some(*fill),
                        Some((*border, *bw)),
                    );
                }
                let x0 = left + indent;
                let mut baseline = y - style.size;
                if let Some((glyph, size)) = bullet {
                    page.text(left, baseline + 1.0, glyph, style.face, *size, style.color);
                }
                for line in &lines {
                    draw_line(page, line, x0, baseline);
                    baseline -= style.leading;
                }
                y -= height + pad + style.space_after;
            }
            Block::Callout {
                style,
                text,
                width: box_w,
                fill,
                border,
                accent,
                pad,
            } => {
                if !first {
                    y -= style.space_before;
                }
                let lines = wrap(text, style, box_w - pad[0] - pad[1]);
                let height = lines.len() as f64 * style.leading + pad[2] + pad[3];
                page.rect(
                    [left, y - height, *box_w, height],
                    Some(*fill),
                    Some((border.0, border.1)),
                );
                page.line([left, y], [left, y - height], accent.1, accent.0);
                let mut baseline = y - pad[2] - style.size;
                for line in &lines {
                    draw_line(page, line, left + pad[0], baseline);
                    baseline -= style.leading;
                }
                y -= height + style.space_after;
            }
        }
        first = false;
    }
    y
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style() -> Style {
        Style {
            face: Face::Sans,
            size: 10.0,
            leading: 12.0,
            color: Rgb(0.0, 0.0, 0.0),
            space_before: 0.0,
            space_after: 0.0,
        }
    }

    #[test]
    fn wraps_at_the_width() {
        let text = Rich::new().t("aaa bbb ccc ddd");
        let one = text_width("aaa bbb", Face::Sans, 10.0);
        let lines = wrap(&text, &style(), one + 0.5);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0][0].text, "aaa bbb");
    }

    #[test]
    fn styles_inside_a_word_stay_together() {
        let text = Rich::new().t("type ").code("n").t(" at");
        let lines = wrap(&text, &style(), 500.0);
        let faces: Vec<Face> = lines[0].iter().map(|p| p.face).collect();
        assert_eq!(faces, vec![Face::Sans, Face::Mono, Face::Sans]);
    }
}
