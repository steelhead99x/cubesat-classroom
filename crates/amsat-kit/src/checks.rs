//! Fit, clearance and strength checks for the kit, and the broken designs the tests feed them.
//! These are CAD design checks and hand calculations for PLA, not test results.

use manifold_csg::Manifold;

use crate::battery;
use crate::boards::{
    NANO_PARTS, nano_board, nano_parts, nano_plug, uno_board, uno_parts, uno_underside,
};
use crate::design::{Design, Edge, Rect, Side};
use crate::geom::{bounds, bx, cyl, overlap, rotate_about};
use crate::joints::{
    contact_r, finger_outline, finger_pass, finger_strain, finger_strain_at, finger_zone,
    hook_post, hook_strain, hook_zone, lip, pin_retention, pin_top,
};
use crate::parts::{
    COUPON_PIN_AT, FIT_LADDER, board_features, bottom_with_columns, branding, coupon_nano_base,
    coupon_pin_base, coupon_pin_tab, ladder_tooth_r, payload_plate, plate_outline, to_frame,
    to_plate,
};

/// mm of floating-point noise to ignore; far below print resolution.
const TOLERANCE: f64 = 1e-6;

pub(crate) type Check = Result<(), String>;

pub(crate) fn require(ok: bool, message: impl Into<String>) -> Check {
    if ok { Ok(()) } else { Err(message.into()) }
}

/// `value >= minimum`, ignoring floating-point noise.
fn at_least(value: f64, minimum: f64) -> bool {
    value >= minimum - TOLERANCE
}

/// Snap-fit design rules and hand calculations, before any geometry.
pub fn rules(d: &Design) -> Check {
    let mut strains = vec![
        ("snap pin fingers".to_string(), finger_strain(d)),
        ("board hook fingers".to_string(), hook_strain(d)),
    ];
    for (fit, _) in FIT_LADDER {
        strains.push((
            format!("{fit} mm coupon tab's fingers"),
            finger_strain_at(d, ladder_tooth_r(d, fit)),
        ));
    }
    for (label, strain) in strains {
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
        d.pin_flat <= d.pin_r - 0.4,
        "the snap pin's flats barely cut it: nothing to key the plate on",
    )?;
    require(
        at_least(2.0 * d.pin_flat, 4.0),
        format!(
            "the snap pin is only {:.1} mm across its flats, under 4 mm",
            2.0 * d.pin_flat
        ),
    )?;
    require(
        d.tooth_half + 0.3 <= d.pin_flat,
        "the snap pin teeth run past the head's round sides onto its flats",
    )?;
    require(
        (0.05..=0.4).contains(&d.pin_fit),
        format!(
            "the snap pin fit is {} mm: under 0.05 mm the plate binds, over 0.4 mm it wobbles",
            d.pin_fit
        ),
    )?;
    require(
        d.finger_slit >= finger_pass(d) + 0.3,
        "too little room behind the snap pin fingers for them to bend",
    )?;
    let tight: Vec<[f64; 2]> = d
        .snap_columns
        .iter()
        .filter(|(c, _)| d.pin_tight.contains(c))
        .map(|(c, _)| d.plate_holes()[*c])
        .collect();
    require(
        tight.len() == d.pin_tight.len() && 2 * tight.len() >= d.snap_columns.len(),
        "at least half the snap pins must be tight, and every tight pin a snap pin",
    )?;
    let opposite = tight.iter().any(|a| {
        tight
            .iter()
            .any(|b| (a[0] - b[0]).abs() > d.plate / 2.0 && (a[1] - b[1]).abs() > d.plate / 2.0)
    });
    require(
        opposite,
        "two tight snap pins must sit on opposite corners, so the plate can't turn",
    )?;
    require(
        tight.len() < d.snap_columns.len(),
        "every snap pin is tight: an error in hole spacing could make the plate bind",
    )?;
    require(
        (0.05..=0.3).contains(&d.pin_spacing_allow),
        "the looser snap pins need 0.05 to 0.3 mm more clearance than the tight ones",
    )?;
    require(
        d.finger_slit_in >= 0.6,
        "the slit inside each snap pin finger is too thin to print open",
    )?;
    require(
        at_least(
            2.0 * (d.pin_head_r + d.finger_clear - d.finger_slit_in),
            d.min_wall,
        ),
        "the rigid tongue between the snap pin fingers is too thin",
    )?;
    require(
        d.pin_tip_r <= d.tooth_r - 0.3,
        "the snap pin's tip is too wide to start between the teeth",
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

/// A plate on its columns (plate coordinates). `make` builds the plate with its teeth moved.
pub(crate) fn plate_on_columns(
    d: &Design,
    columns: &Manifold,
    make: &dyn Fn(Option<f64>) -> Result<Manifold, String>,
    amsat_frame: bool,
) -> Check {
    let plate = &make(None)?;
    let seated = make(Some(contact_r(d) + 0.02))?;
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
    let spread = make(Some(d.pin_head_r + 0.1))?;
    for dz in [0.5, 1.5, 2.5, 3.5, 4.5] {
        require(
            overlap(&spread.translate(0.0, 0.0, dz), columns) < 1e-3,
            "the plate can't be pushed down past the pin heads with its fingers spread",
        )?;
    }
    let step = d.pin_fit + 0.1;
    for (dx, dy) in [(step, 0.0), (-step, 0.0), (0.0, step), (0.0, -step)] {
        require(
            overlap(&seated.translate(dx, dy, 0.0), columns) > 1e-3,
            "the plate can slide on its columns: the flats don't locate it",
        )?;
    }
    require(
        !amsat_frame || bounds(&to_frame(d, plate)).1[2] <= -d.snap_tabs_reach,
        "the plate doesn't clear the AMSAT snap tabs",
    )
}

pub(crate) struct Board<'a> {
    pub name: &'a str,
    pub solid: Manifold,
    pub rect: Rect,
    pub top: f64,
    pub lips: &'a [Edge],
    pub hooks: &'a [Edge],
    /// Axis and point it swings about as it goes in, and which way.
    pub axis: char,
    pub pivot: [f64; 3],
    /// Everything it swings past, with the hooks bent out.
    pub spread: Manifold,
    /// Which way it tilts up as it goes in: -1 when its lips are on its high-y or low-x edge.
    pub sign: f64,
}

/// One board: seated clear of everything, held when pulled, and able to go in by tucking its lip
/// edge under its lips and swinging down past its hooks.
pub(crate) fn board_held(d: &Design, b: &Board, seated: &Manifold) -> Check {
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
    for angle in [12.0, 8.0, 4.0, 2.0] {
        let tilted = rotate_about(&b.solid, b.axis, b.sign * angle, b.pivot);
        require(
            overlap(&b.spread, &tilted) < 1e-3,
            format!("the {name} can't swing down into place (blocked at {angle} degrees)"),
        )?;
    }
    Ok(())
}

/// The cell on the battery plate: held by the board joint, clear of the columns and the snap
/// fingers, strong enough at the launch load.
fn battery_held(d: &Design, columns: &Manifold) -> Check {
    let j = battery::joint(d);
    let cell = battery::cell(d);
    require(
        overlap(&cell, columns) < 1e-3,
        "the battery hits a column or a pin head",
    )?;
    // Every cell the listing allows: shortest and longest, thinnest and thickest, narrowest and
    // widest, against either guide.
    let (short, long) = battery::cell_lengths(d);
    let t = d.cell_tol;
    let (thin, thick) = (d.cell[2] - t, d.cell[2] + t);
    let (narrow, wide) = (d.cell[1] - t, d.cell[1] + t);
    let rest = battery::features(d, 0.0);
    for len in [short, long] {
        let seated = battery::features(d, battery::seated_deflect(d, len) + 0.01);
        for th in [thin, thick] {
            for (w, dx) in [(wide, 0.0), (narrow, 0.0), (narrow, wide - narrow)] {
                let c = battery::cell_sized(d, len, w, th, dx);
                let size = format!("{len} x {w} x {th} mm cell");
                require(
                    overlap(&seated, &c) < 1e-3,
                    format!("the {size} collides with its guides or hooks"),
                )?;
                require(
                    overlap(&rest, &c) > 1e-3,
                    format!("the {size} sits loose: the hooks don't press on its ends"),
                )?;
                let lifted = c.translate(0.0, 0.0, thick - th + 0.3);
                for e in &battery::CELL_HOOKS {
                    let one = hook_post(
                        &j,
                        battery::hook_rect(d),
                        e,
                        battery::cell_top(d),
                        battery::seated_deflect(d, len) + 0.01,
                    );
                    require(
                        overlap(&lifted, &one) > 1e-3,
                        format!("a snap hook doesn't reach over the {size}: it could lift out"),
                    )?;
                }
            }
        }
    }
    let seated = battery::features(d, battery::seated_deflect(d, long) + 0.01);
    for dx in [battery::GUIDE_GAP + 0.1, -battery::GUIDE_GAP - 0.1] {
        require(
            overlap(&seated, &cell.translate(dx, 0.0, 0.0)) > 1e-3,
            "the battery can slide sideways: the guides don't hold it",
        )?;
    }
    require(
        battery::hook_strain(d) <= d.strain_limit + 1e-9,
        format!(
            "the battery hook fingers bend {:.2}% as the longest cell goes in, over {:.1}%",
            battery::hook_strain(d) * 100.0,
            d.strain_limit * 100.0
        ),
    )?;
    // Going in: pressed straight down, the longest, thickest cell spreads both hooks.
    let spread = battery::features(d, battery::seated_deflect(d, long) + j.hook_reach + 0.05);
    for dz in [0.5, 2.0, 4.0, 8.0] {
        require(
            overlap(&spread, &cell.translate(0.0, 0.0, dz)) < 1e-3,
            "the battery can't be pressed straight down past its hooks",
        )?;
    }
    let top = d.plate_t;
    for zone in battery::hook_zones(d, -1.0, top + 2.0) {
        for (i, [x, y]) in d.plate_holes().into_iter().enumerate() {
            if let Some(angle) = d.snap_angle(i) {
                require(
                    overlap(
                        &zone,
                        &finger_zone(d, x, y, angle, d.column_fit(i), -1.0, top + 2.0),
                    ) < 1e-3,
                    "a battery hook's finger runs into a snap pin's fingers",
                )?;
            }
        }
    }
    // Launch: the cell's weight at the design load, shared by the two hooks, pulls each post
    // along its layers' weak direction and bends it by the hook's reach.
    let f = d.cell_mass * 9.81 * d.launch_g * d.launch_fs / 2.0;
    let worst = f / (j.hook_w * j.hook_post_w)
        + 6.0 * f * j.hook_reach / (j.hook_post_w * j.hook_w * j.hook_w);
    require(
        worst <= d.interlayer_mpa,
        format!(
            "a battery post would see {worst:.1} MPa across its layers at {} g (allowed {} MPa)",
            d.launch_g, d.interlayer_mpa
        ),
    )?;
    let plate_mass = 0.02;
    let need = (d.cell_mass + plate_mass) * 9.81 * d.launch_g * 1.5;
    let hold = 3.0 * pin_retention(d, d.plate_t);
    require(
        hold >= need,
        format!(
            "the battery plate's snap pins hold {hold:.0} N, under 1.5 times its {:.0} N launch load",
            need / 1.5
        ),
    )
}

pub fn verify(d: &Design) -> Check {
    rules(d)?;
    let columns = to_plate(d, &bottom_with_columns(d)?);
    plate_on_columns(d, &columns, &|t| payload_plate(d, t, 0.0), true)?;
    // The battery plate snaps onto a second columns piece on the top: the same part.
    plate_on_columns(d, &columns, &|t| battery::battery_plate(d, t, 0.0), true)?;
    battery_held(d, &columns)?;

    let top = d.plate_t;
    let below = bx(-d.plate, -d.plate, -1.0, d.plate, d.plate, top);
    let features = board_features(d, 0.0).difference(&below);
    let seated = board_features(d, d.hook_preload + 0.01).difference(&below); // hooks bent by a seated board
    let inside = plate_outline(d).extrude(80.0).translate(0.0, 0.0, top);
    require(
        overlap(&features, &inside) >= features.volume() - 1e-2,
        "a part reaches past the plate outline, into the frame's corner rails",
    )?;
    let relief = branding(d)?.extrude(1.0).translate(0.0, 0.0, top);
    let wall = plate_outline(d).offset(-d.min_wall, manifold_csg::JoinType::Miter, 2.0, 0);
    for (i, [x, y]) in d.plate_holes().into_iter().enumerate() {
        if let Some(angle) = d.snap_angle(i) {
            let fit = d.column_fit(i);
            let zone = finger_zone(d, x, y, angle, fit, top, 3.0);
            require(
                overlap(&features, &zone) < 1e-3,
                "a part stands on a snap pin's flex fingers",
            )?;
            require(
                overlap(&relief, &zone) < 1e-3,
                "the branding runs onto a snap pin's flex fingers",
            )?;
            require(
                finger_outline(d, x, y, angle, fit).difference(&wall).area() < 1e-3,
                format!(
                    "a snap pin's fingers come within {} mm of the plate edge",
                    d.min_wall
                ),
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
                if let Some(angle) = d.snap_angle(i) {
                    require(
                        overlap(
                            &hook_zone(d, rect, e, -1.0, top + 2.0),
                            &finger_zone(d, x, y, angle, d.column_fit(i), -1.0, top + 2.0),
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
        spread: board_features(d, d.hook_reach + 0.05),
        sign: -1.0,
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
        spread: board_features(d, d.hook_reach + 0.05),
        sign: -1.0,
    };
    board_held(d, &board, &seated)?;
    let plug = nano_plug(d);
    let on_coupon = Board {
        name: "Nano on its coupon",
        spread: coupon_nano_base(d, d.hook_reach + 0.05),
        ..board
    };
    let coupon_seated = coupon_nano_base(d, d.hook_preload + 0.01);
    board_held(d, &on_coupon, &coupon_seated)?;
    for (name, body) in &parts {
        require(
            overlap(&coupon_seated, body) < 1e-3,
            format!("the Nano coupon hits the Nano's {name}"),
        )?;
    }
    require(
        overlap(&coupon_seated, &plug) < 1e-3,
        "something on the Nano coupon blocks its USB plug",
    )?;
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

    // Pin coupon: every tab of the fit ladder goes on, presses, and is located by its flats.
    let base = coupon_pin_base(d);
    let at = |m: Manifold| m.translate(0.0, 0.0, COUPON_PIN_AT);
    for (fit, dots) in FIT_LADDER {
        let seated = at(coupon_pin_tab(d, fit, dots, Some(contact_r(d) + 0.02)));
        require(
            overlap(&base, &seated) < 1e-3,
            format!("the {fit} mm fit tab doesn't go onto its pin"),
        )?;
        require(
            overlap(&base, &at(coupon_pin_tab(d, fit, dots, None))) > 1e-3,
            format!("the {fit} mm fit tab's teeth don't press on the head"),
        )?;
        // The walls alone, with the teeth spread out of the way.
        let walls = at(coupon_pin_tab(d, fit, dots, Some(d.pin_head_r + 0.3)));
        for dy in [fit + 0.05, -fit - 0.05] {
            require(
                overlap(&base, &walls.translate(0.0, dy, 0.0)) > 1e-3,
                format!("the {fit} mm fit tab can slide across the pin's flats"),
            )?;
        }
        for dy in [fit * 0.9, -fit * 0.9] {
            require(
                overlap(&base, &walls.translate(0.0, dy, 0.0)) < 1e-3,
                format!("the {fit} mm fit tab has less clearance on the flats than it should"),
            )?;
        }
    }
    require(
        FIT_LADDER
            .iter()
            .any(|(fit, _)| (fit - d.pin_fit).abs() < 1e-9),
        "the fit ladder doesn't include the kit's own fit",
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
            with(&|d| d.finger_len = 10.0),
        ),
        (
            "snap pin flats cut too deep: the pin would be weak",
            with(&|d| d.pin_flat = 1.8),
        ),
        (
            "snap pin teeth wider than the head's round sides",
            with(&|d| d.tooth_half = 2.3),
        ),
        (
            "a cell size range wider than the battery hooks can take",
            with(&|d| d.cell_tol = 3.0),
        ),
        (
            "no clearance on the flats: the plate would bind",
            with(&|d| d.pin_fit = 0.0),
        ),
        (
            "the looser snap pin as tight as the others: the plate could bind",
            with(&|d| d.pin_spacing_allow = 0.0),
        ),
        (
            "every snap pin tight: the plate could bind",
            with(&|d| d.pin_tight = vec![0, 1, 2]),
        ),
        (
            "tight pins side by side instead of on opposite corners",
            with(&|d| d.pin_tight = vec![0, 1]),
        ),
        (
            "only one tight pin: the plate could turn",
            with(&|d| d.pin_tight = vec![2]),
        ),
        (
            "snap pin turned so its fingers run off the plate",
            with(&|d| d.snap_columns[0].1 += 180.0),
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
            "Uno lip moved into the DC jack",
            with(&|d| d.uno_lips[0] = Edge::new(Side::Left, 8.0, 1)),
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
