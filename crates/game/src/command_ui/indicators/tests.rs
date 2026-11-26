//! The indicators' colours (`systems/controls/hud.md > Color language`; design change 0201's gathering
//! markers).

use super::*;

#[test]
fn indicator_colours_follow_the_colour_language() {
    let p = Vec2::new(10.0, 20.0);
    assert_eq!(order_color(OrderKind::Move(Target::Point(p))), Some(MOVE_COLOR));
    assert_eq!(
        order_color(OrderKind::Move(Target::Object(ObjectId(3)))),
        Some(MOVE_COLOR)
    );
    assert_eq!(order_color(OrderKind::Attack(ObjectId(3))), Some(ATTACK_COLOR));
    assert_eq!(order_color(OrderKind::AttackGround(p)), Some(ATTACK_COLOR));
    assert_eq!(order_color(OrderKind::AttackMove(p)), Some(AGGRESSIVE_COLOR));
    assert_eq!(order_color(OrderKind::Patrol(p)), Some(AGGRESSIVE_COLOR));
    assert_eq!(order_color(OrderKind::Hold), None);
    assert_eq!(order_color(OrderKind::Stop), None);
    assert_eq!(order_color(OrderKind::Gather(ObjectId(3))), Some(GATHER_COLOR));
    assert_eq!(order_color(OrderKind::DropOff(ObjectId(3))), Some(RETURN_COLOR));
}

/// Design change 0201: each gathering marker has a colour none of the others has, the colour language's
/// (`systems/controls/hud.md > Color language`) included; their shapes are the drawing's own
/// (`gathering::draw`).
#[test]
fn the_gathering_markers_have_colours_of_their_own() {
    let mut colors: Vec<[u8; 4]> = GatheringMarker::ALL
        .map(|m| m.color().to_srgba().to_u8_array())
        .to_vec();
    colors.extend([MOVE_COLOR, ATTACK_COLOR, AGGRESSIVE_COLOR].map(|c| c.to_srgba().to_u8_array()));
    let distinct: BTreeSet<[u8; 4]> = colors.iter().copied().collect();
    assert_eq!(distinct.len(), colors.len(), "{colors:?}");
}
