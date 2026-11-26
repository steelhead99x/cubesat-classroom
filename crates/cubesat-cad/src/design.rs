//! Every dimension of the Rev C frame, in one place, in mm.
//!
//! Coordinates follow the CubeSat Design Specification (CDS) Rev 14.1 §2.2.1:
//! origin at the geometric centre, Z along the 113.5 mm length, and the -Z face
//! goes into the deployer first. The two side panels sit at ±Y, each carrying two
//! corner rails. Two covers close the ±X faces; each has a port for the USB plugs
//! and the inhibit jumper, and slots to vent the inside during launch.
//!
//! The payload plate is modelled in its own frame (plate underside at z = 0, board
//! side up) and moved into place by [`Design::plate_bottom_z`]. Its board mounts,
//! latches and fasteners are the AMSAT kit's (hardware/amsat-cubesatsim/generate.py),
//! with the same numbers, so both builds share one tested design.

/// Which Uno edge a latch sits on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

/// Uno snap latch: a cantilever arm beside a board edge with an angled hook over the
/// board, and a backstop that a lock key drops in against.
#[derive(Clone, Debug)]
pub struct Latch {
    pub w: f64,
    pub t: f64,            // arm thickness: three lines with a 0.6 mm nozzle
    pub gap: f64,          // board edge to arm, relaxed
    pub reach: f64,        // how far the hook reaches over the board
    pub root_depth: f64,   // arm starts in a pocket this deep in the plate
    pub angle: f64,        // hook underside, degrees: inward spring becomes a downward force
    pub preload: f64,      // arm stays bent outward this much with the board seated
    pub lip_t: f64,        // hook thickness at its tip
    pub strain_limit: f64, // the arm prints upright, so it bends across its layers
    pub min_reach: f64,
    /// (edge, centre along that edge from the Uno's lower-left corner, gusset depth).
    pub places: [(Edge, f64, f64); 4],
}

/// Lock key: clips in behind a latch arm so it can't open.
#[derive(Clone, Debug)]
pub struct Key {
    pub t: f64,
    pub w: f64,
    pub clear: f64, // each side of the key in its slot
    pub slot: f64,  // split between the two detent prongs
    pub prong: f64,
    pub bump: f64, // detent bump on each prong, 45 degree flanks
    pub notch_z: f64,
    pub tab: f64, // above the latch, with a hole for a pull ribbon
    pub cheek_t: f64,
    pub back_t: f64,
}

/// Quarter-turn fastener: a stud with a spiral cam slot that rides over a cross-bar.
/// A generic push-and-turn panel fastener, described without brand names.
#[derive(Clone, Debug)]
pub struct QuarterTurn {
    pub d: f64,
    pub bore: f64,
    pub bar_w: f64,
    pub bar_h: f64,
    pub bar_z: f64, // bar's near face below the receptacle's entry face
    pub clear: f64, // slot clearance around the bar
    pub lift: f64,  // cam draw-in over the first 75 degrees
    pub det: f64,   // detent: extra push needed to turn back out
    pub tip: f64,   // stud material behind the bar when locked
    pub head_d: f64,
    pub head_h: f64,
}

/// Spring washer printed flat: two spiral arms join a hub to a ring on three feet.
#[derive(Clone, Debug)]
pub struct Washer {
    pub od: f64,
    pub id: f64,
    pub t: f64, // arm thickness, five 0.3 mm layers
    pub arm: f64,
    pub slit: f64,
    pub sweep: f64, // degrees each arm wraps
    pub foot: f64,
}

/// Arduino Nano 33 BLE Sense Rev2, bare board, standing on its long edge in V-grooves
/// like a card in a card cage. Parts face +Y, USB end towards -X. `x0` is the USB
/// end, `y0` the back face, `z0` the lower edge, in plate coordinates.
#[derive(Clone, Debug)]
pub struct NanoCage {
    pub l: f64,
    pub w: f64,
    pub t: f64,
    pub x0: f64,
    pub y0: f64,
    pub z0: f64,
    pub end_play: f64,
    pub v_mouth: f64,
    pub v_apex: f64,
    pub v_depth: f64,
    pub v_wall: f64,
    pub min_seat: f64,
    pub edge_clear: f64,
    pub stop_reach: f64,
    pub plug: [f64; 3], // Micro-USB plug envelope: length, thickness, width
    pub min_plug_room: f64,
    pub preload: f64, // the Nano's top edge lifts its clamp this much
    pub pillar_od: f64,
    pub clamp_t: f64,
    pub anchor: [f64; 2], // clamp pillar near the USB end
    pub far_pillar: [f64; 2],
}

/// One full set of design parameters. `Design::default()` is Rev C as printed.
/// The checks take a `Design`, so a test can change one number and confirm the
/// checks notice.
#[derive(Clone, Debug)]
pub struct Design {
    // CDS envelope (Rev 14.1 §2.2.1, Appendix B): 100 x 100 x 113.5 mm.
    pub width: f64,
    pub length: f64,

    // Rails (§2.2.5 at least 8.5 mm wide; §2.2.7 edges rounded at least 1 mm).
    pub rail: f64,
    pub rail_radius: f64,
    // Panel and cover faces sit this far inside the rail planes, so only the rails
    // can touch the deployer walls.
    pub face_recess: f64,

    // Side panel.
    pub web_t: f64,
    pub window: [f64; 2], // open window in each side panel, X by Z
    pub lug_t: f64,       // thickness (Z) of the end walls the end plates clip to
    pub lug_inner_y: f64, // how far towards the centre the end walls reach

    // End plates. Recessed from the rail ends so only the rails touch at ±Z (§2.2.8).
    pub end_recess: f64,
    pub end_t: f64,
    pub end_gap: f64, // clearance to the rails and panel webs
    pub boss_d: f64,
    pub boss_h: f64,
    pub peg_x: f64, // snap points at (±peg_x, ±peg_y) on each end
    pub peg_y: f64,
    pub top_window_d: f64, // opening in the +Z plate
    pub root_chamfer: f64, // cone at the root of every column and boss

    // ±X covers. Each clips to both end plates like a side panel.
    pub cover_peg: [f64; 2], // snap points at (±x, ±y) on each end
    pub cover_lug_half: f64, // half-span (Y) of a cover's end walls
    // Port in the -X cover for both USB plugs and the D2 jumper, in CDS Y and Z.
    // It runs out through the cover's -Y edge, beside the Nano's plug.
    pub port_y: [f64; 2],
    pub port_z: [f64; 2],
    pub vent: [f64; 2], // each vent slot, Y by Z
    pub vent_count: usize,
    pub vent_center_z: f64,

    // Snap peg (first printed on a P1S with a 0.6 mm nozzle: latched well).
    pub peg_shank: f64,
    pub peg_barb: f64,
    pub peg_tip: f64,
    pub peg_lip: f64,
    pub peg_lead: f64,
    pub peg_slot: f64,
    pub snap_hole: f64,
    pub slot_into_body: f64,

    // Payload plate on four hollow columns from the -Z end plate, held by
    // quarter-turn studs that go down through the plate into the columns.
    pub column_od: f64,
    pub column_length: f64,
    pub columns: [[f64; 2]; 4],
    pub plate: [f64; 2], // X by Y
    pub plate_t: f64,

    // Arduino Uno R3 (Eagle UNO-TH_Rev3e.brd), holes from its lower-left corner,
    // USB on the left (-X) edge.
    pub uno: [f64; 2],
    pub uno_holes: [[f64; 2]; 4],
    pub uno_top_margin: f64, // plate edge to the Uno's top edge: room for the top latch
    pub uno_post_h: f64,
    pub uno_post_top: f64,
    pub uno_pin: f64,
    pub uno_pin_above: f64, // pin tip relative to the board's top face
    pub pcb_uno: f64,

    pub latch: Latch,
    pub key: Key,
    pub qt: QuarterTurn,
    pub washer: Washer,
    pub nano: NanoCage,

    // Loads and limits used by the checks.
    pub payload_mass: f64, // kg on the plate: plate, boards, clamp, keys (estimate)
    pub design_g: f64,     // quasi-static design load
    pub pla_strain_limit: f64,
    pub min_snap_overlap: f64,
    pub min_board_clearance: f64,
}

impl Default for Design {
    fn default() -> Self {
        Self {
            width: 100.0,
            length: 113.5,
            rail: 9.0,
            rail_radius: 1.0,
            face_recess: 0.5,
            web_t: 2.4,
            window: [60.0, 80.0],
            lug_t: 3.0,
            lug_inner_y: 36.0,
            end_recess: 1.5,
            end_t: 3.0,
            end_gap: 0.3,
            boss_d: 9.0,
            boss_h: 4.0,
            peg_x: 25.0,
            peg_y: 42.0,
            top_window_d: 30.0,
            root_chamfer: 1.5,
            cover_peg: [42.0, 25.0],
            cover_lug_half: 35.7,
            port_y: [-42.0, 31.0],
            port_z: [-31.0, -8.0],
            vent: [4.0, 30.0],
            vent_count: 7,
            vent_center_z: 25.0,
            peg_shank: 5.0,
            peg_barb: 5.7,
            peg_tip: 4.0,
            peg_lip: 0.8,
            peg_lead: 1.0,
            peg_slot: 1.4,
            snap_hole: 5.2,
            slot_into_body: 4.0,
            column_od: 10.0,
            column_length: 15.0,
            columns: [[-22.0, 21.0], [18.0, 21.0], [-8.0, -6.0], [14.0, -24.0]],
            plate: [80.0, 88.0],
            plate_t: 3.0,
            uno: [68.58, 53.34],
            uno_holes: [[13.97, 2.54], [15.24, 50.80], [66.04, 7.62], [66.04, 35.56]],
            uno_top_margin: 7.16,
            uno_post_h: 10.0,
            uno_post_top: 4.4,
            uno_pin: 2.9,
            uno_pin_above: -0.3,
            pcb_uno: 1.6,
            latch: Latch {
                w: 8.0,
                t: 1.8,
                gap: 0.3,
                reach: 0.8,
                root_depth: 1.5,
                angle: 20.0,
                preload: 0.15,
                lip_t: 1.0,
                strain_limit: 0.015,
                min_reach: 0.6,
                places: [
                    (Edge::Left, 21.0, 1.5),  // between the DC jack and the USB-B
                    (Edge::Right, 16.0, 1.5), // between hole 2 and the ICSP header
                    (Edge::Top, 14.5, 0.0),   // between the reset button and the 8-13 header
                    (Edge::Bottom, 6.0, 0.0), // beside the DC jack
                ],
            },
            key: Key {
                t: 1.8,
                w: 6.0,
                clear: 0.1,
                slot: 1.4,
                prong: 9.0,
                bump: 0.35,
                notch_z: 2.0,
                tab: 4.5,
                cheek_t: 1.25,
                back_t: 2.4,
            },
            qt: QuarterTurn {
                d: 6.0,
                bore: 6.4,
                bar_w: 2.0,
                bar_h: 1.6,
                bar_z: 0.7,
                clear: 0.3,
                lift: 0.55,
                det: 0.2,
                tip: 1.6,
                head_d: 10.0,
                head_h: 2.5,
            },
            washer: Washer {
                od: 18.0,
                id: 6.8,
                t: 1.5,
                arm: 1.6,
                slit: 0.8,
                sweep: 150.0,
                foot: 1.2,
            },
            nano: NanoCage {
                l: 43.18,
                w: 17.78,
                t: 1.59,
                x0: -18.0,
                y0: -40.0,
                z0: 4.5,
                end_play: 0.5,
                v_mouth: 2.4,
                v_apex: 1.4,
                v_depth: 2.0,
                v_wall: 2.6,
                min_seat: 1.0,
                edge_clear: 2.8,
                stop_reach: 3.0,
                plug: [28.0, 7.0, 10.5],
                min_plug_room: 20.0,
                preload: 0.15,
                pillar_od: 10.0,
                clamp_t: 4.0,
                anchor: [-14.0, -29.2],
                far_pillar: [30.0, -29.2],
            },
            payload_mass: 0.10,
            design_g: 20.0,
            pla_strain_limit: 0.02,
            min_snap_overlap: 0.2,
            min_board_clearance: 0.25,
        }
    }
}

impl Design {
    /// Half the width (X and Y).
    pub fn half(&self) -> f64 {
        self.width / 2.0
    }
    /// Half the length (Z): the rail ends.
    pub fn half_z(&self) -> f64 {
        self.length / 2.0
    }
    /// Outer face of each side panel's web and each cover.
    pub fn face_outer(&self) -> f64 {
        self.half() - self.face_recess
    }
    /// Inner face of each side panel's web and each cover.
    pub fn web_inner_y(&self) -> f64 {
        self.face_outer() - self.web_t
    }
    /// Rail inner faces.
    pub fn rail_inner(&self) -> f64 {
        self.half() - self.rail
    }
    /// The +Z end plate's outer and inner faces (the -Z plate mirrors them).
    pub fn end_outer_z(&self) -> f64 {
        self.half_z() - self.end_recess
    }
    pub fn end_inner_z(&self) -> f64 {
        self.end_outer_z() - self.end_t
    }
    /// The outer face of the end walls (lugs) on the side panels, at +Z.
    pub fn lug_outer_z(&self) -> f64 {
        self.end_inner_z() - self.boss_h
    }
    /// Snap points on each end, in XY.
    pub fn peg_points(&self) -> [[f64; 2]; 4] {
        let (x, y) = (self.peg_x, self.peg_y);
        [[-x, y], [x, y], [-x, -y], [x, -y]]
    }
    /// Snap points for the ±X covers on each end.
    pub fn cover_peg_points(&self) -> [[f64; 2]; 4] {
        let [x, y] = self.cover_peg;
        [[x, y], [x, -y], [-x, y], [-x, -y]]
    }
    /// Every snap point on an end plate: side panels, then covers.
    pub fn all_peg_points(&self) -> [[f64; 2]; 8] {
        let (a, b) = (self.peg_points(), self.cover_peg_points());
        [a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3]]
    }
    /// Vent slot centres in Y on each cover.
    pub fn vent_ys(&self) -> Vec<f64> {
        let n = self.vent_count;
        let pitch = 2.0 * self.vent[0];
        (0..n)
            .map(|i| (i as f64 - (n as f64 - 1.0) / 2.0) * pitch)
            .collect()
    }
    /// Payload plate underside, in CDS coordinates: the plate's own frame starts here.
    pub fn plate_bottom_z(&self) -> f64 {
        -self.end_inner_z() + self.column_length
    }
    pub fn plate_top_z(&self) -> f64 {
        self.plate_bottom_z() + self.plate_t
    }
    /// Uno lower-left corner in XY.
    pub fn uno_origin(&self) -> [f64; 2] {
        [
            -self.uno[0] / 2.0,
            self.plate[1] / 2.0 - self.uno_top_margin - self.uno[1],
        ]
    }
    pub fn uno_hole(&self, i: usize) -> [f64; 2] {
        let o = self.uno_origin();
        [o[0] + self.uno_holes[i][0], o[1] + self.uno_holes[i][1]]
    }
    /// Uno board, bottom and top faces, in plate coordinates.
    pub fn uno_pcb_bottom(&self) -> f64 {
        self.plate_t + self.uno_post_h
    }
    pub fn uno_top(&self) -> f64 {
        self.uno_pcb_bottom() + self.pcb_uno
    }
    /// How deep each Nano edge sits in its V-groove.
    pub fn nano_seat(&self) -> f64 {
        let n = &self.nano;
        n.v_depth * (n.v_mouth - n.t) / (n.v_mouth - n.v_apex)
    }
    /// Mouth of the lower groove and the clamp's groove, in plate coordinates.
    pub fn base_mouth_z(&self) -> f64 {
        self.nano.z0 + self.nano_seat()
    }
    pub fn cap_mouth_z(&self) -> f64 {
        self.nano.z0 + self.nano.w - self.nano_seat()
    }
    /// The clamp's underside as built, resting on its pillars.
    pub fn clamp_bottom_z(&self) -> f64 {
        self.cap_mouth_z() - self.nano.preload
    }
    pub fn pillars(&self) -> [[f64; 2]; 2] {
        [self.nano.anchor, self.nano.far_pillar]
    }
    /// Washer height under a stud head.
    pub fn washer_h(&self) -> f64 {
        self.washer.t + self.washer.foot
    }
    /// Stud head underside to its bar, washer at rest, for the plate and clamp studs.
    pub fn plate_stud_a0(&self) -> f64 {
        self.washer_h() + self.plate_t + self.qt.bar_z
    }
    pub fn clamp_stud_a0(&self) -> f64 {
        self.washer_h() + self.nano.clamp_t + self.qt.bar_z
    }
}
