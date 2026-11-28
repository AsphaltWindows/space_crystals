"""Fill an M6 brief's 'Running beside you' and 'Landed since' at launch.
Args: target (SP|SL|GM|GP|GB|S6b|SC|ST1|GL|GS1), brief path, perf hash, running list (comma-separated, e.g. S6a,GH2,SB).
Add a merged slice to MERGED (and drop it from the running list) before using this after a merge."""
import sys
target, path, perf = sys.argv[1], sys.argv[2], sys.argv[3]
running = [r for r in sys.argv[4].split(",") if r and r != target]
s = open(path).read()
def rep(old, new):
    global s
    assert s.count(old) == 1, old[:60]
    s = s.replace(old, new)

RUN = {
 "S6a": "**M6-S6a, an answer sweep:** a braking crusher crushes (0453); the MinRange back-off and repositioning to aim (0454, 0460); the client's Attack on a target some weapons can't hit (0461); the client's QA mode. It owns the attack behaviours (`orders/engage.rs`, the attack orders, `orders/attack_ground/`).",
 "GH2": "**M6-GH2, landing and rearming:** the GDO's aircraft land on Airfield slots and rearm, the Hornet's missile burst (0458), and a per-object targeting domain (a landed aircraft is a Ground target).",
 "SB": "**M6-SB, the Vault and the Sludge:** the Vault's turn in `network/services.rs`, the Shadebringer's tanks and toggles, and the sludge patches (`Sim` tag 4).",
 "SP": "**M6-SP, the Orbital Command and the Prism:** the Prism's launch through the air doors, Thrust, and `UnitDef::satellite`.",
 "SL": "**M6-SL, the Laser Battery:** the battery and surges, Amplify, and the Battery's deploy pair on GA's model, with a fix to GA's gate for a Shift-queued order behind a queued switch.",
 "GM": "**M6-GM, the Twister's mines:** mines as deployed units, queued buying in `stock.rs`, and the mine's trigger and blast.",
 "GP": "**M6-GP, the Prospector:** GA's deploy model with a site: a unit that walks to a site and becomes the Outpost, a structure.",
 "GB": "**M6-GB, the Field Engineer's Bunker and First Aid Tent:** a third `BuilderGoes` arm, `WorkerDef::crystals` as an `Option`, and the Tent's heal.",
 "ST1": "**M6-ST1, the Subterranean Transport above ground:** the carrier (`UnitDef::carries`, `Place::Carrier`), boarding and Unload All, and `UnitDef::domain` in `combat::kind_domain`.",
 "GL": "**M6-GL, Propulsion Labs and the Dragonfly:** the glider's vertical landing on GH2's slots.",
 "SC": "**M6-SC, the Shadebringer's Cover:** smoke in `Sim::patches` (Sim tag 4), the game's first sight blocker, in vision and the Scan.",
 "GS1": "**M6-GS1, the supply loop:** the Supply Tower, the Chopper's supply runs and schedule, and the GDO's starting Supply of 1 (0436), which moves every GDO golden.",
 "S6b": "**M6-S6b, an answer sweep:** Metabolism and Inspire as percentages, and one rule for attack-speed frames (0455); the air units' SeparationRadius (0456, 0457); ejected vehicles driving to a stop point (0459: the vehicle arm of `sim/network/ejects.rs`, and GF's `park.rs`); the Shroud cloud's soft edge in the client.",
}
EXTRA = {
 ("SP", "GH2"): " Your launch is a rise at the air doors, not a landing: nothing of GH2's.",
 ("SP", "SB"): " Your `NoTransit::OneWay` and the air lane are SD's network code beside SB's services: keep to your arm.",
 ("SP", "SL"): " SR needs both of you: your notes on the Prism go in your report.",
 ("SP", "SC"): " Its smoke reads your `UnitDef::satellite` (satellites see through it). If it adds the same field before you merge, whoever merges later keeps one.",
 ("SC", "SP"): " It adds `UnitDef::satellite` (false for every unit but the Prism), which your smoke reads. If you need it before SP merges, add the same field under the same name, false for every unit, with a test; whoever merges later keeps one, and SP sets it on the Prism.",
 ("SL", "S6a"): " The Battery is tracked, so it crushes, by 0453's trigger if S6a merges first. A mobile Battery with a target outside its 30° turret repositions by 0454 once S6a merges: write your tests so that they don't depend on which merges first.",
 ("SL", "GH2"): " A landed aircraft is a Ground target, so a deployed Battery (air only) can't hit a landed Hornet, and a mobile one (ground only) can.",
 ("SL", "SP"): " SR needs both of you: your notes on the deployed Battery go in your report.",
 ("SL", "GP"): " GP also builds on GA's deploy model. If you both touch `vehicle_deploy::can_take` or the gate's deploy arm, keep each change small; whoever merges later rebases.",
 ("GP", "SL"): " SL also builds on GA's deploy model, and fixes GA's gate for a Shift-queued order behind a queued switch. If you both touch `vehicle_deploy::can_take` or the gate's deploy arm, keep each change small; whoever merges later rebases.",
 ("S6b", "SB"): " Its Vault's pass is in `network/services.rs`, and its toggles switch off where a unit enters the network; your arm is the vehicle eject. Its tests give each ejected Shadebringer an order or a rally point, so they don't depend on your stop point.",
 ("S6b", "S6a"): " Your attack-speed function is new, in `combat`; S6a's behaviours read weapon timings but don't change them.",
 ("GM", "GB"): " GB makes `WorkerDef::crystals` an `Option`; nothing of yours.",
 ("GM", "GS1"): " If it merges first, re-bless any new GDO golden of yours once more, for the starting Supply alone.",
 ("GP", "GS1"): " If it merges first, re-bless any new GDO golden of yours once more, for the starting Supply alone.",
 ("GB", "GS1"): " If it merges first, re-bless any new GDO golden of yours once more, for the starting Supply alone.",
 ("GL", "GS1"): " You both build on GH2's landing: GS1 adds the Tower's and a station's spots, you the glider's arm. If it merges first, re-bless `gdo_dragonflies` once more, for the starting Supply alone.",
 ("GS1", "GL"): " You both build on GH2's landing: GL adds the glider's arm, you the Tower's and a station's spots. Keep to your arms.",
 ("GS1", "ST1"): " GS2 later reuses ST1's carrier for the Chopper's passengers: build the cargo as one kind at a time, as your brief says.",
 ("ST1", "GS1"): " GS2 later puts units in the Chopper beside GS1's Supplies, on your carrier.",
 ("ST1", "S6a"): " It changes `crush.rs` for braking crushers (0453); your crushing is in the def (your brief).",
 ("ST1", "SB"): " It adds the Vault's pass, and 0463 (a finished turn always buffs), to `network/services.rs`: read it, don't change it.",
 ("ST1", "GM"): " Its mine trigger is one of T94's readers: whoever merges later makes it read the kind's domain.",
 ("GM", "ST1"): " Your trigger is one of T94's readers: whoever merges later makes it read the kind's domain (`combat::kind_domain`).",
}
MERGED = [l.rstrip("\n") for l in open(__file__.rsplit("/", 1)[0] + "/merged.txt") if l.strip()]
OWN = {
 "SP": ["**Your tags in the other namespaces** (`artifacts/director/hash-tags.md`): **Behaviour tag 17** and **Mover tag 12** (the unfold after the rise).",
        "**`UnitDef::satellite`** (your Code section): M6-SC reads it later for smoke (\"satellites see through it\"), so name it for what it is, not for the buff."],
 "SL": ["**GA's notes for you:** a `vehicle_deploy` row on each kind of the pair is enough, and `templates.rs` gives R to any type with a row. `defs/tests.rs` asserts every deploy is a two-way pair within one faction: yours is.",
        "**Your tags in the other namespaces** (`artifacts/director/hash-tags.md`): **Behaviour tag 27**, if the surge or Amplify needs one."],
 "GM": ["**Your tags in the other namespaces** (`artifacts/director/hash-tags.md`): **Behaviour tag 15**, if laying a mine needs one."],
 "SC": ["**Your tags in the other namespaces** (`artifacts/director/hash-tags.md`): **Behaviour tag 29**, if needed. Smoke shares `Sim::patches` (Sim tag 4), so Sim tag 5 stays free."],
 "GP": ["**GA's notes for you:** its site arm is marked in `Sim::complete_vehicle_deploy`. `defs/tests.rs` asserts every deploy is a two-way pair within one faction; the Prospector's is one-way (unit to structure), so relax that test for it, by name.",
        "**Your tags in the other namespaces** (`artifacts/director/hash-tags.md`): **Behaviour tag 19**, if the walk to the site needs one."],
 "GB": ["**Your tags in the other namespaces** (`artifacts/director/hash-tags.md`): **Behaviour tag 18**, if the Engineer's build needs one."],
 "ST1": ["**Your tags in the other namespaces** (`artifacts/director/hash-tags.md`): **Place tag 3** (`Place::Carrier`), and **Behaviour tag 16**, if boarding needs one.",
         "**Your carrier is reused twice:** M6-GS2 makes the Supply Chopper carry 4 infantry or 1 vehicle (0442), and M6-CP the Cults' Pilgrim 16 infantry, loading only while landed (0462). Shape `UnitDef::carries` and `Place::Carrier` so that a carrier that flies, and one with a vehicle place, need no second carrier. Put a note for each in your report."],
 "GL": ["**GH2's notes for you:** reuse `landing::claims`, `slots` and `Landing`; a glider needs its own arrival check. `Landing` is hashed under `AircraftState`'s sub-tag 1 (Object tag 14): a new field of yours there takes sub-tag 2.",
        "**Your tags in the other namespaces** (`artifacts/director/hash-tags.md`): **Behaviour tag 20** and **Mover tag 13**, if needed.",
        "**M6-CP (the Cults' Pilgrim, 0462) comes later:** a glider that lands along an Airstrip's runway. Put a note in your report on what of your glider arm it can reuse."],
 "GS1": ["**GH2's notes for you:** reuse `landing::claims`, `slots` and `Landing`, with the Tower's spot and a station's beside the Airfield's slots. `Landing` is hashed under `AircraftState`'s sub-tag 1 (Object tag 14).",
         "**Your tags in the other namespaces** (`artifacts/director/hash-tags.md`): **Behaviour tag 21** and **Mover tag 16**, if needed.",
         "**The goldens 0436 moves:** every golden with a GDO player and a starting bank, `gdo_air` (GH2's) among them. List them in your report."],
 "S6b": ["**SN changed `sim/network/ejects.rs`** (read its Built entry in `artifacts/director/plan.md`): both lanes now call `Sim::leave_network`.",
         "**CM's linger** (24 ticks, `Member::until`) keeps a cloud after its channel ends; your part 4's `Area::of` fix applies to lingering clouds too.",
         "**Your tags in the other namespaces** (`artifacts/director/hash-tags.md`): **Behaviour tag 30**, if needed."],
}
lines = ["- " + RUN[r] + EXTRA.get((target, r), "") for r in running]
if not lines:
    lines = ["- No other slice is running."]
rep("## Running beside you\nFilled in at launch.\n",
    "## Running beside you\n" + "\n".join(lines) +
    "\n\nThe lines you all share: the controls doc; the golden list and its counts; `ARCHITECTURE.md`. Whoever merges later rebases them, and checks the counts by hand (m6-common).\n")
landed = "\n".join("- " + m for m in MERGED + OWN[target]) + f"\n- **Master's perf binary** for an A/B: `target/perf/{perf}`, copied into your worktree with `target/`.\n"
rep("## Landed since the plan (filled in at launch)\nTBD.\n", "## Landed since the plan (filled in at launch)\n" + landed)
open(path, "w").write(s)
print("ok", target, "beside:", ",".join(running) or "none")
