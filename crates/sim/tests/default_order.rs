//! A unit's default command against a target (`sc_sim::orders::resolution`, `Sim::default_order`): the one rule of
//! the right-click and the rally point (`systems/controls/interface.md > RightClickResolution`, design changes
//! 0062, 0063, 0074, 0136, 0139 and 0140; `factions/colonists/units/drone.md > Right-Click Resolution`), as a table
//! of unit kinds against kinds of targets, in a game and as the pure rule.

use sc_sim::combat::domain_allows;
use sc_sim::defs::{Domain, TargetDomain, Transit};
use sc_sim::orders::resolution::{Abilities, Aim, Aimed, resolve};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Ability, DebugCommand, Faction, ObjectId, ObjectKind, OrderKind, PlayerId, SpawnAt, Target, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn at(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// The units of the table, player 1's: ids 1 to 6, the last a Peacekeeper whose Riot Shield is up.
const UNITS: [(&str, ObjectKind); 6] = [
    ("Peacekeeper", ObjectKind::Peacekeeper),
    ("Guard", ObjectKind::Guard),
    ("Drone", ObjectKind::Drone),
    ("Sabre", ObjectKind::SabreLegionnaire),
    ("Rifle", ObjectKind::RifleLegionnaire),
    ("shielded Peacekeeper", ObjectKind::Peacekeeper),
];

/// The targets of the table, by id (the units are 1 to 6).
const OWN_UNIT: u32 = 2;
const OWN_NODE: u32 = 7;
const OWN_STRUCTURE: u32 = 8;
const ENEMY_UNIT: u32 = 9;
const ENEMY_STRUCTURE: u32 = 10;
const PATCH: u32 = 11;
const PLATED_PATCH: u32 = 12;
const STATION: u32 = 14;
const REMEMBERED_STRUCTURE: u32 = 15;
const REMEMBERED_PATCH: u32 = 16;

/// Player 1 (Colonists) with the units of [`UNITS`] in a column, its Egress Node (7) and a Generator (8); player 2's
/// Peacekeeper (9) and Generator (10) in sight; a free patch (11) and one under player 2's Extraction Plate (12, 13),
/// and a Supply Delivery Station (14), in sight; and far east, a Generator of player 2's (15) and a patch (16) that
/// player 1's scout (17) saw in tick 1 before it was removed: remembered.
fn game() -> TestSim {
    let mut builder = ScenarioBuilder::new(&format!("{}\n", ".".repeat(48)).repeat(16)).faction(P1, Faction::Colonists);
    for (i, (_, kind)) in UNITS.iter().enumerate() {
        builder = builder.spawn(*kind, Some(P1), at(300.0, 300.0 + 60.0 * i as f32));
    }
    let builder = builder
        .spawn(ObjectKind::EgressNode, Some(P1), tile(1, 2)) // 7
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 10)) // 8
        .spawn(ObjectKind::Peacekeeper, Some(P2), at(520.0, 420.0)) // 9
        .spawn(ObjectKind::Generator, Some(P2), tile(8, 3)) // 10
        .spawn(ObjectKind::CrystalPatch, None, tile(7, 8)) // 11
        .spawn(ObjectKind::CrystalPatch, None, tile(7, 10)) // 12
        .spawn(ObjectKind::ExtractionPlate, Some(P2), tile(7, 10)) // 13
        .spawn(ObjectKind::SupplyDeliveryStation, None, tile(6, 12)) // 14
        .spawn(ObjectKind::Generator, Some(P2), tile(40, 3)) // 15
        .spawn(ObjectKind::CrystalPatch, None, tile(40, 8)) // 16
        .spawn(ObjectKind::Peacekeeper, Some(P1), at(2500.0, 400.0)); // 17
    let mut t = builder.start();
    t.without_auto_targeting()
        .ability_at(1, P1, &[ObjectId(6)], Ability::RiotShield { raise: true })
        .debug_at(2, DebugCommand::Remove { id: ObjectId(17) })
        .run(12);
    let sim = t.sim();
    for n in [
        OWN_UNIT,
        OWN_NODE,
        OWN_STRUCTURE,
        ENEMY_UNIT,
        ENEMY_STRUCTURE,
        PATCH,
        PLATED_PATCH,
        STATION,
    ] {
        assert!(sim.is_visible(P1, sim.object(ObjectId(n)).unwrap()), "#{n} in sight");
    }
    for n in [REMEMBERED_STRUCTURE, REMEMBERED_PATCH] {
        assert!(
            matches!(sim.target_view(P1, ObjectId(n)), sc_sim::TargetView::Remembered { .. }),
            "#{n}"
        );
    }
    let shielded = sim.object(ObjectId(6)).unwrap();
    assert_eq!(shielded.kind, ObjectKind::PeacekeeperShielded);
    assert!(!sc_sim::riot_shield::allows_attack(shielded));
    t
}

/// The table: every unit kind against every kind of target, in a game. An armed unit attacks an enemy, seen or
/// remembered (design change 0139), unless its shield is up (0074); everything else is a Move, to the point or to
/// the object (a Follow for a unit). The Drone, unarmed (design change 0154: its HoverVehicle base can carry a
/// turret, it has none), gathers a patch it can gather, seen or remembered, and moves to or follows everything else:
/// an enemy too (design change 0136), its own Egress Node while it carries nothing (0140), and a patch under a plate
/// (question Q0037 point 11).
#[test]
fn the_default_order_of_each_unit_kind_on_each_kind_of_target() {
    let t = game();
    let sim = t.sim();
    let point = Vec2::new(900.0, 900.0);
    let follow = |n: u32| Some(OrderKind::Move(Target::Object(ObjectId(n))));
    let attack = |n: u32| Some(OrderKind::Attack(ObjectId(n)));
    let gather = |n: u32| Some(OrderKind::Gather(ObjectId(n)));
    for (i, (name, kind)) in UNITS.iter().enumerate() {
        let unit = ObjectId(i as u32 + 1);
        let armed = *kind != ObjectKind::Drone && *name != "shielded Peacekeeper";
        let worker = *kind == ObjectKind::Drone;
        let expected = [
            (OWN_UNIT, follow(OWN_UNIT)),
            (OWN_NODE, follow(OWN_NODE)),
            (OWN_STRUCTURE, follow(OWN_STRUCTURE)),
            (ENEMY_UNIT, if armed { attack(ENEMY_UNIT) } else { follow(ENEMY_UNIT) }),
            (
                ENEMY_STRUCTURE,
                if armed {
                    attack(ENEMY_STRUCTURE)
                } else {
                    follow(ENEMY_STRUCTURE)
                },
            ),
            (PATCH, if worker { gather(PATCH) } else { follow(PATCH) }),
            (PLATED_PATCH, follow(PLATED_PATCH)),
            (STATION, follow(STATION)),
            (
                REMEMBERED_STRUCTURE,
                if armed {
                    attack(REMEMBERED_STRUCTURE)
                } else {
                    follow(REMEMBERED_STRUCTURE)
                },
            ),
            (
                REMEMBERED_PATCH,
                if worker {
                    gather(REMEMBERED_PATCH)
                } else {
                    follow(REMEMBERED_PATCH)
                },
            ),
        ];
        assert_eq!(
            sim.default_order(unit, Target::Point(point)),
            Some(OrderKind::Move(Target::Point(point))),
            "{name} on the ground"
        );
        for (target, order) in expected {
            assert_eq!(
                sim.default_order(unit, Target::Object(ObjectId(target))),
                order,
                "{name} on #{target}"
            );
        }
        // Nothing to resolve against: an object the player knows nothing of, or none at all.
        assert_eq!(sim.default_order(unit, Target::Object(ObjectId(99))), None, "{name}");
    }
    // Not a unit: a structure has no default order of its own (its right-click sets its rally point).
    assert_eq!(sim.default_order(ObjectId(OWN_STRUCTURE), Target::Point(point)), None);
}

/// The rule itself (`resolve`), on what no unit of the game has yet: a loaded worker on its drop-off (design change
/// 0140: "Own Egress Node (carrying resources) → Drop off resources") and on another's; attacks of each TargetDomain
/// against targets of each domain (`systems/combat.md > ValidTarget` 3); an indestructible enemy; and the Riot
/// Shield, which turns any attack into a follow (0074).
#[test]
fn the_rule_on_abilities_and_targets() {
    let armed = |domain| Abilities {
        kind: ObjectKind::Guard,
        attack: Some(domain),
        may_attack: true,
        worker: false,
        carrying: false,
        gathers_supplies: false,
        illusion: false,
        transit: Transit::Infantry,
        aircraft: None,
    };
    let drone = |carrying| Abilities {
        kind: ObjectKind::Drone,
        attack: None,
        may_attack: true,
        worker: true,
        carrying,
        gathers_supplies: false,
        illusion: false,
        transit: Transit::Vehicles,
        aircraft: None,
    };
    let object = |enemy, destructible, domain, gatherable, drop_off| Aimed {
        id: ObjectId(5),
        enemy,
        destructible,
        domain,
        gatherable,
        station: false,
        drop_off,
        transit: &[],
        accepts: &[],
        full: false,
        site: None,
        repair: None,
        airfield: false,
        refuses_illusions: false,
        carrier: None,
    };
    let follow = OrderKind::Move(Target::Object(ObjectId(5)));
    let attack = OrderKind::Attack(ObjectId(5));
    let node = object(false, true, Domain::Ground, false, true);
    let theirs = object(true, true, Domain::Ground, false, false);
    assert_eq!(resolve(drone(true), Aim::Object(node)), OrderKind::DropOff(ObjectId(5)));
    assert_eq!(resolve(drone(false), Aim::Object(node)), follow);
    assert_eq!(resolve(drone(true), Aim::Object(theirs)), follow);
    for attack_domain in [TargetDomain::Ground, TargetDomain::Air, TargetDomain::Universal] {
        for domain in [Domain::Ground, Domain::Air, Domain::Underground] {
            let target = object(true, true, domain, false, false);
            let expected = if domain_allows(attack_domain, domain) {
                attack
            } else {
                follow
            };
            assert_eq!(
                resolve(armed(attack_domain), Aim::Object(target)),
                expected,
                "{attack_domain:?} on {domain:?}"
            );
            let shielded = Abilities {
                may_attack: false,
                ..armed(attack_domain)
            };
            assert_eq!(
                resolve(shielded, Aim::Object(target)),
                follow,
                "{attack_domain:?} shielded"
            );
            let own = object(false, true, domain, false, false);
            assert_eq!(
                resolve(armed(attack_domain), Aim::Object(own)),
                follow,
                "never an own object (0059)"
            );
        }
    }
    let indestructible = object(true, false, Domain::Ground, false, false);
    assert_eq!(
        resolve(armed(TargetDomain::Universal), Aim::Object(indestructible)),
        follow
    );
    // An own Tunnel (slice M4-C, `systems/controls/commands.md > Enter`): Enter for a unit its tier lets in, after a
    // loaded worker's Drop Off there; (M6-SD) an air unit's at Tier 3 alone, by the air doors (plan decision T96:
    // "Enter on an own Tier 3 Tunnel for an air unit"), a Move to it below.
    let enter = OrderKind::Enter(ObjectId(5));
    let tunnel = |transit, drop_off| Aimed {
        transit,
        ..object(false, true, Domain::Ground, false, drop_off)
    };
    let (tier_1, tier_3) = (
        &[Transit::Infantry][..],
        &[Transit::Infantry, Transit::Vehicles, Transit::Air][..],
    );
    let infantry = armed(TargetDomain::Ground);
    assert_eq!(resolve(infantry, Aim::Object(tunnel(tier_1, false))), enter);
    assert_eq!(
        resolve(drone(false), Aim::Object(tunnel(tier_1, false))),
        follow,
        "a vehicle at Tier 1"
    );
    assert_eq!(resolve(drone(false), Aim::Object(tunnel(tier_3, false))), enter);
    assert_eq!(
        resolve(drone(true), Aim::Object(tunnel(tier_3, true))),
        OrderKind::DropOff(ObjectId(5))
    );
    assert_eq!(
        resolve(drone(false), Aim::Object(tunnel(tier_3, true))),
        enter,
        "not carrying"
    );
    let flyer = Abilities {
        transit: Transit::Air,
        ..infantry
    };
    assert_eq!(resolve(flyer, Aim::Object(tunnel(tier_3, false))), enter);
    let tier_2 = &[Transit::Infantry, Transit::Vehicles][..];
    assert_eq!(
        resolve(flyer, Aim::Object(tunnel(tier_2, false))),
        follow,
        "an air unit at Tier 2"
    );
    let p = Vec2::new(1.0, 2.0);
    assert_eq!(resolve(drone(true), Aim::Ground(p)), OrderKind::Move(Target::Point(p)));
    // A Supply Delivery Station (slice M4-I, `factions/syndicate/units/agent.md > Right-Click Resolution`: "Supply
    // Delivery Station → Gather supplies"): Gather for a worker that gathers Supplies, loaded or not (design change
    // 0256 point 4: it "goes straight to the station"); a Move to it for any other unit, a Drone too.
    let station = Aimed {
        station: true,
        ..object(false, false, Domain::Ground, false, false)
    };
    for carrying in [false, true] {
        let agent = Abilities {
            gathers_supplies: true,
            transit: Transit::Infantry,
            ..drone(carrying)
        };
        assert_eq!(resolve(agent, Aim::Object(station)), OrderKind::Gather(ObjectId(5)));
        assert_eq!(resolve(drone(carrying), Aim::Object(station)), follow);
    }
    assert_eq!(resolve(armed(TargetDomain::Ground), Aim::Object(station)), follow);
    // An own building that holds units (slice M5-F1; `factions/cults/units/recruit.md > Right-Click Resolution`: "Own
    // Armory → Enter Armory"): Enter for a unit of a kind it takes, loaded or not; a Move to it for any other.
    let armory = Aimed {
        accepts: &[ObjectKind::Recruit],
        ..object(false, true, Domain::Ground, false, false)
    };
    for carrying in [false, true] {
        let recruit = Abilities {
            kind: ObjectKind::Recruit,
            attack: Some(TargetDomain::Ground),
            ..drone(carrying)
        };
        assert_eq!(resolve(recruit, Aim::Object(armory)), enter);
    }
    assert_eq!(resolve(armed(TargetDomain::Ground), Aim::Object(armory)), follow);
    // (M5-E) An own building under construction that Recruits build from outside (`factions/cults/units/recruit.md >
    // Right-Click Resolution`: "In-progress own building → Assist Construction"): Assist for a unit that builds its
    // kind, loaded or not; a Move to it for any other. An Armory under construction takes no one in, so a Recruit
    // assists it.
    let assist = OrderKind::Assist(ObjectId(5));
    for kind in [ObjectKind::Storage, ObjectKind::Armory, ObjectKind::RecruitmentHall] {
        let site = Aimed {
            site: Some(kind),
            ..object(false, true, Domain::Ground, false, false)
        };
        for carrying in [false, true] {
            let recruit = Abilities {
                kind: ObjectKind::Recruit,
                attack: Some(TargetDomain::Ground),
                ..drone(carrying)
            };
            assert_eq!(resolve(recruit, Aim::Object(site)), assist, "{kind:?}");
            assert_eq!(resolve(drone(carrying), Aim::Object(site)), follow, "{kind:?}");
        }
        assert_eq!(resolve(armed(TargetDomain::Ground), Aim::Object(site)), follow);
    }
    // (M5-N8) A damaged own building or vehicle, complete, that the unit's type repairs (`factions/cults/units/
    // recruit.md > Right-Click Resolution`: "Damaged own building or vehicle (completed) → Repair, **unless the
    // building can be entered** by the selection: then **Enter** wins"; `factions/syndicate/units/agent.md >
    // Right-Click Resolution`: "Own Tunnel (not carrying resources) → Enter (also when damaged: Enter wins over
    // Repair)", "Damaged own Laser Turret → Repair"): Repair, loaded or not; a Move to it for any other unit.
    let repair = OrderKind::Repair(ObjectId(5));
    let damaged = |kind| Aimed {
        repair: Some(kind),
        ..object(false, true, Domain::Ground, false, false)
    };
    for carrying in [false, true] {
        let recruit = Abilities {
            kind: ObjectKind::Recruit,
            attack: Some(TargetDomain::Ground),
            transit: Transit::Infantry,
            ..drone(carrying)
        };
        let agent = Abilities {
            kind: ObjectKind::Agent,
            attack: Some(TargetDomain::Ground),
            gathers_supplies: true,
            transit: Transit::Infantry,
            ..drone(carrying)
        };
        for kind in [ObjectKind::Storage, ObjectKind::RecruitmentHall, ObjectKind::Gunworks] {
            assert_eq!(resolve(recruit, Aim::Object(damaged(kind))), repair, "{kind:?}");
            assert_eq!(resolve(agent, Aim::Object(damaged(kind))), follow, "{kind:?}");
        }
        // The Armory takes Recruits in: Enter wins.
        let armory = Aimed {
            accepts: &[ObjectKind::Recruit],
            ..damaged(ObjectKind::Armory)
        };
        assert_eq!(resolve(recruit, Aim::Object(armory)), enter);
        // (M5-S2b) Unless it is full (design change 0378: "a damaged enterable building that is **full at the moment
        // of the click** resolves to **Repair** [...]. This applies to every building that holds units"): the Recruit
        // repairs a full damaged Armory or Garrison, and enters a full one with nothing to repair; a Militant, whose
        // type repairs nothing, gets the Enter, which ends at the door (the normal Full-building rule).
        let full = |aimed: Aimed| Aimed { full: true, ..aimed };
        assert_eq!(resolve(recruit, Aim::Object(full(armory))), repair);
        let garrison = Aimed {
            accepts: &[ObjectKind::Recruit, ObjectKind::Militant],
            ..damaged(ObjectKind::Garrison)
        };
        assert_eq!(resolve(recruit, Aim::Object(garrison)), enter, "a free slot");
        assert_eq!(resolve(recruit, Aim::Object(full(garrison))), repair);
        let militant = Abilities {
            kind: ObjectKind::Militant,
            worker: false,
            carrying: false,
            ..recruit
        };
        assert_eq!(resolve(militant, Aim::Object(full(garrison))), enter);
        let whole = Aimed {
            repair: None,
            ..full(armory)
        };
        assert_eq!(resolve(recruit, Aim::Object(whole)), enter, "nothing to repair");
        assert_eq!(resolve(agent, Aim::Object(damaged(ObjectKind::LaserTurret))), repair);
        // No one repairs a Tunnel's expansion: it regenerates.
        assert_eq!(resolve(agent, Aim::Object(damaged(ObjectKind::Hideout))), follow);
        // A damaged Tunnel in operation lets the Agent in: Enter wins; a loaded Agent drops off there first.
        let tunnel = Aimed {
            transit: tier_1,
            drop_off: true,
            ..damaged(ObjectKind::Tunnel)
        };
        let expected = if carrying {
            OrderKind::DropOff(ObjectId(5))
        } else {
            enter
        };
        assert_eq!(resolve(agent, Aim::Object(tunnel)), expected);
    }
    for kind in [ObjectKind::Storage, ObjectKind::LaserTurret, ObjectKind::Tunnel] {
        assert_eq!(resolve(drone(false), Aim::Object(damaged(kind))), follow, "{kind:?}");
        assert_eq!(
            resolve(armed(TargetDomain::Ground), Aim::Object(damaged(kind))),
            follow,
            "{kind:?}"
        );
    }
    // (M6-CM) An illusion (`sc_sim::mirage`) enters a Garrison as a Disciple does, and follows an Altar, which a
    // Disciple enters: "Illusions can't be augmented at the Altar themselves" (`factions/cults/units/disciple.md >
    // Spells > Mirage`).
    let disciple = Abilities {
        kind: ObjectKind::Disciple,
        worker: false,
        transit: Transit::Infantry,
        ..drone(false)
    };
    let illusion = Abilities {
        illusion: true,
        ..disciple
    };
    let infantry = &[ObjectKind::Recruit, ObjectKind::Disciple][..];
    let garrison = Aimed {
        accepts: infantry,
        ..object(false, true, Domain::Ground, false, false)
    };
    let altar = Aimed {
        refuses_illusions: true,
        ..garrison
    };
    let enter = OrderKind::Enter(ObjectId(5));
    assert_eq!(resolve(disciple, Aim::Object(garrison)), enter);
    assert_eq!(resolve(illusion, Aim::Object(garrison)), enter);
    assert_eq!(resolve(disciple, Aim::Object(altar)), enter);
    assert_eq!(resolve(illusion, Aim::Object(altar)), follow);
    // (M6-ST1) An own carrier that loads now (`sc_sim::carry`) is boarded by the infantry, full or not ("**Loading**:
    // right-click the transport with infantry selected (Enter)",
    // `factions/syndicate/units/subterranean_transport.md > Placeholder stats`), and followed by anyone else: a
    // vehicle, and itself.
    let hold = ObjectKind::SubterraneanTransport
        .def()
        .unit
        .as_ref()
        .and_then(|unit| unit.carries.as_ref());
    let transport = Aimed {
        carrier: hold,
        ..object(false, true, Domain::Ground, false, false)
    };
    let guard = armed(TargetDomain::Ground);
    assert_eq!(resolve(guard, Aim::Object(transport)), enter);
    assert_eq!(resolve(disciple, Aim::Object(transport)), enter, "Light Infantry");
    let full = Aimed {
        full: true,
        ..transport
    };
    assert_eq!(resolve(guard, Aim::Object(full)), enter, "full: the arrival judges");
    let bike = Abilities {
        kind: ObjectKind::AssaultBike,
        ..guard
    };
    assert_eq!(resolve(bike, Aim::Object(transport)), follow, "a vehicle");
    let itself = Abilities {
        kind: ObjectKind::SubterraneanTransport,
        attack: None,
        ..guard
    };
    assert_eq!(resolve(itself, Aim::Object(transport)), follow);
    // Underground, or anyone else's: not a carrier to the unit (`Aimed::carrier`).
    let shut = Aimed {
        carrier: None,
        ..transport
    };
    assert_eq!(resolve(guard, Aim::Object(shut)), follow);
}
