//! The InfoPanel (`systems/controls/selection_and_hud.md > HUD Bar`, `core/entities.md > Object Type > InfoPanel`):
//! shown while at least one object is selected. Placeholder text until the panel's art and layout exist: the
//! selected object's name as the title, and under it what the panel shows of it ([`InfoContent`]):
//! - **a unit**: its owner and HP;
//! - **a structure** (`> Default InfoPanel for structures`, design change 0105): one of the local player's own shows
//!   its HP and, in the activity area under it, what it is doing ([`activity`]); another player's, or a neutral one,
//!   shows its owner ("Neutral") and HP only, never what it is doing. An object that has Shields (the Colonists',
//!   `systems/combat.md > Shields > Display`, slice M3-B, design change 0148: "Shields current/max" next to the HP
//!   line) shows them on the line after its HP, "Shields a / b". A type's own section adds to that, whoever looks: the
//!   Extraction Plate shows the remaining Space Crystals of the patch under it (`factions/gdo/economy.md >
//!   ExtractionPlate > InfoPanel`, design change 0099), with the line the patch's own panel has, since a covered patch
//!   can't be selected on its own (`> Selecting a covered patch`) and anyone who sees the patch may see its
//!   RemainingAmount (`core/map.md > SpaceCrystalsPatch > InfoPanel`). The plate's own section doesn't limit it to its
//!   owner, so another player's plate shows it too (the most literal reading);
//! - **a map resource**: "Neutral" and its state (`core/map.md > SpaceCrystalsPatch > InfoPanel`, `>
//!   SupplyDeliveryStation > InfoPanel`).
//!
//! An object the local player selected through its record (`core/vision.md > Explored Selectability`: a GDO player's
//! SupplyDeliveryStation on Explored tiles) shows as last seen, without CurrentSupplies, and with a placeholder note of
//! how long ago that was; but a GDO player always knows a station's real delivery schedule, so its NextDeliveryTime
//! is the live station's (`core/map.md > SupplyDeliveryStation > InfoPanel > Explored`, design change 0057). A record
//! keeps no Shields (`sc_sim::vision::LastKnown`), so it shows none.
//!
//! The design doesn't say what the InfoPanel shows for two or more objects (the SelectionPanel shows them). Until it
//! does, a placeholder: the ActiveGroup's type, how many objects are in it and how many are selected. The group of a
//! type's unfinished buildings (design change 0185: "as if it were another type") is named "(unfinished)".

pub mod activity;

use bevy::prelude::*;
use sc_sim::vision::LastKnown;
use sc_sim::{Object, ObjectKind, PlayerId, ResourceState, Sim, TICKS_PER_SECOND};

use super::{LABEL_VH, SectionLabel, TEXT_COLOR, set_text, show_node, text};
use crate::fog::{LocalSight, Sight};
use crate::selection::{ControlStates, Selection};
use crate::sim_driver::LocalPlayer;
use crate::{ClientMode, hud::HudSection};

/// Size of the InfoPanel's title (placeholder).
const TITLE_VH: f32 = 2.2;
/// The activity area's text colour, set apart from the lines above it (placeholder).
const ACTIVITY_COLOR: Color = Color::srgb(0.62, 0.86, 0.66);
/// A station's NextDeliveryTime while it isn't empty: no countdown runs (`core/map.md > SupplyDeliveryStation >
/// InfoPanel`, design change 0057).
pub const NO_DELIVERY: &str = "No delivery scheduled";

/// The InfoPanel's texts.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum InfoText {
    /// The object's name.
    Title,
    /// The lines under it.
    Body,
    /// The activity area, under the body: what an own structure is doing ([`activity`]); hidden while empty.
    Activity,
}

/// What the InfoPanel shows: a title, the lines under it, and the activity area's lines.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InfoContent {
    pub title: String,
    pub body: Vec<String>,
    /// What the local player's own structure is doing ([`activity::lines`]); empty for anything else.
    pub activity: Vec<String>,
}

/// The InfoPanel's content, under its section label.
pub(super) fn spawn_content(panel: &mut EntityCommands) {
    let hidden = || Node {
        display: Display::None,
        ..default()
    };
    panel.with_children(|panel| {
        panel.spawn((InfoText::Title, text("", TITLE_VH, TEXT_COLOR), hidden()));
        panel.spawn((InfoText::Body, text("", LABEL_VH, TEXT_COLOR), hidden()));
        panel.spawn((InfoText::Activity, text("", LABEL_VH, ACTIVITY_COLOR), hidden()));
    });
}

/// What the InfoPanel shows for `selection`, as the player whose `sight` this is sees it. `None` while nothing is
/// selected, which hides the panel.
pub fn info_text(sim: &Sim, sight: &Sight, selection: &Selection) -> Option<InfoContent> {
    let mut ids = selection.ids();
    let first = ids.next()?;
    if ids.next().is_none() {
        if let Some(object) = sim.object(first).filter(|o| sight.sees(o)) {
            return Some(object_info(sim, sight.player(), Shown::live(object), None));
        }
        let record = sight.remembered_record(first)?;
        let shown = Shown::remembered(sim, sight, record);
        return Some(object_info(sim, sight.player(), shown, Some(record)));
    }
    let group = selection.active_group()?;
    let name = group.kind().def().name;
    Some(InfoContent {
        title: match group.is_unfinished() {
            true => format!("{name} (unfinished)"),
            false => name.to_string(),
        },
        body: vec![
            format!("{} in group", group.instances.len()),
            format!("{} selected", selection.len()),
        ],
        activity: Vec::new(),
    })
}

/// What one object's panel is made from: the live object, or the record shown in its place.
struct Shown<'a> {
    kind: ObjectKind,
    owner: Option<PlayerId>,
    hp: Option<f32>,
    /// Its MaxHP: its kind's, or while it transforms the interpolated one (`Object::max_hp`).
    max_hp: Option<f32>,
    /// Its Shields and MaxShields, the latter interpolated while it transforms (`Object::max_shields`).
    shields: Option<f32>,
    max_shields: Option<f32>,
    resource: Option<ResourceState>,
    /// The live object, when the panel shows it as it is now.
    live: Option<&'a Object>,
}

impl<'a> Shown<'a> {
    fn live(object: &'a Object) -> Self {
        Self {
            kind: object.kind,
            owner: object.owner,
            hp: object.hp,
            max_hp: object.max_hp(),
            shields: object.shields,
            max_shields: object.max_shields(),
            resource: object.resource.clone(),
            live: Some(object),
        }
    }

    /// `record` as the player whose `sight` this is knows it: as last seen, except a station's NextDeliveryTime for a
    /// GDO player, who always knows the real delivery schedule, whatever it sees (`core/map.md >
    /// SupplyDeliveryStation > InfoPanel > Explored`, design change 0057): the live station's, not frozen at the
    /// last sighting and not extrapolated from it. So the countdown of a station emptied in the fog starts on the
    /// panel when it starts (intended, design change 0058). Anyone else sees the last-known value (no one else can
    /// select a remembered station, `core/vision.md > Explored Selectability`). A station gone from the game (only a
    /// debug removal takes one away) keeps its last-known schedule: a placeholder, the design has no such case.
    fn remembered(sim: &Sim, sight: &Sight, record: &LastKnown) -> Self {
        let mut resource = record.resource.clone();
        if let Some(ResourceState::SupplyStation { next_delivery_in, .. }) = &mut resource
            && sight.is_gdo()
            && let Some(ResourceState::SupplyStation {
                next_delivery_in: live, ..
            }) = sim.object(record.id).and_then(|o| o.resource.as_ref())
        {
            *next_delivery_in = *live;
        }
        Self {
            kind: record.kind,
            owner: record.owner,
            hp: record.hp,
            max_hp: record.kind.def().max_hp,
            shields: None,
            max_shields: None,
            resource,
            live: None,
        }
    }
}

/// `core/entities.md > Object Type > InfoPanel`: what one object's panel shows to `viewer` (module docs). From a
/// `record` (an Explored area: `core/map.md > SupplyDeliveryStation > InfoPanel > Explored`), the state last seen
/// without CurrentSupplies, and how long ago that was (placeholder note). A station that isn't empty runs no
/// countdown: no delivery is scheduled (`> SupplyDeliveryStation`, the doc's wording).
fn object_info(sim: &Sim, viewer: PlayerId, shown: Shown, record: Option<&LastKnown>) -> InfoContent {
    let def = shown.kind.def();
    // `> Default InfoPanel for structures` (design change 0105): an own structure shows no owner.
    let own_structure = def.structure.is_some() && shown.owner == Some(viewer);
    let mut body = Vec::new();
    if !own_structure {
        body.push(match shown.owner {
            Some(owner) => sim
                .player(owner)
                .map_or_else(|| format!("Player {}", owner.0), |p| p.name.clone()),
            None => "Neutral".to_string(),
        });
    }
    if let (Some(hp), Some(max)) = (shown.hp, shown.max_hp) {
        // Fractional HP shows rounded up, so a living unit never shows 0.
        body.push(format!("HP {} / {}", hp.ceil(), max.ceil()));
    }
    if let (Some(shields), Some(max)) = (shown.shields, shown.max_shields) {
        // Rounded up as the HP are, so that Shields as full as their maximum show as full.
        body.push(format!("Shields {} / {}", shields.ceil(), max.ceil()));
    }
    match shown.resource {
        Some(ResourceState::CrystalPatch { remaining }) => body.push(format!("Space Crystals: {remaining}")),
        Some(ResourceState::SupplyStation {
            delivery_size,
            delivery_interval,
            current_supplies,
            next_delivery_in,
        }) => {
            if record.is_none() {
                body.push(format!("Supplies: {current_supplies}"));
            }
            body.push(format!("Delivery size: {delivery_size}"));
            body.push(format!("Delivery interval: {}", seconds(delivery_interval)));
            body.push(match next_delivery_in {
                Some(ticks) => format!("Next delivery in {}", seconds(ticks)),
                None => NO_DELIVERY.to_string(),
            });
        }
        None => {}
    }
    // `factions/gdo/economy.md > ExtractionPlate > InfoPanel`: the patch's line, as its own panel has it.
    if let Some(remaining) = shown.live.and_then(|plate| crystals_under(sim, plate)) {
        body.push(format!("Space Crystals: {remaining}"));
    }
    if let Some(record) = record {
        body.push(format!(
            "Last seen {} ago",
            seconds(sim.tick().saturating_sub(record.seen))
        ));
    }
    let activity = match shown.live {
        Some(structure) if own_structure => activity::lines(sim, structure),
        _ => Vec::new(),
    };
    InfoContent {
        title: def.name.to_string(),
        body,
        activity,
    }
}

/// The Space Crystals left in the patch under `plate`, a structure that mines (`factions/gdo/economy.md >
/// ExtractionPlate > InfoPanel`: "displays remaining Space Crystals in the underlying patch"): in the patch it mines
/// (`Object::extractor`), else in the patch its footprint covers, for a plate that stands on one but mines nothing
/// (one without an owner has no bank to pay). `None` for any other object, and for a plate on bare ground.
fn crystals_under(sim: &Sim, plate: &Object) -> Option<u32> {
    plate.def().structure.as_ref()?.mining?;
    let remaining = |o: &Object| match o.resource {
        Some(ResourceState::CrystalPatch { remaining }) => Some(remaining),
        _ => None,
    };
    match plate.extractor {
        Some(extractor) => sim.object(extractor.patch).and_then(remaining),
        None => {
            let footprint = plate.placement?;
            sim.objects()
                .filter(|o| o.placement.is_some_and(|p| p.overlaps(&footprint)))
                .find_map(remaining)
        }
    }
}

/// A number of ticks as whole seconds, rounded up (placeholder format).
fn seconds(ticks: u32) -> String {
    format!("{} s", ticks.div_ceil(TICKS_PER_SECOND))
}

/// Shows the local player's selection in the InfoPanel, or the section's name while nothing is selected.
pub(super) fn update_info_panel(
    sight: LocalSight,
    mode: Res<ClientMode>,
    player: Res<LocalPlayer>,
    states: Res<ControlStates>,
    mut texts: Query<(&InfoText, &mut Text, &mut Node)>,
    mut labels: Query<(&SectionLabel, &mut Node), Without<InfoText>>,
) {
    if !sight.is_changed() && !states.is_changed() {
        return;
    }
    let info = mode
        .is_game()
        .then(|| info_text(sight.sim(), &sight.get(), states.selection(player.0)))
        .flatten();
    for (label, mut node) in &mut labels {
        if label.0 == HudSection::InfoPanel {
            show_node(&mut node, info.is_none());
        }
    }
    for (which, mut text, mut node) in &mut texts {
        let value = match (&info, which) {
            (None, _) => None,
            (Some(info), InfoText::Title) => Some(info.title.clone()),
            (Some(info), InfoText::Body) => Some(info.body.join("\n")),
            (Some(info), InfoText::Activity) => Some(info.activity.join("\n")).filter(|lines| !lines.is_empty()),
        };
        set_text(&mut text, value.as_deref().unwrap_or(""));
        show_node(&mut node, value.is_some());
    }
}

#[cfg(test)]
mod tests {
    use sc_sim::testing::ScenarioBuilder;
    use sc_sim::{
        Command, ConstructionCommand, DebugCommand, ObjectId, PlayerCommand, SimConfig, SpawnAt, TickInputs, TilePos,
    };

    use super::*;

    const P1: PlayerId = PlayerId(1);
    const P2: PlayerId = PlayerId(2);

    #[test]
    fn seconds_round_up() {
        assert_eq!(seconds(960), "60 s");
        assert_eq!(seconds(1), "1 s");
        assert_eq!(seconds(0), "0 s");
    }

    fn tile(x: i32, y: i32) -> SpawnAt {
        SpawnAt::Tile(TilePos::new(x, y))
    }

    /// Ids 1–3 patches; player 1's Barracks 4 and plate 5 (on patch 1); player 2's Generator 6 and plate 7 (on patch
    /// 2); a neutral plate 8 on patch 3; a plate of player 1's on bare ground, 9, which only a debug spawn puts there
    /// (map validation refuses it), in tick 1. All in plain sight of both players: the panel is about who owns what,
    /// the fog's part is tested with the client.
    fn sim() -> Sim {
        use ObjectKind::{Barracks, CrystalPatch, ExtractionPlate, Generator};
        let scenario = ScenarioBuilder::new(&vec![".".repeat(16); 8].join("\n"))
            .config(SimConfig::DEBUG)
            .spawn(CrystalPatch, None, tile(1, 5))
            .spawn(CrystalPatch, None, tile(3, 5))
            .spawn(CrystalPatch, None, tile(5, 5))
            .spawn(Barracks, Some(P1), tile(1, 1))
            .spawn(ExtractionPlate, Some(P1), tile(1, 5))
            .spawn(Generator, Some(P2), tile(6, 1))
            .spawn(ExtractionPlate, Some(P2), tile(3, 5))
            .spawn(ExtractionPlate, None, tile(5, 5))
            .build();
        let mut sim = Sim::new(&scenario);
        let spawn = DebugCommand::Spawn {
            kind: ExtractionPlate,
            owner: Some(P1),
            at: tile(9, 5),
            orientation: Default::default(),
        };
        let command = PlayerCommand {
            player: P1,
            command: Command::Debug(spawn),
        };
        sim.step(&TickInputs {
            commands: vec![command],
        });
        assert_eq!(sim.object(ObjectId(9)).map(|o| o.kind), Some(ExtractionPlate));
        sim
    }

    /// The panel of object `id` alone, as `viewer` sees it without the fog.
    fn panel(sim: &Sim, viewer: PlayerId, id: u32) -> InfoContent {
        let kind = sim.object(ObjectId(id)).unwrap().kind;
        let selection = Selection::new([(ObjectId(id), kind)]);
        info_text(sim, &Sight::new(sim, viewer, false), &selection).unwrap()
    }

    fn content(title: &str, body: &[&str]) -> InfoContent {
        InfoContent {
            title: title.into(),
            body: body.iter().map(|line| line.to_string()).collect(),
            activity: Vec::new(),
        }
    }

    /// Design change 0105: an own structure shows its name, HP and activity (none here: the Barracks is idle and has
    /// no rally point; a busy one's is `activity`'s); another player's its name, owner and HP only, whichever player
    /// looks.
    #[test]
    fn structures_show_their_owner_only_to_other_players() {
        let sim = sim();
        assert_eq!(panel(&sim, P1, 4), content("Barracks", &["HP 300 / 300"]));
        assert_eq!(panel(&sim, P2, 4), content("Barracks", &["Player 1", "HP 300 / 300"]));
        assert_eq!(panel(&sim, P2, 6), content("Generator", &["HP 350 / 350"]));
        assert_eq!(panel(&sim, P1, 6), content("Generator", &["Player 2", "HP 350 / 350"]));
    }

    /// Design change 0099 (`factions/gdo/economy.md > ExtractionPlate > InfoPanel`): a plate shows the remaining
    /// crystals of the patch under it, to its owner and to everyone else, a neutral plate's too; one on bare ground
    /// has none to show. A patch shows its own, as before.
    #[test]
    fn a_plate_shows_the_crystals_left_in_its_patch() {
        let sim = sim();
        let plate = |owner: &[&str]| {
            let mut body: Vec<&str> = owner.to_vec();
            body.extend(["HP 85 / 85", "Space Crystals: 1500"]);
            content("Extraction Plate", &body)
        };
        assert_eq!(panel(&sim, P1, 5), plate(&[]));
        assert_eq!(panel(&sim, P2, 5), plate(&["Player 1"]));
        assert_eq!(panel(&sim, P1, 7), plate(&["Player 2"]));
        assert_eq!(panel(&sim, P1, 8), plate(&["Neutral"]), "a neutral plate mines nothing");
        assert_eq!(panel(&sim, P1, 9), content("Extraction Plate", &["HP 85 / 85"]));
        assert_eq!(
            panel(&sim, P1, 1),
            content("Space Crystals", &["Neutral", "Space Crystals: 1500"])
        );
    }

    /// Slice M3-E: an object being warped in shows its new type, and its HP out of its MaxHP now, interpolated
    /// (`factions/colonists/economy.md > Warp In > HP/Shields Interpolation`): player 1's Beacon (1) warping in a Robo
    /// Portal, 288 of its 640 frames done, has 120 + (600 - 120) × 0.45 = 336 of 336, and Shields 10 + (20 - 10) ×
    /// 0.45 = 14.5 of 14.5, shown rounded up (slice M3-B). Its owner sees the Warp In in the activity area
    /// ([`activity`]); player 2 sees its owner instead.
    #[test]
    fn a_warp_in_shows_its_new_type_and_its_maximum_so_far() {
        let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
            .spawn(ObjectKind::Beacon, Some(P1), tile(8, 6))
            .start();
        let funds = DebugCommand::SetResources {
            player: P1,
            crystals: 1000,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        };
        let warp_in = ConstructionCommand::WarpIn {
            kind: ObjectKind::RoboPortal,
            orientation: Default::default(),
        };
        t.debug_at(1, funds);
        t.at(
            1,
            P1,
            Command::Construction {
                objects: vec![ObjectId(1)],
                command: warp_in,
            },
        );
        t.run_until(288);
        let mut own = content("Robo Portal", &["HP 336 / 336", "Shields 15 / 15"]);
        own.activity = vec!["Warping in: Robo Portal 45%".to_string()];
        assert_eq!(panel(t.sim(), P1, 1), own);
        assert_eq!(
            panel(t.sim(), P2, 1),
            content("Robo Portal", &["Player 1", "HP 336 / 336", "Shields 15 / 15"])
        );
    }

    /// `systems/combat.md > Shields > Display` (design change 0148): "Shields current/max" next to the HP line, for an
    /// object that has Shields, rounded up as the HP are; one without (a Peacekeeper) shows none. Player 1's Sabre
    /// Legionnaire (1), its Shields at 2.25 of 10 (a saved state edited), and player 2's Peacekeeper (2).
    #[test]
    fn an_object_with_shields_shows_them_under_its_hp() {
        let t = ScenarioBuilder::new(&vec![".".repeat(16); 8].join("\n"))
            .spawn(
                ObjectKind::SabreLegionnaire,
                Some(P1),
                SpawnAt::Point(Vec2::new(160.0, 160.0)),
            )
            .spawn(
                ObjectKind::Peacekeeper,
                Some(P2),
                SpawnAt::Point(Vec2::new(544.0, 160.0)),
            )
            .start();
        let mut state = serde_json::to_value(t.sim()).unwrap();
        let objects = state["objects"].as_array_mut().unwrap();
        let sabre = objects.iter_mut().find(|o| o["id"] == 1).unwrap();
        sabre["shields"] = serde_json::json!(2.25);
        let sim: Sim = serde_json::from_value(state).unwrap();
        assert_eq!(panel(&sim, P1, 1).body, ["Player 1", "HP 70 / 70", "Shields 3 / 10"]);
        assert_eq!(panel(&sim, P1, 2).body, ["Player 2", "HP 50 / 50"]);
    }
}
