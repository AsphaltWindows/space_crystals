# Controls

Every key of the game and every faction's command panels, for a playtest ([the playtest notes](README.md)). From
*Keys* on, this page is generated from the code: the keys from the bindings table, the panels from the panel
templates, as the game builds them, with the hints' names and why a button is greyed. A test,
`playtest_docs::controls` in `crates/game/tests/playtest_docs/`, fails when the page no longer matches the code, so
after a change to a template or a binding, regenerate it and commit it with the change:

    UPDATE_DOCS=1 cargo test -p space_crystals --test game playtest_docs::controls

How to read the panels:

- **Keys** is the button's key, and for a button in a menu the keys that open the menu first: `Q → W` is W in the
  menu Q opens. Z (or Esc) goes back out of a menu. A menu that is the same as an earlier one says so instead of
  being listed again.
- **Button** is the button's label, **Command** its name as its hint shows it (the hint, at the foot of the
  InfoPanel, names the button under the pointer, with its key and its price).
- **Notes**: "click a target" (the command waits for a click, see *Aiming and placing* below), "place it" (a
  building's ghost follows the cursor), "Shift queues" (with Shift held, the order goes after the unit's others),
  "opens a menu".
- **Greyed** buttons: pressing one shows why, over the map, in the words the notes quote. "Not available yet" is a
  type or a command not in the game yet; "Not available for this unit" a command of the panel that this type can't
  use. "Requires a Temple" is a prerequisite: the button is greyed, and says so, until the player owns one.
- Each panel is that of one object of the type with everything it counts at hand (Recruits in a pool, a load,
  returning Drones). In a game a button can also be greyed for the moment: too few Space Crystals or Supplies, no
  Recruit of that kind inside, nothing to cancel, the building still being built. Pressing it says why.

Not in the tables: the minimap, where a left-click or a drag centres the camera, a right-click is the selection's
right-click on the ground there, and while a command waits for its target a left-click gives it that ground (see
[the M5 notes](m5.md)); and the selection's portraits: a click keeps only that unit selected, Ctrl+click every unit
of its type in the selection, Shift+click takes it out (Ctrl+Shift+click its type), Alt+click centres the camera on it.

<!-- generated: controls, begin; regenerate it, don't edit it -->

## Keys

### The camera

| Keys | Does |
|---|---|
| Left arrow | Pan west (held) |
| Right arrow | Pan east (held) |
| Up arrow | Pan north (held) |
| Down arrow | Pan south (held) |
| Middle button | Drag the map (held) |
| The cursor at the screen's edge (within 8 pixels) | Pan the camera that way |

### Selecting

| Keys | Does |
|---|---|
| Left click | Select: click a unit or a building, or drag a box; while a command awaits its target, click the target |
| Tab | The next type in the selection, whose panel shows |
| Shift+Tab | The previous type in the selection |
| Shift (held) | With a click or a box: add to the selection, or take a unit out of it; on a portrait, take it out |
| Ctrl (held) | With a click: every unit of that type on screen |
| Alt (held) | With a click on a portrait: centre the camera on that unit |

### Control groups

| Keys | Does |
|---|---|
| Ctrl+0 … Ctrl+9 | Make the selection that control group |
| Shift+0 … Shift+9 | Add the selection to that control group |
| 0 … 9 | Select that control group |
| 0 twice … 9 twice | Select that control group and centre the camera on it |

### Commands

| Keys | Does |
|---|---|
| Right click | The selection's right-click on what is under the cursor, which the pointer previews (below): move, attack, gather, drop off, enter, assist, repair, or a building's rally point; while a command awaits its target or a building is placed, cancel it |
| Esc | Cancel the command that awaits its target; in a menu or while placing, go back one step |
| Shift (held) | With an order: queue it after the unit's orders |

The command panel, at the bottom right, has 12 buttons; a key presses each, or a click. Its keys, as on the keyboard:

| Q | W | E | R |
|---|---|---|---|
| A | S | D | F |
| Z | X | C | V |

Z is Back in every menu, as Esc is, and while a command awaits its target. On every unit, A is Attack (shown while one of the selected units can attack) and X is Stop; on a building, X cancels, and C sets the rally point.

### Placing a building

| Keys | Does |
|---|---|
| R | Turn the building clockwise |
| Shift+R | Turn it the other way |
| F | Flip it left to right |
| Shift+F | Flip it top to bottom |

### The game

| Keys | Does |
|---|---|
| Space | Centre the camera on the latest minimap alert |
| F11, Alt+Enter | Full screen or a window |
| F10 | The playtest menu, which pauses the game: Resume, Restart, Quit |
| F5 | In a play-test from the map editor: back to the editor |

### Debug keys (with debug commands on, as they are unless `--no-debug`)

| Keys | Does |
|---|---|
| G | The tile grid |
| P | The units' paths |
| F2 | Switch to the next player: hot-seat |
| F3 | The fog of war off and on |

### Replays (`--replay`)

| Keys | Does |
|---|---|
| K | Pause and resume |
| . | Play one tick |
| - | Slower |
| = | Faster |

### Aiming and placing

- **Aiming** ("click a target" in the panels' notes): the pointer is a crosshair. Left-click the target; with Shift (held) the order is queued and the aim goes on until Shift is let go. Right click, Esc or Z cancels.
- **Placing** ("place it"): the building's ghost follows the cursor, green where it can stand and red where it can't. Left-click places it; the placing keys above turn it; Esc or Z goes back. Right click stops placing (a building the Deployment Center has ready stays ready).
- **The pointer** shows what a right-click would do there, or the awaited click; over an object, a dot in its owner's colour (yours green, an enemy's red, neutral yellow). Its colours are placeholders:

| Pointer | Means |
|---|---|
| Green | Move there, or follow the unit; a building's rally point |
| Red | Attack it; aiming an Attack at the ground, Attack Move there |
| Crimson | Attack Ground there |
| Orange | Patrol there |
| Cyan | Gather from it |
| Gold | Drop off the load there, or make it the drop-off |
| Violet | Enter it |
| Yellow-green | Help build it |
| Tan | Repair it |
| Pink | Cast there, its circle on the ground |
| Red and green stripes | The selected units would do different things |
| Grey | Nothing |

## GDO

### Units

#### Peacekeeper (PK)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Rev. | Reverse | click a target; Shift queues; greyed: "Not available for this unit" |
| E | Hold | Hold Position | Shift queues |
| R | Raise | Raise Shield |  |
| A | Attack | Attack | click a target; Shift queues |
| S | Patrol | Patrol | click a target; Shift queues |
| D | AtkGnd | Attack Ground | click a target; Shift queues; greyed: "Not available for this unit" |
| X | Stop | Stop | Shift queues |

#### Peacekeeper (Shielded) (PS)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Rev. | Reverse | click a target; Shift queues; greyed: "Not available for this unit" |
| E | Hold | Hold Position | Shift queues |
| R | Lower | Lower Shield |  |
| S | Patrol | Patrol | click a target; Shift queues |
| D | AtkGnd | Attack Ground | click a target; Shift queues; greyed: "Not available for this unit" |
| X | Stop | Stop | Shift queues |

#### Field Engineer (FEN)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Constr | Construct | opens a menu |
| W → Q | BKR | Build Bunker | greyed: "Not available yet" |
| W → W | FAT | Set up First Aid Tent | greyed: "Not available yet" |
| D | Repair | Repair | click a target; Shift queues |
| X | Stop | Stop | Shift queues |

#### Rocketman (RN)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Rev. | Reverse | click a target; Shift queues; greyed: "Not available for this unit" |
| E | Hold | Hold Position | Shift queues |
| R | Lock | Target Lock | click a target; Shift queues |
| A | Attack | Attack | click a target; Shift queues |
| S | Patrol | Patrol | click a target; Shift queues |
| D | AtkGnd | Attack Ground | click a target; Shift queues |
| X | Stop | Stop | Shift queues |

### Buildings

#### Deployment Center (DC)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Basic | Basic Construction | opens a menu |
| Q → Q | GN | Build Generator |  |
| Q → W | BR | Build Barracks |  |
| Q → E | EF | Build Extraction Facility |  |
| Q → S | AL | Build Arsenal | Requires a Barracks |
| Q → D | CC | Build Career Campus | Requires a Barracks |
| W | Adv. | Advanced Construction | greyed: "Not available yet" |

**While it builds:** E Place the building (greyed: "Still under construction"), X Cancel Construction.

**Once the building is ready:** E Place the building, X Cancel Ready Building.

#### Barracks (BR)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | PK | Train Peacekeeper |  |
| W | RN | Train Rocketman | Requires an Arsenal |
| E | FEN | Train Field Engineer | Requires a Career Campus |
| X | Cancel | Cancel Production |  |
| C | Rally | Set Rally Point | click a target |

**Right-click** sets its rally point, on the ground or on an object; on the building itself, it clears it.

#### Extraction Facility (EF)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | EP | Build Extraction Plate |  |

**While it builds:** Q Place the building (greyed: "Still under construction"), X Cancel Construction.

**Once the building is ready:** Q Place the building, X Cancel Ready Plate.

No commands: Generator (GN), Extraction Plate (EP), Career Campus (CC), Arsenal (AL).

## Syndicate

### Units

#### Guard (GD)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Rev. | Reverse | click a target; Shift queues; greyed: "Not available for this unit" |
| E | Hold | Hold Position | Shift queues |
| A | Attack | Attack | click a target; Shift queues |
| S | Patrol | Patrol | click a target; Shift queues |
| D | AtkGnd | Attack Ground | click a target; Shift queues; greyed: "Not available for this unit" |
| X | Stop | Stop | Shift queues |

#### Agent (AG)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | T1 | Build Tunnel | place it; Shift queues |
| A | Attack | Attack | click a target; Shift queues |
| S | Drop | Drop Off Resources | click a target; Shift queues |
| D | Repair | Repair | click a target; Shift queues |
| X | Stop | Stop | Shift queues |

#### Grenadier (GR)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Rev. | Reverse | click a target; Shift queues; greyed: "Not available for this unit" |
| E | Hold | Hold Position | Shift queues |
| A | Attack | Attack | click a target; Shift queues |
| S | Patrol | Patrol | click a target; Shift queues |
| D | AtkGnd | Attack Ground | click a target; Shift queues |
| X | Stop | Stop | Shift queues |

### Buildings

#### Tunnel (T1)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Upgr. | Upgrade Tunnel |  |
| W | Expand | Expand Tunnel | opens a menu |
| W → Q | HI | Build Hideout | place it |
| W → W | OL | Build Optics Lab | place it |
| W → E | CP | Build Chemical Plant | place it |
| W → R | VT | Build Vault | greyed: "Not available yet" |
| W → A | VA | Build Vehicle Assembly | greyed: "Not available yet" |
| W → S | FE | Build Forge | greyed: "Not available yet" |
| W → D | NC | Build Null Chamber | greyed: "Not available yet" |
| W → F | AA | Build Aerospace Assembly | greyed: "Not available yet" |
| W → X | OC | Build Orbital Command | greyed: "Not available yet" |
| W → C | LT | Build Laser Turret | place it; Requires an Optics Lab |
| E | Eject | Eject | opens a menu |
| E → Q | AG | Eject Agent |  |
| E → W | GD | Eject Guard |  |
| E → E | GR | Eject Grenadier |  |
| X | Cancel | Cancel Operation |  |

**Before it is finished:** X Cancel Construction.

#### Hideout (HI)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | AG | Train Agent |  |
| W | GD | Train Guard | Requires an Optics Lab |
| E | GR | Train Grenadier | Requires a Chemical Plant |
| X | Cancel | Cancel Production |  |
| C | Rally | Set Rally Point | click a target |

**Right-click** sets its rally point, on the ground or on an object; on the building itself, it clears it.

**Before it is finished:** X Cancel Construction, C Set Rally Point.

No commands: Optics Lab (OL), Chemical Plant (CP), Laser Turret (LT).

## Cults

### Units

#### Recruit (RT)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Basic | Basic Construct | opens a menu |
| W → Q | RH | Build Recruitment Hall | place it; Shift queues |
| W → W | SE | Build Storage | place it; Shift queues |
| W → A | AY | Build Armory | place it; Shift queues; Requires a Storage |
| W → S | GS | Build Gunworks | place it; Shift queues; Requires an Armory |
| W → D | GA | Build Garrison | place it; Shift queues; Requires an Armory |
| W → X | AR | Build Altar | place it; Shift queues; Requires a Storage |
| E | Adv. | Advanced Construct | opens a menu |
| E → A | CS | Build Chop Shop | place it; Shift queues; Requires an Armory |
| E → S | MS | Build Machine Shop | place it; Shift queues; Requires a Chop Shop |
| E → D | AP | Build Airstrip | place it; Shift queues; Requires a Chop Shop |
| E → F | SH | Build Scrap Hangar | place it; Shift queues; Requires an Airstrip |
| E → X | TE | Build Temple | place it; Shift queues; Requires an Altar |
| E → C | CL | Build Cathedral | place it; Shift queues; Requires a Temple |
| E → V | SR | Build Shrine | place it; Shift queues; Requires a Temple |
| A | Attack | Attack | click a target; Shift queues |
| S | Assist | Assist Construction | click a target; Shift queues |
| D | Repair | Repair | click a target; Shift queues |
| X | Stop | Stop | Shift queues |

#### Militant (MT)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Rev. | Reverse | click a target; Shift queues; greyed: "Not available for this unit" |
| E | Hold | Hold Position | Shift queues |
| A | Attack | Attack | click a target; Shift queues |
| S | Patrol | Patrol | click a target; Shift queues |
| D | AtkGnd | Attack Ground | click a target; Shift queues; greyed: "Not available for this unit" |
| X | Stop | Stop | Shift queues |

#### Gunner (GU)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Rev. | Reverse | click a target; Shift queues; greyed: "Not available for this unit" |
| E | Hold | Hold Position | Shift queues |
| A | Attack | Attack | click a target; Shift queues |
| S | Patrol | Patrol | click a target; Shift queues |
| D | AtkGnd | Attack Ground | click a target; Shift queues |
| X | Stop | Stop | Shift queues |

#### Disciple (DE)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Rev. | Reverse | click a target; Shift queues; greyed: "Not available for this unit" |
| E | Hold | Hold Position | Shift queues |
| R | Shroud | Shroud | Shift queues |
| S | Patrol | Patrol | click a target; Shift queues |
| D | AtkGnd | Attack Ground | click a target; Shift queues; greyed: "Not available for this unit" |
| X | Stop | Stop | Shift queues |

#### Seeker (SKR)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Rev. | Reverse | click a target; Shift queues |
| E | Hold | Hold Position | Shift queues |
| A | Attack | Attack | click a target; Shift queues |
| S | Patrol | Patrol | click a target; Shift queues |
| D | AtkGnd | Attack Ground | click a target; Shift queues; greyed: "Not available for this unit" |
| X | Stop | Stop | Shift queues |

#### Sparrow (SW)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Rev. | Reverse | click a target; Shift queues; greyed: "Not available for this unit" |
| E | Hold | Hold Position | Shift queues |
| A | Attack | Attack | click a target; Shift queues |
| S | Patrol | Patrol | click a target; Shift queues |
| D | AtkGnd | Attack Ground | click a target; Shift queues; greyed: "Not available for this unit" |
| X | Stop | Stop | Shift queues |

#### Hand of Heaven (HOH)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Rev. | Reverse | click a target; Shift queues |
| E | Hold | Hold Position | Shift queues |
| A | Attack | Attack | click a target; Shift queues |
| S | Patrol | Patrol | click a target; Shift queues |
| D | AtkGnd | Attack Ground | click a target; Shift queues |
| X | Stop | Stop | Shift queues |

#### Prayer-mobile (PM)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Rev. | Reverse | click a target; Shift queues |
| E | Hold | Hold Position | Shift queues |
| R | Insp. | Inspire |  |
| S | Patrol | Patrol | click a target; Shift queues |
| F | Suppr. | Suppress |  |
| X | Stop | Stop | Shift queues |

### Buildings

#### Recruitment Hall (RH)

| Keys | Button | Command | Notes |
|---|---|---|---|
| C | Rally | Set Rally Point | click a target |

**Right-click** sets its rally point, on the ground or on an object; on the building itself, it clears it.

**Before it is finished:** X Cancel Construction, C Set Rally Point.

#### Armory (AY)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Plain | Plain Recruits | opens a menu |
| Q → Q | MT | Train Militant |  |
| Q → W | GU | Train Gunner | Requires a Gunworks |
| Q → A | Rel. | Release |  |
| W | Const. | Constitution Recruits | opens a menu, the same as Q's |
| E | Metab. | Metabolism Recruits | opens a menu, the same as Q's |
| R | Both | Both Recruits | opens a menu, the same as Q's |
| A | Eject | Eject All |  |
| X | Cancel | Cancel Production |  |
| C | Rally | Set Rally Point | click a target |

**Right-click** sets its rally point, on the ground or on an object; on the building itself, it clears it.

**Before it is finished:** X Cancel Construction, C Set Rally Point.

#### Garrison (GA)

| Keys | Button | Command | Notes |
|---|---|---|---|
| A | Eject | Eject All |  |

**Before it is finished:** X Cancel Construction.

#### Chop Shop (CS)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Plain | Plain Recruits | opens a menu |
| Q → Q | SKR | Build Seeker |  |
| Q → W | HOH | Build Hand of Heaven |  |
| Q → E | PM | Build Prayer-mobile | Requires a Temple |
| Q → A | Rel. | Release |  |
| W | Const. | Constitution Recruits | opens a menu, the same as Q's |
| E | Metab. | Metabolism Recruits | opens a menu, the same as Q's |
| R | Both | Both Recruits | opens a menu, the same as Q's |
| A | Eject | Eject All |  |
| X | Cancel | Cancel Production |  |
| C | Rally | Set Rally Point | click a target |

**Right-click** sets its rally point, on the ground or on an object; on the building itself, it clears it.

**Before it is finished:** X Cancel Construction, C Set Rally Point.

#### Airstrip (AP)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Plain | Plain Recruits | opens a menu |
| Q → Q | SW | Build Sparrow |  |
| Q → A | Rel. | Release |  |
| W | Const. | Constitution Recruits | opens a menu, the same as Q's |
| E | Metab. | Metabolism Recruits | opens a menu, the same as Q's |
| R | Both | Both Recruits | opens a menu, the same as Q's |
| A | Eject | Eject All |  |
| X | Cancel | Cancel Production |  |
| C | Rally | Set Rally Point | click a target |

**Right-click** sets its rally point, on the ground or on an object; on the building itself, it clears it.

**Before it is finished:** X Cancel Construction, C Set Rally Point.

#### Altar (AR)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | RT | Recruit | opens a menu |
| Q → Q | Plain | Unaugmented | opens a menu |
| Q → Q → Q | Const. | Enhanced Constitution |  |
| Q → Q → W | Metab. | Improved Metabolism |  |
| Q → Q → E | Both | Both Augmentations |  |
| Q → Q → A | Rel. | Release |  |
| Q → W | Const. | Enhanced Constitution only | opens a menu |
| Q → W → W | Metab. | Improved Metabolism |  |
| Q → W → A | Rel. | Release |  |
| Q → E | Metab. | Improved Metabolism only | opens a menu |
| Q → E → Q | Const. | Enhanced Constitution |  |
| Q → E → A | Rel. | Release |  |
| Q → R | Both | Fully augmented | opens a menu |
| Q → R → A | Rel. | Release |  |
| W | MT | Militant | opens a menu, the same as Q's |
| E | GU | Gunner | opens a menu, the same as Q's |
| R | DE | Disciple | opens a menu, the same as Q's |
| A | Eject | Eject All |  |
| X | Cancel | Cancel Production |  |
| C | Rally | Set Rally Point | click a target |

**Right-click** sets its rally point, on the ground or on an object; on the building itself, it clears it.

**Before it is finished:** X Cancel Construction, C Set Rally Point.

#### Temple (TE)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | EC L1 | Research Enhanced Constitution Level 1 | then Enhanced Constitution Level 2, Requires a Cathedral |
| W | IM L1 | Research Improved Metabolism Level 1 | then Improved Metabolism Level 2, Requires a Cathedral |
| A | Plain | Release one Plain Recruit |  |
| S | Const. | Release one Constitution Recruit |  |
| D | Metab. | Release one Metabolism Recruit |  |
| F | Both | Release one Both Recruit |  |
| X | Cancel | Cancel Production |  |
| V | Eject | Eject All |  |

**Before it is finished:** X Cancel Construction.

#### Cathedral (CL)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Plain | Plain Recruits | opens a menu |
| Q → Q | DE | Train Disciple |  |
| Q → A | Rel. | Release |  |
| W | Const. | Constitution Recruits | opens a menu, the same as Q's |
| E | Metab. | Metabolism Recruits | opens a menu, the same as Q's |
| R | Both | Both Recruits | opens a menu, the same as Q's |
| A | Eject | Eject All |  |
| X | Cancel | Cancel Production |  |
| C | Rally | Set Rally Point | click a target |

**Right-click** sets its rally point, on the ground or on an object; on the building itself, it clears it.

**Before it is finished:** X Cancel Construction, C Set Rally Point.

#### Shrine (SR)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Health | Health regeneration | click a target |
| W | Energy | Energy regeneration | click a target |
| A | Plain | Release one Plain Recruit |  |
| S | Const. | Release one Constitution Recruit |  |
| D | Metab. | Release one Metabolism Recruit |  |
| F | Both | Release one Both Recruit |  |
| V | Eject | Eject All |  |

**Before it is finished:** X Cancel Construction.

No commands: Storage (SE), Gunworks (GS), Machine Shop (MS), Scrap Hangar (SH).

## Colonists

### Units

#### Drone (DR)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Deploy | Deploy | place it; Shift queues |
| S | Drop | Set Drop-off Location | click a target |
| X | Stop | Stop | Shift queues |

#### Sabre Legionnaire (SL)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Rev. | Reverse | click a target; Shift queues; greyed: "Not available for this unit" |
| E | Hold | Hold Position | Shift queues |
| R | Conv. | Convert to Rifle Legionnaire |  |
| A | Attack | Attack | click a target; Shift queues |
| S | Patrol | Patrol | click a target; Shift queues |
| D | AtkGnd | Attack Ground | click a target; Shift queues; greyed: "Not available for this unit" |
| X | Stop | Stop | Shift queues |

#### Rifle Legionnaire (RL)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Move | Move | click a target; Shift queues |
| W | Rev. | Reverse | click a target; Shift queues; greyed: "Not available for this unit" |
| E | Hold | Hold Position | Shift queues |
| R | Conv. | Convert to Sabre Legionnaire |  |
| A | Attack | Attack | click a target; Shift queues |
| S | Patrol | Patrol | click a target; Shift queues |
| D | AtkGnd | Attack Ground | click a target; Shift queues; greyed: "Not available for this unit" |
| X | Stop | Stop | Shift queues |

### Buildings

#### Beacon (BN)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | Basic | Warp In (Basic) | opens a menu |
| Q → Q | EN | Warp In Egress Node | place it |
| Q → W | RP | Warp In Robo Portal | place it |
| Q → E | BP | Warp In Bio Portal | place it |
| Q → A | AF | Warp In Alloy Furnace | greyed: "Not available yet" |
| Q → S | ER | Warp In Essence Refinery | greyed: "Not available yet" |
| Q → D | FR | Warp In Flux Reactor | greyed: "Not available yet" |
| W | Adv. | Warp In (Advanced) | opens a menu |
| W → Q | EE | Warp In Exophysics Engine | greyed: "Not available yet" |
| W → W | SK | Warp In Skylance | greyed: "Not available yet" |
| W → E | FC | Warp In Fusion Core | greyed: "Not available yet" |
| W → R | PT | Warp In Plasma Turret | greyed: "Not available yet" |
| W → A | RA | Warp In Relay Array | greyed: "Not available yet" |
| W → S | NS | Warp In Navigation Spire | greyed: "Not available yet" |
| W → D | PA | Warp In Psionic Amplifier | greyed: "Not available yet" |
| W → F | SA | Warp In Structural Analyzer | greyed: "Not available yet" |

**Before it is finished:** X Cancel Deployment.

#### Egress Node (EN)

| Keys | Button | Command | Notes |
|---|---|---|---|
| F | Drones | Select Returning Drones | shown while Drones return to it |

**Before it is finished:** X Cancel Warp In.

#### Robo Portal (RP)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | DR | Train Drone |  |
| W | ST | Train Scout | greyed: "Not available yet" |
| X | Cancel | Cancel Production |  |
| C | Rally | Set Rally Point | click a target |

**Right-click** sets its rally point, on the ground or on an object; on the building itself, it clears it.

**Before it is finished:** X Cancel Warp In, C Set Rally Point.

#### Bio Portal (BP)

| Keys | Button | Command | Notes |
|---|---|---|---|
| Q | SL | Train Sabre Legionnaire |  |
| W | LA | Train Lancer | greyed: "Not available yet" |
| E | PU | Train Psionic unit | greyed: "Not available yet" |
| X | Cancel | Cancel Production |  |
| C | Rally | Set Rally Point | click a target |

**Right-click** sets its rally point, on the ground or on an object; on the building itself, it clears it.

**Before it is finished:** X Cancel Warp In, C Set Rally Point.

<!-- generated: controls, end -->
