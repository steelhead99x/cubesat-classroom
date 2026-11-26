//! The camera side for the classroom kit: AMSAT's plain side with its middle window filled in,
//! a chamfered lens port, and a cradle for a Seeed XIAO ESP32-S3 Sense with its OV5640 camera.
//! Press the camera straight down into the cradle, lens first, until a snap hook clicks over
//! each long edge of the XIAO: the battery's push-down hook, because the stack is 15 mm tall.
//! No screws.
//!
//! Side coordinates: AMSAT's side as it prints, outer face on the bed (z = 0), its 3 mm web up to
//! z = 3, the frame's rails above. Assumed, not measured (the camera coupon checks them): the
//! lens sits in the middle of the XIAO's footprint, the camera module is about 9 x 9 mm and 4 mm
//! deep with an 8 mm lens barrel, and nothing on the XIAO's top is within 2 mm of its long edges.

use crate::battery;
use crate::checks::{Check, require};
use crate::design::{Design, Edge, Rect, Side};
use crate::geom::{EPS, bx, cone, cyl, overlap, union_all};
use crate::joints::{hook_cut, hook_post};
use crate::parts::upstream;
use manifold_csg::{CrossSection, Manifold};

/// The side's web, and its middle window (which this part fills in).
pub const WEB_T: f64 = 3.0;
const MIDDLE: [f64; 4] = [-38.1, -6.72, 44.9, 18.28];

#[derive(Clone, Debug)]
pub struct Camera {
    /// The XIAO ESP32-S3 Sense with its camera: footprint and height on the web.
    pub xiao: [f64; 2],
    pub stack: f64,
    /// Where the lens is: the middle of the footprint.
    pub at: [f64; 2],
    pub lens: f64,
    pub port: f64,
    /// Field of view the port's chamfer leaves clear, degrees (diagonal).
    pub fov: f64,
    pub guide_h: f64,
    pub guide_gap: f64,
    /// A XIAO with its camera and antenna, for the load check.
    pub mass: f64,
    /// The side's low edge (y = -51.22) sits at the cube's bottom face.
    pub side_base: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            xiao: [21.0, 17.8],
            stack: 15.0,
            at: [3.4, 5.78],
            lens: 8.0,
            port: 10.0,
            fov: 75.0,
            guide_h: 6.0,
            guide_gap: 0.5,
            mass: 0.012,
            side_base: 51.22,
        }
    }
}

impl Camera {
    pub fn rect(&self) -> Rect {
        let [w, h] = self.xiao;
        [self.at[0] - w / 2.0, self.at[1] - h / 2.0, w, h]
    }

    pub fn top(&self) -> f64 {
        WEB_T + self.stack
    }
}

/// One snap hook over each long edge, in the middle.
pub const HOOKS: [Edge; 2] = [
    Edge::new(Side::Bottom, 10.5, 1),
    Edge::new(Side::Top, 10.5, -1),
];

/// The XIAO ESP32-S3 Sense with its camera, seated lens-down on the web; its USB-C end faces -x.
pub fn xiao_sense(c: &Camera) -> Manifold {
    let [x0, y0, w, h] = c.rect();
    let [cx, cy] = c.at;
    let top = c.top();
    union_all(&[
        cyl(c.lens, 1.0 + EPS, cx, cy, WEB_T - 1.0),
        bx(
            cx - 4.5,
            cy - 4.5,
            WEB_T,
            cx + 4.5,
            cy + 4.5,
            WEB_T + 4.0 + EPS,
        ),
        bx(x0, y0, WEB_T + 4.0, x0 + w, y0 + h, top),
        bx(
            x0 + 2.0,
            y0 + 2.0,
            top - EPS,
            x0 + w - 2.0,
            y0 + h - 2.0,
            top + 2.0,
        ),
        bx(x0 - 1.3, cy - 4.5, top - EPS, x0 + 6.1, cy + 4.5, top + 3.2),
    ])
}

/// Guides at both short ends, low enough to stay under the USB-C plug.
fn guides(c: &Camera) -> Manifold {
    let [x0, y0, w, h] = c.rect();
    let (z0, z1) = (WEB_T - EPS, WEB_T + c.guide_h);
    let g = c.guide_gap;
    union_all(&[
        bx(x0 - g - 2.0, y0 + 2.0, z0, x0 - g, y0 + h - 2.0, z1),
        bx(x0 + w + g, y0 + 2.0, z0, x0 + w + g + 2.0, y0 + h - 2.0, z1),
    ])
}

/// Guides and hooks. `deflect` bends the hooks outward.
pub fn features(d: &Design, c: &Camera, deflect: f64) -> Manifold {
    let j = battery::joint(d);
    let mut parts = vec![guides(c)];
    for e in &HOOKS {
        parts.push(hook_post(&j, c.rect(), e, c.top(), deflect));
    }
    union_all(&parts)
}

fn hook_cuts(d: &Design, c: &Camera) -> Manifold {
    let j = battery::joint(d);
    union_all(
        &HOOKS
            .iter()
            .map(|e| hook_cut(&j, c.rect(), e))
            .collect::<Vec<_>>(),
    )
}

/// The lens port, with a chamfer outside that clears the camera's field of view.
fn port(c: &Camera) -> Manifold {
    let [cx, cy] = c.at;
    let flare = c.port + 2.0 * WEB_T * (c.fov / 2.0).to_radians().tan();
    cyl(c.port, WEB_T + 2.0, cx, cy, -1.0).union(&cone(flare, c.port, WEB_T, cx, cy, -EPS))
}

/// The web where slits can be cut: the side's own web plus the filled middle window.
fn web(side: &Manifold) -> CrossSection {
    let [x0, y0, x1, y1] = MIDDLE;
    side.slice_to_cross_section(WEB_T / 2.0)
        .union(&crate::geom::rect(x0 - 0.5, y0 - 0.5, x1 + 0.5, y1 + 0.5))
}

/// The camera side. `deflect` bends the hooks outward, for the checks.
pub fn side_with_camera(d: &Design, c: &Camera, deflect: f64) -> Result<Manifold, String> {
    let side = upstream("cubesatsim_frame_side.v2.0.stl")?.to_solid();
    let [x0, y0, x1, y1] = MIDDLE;
    let fill = bx(x0 - EPS, y0 - EPS, 0.0, x1 + EPS, y1 + EPS, WEB_T);
    Ok(side
        .union(&fill)
        .difference(&port(c))
        .difference(&hook_cuts(d, c))
        .union(&features(d, c, deflect)))
}

/// The camera coupon: the filled window with its port and cradle, and a rim.
pub fn coupon(d: &Design, c: &Camera) -> Result<Manifold, String> {
    let cx = c.at[0];
    let (x0, x1) = (cx - 24.0, cx + 24.0);
    let (y0, y1) = (MIDDLE[1] - 7.0, MIDDLE[3] + 7.0);
    let piece = side_with_camera(d, c, 0.0)?.intersection(&bx(x0, y0, -1.0, x1, y1, WEB_T + 30.0));
    let rim = crate::geom::rect(x0, y0, x1, y1).difference(&crate::geom::rect(
        x0 + 3.0,
        y0 + 3.0,
        x1 - 3.0,
        y1 - 3.0,
    ));
    Ok(piece.union(&rim.extrude(WEB_T)))
}

/// Heights in the cube, from its bottom face: the side's low edge is the bottom; the payload
/// plate's underside is 15 mm up and the battery plate's 15 mm down from the top.
pub fn camera_band(c: &Camera) -> (f64, f64) {
    let base = c.side_base;
    let [_, y0, _, h] = c.rect();
    (y0 - 1.3 + base, y0 + h + 1.3 + base)
}

pub fn verify(d: &Design, c: &Camera) -> Check {
    let j = battery::joint(d);
    let side = upstream("cubesatsim_frame_side.v2.0.stl")?.to_solid();
    require(
        c.port >= c.lens + 1.0,
        "the lens port is too tight for the lens",
    )?;
    // The slits must stay in solid web, 1.5 mm in from any window or edge.
    let margin = web(&side).offset(-1.5, manifold_csg::JoinType::Miter, 2.0, 0);
    require(
        hook_cuts(d, c)
            .slice_to_cross_section(WEB_T / 2.0)
            .difference(&margin)
            .area()
            < 1e-3,
        "a camera hook's slits run into the side's windows or edges",
    )?;
    let stack = xiao_sense(c);
    let seated = side_with_camera(d, c, j.hook_preload + 0.01)?;
    require(
        overlap(&seated, &stack) < 1e-3,
        "the camera doesn't fit its cradle",
    )?;
    for e in &HOOKS {
        require(
            overlap(
                &stack.translate(0.0, 0.0, 0.3),
                &hook_post(&j, c.rect(), e, c.top(), 0.0),
            ) > 1e-3,
            "a snap hook doesn't reach over the camera's XIAO",
        )?;
    }
    // Play along the cradle: no more than 0.6 mm.
    for dx in [0.6, -0.6] {
        require(
            overlap(&seated, &stack.translate(dx, 0.0, 0.0)) > 1e-3,
            "the camera can slide along its cradle",
        )?;
    }
    for dy in [j.hook_gap + 0.15, -j.hook_gap - 0.15] {
        require(
            overlap(&seated, &stack.translate(0.0, dy, 0.0)) > 1e-3,
            "the camera can slide sideways in its cradle",
        )?;
    }
    let spread = features(d, c, j.hook_reach + 0.05);
    for dz in [0.5, 2.0, 5.0, 10.0] {
        require(
            overlap(&spread, &stack.translate(0.0, 0.0, dz)) < 1e-3,
            "the camera can't be pressed straight down past its hooks",
        )?;
    }
    // Inside the cube: between the payload's tallest part and the battery's lowest point.
    let (lo, hi) = camera_band(c);
    let payload_top = 15.0 + d.uno_top() + 11.0;
    let battery_low = 113.5 - 15.0 - battery::cell_top(d);
    require(
        lo >= payload_top + 2.0 && hi <= battery_low - 2.0,
        format!(
            "the camera ({lo:.0}-{hi:.0} mm up the cube) hits the payload (to {payload_top:.0} mm) or the \
             battery (from {battery_low:.0} mm)"
        ),
    )?;
    let f = c.mass * 9.81 * d.launch_g * d.launch_fs / 2.0;
    let stress = f / (j.hook_w * j.hook_post_w)
        + 6.0 * f * j.hook_reach / (j.hook_post_w * j.hook_w * j.hook_w);
    require(
        stress <= d.interlayer_mpa,
        format!("a camera hook would see {stress:.1} MPa across its layers at launch"),
    )
}

/// Deliberately broken camera designs.
pub fn broken_designs() -> Vec<(&'static str, Camera)> {
    let with = |f: &dyn Fn(&mut Camera)| {
        let mut c = Camera::default();
        f(&mut c);
        c
    };
    vec![
        ("lens port too tight for the lens", with(&|c| c.port = 8.5)),
        (
            "camera moved so its hook slits cut into a window",
            with(&|c| c.at[1] = 14.0),
        ),
        (
            "camera guides too loose to hold it",
            with(&|c| c.guide_gap = 1.5),
        ),
        (
            "camera side fitted 30 mm lower: the camera hits the payload",
            with(&|c| c.side_base = 21.0),
        ),
        (
            "a 1 kg camera on hooks made for a 12 g XIAO",
            with(&|c| c.mass = 1.0),
        ),
    ]
}
