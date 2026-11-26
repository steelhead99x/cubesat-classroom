//! The two Arduino boards as solids, for fit checks, in plate coordinates.
//!
//! Geometry from Arduino's own CAD (CC BY-SA 4.0, github.com/arduino/docs-content); see
//! hardware/amsat-cubesatsim/reference/README.md.

use manifold_csg::Manifold;

use crate::design::Design;
use crate::geom::{EPS, bx, cyl};

/// Uno R3 top-side parts (x0, y0, x1, y1, height above the board) from the Eagle layout, from the
/// board's lower-left corner. Heights are the parts' catalogue heights, rounded up.
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

/// Nano 33 BLE Sense Rev2 top-side parts (along from the USB end, across, along, across, height),
/// measured from the CAD's 3D models. The Micro-USB overhangs the USB end by 1.48 mm.
pub const NANO_PARTS: [(&str, [f64; 5]); 4] = [
    ("Micro-USB", [-1.48, 4.50, 4.13, 13.30, 2.8]),
    ("NINA-B306 module", [28.05, 3.90, 43.05, 13.90, 2.32]),
    ("reset button", [9.65, 6.65, 12.55, 11.15, 2.05]),
    ("sensors and passives", [0.0, 2.80, 43.18, 14.90, 1.4]),
];
/// Micro-USB receptacle centre across the board, and above its top face.
pub const NANO_USB_CENTRE: f64 = 8.90;
pub const NANO_USB_ABOVE: f64 = 1.4;

pub fn uno_board(d: &Design) -> Manifold {
    let [x0, y0, w, h] = d.uno_rect();
    let mut board = bx(x0, y0, d.uno_bottom(), x0 + w, y0 + h, d.uno_top());
    for [hx, hy] in d.uno_holes {
        board = board.difference(&cyl(
            3.2,
            d.pcb_uno + 2.0,
            x0 + hx,
            y0 + hy,
            d.uno_bottom() - 1.0,
        ));
    }
    board
}

/// Where the Uno's through-hole solder joints are: 2 mm under the board, less its posts.
pub fn uno_underside(d: &Design) -> Manifold {
    let [x0, y0, w, h] = d.uno_rect();
    let mut zone = bx(x0, y0, d.uno_bottom() - 2.0, x0 + w, y0 + h, d.uno_bottom());
    for [hx, hy] in d.uno_holes {
        zone = zone.difference(&cyl(7.0, 4.0, x0 + hx, y0 + hy, d.uno_bottom() - 3.0));
    }
    zone
}

pub fn uno_parts(d: &Design) -> Vec<(&'static str, Manifold)> {
    let [x0, y0, _, _] = d.uno_rect();
    let top = d.uno_top();
    UNO_PARTS
        .iter()
        .map(|(name, [a, b, c, e, h])| {
            (
                *name,
                bx(x0 + a, y0 + b, top - EPS, x0 + c, y0 + e, top + h),
            )
        })
        .collect()
}

/// A box in the Nano's own axes: along runs from the USB end towards -x, across towards +y.
fn nano_box(d: &Design, along: [f64; 2], across: [f64; 2], z: [f64; 2]) -> Manifold {
    let x_usb = d.nano_usb_x;
    bx(
        x_usb - along[1],
        d.nano_y0 + across[0],
        z[0],
        x_usb - along[0],
        d.nano_y0 + across[1],
        z[1],
    )
}

pub fn nano_board(d: &Design) -> Manifold {
    nano_box(
        d,
        [0.0, d.nano[0]],
        [0.0, d.nano[1]],
        [d.nano_bottom(), d.nano_top()],
    )
}

pub fn nano_parts(d: &Design) -> Vec<(&'static str, Manifold)> {
    let top = d.nano_top();
    NANO_PARTS
        .iter()
        .map(|(name, [a, b, c, e, h])| {
            (*name, nano_box(d, [*a, *c], [*b, *e], [top - EPS, top + h]))
        })
        .collect()
}

/// Envelope of a Micro-USB plug in the Nano's socket, pointing +x.
pub fn nano_plug(d: &Design) -> Manifold {
    let [length, thick, wide] = d.plug;
    let x = d.nano_usb_x - NANO_PARTS[0].1[0];
    let y = d.nano_y0 + NANO_USB_CENTRE;
    let z = d.nano_top() + NANO_USB_ABOVE;
    bx(
        x,
        y - wide / 2.0,
        z - thick / 2.0,
        x + length,
        y + wide / 2.0,
        z + thick / 2.0,
    )
}
