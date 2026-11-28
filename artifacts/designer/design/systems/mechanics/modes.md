# Mechanic: Unit Modes (Deploy and Toggle)

(Mechanics doc, 2026-11-28; Q0193. Defined independently of any unit; gathers rules previously written in the Artillery Rig's, Laser Battery's, Peacekeeper's and Weapon Conversion docs; no rule changed in the move.)

Some units have **two forms** (modes) with different abilities, e.g. mobile and deployed, or shield down and up. Each form is its **own unit type**, with its own SelectionGroup (systems/controls/selection.md). There are two archetypes for switching, which differ in **when the type changes** and **how orders are treated**. A unit's doc says which archetype it uses, gives the parameters, and lists only its exceptions.

## Parameters (every modal unit)
- **The two forms**: what each can and can't do (move, attack, abilities, target domains).
- **Switch time** each way (frames).
- **Hotkeys**.
- **Archetype-specific parameters** (below).

## Archetype: Deploy (an order)
(user, 2026-11-25) **The switching process is what changes the unit**: it stays the old type until the process completes, then becomes the new type (type and SelectionGroup).
- **An order**: Deploy (and Undeploy, if the unit can switch back) is a normal order. **Queueable**: Shift-Deploy waits until the earlier orders are done (e.g. move here, then deploy). Without Shift it replaces the current orders and starts at once.
- **While switching** the unit can **neither move nor attack**.
- **No reversing mid-way**: the reverse switch is available only once the switch is complete (and the same for the way back).
- **Not cancellable** (user, 2026-11-25): once started, a switch always runs to completion; Stop doesn't interrupt it.
- **Orders given while switching** (user, 2026-11-25; Q0071): the **last order given** (without Shift; each new one replaces the previous) is **held**, and is the first thing tried once the switch finishes; Shift-queued orders line up behind it. Then each order is checked against the new type: one it can't carry out (e.g. Move for a unit that can't move) is **dropped** with the usual unavailable-command feedback, and the next is tried.
- When the switch makes a SelectionGroup disappear, the group the units joined becomes active (systems/controls/selection.md > Selection > ActiveGroup > When the ActiveGroup disappears).
- **Parameters**: whether it can switch back (some deploys are **permanent**, e.g. into a structure); whether Deploy is **targeted** (a location: the unit drives there and deploys on arrival, validated on arrival like a worker-built structure, core/entities.md > Placement Validation > Worker-Built Structures); what carries over (e.g. damage taken).

## Archetype: Toggle (a mode switch)
(user, 2026-11-24/25) **The unit's type switches at the press**; the unit then works its way into the new form.
- **A toggle sets a target form** (Q0049/Q0053): pressing flips the target form. However fast it is pressed, every unit ends up in the same form; quick presses skip the in-between switches. The pressed units join the SelectionGroup of the target form, and that group becomes the ActiveGroup.
- **Not an order**: it doesn't replace the unit's orders. **Orders are kept**: the unit carries on with its current orders in the new form. An order the new form can't carry out is **on hold**, not dropped, and resumes when the form allows it (e.g. an Attack while shielded follows the target, then attacks once the shield is down).
- **Which orders are accepted follows the new type from the press; carrying them out waits for the switch to finish** (Q0053).
- **Not queueable**: it always happens immediately; Shift changes nothing.
- **Pressed mid-attack**: during Firing or Cooldown (not interruptible, systems/combat.md > AttackPhase) the switch is **deferred**: the shot finishes, then the switch starts at once. During Aiming or Reloading it interrupts immediately. Pressed again while still deferred, the target flips back and the waiting switch is dropped.
- **Parameter: pressed again mid-switch**: either it **reverses from the current point** (e.g. the Riot Shield: halfway up, lowering takes only the half already moved) or the switch **completes, then switches back** (e.g. Weapon Conversion).
- **Parameter: what the in-between state allows** (e.g. half speed, no attack; whether the new form's effects apply only once complete).

## Not these archetypes
The Colonists Drone's Deploy into a Beacon is its own rule: it becomes a Beacon **immediately** and the transformation **can be cancelled** (factions/colonists/economy.md > Beacon Deployment).

## Used by (index only)
Deploy: Artillery Rig, Laser Battery, Subterranean Transport (Burrow/Surface), Prospector (permanent, into an Outpost). Toggle: Peacekeeper's Riot Shield, Colonists Weapon Conversion.
