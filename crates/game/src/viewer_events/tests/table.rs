//! The rows of the filter's table (`every_kind_of_event_reaches_whom_the_table_says`), in the world of `super::world`;
//! those of the buildings that hold units in `table/holding.rs`, and (M5-N4b) Shroud's in `table/shroud.rs`.

use sc_sim::{ArmyResource, BuildingOrientation, PlacementCheck, Refusal};

use super::*;

mod holding;
mod shroud;

/// One event of every kind, a few kinds more than once, each with what four viewers perceive of it: player 1, the
/// owner of what it is about (or the shooter's owner); player 2, an enemy who sees where it is; player 3, an enemy in
/// the fog; and player 3 with the fog off.
pub(super) fn table(sim: &Sim) -> Vec<(SimEvent, [Seen; 4])> {
    let at = |id| sim.object(id).unwrap().pos;
    let (unit, watcher, hideout, turret) = (at(UNIT), at(WATCHER), at(HIDEOUT), at(TURRET));
    let hidden = Hiding::of(sim.object(HIDEOUT).unwrap());
    // Player 1's turret as it stands fully lowered in this world (slice M4-S2; design change 0360: from its first
    // tick), and as it is once raised: the rule's own answers for the two (`Hiding::of`).
    let lowered = sim.object(TURRET).unwrap();
    assert_eq!(lowered.popup, Some(sc_sim::popup::PopUp::LOWERED));
    let raised = sc_sim::Object {
        popup: Some(sc_sim::popup::PopUp::Raised {
            idle: sc_sim::Progress::new(32),
        }),
        ..lowered.clone()
    };
    let (lowered, raised) = (Hiding::of(lowered), Hiding::of(&raised));
    assert!(lowered.hides_from(P2) && !lowered.hides_from(P1) && raised == Hiding::default());
    let origin = TilePos::new(12, 2);
    let orientation = BuildingOrientation::default();
    let problems = PlacementCheck::default();
    let (kind, refund) = (ObjectKind::Hideout, 75);
    let mut rows = vec![
        (SimEvent::Spawned(UNIT), [W, W, N, W]),
        (SimEvent::Spawned(HIDEOUT), [W, N, N, W]),
        (SimEvent::Removed(DEAD), [W, W, N, W]),
        (SimEvent::Removed(GONE), [W, W, N, W]),
        (SimEvent::Removed(HELD), [W, N, N, N]),
        (SimEvent::Removed(HIDDEN_DEAD), [W, N, N, W]),
        (
            SimEvent::SuppliesDelivered {
                station: TUNNEL,
                amount: 4,
            },
            [W, W, N, W],
        ),
        (
            SimEvent::SupplyStationEmpty {
                station: TUNNEL,
                owner: P1,
            },
            [W, N, N, N],
        ),
        (SimEvent::MoveFailed { unit: UNIT }, [W, N, N, N]),
        (
            SimEvent::Attacked {
                attacker: UNIT,
                target: WATCHER,
                from: unit,
                to: watcher,
            },
            [P, P, P, W],
        ),
        // A structure's shot, as any other.
        (
            SimEvent::Attacked {
                attacker: TURRET,
                target: WATCHER,
                from: turret,
                to: watcher,
            },
            [P, P, P, W],
        ),
        (
            SimEvent::Hit {
                attacker: WATCHER,
                target: UNIT,
                damage: 9.0,
                shield_damage: 0.0,
            },
            [W, W, N, W],
        ),
        (
            SimEvent::Hit {
                attacker: UNIT,
                target: HIDEOUT,
                damage: 9.0,
                shield_damage: 0.0,
            },
            [W, N, N, W],
        ),
        (
            SimEvent::Hit {
                attacker: WATCHER,
                target: DEAD,
                damage: 9.0,
                shield_damage: 0.0,
            },
            [W, W, N, W],
        ),
        (
            SimEvent::Died {
                id: DEAD,
                kind: ObjectKind::Peacekeeper,
                owner: Some(P1),
                pos: centre(5, 5),
                hiding: Hiding::default(),
            },
            [W, W, N, W],
        ),
        (
            SimEvent::Died {
                id: HIDDEN_DEAD,
                kind: ObjectKind::Hideout,
                owner: Some(P1),
                pos: hideout,
                hiding: hidden,
            },
            [W, N, N, W],
        ),
        // A lowered pop-up turret's death: shown to its owner, and to no enemy, though one sees its tile.
        (
            SimEvent::Died {
                id: LOWERED,
                kind: ObjectKind::LaserTurret,
                owner: Some(P1),
                pos: turret,
                hiding: lowered,
            },
            [W, N, N, W],
        ),
        // The same turret dying raised is seen like any structure's death.
        (
            SimEvent::Died {
                id: DEAD,
                kind: ObjectKind::LaserTurret,
                owner: Some(P1),
                pos: turret,
                hiding: raised,
            },
            [W, W, N, W],
        ),
        // (M5-V4) A crush is a death, and the crushed unit's removal reaches whom its death did.
        (
            SimEvent::Crushed {
                id: CRUSHED,
                kind: ObjectKind::Peacekeeper,
                owner: Some(P1),
                pos: centre(5, 5),
                hiding: Hiding::default(),
                crusher: WATCHER,
            },
            [W, W, N, W],
        ),
        (SimEvent::Removed(CRUSHED), [W, W, N, W]),
        (
            SimEvent::PatchDepleted {
                patch: ObjectId(80),
                plate: TUNNEL,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::ProductionQueued {
                producer: TUNNEL,
                kind: ObjectKind::Agent,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::ProductionRefused {
                producer: TUNNEL,
                kind: ObjectKind::Agent,
                reason: Refusal::Crystals,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::ProductionCancelled {
                producer: TUNNEL,
                kind: ObjectKind::Agent,
                refund,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::ProductionWaiting {
                producer: TUNNEL,
                reason: ArmyResource::TunnelSpace,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::UnitProduced {
                producer: TUNNEL,
                unit: UNIT,
            },
            [W, W, N, W],
        ),
        (
            SimEvent::UnitProduced {
                producer: TUNNEL,
                unit: HELD,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::ConstructionCompleted {
                constructor: TUNNEL,
                kind,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::ConstructionCancelled {
                constructor: TUNNEL,
                kind,
                refund,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::StructurePlaced {
                constructor: TUNNEL,
                structure: TUNNEL,
            },
            [W, W, N, W],
        ),
        (
            SimEvent::PlacementRefused {
                constructor: TUNNEL,
                kind,
                origin,
                orientation,
                problems: problems.clone(),
            },
            [W, N, N, N],
        ),
        (SimEvent::PlayerDefeated { player: P1 }, [W, W, W, W]),
        (SimEvent::GameOver { winners: vec![P2] }, [W, W, W, W]),
        (
            SimEvent::ResourcesDelivered {
                worker: UNIT,
                drop_off: TUNNEL,
                amount: 5,
                resource: Default::default(),
            },
            [W, N, N, N],
        ),
        (
            SimEvent::KindChanged {
                id: UNIT,
                from: ObjectKind::Peacekeeper,
                to: ObjectKind::Peacekeeper,
            },
            [W, W, N, W],
        ),
        (
            SimEvent::TransformCompleted {
                id: TUNNEL,
                kind: ObjectKind::Tunnel,
            },
            [W, W, N, W],
        ),
        (
            SimEvent::TransformCancelled {
                id: TUNNEL,
                kind: ObjectKind::Tunnel,
                refund,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::DeployFailed {
                drone: UNIT,
                origin,
                problems: problems.clone(),
            },
            [W, N, N, N],
        ),
        (
            SimEvent::EnteredNetwork {
                unit: HELD,
                tunnel: TUNNEL,
            },
            [W, W, N, W],
        ),
        (
            SimEvent::LeftNetwork {
                unit: UNIT,
                tunnel: TUNNEL,
            },
            [W, W, N, W],
        ),
        (
            SimEvent::EnterFailed {
                unit: UNIT,
                target: TUNNEL,
                full: false,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::TargetTooClose {
                unit: UNIT,
                target: WATCHER,
            },
            [W, N, N, N],
        ),
        // Slice M5-O3: an Attack Ground's location inside the MinRange of a unit that can't back off.
        (
            SimEvent::LocationTooClose {
                unit: UNIT,
                at: watcher,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::BuildFailed {
                worker: UNIT,
                kind,
                origin,
                orientation,
                problems,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::BuildRefunded {
                worker: HELD,
                kind,
                refund,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::SiteStarted {
                worker: HELD,
                site: TUNNEL,
            },
            [W, W, N, W],
        ),
        (
            SimEvent::SiteCompleted {
                site: TUNNEL,
                worker: HELD,
            },
            [W, W, N, W],
        ),
        (SimEvent::SiteBuilt { site: TUNNEL }, [W, W, N, W]),
        (
            SimEvent::SiteCancelled {
                site: GONE,
                kind,
                refund,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::LeftSite {
                worker: UNIT,
                site: GONE,
            },
            [W, W, N, W],
        ),
        (
            SimEvent::ExpansionStarted {
                tunnel: TUNNEL,
                site: HIDEOUT,
            },
            [W, N, N, W],
        ),
        (
            SimEvent::ExpansionCompleted {
                tunnel: TUNNEL,
                site: HIDEOUT,
            },
            [W, N, N, W],
        ),
        (
            SimEvent::ExpansionCancelled {
                tunnel: TUNNEL,
                site: GONE,
                kind,
                refund,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::UpgradeStarted {
                tunnel: TUNNEL,
                to: 2,
                price: 100,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::UpgradeCompleted {
                tunnel: TUNNEL,
                tier: 2,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::UpgradeCancelled {
                tunnel: TUNNEL,
                to: 2,
                refund: 100,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::ProjectileLaunched {
                shooter: UNIT,
                target: Some(WATCHER),
                from: unit,
                to: watcher,
                lands_at: 30,
                homing: false,
            },
            [P, P, P, W],
        ),
        (
            SimEvent::Blast {
                at: unit,
                radius: 40.0,
                attacker: None,
            },
            [W, W, N, W],
        ),
        // (M5-N5) A Shrine's cast, at a point player 1 sees (the gate's rule): where it was cast, as a blast, to whoever
        // sees the point. The caster stands for a Shrine of player 1's: the point is what counts.
        (
            SimEvent::Cast {
                caster: TUNNEL,
                ability: sc_sim::AreaAbility::HealthRegeneration,
                at: unit,
            },
            [W, W, N, W],
        ),
        // A TailDisjointed shot that hit nothing (slice M5-O3), aimed at an object or at the ground: a shot as any
        // other.
        (
            SimEvent::Missed {
                attacker: UNIT,
                target: Some(WATCHER),
                from: unit,
                to: watcher,
            },
            [P, P, P, W],
        ),
        (
            SimEvent::Missed {
                attacker: UNIT,
                target: None,
                from: unit,
                to: watcher,
            },
            [P, P, P, W],
        ),
    ];
    rows.extend(holding::rows());
    rows.extend(shroud::rows(unit));
    rows
}
