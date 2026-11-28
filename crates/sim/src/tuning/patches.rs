//! (M6-SB) The patches on the ground (`crate::patches`, plan decision T93): the Shadebringer's trails. A trail's own
//! numbers, its width and its patches' life, its slow and its tank's, are its layer's def's (`defs::TankDef`).

tunables! {
    /// The tunables of the patches (module docs).
    Patches;

    /// Plan decision T93: how far the point a trail is laid at moves between two of its patches, space units: "drops a
    /// patch behind its motion [...] each time it has moved 24 su since the last", half the sludge's 48 su across, so
    /// that the discs overlap by half and the trail has no gap. Until then its last patch is laid again
    /// (`crate::patches::lay`). **Placeholder (not in the design: how a trail is made of patches)**.
    spacing: f32 = 24.0,
}
