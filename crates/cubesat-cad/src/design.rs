//! Every dimension of the Rev C frame, in one place, in mm.
//!
//! Coordinates follow the CubeSat Design Specification (CDS) Rev 14.1 §2.2.1:
//! origin at the geometric centre, Z along the 113.5 mm length, and the -Z face
//! goes into the deployer first. The two side panels sit at ±Y, each carrying two
//! corner rails. The ±X faces stay open for access to the boards and the inhibit.

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

    // Snap peg (first printed on a P1S with a 0.6 mm nozzle: latched well).
    pub peg_shank: f64,
    pub peg_barb: f64,
    pub peg_tip: f64,
    pub peg_lip: f64,
    pub peg_lead: f64,
    pub peg_slot: f64,
    pub snap_hole: f64,
    pub slot_into_body: f64,

    // Payload plate on four columns from the -Z end plate.
    pub column_d: f64,
    pub column_length: f64,
    pub columns: [[f64; 2]; 4],
    pub plate: [f64; 2], // X by Y
    pub plate_t: f64,

    // Arduino Uno R3, holes from its lower-left corner, USB on the left (-X) edge.
    pub uno: [f64; 2],
    pub uno_holes: [[f64; 2]; 4],
    pub uno_top_margin: f64,
    pub uno_post_h: f64,
    pub uno_post_top: f64,
    pub uno_pin: f64,
    pub pcb_uno: f64,
    pub clip_t: f64,
    pub clip_w: f64,
    pub clip_gap: f64,
    pub clip_lip: f64,

    // Arduino Nano 33 BLE Sense Rev2 with headers, pins down.
    pub nano: [f64; 2],
    pub nano_center: [f64; 2],
    pub nano_row: f64,
    pub nano_rail_h: f64,
    pub pcb_nano: f64,
    pub strap_arch: f64,
    pub strap_foot_above_pcb: f64,
    pub strap_t: f64,
    pub strap_w: f64,
    pub strap_post: f64,

    // Limits used by the checks.
    pub pla_strain_limit: f64,
    pub min_snap_overlap: f64,
    pub min_clip_reach: f64,
}

impl Default for Design {
    fn default() -> Self {
        Self {
            width: 100.0,
            length: 113.5,
            rail: 9.0,
            rail_radius: 1.0,
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
            peg_shank: 5.0,
            peg_barb: 5.7,
            peg_tip: 4.0,
            peg_lip: 0.8,
            peg_lead: 1.0,
            peg_slot: 1.4,
            snap_hole: 5.2,
            slot_into_body: 4.0,
            column_d: 7.0,
            column_length: 15.0,
            columns: [[-36.0, 26.0], [24.0, 30.0], [-36.0, -26.0], [36.0, -26.0]],
            plate: [80.0, 88.0],
            plate_t: 3.0,
            uno: [68.58, 53.34],
            uno_holes: [[13.97, 2.54], [15.24, 50.80], [66.04, 7.62], [66.04, 35.56]],
            uno_top_margin: 1.0,
            uno_post_h: 10.0,
            uno_post_top: 4.4,
            uno_pin: 2.9,
            pcb_uno: 1.6,
            clip_t: 1.8,
            clip_w: 8.0,
            clip_gap: 0.3,
            clip_lip: 0.7,
            nano: [43.18, 17.78],
            nano_center: [-2.0, -28.0],
            nano_row: 15.24,
            nano_rail_h: 20.0,
            pcb_nano: 1.6,
            strap_arch: 3.0,
            strap_foot_above_pcb: 1.0,
            strap_t: 3.0,
            strap_w: 7.0,
            strap_post: 12.0,
            pla_strain_limit: 0.02,
            min_snap_overlap: 0.2,
            min_clip_reach: 0.3,
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
    /// Inner face of each side panel's web.
    pub fn web_inner_y(&self) -> f64 {
        self.half() - self.web_t
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
    /// Payload plate: the side facing the -Z end plate, and the board side.
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
    pub fn uno_pcb_bottom_z(&self) -> f64 {
        self.plate_top_z() + self.uno_post_h
    }
    pub fn nano_pcb_bottom_z(&self) -> f64 {
        self.plate_top_z() + self.nano_rail_h
    }
    pub fn strap_bottom_z(&self) -> f64 {
        self.nano_pcb_bottom_z() + self.pcb_nano + self.strap_arch
    }
}
