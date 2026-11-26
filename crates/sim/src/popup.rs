//! The pop-up turret (plan decision T50, slice M4-S2; slice M5-Y, design changes 0358 to 0360): the Laser Turret —
//! `factions/syndicate/structures/laser_turret.md`: "it is always a hidden **pop-up turret**, and is **only active
//! inside its Tunnel's area** ... There is no upright form any more".
//!
//! - **Every turret is a pop-up**: an object of a type with a pop-up form (`StructureDef::popup`) has a [`PopUp`]
//!   state (`Object::popup`) from its first tick, lowered (`Sim::new_object`): its site too, which only a Tunnel builds
//!   (`> Construction`: "The **site is hidden** like an expansion under construction"), and so it completes lowered.
//!   The state is hashed and saved only while set, so the games without a turret are as they were.
//! - **Active only inside its Tunnel's area** ([`is_active`]): a turret in operation is active while it has a parent
//!   Tunnel (`tunnel::area::parent`, the expansions' rule: an own Tunnel in operation whose current Tunnel Area fully
//!   contains it). **A reading**: "in operation", as for reclaiming (design change 0258): an upgrading Tunnel's area is
//!   its current tier's until the upgrade completes (design change 0249). An inactive turret is **lowered at once** and
//!   stays down whatever it sees ([`step`]), so it never fires: a neutral one (`> When its Tunnel is lost`: "keeps its
//!   **high underground armor** (a raised turret retracts at once; it never fires again while neutral)",
//!   `sim/tunnels.rs`), and one that a map or a debug spawn stands outside every own Tunnel Area. **A reading** of
//!   "retracts at once": down in the tick, with no retract frames, so that its armour is the underground one from that
//!   tick (the high armour it "keeps"). A turret a Tunnel takes back is active again from the next step.
//! - **The cycle** ([`step`], once a tick, right after phase 5e: `Sim::step_popups`): [`PopUp::Lowered`] while it has
//!   no target; with one, [`PopUp::Rising`] for the def's rise frames, then [`PopUp::Raised`], where it fires; after
//!   the def's idle frames with no target, [`PopUp::Retracting`] for its retract frames, then lowered again ("rises
//!   in 8 frames (already facing its target); retracts after 32 frames with no target, taking 8 frames. It can't fire
//!   while rising or retracting"). A phase that starts in tick `t` does its first frame in `t + 1`, as a site does:
//!   a turret that picks a target in tick `t` is raised at the end of `t + 8` and aims from `t + 9`. Each phase runs
//!   its full length (design change 0358 point 2: "**a new target during retracting doesn't interrupt it**: it
//!   finishes retracting, then rises"): it rises again in the tick it is down.
//! - **Rising needs a clear footprint** (design change 0296 point 3, `> Pop-up rules`: "it can't rise while any unit
//!   stands over it"): a lowered turret that wants up while a ground unit's box overlaps its footprint stays down,
//!   keeping its target, and notes it ([`PopUp::Lowered::blocked`]); the movement step then asks its owner's idle
//!   units on the footprint to step off (`movement::yielding::make_way`: "The **owner's own idle units** over it are
//!   pushed off by the idle push"), and "any other unit over it (an enemy, or an own unit with an order) keeps it
//!   **underground** until the spot is clear".
//! - **What reads the state**, each through one function here: hiding and the underground armour only while fully
//!   lowered ([`is_lowered`], `vision::hidden`, [`Object::armor`]; design change 0296 point 1: "While rising, raised
//!   and retracting it is visible and has its raised armor"); blocking ([`blocks`]); firing ([`may_fire`]).
//! - **Blocking, a reading of 0296 point 3** (`> Pop-up rules`: "a **raised** turret blocks movement like any
//!   building. A lowered one never blocks"): the footprint blocks from the tick the turret starts to rise until it is
//!   fully lowered again, so rising and retracting count as raised. Otherwise a unit could step onto the footprint
//!   during the rise, and the raised turret would stand under it.

use serde::{Deserialize, Serialize};

use crate::defs::{Armor, ObjectKind, PopUpDef};
use crate::economy::RATIO_ONE;
use crate::hash::{StateHash, StateHasher};
use crate::ids::PlayerId;
use crate::nav::TileRect;
use crate::object::Object;
use crate::progress::Progress;
use crate::tunnel;

/// Where a turret is between underground and the surface (module docs).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PopUp {
    /// Underground: hidden from the other players, at its underground armour, and walked over.
    Lowered {
        /// It wanted to rise in its latest step and a unit stood over it: its owner's idle units there are asked to
        /// step off (module docs).
        blocked: bool,
    },
    /// On its way up, the frames done of its def's `rise_frames`: seen, blocking, at its raised armour, not firing.
    Rising(Progress),
    /// Up, and firing if it has a target; `idle` counts the ticks it has had none, of its def's `idle_frames`.
    Raised { idle: Progress },
    /// On its way down, the frames done of its def's `retract_frames`: as [`PopUp::Rising`].
    Retracting(Progress),
}

impl PopUp {
    /// Lowered, with nothing in its way noted: how every turret starts, and where an inactive one is.
    pub const LOWERED: PopUp = PopUp::Lowered { blocked: false };

    /// Just raised: no idle tick counted.
    fn raised(def: &PopUpDef) -> Self {
        PopUp::Raised {
            idle: Progress::new(def.idle_frames),
        }
    }

    /// How far up it is, from 0 (lowered) to 1 (raised): for drawing.
    pub fn height(&self) -> f32 {
        match self {
            PopUp::Lowered { .. } => 0.0,
            PopUp::Rising(progress) => progress.ratio(),
            PopUp::Raised { .. } => 1.0,
            PopUp::Retracting(progress) => 1.0 - progress.ratio(),
        }
    }
}

/// The pop-up form of a structure of `kind` (`StructureDef::popup`): `None` for a kind without one.
pub fn def_of(kind: ObjectKind) -> Option<&'static PopUpDef> {
    kind.def().structure.as_ref()?.popup.as_ref()
}

/// Whether `o` is fully lowered, underground: hidden from the other players and at its underground armour (design
/// change 0296 point 1: "**Hidden only while fully lowered**").
pub fn is_lowered(o: &Object) -> bool {
    matches!(o.popup, Some(PopUp::Lowered { .. }))
}

/// Whether a record of an object of `kind` that `owner` owned when last seen (`vision::LastKnown`) is of a turret
/// fully lowered: a neutral turret's, which is always down (module docs: an inactive turret). A record of an owned
/// turret is of one seen, so not lowered: lowered, it is hidden from the other players, and none remembers what is
/// hidden from it (`vision::memory`). What a player's planning grids (`paths::knowledge`) and the client draw of it.
pub fn remembered_lowered(kind: ObjectKind, owner: Option<PlayerId>) -> bool {
    owner.is_none() && def_of(kind).is_some()
}

/// Whether `o`'s footprint blocks movement as far as its pop-up state goes (module docs, the reading of 0296 point
/// 3): always, but while it is fully lowered.
pub fn blocks(o: &Object) -> bool {
    !is_lowered(o)
}

/// Whether `o` may run its attack sequence: an object without a pop-up state (a unit), and a turret while raised ("It
/// can't fire while rising or retracting"; lowered, it is underground).
pub fn may_fire(o: &Object) -> bool {
    matches!(o.popup, None | Some(PopUp::Raised { .. }))
}

/// Whether `o` is a lowered turret that wants to rise and has a unit over it (`PopUp::Lowered::blocked`).
pub fn is_blocked(o: &Object) -> bool {
    matches!(o.popup, Some(PopUp::Lowered { blocked: true }))
}

/// Whether `o` is an active turret (module docs): a structure in operation of a type with a pop-up form that has a
/// parent Tunnel among `tunnels` (`tunnel::area::parent`: one of its owner's Tunnels in operation whose current Tunnel
/// Area fully contains its footprint). A neutral turret never is.
pub fn is_active<'a>(o: &Object, tunnels: impl IntoIterator<Item = &'a Object>) -> bool {
    def_of(o.kind).is_some() && o.is_operational() && tunnel::area::parent(o, tunnels).is_some()
}

/// [`is_active`] for a turret in operation, against the Tunnel Areas `areas`, each with its owner: those of the
/// Tunnels in operation.
pub(crate) fn active_in(o: &Object, areas: &[(PlayerId, TileRect)]) -> bool {
    let (Some(owner), Some(p)) = (o.owner, o.placement) else {
        return false;
    };
    let footprint = TileRect::new(p.origin, p.w, p.h);
    areas
        .iter()
        .any(|&(of, area)| of == owner && tunnel::area::encloses(area, footprint))
}

impl Object {
    /// Its armour now: its type's (`ObjectDef::armor`), or its pop-up form's underground armour while it is fully
    /// lowered (`PopUpDef::lowered_armor`; `factions/syndicate/structures/laser_turret.md`: "raised PointArmor 1,
    /// FullArmor 1; underground (pop-up) PointArmor 6, FullArmor 6"). What every hit meets (`combat::targeting`).
    pub fn armor(&self) -> Armor {
        match def_of(self.kind) {
            Some(def) if is_lowered(self) => def.lowered_armor,
            _ => self.def().armor,
        }
    }
}

/// One tick of a turret's pop-up state (module docs), from `state` to the state it leaves. `active`: it is active now
/// ([`is_active`]); `target`: it engages one; `clear`: no unit stands over its footprint, asked only when it would
/// start to rise.
pub(crate) fn step(state: PopUp, def: &PopUpDef, active: bool, target: bool, clear: impl FnOnce() -> bool) -> PopUp {
    // Inactive (neutral, or outside every own Tunnel Area): down at once, and it stays down.
    if !active {
        return PopUp::LOWERED;
    }
    let rise = |clear: bool| match clear {
        true => PopUp::Rising(Progress::new(def.rise_frames)),
        false => PopUp::Lowered { blocked: true },
    };
    match state {
        PopUp::Lowered { .. } if target => rise(clear()),
        PopUp::Lowered { .. } => PopUp::LOWERED,
        PopUp::Rising(mut progress) => match progress.step(RATIO_ONE) {
            true => PopUp::raised(def),
            false => PopUp::Rising(progress),
        },
        PopUp::Raised { .. } if target => PopUp::raised(def),
        PopUp::Raised { mut idle } => match idle.step(RATIO_ONE) {
            true => PopUp::Retracting(Progress::new(def.retract_frames)),
            false => PopUp::Raised { idle },
        },
        // Design change 0358 point 2: "a new target during retracting doesn't interrupt it: it finishes retracting,
        // then rises".
        PopUp::Retracting(mut progress) => match progress.step(RATIO_ONE) {
            // Down; one with a target rises at once: nothing came onto its footprint, which blocked until now.
            true if target => rise(true),
            true => PopUp::LOWERED,
            false => PopUp::Retracting(progress),
        },
    }
}

impl StateHash for PopUp {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            PopUp::Lowered { blocked } => {
                h.tag(0);
                blocked.hash(h);
            }
            PopUp::Rising(progress) => {
                h.tag(1);
                progress.hash(h);
            }
            PopUp::Raised { idle } => {
                h.tag(2);
                idle.hash(h);
            }
            PopUp::Retracting(progress) => {
                h.tag(3);
                progress.hash(h);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def() -> &'static PopUpDef {
        def_of(ObjectKind::LaserTurret).expect("the Laser Turret has a pop-up form")
    }

    /// One tick of an active turret with a clear footprint.
    fn tick(state: PopUp, target: bool) -> PopUp {
        step(state, def(), true, target, || true)
    }

    /// `factions/syndicate/structures/laser_turret.md`: "rises in 8 frames ...; retracts after 32 frames with no
    /// target, taking 8 frames". A lowered pop-up that gets a target in tick 0 is raised at the end of tick 8; its
    /// last target in tick `t`, it starts to retract in `t + 32` and is down at the end of `t + 40`.
    #[test]
    fn a_popup_rises_in_8_frames_and_retracts_32_frames_after_its_last_target_in_8() {
        let mut state = tick(PopUp::LOWERED, true);
        assert_eq!(state, PopUp::Rising(Progress::new(8)));
        for frame in 1..=8 {
            assert!(matches!(state, PopUp::Rising(_)), "frame {frame}");
            state = tick(state, true);
        }
        assert_eq!(state, PopUp::raised(def()));
        // While it has a target nothing counts.
        for _ in 0..100 {
            state = tick(state, true);
        }
        assert_eq!(state, PopUp::raised(def()));
        for idle in 1..=32 {
            assert!(matches!(state, PopUp::Raised { .. }), "idle tick {idle}");
            state = tick(state, false);
        }
        assert_eq!(state, PopUp::Retracting(Progress::new(8)));
        for frame in 1..=8 {
            assert!(matches!(state, PopUp::Retracting(_)), "frame {frame}");
            state = tick(state, false);
        }
        assert_eq!(state, PopUp::LOWERED);
        assert_eq!(tick(state, false), PopUp::LOWERED);
    }

    /// A target seen again resets the idle count; design change 0358 point 2: one seen while the turret retracts
    /// doesn't interrupt it: it goes down, and rises again in the tick it is down, its footprint never open in between.
    #[test]
    fn a_target_resets_the_idle_count_and_a_retracting_turret_goes_down_first() {
        let mut state = PopUp::raised(def());
        for _ in 0..31 {
            state = tick(state, false);
        }
        state = tick(state, true);
        assert_eq!(state, PopUp::raised(def()));
        for _ in 0..32 {
            state = tick(state, false);
        }
        assert!(matches!(state, PopUp::Retracting(_)));
        for _ in 0..7 {
            state = tick(state, true);
            assert!(matches!(state, PopUp::Retracting(_)));
        }
        // No unit can be over it: the footprint blocked all along, so `clear` isn't asked.
        let risen = step(state, def(), true, true, || unreachable!("blocked until now"));
        assert_eq!(risen, PopUp::Rising(Progress::new(8)));
    }

    /// Design change 0296 point 3: "it can't rise while any unit stands over it": it stays lowered, noting it, and
    /// rises in the tick the footprint is clear; with no target it forgets the note.
    #[test]
    fn a_unit_over_it_keeps_it_down() {
        let blocked = step(PopUp::LOWERED, def(), true, true, || false);
        assert_eq!(blocked, PopUp::Lowered { blocked: true });
        assert_eq!(step(blocked, def(), true, true, || false), blocked);
        assert_eq!(tick(blocked, false), PopUp::LOWERED);
        assert_eq!(tick(blocked, true), PopUp::Rising(Progress::new(8)));
    }

    /// Design change 0360 (module docs): an inactive turret is down at once from every state, whatever it sees, and
    /// stays down, asking nothing of its footprint; active again, it follows its cycle from there.
    #[test]
    fn an_inactive_turret_is_down_at_once_and_stays_down() {
        let mut rising = Progress::new(8);
        rising.step(RATIO_ONE);
        let states = [
            PopUp::LOWERED,
            PopUp::Lowered { blocked: true },
            PopUp::Rising(rising),
            PopUp::raised(def()),
            PopUp::Retracting(Progress::new(8)),
        ];
        for state in states {
            for target in [false, true] {
                let after = step(state, def(), false, target, || unreachable!("it never rises"));
                assert_eq!(after, PopUp::LOWERED, "{state:?}, target {target}");
            }
        }
        assert_eq!(tick(PopUp::LOWERED, true), PopUp::Rising(Progress::new(8)), "active again");
    }

    /// Each phase hashes apart, and its progress counts.
    #[test]
    fn the_phases_hash_apart() {
        let hash = |state: PopUp| {
            let mut h = StateHasher::new();
            state.hash(&mut h);
            h.finish()
        };
        let mut started = Progress::new(8);
        started.step(RATIO_ONE);
        let states = [
            PopUp::LOWERED,
            PopUp::Lowered { blocked: true },
            PopUp::Rising(Progress::new(8)),
            PopUp::Rising(started),
            PopUp::Raised {
                idle: Progress::new(32),
            },
            PopUp::Retracting(Progress::new(8)),
        ];
        for (i, a) in states.iter().enumerate() {
            for b in &states[i + 1..] {
                assert_ne!(hash(*a), hash(*b), "{a:?} {b:?}");
            }
        }
        assert_eq!(states.map(|state| state.height()), [0.0, 0.0, 0.0, 0.125, 1.0, 1.0]);
    }
}
