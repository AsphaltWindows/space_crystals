use crate::types::Owner;

/// Check if two owners are enemies
pub fn is_enemy(owner1: &Owner, owner2: &Owner) -> bool {
    match (owner1.0, owner2.0) {
        (Some(p1), Some(p2)) => p1 != p2,
        _ => false,
    }
}
