//! The map ([`MapDef`]) and its versioned file form (see the `scenario` module docs).

use std::collections::BTreeMap;

use glam::Vec2;
use serde::de::{Error as _, IgnoredAny};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::settings::SpawnSettings;
use crate::defs::ObjectKind;
use crate::ids::SlotId;
use crate::map::Map;
use crate::object::Rotation;
use crate::player::Faction;
use crate::symmetry::Symmetry;
use crate::units::TilePos;

/// Version of the map file format. Bump it on any incompatible change.
///
/// - 1: the first format (M0), with one elevation per tile.
/// - 2: corner heights (`core/map.md > Elevation`) and per-object spawn settings.
/// - 3: the map alone, without players or seed (they are the game setup): map type, description, symmetry, slots
///   with Starting Layouts, spawns owned by slots, and a look per custom preset.
pub const MAP_FORMAT_VERSION: u32 = 3;

/// `core/map_editor.md > Map Types`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapType {
    /// The standard competitive type: symmetric by construction, any faction in any slot, and no player-owned
    /// objects outside the Starting Layouts.
    SkirmishDefault,
    /// No restrictions, symmetry optional. Its slot restrictions and triggers are not designed yet.
    Custom,
}

impl MapType {
    pub const ALL: [MapType; 2] = [MapType::SkirmishDefault, MapType::Custom];

    pub fn label(self) -> &'static str {
        match self {
            MapType::SkirmishDefault => "Skirmish Default",
            MapType::Custom => "Custom",
        }
    }
}

/// A map: everything a map file holds (see the `scenario` module docs).
#[derive(Clone, Debug, PartialEq)]
pub struct MapDef {
    /// Fixed when the map is created.
    pub map_type: MapType,
    pub name: String,
    pub description: String,
    pub terrain: Map,
    /// `core/map_editor.md > Symmetry`: fixed when the map is created. The editable part is edited, the rest is
    /// generated from it ([`MapDef::regenerate`]).
    pub symmetry: Symmetry,
    /// The player slots, slot `n` at index `n - 1`. On a symmetric map they come in whole sets of images: slot
    /// `k·n + 1` is edited and slots `k·n + 2 ..= k·n + n` are its images, `n` being the number of images.
    pub slots: Vec<Slot>,
    /// The pre-placed objects, created in order at tick 0 before the Starting Layouts, so their ids ascend in list
    /// order.
    pub spawns: Vec<Spawn>,
}

impl MapDef {
    /// The slot with this id.
    pub fn slot(&self, id: SlotId) -> Option<&Slot> {
        self.slots.get(id.index()?)
    }

    pub fn slot_mut(&mut self, id: SlotId) -> Option<&mut Slot> {
        self.slots.get_mut(id.index()?)
    }

    /// The ids of the slots, in order.
    pub fn slot_ids(&self) -> impl Iterator<Item = SlotId> + '_ {
        (0..self.slots.len()).filter_map(SlotId::from_index)
    }
}

/// `core/map_editor.md > Map Creation`: a player slot, which is a start position.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slot {
    /// The start position.
    pub start: TilePos,
    /// `core/camera.md > Starting Position`: where the camera centers at game start, if the map sets it.
    #[serde(default)]
    pub camera_start: Option<Vec2>,
    /// `core/map_editor.md > Starting Layouts`: one per faction. They stay empty until the factions' Starting
    /// Conditions exist (M2).
    #[serde(default)]
    pub layouts: StartingLayouts,
}

impl Slot {
    /// A slot starting at `start`, without a camera start, with empty layouts.
    pub fn new(start: TilePos) -> Self {
        Self {
            start,
            camera_start: None,
            layouts: StartingLayouts::default(),
        }
    }
}

/// `core/map_editor.md > Starting Layouts`: the starting objects of each faction at one start position. At game
/// start, only the layout of the faction the slot's player picked is used. Serialized as an object with a list per
/// faction, named like [`Faction`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct StartingLayouts {
    #[serde(default)]
    gdo: Vec<LayoutObject>,
    #[serde(default)]
    syndicate: Vec<LayoutObject>,
    #[serde(default)]
    cults: Vec<LayoutObject>,
    #[serde(default)]
    colonists: Vec<LayoutObject>,
}

impl StartingLayouts {
    pub fn get(&self, faction: Faction) -> &[LayoutObject] {
        match faction {
            Faction::Gdo => &self.gdo,
            Faction::Syndicate => &self.syndicate,
            Faction::Cults => &self.cults,
            Faction::Colonists => &self.colonists,
        }
    }

    pub fn get_mut(&mut self, faction: Faction) -> &mut Vec<LayoutObject> {
        match faction {
            Faction::Gdo => &mut self.gdo,
            Faction::Syndicate => &mut self.syndicate,
            Faction::Cults => &mut self.cults,
            Faction::Colonists => &mut self.colonists,
        }
    }

    /// Every faction's layout, in [`Faction::ALL`] order.
    pub fn iter(&self) -> impl Iterator<Item = (Faction, &[LayoutObject])> + '_ {
        Faction::ALL.iter().map(|&f| (f, self.get(f)))
    }

    /// Whether every layout is empty.
    pub fn is_empty(&self) -> bool {
        self.iter().all(|(_, objects)| objects.is_empty())
    }
}

/// One object of a Starting Layout: what it is, where, and how a building is turned. The player who takes the slot
/// owns it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutObject {
    pub kind: ObjectKind,
    pub at: SpawnAt,
    #[serde(default, skip_serializing_if = "BuildingOrientation::is_default")]
    pub orientation: BuildingOrientation,
}

/// `core/entities.md > Structure Type`: how a building is placed, flipped first in its own frame, then rotated
/// clockwise ([`crate::Orientation::of_building`]). Only structures have one; other objects keep the default.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildingOrientation {
    #[serde(default)]
    pub rotation: Rotation,
    #[serde(default)]
    pub flip_h: bool,
    #[serde(default)]
    pub flip_v: bool,
}

impl BuildingOrientation {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// Whether it turns a footprint on its side (90° or 270°), so that a `w × h` footprint covers `h × w` tiles.
    pub fn swaps_axes(&self) -> bool {
        matches!(self.rotation, Rotation::R90 | Rotation::R270)
    }
}

/// A pre-placed object of a map.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spawn {
    pub kind: ObjectKind,
    /// Neutral, or owned by a slot: the player who takes the slot owns it, and nobody does when no player takes it.
    /// Skirmish Default maps have no owned objects outside the Starting Layouts (`core/map_editor.md > Map Types`).
    pub owner: Option<SlotId>,
    pub at: SpawnAt,
    #[serde(default, skip_serializing_if = "BuildingOrientation::is_default")]
    pub orientation: BuildingOrientation,
    /// The object's per-object values (`core/map_editor.md > Per-object Resource Values`): required for the
    /// kinds that have them, `None` for the others ([`SpawnSettings::defaults`]).
    pub settings: Option<SpawnSettings>,
}

impl Spawn {
    /// A spawn with the default values of its kind and the default orientation, as the map editor places it.
    pub fn new(kind: ObjectKind, owner: Option<SlotId>, at: SpawnAt) -> Self {
        Self {
            kind,
            owner,
            at,
            orientation: BuildingOrientation::default(),
            settings: SpawnSettings::defaults(kind),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SpawnAt {
    /// Tile-based objects: the top-left tile of the footprint as placed (turned by its orientation).
    Tile(TilePos),
    /// Units: center position in space units.
    Point(Vec2),
}

/// The serialized form of [`MapDef`], borrowed for writing.
#[derive(Serialize)]
struct MapFileRef<'a> {
    version: u32,
    map_type: MapType,
    name: &'a str,
    description: &'a str,
    symmetry: Symmetry,
    terrain: &'a Map,
    slots: &'a [Slot],
    spawns: &'a [Spawn],
}

/// The serialized form of [`MapDef`], owned for reading. Every field but the version is optional here, and unknown
/// fields are collected, so that a file of another version gets a version error rather than a field error.
#[derive(Deserialize)]
struct MapFile {
    version: u32,
    map_type: Option<MapType>,
    name: Option<String>,
    #[serde(default)]
    description: String,
    symmetry: Option<Symmetry>,
    terrain: Option<Map>,
    slots: Option<Vec<Slot>>,
    spawns: Option<Vec<Spawn>>,
    #[serde(flatten)]
    unknown: BTreeMap<String, IgnoredAny>,
}

impl Serialize for MapDef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let MapDef {
            map_type,
            name,
            description,
            terrain,
            symmetry,
            slots,
            spawns,
        } = self;
        MapFileRef {
            version: MAP_FORMAT_VERSION,
            map_type: *map_type,
            name,
            description,
            symmetry: *symmetry,
            terrain,
            slots,
            spawns,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MapDef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let file = MapFile::deserialize(deserializer)?;
        if file.version != MAP_FORMAT_VERSION {
            return Err(D::Error::custom(format!(
                "unsupported map format version {} (this build reads version {MAP_FORMAT_VERSION})",
                file.version
            )));
        }
        if let Some(field) = file.unknown.keys().next() {
            return Err(D::Error::custom(format!("unknown field `{field}` in a map file")));
        }
        fn required<T, E: serde::de::Error>(value: Option<T>, field: &'static str) -> Result<T, E> {
            value.ok_or_else(|| E::missing_field(field))
        }
        Ok(MapDef {
            map_type: required(file.map_type, "map_type")?,
            name: required(file.name, "name")?,
            description: file.description,
            terrain: required(file.terrain, "terrain")?,
            symmetry: required(file.symmetry, "symmetry")?,
            slots: required(file.slots, "slots")?,
            spawns: required(file.spawns, "spawns")?,
        })
    }
}
