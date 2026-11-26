//! Every dimension of the AMSAT payload kit, in one place, in mm.
//!
//! `Design::default()` is the kit as built. The checks take a `Design`, so a test can change one
//! number and confirm the checks notice.
//!
//! Two frames of reference:
//! - plate coordinates: payload plate underside at z = 0, board side up (how it prints);
//! - frame coordinates: the AMSAT end piece's face from z = 0 to +3, the CubeSat towards -z.
//!
//! `parts::to_frame` converts from one to the other.

/// A board edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

/// A joint on one edge of a board: which edge, where along it (from the board's own lower-left
/// corner, along +x or +y), and for a snap hook, which way its finger runs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    pub side: Side,
    pub along: f64,
    /// +1: the finger's root is further along the edge than its hook.
    pub root: i8,
}

impl Edge {
    pub const fn new(side: Side, along: f64, root: i8) -> Self {
        Self { side, along, root }
    }
}

/// x0, y0, width, height.
pub type Rect = [f64; 4];

#[derive(Clone, Debug)]
pub struct Design {
    // AMSAT end piece. Stack mounting holes measured from the v2.0 STL (frame coordinates).
    pub stack_holes: [[f64; 2]; 4],
    pub face_t: f64,
    /// AMSAT snap tabs reach this far below the face.
    pub snap_tabs_reach: f64,
    /// Its top is the shoulder the plate sits on.
    pub column_od: f64,
    /// Face to plate; clears the snap tabs.
    pub column_length: f64,

    /// AMSAT boards are 90.5 mm square.
    pub plate: f64,
    pub plate_chamfer: f64,
    pub plate_t: f64,

    // Snap-fit design rules.
    /// 15% of PLA's ~8% elongation at break.
    pub strain_limit: f64,
    /// Nothing that flexes or carries load is thinner.
    pub min_wall: f64,
    /// MPa.
    pub pla_e: f64,
    /// PLA on PLA or on a circuit board.
    pub friction: f64,
    /// kg on the plate: plate and both boards (estimate).
    pub payload_mass: f64,
    /// Quasi-static design load for small-satellite launch.
    pub design_g: f64,
    pub retention_margin: f64,

    // Snap pins (plate to frame): a pin with two flats and a 45 degree head; a keyed slot in the
    // plate with rigid walls on the flats and two straight flex fingers either side.
    pub pin_r: f64,
    pub pin_head_r: f64,
    /// The head's 45 degree face starts this far below the plate's top.
    pub pin_bite: f64,
    pub pin_cap: f64,
    /// Radius at the top of the lead-in cone.
    pub pin_tip_r: f64,
    /// Lead-in slope from vertical, degrees: gentler than 45 so the head starts the fingers apart
    /// without a hard push.
    pub pin_lead_angle: f64,
    /// Half the distance across the flats.
    pub pin_flat: f64,
    /// Clearance between each flat and its wall: the fit. The fit-ladder coupon tries others.
    pub pin_fit: f64,
    pub locate_clear: f64,
    /// Locating pin above the plate: low enough for the Nano's USB plug to pass over it.
    pub locate_above: f64,
    /// Head to the fingers' inner faces.
    pub finger_clear: f64,
    pub finger_w: f64,
    /// Behind each finger: room to bend.
    pub finger_slit: f64,
    /// Between each finger and the rigid tongue between them.
    pub finger_slit_in: f64,
    /// Root to tip.
    pub finger_len: f64,
    pub finger_tip_gap: f64,
    /// Where each tooth's face crosses the lobe axis, at rest.
    pub tooth_r: f64,
    /// Each tooth runs this far either side of the lobe axis.
    pub tooth_half: f64,
    /// Stack holes with snap pins (the one by the Nano locates), with each joint's angle: the
    /// way its fingers' roots point is this angle less 90 degrees.
    pub snap_columns: Vec<(usize, f64)>,

    // Board snap hooks: a straight flex finger cut into the plate beside a board edge, with a
    // post on its tip and a hook over the board. The finger bends in the plate's plane.
    pub hook_len: f64,
    /// Post thickness across the edge; the finger is 0.2 mm wider.
    pub hook_w: f64,
    pub hook_post_w: f64,
    pub hook_slit_in: f64,
    pub hook_slit_out: f64,
    pub hook_tip_gap: f64,
    /// Board edge to post.
    pub hook_gap: f64,
    pub hook_reach: f64,
    /// Hook underside, degrees: under the friction angle, so pulling the board up can't push
    /// the hook open.
    pub hook_angle: f64,
    /// The finger stays bent outward this much with the board in.
    pub hook_preload: f64,
    pub hook_lip_t: f64,

    // Fixed lips: a post with a lip whose 45 degree underside the board's edge tucks under.
    pub lip_w: f64,
    pub lip_post_t: f64,
    pub lip_reach: f64,
    /// Lip underside above the board's edge, seated.
    pub lip_gap: f64,
    pub lip_t: f64,

    // Arduino Uno R3 (A000066, Eagle UNO-TH_Rev3e.brd), from its lower-left corner, USB left.
    pub uno: [f64; 2],
    pub uno_holes: [[f64; 2]; 4],
    pub uno_origin: [f64; 2],
    /// 2 mm for its solder joints, over the pin heads.
    pub uno_post_h: f64,
    pub uno_post_top: f64,
    pub uno_post_top_h: f64,
    pub uno_pin: f64,
    /// Pin tip relative to the board's top face.
    pub uno_pin_above: f64,
    pub pcb_uno: f64,
    pub uno_lips: Vec<Edge>,
    pub uno_hooks: Vec<Edge>,

    // Arduino Nano 33 BLE Sense Rev2, bare (ABX00069): flat on pads, parts up, USB towards +x.
    pub nano: [f64; 2],
    pub pcb_nano: f64,
    /// The board's USB end.
    pub nano_usb_x: f64,
    /// The board's -y edge.
    pub nano_y0: f64,
    /// Pads under the board, so its USB plug clears the plate.
    pub nano_pad_h: f64,
    pub nano_end_play: f64,
    pub nano_stop_t: f64,
    pub nano_lips: Vec<Edge>,
    pub nano_hooks: Vec<Edge>,
    /// Micro-USB plug envelope: length, thickness, width.
    pub plug: [f64; 3],
    pub min_plug_room: f64,

    // Branding: the CubeSat Classroom mark and lettering, raised for a second colour.
    pub letter_h: f64,
    pub mark_size: f64,
    pub mark_at: [f64; 2],
}

impl Default for Design {
    fn default() -> Self {
        use Side::{Bottom, Left, Right, Top};
        Self {
            stack_holes: [
                [-32.99, 32.99],
                [-32.71, -25.01],
                [29.0, -38.99],
                [29.0, 31.51],
            ],
            face_t: 3.0,
            snap_tabs_reach: 12.0,
            column_od: 10.0,
            column_length: 15.0,
            plate: 88.0,
            plate_chamfer: 8.0,
            plate_t: 3.0,
            strain_limit: 0.012,
            min_wall: 2.0,
            pla_e: 3500.0,
            friction: 0.3,
            payload_mass: 0.08,
            design_g: 20.0,
            retention_margin: 1.5,
            pin_r: 3.0,
            pin_head_r: 3.7,
            pin_bite: 0.35,
            pin_cap: 0.6,
            pin_tip_r: 2.5,
            pin_lead_angle: 30.0,
            pin_flat: 2.4,
            pin_fit: 0.2,
            locate_clear: 0.4,
            locate_above: 0.5,
            finger_clear: 0.3,
            finger_w: 2.0,
            finger_slit: 1.1,
            finger_slit_in: 0.8,
            finger_len: 14.0,
            finger_tip_gap: 0.7,
            tooth_r: 3.1,
            tooth_half: 1.3,
            snap_columns: vec![(0, 183.0), (1, 8.0), (2, 356.5)],
            hook_len: 18.0,
            hook_w: 2.0,
            hook_post_w: 4.0,
            hook_slit_in: 1.0,
            hook_slit_out: 1.4,
            hook_tip_gap: 1.2,
            hook_gap: 0.3,
            hook_reach: 0.8,
            hook_angle: 10.0,
            hook_preload: 0.1,
            hook_lip_t: 1.0,
            lip_w: 6.0,
            lip_post_t: 2.0,
            lip_reach: 0.8,
            lip_gap: 0.05,
            lip_t: 1.2,
            uno: [68.58, 53.34],
            uno_holes: [[66.04, 35.56], [66.04, 7.62], [15.24, 50.80], [13.97, 2.54]],
            uno_origin: [-35.66, 32.0 - 53.34],
            uno_post_h: 5.5,
            uno_post_top: 4.4,
            uno_post_top_h: 2.5,
            uno_pin: 2.9,
            uno_pin_above: -0.3,
            pcb_uno: 1.6,
            // Between the DC jack and the USB-B.
            uno_lips: vec![Edge::new(Left, 21.0, 1)],
            // Between holes 1 and 0; past hole 2, over the 8-13 header's clear strip.
            uno_hooks: vec![Edge::new(Right, 15.0, 1), Edge::new(Top, 40.0, -1)],
            nano: [43.18, 17.78],
            pcb_nano: 1.59,
            nano_usb_x: 22.5,
            nano_y0: -41.6,
            nano_pad_h: 1.5,
            nano_end_play: 0.5,
            nano_stop_t: 2.0,
            nano_lips: vec![Edge::new(Top, 4.7, 1), Edge::new(Top, 35.7, 1)],
            nano_hooks: vec![Edge::new(Bottom, 4.7, 1), Edge::new(Bottom, 38.7, -1)],
            plug: [28.0, 7.0, 10.5],
            min_plug_room: 20.0,
            letter_h: 0.6,
            mark_size: 17.0,
            mark_at: [0.0, -7.0],
        }
    }
}

impl Design {
    /// Stack holes seen from the board side of the plate.
    pub fn plate_holes(&self) -> [[f64; 2]; 4] {
        self.stack_holes.map(|[x, y]| [x, -y])
    }

    /// The snap pin's angle at a column, if it has one.
    pub fn snap_angle(&self, column: usize) -> Option<f64> {
        self.snap_columns
            .iter()
            .find(|(c, _)| *c == column)
            .map(|(_, a)| *a)
    }

    pub fn uno_bottom(&self) -> f64 {
        self.plate_t + self.uno_post_h
    }

    pub fn uno_top(&self) -> f64 {
        self.uno_bottom() + self.pcb_uno
    }

    pub fn nano_x0(&self) -> f64 {
        self.nano_usb_x - self.nano[0]
    }

    pub fn nano_bottom(&self) -> f64 {
        self.plate_t + self.nano_pad_h
    }

    pub fn nano_top(&self) -> f64 {
        self.nano_bottom() + self.pcb_nano
    }

    pub fn uno_rect(&self) -> Rect {
        [
            self.uno_origin[0],
            self.uno_origin[1],
            self.uno[0],
            self.uno[1],
        ]
    }

    pub fn nano_rect(&self) -> Rect {
        [self.nano_x0(), self.nano_y0, self.nano[0], self.nano[1]]
    }

    pub fn friction_angle(&self) -> f64 {
        self.friction.atan().to_degrees()
    }

    /// The payload's weight at the design load, N.
    pub fn payload_load(&self) -> f64 {
        self.payload_mass * 9.81 * self.design_g
    }
}
