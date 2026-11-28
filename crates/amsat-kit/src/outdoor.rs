//! Outdoor ground-station cases: a sealed box with the kit's columns and keyed snap pins inside,
//! so the same plates snap in. The 3000 mAh cell lies in a pocket on a deck under the plate,
//! held in by the plate itself. There are no screws: an O-ring on a skirt under the lid seals
//! sideways against the box's inner wall, so the seal doesn't need clamping, and four reusable
//! snap latches hold the lid on. Each latch is a long finger in the lid's own plane that clicks
//! under a lug on the rim; push its tab inward to let go. Cables come in through glands in the
//! wall that faces down, antennas leave
//! through the wall that faces up, a breathable vent stops condensation, and a 1/4"-20 camera
//! thread in the middle of the back takes a tripod.
//!
//! Print the box, lid and plate in ASA or PETG: PLA softens in sun. Body coordinates: z = 0 is
//! the back's outside face, +y is up when it stands on a tripod with its lid facing out.

use manifold_csg::{CrossSection, Manifold};

use crate::design::Design;
use manifold_csg::JoinType;

use crate::geom::{
    EPS, FACETS, bx, chamfered_square, cone, cyl, offset, poly, rect, round_corners, union_all,
};
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
    /// Rounding on the cavity's corners, so an O-ring can follow them.
    pub cavity_round: f64,
    /// The lid: how far past the cavity its edge reaches, and its thickness.
    pub lid_over: f64,
    pub lid_t: f64,
    /// The skirt under the lid that goes into the box: its height, its clearance to the box's
    /// inner wall, and its wall behind the O-ring groove.
    pub skirt_h: f64,
    pub skirt_clear: f64,
    pub skirt_wall: f64,
    /// O-ring cross-section; the radial gap it's squeezed into (groove bottom to the box's inner
    /// wall); the groove's width at the skirt's face; how far below the lid its centre is.
    pub oring_d: f64,
    pub gland: f64,
    pub groove_w: f64,
    pub groove_z: f64,
    /// How many lid latches: 4 (one on each side) or 2 (on opposite sides, for the coupon).
    pub latches: usize,
    /// Lid latches: finger width, slit behind it, and its length to the lug's
    /// middle and on to its tip, where a thumb pushes it in.
    pub latch_w: f64,
    pub latch_slit: f64,
    pub latch_to_lug: f64,
    pub latch_tab: f64,
    /// The lugs on the rim: width along the side, thickness, gap to the lid's edge, and how far
    /// their catch reaches over the lid.
    pub lug_w: f64,
    pub lug_t: f64,
    pub lug_gap: f64,
    pub lug_reach: f64,
    /// Gap between the lid's top and the lugs' catch.
    pub lug_clear: f64,
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
            latches: 4,
            wall: 3.0,
            floor: 3.0,
            top: 76.0,
            flange: 7.0,
            flange_h: 6.0,
            cavity_round: 5.0,
            lid_over: 5.5,
            lid_t: 4.0,
            skirt_h: 9.0,
            skirt_clear: 0.3,
            skirt_wall: 2.0,
            oring_d: 2.5,
            gland: 1.9,
            groove_w: 5.0,
            groove_z: 4.6,
            latch_w: 2.4,
            latch_slit: 1.4,
            latch_to_lug: 20.0,
            latch_tab: 6.0,
            lug_w: 8.0,
            lug_t: 2.4,
            lug_gap: 0.3,
            lug_reach: 1.0,
            lug_clear: 0.1,
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
        round_corners(
            &chamfered_square(self.cavity, self.cavity_chamfer),
            self.cavity_round,
        )
    }

    /// A crisp, mitred offset of the cavity, for the outside faces.
    pub fn crisp(&self, by: f64) -> CrossSection {
        chamfered_square(self.cavity, self.cavity_chamfer).offset(by, JoinType::Miter, 4.0, 0)
    }

    /// O-ring squeeze, and how full its groove is.
    pub fn oring_squeeze(&self) -> f64 {
        (self.oring_d - self.gland) / self.oring_d
    }

    pub fn groove_depth(&self) -> f64 {
        self.gland - self.skirt_clear
    }

    pub fn groove_fill(&self) -> f64 {
        let depth = self.groove_depth();
        // Flat on the lid side, 45 degrees on the far side, so it prints.
        let area = (self.groove_w + (self.groove_w - depth)) / 2.0 * depth;
        std::f64::consts::PI * self.oring_d * self.oring_d / 4.0 / area
    }

    /// The O-ring's size: the groove bottom's length, and an inside diameter that stretches it
    /// about 3% onto the skirt.
    pub fn oring(&self) -> (f64, f64) {
        let bottom = offset(&self.cavity_outline(), -self.gland, FACETS);
        let length: f64 = bottom
            .to_polygons()
            .iter()
            .map(|p| {
                (0..p.len())
                    .map(|i| {
                        let (a, b) = (p[i], p[(i + 1) % p.len()]);
                        ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt()
                    })
                    .sum::<f64>()
            })
            .fold(0.0, f64::max);
        (length, length / std::f64::consts::PI / 1.03)
    }

    /// Latch strain: the finger bends by the lug's reach (plus 0.1 mm) at the lug.
    pub fn latch_strain(&self) -> f64 {
        1.5 * self.latch_w * (self.lug_reach + 0.1) / self.latch_to_lug.powi(2)
    }

    /// The four latches: each side's outward direction, in degrees.
    pub fn latch_angles(&self) -> Vec<f64> {
        if self.latches == 2 {
            vec![0.0, 180.0]
        } else {
            vec![0.0, 90.0, 180.0, 270.0]
        }
    }

    /// The lid's edge, from the middle.
    pub fn lid_edge(&self) -> f64 {
        self.cavity / 2.0 + self.lid_over
    }

    /// The plate's underside: over the deck, the largest cell and its clearance.
    pub fn plate_z(&self, d: &Design) -> f64 {
        self.deck_z + d.cell_thick() + self.cell_clear
    }

    /// Where the 45 degree slope under the flange starts.
    pub fn flare_z(&self) -> f64 {
        self.top - self.flange_h - self.flange
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

/// A latch's frame: the side's outward direction `a` (degrees); u outward from the middle, v
/// along the side. Builds a box in (u, v, z) and turns it into place.
fn on_side(a: f64, u0: f64, v0: f64, z0: f64, u1: f64, v1: f64, z1: f64) -> Manifold {
    bx(u0, v0, z0, u1, v1, z1).rotate(0.0, 0.0, a)
}

/// The lugs on the rim, each with a flat catch over the lid and a 45 degree lead-in on top.
pub fn lugs(o: &Outdoor) -> Manifold {
    let e = o.lid_edge();
    let (z0, zc) = (o.top - EPS, o.top + o.lid_t + o.lug_clear);
    let (u0, u1) = (e + o.lug_gap, e + o.lug_gap + o.lug_t);
    let lead = 0.5;
    let top = zc + lead + o.lug_reach + o.lug_gap;
    let v = o.latch_to_lug - o.latch_to_lug / 2.0;
    let mut parts = Vec::new();
    for a in o.latch_angles() {
        let (v0, v1) = (v - o.lug_w / 2.0, v + o.lug_w / 2.0);
        // Profile in (u, z): post, then the catch reaching in over the lid.
        let profile = poly(&[
            [u1, z0],
            [u1, top],
            [u0, top],
            [e - o.lug_reach, zc + lead],
            [e - o.lug_reach, zc],
            [u0, zc],
            [u0, z0],
        ]);
        let lug = profile
            .extrude(v1 - v0)
            .rotate(90.0, 0.0, 0.0)
            .translate(0.0, v1, 0.0);
        parts.push(lug.rotate(0.0, 0.0, a));
    }
    union_all(&parts)
}

/// The box's outside: crisp chamfered walls, then the rim's flange over a 45 degree slope.
fn outer_walls(o: &Outdoor) -> Manifold {
    let wall = o.crisp(o.wall);
    let rim = o.crisp(o.wall + o.flange);
    let zf = o.flare_z();
    wall.extrude(zf + EPS).union(&Manifold::batch_hull(&[
        wall.extrude(EPS).translate(0.0, 0.0, zf),
        rim.extrude(o.flange_h)
            .translate(0.0, 0.0, o.top - o.flange_h),
    ]))
}

/// A lead-in round the top of the cavity, so the O-ring slides in.
fn lead_in(o: &Outdoor) -> Manifold {
    let outline = o.cavity_outline();
    Manifold::batch_hull(&[
        offset(&outline, 1.2, FACETS)
            .extrude(EPS)
            .translate(0.0, 0.0, o.top),
        outline.extrude(EPS).translate(0.0, 0.0, o.top - 1.2),
    ])
}

/// The box: walls with crisp chamfered faces, the rim with its lugs, columns, battery pocket,
/// ports and tripod socket. `clear` is the tripod thread's clearance.
pub fn body(d: &Design, o: &Outdoor, clear: f64) -> Manifold {
    let outline = o.cavity_outline();
    let outer = outer_walls(o);
    let lead = lead_in(o);
    let mut cuts = vec![
        outline.extrude(o.top).translate(0.0, 0.0, o.floor),
        lead,
        socket_cut(o, clear),
    ];
    for p in &o.ports {
        cuts.push(port_hole(o, p));
    }
    let shell = outer.difference(&union_all(&cuts)).union(&lugs(o));
    let inside =
        union_all(&[columns(d, o), battery_pocket(d, o)]).difference(&socket_cut(o, clear));
    shell.union(&inside)
}

/// The slits that free each latch finger, in the lid's frame (z = 0 is its underside).
fn latch_slits(o: &Outdoor) -> Manifold {
    let e = o.lid_edge();
    let (w, s) = (o.latch_w, o.latch_slit);
    let (root, tip) = (-o.latch_to_lug / 2.0, o.latch_to_lug / 2.0 + o.latch_tab);
    let mut cuts = Vec::new();
    for a in o.latch_angles() {
        cuts.push(on_side(
            a,
            e - w - s,
            root,
            -1.0,
            e - w,
            tip + s,
            o.lid_t + 1.0,
        ));
        cuts.push(on_side(
            a,
            e - w - s,
            tip,
            -1.0,
            e + 1.0,
            tip + s,
            o.lid_t + 1.0,
        ));
    }
    union_all(&cuts)
}

/// The latch fingers themselves (lid frame), for moving them in the checks.
fn latch_fingers(o: &Outdoor) -> Manifold {
    let e = o.lid_edge();
    let (root, tip) = (-o.latch_to_lug / 2.0, o.latch_to_lug / 2.0 + o.latch_tab);
    let fingers: Vec<Manifold> = o
        .latch_angles()
        .into_iter()
        .map(|a| {
            on_side(
                a,
                e - o.latch_w,
                root + 0.5,
                -EPS,
                e + EPS,
                tip,
                o.lid_t + EPS,
            )
        })
        .collect();
    union_all(&fingers)
}

/// The lid in its own frame: z = 0 is the underside that sits on the rim, the skirt hangs below
/// it. `deflect` pushes the latch fingers inward, for the checks.
pub fn lid(_d: &Design, o: &Outdoor, deflect: f64) -> Result<Manifold, String> {
    let outline = o.cavity_outline();
    let plate = o.crisp(o.lid_over).extrude(o.lid_t);
    let face = offset(&outline, -o.skirt_clear, FACETS);
    let inner = offset(&outline, -(o.gland + o.skirt_wall), FACETS);
    // The skirt, with a lead-in at its bottom edge.
    let skirt = Manifold::batch_hull(&[
        face.extrude(o.skirt_h - 1.0)
            .translate(0.0, 0.0, -(o.skirt_h - 1.0)),
        offset(&face, -1.0, FACETS)
            .extrude(EPS)
            .translate(0.0, 0.0, -o.skirt_h),
    ])
    .difference(
        &inner
            .extrude(o.skirt_h + 2.0)
            .translate(0.0, 0.0, -o.skirt_h - 1.0),
    );
    // The O-ring groove: flat on the lid's side, 45 degrees on the far side.
    let depth = o.groove_depth();
    let (g0, g1) = (
        -o.groove_z + o.groove_w / 2.0,
        -o.groove_z - o.groove_w / 2.0,
    );
    let ring = |by: f64| offset(&outline, by, FACETS);
    let bottom = ring(-o.gland);
    let groove = Manifold::batch_hull(&[
        ring(1.0)
            .extrude(g0 - (g1 + depth))
            .translate(0.0, 0.0, g1 + depth),
        ring(1.0).extrude(EPS).translate(0.0, 0.0, g1),
    ])
    .difference(
        &bottom
            .extrude(o.groove_w + 2.0)
            .translate(0.0, 0.0, g1 - 1.0),
    );
    let grooved = skirt.difference(&groove);
    // The branding is a second-colour inlay in the top face (see `lid_brand`).
    let body = plate.union(&grooved).difference(&latch_slits(o));
    if deflect == 0.0 {
        return Ok(body);
    }
    let fingers = body.intersection(&latch_fingers(o));
    let mut moved = Vec::new();
    for a in o.latch_angles() {
        let e = o.lid_edge();
        let one = fingers.intersection(&on_side(a, e - 10.0, -30.0, -50.0, e + 10.0, 30.0, 50.0));
        let r = a.to_radians();
        moved.push(one.translate(-deflect * r.cos(), -deflect * r.sin(), 0.0));
    }
    Ok(body.difference(&latch_fingers(o)).union(&union_all(&moved)))
}

/// The lid in place on the box.
pub fn lid_in_place(d: &Design, o: &Outdoor, deflect: f64) -> Result<Manifold, String> {
    Ok(lid(d, o, deflect)?.translate(0.0, 0.0, o.top))
}

/// The lid as it prints: top face down, skirt up.
pub fn lid_to_print(d: &Design, o: &Outdoor) -> Result<Manifold, String> {
    Ok(lid(d, o, 0.0)?.rotate(180.0, 0.0, 0.0))
}

/// The lid's logo inlay, in print coordinates: the mark and words in the middle of the top
/// face, which prints on the bed.
pub fn lid_brand(d: &Design, o: &Outdoor) -> Result<Manifold, String> {
    Ok(branding(d)?
        .extrude(d.letter_h + 1.0)
        .translate(0.0, 0.0, o.lid_t - d.letter_h)
        .rotate(180.0, 0.0, 0.0))
}

/// True if every latch finger, its slit and its lug sit on straight parts of the lid's edge and
/// the rim, clear of the corner chamfers by at least 1 mm.
pub fn latches_on_edges(o: &Outdoor) -> bool {
    let e = o.lid_edge();
    let (root, tip) = (-o.latch_to_lug / 2.0, o.latch_to_lug / 2.0 + o.latch_tab);
    let lid_face = o.crisp(o.lid_over);
    let rim = o.crisp(o.wall + o.flange);
    let v = o.latch_to_lug / 2.0;
    o.latch_angles().into_iter().all(|a| {
        // The finger and its slits, from the slit's inside edge out to the lid's edge, and 1 mm
        // more along the edge each way.
        let finger = rect(
            e - o.latch_w - o.latch_slit,
            root - 1.0,
            e - 0.01,
            tip + o.latch_slit + 1.0,
        )
        .rotate(a);
        let lug = rect(
            e + 0.01,
            v - o.lug_w / 2.0 - 1.0,
            e + o.lug_gap + o.lug_t,
            v + o.lug_w / 2.0 + 1.0,
        )
        .rotate(a);
        finger.difference(&lid_face).area() < 1e-3 && lug.difference(&rim).area() < 1e-3
    })
}

/// The latch coupon's case: a small square box with the real wall, rim, flange, lugs and
/// lead-in, and a small lid with the real skirt, O-ring groove and latch fingers, on two opposite
/// sides. It goes on and comes off exactly like the real lid: the skirt holds it square, so
/// the lugs bend the fingers instead of pushing the lid aside.
pub fn latch_coupon_case(o: &Outdoor) -> Outdoor {
    Outdoor {
        // As small as the latches allow: each finger and lug needs a straight edge.
        cavity: 40.0,
        cavity_chamfer: 3.0,
        cavity_round: 2.0,
        latches: 2,
        // Only as tall as the skirt, with the thinnest wall the guide allows and a short
        // flange over its 45 degree slope. No floor: it's an open ring, to save plastic.
        wall: 2.4,
        flange_h: 2.0,
        top: o.skirt_h + 1.0,
        floor: -1.0,
        ports: Vec::new(),
        ..o.clone()
    }
}

/// Where the coupon keeps its full lid and rim: a strip along each latched side, as long as
/// the finger and its lug with 3 mm to spare. Elsewhere the rim is just the wall, and the lid
/// just covers it.
fn latch_strips(c: &Outdoor, from: f64) -> CrossSection {
    let (root, tip) = (-c.latch_to_lug / 2.0, c.latch_to_lug / 2.0 + c.latch_tab);
    let mut strips = CrossSection::empty();
    for a in c.latch_angles() {
        strips =
            strips.union(&rect(from, root - 3.0, from + 20.0, tip + c.latch_slit + 3.0).rotate(a));
    }
    strips
}

/// The coupon's box, as it prints: rim up. It has no floor, and its flange is only where the
/// lugs stand.
pub fn latch_coupon_box(c: &Outdoor) -> Manifold {
    let cuts = union_all(&[
        c.cavity_outline()
            .extrude(c.top)
            .translate(0.0, 0.0, c.floor),
        lead_in(c),
    ]);
    let keep = c
        .crisp(c.wall)
        .union(&latch_strips(c, c.cavity / 2.0))
        .extrude(c.top + 1.0)
        .translate(0.0, 0.0, -0.5);
    outer_walls(c)
        .intersection(&keep)
        .difference(&cuts)
        .union(&lugs(c))
}

/// The coupon's lid in its own frame: the real lid's edge, latches and skirt, with its middle
/// open inside the skirt, to save plastic.
pub fn latch_coupon_lid(d: &Design, c: &Outdoor, deflect: f64) -> Result<Manifold, String> {
    // The open middle, less a patch on one plain side for the logo.
    let inner = c.cavity / 2.0 - c.gland - c.skirt_wall;
    let middle = offset(&c.cavity_outline(), -(c.gland + c.skirt_wall), FACETS)
        .difference(&rect(-7.0, inner - 9.0, 7.0, inner + 1.0))
        .extrude(c.lid_t + 2.0)
        .translate(0.0, 0.0, -1.0);
    // Away from the latches the lid only needs to cover the wall.
    let keep = c
        .crisp(c.wall - 0.5)
        .union(&latch_strips(c, c.cavity / 2.0))
        .extrude(c.lid_t + c.skirt_h + 4.0)
        .translate(0.0, 0.0, -c.skirt_h - 2.0);
    Ok(lid(d, c, deflect)?.difference(&middle).intersection(&keep))
}

/// The latch coupon: its box and its lid (top face down, as it prints). `o` is the real case.
pub fn latch_coupon(d: &Design, o: &Outdoor) -> Result<(Manifold, Manifold), String> {
    let c = latch_coupon_case(o);
    Ok((
        latch_coupon_box(&c),
        latch_coupon_lid(d, &c, 0.0)?.rotate(180.0, 0.0, 0.0),
    ))
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
