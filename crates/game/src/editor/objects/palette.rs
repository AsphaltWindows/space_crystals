//! The editor's object palette: every object type, grouped by faction and category.

use sc_sim::{Category, Faction, ObjectKind};

/// A heading of the object palette and its object types.
#[derive(Clone, Debug, PartialEq)]
pub struct PaletteGroup {
    pub title: String,
    pub kinds: Vec<ObjectKind>,
}

/// The object palette: every type of `ObjectKind::ALL`, grouped by faction (neutral first, then
/// `Faction::ALL`) and category, so new types appear without editor changes.
pub fn palette_groups() -> Vec<PaletteGroup> {
    let factions = std::iter::once(None).chain(Faction::ALL.iter().copied().map(Some));
    let mut groups = Vec::new();
    for faction in factions {
        for category in [Category::Resource, Category::Structure, Category::Unit, Category::Mine] {
            let kinds: Vec<ObjectKind> = ObjectKind::ALL
                .iter()
                .copied()
                .filter(|k| k.def().faction == faction && k.def().category == category)
                .collect();
            if !kinds.is_empty() {
                groups.push(PaletteGroup {
                    title: group_title(faction, category),
                    kinds,
                });
            }
        }
    }
    groups
}

fn group_title(faction: Option<Faction>, category: Category) -> String {
    let category = match category {
        Category::Resource => "Resources",
        Category::Structure => "Structures",
        Category::Unit => "Units",
        Category::Mine => "Mines",
    };
    match faction {
        None => category.to_string(),
        Some(f) => format!("{} {}", faction_name(f), category.to_lowercase()),
    }
}

/// A faction's display name.
pub fn faction_name(faction: Faction) -> &'static str {
    match faction {
        Faction::Gdo => "GDO",
        Faction::Syndicate => "Syndicate",
        Faction::Cults => "Cults",
        Faction::Colonists => "Colonists",
    }
}
