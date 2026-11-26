//! Drawings for the README, as SVG: the payload plate from the board side, and sections through a
//! snap pin and a board snap hook, each pushing on and clicked. They are cut from the same solids
//! that are printed, so they can't drift from the design.

use std::fmt::Write as _;
use std::path::Path;

use manifold_csg::Manifold;

use crate::design::Design;

use crate::boards::nano_board;
use crate::joints::{contact_r, edge_frame, finger_pass, finger_strain, hook_strain};
use crate::parts::{self, COUPON_PIN_AT};

const INK: &str = "#1d2228";
const FRAME: &str = "#8d96a3";
const PLATE: &str = "#c9ced6";
const BOARD: &str = "#1a7359";
const BLUE: &str = "#1f6fb2";
const RED: &str = "#c9382f";
const TEAL: &str = "#0b8f8a";

/// One panel in millimetres, y up, drawn into an SVG group.
struct Panel {
    /// Pixels per millimetre.
    px: f64,
    lo: [f64; 2],
    hi: [f64; 2],
    body: String,
}

fn n(v: f64) -> String {
    let s = format!("{v:.2}");
    if s == "-0.00" { "0.00".into() } else { s }
}

impl Panel {
    fn new(px: f64, lo: [f64; 2], hi: [f64; 2]) -> Self {
        Self {
            px,
            lo,
            hi,
            body: String::new(),
        }
    }

    fn width(&self) -> f64 {
        (self.hi[0] - self.lo[0]) * self.px
    }

    fn height(&self) -> f64 {
        (self.hi[1] - self.lo[1]) * self.px
    }

    fn at(&self, p: [f64; 2]) -> (String, String) {
        (
            n((p[0] - self.lo[0]) * self.px),
            n((self.hi[1] - p[1]) * self.px),
        )
    }

    /// Filled polygons with holes (even-odd), as a slice returns them.
    fn polygons(&mut self, polys: &[Vec<[f64; 2]>], fill: &str) {
        if polys.is_empty() {
            return;
        }
        let mut d = String::new();
        for ring in polys {
            for (i, p) in ring.iter().enumerate() {
                let (x, y) = self.at(*p);
                let _ = write!(d, "{}{x} {y} ", if i == 0 { "M" } else { "L" });
            }
            d.push_str("Z ");
        }
        let _ = writeln!(
            self.body,
            "<path d=\"{}\" fill=\"{fill}\" fill-rule=\"evenodd\" stroke=\"{INK}\" stroke-width=\"0.8\"/>",
            d.trim_end()
        );
    }

    fn rect(&mut self, x0: f64, y0: f64, w: f64, h: f64, style: &str) {
        let (x, y) = self.at([x0, y0 + h]);
        let _ = writeln!(
            self.body,
            "<rect x=\"{x}\" y=\"{y}\" width=\"{}\" height=\"{}\" {style}/>",
            n(w * self.px),
            n(h * self.px)
        );
    }

    fn circle(&mut self, c: [f64; 2], r: f64, style: &str) {
        let (x, y) = self.at(c);
        let _ = writeln!(
            self.body,
            "<circle cx=\"{x}\" cy=\"{y}\" r=\"{}\" {style}/>",
            n(r * self.px)
        );
    }

    /// Text, one line per entry, with an optional arrow (from, to).
    fn note(&mut self, at: [f64; 2], lines: &[String], arrow: Option<([f64; 2], [f64; 2])>) {
        if let Some((f, t)) = arrow {
            let ((x0, y0), (x1, y1)) = (self.at(f), self.at(t));
            let _ = writeln!(
                self.body,
                "<line x1=\"{x0}\" y1=\"{y0}\" x2=\"{x1}\" y2=\"{y1}\" stroke=\"{INK}\" stroke-width=\"1\" \
                 marker-end=\"url(#arrow)\"/>"
            );
        }
        let (x, y) = self.at(at);
        let _ = write!(self.body, "<text x=\"{x}\" y=\"{y}\" font-size=\"13\">");
        for (i, line) in lines.iter().enumerate() {
            let _ = write!(
                self.body,
                "<tspan x=\"{x}\" dy=\"{}\">{}</tspan>",
                if i == 0 { "-4" } else { "16" },
                escape(line)
            );
        }
        self.body.push_str("</text>\n");
    }

    fn grid(&mut self, step: f64) {
        let mut v = (self.lo[0] / step).ceil() * step;
        while v <= self.hi[0] {
            let ((x, y0), (_, y1)) = (self.at([v, self.lo[1]]), self.at([v, self.hi[1]]));
            let _ = writeln!(
                self.body,
                "<line x1=\"{x}\" y1=\"{y0}\" x2=\"{x}\" y2=\"{y1}\" stroke=\"#e4e7eb\"/>"
            );
            v += step;
        }
        let mut v = (self.lo[1] / step).ceil() * step;
        while v <= self.hi[1] {
            let ((x0, y), (x1, _)) = (self.at([self.lo[0], v]), self.at([self.hi[0], v]));
            let _ = writeln!(
                self.body,
                "<line x1=\"{x0}\" y1=\"{y}\" x2=\"{x1}\" y2=\"{y}\" stroke=\"#e4e7eb\"/>"
            );
            v += step;
        }
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Panels side by side under a title, with a legend underneath.
fn page(title: &str, panels: &[(String, Panel)], legend: &[(&str, &str)]) -> String {
    let gap = 30.0;
    let width =
        panels.iter().map(|(_, p)| p.width()).sum::<f64>() + gap * (panels.len() as f64 + 1.0);
    let height = panels.iter().map(|(_, p)| p.height()).fold(0.0, f64::max);
    let total = 60.0 + height + 50.0 + 22.0 * legend.len().div_ceil(2) as f64;
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" font-family=\"Helvetica, Arial, sans-serif\" \
         fill=\"{INK}\">\n<defs><marker id=\"arrow\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" markerWidth=\"7\" \
         markerHeight=\"7\" orient=\"auto\"><path d=\"M0 0 L10 5 L0 10 Z\" fill=\"{INK}\"/></marker></defs>\n\
         <rect width=\"100%\" height=\"100%\" fill=\"white\"/>\n<text x=\"{}\" y=\"28\" font-size=\"17\" \
         text-anchor=\"middle\" font-weight=\"bold\">{}</text>\n",
        n(width),
        n(total),
        n(width / 2.0),
        escape(title)
    );
    let mut x = gap;
    for (i, (caption, p)) in panels.iter().enumerate() {
        let (w, h) = (n(p.width()), n(p.height()));
        let _ = writeln!(
            out,
            "<text x=\"{}\" y=\"52\" font-size=\"14\">{}</text>\n<clipPath id=\"panel{i}\"><rect width=\"{w}\" \
             height=\"{h}\"/></clipPath>\n<g transform=\"translate({} 62)\">\n<g clip-path=\"url(#panel{i})\">\n{}</g>\n\
             <rect width=\"{w}\" height=\"{h}\" fill=\"none\" stroke=\"#c3c8cf\"/>\n</g>",
            n(x),
            escape(caption),
            n(x),
            p.body
        );
        x += p.width() + gap;
    }
    for (i, (colour, label)) in legend.iter().enumerate() {
        let lx = gap + (i % 2) as f64 * (width - 2.0 * gap) / 2.0;
        let ly = 62.0 + height + 30.0 + 22.0 * (i / 2) as f64;
        let _ = writeln!(
            out,
            "<rect x=\"{}\" y=\"{}\" width=\"16\" height=\"12\" fill=\"{colour}\" stroke=\"{INK}\" stroke-width=\"0.6\"/>\
             <text x=\"{}\" y=\"{}\" font-size=\"13\">{}</text>",
            n(lx),
            n(ly - 11.0),
            n(lx + 24.0),
            n(ly),
            escape(label)
        );
    }
    out.push_str("</svg>\n");
    out
}

/// Cut a solid along the vertical plane through (x, y) at `angle` degrees: (distance, height).
fn section(solid: &Manifold, x: f64, y: f64, angle: f64) -> Vec<Vec<[f64; 2]>> {
    solid
        .translate(-x, -y, 0.0)
        .rotate(0.0, 0.0, -angle)
        .rotate(-90.0, 0.0, 0.0)
        .slice_at_z(0.0)
}

fn snap_pin(d: &Design) -> Result<String, String> {
    // On the coupon: the pin's lobes face +-x, its flats +-y.
    let base = parts::coupon_pin_base(d);
    let tab = |tooth: Option<f64>, lift: f64| {
        parts::coupon_pin_tab(d, d.pin_fit, 2, tooth).translate(0.0, 0.0, COUPON_PIN_AT + lift)
    };
    let head = d.pin_head_r + 0.1;
    let z0 = COUPON_PIN_AT;
    let mut panels = Vec::new();
    for (i, (caption, lift, tooth)) in [
        (
            "1. Push down: the gentle lead-in spreads the teeth",
            1.6,
            Some(head),
        ),
        (
            "2. Click: the teeth spring in under the 45° face",
            0.0,
            None,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let mut p = Panel::new(30.0, [-11.0, z0 - 4.0], [11.0, z0 + 9.0]);
        p.grid(2.0);
        p.polygons(&section(&base, 0.0, 0.0, 0.0), FRAME);
        p.polygons(&section(&tab(tooth, lift), 0.0, 0.0, 0.0), PLATE);
        let lines = if i == 0 {
            vec![
                format!("each finger bends {:.2} mm", finger_pass(d)),
                format!(
                    "({:.2}% strain, limit {:.1}%),",
                    finger_strain(d) * 100.0,
                    d.strain_limit * 100.0
                ),
                "in the plate's own plane".into(),
            ]
        } else {
            vec![
                format!(
                    "tooth held {:.2} mm out from rest:",
                    contact_r(d) - d.tooth_r
                ),
                "the 45° face pulls the tab down,".into(),
                "so it can't rattle".into(),
            ]
        };
        let target = if i == 0 {
            [-head - 0.1, z0 + d.plate_t + lift - 0.3]
        } else {
            [-contact_r(d) - 0.1, z0 + d.plate_t - 0.1]
        };
        p.note([-10.5, z0 + 8.2], &lines, Some(([-6.0, z0 + 6.9], target)));
        panels.push((caption.to_string(), p));
    }
    // Seen from above, cut through the middle of the tab.
    let z = z0 + d.plate_t / 2.0;
    let mut p = Panel::new(30.0, [-11.0, -14.5], [11.0, 7.0]);
    p.grid(2.0);
    p.polygons(&tab(None, 0.0).slice_at_z(z), PLATE);
    p.polygons(&base.slice_at_z(z), FRAME);
    let wall = d.pin_flat + d.pin_fit;
    p.note(
        [-10.5, 6.4],
        &[
            format!("flats sit {} mm from rigid walls:", d.pin_fit),
            "the tab goes on one way, centred".into(),
        ],
        Some(([-2.0, 5.0], [0.0, wall + 0.1])),
    );
    p.note(
        [-10.5, -12.6],
        &[
            "fingers bend out; the tongue between".into(),
            "them stays put".into(),
        ],
        Some(([2.0, -11.0], [d.pin_head_r + d.finger_clear + 1.0, -7.0])),
    );
    panels.push(("3. From above: the keyed slot".to_string(), p));
    Ok(page(
        "Snap pin: cut through the pin and both teeth, and seen from above (mm)",
        &panels,
        &[
            (FRAME, "column and keyed snap pin"),
            (
                PLATE,
                "plate (here the coupon tab) with its slot and flex fingers",
            ),
        ],
    ))
}

fn board_hook(d: &Design) -> String {
    let rect = d.nano_rect();
    let edge = d.nano_hooks[0];
    let (angle, [ox, oy], _) = edge_frame(rect, &edge);
    let top = d.nano_top();
    let mut panels = Vec::new();
    for (i, (caption, lift, deflect)) in [
        (
            "1. Swing the Nano down: its edge bends the hook out",
            d.pcb_nano + 0.3,
            d.hook_reach,
        ),
        ("2. Click: the hook springs back over the board", 0.0, 0.0),
    ]
    .into_iter()
    .enumerate()
    {
        let mut p = Panel::new(30.0, [-9.0, -1.0], [6.0, top + 6.0]);
        p.grid(2.0);
        p.polygons(
            &section(&parts::coupon_nano_base(d, deflect), ox, oy, angle),
            PLATE,
        );
        let board = nano_board(d).translate(0.0, 0.0, lift);
        p.polygons(&section(&board, ox, oy, angle), BOARD);
        let lines = if i == 0 {
            vec![format!(
                "the finger bends {} mm ({:.2}% strain)",
                d.hook_reach,
                hook_strain(d) * 100.0
            )]
        } else {
            vec![
                format!("hook underside at {:.0}°, under the", d.hook_angle),
                format!("{:.0}° friction angle: lifting the", d.friction_angle()),
                "board can't push it open".into(),
            ]
        };
        p.note(
            [-8.6, top + 5.2],
            &lines,
            Some(([-1.0, top + 3.6], [deflect - 0.3, top + 0.6])),
        );
        panels.push((caption.to_string(), p));
    }
    page(
        "Board snap hook: a cut through a hook on the Nano coupon, with the bare Nano (mm)",
        &panels,
        &[
            (PLATE, "Nano coupon: a pad and the snap hook"),
            (BOARD, "Nano 33 BLE Sense Rev2, 1.59 mm"),
        ],
    )
}

fn layout(d: &Design, plate: &Manifold) -> String {
    // Section heights through the plate, lightest (the plate) to darkest (the tallest parts).
    let levels = [
        (1.5, "#d9dde3"),
        (4.0, "#b8bec7"),
        (8.0, "#8f98a3"),
        (14.0, "#6b7480"),
        (20.0, "#4a525c"),
    ];
    let half = d.plate / 2.0 + 3.0;
    let mut p = Panel::new(8.0, [-half, -half], [half, half]);
    p.grid(10.0);
    for (z, colour) in levels {
        p.polygons(&plate.slice_at_z(z), colour);
    }
    let outline = |c: &str| format!("fill=\"none\" stroke=\"{c}\" stroke-width=\"2\"");
    let [ux, uy, uw, uh] = d.uno_rect();
    p.rect(ux, uy, uw, uh, &outline(BLUE));
    let [nx, ny, nw, nh] = d.nano_rect();
    p.rect(nx, ny, nw, nh, &outline(TEAL));
    for (i, c) in d.plate_holes().iter().enumerate() {
        let dashed = if d.snap_columns.iter().any(|(k, _)| *k == i) {
            ""
        } else {
            " stroke-dasharray=\"5 4\""
        };
        p.circle(*c, d.column_od / 2.0, &format!("{}{dashed}", outline(RED)));
    }
    page(
        "Payload plate, board side up (darker = taller, mm)",
        &[(String::new(), p)],
        &[
            (BLUE, "Uno R3 outline"),
            (TEAL, "Nano 33 BLE Sense outline"),
            (RED, "columns: snap pins (solid), locating pin (dashed)"),
            ("#8f98a3", "plate, posts, lips and hooks"),
        ],
    )
}

/// Write the drawings next to the README.
pub fn write_all(d: &Design, dir: &Path) -> Result<(), String> {
    let plate = parts::payload_plate(d, None, 0.0)?;
    for (file, svg) in [
        ("payload-plate-layout.svg", layout(d, &plate)),
        ("snap-pin-section.svg", snap_pin(d)?),
        ("board-hook-section.svg", board_hook(d)),
    ] {
        std::fs::write(dir.join(file), svg).map_err(|e| e.to_string())?;
        println!("wrote {file}");
    }
    Ok(())
}
