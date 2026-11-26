//! Fit, clearance and strength checks for the kit, and the broken designs the tests feed them.
//! These are CAD design checks and hand calculations for PLA, not test results.

use manifold_csg::Manifold;

use crate::boards::{
    NANO_PARTS, nano_board, nano_parts, nano_plug, uno_board, uno_parts, uno_underside,
};
use crate::design::{Design, Edge, Rect, Side};
use crate::geom::{bounds, bx, cyl, overlap, rotate_about};
use crate::joints::{
    contact_r, finger_strain, finger_zone, hook_post, hook_strain, hook_zone, lip, pin_retention,
    pin_top,
};
use crate::parts::{
    COUPON_PIN_AT, board_features, bottom_with_columns, coupon_board, coupon_board_top,
    coupon_hook_base, coupon_pin_base, coupon_pin_tab, payload_plate, plate_outline, to_frame,
    to_plate,
};

/// mm of floating-point noise to ignore; far below print resolution.
const TOLERANCE: f64 = 1e-6;

type Check = Result<(), String>;

fn require(ok: bool, message: impl Into<String>) -> Check {
    if ok { Ok(()) } else { Err(message.into()) }
}

/// `value >= minimum`, ignoring floating-point noise.
fn at_least(value: f64, minimum: f64) -> bool {
    value >= minimum - TOLERANCE
}

/// Snap-fit design rules and hand calculations, before any geometry.
pub fn rules(d: &Design) -> Check {
    for (label, strain) in [
        ("snap pin fingers", finger_strain(d)),
        ("board hook fingers", hook_strain(d)),
    ] {
        require(
            at_least(d.strain_limit, strain),
            format!(
                "the {label} bend {:.2}%, over {:.1}%",
                strain * 100.0,
                d.strain_limit * 100.0
            ),
        )?;
    }
    for (label, value) in [
        ("snap pin finger", d.finger_w),
        ("hook finger", d.hook_w),
        ("lip post", d.lip_post_t),
    ] {
        require(
            at_least(value, d.min_wall),
            format!(
                "the {label} is {value:.2} mm thick, under {} mm",
                d.min_wall
            ),
        )?;
    }
    let preload = contact_r(d) - d.tooth_r;
    require(
        preload >= 0.2,
        format!(
            "the snap pin teeth meet the head {preload:.2} mm out from rest; under 0.2 mm, print error could \
             leave the plate loose"
        ),
    )?;
    require(
        d.pin_head_r - contact_r(d) >= 0.3,
        "the snap pin teeth sit too near the edge of the head to hold",
    )?;
    let need = d.payload_load() * d.retention_margin;
    let have = d.snap_columns.len() as f64 * pin_retention(d, d.plate_t);
    require(
        have >= need,
        format!(
            "the snap pins hold {have:.1} N, less than {need:.1} N (payload at {:.0} g, times {})",
            d.design_g, d.retention_margin
        ),
    )?;
    require(
        180.0 - d.finger_span - d.finger_gap >= 6.0,
        "too little root left for each snap pin finger",
    )?;
    require(
        d.hook_angle + 3.0 <= d.friction_angle(),
        format!(
            "the hook underside at {:.0} degrees isn't safely under the {:.0} degree friction angle: pulling the \
             board up could push a hook open",
            d.hook_angle,
            d.friction_angle()
        ),
    )?;
    require(
        d.hook_preload > 0.0 && d.hook_preload < d.hook_reach / 2.0,
        "hook preload must be positive and well inside its reach",
    )?;
    require(
        at_least(d.hook_reach, 0.6),
        format!(
            "the hooks reach {} mm over the board, under 0.6 mm",
            d.hook_reach
        ),
    )?;
    let room = plate_outline(d).bounds().max()[0] - (d.nano_usb_x - NANO_PARTS[0].1[0]);
    require(
        at_least(room, d.min_plug_room),
        format!("only {room:.1} mm from the Nano's USB to the plate edge"),
    )
}

fn plate_on_columns(d: &Design, columns: &Manifold, plate: &Manifold) -> Check {
    let seated = payload_plate(d, Some(contact_r(d) + 0.02), 0.0)?;
    require(
        overlap(&seated, columns) < 1e-3,
        "the plate collides with the columns or the end piece",
    )?;
    require(
        overlap(plate, columns) > 1e-3,
        "the snap pin teeth don't press on the heads: the plate could rattle",
    )?;
    require(
        overlap(&seated.translate(0.0, 0.0, 0.3), columns) > 1e-3,
        "nothing holds the plate down on its columns",
    )?;
    let spread = payload_plate(d, Some(d.pin_head_r + 0.1), 0.0)?;
    for dz in [0.5, 1.5, 2.5, 3.5, 4.5] {
        require(
            overlap(&spread.translate(0.0, 0.0, dz), columns) < 1e-3,
            "the plate can't be pushed down past the pin heads with its fingers spread",
        )?;
    }
    require(
        bounds(&to_frame(d, plate)).1[2] <= -d.snap_tabs_reach,
        "the plate doesn't clear the AMSAT snap tabs",
    )
}

struct Board<'a> {
    name: &'a str,
    solid: Manifold,
    rect: Rect,
    top: f64,
    lips: &'a [Edge],
    hooks: &'a [Edge],
    /// Axis and point it swings about as it goes in, and which way.
    axis: char,
    pivot: [f64; 3],
}

/// One board: seated clear of everything, held when pulled, and able to go in by tucking its lip
/// edge under its lips and swinging down past its hooks.
fn board_held(d: &Design, b: &Board, seated: &Manifold) -> Check {
    let name = b.name;
    require(
        overlap(seated, &b.solid) < 1e-3,
        format!("the {name} board collides with the plate's parts"),
    )?;
    let lifted = b.solid.translate(0.0, 0.0, 0.3);
    require(
        overlap(&lifted, seated) > 1e-3,
        format!("nothing holds the {name} down"),
    )?;
    for e in b.hooks {
        let post = hook_post(d, b.rect, e, b.top, 0.0);
        require(
            overlap(&lifted, &post) > 1e-3,
            format!("a snap hook doesn't reach over the {name}"),
        )?;
    }
    for e in b.lips {
        require(
            overlap(&lifted, &lip(d, b.rect, e, b.top)) > 1e-3,
            format!("a lip doesn't reach over the {name}"),
        )?;
    }
    // Going in: tilted up on its lip edge, the board swings down; the hooks give way.
    let spread = board_features(d, d.hook_reach + 0.05);
    for angle in [12.0, 8.0, 4.0, 2.0] {
        let tilted = rotate_about(&b.solid, b.axis, -angle, b.pivot);
        require(
            overlap(&spread, &tilted) < 1e-3,
            format!("the {name} can't swing down into place (blocked at {angle} degrees)"),
        )?;
    }
    Ok(())
}

pub fn verify(d: &Design) -> Check {
    rules(d)?;
    let columns = to_plate(d, &bottom_with_columns(d)?);
    let plate = payload_plate(d, None, 0.0)?;
    plate_on_columns(d, &columns, &plate)?;

    let top = d.plate_t;
    let below = bx(-d.plate, -d.plate, -1.0, d.plate, d.plate, top);
    let features = board_features(d, 0.0).difference(&below);
    let seated = board_features(d, d.hook_preload + 0.01).difference(&below); // hooks bent by a seated board
    let inside = plate_outline(d).extrude(80.0).translate(0.0, 0.0, top);
    require(
        overlap(&features, &inside) >= features.volume() - 1e-2,
        "a part reaches past the plate outline, into the frame's corner rails",
    )?;
    for (i, [x, y]) in d.plate_holes().into_iter().enumerate() {
        if d.snap_a0(i).is_some() {
            require(
                overlap(&features, &finger_zone(d, x, y, top, 3.0)) < 1e-3,
                "a part stands on a snap pin's flex fingers",
            )?;
            let head = cyl(
                2.0 * d.pin_head_r + 0.6,
                pin_top(d, 0.0, d.plate_t) - top + 0.3,
                x,
                y,
                top,
            );
            require(
                overlap(&features, &head) < 1e-3,
                "a part sits where a column's pin head comes through",
            )?;
        }
    }
    let hooked = [
        (d.uno_rect(), &d.uno_hooks, d.uno_top()),
        (d.nano_rect(), &d.nano_hooks, d.nano_top()),
    ];
    for (rect, edges, board_top) in hooked {
        for e in edges {
            let others = features.difference(&hook_post(d, rect, e, board_top, 0.0));
            require(
                overlap(&others, &hook_zone(d, rect, e, top, 2.0)) < 1e-3,
                "a part stands on a board hook's flex finger",
            )?;
            for (i, [x, y]) in d.plate_holes().into_iter().enumerate() {
                if d.snap_a0(i).is_some() {
                    require(
                        overlap(
                            &hook_zone(d, rect, e, -1.0, top + 2.0),
                            &finger_zone(d, x, y, -1.0, top + 2.0),
                        ) < 1e-3,
                        "a board hook's finger runs into a snap pin's fingers",
                    )?;
                }
            }
        }
    }

    // Uno R3: tucks under its left lip, swings down past its hooks.
    let uno = uno_board(d);
    let [x0, y0, w, h] = d.uno_rect();
    let under = uno_underside(d);
    require(
        overlap(&seated, &under) < 1e-3,
        "something under the Uno would hit its solder joints",
    )?;
    require(
        overlap(&columns, &under) < 1e-3 && overlap(&columns, &uno) < 1e-3,
        "a column's pin head comes up under the Uno's solder joints",
    )?;
    for (name, body) in uno_parts(d) {
        require(
            overlap(&seated, &body) < 1e-3,
            format!("a lip, hook or post hits the Uno's {name}"),
        )?;
    }
    let board = Board {
        name: "Uno",
        solid: uno.clone(),
        rect: d.uno_rect(),
        top: d.uno_top(),
        lips: &d.uno_lips,
        hooks: &d.uno_hooks,
        axis: 'y',
        pivot: [x0, 0.0, d.uno_top()],
    };
    board_held(d, &board, &seated)?;
    let pcb = bx(x0, y0, d.uno_bottom(), x0 + w, y0 + h, d.uno_top());
    for (dx, dy) in [(0.4, 0.0), (-0.4, 0.0), (0.0, 0.4), (0.0, -0.4)] {
        require(
            overlap(&pcb.translate(dx, dy, 0.0), &features) > 1e-3,
            "the Uno could slide off its pins",
        )?;
    }

    // Nano 33 BLE Sense: tucks under its +y lips, swings down past its -y hooks.
    let nano = nano_board(d);
    let [_, ny0, _, nh] = d.nano_rect();
    let parts = nano_parts(d);
    for (name, body) in &parts {
        require(
            overlap(&seated, body) < 1e-3,
            format!("a lip, hook or stop hits the Nano's {name}"),
        )?;
    }
    let board = Board {
        name: "Nano",
        solid: nano.clone(),
        rect: d.nano_rect(),
        top: d.nano_top(),
        lips: &d.nano_lips,
        hooks: &d.nano_hooks,
        axis: 'x',
        pivot: [0.0, ny0 + nh, d.nano_top()],
    };
    board_held(d, &board, &seated)?;
    let play = d.nano_end_play / 2.0 + 0.2;
    for (label, dx, dy) in [
        ("towards its USB end", play, 0.0),
        ("away from its USB end", -play, 0.0),
        ("sideways", 0.0, 0.4),
        ("the other way sideways", 0.0, -0.4),
    ] {
        require(
            overlap(&nano.translate(dx, dy, 0.0), &features) > 1e-3,
            format!("the Nano could slide {label}"),
        )?;
    }
    let plug = nano_plug(d);
    require(
        overlap(&features, &plug) < 1e-3,
        "something blocks the Nano's USB plug",
    )?;
    require(
        overlap(&columns, &plug) < 1e-3,
        "a column's pin is in the way of the Nano's USB plug",
    )?;
    require(
        overlap(&nano, &under) < 1e-3 && overlap(&nano, &uno) < 1e-3,
        "the boards overlap",
    )?;
    for (_, body) in &parts {
        require(
            overlap(body, &uno) < 1e-3 && overlap(body, &under) < 1e-3,
            "a Nano part reaches under the Uno",
        )?;
    }

    // Coupons: the same joints on their own fit the same way.
    let base = coupon_pin_base(d);
    let at = |m: Manifold| m.translate(0.0, 0.0, COUPON_PIN_AT);
    require(
        overlap(&base, &at(coupon_pin_tab(d, Some(contact_r(d) + 0.02)))) < 1e-3,
        "the snap-pin coupon doesn't fit together",
    )?;
    require(
        overlap(&base, &at(coupon_pin_tab(d, None))) > 1e-3,
        "the snap-pin coupon's teeth don't press",
    )?;
    let strip = coupon_board(d).translate(0.0, 0.0, coupon_board_top(d) - d.pcb_uno);
    require(
        overlap(&coupon_hook_base(d, d.hook_preload + 0.01), &strip) < 1e-3,
        "the hook coupon's board strip doesn't fit",
    )?;
    require(
        overlap(&coupon_hook_base(d, 0.0), &strip.translate(0.0, 0.0, 0.3)) > 1e-3,
        "the hook coupon doesn't hold its board strip",
    )
}

/// Deliberately broken designs: each must fail `verify`. (label, change)
pub fn broken_designs() -> Vec<(&'static str, Design)> {
    let base = Design::default();
    let with = |f: &dyn Fn(&mut Design)| {
        let mut d = base.clone();
        f(&mut d);
        d
    };
    vec![
        (
            "plate inside the snap tabs",
            with(&|d| d.column_length = 10.0),
        ),
        (
            "snap pin fingers too thick to bend",
            with(&|d| d.finger_w = 3.0),
        ),
        (
            "snap pin fingers too short, so they would crack",
            with(&|d| d.finger_span = 120.0),
        ),
        (
            "pin head too small to hold the plate",
            with(&|d| d.pin_head_r = 3.5),
        ),
        (
            "no preload on the snap pins: the plate could rattle",
            with(&|d| d.pin_bite = 0.25),
        ),
        (
            "snap pin on the column beside the Nano",
            with(&|d| d.snap_columns.push((3, 0.0))),
        ),
        (
            "hook fingers too short, so they would crack",
            with(&|d| d.hook_len = 12.0),
        ),
        (
            "hook underside too steep: pulling the board pushes it open",
            with(&|d| d.hook_angle = 20.0),
        ),
        (
            "hooks too short to hold a board",
            with(&|d| d.hook_reach = 0.4),
        ),
        (
            "Uno posts too low for its solder joints over the pin heads",
            with(&|d| d.uno_post_h = 3.5),
        ),
        (
            "Uno hook moved onto the post at hole 1",
            with(&|d| d.uno_hooks[0] = Edge::new(Side::Right, 8.0, 1)),
        ),
        (
            "Uno lip moved into the USB-B connector",
            with(&|d| d.uno_lips[0] = Edge::new(Side::Left, 38.0, 1)),
        ),
        (
            "Uno moved 3 mm towards the Nano",
            with(&|d| d.uno_origin[1] -= 3.0),
        ),
        (
            "Nano too close to the plate edge for its USB plug",
            with(&|d| d.nano_usb_x = 26.0),
        ),
        (
            "Nano pads too low for its USB plug",
            with(&|d| d.nano_pad_h = 0.5),
        ),
        (
            "lips that don't reach over the boards",
            with(&|d| d.lip_reach = 0.0),
        ),
        (
            "no end play for the Nano",
            with(&|d| d.nano_end_play = -0.4),
        ),
    ]
}

/// Designs exactly at a limit: each must pass, because the limits are inclusive.
pub fn boundary_designs() -> Vec<(&'static str, Design)> {
    let mut d = Design::default();
    d.hook_len = (1.5 * (d.hook_w + 0.2) * (d.hook_reach + 0.1) / d.strain_limit).sqrt();
    vec![("hook fingers exactly at the 1.2% strain limit", d)]
}
