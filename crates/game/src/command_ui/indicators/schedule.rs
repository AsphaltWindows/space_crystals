//! The scheduled stations of the selected Supply Towers (`factions/gdo/economy.md > SupplyTower > Scheduled
//! Deliveries`): **Placeholder (not in the design: `systems/controls/hud.md > CommandIndicators > Indicator
//! assignments` has no row for a schedule)**, shown as a rally point is ([`super::rally`]): the Gather marker on the
//! station, where the Tower's chopper picks up (`GatheringMarker::Gather`, the marker of an Agent's Gather there), with
//! a line from the Tower's footprint to it, in the marker's colour. The station as the player knows it
//! ([`super::targets`]): where it is drawn while they see it, where its record is while they remember it; nothing for
//! one hidden from them.

use sc_sim::Sim;

use super::{GatheringMarker, Indicator, RallyIndicator, own_selected};
use crate::fog::Sight;
use crate::selection::ControlStates;

/// The scheduled stations of the selected Supply Towers of the player whose `sight` this is, in the Towers' id order:
/// one for each Tower that has a schedule (`sc_sim::Object::schedule`) on a station the player knows, drawn as a rally
/// point is, the marker with its line from the Tower ([`RallyIndicator`]).
pub fn schedule_indicators(sim: &Sim, sight: &Sight, states: &ControlStates) -> Vec<RallyIndicator> {
    own_selected(sim, states, sight.player())
        .filter_map(|tower| {
            let station = tower.schedule?;
            let marker = Indicator::Gathering {
                id: station,
                marker: GatheringMarker::Gather,
            };
            (sight.target_view(sim, station).is_known()).then_some(RallyIndicator {
                producer: tower.id,
                marker,
            })
        })
        .collect()
}
