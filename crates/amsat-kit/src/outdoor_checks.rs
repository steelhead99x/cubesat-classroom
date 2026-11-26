//! Checks for the outdoor cases. They build the parts and fail if the seal, the screws, the
//! tripod thread, the ports, the battery or the boards don't fit. Design checks only: no case
//! has been printed, and nothing here is a water test.

use manifold_csg::Manifold;

use crate::boards::{nano_board, nano_parts, uno_board, uno_parts};
use crate::checks::{Board, Check, board_held, plate_on_columns, require};
use crate::design::Design;
use crate::geom::{EPS, bounds, bx, overlap, union_all};
use crate::joints::{contact_r, finger_zone, hook_zone};
use crate::outdoor::{
    self, Outdoor, battery_pocket, cell, columns, gasket_squeeze, port_inside, port_outside,
};
use crate::parts::{payload_plate, plate_outline};
use crate::xiao;

/// What the case carries: a plate (built with its teeth moved) and the boards on it, in plate
/// coordinates.
pub struct Payload<'a> {
    pub name: &'a str,
    pub make: &'a dyn Fn(Option<f64>) -> Result<Manifold, String>,
    pub boards: Manifold,
}

/// The Uno and Nano on the payload plate. The Nano's USB plug is a right-angle one inside the
/// box (or it runs from VIN): 12 mm of plug past its end.
pub fn uno_nano(d: &Design) -> Manifold {
    let mut parts = vec![uno_board(d), nano_board(d)];
    parts.extend(uno_parts(d).into_iter().map(|(_, m)| m));
    parts.extend(nano_parts(d).into_iter().map(|(_, m)| m));
    let [x0, y0, w, _] = d.nano_rect();
    let usb = x0 + w;
    parts.push(bx(
        usb,
        y0 + 4.4,
        d.nano_bottom() - 1.0,
        usb + 12.0,
        y0 + 13.4,
        d.nano_top() + 4.0,
    ));
    union_all(&parts)
}

pub fn rules(o: &Outdoor) -> Check {
    require(
        o.wall >= 2.4 && o.floor >= 2.4,
        "outdoor walls and back must be at least 2.4 mm",
    )?;
    let squeeze = gasket_squeeze(o);
    require(
        (0.2..=0.3).contains(&squeeze),
        format!(
            "the gasket is squeezed {:.0}%: 20 to 30% seals without crushing it",
            squeeze * 100.0
        ),
    )?;
    let (g0, g1) = (
        o.groove_at - o.groove_w / 2.0,
        o.groove_at + o.groove_w / 2.0,
    );
    require(
        g0 >= 1.5,
        "the gasket groove is too close to the inside edge",
    )?;
    require(o.gasket < o.groove_w, "the gasket doesn't fit its groove")?;
    let (s0, s1) = (o.screw_at - o.insert_d / 2.0, o.screw_at + o.insert_d / 2.0);
    require(
        s0 >= g1 + 1.5,
        "the screws are too close to the gasket: they must sit outside the seal with plastic between",
    )?;
    require(
        s1 <= o.wall + o.flange - 1.5,
        "the screw inserts break out of the rim",
    )?;
    let slope_z = o.top - o.flange_h - (s1 - o.wall);
    require(
        o.top - o.insert_depth >= slope_z + 1.5,
        "the screw inserts break through the slope under the rim",
    )?;
    let cap = o.deck_z - o.deck_t - (o.thread_len - 0.5);
    require(
        cap >= 1.5,
        format!("only {cap:.1} mm of plastic above the tripod thread: it must stay sealed"),
    )?;
    require(
        o.thread_len - 0.5 >= 6.0,
        "the tripod thread is too short to hold (at least 6 mm, about 5 turns)",
    )?;
    require(
        o.cell_clear >= 0.3,
        "the battery's top is too close to the plate",
    )
}

pub fn verify(d: &Design, o: &Outdoor, payload: &Payload) -> Check {
    rules(o)?;
    let body = outdoor::body(d, o, o.thread_clear);
    let zp = o.plate_z(d);
    let name = payload.name;
    require(
        plate_outline(d)
            .offset(1.0, manifold_csg::JoinType::Round, 2.0, 16)
            .difference(&o.cavity_outline())
            .area()
            < 1e-3,
        "the plate doesn't fit the box with 1 mm all round",
    )?;
    plate_on_columns(d, &body.translate(0.0, 0.0, -zp), payload.make, false)?;

    let seated = (payload.make)(Some(contact_r(d) + 0.02))?.translate(0.0, 0.0, zp);
    let boards = payload.boards.translate(0.0, 0.0, zp);
    // The battery: in its pocket, held in by the plate above and the walls round it.
    let c = cell(d, o);
    require(
        overlap(&c, &body) < 1e-3,
        "the battery doesn't fit its pocket",
    )?;
    require(
        overlap(&c.translate(0.0, 0.0, o.cell_clear + 0.2), &seated) > 1e-3,
        "nothing holds the battery down: the plate must sit just above it",
    )?;
    for (dx, dy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
        let s = o.cell_gap + 0.1;
        require(
            overlap(&c.translate(dx * s, dy * s, 0.0), &body) > 1e-3,
            "the battery can slide about in its pocket",
        )?;
    }
    // The boards, their parts and the plate: inside the box and under the lid.
    let inside = union_all(&[seated.clone(), boards.clone()]);
    require(
        overlap(&boards, &body) < 1e-3,
        format!("the {name} boards hit the box"),
    )?;
    require(
        bounds(&inside).1[2] <= o.top - 1.0,
        format!("the {name} boards stand too tall for the lid"),
    )?;
    let lid = outdoor::lid_in_place(d, o)?;
    require(overlap(&lid, &body) < 1e-3, "the lid collides with the box")?;
    // Ports: each on a flat part of its wall below the rim's slope, with room for its nut.
    let flat = o.cavity / 2.0 - o.cavity_chamfer;
    let obstacles = union_all(&[
        columns(d, o),
        battery_pocket(d, o),
        c.clone(),
        inside.clone(),
    ]);
    for (i, p) in o.ports.iter().enumerate() {
        let what = p.kind.name();
        require(
            p.x.abs() + p.kind.outside_d() / 2.0 <= flat
                && p.x.abs() + p.kind.inside().0 / 2.0 <= flat,
            format!("the {what} at x = {} runs onto the box's corner", p.x),
        )?;
        require(
            p.z + p.kind.outside_d() / 2.0 <= o.flare_z() - 0.5,
            format!("the {what} runs into the slope under the rim"),
        )?;
        require(
            p.z - p.kind.inside().0 / 2.0 >= o.floor + 1.0,
            format!("the {what} sits too low in the wall"),
        )?;
        require(
            overlap(&port_inside(o, p), &obstacles) < 1e-3,
            format!("the {what}'s nut hits something inside the box"),
        )?;
        require(
            overlap(&port_outside(o, p), &lid) < 1e-3,
            format!("the {what} hits the lid"),
        )?;
        for q in &o.ports[i + 1..] {
            require(
                overlap(&port_inside(o, p), &port_inside(o, q)) < 1e-3
                    && overlap(&port_outside(o, p), &port_outside(o, q)) < 1e-3,
                "two ports are too close together",
            )?;
        }
    }
    // The tripod thread opens at the back and stops inside the boss.
    let thread = outdoor::socket_cut(o, o.thread_clear);
    require(
        bounds(&thread).1[2] <= o.deck_z - o.deck_t - 1.5 + EPS,
        "the tripod thread breaks into the box",
    )?;
    Ok(())
}

/// The payload plate and its boards, for the Uno and Nano case.
pub fn verify_uno_nano(d: &Design, o: &Outdoor) -> Check {
    let make = |t: Option<f64>| payload_plate(d, t, 0.0);
    verify(
        d,
        o,
        &Payload {
            name: "Uno and Nano",
            make: &make,
            boards: uno_nano(d),
        },
    )
}

/// The XIAO plate and boards, for the XIAO case: each board goes in and is held.
pub fn verify_xiao(d: &Design, o: &Outdoor) -> Check {
    let j = xiao::mg24_joint(d);
    let seated = xiao::features(d, d.hook_preload + 0.01);
    let spread = xiao::xiao_plate(d, None, d.hook_reach + 0.05);
    let [_, hy, _, hh] = xiao::HALOW;
    board_held(
        d,
        &Board {
            name: "HaLow",
            solid: xiao::halow_board(d),
            rect: xiao::HALOW,
            top: xiao::halow_top(d),
            lips: &xiao::HALOW_LIPS,
            hooks: &xiao::HALOW_HOOKS,
            axis: 'x',
            pivot: [0.0, hy + hh, xiao::halow_top(d)],
            spread: spread.clone(),
            sign: -1.0,
        },
        &seated,
    )?;
    let [_, my, _, _] = xiao::MG24;
    board_held(
        &j,
        &Board {
            name: "MG24 Sense",
            solid: xiao::mg24_board(d),
            rect: xiao::MG24,
            top: xiao::mg24_top(d),
            lips: &xiao::MG24_LIPS,
            hooks: &xiao::MG24_HOOKS,
            axis: 'x',
            pivot: [0.0, my, xiao::mg24_top(d)],
            spread,
            sign: 1.0,
        },
        &seated,
    )?;
    require(
        overlap(
            &xiao::mg24_board(d),
            &xiao::xiao_plate(d, None, d.hook_preload + 0.01),
        ) < 1e-3,
        "the MG24's pins don't go through their slots",
    )?;
    require(
        overlap(&xiao::halow_board(d), &xiao::mg24_board(d)) < 1e-3,
        "the two boards overlap",
    )?;
    // Nothing stands on a snap pin's fingers, and no hook's finger runs into them.
    let top = d.plate_t;
    let mut zones: Vec<Manifold> = xiao::HALOW_HOOKS
        .iter()
        .map(|e| hook_zone(d, xiao::HALOW, e, -1.0, top + 2.0))
        .collect();
    zones.extend(
        xiao::MG24_HOOKS
            .iter()
            .map(|e| hook_zone(&j, xiao::MG24, e, -1.0, top + 2.0)),
    );
    let feats =
        xiao::features(d, 0.0).difference(&bx(-d.plate, -d.plate, -1.0, d.plate, d.plate, top));
    for (i, [x, y]) in d.plate_holes().into_iter().enumerate() {
        if let Some(angle) = d.snap_angle(i) {
            let fit = d.column_fit(i);
            require(
                overlap(&feats, &finger_zone(d, x, y, angle, fit, top, 3.0)) < 1e-3,
                "a XIAO mount stands on a snap pin's flex fingers",
            )?;
            for z in &zones {
                require(
                    overlap(z, &finger_zone(d, x, y, angle, fit, -1.0, top + 2.0)) < 1e-3,
                    "a XIAO board hook's finger runs into a snap pin's fingers",
                )?;
            }
        }
    }
    require(
        feats
            .slice_to_cross_section(top + 0.1)
            .difference(&plate_outline(d))
            .area()
            < 1e-3,
        "a XIAO mount reaches past the plate's edge",
    )?;
    let make = |t: Option<f64>| Ok(xiao::xiao_plate(d, t, 0.0));
    verify(
        d,
        o,
        &Payload {
            name: "XIAO",
            make: &make,
            boards: xiao::boards(d),
        },
    )
}

/// Deliberately broken outdoor designs: the checks must reject every one.
pub fn broken_designs() -> Vec<(&'static str, Design, Outdoor, bool)> {
    let with = |f: &dyn Fn(&mut Outdoor)| {
        let mut o = Outdoor::default();
        f(&mut o);
        o
    };
    let d = Design::default();
    vec![
        (
            "outdoor walls too thin",
            d.clone(),
            with(&|o| o.wall = 2.0),
            false,
        ),
        (
            "gasket barely squeezed: it would leak",
            d.clone(),
            with(&|o| o.gasket = 2.2),
            false,
        ),
        (
            "gasket crushed",
            d.clone(),
            with(&|o| o.gasket = 3.0),
            false,
        ),
        (
            "lid screws inside the seal",
            d.clone(),
            with(&|o| o.screw_at = 7.0),
            false,
        ),
        (
            "box too small for the plate",
            d.clone(),
            with(&|o| o.cavity = 90.0),
            false,
        ),
        (
            "box too short for the boards",
            d.clone(),
            with(&|o| {
                o.top = 46.0;
                o.ports.clear();
            }),
            false,
        ),
        (
            "tripod thread breaking into the box",
            d.clone(),
            with(&|o| o.thread_len = 11.0),
            false,
        ),
        (
            "tripod thread too short to hold",
            d.clone(),
            with(&|o| o.thread_len = 5.0),
            false,
        ),
        (
            "battery jammed against the plate",
            d.clone(),
            with(&|o| o.cell_clear = 0.0),
            false,
        ),
        (
            "cable gland on the box's corner",
            d.clone(),
            with(&|o| o.ports[0].x = -33.0),
            false,
        ),
        (
            "cable gland in the slope under the rim",
            d.clone(),
            with(&|o| o.ports[0].z = 56.0),
            false,
        ),
        (
            "cable gland's nut on the Nano",
            d.clone(),
            with(&|o| o.ports[0].z = 41.0),
            false,
        ),
        (
            "vent on top of the gland",
            d.clone(),
            with(&|o| o.ports[1].x = -15.0),
            false,
        ),
        (
            "antenna bulkhead's nut on the XIAO plate",
            d.clone(),
            {
                let mut o = Outdoor::xiao();
                o.ports[3].z = 30.0;
                o
            },
            true,
        ),
    ]
}
