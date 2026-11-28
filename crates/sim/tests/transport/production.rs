//! The Vehicle Assembly trains the Transport (`factions/syndicate/structures/vehicle_assembly.md > Placeholder stats`:
//! "**Subterranean Transport** 200 SC, 400 f, 3", "**Subterranean Transport** (requires a Forge)", "**R** Subterranean
//! Transport"): into the Tunnel Network, as every Syndicate vehicle (plan decision T100), in its surfaced kind, which a
//! Tier 2 Tunnel lets out and its owner's infantry board.

use sc_sim::network::Place;
use sc_sim::{ProductionCommand, TICKS_PER_SECOND};

use super::*;

const ASSEMBLY: ObjectKind = ObjectKind::VehicleAssembly;
const FORGE: ObjectKind = ObjectKind::Forge;
/// Where the Vehicle Assembly stands: (9, 7) to (11, 9), north-west of the Tunnel, in its Tier 2 area.
const ASSEMBLY_AT: TilePos = TilePos::new(9, 7);
/// Where the Forge stands: (16, 8) to (17, 9), north-east of the Tunnel, clear of Side A's paths.
const FORGE_AT: TilePos = TilePos::new(16, 8);
/// `> Placeholder stats`: "200 SC, 400 f".
const PRICE: u32 = 200;
const FRAMES: Tick = 400;

/// A Train of a Transport at `producer`.
fn train(producer: ObjectId) -> Command {
    Command::Production {
        objects: vec![producer],
        command: ProductionCommand::Train(TRANSPORT),
    }
}

/// The plan's "the VA's R trains it behind a Forge": without a Forge the Train is refused ([`Reject::Requires`]); with
/// one, paid at the click, its Tunnel Space of 3 reserved from the start, the Transport comes into the game 400 frames
/// on inside the network, in its surfaced kind. Ejected by the Tier 2 Tunnel, it comes out of Side A's exit path, and
/// a Guard boards it.
#[test]
fn the_vehicle_assembly_trains_it_behind_a_forge_for_200_in_400_frames() {
    let (no_forge, ..) = network(&[(ASSEMBLY, ASSEMBLY_AT)], 0, 2);
    let va = id(2);
    assert_eq!(object(&no_forge, va).kind, ASSEMBLY);
    assert_eq!(gate(&no_forge, P1, train(va)), Err(Reject::Requires(FORGE)));
    let (mut t, transport, guards) = network(&[(ASSEMBLY, ASSEMBLY_AT), (FORGE, FORGE_AT)], 1, 2);
    let guard = guards[0];
    let space = space_used(&t, P1);
    assert_eq!(gate(&t, P1, train(va)), Ok(()));
    t.at(2, P1, train(va));
    t.run_until(2);
    assert_eq!(
        t.sim().player(P1).map(|p| p.crystals),
        Some(BANK - PRICE),
        "paid at the click"
    );
    assert_eq!(
        space_used(&t, P1),
        space + TRANSPORT_SPACE,
        "its Tunnel Space, from the start"
    );
    let before: Vec<ObjectId> = inside(&t);
    step_until(&mut t, FRAMES + 10, "it is trained", |t| inside(t).len() > before.len());
    assert_eq!(
        t.sim().tick(),
        2 + FRAMES - 1,
        "400 frames, {} s",
        FRAMES / TICKS_PER_SECOND
    );
    let [trained] = inside(&t)[..] else {
        panic!("one unit inside: {:?}", inside(&t));
    };
    assert_ne!(trained, transport);
    let held = t.sim().held_unit(trained).expect("held");
    assert_eq!((held.object.kind, held.place), (TRANSPORT, Place::Network));
    assert_eq!(space_used(&t, P1), space + TRANSPORT_SPACE);
    eject_out(&mut t, T1, TRANSPORT, trained);
    let out = object(&t, trained);
    assert_eq!(out.kind, TRANSPORT);
    assert!(
        out.pos.y < 640.0 && (out.pos.x - 926.0).abs() < 1.0,
        "out of Side A's exit path: {}",
        out.pos
    );
    board(&mut t, &[guard], trained);
    assert_eq!(aboard(&t, trained), [guard]);
}
