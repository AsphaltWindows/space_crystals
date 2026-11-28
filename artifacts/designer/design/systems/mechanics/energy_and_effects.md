# Mechanic: Energy, Casts and Timed Effects

(Mechanics doc, 2026-11-28; Q0193. Defined independently of any unit; gathers rules previously written in the Disciple's, Prayer-mobile's, Shrine's, Null Chamber's and Augmentations docs; no rule changed in the move.)

Three parts: **energy pools** (what casters spend), **cast archetypes** (how an ability is used), and **timed effects** (buffs and debuffs that casts and services apply). A caster's doc says which it uses, with parameters and exceptions.

## Energy pools
- **Parameters**: MaxEnergy; starting energy; regeneration (a flat rate per second, or per unit inside, e.g. per Recruit in a Shrine, with none when empty).
- **Regeneration continues while channelling** (user, 2026-11-26; Q0113).
- A pool can be **boosted** by a timed effect (e.g. a regeneration buff adds to the rate).
- **Shown to every player** who sees the caster (bar and InfoPanel line), as HP and Shields are (placeholder M5-N4a, kept by the user).
- **One press, one caster**: casters are **not Groupable** (each press casts from one caster, Tab to the next; systems/controls/selection.md > No smart-casting).
- An ability pressed without enough energy is refused with the unavailable-command feedback (systems/controls/interface.md > Pressing an unavailable command).

## Cast archetypes

### Instant cast
Pressed, it happens at once and spends its energy. Shape parameters:
- **Around the caster** (e.g. a circle of radius r around it), **in front of it** (an arc of a given angle and length, aimed by the caster's facing), or **on itself** (e.g. creating objects beside it).
- **Who it affects** (e.g. own infantry; every unit, own and enemy alike; not buildings; air included or not).
- An object is in the area when its **centre** is inside.

### Targeted area cast
Pressed, the player left-clicks a point (in vision, if the doc says so); the area there is affected at once. Same shape and "who" parameters; centre inside.

### Channel
(user, 2026-11-26; the Disciple's Shroud) Pressed, the caster **keeps an effect going** while its energy **drains gradually**.
- **Parameters**: drain per second; **minimum energy to start**; what the caster can do meanwhile (e.g. can't move).
- **What ends it**: only the caster's **death**, a **cancel** (Stop, or any other order **given without Shift**; casting another of its abilities if its doc says so), or running **out of energy**. **Taking damage doesn't end it.** A Shift-queued order waits and runs when the channel ends (systems/controls/commands.md > Unit Command; Q0171).
- **After it ends** the effect may **linger** for a set time (parameter; e.g. Shroud's cloud, 1.5 s, stays where it was).

### Queueing
An instant cast is **Shift-queueable** as an order step when its doc says so (e.g. Mirage, Q0188): it casts when its turn comes if there is enough energy, else it is dropped with the refusal feedback.

## Timed effects (buffs and debuffs)
- **Applied at the moment of casting** (Shrine rule, user, 2026-11-26): units in the area when it is cast receive it for its duration and **keep it** even if they leave; units entering afterward don't get it.
- **Recasting restarts the duration**; an effect **doesn't stack with itself** (Q0118).
- **Different sources combine**: percentage speed effects follow systems/combat.md > AttackSpeed (augmentations add, then buffs multiply).
- **A buff never changes how a unit behaves** (user, 2026-11-27; Q0187/Q0190): e.g. extra sight widens vision only, not target scans (which measure from the centre, as vision).
- **Buff marks** (user, 2026-11-27; Q0145): a buffed unit shows a small mark beside its HP bar to **every player who sees it**, enemies included.
- **Copies** (Q0188): a copy of a unit (e.g. a Disciple's illusion) gets the original's timed effects for the time each has left.
- **Debuffs** take the same rules (e.g. Suppress: affected units can't make **weapon attacks** or use **abilities that use a weapon** for its duration; other abilities unaffected).
- **Services** that apply an effect on leaving (e.g. the Syndicate Null Chamber's exit buff) give it at the moment the unit leaves, with the full duration; leaving again refreshes it.

## Used by (index only)
Energy pools: Cults Disciple, Prayer-mobile, Shrine. Instant cast: Inspire, Suppress (Prayer-mobile), Mirage (Disciple). Targeted area cast: Shrine's two abilities. Channel: Shroud (Disciple). Timed effects: Inspire, Suppress, Shrine regeneration and energy regeneration, Null Chamber exit buff.
