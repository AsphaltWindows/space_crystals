# Adding a unit or structure — detail

Read this when you add a unit or structure and need the long checklist; the core has the short one.

## Adding a unit or structure

One kind (plan decision T54, slices M4-R1 and M4-R2): its own files, and one line in each registry.

1. **Its def module**, `crates/sim/src/defs/<faction>/{units,structures}/<object>.rs`, named as the design's file of
   the object (`factions/<faction>/{units,structures}/<object>.md`; an economy building's is
   `factions/<faction>/economy.md`, and `defs/map/` holds the map's objects). Its doc comment cites that file. It
   holds the kind's `ObjectDef`, with every number the design gives the object, each cited, and the object's tests;
   the literal ends with `..ObjectDef::NONE` (its `UnitDef` or `StructureDef` with theirs: *Object model*).
   What the type does differently goes in a field of its def, which the logic reads, rather than in a `match` on its
   kind. Then its line in the sim's kind registry (`kind_registry!` in `defs.rs`), **at the end**: that one line makes
   the `ObjectKind` variant, its place in `ObjectKind::ALL` and its arm of `ObjectKind::def`; the state hash numbers
   kinds in that order, and maps and replays name them by their serialized names, which must not change. A rule of
   its faction rather than of the type goes in `defs/<faction>.rs`. Then *Object model > Adding an object type* for
   its capabilities, and *Adding a capability* for anything new it does. What M4's kinds put in their defs, as
   examples: how a structure is built, `StructureDef::site` (a `SiteDef`: a worker builds it, the Tunnel) or
   `StructureDef::expansion` (an `ExpansionDef`: a Tunnel builds it in its area, the Hideout, the Optics Lab, the
   Chemical Plant and the Laser Turret, the turret's one way since design change 0359), with `StructureDef::underground`
   for an expansion; what must stand first, `StructureDef::requires` for a structure and `Recipe::requires` in its
   producer's `ProductionDef` for a unit (the Guard and the Grenadier, slice M4-K); a structure's weapon,
   `StructureDef::attack`, and its pop-up form, `StructureDef::popup` (the Laser Turret, slices M4-S1 and M4-S2; every
   object of such a type a pop-up since slice M5-Y). A new expansion also turns its `ExpandEntry::Planned` into its
   `Kind` in `defs::syndicate::EXPAND_MENU`, and a new Syndicate unit joins `EJECT_MENU_ORDER` there. A unit's second
   type (the Peacekeeper (Shielded); slice M6-GK's Artillery Rig (Deployed); slice M6-SK's Laser Battery (Deployed) and
   Subterranean Transport (Burrowed)) is a kind of its own in its first type's module (and client module), its literal
   made from the first's; a deployed type is immobile by its def, MaxSpeed and ReverseMaxSpeed 0 (plan decision T87),
   named in `IMMOBILE` (`defs/tests.rs`), and a type that never enters the Tunnel Network stays out of
   `EJECT_MENU_ORDER` (`NEVER_INSIDE`, `defs/syndicate/tests.rs`). A kind registered before its slice makes it (the
   GDO's, slice M6-GK) keeps its `Recipe` in its def module, held in `defs::PLANNED_RECIPES` with its producer; its
   slice moves it into the producer's `constructs`, `production` or `builds` (and its `Planned` menu entry, if it has
   one, becomes its command). (M6-GH1) An aircraft that rearms at an Airfield (the Hornet and, slice M6-GL, the
   Dragonfly) has its ammo in `UnitDef::ammo` (an `AmmoDef`), which alone gives it `Object::aircraft`, its assignment,
   the gate's ammo checks, its F Return to Base and its pips (*Production > The Airfield*), and, a Glider, its landing
   approach (`movement/approach.rs`); an Airfield's limit is its `StructureDef::airfield` (an `AirfieldDef`).
2. **Its client module**, `crates/game/src/kinds/<faction>/<object>.rs` (*Client architecture > Kinds*): its
   `ClientKind`, with
   - its label, two letters of its own (`kinds::tests::every_label_is_its_own` checks them against every other
     label, the planned buttons' too);
   - its look (`object_render::Look`): a unit's body shape and whether it carries its label, a structure, a
     structure that attacks (`Look::Turret`), or one whose aircraft take off along it (`Look::Runway`, slice M5-N7). A
     new shape adds a `Look` or `UnitShape` variant and its drawing, in `object_render/`;
   - its shots' style (`combat_effects::ShotStyle`) if it has a weapon, none if not
     (`kinds::tests::every_kind_module_fits_its_def` checks both against its def);
   - its panel template. A shared one: `PanelTemplate::of(InterfaceTemplate::BasicCombatUnit)`, or `Production` for a
     producer, whose grid comes from its `ProductionDef`, or `PoolProducer` for a pool building's (slice M5-F2, the
     Armory's `templates/cults/armory.rs`, its KindMenus from its `ProductionDef`). `PanelTemplate::NONE` until its
     slice builds its panel.

   Then its line in the client's registry, `kinds::of`: the `match` is exhaustive, so the compiler names a kind
   without one.
3. **A panel of its own**, `crates/game/src/command_ui/interface/templates/<faction>/<object>.rs`: its `TEMPLATE`, its
   grid and menus, and its `InterfaceTemplate` variant, with its arms in `InterfaceTemplate::grid` and in the
   right-click (`command_ui/resolution.rs`). A unit or building of its panel that isn't in the game yet holds its slot
   (`PanelTemplate::planned`, `PanelCommand::Planned`, with the type it will require in `Planned::requires`), a
   producer's unit the design puts in a slot of its own is placed there (`PanelTemplate::trains`), and a command the
   design names after the object has its name there (`PanelTemplate::names`). A menu whose entries
   follow a design list in grid order takes the list from the sim's data (`sc_sim::defs::EJECT_MENU_ORDER`, and the
   ExpandMenu's `EXPAND_MENU`), so a new unit or expansion joins it there. A builder also needs its `Builder`
   (*Adding a faction*, step 7). Of M4's eight kinds, three have a module here: the Tunnel, the Agent, and the
   Hideout (the Production template with the design's letters); the Guard and the Grenadier take the BasicCombatUnit
   template, and the Optics Lab, the Chemical Plant and the Laser Turret have no commands (`PanelTemplate::NONE`; a
   site shows its Cancel, `SITE_GRID`).
4. **Where numbers go**:
   - the object's own numbers in its def, as fields, not in logic files;
   - a number a system uses for every kind in that system's tuning module (*Tunables*);
   - the client's feel numbers in `crates/game/src/tuning/`;
   - colours and drawing sizes with the drawing code (`object_render/`, `combat_effects/`, `command_ui/indicators/`).

   Each has a design citation, or "Placeholder (not in the design)".
5. **Tests**: its def's in its module; a scenario test of what it does in `crates/sim/tests/`; the client's, if it
   has a panel, beside the other panels' (`crates/game/tests/commands/`).
