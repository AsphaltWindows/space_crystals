//! The parts of a tick, for measurements ([`Sim::step_observed`](super::Sim::step_observed)).

/// The parts of a tick, in the order [`Sim::step`](super::Sim::step) runs them (`ARCHITECTURE.md > Tick pipeline`):
/// [`Sim::step_observed`](super::Sim::step_observed) marks each as it starts, then [`Phase::End`] when the tick is
/// done, so the time from one mark to the next is that phase's. Only measurements look at them (slice M2-J2,
/// `tests/structures_perf.rs`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Phase {
    /// The tick's bookkeeping: the events cleared, the positions kept for interpolation.
    Start,
    /// The defeat check's first walk: who owns structures as the tick starts (`economy::defeat`).
    DefeatOwners,
    /// The players' commands, through the gate.
    Commands,
    /// The modes (the Riot Shield) and the behaviours (5a).
    Behaviours,
    /// The true grids follow the footprints (5b).
    NavSync,
    /// Path serving (5c).
    PathServing,
    /// Movement (5d).
    Movement,
    /// Combat (5e).
    Combat,
    /// The dead leave the game and the true grids follow their footprints (5e).
    Deaths,
    /// The economy: construction, mining, then the producers (plan decision T5).
    Economy,
    /// The defeat check's second walk, which ends the economy phase.
    Defeat,
    /// Vision (5f), the supply stations, the records of what each player sees and the rally points that follow them.
    Vision,
    /// Path knowledge: what each player has seen of the map's resources, and its planning grids.
    PathKnowledge,
    /// The tick is done.
    End,
}

impl Phase {
    /// Every phase, in order, [`Phase::End`] last.
    pub const ALL: [Phase; 14] = [
        Phase::Start,
        Phase::DefeatOwners,
        Phase::Commands,
        Phase::Behaviours,
        Phase::NavSync,
        Phase::PathServing,
        Phase::Movement,
        Phase::Combat,
        Phase::Deaths,
        Phase::Economy,
        Phase::Defeat,
        Phase::Vision,
        Phase::PathKnowledge,
        Phase::End,
    ];
}
