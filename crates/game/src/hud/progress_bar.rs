//! A progress bar, the HUD's one piece for it (placeholder look until the art exists, Q0001): a dark track along the
//! bottom of the node it is spawned in, its fill growing from the left as far as the share done ([`spawn`]), which
//! [`show`] moves on in place. The CommandPanel's Place buttons carry one while their building is constructed
//! (`super::command_panel`; design changes 0388 and 0392: "a running progress bar on the button showing the
//! construction's progress"), the progress the sim's.

use bevy::prelude::*;

/// The track's and the fill's colours (placeholders).
const TRACK_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.65);
const FILL_COLOR: Color = Color::srgb(0.45, 0.82, 0.48);
/// The track's height, and how far in from its parent's sides and bottom it lies, in percent of the surface height
/// (placeholders).
const HEIGHT_VH: f32 = 0.6;
const INSET_VH: f32 = 0.5;

/// A progress bar's fill, and the share done it shows, from 0 to 1.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub struct ProgressFill(pub f32);

/// Spawns a progress bar showing `ratio` (0 to 1) along the bottom of `parent`, over what else it shows. Returns its
/// fill, which [`show`] moves on.
pub fn spawn(commands: &mut Commands, parent: Entity, ratio: f32) -> Entity {
    let track = commands
        .spawn((
            Name::new("Progress bar"),
            Node {
                position_type: PositionType::Absolute,
                left: vh(INSET_VH),
                right: vh(INSET_VH),
                bottom: vh(INSET_VH),
                height: vh(HEIGHT_VH),
                ..default()
            },
            BackgroundColor(TRACK_COLOR),
            ChildOf(parent),
        ))
        .id();
    let mut fill = ProgressFill::default();
    let mut node = Node {
        height: percent(100.0),
        ..default()
    };
    show(&mut fill, &mut node, ratio);
    commands
        .spawn((
            Name::new("Progress bar fill"),
            fill,
            node,
            BackgroundColor(FILL_COLOR),
            ChildOf(track),
        ))
        .id()
}

/// Shows `ratio` on a bar's fill, its node as wide as that share of the track: none at 0 and all of it at 1, a ratio
/// outside them held to the nearest.
pub fn show(fill: &mut ProgressFill, node: &mut Node, ratio: f32) {
    let ratio = if ratio.is_nan() { 0.0 } else { ratio.clamp(0.0, 1.0) };
    fill.0 = ratio;
    node.width = percent(100.0 * ratio);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fill is the share done of the track, held between none and all of it.
    #[test]
    fn the_fill_is_the_share_done() {
        let mut fill = ProgressFill::default();
        let mut node = Node::default();
        for (ratio, shown) in [(0.25, 0.25), (1.0, 1.0), (1.5, 1.0), (-0.5, 0.0), (f32::NAN, 0.0)] {
            show(&mut fill, &mut node, ratio);
            assert_eq!((fill.0, node.width), (shown, percent(100.0 * shown)), "{ratio}");
        }
    }
}
