//! The Factory's production (`factions/gdo/structures/factory.md > Produces`, "Production as the Barracks (queue 20,
//! refunds, rally)"): the Raptor and the Twister at their cost and time, Unit Control 2 each
//! (`factions/gdo/units/raptor.md`, `factions/gdo/units/twister.md > Placeholder stats`: "UnitControlCost 2"),
//! reserved as its production starts (design change 0143), and the
//! Twister's Supply (design change 0441: "+ 1 Supply"): the tests give player 1 theirs with
//! `DebugCommand::SetResources`, over the GDO's starting 1 (`factions/gdo/economy.md > Starting Conditions`). The
//! Prospector and the Artillery Rig, its E and R, are held until slices M6-GP and M6-GA
//! (`sc_sim::defs::PLANNED_RECIPES`): it trains neither.

use sc_sim::economy::ArmyResource;

use super::*;

/// Player 1's Unit Control in use: their units' and the ones in production's (`Sim::army_cap`).
fn unit_control(t: &TestSim) -> u32 {
    let cap = t.sim().army_cap(P1).expect("player 1's");
    assert_eq!(cap.resource, ArmyResource::UnitControl);
    cap.used
}

/// "Raptor: 100 Space Crystals, 240 frames (15 seconds)" and "Twister: 125 Space Crystals, 320 frames (20 seconds)",
/// "+ 1 Supply": a Raptor and a Twister queued in tick 2 take 225 Space Crystals and the Supply at once; the Raptor,
/// first, is in production at once, its Unit Control 2 reserved from that tick (`Sim::army_cap`), the Twister's not
/// yet. The Raptor comes out at the end of tick 241, 240 frames with the command's own tick the first
/// (`docs/architecture/economy.md > Economy > Recipe timing`); the Twister starts in the next tick, its 2 reserved then, and comes
/// out 320 frames later. Both are player 1's, each 2 of the 4 then in use.
#[test]
fn the_factory_trains_both_vehicles_at_their_cost_and_time_unit_control_reserved_at_the_start() {
    let mut t = works(1);
    let used = unit_control(&t);
    assert_eq!(gate(&t, P1, train(RAPTOR)), Ok(()));
    t.at(2, P1, train(RAPTOR)).at(2, P1, train(TWISTER));
    t.run_until(2);
    assert_eq!(bank(&t), (BANK - RAPTOR_PRICE - TWISTER_PRICE, 0));
    assert_eq!(unit_control(&t), used + 2, "the Raptor's, at its start");
    let status = t.sim().production_status(WORKS).expect("in production");
    assert_eq!(status.entry.unit(), Some(RAPTOR));
    t.run_until(1 + RAPTOR_FRAMES);
    let raptor_out = 1 + RAPTOR_FRAMES;
    let [(tick, raptor)] = produced(&t, WORKS)[..] else {
        panic!("one Raptor: {:?}", produced(&t, WORKS));
    };
    assert_eq!(tick, raptor_out);
    t.step();
    let status = t.sim().production_status(WORKS).expect("in production");
    assert_eq!(status.entry.unit(), Some(TWISTER));
    assert_eq!(unit_control(&t), used + 4, "the Raptor's and the Twister's");
    t.run_until(raptor_out + TWISTER_FRAMES);
    let twister = match produced(&t, WORKS)[..] {
        [_, (tick, twister)] => {
            assert_eq!(tick, raptor_out + TWISTER_FRAMES);
            twister
        }
        ref all => panic!("a Raptor and a Twister: {all:?}"),
    };
    for (unit, kind) in [(raptor, RAPTOR), (twister, TWISTER)] {
        let o = object(&t, unit);
        assert_eq!((o.kind, o.owner), (kind, Some(P1)));
    }
    assert_eq!(unit_control(&t), used + 4);
    assert_eq!(t.sim().production_status(WORKS), None, "its queue done");
}

/// The Twister's Supply (design change 0441, slice M6-GD's gate): with no Supply its W is refused
/// (`Reject::InsufficientSupplies`, the client's "Not enough Supplies" on a greyed button) and nothing is paid or
/// queued, the Raptor's Q taken all the while; with one it is taken, its 125 Space Crystals and the Supply paid
/// together. A cancel gives both back.
#[test]
fn the_twister_needs_a_supply() {
    let mut t = works(0);
    assert_eq!(gate(&t, P1, train(TWISTER)), Err(Reject::InsufficientSupplies));
    assert_eq!(gate(&t, P1, train(RAPTOR)), Ok(()));
    t.at(2, P1, train(TWISTER));
    t.run_until(2);
    assert_eq!(bank(&t), (BANK, 0), "nothing paid");
    assert_eq!(t.sim().production_status(WORKS), None, "nothing queued");

    let mut t = works(1);
    assert_eq!(gate(&t, P1, train(TWISTER)), Ok(()));
    t.at(2, P1, train(TWISTER));
    t.run_until(2);
    assert_eq!(bank(&t), (BANK - TWISTER_PRICE, 0));
    assert_eq!(
        gate(&t, P1, train(TWISTER)),
        Err(Reject::InsufficientSupplies),
        "the one spent"
    );
    let status = t.sim().production_status(WORKS).expect("in production");
    assert_eq!(status.entry.unit(), Some(TWISTER));
    t.at(3, P1, production(WORKS, ProductionCommand::CancelProduction));
    t.run_until(3);
    assert_eq!(bank(&t), (BANK, 1), "both back");
}

/// "**E** Prospector, **R** Artillery Rig (greyed without Advanced Mechanics)": slice M6-GP's Prospector, its entry
/// held until then, which the Factory doesn't train, with Advanced Mechanics or without (`Reject::CannotProduce`; the
/// client greys E, "Not available yet"). (M6-GA) The Artillery Rig, live: refused without Advanced Mechanics
/// (`Reject::Requires`), queued with it (`tests/artillery_rig/` tests the rest).
#[test]
fn it_trains_no_prospector_yet_and_the_rig_behind_advanced_mechanics() {
    let without = works(5);
    let with = started(
        base().spawn(FACTORY, Some(P1), SpawnAt::Tile(WORKS_AT)).spawn(
            ObjectKind::AdvancedMechanics,
            Some(P1),
            tile(16, 16),
        ),
        5,
    );
    for t in [&without, &with] {
        let kind = ObjectKind::Prospector;
        assert_eq!(gate(t, P1, train(kind)), Err(Reject::CannotProduce(kind)));
    }
    let rig = ObjectKind::ArtilleryRig;
    let mechanics = ObjectKind::AdvancedMechanics;
    assert_eq!(gate(&without, P1, train(rig)), Err(Reject::Requires(mechanics)));
    assert_eq!(gate(&with, P1, train(rig)), Ok(()));
}
