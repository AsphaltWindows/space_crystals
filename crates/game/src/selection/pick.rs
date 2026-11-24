//! What the local player can select and where it is drawn (`ARCHITECTURE.md > Client architecture > Visuals`):
//! picking by click and by box (`systems/controls/selection_and_hud.md > MapClickSelection`, `> BoxSelection`) in
//! view space, against each object's box as it is drawn this frame, lifted by its terrain height. The live objects
//! the player sees count, and on Explored tiles the records shown in their place (`fog::Sight::remembered`).

use bevy::math::{Rect, Vec2};
use sc_sim::vision::LastKnown;
use sc_sim::{Category, Map, Object, ObjectId, ObjectKind, Placement, PlayerId, SU_PER_GU, Sim, VisibilityState};

use crate::coords::drawn_at_height;
use crate::fog::Sight;
use crate::object_render::object_height;

/// A click this close to an object's drawn box (space units) still picks it. Placeholder, unspecified: a few
/// pixels, so small units stay easy to click.
pub const PICK_SLOP_SU: f32 = 4.0;

/// How an object's owner relates to the local player.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Relation {
    Own,
    /// Another player on the local player's team. Never produced yet: there are no teams (fixed teams are
    /// deferred, `core/map_editor.md > Map Types > Custom`), so every other player is an enemy.
    Ally,
    Enemy,
    /// No owner: map resources (`core/map.md > Resource`) and neutral objects.
    Neutral,
}

impl Relation {
    /// The relation of an object owned by `owner` to `player`.
    pub fn of(player: PlayerId, owner: Option<PlayerId>) -> Self {
        match owner {
            None => Relation::Neutral,
            Some(p) if p == player => Relation::Own,
            Some(_) => Relation::Enemy,
        }
    }
}

/// Whether the player whose `sight` this is may select the live `object` now: its type must be Selectable, and the
/// object the player's own or on a tile it sees (Visible; an object counts as in the best state among the tiles it
/// covers). With the fog off, everything counts as Visible. Objects in Explored areas are shown, and selected, by
/// their records ([`remembered_selectable_by`]); nothing in Unexplored ones is.
pub fn selectable_by(object: &Object, sight: &Sight) -> bool {
    object.def().selectable
        && (object.owner == Some(sight.player()) || sight.object(object) == VisibilityState::Visible)
}

/// `core/vision.md > Explored Selectability`: whether the player whose `sight` this is may select `record`, one of
/// the records its Explored tiles show (`fog::Sight::remembered`). Objects in Explored areas are not selectable,
/// except a SupplyDeliveryStation for GDO players.
pub fn remembered_selectable_by(record: &LastKnown, sight: &Sight) -> bool {
    record.def().selectable && record.kind == ObjectKind::SupplyDeliveryStation && sight.is_gdo()
}

/// Whether the player whose `sight` this is may select object `id` now, live ([`selectable_by`]) or by the record
/// shown in its place ([`remembered_selectable_by`]).
pub fn selectable_id(sim: &Sim, sight: &Sight, id: ObjectId) -> bool {
    sim.object(id).is_some_and(|o| selectable_by(o, sight))
        || sight
            .remembered_record(id)
            .is_some_and(|r| remembered_selectable_by(r, sight))
}

/// An object the local player can select, as it is drawn this frame.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Selectable {
    pub id: ObjectId,
    pub kind: ObjectKind,
    pub relation: Relation,
    /// Where its anchor, its ground position, is drawn (view space).
    pub drawn: Vec2,
    /// The box it is drawn in (view space): a unit's silhouette or a tile-based object's footprint, lifted by the
    /// height it stands at.
    pub rect: Rect,
    /// Its ground y, which orders the drawing: southern objects are drawn in front.
    pub depth: f32,
}

impl Selectable {
    /// The selectable view of `object`, drawn at `pos` (its interpolated position).
    pub fn of(object: &Object, player: PlayerId, map: &Map, pos: Vec2) -> Self {
        let relation = Relation::of(player, object.owner);
        if let Some(p) = &object.placement {
            return Self::placed(object.id, object.kind, relation, p, map);
        }
        let half = object.def().half_extents().unwrap_or(Vec2::splat(SU_PER_GU / 4.0));
        let drawn = drawn_at_height(pos, object_height(map, pos, None));
        Self {
            id: object.id,
            kind: object.kind,
            relation,
            drawn,
            rect: Rect::from_center_half_size(drawn, half),
            depth: pos.y,
        }
    }

    /// The selectable view of `record`, drawn where the record has its footprint (`object_render::remembered`).
    pub fn remembered(record: &LastKnown, player: PlayerId, map: &Map) -> Self {
        let relation = Relation::of(player, record.owner);
        Self::placed(record.id, record.kind, relation, &record.placement, map)
    }

    /// A tile-based object, drawn over its footprint at the footprint's height.
    fn placed(id: ObjectId, kind: ObjectKind, relation: Relation, placement: &Placement, map: &Map) -> Self {
        let center = placement.center();
        let half = Vec2::new(placement.w as f32, placement.h as f32) * SU_PER_GU / 2.0;
        let drawn = drawn_at_height(center, object_height(map, center, Some(placement)));
        Self {
            id,
            kind,
            relation,
            drawn,
            rect: Rect::from_center_half_size(drawn, half),
            depth: center.y,
        }
    }

    pub fn category(&self) -> Category {
        self.kind.def().category
    }

    pub fn is_own(&self) -> bool {
        self.relation == Relation::Own
    }

    /// `> MapClickSelection`: the modified clicks (toggle, same type) apply to own units.
    pub fn is_own_unit(&self) -> bool {
        self.is_own() && self.category() == Category::Unit
    }
}

/// `> BoxSelection > Priority tiers`, highest first.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BoxTier {
    OwnUnits,
    OwnBuildings,
    EnemyUnits,
    EnemyBuildings,
    Neutral,
}

impl BoxTier {
    /// The tier of an object. Allies have no tier of their own in the design; they count as another player's, like
    /// enemies (there are none yet).
    pub fn of(object: &Selectable) -> Self {
        let unit = object.category() == Category::Unit;
        match (object.relation, unit) {
            (Relation::Neutral, _) => BoxTier::Neutral,
            (Relation::Own, true) => BoxTier::OwnUnits,
            (Relation::Own, false) => BoxTier::OwnBuildings,
            (Relation::Ally | Relation::Enemy, true) => BoxTier::EnemyUnits,
            (Relation::Ally | Relation::Enemy, false) => BoxTier::EnemyBuildings,
        }
    }
}

/// The objects the local player can select this frame, in id order.
#[derive(Clone, Debug, Default)]
pub struct Selectables {
    objects: Vec<Selectable>,
}

impl Selectables {
    /// Every object of `sim` that the player whose `sight` this is may select ([`selectable_by`]), drawn where the
    /// frame draws it: at `prev_pos.lerp(pos, alpha)` (`ARCHITECTURE.md > Client architecture > Visuals >
    /// Interpolation`); and the records shown in place of objects that it may select
    /// ([`remembered_selectable_by`]), where they show them.
    pub fn new(sim: &Sim, sight: &Sight, alpha: f32) -> Self {
        let player = sight.player();
        let live = sim
            .objects()
            .filter(|o| selectable_by(o, sight))
            .map(|o| Selectable::of(o, player, sim.map(), o.prev_pos.lerp(o.pos, alpha)));
        let remembered = sight
            .remembered()
            .filter(|r| remembered_selectable_by(r, sight))
            .map(|r| Selectable::remembered(r, player, sim.map()));
        let mut objects: Vec<Selectable> = live.chain(remembered).collect();
        // A record shows only while its object isn't seen, so an id comes once; were it twice, the live one counts.
        objects.sort_by_key(|o| o.id);
        objects.dedup_by_key(|o| o.id);
        Self { objects }
    }

    /// A set of objects as given (tests), sorted by id.
    pub fn from_objects(mut objects: Vec<Selectable>) -> Self {
        objects.sort_by_key(|o| o.id);
        Self { objects }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Selectable> + '_ {
        self.objects.iter()
    }

    pub fn get(&self, id: ObjectId) -> Option<&Selectable> {
        let i = self.objects.binary_search_by_key(&id, |o| o.id).ok()?;
        Some(&self.objects[i])
    }

    /// The object a click at the view point `view` picks: of the objects whose drawn box (grown by
    /// [`PICK_SLOP_SU`]) contains it, the one drawn in front (the southernmost), then anything but a map resource,
    /// then the later id. A plate and the patch it covers are drawn as one, and the click selects the plate: the
    /// covered patch can't be selected on its own (`factions/gdo/economy.md > ExtractionPlate > Selecting a covered
    /// patch`, design change 0099).
    pub fn at(&self, view: Vec2) -> Option<&Selectable> {
        let over_resources = |o: &Selectable| o.category() != Category::Resource;
        self.objects
            .iter()
            .filter(|o| o.rect.inflate(PICK_SLOP_SU).contains(view))
            .max_by(|a, b| {
                a.depth
                    .total_cmp(&b.depth)
                    .then(over_resources(a).cmp(&over_resources(b)))
                    .then(a.id.cmp(&b.id))
            })
    }

    /// `> BoxSelection`: the objects a box (view space) selects. Of the objects whose drawn box overlaps it, only
    /// the highest tier's count: all own units in it, or else the one object of the best tier closest to the box's
    /// center (by where its anchor is drawn; ties to the lower id). Nothing when the box holds nothing.
    pub fn in_box(&self, rect: Rect) -> Vec<ObjectId> {
        let inside: Vec<&Selectable> = self.objects.iter().filter(|o| overlaps(o.rect, rect)).collect();
        let Some(tier) = inside.iter().map(|o| BoxTier::of(o)).min() else {
            return Vec::new();
        };
        let in_tier = inside.into_iter().filter(|o| BoxTier::of(o) == tier);
        if tier == BoxTier::OwnUnits {
            return in_tier.map(|o| o.id).collect();
        }
        let center = rect.center();
        in_tier
            .min_by(|a, b| {
                a.drawn
                    .distance_squared(center)
                    .total_cmp(&b.drawn.distance_squared(center))
            })
            .map(|o| vec![o.id])
            .unwrap_or_default()
    }

    /// `> MapClickSelection > Ctrl-click`: the own units of `kind` currently on screen, the world view `visible`
    /// (view space). Placeholder bound, unspecified: a unit counts as on screen while any part of its drawn box is.
    pub fn own_units_on_screen(&self, kind: ObjectKind, visible: Rect) -> Vec<ObjectId> {
        self.objects
            .iter()
            .filter(|o| o.kind == kind && o.is_own_unit() && overlaps(o.rect, visible))
            .map(|o| o.id)
            .collect()
    }
}

/// Whether two rectangles share any area or edge.
fn overlaps(a: Rect, b: Rect) -> bool {
    a.min.x <= b.max.x && b.min.x <= a.max.x && a.min.y <= b.max.y && b.min.y <= a.max.y
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stand-in object of `kind` drawn at `at`, in a 24 × 24 box: buildings don't exist yet, so the tiers'
    /// structures are tested through these.
    fn object(id: u32, kind: ObjectKind, relation: Relation, at: Vec2) -> Selectable {
        Selectable {
            id: ObjectId(id),
            kind,
            relation,
            drawn: at,
            rect: Rect::from_center_half_size(at, Vec2::splat(12.0)),
            depth: at.y,
        }
    }

    #[test]
    fn relations_follow_the_owner() {
        let me = PlayerId(2);
        assert_eq!(Relation::of(me, Some(PlayerId(2))), Relation::Own);
        assert_eq!(Relation::of(me, Some(PlayerId(1))), Relation::Enemy);
        assert_eq!(Relation::of(me, None), Relation::Neutral);
    }

    /// `> BoxSelection > Priority tiers`. No structure type exists yet, so a building is any object whose category
    /// isn't Unit: a resource stands in (never owned in a game, but the tier reads only owner and category).
    #[test]
    fn a_box_selects_the_highest_tier_it_holds() {
        use sc_sim::ObjectKind as K;
        let building = K::SupplyDeliveryStation;
        assert_ne!(building.def().category, Category::Unit);
        let unit = K::Peacekeeper;
        let at = |x: f32| Vec2::new(x, 100.0);
        let all = vec![
            object(1, unit, Relation::Own, at(100.0)),
            object(2, unit, Relation::Own, at(300.0)),
            object(3, building, Relation::Own, at(190.0)),
            object(4, building, Relation::Own, at(205.0)),
            object(5, unit, Relation::Enemy, at(210.0)),
            object(6, unit, Relation::Enemy, at(180.0)),
            object(7, building, Relation::Enemy, at(200.0)),
            object(8, K::CrystalPatch, Relation::Neutral, at(206.0)),
            object(9, unit, Relation::Neutral, at(199.0)),
        ];
        let boxed = |keep: &[u32]| {
            let objects = all.iter().copied().filter(|o| keep.contains(&o.id.0)).collect();
            Selectables::from_objects(objects).in_box(Rect::new(0.0, 0.0, 400.0, 200.0))
        };
        let ids = |ids: &[u32]| ids.iter().map(|&i| ObjectId(i)).collect::<Vec<_>>();
        // Tier 1: every own unit.
        assert_eq!(boxed(&[1, 2, 3, 4, 5, 6, 7, 8, 9]), ids(&[1, 2]));
        // Tier 2: the own building closest to the center (200, 100).
        assert_eq!(boxed(&[3, 4, 5, 6, 7, 8, 9]), ids(&[4]));
        // Tier 3: the enemy unit closest to the center; then tier 4, then tier 5.
        assert_eq!(boxed(&[5, 6, 7, 8, 9]), ids(&[5]));
        assert_eq!(boxed(&[6, 7, 8, 9]), ids(&[6]));
        assert_eq!(boxed(&[7, 8, 9]), ids(&[7]));
        assert_eq!(boxed(&[8, 9]), ids(&[9]), "neutral units count as neutral objects");
        assert_eq!(boxed(&[]), ids(&[]));
        // Allies count with the enemies.
        let ally = object(10, unit, Relation::Ally, at(200.0));
        assert_eq!(BoxTier::of(&ally), BoxTier::EnemyUnits);
    }

    #[test]
    fn equally_close_candidates_go_to_the_lower_id() {
        let objects = Selectables::from_objects(vec![
            object(4, ObjectKind::Guard, Relation::Enemy, Vec2::new(90.0, 100.0)),
            object(3, ObjectKind::Guard, Relation::Enemy, Vec2::new(110.0, 100.0)),
        ]);
        assert_eq!(objects.in_box(Rect::new(0.0, 0.0, 200.0, 200.0)), [ObjectId(3)]);
    }

    #[test]
    fn a_click_picks_the_object_drawn_in_front() {
        let objects = Selectables::from_objects(vec![
            object(1, ObjectKind::Peacekeeper, Relation::Own, Vec2::new(100.0, 100.0)),
            object(2, ObjectKind::Guard, Relation::Enemy, Vec2::new(100.0, 110.0)),
        ]);
        // Both boxes contain (100, 105): the southern one is drawn in front.
        assert_eq!(objects.at(Vec2::new(100.0, 105.0)).map(|o| o.id), Some(ObjectId(2)));
        assert_eq!(objects.at(Vec2::new(100.0, 90.0)).map(|o| o.id), Some(ObjectId(1)));
        // A little outside the box still picks it; further out nothing does.
        let east_of_1 = |dx: f32| Vec2::new(100.0 + 12.0 + PICK_SLOP_SU + dx, 92.0);
        assert_eq!(objects.at(east_of_1(-0.5)).map(|o| o.id), Some(ObjectId(1)));
        assert_eq!(objects.at(east_of_1(0.5)), None);
    }

    /// Design change 0099: a click on a plate's tile selects the plate, never the patch it covers, whichever of
    /// them came first.
    #[test]
    fn a_click_on_a_covered_patch_picks_its_plate() {
        let tile = Vec2::new(96.0, 96.0);
        let patch = |id| object(id, ObjectKind::CrystalPatch, Relation::Neutral, tile);
        let plate = |id| object(id, ObjectKind::ExtractionPlate, Relation::Enemy, tile);
        for objects in [vec![patch(1), plate(2)], vec![plate(1), patch(2)]] {
            let objects = Selectables::from_objects(objects);
            assert_eq!(objects.at(tile).map(|o| o.kind), Some(ObjectKind::ExtractionPlate));
        }
    }

    #[test]
    fn on_screen_means_any_part_of_the_drawn_box_in_view() {
        let pk = ObjectKind::Peacekeeper;
        let objects = Selectables::from_objects(vec![
            object(1, pk, Relation::Own, Vec2::new(50.0, 50.0)),
            object(2, pk, Relation::Own, Vec2::new(-11.0, 50.0)),
            object(3, pk, Relation::Own, Vec2::new(-13.0, 50.0)),
            object(4, pk, Relation::Enemy, Vec2::new(60.0, 50.0)),
            object(5, ObjectKind::Guard, Relation::Own, Vec2::new(70.0, 50.0)),
        ]);
        let visible = Rect::new(0.0, 0.0, 400.0, 300.0);
        assert_eq!(objects.own_units_on_screen(pk, visible), [ObjectId(1), ObjectId(2)]);
    }
}
