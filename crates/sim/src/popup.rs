//! The pop-up form (plan decision T50, slice M4-S2): the Laser Turret inside an own Tunnel Area —
//! `factions/syndicate/structures/laser_turret.md`.
//!
//! - **The form is derived, never stored** (`> Upright vs Pop-up`, `> Conversion`: "The form updates automatically
//!   whenever Tunnel Areas change"): a structure of a type with a pop-up form (`StructureDef::popup`) is a pop-up while
//!   its footprint lies inside the current Tunnel Area of one of its owner's Tunnels in operation ([`in_area`]:
//!   `tunnel::area::parent`, an expansion's parent rule), and upright otherwise. So a new Tunnel, an upgrade and a
//!   Tunnel's loss convert it with no event: each only changes the areas. **A reading**: "operational", as for
//!   reclaiming (design change 0258): a Tunnel still a site has no pop-up, and an upgrading Tunnel's area is its
//!   current tier's until the upgrade completes (design change 0249).
//! - **What is stored** is where the turret is between underground and the surface: [`PopUp`], `Object::popup`.
//!   `None` is a turret standing upright with nothing counting, which every turret outside an area is, so the state is
//!   hashed and saved only for a pop-up (and for a turret that lost its Tunnel while down, until it is up).
//! - **The cycle** ([`step`], once a tick, right after phase 5e: `Sim::step_popups`): [`PopUp::Lowered`] while it has
//!   no target; with one, [`PopUp::Rising`] for the def's rise frames, then [`PopUp::Raised`], where it fires; after
//!   the def's idle frames with no target, [`PopUp::Retracting`] for its retract frames, then lowered again ("rises
//!   in 8 frames (already facing its target); retracts after 32 frames with no target, taking 8 frames. It can't fire
//!   while rising or retracting"). A phase that starts in tick `t` does its first frame in `t + 1`, as a site does:
//!   a turret that picks a target in tick `t` is raised at the end of `t + 8` and aims from `t + 9`. Each phase runs
//!   its full length: a turret that gets a target while it retracts goes all the way down and rises again at once
//!   (Placeholder (Q0091 point 2), in [`step`]: the simplest predictable rule, design change 0223; the design doesn't
//!   say).
//! - **Rising needs a clear footprint** (design change 0296 point 3, `> Pop-up rules`: "it can't rise while any unit
//!   stands over it"): a lowered turret that wants up while a ground unit's box overlaps its footprint stays down,
//!   keeping its target, and notes it ([`PopUp::Lowered::blocked`]); the movement step then asks its owner's idle
//!   units on the footprint to step off (`movement::yielding::make_way`: "The **owner's own idle units** over it are
//!   pushed off by the idle push"), and "any other unit over it (an enemy, or an own unit with an order) keeps it
//!   **underground** until the spot is clear".
//! - **What reads the state**, each through one function here: hiding and the underground armour only while fully
//!   lowered ([`is_lowered`], `vision::hidden`, [`Object::armor`]; design change 0296 point 1: "While rising, raised
//!   and retracting it is visible and has its raised armor"); blocking ([`blocks`]); firing ([`may_fire`]).
//! - **Blocking, a reading of 0296 point 3** ("an upright turret blocks movement like any building, and so does a
//!   raised pop-up. A lowered pop-up never blocks"): the footprint blocks from the tick the turret starts to rise until
//!   it is fully lowered again, so rising and retracting count as raised. Otherwise a unit could step onto the
//!   footprint during the rise, and the raised turret would stand under it.
//! - **A turret that loses its Tunnel while it isn't up** is upright by its form, and comes up by the same rule: it
//!   wants up whatever it sees, waits for a clear footprint as a pop-up does, hidden and armoured as lowered until it
//!   rises (Placeholder (Q0091 point 1), in [`step`]), and its state goes once it stands.
//! - **A Tunnel's turret site** is lowered from its first frame (`Sim::start_expansion`; design change 0297 point 6: "A
//!   Tunnel's site is hidden like an expansion under construction"), and so is the turret as it completes; an Agent's
//!   site stands on the surface, and its turret retracts once complete inside an area.

use serde::{Deserialize, Serialize};

use crate::defs::{Armor, ObjectKind, PopUpDef};
use crate::economy::RATIO_ONE;
use crate::hash::{StateHash, StateHasher};
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
    /// Lowered, with nothing in its way noted: how a Tunnel's turret starts.
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

/// Whether `o`'s footprint blocks movement as far as its pop-up state goes (module docs, the reading of 0296 point
/// 3): always, but while it is fully lowered.
pub fn blocks(o: &Object) -> bool {
    !is_lowered(o)
}

/// Whether `o` may run its attack sequence: an upright turret, and a pop-up while raised ("It can't fire while rising
/// or retracting"; lowered, it is underground).
pub fn may_fire(o: &Object) -> bool {
    matches!(o.popup, None | Some(PopUp::Raised { .. }))
}

/// Whether `o` is a lowered turret that wants to rise and has a unit over it (`PopUp::Lowered::blocked`).
pub fn is_blocked(o: &Object) -> bool {
    matches!(o.popup, Some(PopUp::Lowered { blocked: true }))
}

/// Whether `o` is a pop-up by its form (module docs): a structure in operation of a type with a pop-up form, whose
/// footprint lies inside the current Tunnel Area of one of its owner's Tunnels in operation, among `tunnels`.
pub fn in_area<'a>(o: &Object, tunnels: impl IntoIterator<Item = &'a Object>) -> bool {
    def_of(o.kind).is_some() && tunnel::area::parent(o, tunnels).is_some()
}

/// [`in_area`] against the Tunnel Areas `areas`, each with its owner: those of the Tunnels in operation.
pub(crate) fn in_areas(o: &Object, areas: &[(crate::ids::PlayerId, TileRect)]) -> bool {
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

/// One tick of a turret's pop-up state (module docs), from `state` to the state it leaves. `pop_up`: its form now
/// ([`in_area`]); `target`: it engages one; `clear`: no unit stands over its footprint, asked only when it would
/// start to rise.
pub(crate) fn step(
    state: Option<PopUp>,
    def: &PopUpDef,
    pop_up: bool,
    target: bool,
    clear: impl FnOnce() -> bool,
) -> Option<PopUp> {
    // An upright turret always stands; a pop-up comes up for a target.
    let wants_up = !pop_up || target;
    let rise = |clear: bool| match clear {
        true => PopUp::Rising(Progress::new(def.rise_frames)),
        false => PopUp::Lowered { blocked: true },
    };
    let state = match state {
        // Upright, and an area now covers it: it stands raised, and retracts once it has had no target for long enough.
        None if pop_up => PopUp::raised(def),
        None => return None,
        Some(state) => state,
    };
    match state {
        // Placeholder (Q0091 point 1): one whose Tunnel is gone, upright by its form, waits for a clear footprint as a
        // pop-up does, lowered: hidden, and at its underground armour.
        PopUp::Lowered { .. } if wants_up => Some(rise(clear())),
        PopUp::Lowered { .. } => Some(PopUp::LOWERED),
        PopUp::Rising(mut progress) => Some(match progress.step(RATIO_ONE) {
            true => PopUp::raised(def),
            false => PopUp::Rising(progress),
        }),
        // Its Tunnel is gone: it stands upright.
        PopUp::Raised { .. } if !pop_up => None,
        PopUp::Raised { .. } if target => Some(PopUp::raised(def)),
        PopUp::Raised { mut idle } => Some(match idle.step(RATIO_ONE) {
            true => PopUp::Retracting(Progress::new(def.retract_frames)),
            false => PopUp::Raised { idle },
        }),
        // Placeholder (Q0091 point 2): a target doesn't turn it back; it goes all the way down first.
        PopUp::Retracting(mut progress) => Some(match progress.step(RATIO_ONE) {
            // Down; one that wants up again rises at once: nothing came onto its footprint, which blocked until now.
            true if wants_up => rise(true),
            true => PopUp::LOWERED,
            false => PopUp::Retracting(progress),
        }),
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

    /// One tick of a pop-up (`pop_up` true) with a clear footprint.
    fn tick(state: Option<PopUp>, target: bool) -> Option<PopUp> {
        step(state, def(), true, target, || true)
    }

    /// `factions/syndicate/structures/laser_turret.md`: "rises in 8 frames ...; retracts after 32 frames with no
    /// target, taking 8 frames". A lowered pop-up that gets a target in tick 0 is raised at the end of tick 8; its
    /// last target in tick `t`, it starts to retract in `t + 32` and is down at the end of `t + 40`.
    #[test]
    fn a_popup_rises_in_8_frames_and_retracts_32_frames_after_its_last_target_in_8() {
        let mut state = tick(Some(PopUp::LOWERED), true);
        assert_eq!(state, Some(PopUp::Rising(Progress::new(8))));
        for frame in 1..=8 {
            assert!(matches!(state, Some(PopUp::Rising(_))), "frame {frame}");
            state = tick(state, true);
        }
        assert_eq!(state, Some(PopUp::raised(def())));
        // While it has a target nothing counts.
        for _ in 0..100 {
            state = tick(state, true);
        }
        assert_eq!(state, Some(PopUp::raised(def())));
        for idle in 1..=32 {
            assert!(matches!(state, Some(PopUp::Raised { .. })), "idle tick {idle}");
            state = tick(state, false);
        }
        assert_eq!(state, Some(PopUp::Retracting(Progress::new(8))));
        for frame in 1..=8 {
            assert!(matches!(state, Some(PopUp::Retracting(_))), "frame {frame}");
            state = tick(state, false);
        }
        assert_eq!(state, Some(PopUp::LOWERED));
        assert_eq!(tick(state, false), Some(PopUp::LOWERED));
    }

    /// A target seen again resets the idle count; one seen while the turret retracts doesn't turn it back: it goes
    /// down, and rises again in the tick it is down, its footprint never open in between.
    #[test]
    fn a_target_resets_the_idle_count_and_a_retracting_turret_goes_down_first() {
        let mut state = Some(PopUp::raised(def()));
        for _ in 0..31 {
            state = tick(state, false);
        }
        state = tick(state, true);
        assert_eq!(state, Some(PopUp::raised(def())));
        for _ in 0..32 {
            state = tick(state, false);
        }
        assert!(matches!(state, Some(PopUp::Retracting(_))));
        for _ in 0..7 {
            state = tick(state, true);
            assert!(matches!(state, Some(PopUp::Retracting(_))));
        }
        // No unit can be over it: the footprint blocked all along, so `clear` isn't asked.
        let risen = step(state, def(), true, true, || unreachable!("blocked until now"));
        assert_eq!(risen, Some(PopUp::Rising(Progress::new(8))));
    }

    /// Design change 0296 point 3: "it can't rise while any unit stands over it": it stays lowered, noting it, and
    /// rises in the tick the footprint is clear; with no target it forgets the note.
    #[test]
    fn a_unit_over_it_keeps_it_down() {
        let blocked = step(Some(PopUp::LOWERED), def(), true, true, || false);
        assert_eq!(blocked, Some(PopUp::Lowered { blocked: true }));
        assert_eq!(step(blocked, def(), true, true, || false), blocked);
        assert_eq!(tick(blocked, false), Some(PopUp::LOWERED));
        assert_eq!(tick(blocked, true), Some(PopUp::Rising(Progress::new(8))));
    }

    /// `> Conversion`: an upright turret an area comes to cover stands raised as a pop-up, and retracts; a raised
    /// pop-up whose Tunnel is lost is upright at once; one that is down, rising or retracting comes up by the pop-up's
    /// own rules, whatever it sees, and is upright once it stands.
    #[test]
    fn the_state_follows_the_form() {
        let upright = |state, target| step(state, def(), false, target, || true);
        assert_eq!(upright(None, false), None);
        assert_eq!(upright(None, true), None);
        assert_eq!(tick(None, true), Some(PopUp::raised(def())));
        // With no target its first idle tick counts at once.
        let mut idle = Progress::new(32);
        idle.step(RATIO_ONE);
        assert_eq!(tick(None, false), Some(PopUp::Raised { idle }));
        assert_eq!(upright(Some(PopUp::raised(def())), false), None);
        let mut state = upright(Some(PopUp::LOWERED), false);
        assert_eq!(state, Some(PopUp::Rising(Progress::new(8))));
        for _ in 0..8 {
            state = upright(state, false);
        }
        assert_eq!(state, Some(PopUp::raised(def())));
        assert_eq!(upright(state, false), None);
        // Kept down by a unit, as a pop-up is.
        let held = step(Some(PopUp::LOWERED), def(), false, false, || false);
        assert_eq!(held, Some(PopUp::Lowered { blocked: true }));
        // Retracting when its Tunnel falls: down, then up at once.
        let mut state = Some(PopUp::Retracting(Progress::new(8)));
        for _ in 0..8 {
            state = upright(state, false);
        }
        assert_eq!(state, Some(PopUp::Rising(Progress::new(8))));
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
