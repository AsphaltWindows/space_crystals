//! (M6-GR) The Radar Station's clicks and pointers: its Scan awaits any point on the map, seen or not, where a click
//! scans, under the Cast pointer; a point off the map is an invalid target, under the Inactive pointer. The station
//! has no right-click.

use sc_sim::AreaAbility;

use super::*;

const SCAN: PanelCommand = PanelCommand::Cast(AreaAbility::Scan);

/// `factions/gdo/structures/radar_station.md > Placeholder stats`: "**Scan** (hotkey **Q**, targeted: left-click any
/// point on the map)". A click on a point the player sees scans there, and so does one on a point they don't (the
/// Shrine's casts take only the first, `super::shrine`), on the ground or over an object; the pointer shows Cast. Off
/// the map, nothing (an invalid target), under the Inactive pointer. A selected station resolves no right-click.
#[test]
fn a_scan_takes_any_point_on_the_map() {
    use PointerDisplay as D;
    let station = [UnitCapabilities::of(id(4), ObjectKind::RadarStation, &Modes::default())];
    let awaiting = InterfaceState::AwaitingTarget {
        pending: SCAN,
        queued: false,
    };
    for seen in [MapTarget::ground(P), own(), enemy()] {
        let unseen = MapTarget { seen: false, ..seen };
        for target in [seen, unseen] {
            let scan = Some(Resolved::Cast(AreaAbility::Scan, P));
            assert_eq!(target_resolution(SCAN, &target), scan, "{target:?}");
            assert_eq!(pointer_display(awaiting, &station, &target), D::Cast);
        }
        let off = MapTarget {
            seen: false,
            on_map: false,
            ..seen
        };
        assert_eq!(target_resolution(SCAN, &off), None, "{off:?}");
        assert_eq!(pointer_display(awaiting, &station, &off), D::Inactive);
    }
    assert_eq!(station[0].template, Some(InterfaceTemplate::RadarStation));
    for target in [MapTarget::ground(P), own(), enemy()] {
        assert_eq!(object_right_click(station[0], &target), None, "{target:?}");
    }
}
