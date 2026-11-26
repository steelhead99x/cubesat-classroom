//! The two boards as solids, for fit checks. Geometry comes from Arduino's own CAD
//! (CC BY-SA 4.0, github.com/arduino/docs-content), the same data as the AMSAT kit:
//! see hardware/amsat-cubesatsim/reference/README.md. Plate coordinates.

use manifold_csg::Manifold;

use crate::design::Design;
use crate::geom::{EPS, cuboid, cylinder, union_all};

/// Uno R3 top-side parts (x0, y0, x1, y1, height above the board) from the Eagle
/// layout, from the board's lower-left corner. Heights are catalogue heights, rounded up.
pub const UNO_PARTS: [(&str, [f64; 5]); 16] = [
    ("DC jack", [-1.87, 1.30, 13.68, 12.35, 11.0]),
    ("USB-B", [-6.47, 30.49, 9.53, 45.71, 11.0]),
    ("digital header 8-13", [17.45, 49.45, 43.00, 52.14, 8.5]),
    ("digital header 0-7", [44.37, 49.45, 64.84, 52.14, 8.5]),
    ("power header", [26.59, 1.19, 47.06, 3.88, 8.5]),
    ("analog header", [49.45, 1.19, 64.84, 3.88, 8.5]),
    ("ICSP (328P)", [62.28, 24.05, 67.51, 31.82, 8.5]),
    ("ICSP (16U2)", [14.40, 43.61, 22.17, 48.84, 8.5]),
    ("JP2", [16.94, 38.53, 22.17, 43.76, 8.5]),
    ("ATmega328P in socket", [27.73, 11.50, 64.97, 21.26, 8.0]),
    ("capacitor PC1", [22.22, 5.24, 28.82, 13.04, 7.7]),
    ("capacitor PC2", [15.11, 5.24, 21.71, 13.04, 7.7]),
    ("reset button", [1.35, 46.45, 11.35, 52.35, 5.0]),
    ("crystal Y1", [13.13, 23.80, 24.71, 28.52, 4.5]),
    ("diode D1", [18.19, 0.84, 24.99, 4.75, 2.5]),
    ("regulator U1", [3.54, 14.02, 11.96, 20.77, 1.8]),
];

/// Nano 33 BLE Sense Rev2 top-side parts (along, across, along, across, height) from
/// the CAD's 3D models. The Micro-USB overhangs the USB end by 1.48 mm.
pub const NANO_PARTS: [(&str, [f64; 5]); 4] = [
    ("Micro-USB", [-1.48, 4.50, 4.13, 13.30, 2.8]),
    ("NINA-B306 module", [28.05, 3.90, 43.05, 13.90, 2.32]),
    ("reset button", [9.65, 6.65, 12.55, 11.15, 2.05]),
    ("sensors and passives", [0.0, 2.80, 43.18, 14.90, 1.4]),
];
/// Micro-USB receptacle centre across the board, and above its top face.
pub const NANO_USB_CENTRE: f64 = 8.90;
pub const NANO_USB_ABOVE: f64 = 1.4;

/// The Uno's PCB, with its four holes.
pub fn uno_pcb(d: &Design) -> Manifold {
    let [ux, uy] = d.uno_origin();
    let z = d.uno_pcb_bottom();
    let mut pcb = cuboid([ux, uy, z], [ux + d.uno[0], uy + d.uno[1], d.uno_top()]);
    for i in 0..4 {
        let [x, y] = d.uno_hole(i);
        pcb = pcb.difference(&cylinder(3.2, d.pcb_uno + 2.0, [x, y, z - 1.0], None));
    }
    pcb
}

/// The zone under the Uno where its solder joints are, outside its posts.
pub fn uno_underside(d: &Design) -> Manifold {
    let [ux, uy] = d.uno_origin();
    let z = d.uno_pcb_bottom();
    let posts: Vec<Manifold> = (0..4)
        .map(|i| {
            let [x, y] = d.uno_hole(i);
            cylinder(
                7.0,
                d.uno_post_h + d.pcb_uno + 2.0,
                [x, y, d.plate_t - 1.0],
                None,
            )
        })
        .collect();
    cuboid([ux, uy, z - 2.0], [ux + d.uno[0], uy + d.uno[1], z]).difference(&union_all(&posts))
}

/// Envelopes of the Uno's top-side parts, by name.
pub fn uno_parts(d: &Design) -> Vec<(&'static str, Manifold)> {
    let [ux, uy] = d.uno_origin();
    let top = d.uno_top();
    UNO_PARTS
        .iter()
        .map(|(name, [x0, y0, x1, y1, h])| {
            (
                *name,
                cuboid([ux + x0, uy + y0, top - EPS], [ux + x1, uy + y1, top + h]),
            )
        })
        .collect()
}

/// A box in the Nano's own axes turned into its place: standing on its long edge,
/// parts facing +Y, USB end towards -X. Turning it that way puts the CAD's
/// across-the-board axis upside down, so z = z0 + w - across.
fn nano_box(d: &Design, along: [f64; 2], across: [f64; 2], out: [f64; 2]) -> Manifold {
    let n = &d.nano;
    cuboid(
        [n.x0 + along[0], n.y0 + out[0], n.z0 + n.w - across[1]],
        [n.x0 + along[1], n.y0 + out[1], n.z0 + n.w - across[0]],
    )
}

/// The Nano in place: PCB plus envelopes of its top-side parts.
pub fn nano_card(d: &Design) -> Manifold {
    let n = &d.nano;
    let mut body = nano_box(d, [0.0, n.l], [0.0, n.w], [0.0, n.t]);
    for (_, [x0, y0, x1, y1, h]) in NANO_PARTS {
        body = body.union(&nano_box(d, [x0, x1], [y0, y1], [n.t - EPS, n.t + h]));
    }
    body
}

/// The Nano's PCB alone, shrunk 0.02 mm so touching its grooves doesn't count.
pub fn nano_pcb(d: &Design) -> Manifold {
    let n = &d.nano;
    cuboid(
        [n.x0, n.y0 + 0.02, n.z0 + 0.02],
        [n.x0 + n.l, n.y0 + n.t - 0.02, n.z0 + n.w - 0.02],
    )
}

/// The parts on the Nano's top face, without the board.
pub fn nano_top_parts(d: &Design) -> Manifold {
    let n = &d.nano;
    nano_card(d).difference(&cuboid(
        [n.x0 - 1.0, n.y0 - 1.0, n.z0 - 1.0],
        [n.x0 + n.l + 1.0, n.y0 + n.t, n.z0 + n.w + 1.0],
    ))
}

/// Envelope of a Micro-USB plug in the Nano's socket.
pub fn nano_plug(d: &Design) -> Manifold {
    let n = &d.nano;
    let usb_x = n.x0 + NANO_PARTS[0].1[0];
    let y = n.y0 + n.t + NANO_USB_ABOVE;
    let z = n.z0 + n.w - NANO_USB_CENTRE;
    let [len, thick, wide] = n.plug;
    cuboid(
        [usb_x - len, y - thick / 2.0, z - wide / 2.0],
        [usb_x, y + thick / 2.0, z + wide / 2.0],
    )
}

/// Envelope of a USB-B cable plug in the Uno's socket: 16 mm square overmould,
/// reaching 20 mm out past the socket.
pub fn uno_plug(d: &Design) -> Manifold {
    let [x0, y0, _, y1, h] = UNO_PARTS[1].1;
    let [ox, oy] = d.uno_origin();
    let (yc, zc) = (oy + (y0 + y1) / 2.0, d.uno_top() + h / 2.0);
    cuboid(
        [ox + x0 - 20.0, yc - 8.0, zc - 8.0],
        [ox + x0, yc + 8.0, zc + 8.0],
    )
}
