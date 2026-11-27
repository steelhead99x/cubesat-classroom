//! Outdoor ground-station cases: a sealed box with the kit's columns and keyed snap pins inside,
//! so the same plates snap in. The 3000 mAh cell lies in a pocket on a deck under the plate,
//! held in by the plate itself. A gasket in a groove round the rim seals a lid held by eight
//! captive screws. Cables come in through glands in the wall that faces down, antennas leave
//! through the wall that faces up, a breathable vent stops condensation, and a 1/4"-20 camera
//! thread in the middle of the back takes a tripod.
//!
//! Print the box, lid and plate in ASA or PETG: PLA softens in sun. Body coordinates: z = 0 is
//! the back's outside face, +y is up when it stands on a tripod with its lid facing out.

use manifold_csg::{CrossSection, Manifold};

use crate::design::Design;
use crate::geom::{EPS, FACETS, bx, chamfered_square, cone, cyl, offset, union_all};
use crate::joints::{locating_pin, pin};
use crate::parts::branding;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PortKind {
    /// M12 x 1.5 cable gland (3-6.5 mm cable): a USB cable for power and data.
    Gland,
    /// M12 x 1.5 breathable membrane vent.
    Vent,
    /// SMA bulkhead jack (1/4"-36), O-ring outside.
    Sma,
}

impl PortKind {
    pub fn hole(self) -> f64 {
        match self {
            Self::Gland | Self::Vent => 12.3,
            Self::Sma => 6.6,
        }
    }
    /// Room for the part's outside body and its inside nut: diameter and depth inside.
    pub fn outside_d(self) -> f64 {
        match self {
            Self::Gland | Self::Vent => 19.0,
            Self::Sma => 11.0,
        }
    }
    pub fn inside(self) -> (f64, f64) {
        match self {
            Self::Gland => (20.0, 8.0),
            Self::Vent => (20.0, 6.0),
            Self::Sma => (11.0, 10.0),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Gland => "M12 cable gland",
            Self::Vent => "M12 breathable vent",
            Self::Sma => "SMA bulkhead",
        }
    }
}

/// A port through the wall that faces down (`up` false) or up (`up` true).
#[derive(Clone, Copy, Debug)]
pub struct Port {
    pub kind: PortKind,
    pub up: bool,
    pub x: f64,
    pub z: f64,
}

#[derive(Clone, Debug)]
pub struct Outdoor {
    /// Inside of the box: a chamfered square, like the plate with room round it.
    pub cavity: f64,
    pub cavity_chamfer: f64,
    pub wall: f64,
    pub floor: f64,
    /// Height of the rim above the back's outside face.
    pub top: f64,
    /// The rim's flange, outward past the wall, and how tall its straight part is. Under it a
    /// 45 degree slope, so it prints without supports.
    pub flange: f64,
    pub flange_h: f64,
    /// Gasket groove: centre, from the cavity; width; depth. The gasket is a printed TPU ring
    /// (or 2.5 mm silicone cord).
    pub groove_at: f64,
    pub groove_w: f64,
    pub groove_d: f64,
    pub gasket: f64,
    /// Screws: centre from the cavity; heat-set insert hole and depth; screw clearance and head.
    pub screw_at: f64,
    pub insert_d: f64,
    pub insert_depth: f64,
    pub screw_clear: f64,
    /// The bottom of each lid hole is this narrow, so a screw cuts through once and stays in.
    pub screw_captive: f64,
    pub screw_head: f64,
    pub lid_t: f64,
    /// Tripod thread, 1/4"-20 UNC, printed: printed holes come out small, so the thread is cut
    /// larger by this much on the diameter.
    pub thread_major: f64,
    pub thread_minor: f64,
    pub thread_pitch: f64,
    pub thread_clear: f64,
    pub thread_len: f64,
    pub boss_d: f64,
    /// The battery deck's top, and its thickness; the pocket's walls.
    pub deck_z: f64,
    pub deck_t: f64,
    pub pocket_wall: f64,
    pub pocket_h: f64,
    pub cell_gap: f64,
    /// Largest cell top to the plate's underside.
    pub cell_clear: f64,
    pub ports: Vec<Port>,
}

impl Default for Outdoor {
    fn default() -> Self {
        Self {
            cavity: 92.0,
            cavity_chamfer: 10.0,
            wall: 3.0,
            floor: 3.0,
            top: 76.0,
            flange: 10.0,
            flange_h: 6.0,
            groove_at: 4.5,
            groove_w: 3.0,
            groove_d: 1.9,
            gasket: 2.6,
            screw_at: 9.5,
            insert_d: 4.0,
            insert_depth: 6.0,
            screw_clear: 3.4,
            screw_captive: 2.7,
            screw_head: 6.2,
            lid_t: 4.0,
            thread_major: 6.35,
            thread_minor: 5.35,
            thread_pitch: 1.27,
            thread_clear: 0.3,
            thread_len: 9.0,
            boss_d: 20.0,
            deck_z: 12.5,
            deck_t: 2.0,
            pocket_wall: 2.0,
            pocket_h: 8.0,
            cell_gap: 0.5,
            cell_clear: 0.5,
            ports: vec![
                Port {
                    kind: PortKind::Gland,
                    up: false,
                    x: -25.0,
                    z: 48.0,
                },
                Port {
                    kind: PortKind::Vent,
                    up: false,
                    x: 25.0,
                    z: 48.0,
                },
            ],
        }
    }
}

impl Outdoor {
    /// The XIAO case: the same box with two SMA bulkheads in the wall that faces up, for the
    /// HaLow antenna and the XIAOs' 2.4 GHz antenna.
    pub fn xiao() -> Self {
        let mut o = Self::default();
        o.ports.push(Port {
            kind: PortKind::Sma,
            up: true,
            x: -20.0,
            z: 48.0,
        });
        o.ports.push(Port {
            kind: PortKind::Sma,
            up: true,
            x: 20.0,
            z: 48.0,
        });
        o
    }

    pub fn cavity_outline(&self) -> CrossSection {
        chamfered_square(self.cavity, self.cavity_chamfer)
    }

    /// The plate's underside: over the deck, the largest cell and its clearance.
    pub fn plate_z(&self, d: &Design) -> f64 {
        self.deck_z + d.cell_thick() + self.cell_clear
    }

    /// Where the 45 degree slope under the flange starts.
    pub fn flare_z(&self) -> f64 {
        self.top - self.flange_h - self.flange
    }

    /// The eight screws: the chamfers' middles and the sides' middles.
    pub fn screws(&self) -> Vec<[f64; 2]> {
        let a = self.cavity / 2.0 + self.screw_at;
        let c = (self.cavity - self.cavity_chamfer) / 2.0 + self.screw_at / 2f64.sqrt();
        let mut s = Vec::new();
        for (sx, sy) in [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
            s.push([sx * c, sy * c]);
        }
        s.extend([[a, 0.0], [0.0, a], [-a, 0.0], [0.0, -a]]);
        s
    }

    fn ring(&self, at: f64, w: f64) -> CrossSection {
        let outline = self.cavity_outline();
        offset(&outline, at + w / 2.0, FACETS).difference(&offset(&outline, at - w / 2.0, FACETS))
    }

    /// The wall a port goes through: its outside face's y, and which way is out.
    pub fn port_wall(&self, p: &Port) -> (f64, f64) {
        let face = self.cavity / 2.0 + self.wall;
        if p.up { (face, 1.0) } else { (-face, -1.0) }
    }
}

/// A cylinder along y through a port, from `y0` to `y1`.
fn along_y(d: f64, x: f64, z: f64, y0: f64, y1: f64) -> Manifold {
    let (lo, hi) = (y0.min(y1), y0.max(y1));
    cyl(d, hi - lo, 0.0, 0.0, 0.0)
        .rotate(-90.0, 0.0, 0.0)
        .translate(x, lo, z)
}

pub fn port_hole(o: &Outdoor, p: &Port) -> Manifold {
    let (face, out) = o.port_wall(p);
    along_y(
        p.kind.hole(),
        p.x,
        p.z,
        face + out,
        face - out * (o.wall + 1.0),
    )
}

/// Room the port's outside body needs on the wall's face.
pub fn port_outside(o: &Outdoor, p: &Port) -> Manifold {
    let (face, out) = o.port_wall(p);
    along_y(p.kind.outside_d(), p.x, p.z, face, face + out * 5.0)
}

/// Room the port's nut and cable need inside.
pub fn port_inside(o: &Outdoor, p: &Port) -> Manifold {
    let (face, out) = o.port_wall(p);
    let (dia, depth) = p.kind.inside();
    let inner = face - out * o.wall;
    along_y(dia, p.x, p.z, inner - out * 0.05, inner - out * depth)
}

/// The tripod thread: the shape of a 1/4"-20 screw, grown by the clearance, `len` long from z.
pub fn thread(o: &Outdoor, len: f64, z: f64, clear: f64) -> Manifold {
    let (r0, r1) = (
        (o.thread_minor + clear) / 2.0,
        (o.thread_major + clear) / 2.0,
    );
    let n = 96;
    // One turn of a 60 degree profile: flank, crest, flank, root, a quarter turn each.
    let pts: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let r = if t < 0.25 {
                r0 + (r1 - r0) * t / 0.25
            } else if t < 0.5 {
                r1
            } else if t < 0.75 {
                r1 - (r1 - r0) * (t - 0.5) / 0.25
            } else {
                r0
            };
            let a = 2.0 * std::f64::consts::PI * t;
            [r * a.cos(), r * a.sin()]
        })
        .collect();
    let section = CrossSection::from_polygons(&[pts]);
    let turns = len / o.thread_pitch;
    // A right-hand thread: the profile turns anticlockwise going up.
    let slices = (turns * 48.0).ceil() as i32;
    Manifold::extrude_with_options(&section, len, slices, 360.0 * turns, 1.0, 1.0)
        .translate(0.0, 0.0, z)
}

/// The cut for the tripod socket: the thread with a small lead-in at the back's face.
pub fn socket_cut(o: &Outdoor, clear: f64) -> Manifold {
    let lead = cone(
        o.thread_major + clear + 1.2,
        o.thread_minor + clear - 0.2,
        0.9,
        0.0,
        0.0,
        -EPS,
    );
    thread(o, o.thread_len, -0.5, clear).union(&lead)
}

/// The battery pocket's floor, walls and the ribs under its deck (body coordinates).
pub fn battery_pocket(d: &Design, o: &Outdoor) -> Manifold {
    let [x0, y0, w, h] = d.cell_rect();
    let g = o.cell_gap;
    let t = o.pocket_wall;
    let (ix0, iy0, ix1, iy1) = (x0 - g, y0 - g, x0 + w + g, y0 + h + g);
    let (ox0, oy0, ox1, oy1) = (ix0 - t, iy0 - t, ix1 + t, iy1 + t);
    let mut parts = vec![bx(ox0, oy0, o.deck_z - o.deck_t, ox1, oy1, o.deck_z)];
    // Ribs across under the deck, so it bridges no more than 10 mm.
    let n = ((oy1 - oy0) / 10.0).ceil() as usize;
    for i in 0..=n {
        let y = oy0 + (oy1 - oy0) * i as f64 / n as f64;
        let (ya, yb) = ((y - 0.8).max(oy0), (y + 0.8).min(oy1));
        parts.push(bx(
            ox0,
            ya,
            o.floor - EPS,
            ox1,
            yb,
            o.deck_z - o.deck_t + EPS,
        ));
    }
    // The bays between the ribs stay open at their ends, so no air is sealed in.
    // Walls round the cell, with a gap at the top end for its wire.
    let z1 = o.deck_z + o.pocket_h;
    let xm = (ix0 + ix1) / 2.0;
    parts.extend([
        bx(ox0, oy0, o.deck_z - EPS, ix0, oy1, z1),
        bx(ix1, oy0, o.deck_z - EPS, ox1, oy1, z1),
        bx(ix0 - EPS, oy0, o.deck_z - EPS, ix1 + EPS, iy0, z1),
        bx(ix0 - EPS, iy1, o.deck_z - EPS, xm - 7.0, oy1, z1),
        bx(xm + 7.0, iy1, o.deck_z - EPS, ix1 + EPS, oy1, z1),
    ]);
    parts.push(cyl(
        o.boss_d,
        o.deck_z - o.deck_t - o.floor + 2.0 * EPS,
        0.0,
        0.0,
        o.floor - EPS,
    ));
    union_all(&parts)
}

/// The largest cell in its pocket (body coordinates).
pub fn cell(d: &Design, o: &Outdoor) -> Manifold {
    let [x0, y0, w, h] = d.cell_rect();
    bx(x0, y0, o.deck_z, x0 + w, y0 + h, o.deck_z + d.cell_thick())
}

/// The columns and their snap or locating pins (body coordinates).
pub fn columns(d: &Design, o: &Outdoor) -> Manifold {
    let zp = o.plate_z(d);
    let mut parts = Vec::new();
    for (i, [x, y]) in d.plate_holes().into_iter().enumerate() {
        parts.push(cyl(d.column_od, zp - o.floor + EPS, x, y, o.floor - EPS));
        parts.push(match d.snap_angle(i) {
            Some(angle) => pin(d, x, y, angle, zp, d.plate_t),
            None => locating_pin(d, x, y, zp, d.plate_t),
        });
    }
    union_all(&parts)
}

/// The box: walls, flange, groove, insert holes, columns, battery pocket, ports and tripod
/// socket. `clear` is the tripod thread's clearance.
pub fn body(d: &Design, o: &Outdoor, clear: f64) -> Manifold {
    let outline = o.cavity_outline();
    let wall = offset(&outline, o.wall, FACETS);
    let rim = offset(&outline, o.wall + o.flange, FACETS);
    let zf = o.flare_z();
    let outer = wall.extrude(zf + EPS).union(&Manifold::batch_hull(&[
        wall.extrude(EPS).translate(0.0, 0.0, zf),
        rim.extrude(o.flange_h)
            .translate(0.0, 0.0, o.top - o.flange_h),
    ]));
    let mut cuts = vec![
        outline.extrude(o.top).translate(0.0, 0.0, o.floor),
        o.ring(o.groove_at, o.groove_w)
            .extrude(o.groove_d + 1.0)
            .translate(0.0, 0.0, o.top - o.groove_d),
        socket_cut(o, clear),
    ];
    for [x, y] in o.screws() {
        cuts.push(cyl(
            o.insert_d,
            o.insert_depth + 1.0,
            x,
            y,
            o.top - o.insert_depth,
        ));
    }
    for p in &o.ports {
        cuts.push(port_hole(o, p));
    }
    let shell = outer.difference(&union_all(&cuts));
    let inside =
        union_all(&[columns(d, o), battery_pocket(d, o)]).difference(&socket_cut(o, clear));
    shell.union(&inside)
}

/// The lid, printed sealing face down: captive screw holes, and the branding on top.
pub fn lid(d: &Design, o: &Outdoor) -> Result<Manifold, String> {
    let rim = offset(&o.cavity_outline(), o.wall + o.flange, FACETS);
    let mut cuts = Vec::new();
    for [x, y] in o.screws() {
        cuts.push(cyl(o.screw_captive, 1.0 + EPS, x, y, -EPS));
        cuts.push(cyl(o.screw_clear, o.lid_t, x, y, 1.0));
        cuts.push(cyl(o.screw_head, 1.6 + EPS, x, y, o.lid_t - 1.6));
    }
    let relief = branding(d)?
        .extrude(0.6 + EPS)
        .translate(0.0, 0.0, o.lid_t - EPS);
    Ok(rim
        .extrude(o.lid_t)
        .difference(&union_all(&cuts))
        .union(&relief))
}

/// The lid in place on the box.
pub fn lid_in_place(d: &Design, o: &Outdoor) -> Result<Manifold, String> {
    Ok(lid(d, o)?.translate(0.0, 0.0, o.top))
}

/// The gasket: a printed TPU ring for the groove.
pub fn gasket(o: &Outdoor) -> Manifold {
    o.ring(o.groove_at, o.gasket).extrude(o.gasket)
}

/// Gasket squeeze with the lid down.
pub fn gasket_squeeze(o: &Outdoor) -> f64 {
    (o.gasket - o.groove_d) / o.gasket
}

/// A tripod-thread coupon: two sockets, the box's clearance and 0.15 mm looser, in a block as
/// thick as the box's boss.
pub fn thread_coupon(o: &Outdoor) -> Manifold {
    let block = bx(-16.0, -9.0, 0.0, 16.0, 9.0, o.deck_z - o.deck_t);
    let mark = cyl(1.6, 0.6 + EPS, 13.5, 6.0, o.deck_z - o.deck_t - EPS);
    block
        .difference(&socket_cut(o, o.thread_clear).translate(-8.0, 0.0, 0.0))
        .difference(&socket_cut(o, o.thread_clear + 0.15).translate(8.0, 0.0, 0.0))
        .union(&mark)
}
