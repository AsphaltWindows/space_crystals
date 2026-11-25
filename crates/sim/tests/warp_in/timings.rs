//! The Warp In's timings (the plan's acceptance, "Timings"; `ARCHITECTURE.md > Economy > Recipe timing`):
//! each building's cost paid as the command applies, its WarpInTime, and the building at work from the next tick.

use super::*;

/// The plan's acceptance, "Timings" (`ARCHITECTURE.md > Economy > Recipe timing`): for each building, the cost is
/// paid when the command applies; ordered in tick 1, the Warp In is complete at the end of tick 1 + N - 1 = N (its
/// WarpInTime), and the building works from tick N + 1: a portal refuses Train until then and takes it in that
/// tick. From its first frame the object is the building, still transforming, and stays the same object.
#[test]
fn warp_ins_take_their_warp_in_time_and_work_from_the_next_tick() {
    for Building {
        kind,
        origin: (x, y),
        cost,
        frames,
        unit,
    } in BUILDINGS
    {
        let mut t = base().start();
        t.debug_at(1, fund(P1, 1000)).at(1, P1, construction(&[1], warp(kind)));
        t.step();
        assert_eq!(bank(&t, P1), 1000 - cost, "{kind:?}: paid at once");
        assert!(
            t.sim().events().contains(&SimEvent::KindChanged {
                id: id(1),
                from: Beacon,
                to: kind
            }),
            "{kind:?}"
        );
        let o = object(&t, 1);
        assert_eq!(o.kind, kind);
        assert_eq!(
            o.placement.map(|p| (p.origin, p.w)),
            Some((at(x, y), kind.def().tile_size().unwrap().0))
        );
        let w = transform(&t, 1);
        assert_eq!((w.from, w.paid, w.total, w.frames_done()), (Beacon, cost, frames, 1));
        assert_eq!(w.from_placement.map(|p| p.origin), Some(at(8, 6)));
        let trained = unit.map(|unit| PlayerCommand {
            player: P1,
            command: train(&[1], unit),
        });
        t.run_until(frames - 1);
        assert!(
            object(&t, 1).is_transforming(),
            "{kind:?}: still at the end of tick {}",
            frames - 1
        );
        assert_eq!(completions(&t), []);
        // Design change 0116: "Production, drop-off, research and abilities start at completion"; "no rally can be
        // set".
        let rally = PlayerCommand {
            player: P1,
            command: Command::Production {
                objects: vec![id(1)],
                command: ProductionCommand::SetRally(Target::Point(at(3, 12).center())),
            },
        };
        assert_eq!(
            t.sim().validate(&rally),
            Err(Reject::NoProducers),
            "{kind:?}: no rally yet"
        );
        if let Some(cmd) = &trained {
            assert_eq!(
                t.sim().validate(cmd),
                Err(Reject::NoProducers),
                "{kind:?} doesn't work yet"
            );
        }
        t.step();
        assert_eq!(
            completions(&t),
            [(frames, 1, kind)],
            "{kind:?}: complete at the end of tick {frames}"
        );
        assert!(!object(&t, 1).is_transforming());
        assert_eq!(bank(&t, P1), 1000 - cost);
        if let Some(cmd) = trained {
            assert_eq!(
                t.sim().validate(&cmd),
                Ok(()),
                "{kind:?} works from tick {}",
                frames + 1
            );
            next(&mut t, cmd.command);
            assert!(
                t.sim()
                    .events()
                    .iter()
                    .any(|e| matches!(e, SimEvent::ProductionQueued { producer, .. } if *producer == id(1))),
                "{kind:?}"
            );
        }
    }
}
