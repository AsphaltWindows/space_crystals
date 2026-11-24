//! The InfoPanel (`systems/controls/selection_and_hud.md > HUD Bar`, `core/entities.md > Object Type > InfoPanel`):
//! shown while at least one object is selected. Placeholder text until the panel's art and layout exist: the
//! selected object's name, its owner, its HP when it has any, and a map resource's state (`core/map.md >
//! SpaceCrystalsPatch > InfoPanel`, `> SupplyDeliveryStation > InfoPanel`).
//!
//! An object the local player selected through its record (`core/vision.md > Explored Selectability`: a GDO player's
//! SupplyDeliveryStation on Explored tiles) shows as last seen, without CurrentSupplies (`core/map.md >
//! SupplyDeliveryStation > InfoPanel > Explored`), and with a placeholder note of how long ago that was.
//!
//! The design doesn't say what the InfoPanel shows for two or more objects (the SelectionPanel shows them). Until it
//! does, a placeholder: the ActiveGroup's type, how many objects are in it and how many are selected.

use bevy::prelude::*;
use sc_sim::vision::LastKnown;
use sc_sim::{ObjectKind, PlayerId, ResourceState, Sim, TICKS_PER_SECOND};

use super::{LABEL_VH, SectionLabel, TEXT_COLOR, set_text, show_node, text};
use crate::fog::{LocalSight, Sight};
use crate::selection::{ControlStates, Selection};
use crate::sim_driver::LocalPlayer;
use crate::{ClientMode, hud::HudSection};

/// Size of the InfoPanel's title (placeholder).
const TITLE_VH: f32 = 2.2;

/// The InfoPanel's texts.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub enum InfoText {
    /// The object's name.
    Title,
    /// The lines under it.
    Body,
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
    });
}

/// What the InfoPanel shows for `selection`, as the player whose `sight` this is sees it: a title and the lines
/// under it. `None` while nothing is selected, which hides the panel.
pub fn info_text(sim: &Sim, sight: &Sight, selection: &Selection) -> Option<(String, String)> {
    let mut ids = selection.ids();
    let first = ids.next()?;
    if ids.next().is_none() {
        if let Some(object) = sim.object(first).filter(|o| sight.sees(o)) {
            let shown = Shown {
                kind: object.kind,
                owner: object.owner,
                hp: object.hp,
                resource: object.resource.as_ref(),
            };
            return Some(object_info(sim, shown, None));
        }
        let record = sight.remembered_record(first)?;
        return Some(object_info(sim, Shown::remembered(record), Some(record)));
    }
    let group = selection.active_group()?;
    let lines = format!("{} in group\n{} selected", group.instances.len(), selection.len());
    Some((group.kind().def().name.to_string(), lines))
}

/// What one object's panel is made from: the live object, or the record shown in its place.
struct Shown<'a> {
    kind: ObjectKind,
    owner: Option<PlayerId>,
    hp: Option<f32>,
    resource: Option<&'a ResourceState>,
}

impl<'a> Shown<'a> {
    fn remembered(record: &'a LastKnown) -> Self {
        Self {
            kind: record.kind,
            owner: record.owner,
            hp: record.hp,
            resource: record.resource.as_ref(),
        }
    }
}

/// `core/entities.md > Object Type > InfoPanel`: what one object's panel shows. From a `record` (an Explored area:
/// `core/map.md > SupplyDeliveryStation > InfoPanel > Explored`), the state last seen without CurrentSupplies, and
/// how long ago that was (placeholder note).
fn object_info(sim: &Sim, shown: Shown, record: Option<&LastKnown>) -> (String, String) {
    let mut lines = vec![match shown.owner {
        Some(owner) => sim
            .player(owner)
            .map_or_else(|| format!("Player {}", owner.0), |p| p.name.clone()),
        None => "Neutral".to_string(),
    }];
    if let (Some(hp), Some(max)) = (shown.hp, shown.kind.def().max_hp) {
        // Fractional HP shows rounded up, so a living unit never shows 0.
        lines.push(format!("HP {} / {}", hp.ceil(), max.ceil()));
    }
    match shown.resource {
        Some(&ResourceState::CrystalPatch { remaining }) => lines.push(format!("Space Crystals: {remaining}")),
        Some(&ResourceState::SupplyStation {
            delivery_size,
            delivery_interval,
            current_supplies,
            next_delivery_in,
        }) => {
            if record.is_none() {
                lines.push(format!("Supplies: {current_supplies}"));
            }
            lines.push(format!("Delivery size: {delivery_size}"));
            lines.push(format!("Delivery interval: {}", seconds(delivery_interval)));
            lines.push(match next_delivery_in {
                Some(ticks) => format!("Next delivery in {}", seconds(ticks)),
                None => "Next delivery: when emptied".to_string(),
            });
        }
        None => {}
    }
    if let Some(record) = record {
        lines.push(format!(
            "Last seen {} ago",
            seconds(sim.tick().saturating_sub(record.seen))
        ));
    }
    (shown.kind.def().name.to_string(), lines.join("\n"))
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
    let (title, body) = info.as_ref().map_or(("", ""), |(t, b)| (t.as_str(), b.as_str()));
    for (which, mut text, mut node) in &mut texts {
        let value = match which {
            InfoText::Title => title,
            InfoText::Body => body,
        };
        set_text(&mut text, value);
        show_node(&mut node, info.is_some());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_round_up() {
        assert_eq!(seconds(960), "60 s");
        assert_eq!(seconds(1), "1 s");
        assert_eq!(seconds(0), "0 s");
    }
}
