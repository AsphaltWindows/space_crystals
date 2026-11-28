//! A Tunnel builds the Vault (`factions/syndicate/structures/vault.md`: "Tier Requirement - 1", "TechPrerequisite -
//! player owns at least one Hideout (anywhere in the Tunnel Network)", "Size 2x2; MaxHP 400 ...; Cost 150 Space
//! Crystals; Build Time 400 frames (25 seconds)"; `factions/syndicate/tunnel_menus.md > Expansions under construction`,
//! design change 0254), and the Vehicle Assembly trains the Shadebringer
//! (`factions/syndicate/structures/vehicle_assembly.md > Placeholder stats`: "**Shadebringer** 150 SC, 320 f, 2";
//! "**W** Shadebringer ... (greyed until their prerequisites are owned)"; "requires a Vault").

use sc_sim::network::Place;
use sc_sim::patches::Tanks;
use sc_sim::tunnel::Operation;

use super::*;

/// Where a Hideout stands in [`network`]'s Tier 1 area, south-east of the Tunnel: (16, 14) to (17, 15).
const HIDEOUT_AT: TilePos = TilePos::new(16, 14);
/// Where [`network`]'s Vehicle Assembly stands: (9, 7) to (11, 9), north-west of the Tunnel, in its Tier 2 area.
const ASSEMBLY_AT: TilePos = TilePos::new(9, 7);

/// An Expand of a Vault at [`VAULT_AT`], unturned, to [`T1`].
fn expand_vault() -> Command {
    let command = TunnelCommand::Expand {
        kind: VAULT,
        origin: VAULT_AT,
        orientation: BuildingOrientation::default(),
    };
    Command::Tunnel { tunnel: T1, command }
}

/// A Train of a Shadebringer at `producer`.
fn train(producer: ObjectId) -> Command {
    Command::Production {
        objects: vec![producer],
        command: ProductionCommand::Train(SHADEBRINGER),
    }
}

/// The plan's "A Tunnel builds it (Tier 1, with a Hideout)": without a Hideout a Tier 1 Tunnel is refused the Vault for
/// it ([`Reject::Requires`]), with one it takes it. At Tier 2, with a Shadebringer emptied and driven into the network,
/// the Vault is built for 150 Space Crystals, paid at the click, in 400 frames, from 40 of its 400 HP; the
/// Shadebringer's tank stays empty while it is a site ("While the player owns a Vault in operation", as the Null
/// Chamber's, `Object::is_operational`), and fills from the tick it is finished, 5 a second.
#[test]
fn a_tunnel_with_a_hideout_builds_the_vault_for_150_in_400_frames_which_serves_once_finished() {
    let (no_hideout, _) = network_at(1, &[], 0);
    assert_eq!(
        gate(&no_hideout, P1, expand_vault()),
        Err(Reject::Requires(ObjectKind::Hideout))
    );
    let (tier_1, _) = network_at(1, &[(ObjectKind::Hideout, HIDEOUT_AT)], 0);
    assert_eq!(gate(&tier_1, P1, expand_vault()), Ok(()), "Tier 1");
    let (mut t, sbs) = network(&[(ObjectKind::Hideout, HIDEOUT_AT)], 1);
    let sb = sbs[0];
    emptied(&mut t, &sbs);
    drive_in(&mut t, sb);
    let click = t.sim().tick() + 1;
    t.at(click, P1, expand_vault());
    run_until(&mut t, click);
    assert_eq!(
        t.sim().player(P1).map(|p| p.crystals),
        Some(BANK - 150),
        "paid at the click"
    );
    let site = match object(&t, T1).tunnel.as_ref().and_then(|state| state.operation) {
        Some(Operation::Building(site)) => site,
        other => panic!("the Tunnel builds nothing: {other:?}"),
    };
    let o = object(&t, site);
    assert_eq!(
        (o.kind, o.is_site(), o.hp, o.max_hp()),
        (VAULT, true, Some(40.0), Some(40.0))
    );
    while object(&t, site).is_site() {
        assert_eq!(
            tank(&t, sb).used,
            CAPACITY,
            "tick {}: no refill at a site",
            t.sim().tick()
        );
        t.step();
    }
    let o = object(&t, site);
    assert_eq!(
        (t.sim().tick(), o.hp, o.max_hp()),
        (click + 400, Some(400.0), Some(400.0))
    );
    assert_eq!(t.sim().parent_tunnel(site), Some(T1));
    assert_eq!(tank(&t, sb).used, CAPACITY - REFILL, "from the tick it is finished");
    assert!(o.production.is_none(), "no commands");
}

/// The plan's "The Vehicle Assembly trains it (W, with a Vault)": without a Vault the Train is refused for it
/// ([`Reject::Requires`]); with one, paid at the click, the Shadebringer comes into the game 320 frames on inside the
/// network (plan decision T100: "Syndicate vehicles go into the network"), its Tunnel Space of 2 taken, and its tanks
/// full and off ("a toggle can be switched on with **any charge**": full, **Placeholder (not in the design: a new
/// Shadebringer's charges)**). Ejected, it switches its Sludge on at once.
#[test]
fn the_vehicle_assembly_trains_the_shadebringer_behind_a_vault_for_150_in_320_frames() {
    let assembly = (ObjectKind::VehicleAssembly, ASSEMBLY_AT);
    let (no_vault, _) = network(&[assembly], 0);
    let va = id(2);
    assert_eq!(object(&no_vault, va).kind, ObjectKind::VehicleAssembly);
    assert_eq!(gate(&no_vault, P1, train(va)), Err(Reject::Requires(VAULT)));
    let (mut t, _) = network(&[assembly, (VAULT, VAULT_AT)], 0);
    assert_eq!(gate(&t, P1, train(va)), Ok(()));
    t.at(2, P1, train(va));
    run_until(&mut t, 2);
    assert_eq!(
        t.sim().player(P1).map(|p| p.crystals),
        Some(BANK - 150),
        "paid at the click"
    );
    let cap = |t: &TestSim| t.sim().army_cap(P1).expect("a Syndicate player").used;
    assert_eq!(cap(&t), 2, "its Tunnel Space, from the start");
    step_until(&mut t, 400, "it is trained", |t| !inside(t).is_empty());
    let done = t.sim().tick();
    assert_eq!(done, 2 + 320 - 1, "320 frames");
    let [sb] = inside(&t)[..] else {
        panic!("one unit inside: {:?}", inside(&t));
    };
    let held = t.sim().held_unit(sb).expect("held");
    assert_eq!((held.object.kind, held.place), (SHADEBRINGER, Place::Network));
    assert_eq!(held.object.tanks, Some(Tanks::default()), "full and off");
    assert_eq!(cap(&t), 2);
    let out = eject_out(&mut t);
    t.at(out + 1, P1, press(&[sb], sludge(true)));
    run_until(&mut t, out + 1);
    assert!(tank(&t, sb).on, "on at once");
}
