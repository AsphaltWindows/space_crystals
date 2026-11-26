//! The Recruit's panel and its Construct menus as plain data (slice M5-J1; design changes 0322 and 0327): its letters,
//! D Repair awaiting its target (slice M5-N8); the two menus' fixed slots, every building greyed with the gate's reason (its
//! TechPrerequisite, the bank: read from a game) or as not in the game yet; a building's placement for the whole
//! group, which goes back to its menu; and Tab, which stops once on the Recruits (design change 0072).

use sc_sim::construction::site::BuildPrice;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{DebugCommand, Faction, PlayerId, SpawnAt, TilePos};

use super::*;
use crate::command_ui::feedback::{NOT_YET_TEXT, requires_text, unavailable_text};

const RECRUIT: ObjectKind = ObjectKind::Recruit;
const MILITANT: ObjectKind = ObjectKind::Militant;
const HALL: ObjectKind = ObjectKind::RecruitmentHall;
const STORAGE: ObjectKind = ObjectKind::Storage;
const ARMORY: ObjectKind = ObjectKind::Armory;
const GUNWORKS: ObjectKind = ObjectKind::Gunworks;
const GARRISON: ObjectKind = ObjectKind::Garrison;
const MACHINE_SHOP: ObjectKind = ObjectKind::MachineShop;
const SCRAP_HANGAR: ObjectKind = ObjectKind::ScrapHangar;
const ALTAR: ObjectKind = ObjectKind::Altar;
const CATHEDRAL: ObjectKind = ObjectKind::Cathedral;
const TEMPLE: ObjectKind = ObjectKind::Temple;
const SHRINE: ObjectKind = ObjectKind::Shrine;
const BASIC: InterfaceState = InterfaceState::BuildMenu(BuildMenu::Basic);
const ADVANCED: InterfaceState = InterfaceState::BuildMenu(BuildMenu::Advanced);
const P1: PlayerId = PlayerId(1);

/// Four Recruits, ids 1 to 4: one SelectionGroup (Groupable, design change 0072).
fn recruits() -> Selection {
    Selection::new((1..=4).map(|n| (id(n), RECRUIT)))
}

/// What the panel's `slot` says when pressed greyed out, if it is.
fn why_text(panel: &Panel, slot: CommandSlot) -> Option<&'static str> {
    panel.get(slot)?.why.map(unavailable_text)
}

/// The labels and the hints' names of the buttons the panel shows, slot by slot.
fn labels_and_names(panel: &Panel) -> Vec<(&'static str, String)> {
    let named = |(slot, shown): (CommandSlot, Option<PanelSlot>)| Some((shown?.label(), panel.name(slot)?));
    panel.slots().filter_map(named).collect()
}

/// `factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState commands` (design changes 0322, 0327
/// and 0398): "**Q** Move, **W** Basic Construct, **E** Advanced Construct, **A** Attack, **S** Assist, **D** Repair,
/// **X** Stop", each named as the design names it. 0398's Verification: "Recruits: W opens Basic Construct, E opens
/// Advanced Construct, A attacks": A awaits the target of an Attack, "as BasicCombatUnitInterfaceState's Attack"
/// (design change 0389's E, moved to A). Q, S and D (slice M5-N8) await their targets too, and X stops at once. No Hold
/// Position or Patrol button. Every command goes to all four ("both right-click and panel commands (Construct, Assist
/// Construction, Repair) are issued to all of them").
#[test]
fn the_recruit_has_q_move_w_and_e_construct_a_attack_s_assist_d_repair_and_x_stop() {
    use ButtonLook::Common;
    use PanelCommand::{Assist, Attack, BuildMenu as Menu, Move, Repair, Stop};
    let selection = recruits();
    let shown_panel = panel(&selection, InterfaceState::DefaultState, &Conditions::default());
    let c = PanelButton::Command;
    let (q, w, e) = (slot(0, 0), slot(0, 1), slot(0, 2));
    let (a, s, d, x) = (slot(1, 0), slot(1, 1), slot(1, 2), slot(2, 1));
    assert_eq!(
        shown(&shown_panel),
        [
            (q, c(Move), Common),
            (w, c(Menu(BuildMenu::Basic)), Common),
            (e, c(Menu(BuildMenu::Advanced)), Common),
            (a, c(Attack), Common),
            (s, c(Assist), Common),
            (d, c(Repair), Common),
            (x, c(Stop), Common),
        ]
    );
    let named = [
        ("Move", "Move"),
        ("Basic", "Basic Construct"),
        ("Adv.", "Advanced Construct"),
        ("Attack", "Attack"),
        ("Assist", "Assist Construction"),
        ("Repair", "Repair"),
        ("Stop", "Stop"),
    ];
    assert_eq!(labels_and_names(&shown_panel), named.map(|(l, n)| (l, n.to_string())));
    assert_eq!(press(&shown_panel, q), SlotPress::AwaitTarget(Move));
    assert_eq!(press(&shown_panel, w), SlotPress::OpenMenu(BuildMenu::Basic));
    assert_eq!(press(&shown_panel, e), SlotPress::OpenMenu(BuildMenu::Advanced));
    assert_eq!(press(&shown_panel, a), SlotPress::AwaitTarget(Attack));
    assert_eq!(press(&shown_panel, s), SlotPress::AwaitTarget(Assist));
    assert_eq!(press(&shown_panel, x), SlotPress::Issue(Stop, OrderKind::Stop));
    assert_eq!(press(&shown_panel, d), SlotPress::AwaitTarget(Repair));
    assert_eq!(why_text(&shown_panel, d), None);
    for absent in [PanelCommand::HoldPosition, PanelCommand::Patrol] {
        assert!(!offers(RECRUIT, absent), "{absent:?}");
    }
    let all: Vec<ObjectId> = (1..=4).map(id).collect();
    for command in [Move, Attack, Assist, Repair, Stop, PanelCommand::Build(STORAGE)] {
        assert_eq!(recipients(&selection, command), all, "{command:?}");
    }
    assert!(still_valid(
        InterfaceState::awaiting(Assist),
        &selection,
        true,
        &Conditions::default()
    ));
}

/// `factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState commands`, BuildMenu[Basic] (design
/// change 0327: "Fixed slots: every building keeps its key and is greyed until its TechPrerequisite is met, so no
/// hotkey ever moves"): Q Recruitment Hall, W Storage, A Armory, S Gunworks, D Garrison, X Altar, and Z back. Read from
/// a game with 250 Space Crystals: the Hall, 300, is greyed for the bank, the Armory, the Gunworks (slice M5-G2), the
/// Garrison (slice M5-N2) and the Altar (slice M5-N1b) for their TechPrerequisites ("Requires: Storage", "Requires:
/// Armory", "TechPrerequisite - player owns at least one Armory", the Altar's "Requires: Storage"); the Storage enters
/// its placement. Each Build shows its price. Once a Storage stands and the bank holds 300, A enters the Armory's
/// placement (0327's Verification: "Recruits: A then A places an Armory"), and so do Q the Hall's and X the Altar's;
/// once an Armory stands too, S enters the Gunworks' and D the Garrison's.
#[test]
fn the_basic_menu_keeps_its_slots_greying_what_cant_be_built() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::Build;
    let tile = |x, y| SpawnAt::Point(TilePos::new(x, y).center());
    let mut t = ScenarioBuilder::new(&vec![".".repeat(32); 24].join("\n"))
        .faction(P1, Faction::Cults)
        .spawn(HALL, Some(P1), SpawnAt::Tile(TilePos::new(4, 4)))
        .spawn(RECRUIT, Some(P1), tile(14, 12))
        .spawn(RECRUIT, Some(P1), tile(15, 12))
        .start();
    let bank = |crystals| DebugCommand::SetResources {
        player: P1,
        crystals,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, bank(250)).run(1);
    let selection = Selection::new([(id(2), RECRUIT), (id(3), RECRUIT)]);
    let menu = panel(&selection, BASIC, &Conditions::of(t.sim(), P1, &selection));
    let c = PanelButton::Command;
    let (q, w, a, s, d, x) = (slot(0, 0), slot(0, 1), slot(1, 0), slot(1, 1), slot(1, 2), slot(2, 1));
    assert_eq!(
        shown(&menu),
        [
            (q, c(Build(HALL)), Disabled),
            (w, c(Build(STORAGE)), Common),
            (a, c(Build(ARMORY)), Disabled),
            (s, c(Build(GUNWORKS)), Disabled),
            (d, c(Build(GARRISON)), Disabled),
            (CommandSlot::BACK, PanelButton::Back, Back),
            (x, c(Build(ALTAR)), Disabled),
        ]
    );
    let named = [
        ("RH", "Build Recruitment Hall"),
        ("SE", "Build Storage"),
        ("AY", "Build Armory"),
        ("GS", "Build Gunworks"),
        ("GA", "Build Garrison"),
        ("Back", "Back"),
        ("AR", "Build Altar"),
    ];
    assert_eq!(labels_and_names(&menu), named.map(|(l, n)| (l, n.to_string())));
    let price = |at| menu.get(at).and_then(|shown| shown.price);
    assert_eq!(
        [q, w, a, s, d, x].map(price),
        [300, 100, 200, 150, 150, 150].map(|crystals| Some(BuildPrice::crystals(crystals)))
    );
    assert_eq!(why_text(&menu, q), Some("Not enough Space Crystals"));
    assert_eq!(why_text(&menu, a), Some(requires_text(STORAGE)));
    assert_eq!(why_text(&menu, s), Some("Requires an Armory"));
    assert_eq!(why_text(&menu, d), Some("Requires an Armory"));
    assert_eq!(why_text(&menu, x), Some(requires_text(STORAGE)));
    for greyed in [q, a, s, d, x] {
        assert_eq!(press(&menu, greyed), SlotPress::Nothing, "{greyed:?}");
    }
    assert_eq!(press(&menu, w), SlotPress::AwaitBuild(STORAGE));
    assert_eq!(press(&menu, CommandSlot::BACK), SlotPress::Back);
    assert_eq!(BASIC.back(&selection), InterfaceState::DefaultState);

    let storage = DebugCommand::Spawn {
        kind: STORAGE,
        owner: Some(P1),
        at: SpawnAt::Tile(TilePos::new(20, 4)),
        orientation: Default::default(),
    };
    t.debug_at(2, storage).debug_at(2, bank(300)).run(1);
    let menu = panel(&selection, BASIC, &Conditions::of(t.sim(), P1, &selection));
    assert_eq!(press(&menu, a), SlotPress::AwaitBuild(ARMORY), "A then A");
    assert_eq!(press(&menu, q), SlotPress::AwaitBuild(HALL));
    assert_eq!(press(&menu, x), SlotPress::AwaitBuild(ALTAR), "A then X");
    assert_eq!(menu_of(RECRUIT, ARMORY), Some(BuildMenu::Basic));
    assert_eq!(menu_of(RECRUIT, ALTAR), Some(BuildMenu::Basic));
    assert_eq!(why_text(&menu, s), Some("Requires an Armory"));

    let armory = DebugCommand::Spawn {
        kind: ARMORY,
        owner: Some(P1),
        at: SpawnAt::Tile(TilePos::new(24, 4)),
        orientation: Default::default(),
    };
    t.debug_at(3, armory).run(1);
    let menu = panel(&selection, BASIC, &Conditions::of(t.sim(), P1, &selection));
    assert_eq!(menu.get(s).map(|shown| (shown.look, shown.why)), Some((Common, None)));
    assert_eq!(press(&menu, s), SlotPress::AwaitBuild(GUNWORKS), "A then S");
    assert_eq!(menu_of(RECRUIT, GUNWORKS), Some(BuildMenu::Basic));
    assert_eq!(menu.get(d).map(|shown| (shown.look, shown.why)), Some((Common, None)));
    assert_eq!(press(&menu, d), SlotPress::AwaitBuild(GARRISON), "A then D");
    assert_eq!(menu_of(RECRUIT, GARRISON), Some(BuildMenu::Basic));
}

/// `factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState commands`, BuildMenu[Advanced]
/// (design change 0327): A Chop Shop, S Machine Shop, D Airstrip, F Scrap Hangar; X Temple, C Cathedral, V Shrine; and
/// Z back, every label its own. The Machine Shop and the Scrap Hangar are built since slice M5-G2, the Temple since
/// slice M5-N3, the Cathedral since slice M5-N4a and the Shrine since slice M5-N5, each greyed by the gate while its
/// TechPrerequisite isn't met (`factions/cults/units/recruit.md > Constructs`: "Machine Shop [...] Requires: Chop
/// Shop", "Scrap Hangar [...] Requires: Airstrip", "Temple: 200 Space Crystals [...] Requires: Altar.", "Cathedral
/// [...] Requires: Temple", "Shrine: 200 Space Crystals [...] Requires: Temple."); the two the Recruits don't build yet
/// are greyed, "Not available yet" (0327's Verification: "W then A places a Chop Shop (greyed without an Armory)": its
/// Build comes with its slice). Read from a game with 1000 Space Crystals; once a Chop Shop, an Airstrip, an Altar and a
/// Temple stand (spawned), S, F, X, C and V enter their buildings' placements.
#[test]
fn the_advanced_menu_greys_what_cant_be_built() {
    use ButtonLook::{Back, Common, Disabled, Unavailable};
    use PanelCommand::{Build, Planned as P};
    let mut t = ScenarioBuilder::new(&vec![".".repeat(32); 24].join("\n"))
        .faction(P1, Faction::Cults)
        .spawn(RECRUIT, Some(P1), SpawnAt::Point(TilePos::new(14, 12).center()))
        .start();
    let bank = DebugCommand::SetResources {
        player: P1,
        crystals: 1000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, bank).run(1);
    let selection = Selection::new([(id(1), RECRUIT)]);
    let menu = panel(&selection, ADVANCED, &Conditions::of(t.sim(), P1, &selection));
    let entries = [
        (slot(1, 0), P(Planned::ChopShop), Unavailable, "CS", "Build Chop Shop"),
        (slot(1, 1), Build(MACHINE_SHOP), Disabled, "MS", "Build Machine Shop"),
        (slot(1, 2), P(Planned::Airstrip), Unavailable, "AP", "Build Airstrip"),
        (slot(1, 3), Build(SCRAP_HANGAR), Disabled, "SH", "Build Scrap Hangar"),
        (slot(2, 1), Build(TEMPLE), Disabled, "TE", "Build Temple"),
        (slot(2, 2), Build(CATHEDRAL), Disabled, "CL", "Build Cathedral"),
        (slot(2, 3), Build(SHRINE), Disabled, "SR", "Build Shrine"),
    ];
    // Z, Back, comes in slot order: after the A row, before the X row.
    let mut expected: Vec<_> = (entries.iter())
        .map(|&(at, entry, look, ..)| (at, PanelButton::Command(entry), look))
        .collect();
    expected.insert(4, (CommandSlot::BACK, PanelButton::Back, Back));
    assert_eq!(shown(&menu), expected);
    let mut named: Vec<_> = (entries.iter())
        .map(|&(.., label, name)| (label, name.to_string()))
        .collect();
    named.insert(4, ("Back", "Back".to_string()));
    assert_eq!(labels_and_names(&menu), named);
    let (s, f, x, c, v) = (slot(1, 1), slot(1, 3), slot(2, 1), slot(2, 2), slot(2, 3));
    assert_eq!(why_text(&menu, s), Some("Requires a Chop Shop"));
    assert_eq!(why_text(&menu, f), Some("Requires an Airstrip"));
    assert_eq!(why_text(&menu, x), Some("Requires an Altar"));
    assert_eq!(why_text(&menu, c), Some("Requires a Temple"));
    assert_eq!(why_text(&menu, v), Some("Requires a Temple"));
    let price = |at| menu.get(at).and_then(|shown| shown.price);
    assert_eq!(
        [s, f, x, c, v].map(price),
        [150, 200, 200, 250, 200].map(|crystals| Some(BuildPrice::crystals(crystals)))
    );
    for (at, entry, ..) in entries {
        if matches!(entry, P(_)) {
            assert_eq!(why_text(&menu, at), Some(NOT_YET_TEXT), "{entry:?}");
        }
        assert_eq!(press(&menu, at), SlotPress::Nothing, "{entry:?}");
    }
    assert!(offers(RECRUIT, PanelCommand::BuildMenu(BuildMenu::Advanced)));
    assert!(still_valid(ADVANCED, &selection, true, &Conditions::default()));
    assert_eq!(ADVANCED.back(&selection), InterfaceState::DefaultState);

    for (kind, x) in [
        (ObjectKind::ChopShop, 2),
        (ObjectKind::Airstrip, 8),
        (ObjectKind::Temple, 14),
        (ALTAR, 20),
    ] {
        let spawn = DebugCommand::Spawn {
            kind,
            owner: Some(P1),
            at: SpawnAt::Tile(TilePos::new(x, 2)),
            orientation: Default::default(),
        };
        t.debug_at(2, spawn);
    }
    t.run(1);
    let menu = panel(&selection, ADVANCED, &Conditions::of(t.sim(), P1, &selection));
    for (at, building) in [
        (s, MACHINE_SHOP),
        (f, SCRAP_HANGAR),
        (x, TEMPLE),
        (c, CATHEDRAL),
        (v, SHRINE),
    ] {
        assert_eq!(menu.get(at).map(|shown| (shown.look, shown.why)), Some((Common, None)));
        assert_eq!(press(&menu, at), SlotPress::AwaitBuild(building));
        assert_eq!(menu_of(RECRUIT, building), Some(BuildMenu::Advanced));
    }
}

/// A Construct menu's entry places its building for the whole ActiveGroup (`factions/cults/units/recruit.md > Recruit
/// ObjectInterfaceState`: "Construct sends every selected Recruit to build the **same** building together";
/// [`Builder::Recruits`]): the entry is marked while it is placed, the other entries wait, and Escape goes back to its
/// menu. The placement holds while the ActiveGroup is Recruits, and ends once it isn't. Recruits are Groupable, so
/// beside Militants they are one group: Tab stops once on them, then on the Militants, and back (design change 0072,
/// Verification 4: "Pressing Tab with Recruits and Soldiers selected stops once on the Recruits as a group"); there the
/// Recruits' commands are group commands, and their Build goes to the Recruits alone.
#[test]
fn a_construct_entry_places_its_building_for_the_whole_group() {
    use ButtonLook::{Disabled, Group, Pending};
    use PanelCommand::Build;
    let selection = recruits();
    let placing = Placing::recruits(STORAGE);
    assert_eq!(placing.builder, Builder::Recruits { queued: false });
    assert_eq!(placing.command(), Build(STORAGE));
    let placement = InterfaceState::AwaitingPlacement(placing);
    let conditions = Conditions::default();
    let shown_panel = panel(&selection, placement, &conditions);
    let look = |at| shown_panel.get(at).map(|shown| shown.look);
    assert_eq!(look(slot(0, 1)), Some(Pending), "the Storage");
    assert_eq!(look(slot(0, 0)), Some(Disabled), "the Hall waits");
    assert_eq!(placement.back(&selection), BASIC);
    assert!(still_valid(placement, &selection, true, &conditions));
    assert!(!still_valid(placement, &selection, false, &conditions));
    let militants = Selection::new([(id(5), MILITANT)]);
    assert!(!still_valid(placement, &militants, true, &conditions));

    let mut mixed = Selection::new([(id(1), RECRUIT), (id(2), RECRUIT), (id(5), MILITANT), (id(6), MILITANT)]);
    assert_eq!(mixed.groups().len(), 2);
    let mut stops = Vec::new();
    for _ in 0..4 {
        stops.push(mixed.active().map(|active| active.kind));
        mixed.cycle(true);
    }
    let order = if stops[0] == Some(RECRUIT) {
        [RECRUIT, MILITANT]
    } else {
        [MILITANT, RECRUIT]
    };
    assert_eq!(stops, [order[0], order[1], order[0], order[1]].map(Some));
    while mixed.active().map(|active| active.kind) != Some(RECRUIT) {
        mixed.cycle(true);
    }
    let shown_mixed = panel(&mixed, InterfaceState::DefaultState, &conditions);
    let look = |at| shown_mixed.get(at).map(|shown| shown.look);
    assert_eq!(look(slot(0, 1)), Some(Group), "Basic Construct");
    assert_eq!(look(slot(1, 0)), Some(ButtonLook::Common), "Attack, to all four");
    assert_eq!(look(slot(1, 1)), Some(Group), "Assist Construction");
    assert_eq!(recipients(&mixed, Build(STORAGE)), [id(1), id(2)]);
    assert!(still_valid(placement, &mixed, true, &conditions));
}

/// The Builds a Recruit offers are its Construct menus' buildings it builds, and no other (the Agent's Tunnel and
/// Laser Turret are not, nor the Cults' buildings whose slices come later, which hold their slots as `Planned`
/// entries), each in one menu: slice M5-A's three, the Gunworks, the Garrison (slice M5-N2) and the Altar (slice
/// M5-N1b) in the Basic one, the Machine Shop and the Scrap Hangar (slice M5-G2), the Temple (slice M5-N3), the
/// Cathedral (slice M5-N4a) and the Shrine (slice M5-N5) in the Advanced one; the panel's other commands are those of
/// its grid.
#[test]
fn a_recruit_builds_what_its_menus_hold() {
    let mut builds: Vec<ObjectKind> = (commands(RECRUIT).into_iter())
        .filter_map(|command| match command {
            PanelCommand::Build(kind) => Some(kind),
            _ => None,
        })
        .collect();
    builds.sort_unstable();
    let basic = [HALL, STORAGE, ARMORY, GUNWORKS, GARRISON, ALTAR];
    let advanced = [MACHINE_SHOP, SCRAP_HANGAR, TEMPLE, CATHEDRAL, SHRINE];
    let mut expected = [&basic[..], &advanced[..]].concat();
    expected.sort_unstable();
    assert_eq!(builds, expected);
    for kind in [ObjectKind::Tunnel, ObjectKind::LaserTurret, ObjectKind::ChopShop] {
        assert!(!offers(RECRUIT, PanelCommand::Build(kind)), "{kind:?}");
    }
    assert!(!offers(ObjectKind::Agent, PanelCommand::Build(STORAGE)));
    for kind in basic {
        assert_eq!(menu_of(RECRUIT, kind), Some(BuildMenu::Basic), "{kind:?}");
    }
    for kind in advanced {
        assert_eq!(menu_of(RECRUIT, kind), Some(BuildMenu::Advanced), "{kind:?}");
    }
    assert_eq!(InterfaceTemplate::of(RECRUIT), Some(InterfaceTemplate::Recruit));
}
