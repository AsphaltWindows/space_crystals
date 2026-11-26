//! Phase 5e's run (`super`, module docs): the attacks of the tick, the projectiles due, and the tick's list of hits
//! applied together.

use std::collections::BTreeMap;

use super::{AttackPhase, Projectile, damage};
use super::{occupants, phases, projectiles, structures, targeting, weapon_ref};
use crate::augmentation::Research;
use crate::conversion;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::map::Map;
use crate::movement;
use crate::network::Held;
use crate::object::Object;
use crate::popup;
use crate::riot_shield;
use crate::units::Tick;
use crate::vision::Vision;

/// Phase 5e of `tick` (`docs/movement.md > 4`): every unit before `first_new`, in id order, runs one tick of its
/// attack sequence against a snapshot of the targets taken before any of them acts, and so does every complete
/// structure that attacks, having picked its target first ([`structures`]; none while `auto_targeting` is off, as no
/// unit auto-targets then: `DebugCommand::SetAutoTargeting`), and the projectiles they launch
/// join `projectiles`, the store (`Sim::projectiles`); then the projectiles due land, in launch order (slice M4-O,
/// plan decision T45), their blasts' hits ahead of the tick's new hits on the list; then the tick's hits are applied
/// together, and each reveals its attacker to the owner of the object hit if that player didn't see it
/// (`vision::reveal`; a blast's attacker as its damage lands, if it is still alive: design change 0293). Returns the
/// objects destroyed, in id order, for the removal path (`Sim::remove_objects`). A death that removes a blocking
/// footprint must re-sync the grids (`paths::sync`), as phase 5b does. `vision` is the last tick's: what each owner
/// sees, all through the phase (the reveals don't change what it sees of tiles). `turns` holds the angle phase 5d of
/// this tick turned each unit: the aim turns a unit only by what is left of its TurnRate (one budget per tick).
/// (M5-N1a) Each attacks with its weapon as its Augmentations make it, at its owner's levels in `research`
/// ([`super::weapon`]).
#[expect(
    clippy::too_many_arguments,
    reason = "phase 5e's state, in the parts of the sim it reads and writes"
)]
pub(crate) fn step(
    objects: &mut BTreeMap<ObjectId, Object>,
    held: &mut BTreeMap<ObjectId, Held>,
    map: &Map,
    vision: &mut Vision,
    projectiles: &mut Vec<Projectile>,
    turns: &movement::Turns,
    first_new: ObjectId,
    tick: Tick,
    auto_targeting: bool,
    research: Research,
    events: &mut Vec<SimEvent>,
) -> Vec<ObjectId> {
    // A structure that attacks is always busy: it looks for its targets in this phase. So is a turret with a target
    // locked, which it turns toward.
    let busy = |o: &Object| {
        o.attack.as_ref().is_some_and(|a| {
            a.target.is_some()
                || a.location.is_some()
                || a.phase.is_some()
                || a.locked().is_some()
                || o.placement.is_some()
        })
    };
    // (M5-N2) So is a unit inside a building whose occupants fire (`occupants`).
    let inside = occupants::firing(objects, held, first_new);
    if projectiles.is_empty() && inside.is_empty() && !objects.range(..first_new).any(|(_, o)| busy(o)) {
        return Vec::new();
    }
    let targets = targeting::Targets::snapshot(map, objects);
    let mut volley = phases::Volley::new(tick);
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        // `factions/colonists/units/weapon_conversion.md`: a unit converting its weapon attacks nothing; what it
        // engages waits for the new weapon (`conversion`).
        if conversion::is_converting(o) {
            continue;
        }
        let may_attack = riot_shield::allows_attack(o);
        // The weapon in hand: the form's, which a Legionnaire whose conversion waits still fights with (design change
        // 0198); its shape and its turn rate are its type's, the same in either form. A structure's is its type's.
        // (M5-N1a) With its Augmentations' effects.
        let mut augmented = None;
        let Some(attrs) = weapon_ref(o, research, &mut augmented) else {
            continue;
        };
        if o.attack.is_none() {
            continue;
        }
        // A structure that attacks (plan decision T49): only once complete (design change 0234: a site "does nothing
        // but Cancel"); it picks its own targets.
        let structure = o.def().unit.is_none();
        if structure && !o.is_operational() {
            continue;
        }
        // (M4-S2) A pop-up turret fires only while raised (`popup::may_fire`: "It can't fire while rising or
        // retracting"; lowered, it is underground).
        let may_fire = popup::may_fire(o);
        let sight = o.owner.and_then(|owner| vision.player(owner));
        let shooter = phases::Shooter::new(map, o, attrs, sight, turns.angle(o.id));
        let Some(state) = &mut o.attack else {
            continue;
        };
        if !may_attack {
            state.engage(None);
            state.lock(None);
        }
        if structure {
            structures::pick(state, &shooter, &targets, auto_targeting);
            structures::face(state, &mut o.facing, &shooter, &targets);
        }
        if !may_fire {
            // It keeps the target it picked, which it rises for and faces from its first rising frame, and starts no
            // sequence: an aim is cancelled, and a shot or a reload under way runs on to its end.
            if matches!(
                state.phase,
                Some(AttackPhase::Aiming { .. } | AttackPhase::AimingAt { .. })
            ) {
                state.cancel();
            }
            if state.phase.is_none() {
                continue;
            }
        }
        phases::run(state, &mut o.facing, &shooter, &targets, &mut volley, events);
    }
    occupants::step(
        &inside,
        objects,
        held,
        map,
        vision,
        &targets,
        auto_targeting,
        research,
        &mut volley,
        events,
    );
    projectiles.append(&mut volley.launched);
    // The blasts first: due projectiles land before this tick's new hits are applied (plan decision T45).
    let mut hits = Vec::new();
    projectiles::land(projectiles, tick, &targets, &mut hits, events);
    hits.append(&mut volley.hits);
    let dead = damage::apply(objects, &hits, events);
    // Design change 0293 (answering Q0083 point 2; `core/vision.md > Attacks from outside vision`): a blast reveals its
    // attacker as a shot does, here, when its damage lands, not at the throw; only to the owners of the objects it
    // damaged (every hit takes something, its minimum at least, so a blast that hits nothing, a dodged grenade,
    // reveals nobody); and only if the attacker is still alive: one that died during the flight is out of the game
    // (plan decision T45). One killed in this tick, by its own blast too, leaves with the tick's dead, unseen. (M5-N2) A
    // unit firing from inside a building reveals the building (`occupants::building_of`).
    for hit in &hits {
        let victim = objects.get(&hit.target).and_then(|o| o.owner);
        let attacker = (objects.get(&hit.attacker)).or_else(|| occupants::building_of(objects, held, hit.attacker));
        if let (Some(victim), Some(attacker)) = (victim, attacker) {
            vision.reveal(victim, attacker, tick);
        }
    }
    dead
}
