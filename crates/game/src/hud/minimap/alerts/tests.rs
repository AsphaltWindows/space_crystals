//! The alerts as plain data: the throttle, an area at a time for 10 s; the latest alert; the ring as it ages; and a
//! new sim's start. Also the timing of their work a frame ([`alerts_perf`], ignored by default).

use std::hint::black_box;
use std::time::{Duration, Instant};

use sc_sim::SU_PER_GU;

use super::*;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const SPOT: Vec2 = Vec2::new(1000.0, 1000.0);

/// `at` moved `gu` grid units east.
fn east(at: Vec2, gu: f32) -> Vec2 {
    at + Vec2::X * gu * SU_PER_GU
}

/// "At most one ping per area every 10 s (placeholder; an area: within about 8 grid units of an earlier ping still in
/// its 10 s)": a hit 8 grid units from a ping in its 10 s, or right on it 159 ticks after it, long after it faded,
/// makes none; one a little farther does, and so does one on the spot 160 ticks (10 s) after it. Another player's
/// pings are their own.
#[test]
fn an_area_pings_once_every_ten_seconds() {
    let mut alerts = MinimapAlerts::default();
    assert!(alerts.alert(P1, SPOT, 100));
    assert!(!alerts.alert(P1, SPOT, 100), "the same tick, the same spot");
    assert!(
        !alerts.alert(P1, east(SPOT, PING_AREA_GU), 101),
        "8 grid units away: the same area"
    );
    assert!(!alerts.alert(P1, SPOT, 100 + PING_TICKS), "faded, still in its 10 s");
    assert!(!alerts.alert(P1, SPOT, 100 + PING_THROTTLE_TICKS - 1));
    assert!(alerts.alert(P2, SPOT, 101), "player 2's pings are their own");
    let farther = east(SPOT, -(PING_AREA_GU + 0.1));
    assert!(
        alerts.alert(P1, farther, 102),
        "a little more than 8 grid units away: another area"
    );
    assert!(alerts.alert(P1, SPOT, 100 + PING_THROTTLE_TICKS), "10 s on");
    assert_eq!(alerts.made, 4);
    let pings: Vec<_> = alerts.pings().iter().map(|p| (p.player, p.tick)).collect();
    assert_eq!(pings, [(P1, 100), (P2, 101), (P1, 102), (P1, 260)]);
}

/// The pings are kept for the throttle for 10 s, and forgotten then: they neither show nor throttle any more.
#[test]
fn a_ping_is_forgotten_after_ten_seconds() {
    let mut alerts = MinimapAlerts::default();
    alerts.alert(P1, SPOT, 10);
    alerts.alert(P1, east(SPOT, 20.0), 50);
    alerts.forget(10 + PING_THROTTLE_TICKS - 1);
    assert_eq!(alerts.pings().len(), 2);
    alerts.forget(10 + PING_THROTTLE_TICKS);
    assert_eq!(alerts.pings().iter().map(|p| p.tick).collect::<Vec<_>>(), [50]);
    assert_eq!(alerts.latest(P1), Some(east(SPOT, 20.0)), "the latest stays");
}

/// The brief's reading: the latest alert is the latest ping made, not the latest hit, which the throttle may keep from
/// pinging; each player has their own, and none before their first ping.
#[test]
fn the_latest_alert_is_the_latest_ping_made() {
    let mut alerts = MinimapAlerts::default();
    assert_eq!(alerts.latest(P1), None);
    alerts.alert(P1, SPOT, 10);
    assert_eq!(alerts.latest(P1), Some(SPOT));
    let far = east(SPOT, 30.0);
    alerts.alert(P1, far, 20);
    assert_eq!(alerts.latest(P1), Some(far));
    assert!(!alerts.alert(P1, east(SPOT, 1.0), 30), "throttled");
    assert_eq!(alerts.latest(P1), Some(far), "a throttled hit is no alert");
    assert_eq!(alerts.latest(P2), None);
}

/// "A ring that shrinks and fades over about 2 s": as large as it gets and opaque as it is made, smaller and clearer
/// every tick, gone at [`PING_TICKS`].
#[test]
fn the_ring_shrinks_and_fades_for_two_seconds() {
    assert_eq!(ring_at(0.0), Some((RING_START_PX, 1.0)));
    let rings: Vec<_> = (0..PING_TICKS).map(|t| ring_at(t as f32).expect("shown")).collect();
    for pair in rings.windows(2) {
        let [(size, alpha), (next_size, next_alpha)] = pair else {
            unreachable!()
        };
        assert!(next_size < size && next_alpha < alpha, "{pair:?}");
    }
    let (size, alpha) = ring_at(PING_TICKS as f32 - 0.5).expect("not yet gone");
    assert!(size > RING_END_PX && size < RING_END_PX + 1.0 && alpha < 0.02);
    assert_eq!(ring_at(PING_TICKS as f32), None);
    assert_eq!(ring_at(PING_TICKS as f32 * 3.0), None);
    assert_eq!(PING_TICKS, 32, "2 s at 16 ticks a second");
}

/// A new sim (a restart, a play-test) starts with no pings and no latest alert; the same sim keeps them.
#[test]
fn a_new_sim_has_no_alerts() {
    let mut alerts = MinimapAlerts::default();
    alerts.follow(3);
    alerts.alert(P1, SPOT, 10);
    alerts.follow(3);
    assert_eq!((alerts.pings().len(), alerts.latest(P1)), (1, Some(SPOT)));
    alerts.follow(4);
    assert_eq!((alerts.pings().len(), alerts.latest(P1), alerts.made), (0, None, 0));
}

/// Timings of the alerts' work a frame (slice M5-MM2; the brief: "a ping costs nothing noticeable"). Ignored by
/// default:
///
/// ```sh
/// cargo test -p space_crystals --release --lib alerts_perf -- --ignored --nocapture --test-threads=1
/// ```
///
/// Far beyond any game: for each of 480 ticks (30 s), [`HITS`] hits on the player's own objects off screen, spread
/// evenly at random over a 128 × 128 map, each through the throttle ([`MinimapAlerts::alert`]) against the pings it
/// keeps for 10 s, then the rings of the pings that show ([`ring_at`]): what `take_alerts` and `place_pings` do with
/// them, but the frame's events, which every consumer of them walks. It prints the mean, the 99th percentile and the
/// worst frame, and the most pings kept and shown. Timings are printed, never asserted.
#[test]
#[ignore = "perf: run in release, with --nocapture"]
fn alerts_perf() {
    const HITS: usize = 200;
    const MAP_SU: f32 = 128.0 * SU_PER_GU;
    // xorshift64*: the same hits every run.
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut random = move || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        (state.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 40) as f32 / (1u64 << 24) as f32
    };
    let mut alerts = MinimapAlerts::default();
    let (mut times, mut kept, mut shown) = (Vec::new(), 0, 0);
    for tick in 0..480 {
        let hits: Vec<Vec2> = (0..HITS).map(|_| Vec2::new(random(), random()) * MAP_SU).collect();
        let start = Instant::now();
        alerts.forget(tick);
        for &at in &hits {
            alerts.alert(P1, black_box(at), tick);
        }
        let rings = (alerts.pings().iter())
            .filter_map(|p| ring_at((tick - p.tick) as f32))
            .count();
        times.push(start.elapsed());
        kept = kept.max(alerts.pings().len());
        shown = shown.max(black_box(rings));
    }
    times.sort();
    let us = |d: Duration| d.as_secs_f64() * 1e6;
    let mean = us(times.iter().sum::<Duration>()) / times.len() as f64;
    let p99 = us(times[(times.len() * 99).div_ceil(100) - 1]);
    println!(
        "alerts, {HITS} hits a tick: mean {mean:.1} us, p99 {p99:.1} us, worst {:.1} us a frame; at most {kept} pings \
         kept, {shown} shown; {} made",
        us(*times.last().unwrap()),
        alerts.made
    );
}
