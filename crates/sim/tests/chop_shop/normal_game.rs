//! The Chop Shop in a normal game (the plan's acceptance, slice M5-N6: "from the Cults' start: an Armory, then a Chop
//! Shop, then a Seeker. The Airstrip and the Machine Shop are no longer greyed once the Chop Shop stands"): `dev_skirmish`
//! from slot 1's Cults Starting Layout (a Recruitment Hall, a Storage, six Recruits and no Space Crystals), the player
//! giving their own commands through the gate, no debug command, as a `testing::cults` line plays them.

use sc_sim::testing::cults::skirmish::{self, ARMORY_SITE, CULTS, NEAR};
use sc_sim::testing::cults::{Build, Crew, Cults};
use sc_sim::{BuildingOrientation, Sim};

use super::*;

/// Where the line builds its Chop Shop: on (17, 2) to (19, 4), unturned, east of the Gunworks' site of the golden games
/// and north-east of the Armory's, on open Plane west of the lake; its entrance, B, east, its exit, C, west.
const CHOP_SHOP_SITE: TilePos = TilePos::new(17, 2);

/// Whether player 1's Recruits' Construct menu would grey a Build of `kind` for its TechPrerequisite: the gate's verdict
/// on the Build the panel probes (`conditions/probes.rs`: at the map's corner, unturned, the placement's own check left
/// out), `Reject::Requires`.
fn behind_a_requirement(sim: &Sim, kind: ObjectKind) -> Option<ObjectKind> {
    let recruit = (sim.objects()).find(|o| o.owner == Some(CULTS) && o.kind == RECRUIT)?;
    let build = Command::Order {
        units: vec![recruit.id],
        kind: OrderKind::Build {
            kind,
            origin: TilePos::new(0, 0),
            orientation: BuildingOrientation::default(),
        },
        queue: false,
    };
    match sim.validate(&PlayerCommand {
        player: CULTS,
        command: build,
    }) {
        Err(Reject::Requires(missing)) => Some(missing),
        _ => None,
    }
}

/// Player 1's units of `kind` on the map with no order.
fn idle(sim: &Sim, kind: ObjectKind) -> Vec<ObjectId> {
    (sim.objects())
        .filter(|o| o.owner == Some(CULTS) && o.kind == kind)
        .filter(|o| o.orders.as_ref().is_some_and(|orders| orders.is_idle()))
        .map(|o| o.id)
        .collect()
}

/// The plan's acceptance. The line deals the Recruits to the field nearest the Storage, builds an Armory and, once it
/// stands, a Chop Shop (`factions/cults/units/recruit.md > Constructs`: "Chop Shop: 200 Space Crystals, 640 frames
/// [...] Requires: Armory"), which its Recruits can't order before then: the gate refuses it for the missing Armory,
/// and the Airstrip's and the Machine Shop's Builds for the missing Chop Shop (`factions/cults/structures/airstrip.md`,
/// `factions/cults/structures/machine_shop.md`: TechPrerequisite a Chop Shop) until it is complete. Then two spare
/// Recruits are ordered in, and the Chop Shop trains a Seeker of them for 75 Space Crystals, which drives out of side C.
#[test]
fn from_the_cults_start_an_armory_then_a_chop_shop_then_a_seeker() {
    let mut t = TestSim::new(skirmish::scenario(false));
    let mut line = Cults::new(CULTS)
        .deal(NEAR)
        .build(Build::new(ObjectKind::Armory, ARMORY_SITE, Crew::Nearest(3)))
        .build(Build::new(CHOP_SHOP, CHOP_SHOP_SITE, Crew::Nearest(3)));
    // A tick of the game: the line's commands, then the test's own, which come last and so win over the line's.
    let play_with = |t: &mut TestSim, line: &mut Cults, own: Option<Command>| {
        let tick = t.sim().tick() + 1;
        for c in line.commands(t.sim()) {
            t.at(tick, c.player, c.command);
        }
        if let Some(command) = own {
            t.at(tick, CULTS, command);
        }
        step_checked(t);
    };
    let play = |t: &mut TestSim, line: &mut Cults| play_with(t, line, None);
    assert_eq!(behind_a_requirement(t.sim(), CHOP_SHOP), Some(ObjectKind::Armory));
    let complete = |sim: &Sim, kind| (sim.objects()).any(|o| o.owner == Some(CULTS) && o.kind == kind && !o.is_site());
    while !complete(t.sim(), ObjectKind::Armory) {
        assert!(t.sim().tick() < 8000, "the Armory");
        play(&mut t, &mut line);
        assert_eq!(
            behind_a_requirement(t.sim(), CHOP_SHOP).is_some(),
            !complete(t.sim(), ObjectKind::Armory)
        );
    }
    let armory = t.sim().tick();
    while !complete(t.sim(), CHOP_SHOP) {
        assert!(t.sim().tick() < armory + 8000, "the Chop Shop");
        for kind in [ObjectKind::Airstrip, ObjectKind::MachineShop] {
            assert_eq!(behind_a_requirement(t.sim(), kind), Some(CHOP_SHOP), "{kind:?}");
        }
        play(&mut t, &mut line);
    }
    let chop_shop = (t.sim().objects())
        .find(|o| o.kind == CHOP_SHOP && o.owner == Some(CULTS))
        .map(|o| o.id)
        .expect("the Chop Shop");
    for kind in [ObjectKind::Airstrip, ObjectKind::MachineShop, ObjectKind::ScrapHangar] {
        let expected = (kind == ObjectKind::ScrapHangar).then_some(ObjectKind::Airstrip);
        assert_eq!(behind_a_requirement(t.sim(), kind), expected, "{kind:?}");
    }
    println!(
        "the Armory complete in tick {armory}, the Chop Shop in {}",
        t.sim().tick()
    );
    let built = t.sim().tick();
    let crew = loop {
        let spare = idle(t.sim(), RECRUIT);
        if spare.len() >= 2 && t.sim().player(CULTS).is_some_and(|p| p.crystals >= PRICE) {
            break [spare[0], spare[1]];
        }
        assert!(
            t.sim().tick() < built + 4000,
            "two spare Recruits and 75 Space Crystals"
        );
        play(&mut t, &mut line);
    };
    play_with(&mut t, &mut line, Some(enter(&crew, chop_shop)));
    while inside(&t, chop_shop).len() < 2 {
        assert!(t.sim().tick() < built + 6000, "in");
        play(&mut t, &mut line);
    }
    let start = t.sim().tick() + 1;
    play_with(&mut t, &mut line, Some(train(&[chop_shop], RecruitKind::Plain)));
    let seeker = loop {
        if let Some(&(_, _, unit)) = produced(&t).iter().find(|&&(_, by, _)| by == chop_shop) {
            break unit;
        }
        assert!(t.sim().tick() < start + FRAMES + 2, "the Seeker");
        play(&mut t, &mut line);
    };
    let o = t.sim().object(seeker).expect("out");
    assert_eq!(o.kind, SEEKER);
    assert!(crew.contains(&seeker));
    println!("the Seeker out in tick {}", t.sim().tick());
    let out = t.sim().tick();
    while t.mover(seeker.0).exit.is_some() {
        assert!(t.sim().tick() < out + 60, "out of the doorway");
        play(&mut t, &mut line);
    }
}
