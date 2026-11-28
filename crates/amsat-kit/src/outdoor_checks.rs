//! Checks for the outdoor cases. They build the parts and fail if the seal, the latches, the
//! tripod thread, the ports, the battery or the boards don't fit. Design checks only: no case
//! has been printed, and nothing here is a water test.

use manifold_csg::Manifold;

use crate::boards::{nano_board, nano_parts, uno_board, uno_parts};
use crate::checks::{Board, Check, board_held, plate_on_columns, require};
use crate::design::Design;
use crate::geom::{EPS, bounds, bx, overlap, union_all};
use crate::joints::{contact_r, finger_zone, hook_zone};
use crate::outdoor::{self, Outdoor, battery_pocket, cell, columns, port_inside, port_outside};
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
    let squeeze = o.oring_squeeze();
    require(
        (0.18..=0.3).contains(&squeeze),
        format!(
            "the O-ring is squeezed {:.0}%: 18 to 30% seals without crushing it",
            squeeze * 100.0
        ),
    )?;
    require(
        o.groove_fill() <= 0.85,
        format!(
            "the O-ring fills {:.0}% of its groove: over 85% and it has nowhere to go when squeezed",
            o.groove_fill() * 100.0
        ),
    )?;
    require(
        o.skirt_wall >= 2.0 && o.groove_depth() > 0.5,
        "the lid's skirt is too thin behind its O-ring groove",
    )?;
    require(
        o.latch_strain() <= 0.012,
        format!(
            "a lid latch would bend {:.2}% (limit 1.2%)",
            o.latch_strain() * 100.0
        ),
    )?;
    require(
        o.latch_w >= 2.0 && o.lug_t >= 2.0,
        "the lid latches and lugs must be at least 2 mm",
    )?;
    let e = o.lid_over;
    require(
        e - o.latch_w - o.latch_slit >= o.skirt_clear + 0.5,
        "the lid's latch slits run into its skirt",
    )?;
    require(
        e + o.lug_gap + o.lug_t <= o.wall + o.flange - 1.0,
        "the lugs run off the rim",
    )?;
    require(o.lug_reach >= 0.6, "the lugs barely reach over the lid")?;
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
    let lid = outdoor::lid_in_place(d, o, 0.0)?;
    lid_latches(o, &body, &|f| outdoor::lid_in_place(d, o, f))?;
    require(
        overlap(&lid, &inside) < 1e-3,
        format!("the lid's skirt hits the {name} boards or plate"),
    )?;
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
/// The lid on a box: it sits on the rim and under the lugs' catches, can't lift off, goes on
/// with its latches pushed in, and can't slide sideways off a catch, because its skirt holds
/// it square in the opening.
/// `lid` builds the lid in place on the box, with its fingers pushed in by the given amount.
pub fn lid_latches(
    o: &Outdoor,
    body: &Manifold,
    lid: &dyn Fn(f64) -> Result<Manifold, String>,
) -> Check {
    require(
        outdoor::latches_on_edges(o),
        "a latch finger, its slit or its lug runs onto a corner chamfer",
    )?;
    let make = lid;
    let lid = make(0.0)?;
    require(overlap(&lid, body) < 1e-3, "the lid collides with the box")?;
    require(
        overlap(&lid.translate(0.0, 0.0, 0.3), body) > 1e-3,
        "nothing holds the lid down: the lugs don't catch it",
    )?;
    // A finger can only go in as far as the slit behind it allows.
    let push = (o.lug_reach + 0.15).min(o.latch_slit - 0.2);
    let pushed = make(push)?;
    for dz in [0.5, 1.0, 2.0, 3.0] {
        require(
            overlap(&pushed.translate(0.0, 0.0, dz), body) < 1e-3,
            "the lid can't be pressed on past its lugs",
        )?;
    }
    // Pushed sideways away from a lug by as far as its catch reaches, the lid must hit the box
    // wall: otherwise the lug just shoves the lid aside instead of bending the finger.
    for a in o.latch_angles() {
        let r = a.to_radians();
        let away = lid.translate(-o.lug_reach * r.cos(), -o.lug_reach * r.sin(), 0.0);
        require(
            overlap(&away, body) > 1e-3,
            "the lid can slide sideways off its catches: nothing holds it square",
        )?;
    }
    Ok(())
}

/// The latch coupon: its box and lid go together exactly like the real ones.
pub fn verify_latch_coupon(d: &Design, o: &Outdoor) -> Check {
    let c = outdoor::latch_coupon_case(o);
    let body = outdoor::latch_coupon_box(&c);
    lid_latches(&c, &body, &|f| {
        Ok(outdoor::latch_coupon_lid(d, &c, f)?.translate(0.0, 0.0, c.top))
    })
}

/// Latch coupons that must be refused, each by a different check. `o` is the real case.
pub fn broken_latch_coupons() -> Vec<(&'static str, Outdoor)> {
    let o = Outdoor::default();
    let with = |f: &dyn Fn(&mut Outdoor)| {
        let mut o = o.clone();
        f(&mut o);
        o
    };
    vec![
        (
            "latch coupon lid loose between its skirt and lugs: it could slide off a catch",
            with(&|o| {
                o.skirt_clear = 1.2;
                o.lug_gap = 1.5;
            }),
        ),
        (
            "latch coupon fingers too long for its lid's straight edge",
            with(&|o| o.latch_tab = 12.0),
        ),
    ]
}

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
            "O-ring barely squeezed: it would leak",
            d.clone(),
            with(&|o| o.gland = 2.2),
            false,
        ),
        ("O-ring crushed", d.clone(), with(&|o| o.gland = 1.5), false),
        (
            "O-ring groove too narrow",
            d.clone(),
            with(&|o| o.groove_w = 3.4),
            false,
        ),
        (
            "lid latch too short: it would crack",
            d.clone(),
            with(&|o| o.latch_to_lug = 15.0),
            false,
        ),
        (
            "lid latch slits into the skirt",
            d.clone(),
            with(&|o| o.lid_over = 3.5),
            false,
        ),
        (
            "lugs barely reaching over the lid",
            d.clone(),
            with(&|o| o.lug_reach = 0.3),
            false,
        ),
        (
            "lugs' catch 0.6 mm above the lid: it would rattle",
            d.clone(),
            with(&|o| o.lug_clear = 0.6),
            false,
        ),
        (
            "latch slit too narrow for the finger to clear its lug",
            d.clone(),
            with(&|o| o.latch_slit = 0.9),
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
