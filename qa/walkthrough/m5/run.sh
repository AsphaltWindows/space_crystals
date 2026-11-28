#!/usr/bin/env bash
# The M5 walkthrough (slice M5-Mb, `artifacts/director/m5-plan.md > 7. QA and the playtest`: "Scripted walkthrough
# (M). Screenshots at fixed ticks, with --input-script"; the addendum's and the later slices' additions).
#
#     qa/walkthrough/m5/run.sh [out-dir]       (from the repository's root; out-dir defaults to target/walkthrough-m5)
#
# Each scene is a QA run of the game (ARCHITECTURE.md > QA mode) with its --commands and --input-script files from
# this directory; every one is on dev_skirmish with player 1's view. Look at every PNG it writes. Every scene runs
# --fast but a5, whose tuning file the wall clock times (about 12 seconds). (Slice M6-S6a: a1 and a2 ran in real time
# too, since under --fast a tapped click right before a long gap in the script was read only once the sim had run on
# to the next stop, W then A then a click placing no Armory in time; the sim now waits a frame at each step's tick,
# `qa::input_script`.)
#
# a1_economy (the Cults' start, --fast; 600 Space Crystals by a debug command at tick 1):
#   01-start           tick 0: Unit Control 6/20 in the top bar
#   02-recruit-out     the Hall's seventh Recruit out of its west door (it starts at once, out at tick 192)
#   03-at-storage      three Recruits right-clicked onto a patch near the Storage, gathering
#   04-site-10pct      four Recruits' group Build of an Armory: the site at 10% (48 of 480 frames), its builders round it
#   05-site-selected   the site selected: X Cancel and C Rally before it is finished
#   06-shift-chain     the eighth Recruit: W, W, then three Storages placed with Shift held, the fourth ghost on the cursor
#   07a-site, 07b-cancelled   the first Storage's site selected, then X: the site gone, its refund in the bank
# a2_armory (an Armory spawned at tick 1, 1000 Space Crystals, --fast):
#   08-entering        the six Recruits right-clicked onto it, going in at side B (the right)
#   09-militant-at-c   Q, Q twice: a Militant out of side C (the left)
#   10-w-greyed        the Militant selected and W pressed: Reverse greyed, "Not available for this unit"
#   11-eject           the Armory's A (Eject All): the Recruits left inside streaming out of side C
#   12a-before, 12b-evacuated   back in, a Militant queued, the Armory removed (DebugCommand::Remove): all come out
#                      in its footprint, the Militant's 20 crystals refunded
#   14-claims          the Hall's eighth Recruit placing a second Recruitment Hall: W, Q, the claims overlay
# a3_cap (--fast):     13-cap-hall-idle: Unit Control 20/20, the Hall selected, training nothing
# a4_defeat (--fast):  15b-defeat: the Hall removed, eight GDO Peacekeepers destroy the Storage: DEFEAT
# a5_tuning (real time, --tuning on an empty directory; this script writes the Militant's max_hp = 100 into it
#   about 5 s in, which the game's poll of the directory, every 0.5 s of real time, finds between the two
#   screenshots only in real time): 16a-before (HP 40 / 40), 16b-after (HP 100 / 100, "Tuned: 1 file" in the top bar)
# b_tech (--fast; 5000 crystals and six debug-spawned Recruits, so Unit Control shows above 20: orphans count in the
#   display only, design change Q0101): both branches by Build orders, Armory, Chop Shop, Airstrip; Altar, Temple,
#   Shrine, Cathedral
#   b1-base            every building of both branches; a Seeker, a Sparrow in the air
#   b2-seeker-sparrow  the Seeker out of the Chop Shop, the Sparrow on the runway's B end
#   b7a-damaged, b7b-repair, b7c-repair   six GDO Peacekeepers damage the Storage; a Recruit repairs it
#   b4-shroud          a Disciple from the Cathedral channelling Shroud
#   b5-energy-regen    the Shrine (Recruits inside) casting Energy regeneration on the Disciple: 6 / 200 left
#   b6-health-regen    the Shrine casting Health regeneration on the Hall's Recruits
# c_fight (--fast, auto-targeting off, every attack ordered), east of the lake:
#   c1-inspire         a Prayer-mobile's Inspire on three Militants
#   c2-missiles        Rocketmen's missiles in flight at the Hand of Heaven; the Seeker firing
#   c3-suppress        the other Prayer-mobile's Suppress on four Peacekeepers; the Hand of Heaven's shell in the air
#   c4-burst           a Rocketman's missile bursting on the Hand of Heaven
# d_minimap (--fast):  d1-ping (a Recruit hit out of view: the red ring on the minimap), d2-space (Space: the camera on
#   it), d3-menu (F10: the playtest menu)
# d_agent (--fast, the Syndicate): d4-damaged, d5-agent-repair: the Agent repairs its Tunnel, 384 to 579 HP
# e_rush (--fast, --ai 2:rush): e1-rush: the GDO's first wave at the Cults' Hall
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
out="${1:-target/walkthrough-m5}"
mkdir -p "$out"
cargo build -p space_crystals
game=target/debug/space_crystals

shots() { # scene, then tick:label pairs
    local scene="$1"
    shift
    for s in "$@"; do printf -- '--screenshot\n%s:%s/%s-%s.png\n' "${s%%:*}" "$out" "$scene" "${s#*:}"; done
}

run() { # scene, fast or real, the game's flags, then -- and the screenshots
    local scene="$1" pace="$2"
    shift 2
    local flags=()
    while [ "$1" != "--" ]; do flags+=("$1"); shift; done
    shift
    [ "$pace" = fast ] && flags+=(--fast)
    [ -f "$here/$scene.commands.json" ] && flags+=(--commands "$here/$scene.commands.json")
    [ -f "$here/$scene.input.json" ] && flags+=(--input-script "$here/$scene.input.json")
    mapfile -t screenshots < <(shots "$scene" "$@")
    echo "== $scene"
    "$game" "${flags[@]}" "${screenshots[@]}" > "$out/$scene.log" 2>&1
}

run a1_economy fast --factions cults,gdo -- 0:01-start 193:02-recruit-out 100:03-at-storage 355:04-site-10pct \
    360:05-site-selected 447:06-shift-chain 518:07a-site 522:07b-cancelled
run a2_armory fast --factions cults,gdo -- 220:08-entering 302:09-militant-at-c 349:10-w-greyed 366:11-eject \
    560:12a-before 561:12b-evacuated 610:14-claims
run a3_cap fast --factions cults,gdo -- 2884:13-cap-hall-idle
run a4_defeat fast --factions cults,gdo -- 70:15b-defeat

tuning="$out/tuning_live"
rm -rf "$tuning"
mkdir -p "$tuning"
(sleep 5 && mkdir -p "$tuning/defs/cults/units" && echo '{"max_hp": 100.0}' > "$tuning/defs/cults/units/militant.json") &
run a5_tuning real --factions cults,gdo --tuning "$tuning" -- 32:16a-before 192:16b-after
wait

run b_tech fast --factions cults,gdo -- 962:b2-seeker-sparrow 1000:b1-base 1060:b7a-damaged 1160:b7b-repair \
    1220:b7c-repair 1405:b4-shroud 1424:b5-energy-regen 2074:b6-health-regen
run c_fight fast --factions cults,gdo -- 823:c1-inspire 1590:c2-missiles 1622:c3-suppress 1632:c4-burst
run d_minimap fast --factions cults,gdo -- 112:d1-ping 134:d2-space 140:d3-menu
run d_agent fast --factions syndicate,gdo -- 74:d4-damaged 160:d5-agent-repair
run e_rush fast --factions cults,gdo --ai 2:rush -- 2400:e1-rush
echo "the screenshots are in $out"
