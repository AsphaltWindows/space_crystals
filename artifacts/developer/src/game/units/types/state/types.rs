// Shared type aliases and data components for the state module.
// Types used across commands.rs and behavior.rs belong here.
use bevy::prelude::*;

/// Component tracking what resources an Agent is currently carrying.
/// Added to Agent entities at spawn time.
#[derive(Component, Default, Clone, Debug)]
pub struct AgentCarryState {
    pub crystals: u32,
    pub supplies: u32,
}

impl AgentCarryState {
    /// Whether the agent is carrying any resources
    pub fn is_carrying(&self) -> bool {
        self.crystals > 0 || self.supplies > 0
    }

    /// Whether the agent is carrying crystals specifically
    pub fn carrying_crystals(&self) -> bool {
        self.crystals > 0
    }

    /// Whether the agent is carrying supplies specifically
    pub fn carrying_supplies(&self) -> bool {
        self.supplies > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_carry_state_default_not_carrying() {
        let state = AgentCarryState::default();
        assert!(!state.is_carrying());
        assert!(!state.carrying_crystals());
        assert!(!state.carrying_supplies());
    }

    #[test]
    fn agent_carry_state_with_crystals() {
        let state = AgentCarryState { crystals: 10, supplies: 0 };
        assert!(state.is_carrying());
        assert!(state.carrying_crystals());
        assert!(!state.carrying_supplies());
    }

    #[test]
    fn agent_carry_state_with_supplies() {
        let state = AgentCarryState { crystals: 0, supplies: 5 };
        assert!(state.is_carrying());
        assert!(!state.carrying_crystals());
        assert!(state.carrying_supplies());
    }

    #[test]
    fn agent_carry_state_with_both() {
        let state = AgentCarryState { crystals: 3, supplies: 7 };
        assert!(state.is_carrying());
        assert!(state.carrying_crystals());
        assert!(state.carrying_supplies());
    }

    #[test]
    fn agent_carry_state_empty_after_clear() {
        let mut state = AgentCarryState { crystals: 10, supplies: 5 };
        state.crystals = 0;
        state.supplies = 0;
        assert!(!state.is_carrying());
    }
}
