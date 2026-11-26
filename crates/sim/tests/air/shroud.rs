//! Shroud and air units (`super`): `factions/cults/units/disciple.md > Spells > Shroud`: "It affects every ground
//! object under it (user, 2026-11-26): **ground units and buildings**, whoever owns them, **not air units**. **All
//! anti-ground damage** landing on objects under the cloud, single-target and area alike, is **absorbed and
//! redistributed** over the connected cloud" (`combat::damage::absorb`, slice M5-N4b: the hits a Ground area attack
//! would make, which no air unit takes).

use sc_sim::shroud::Cloud;

use super::*;

/// "A glider under a cloud, hit by an anti-air attack, takes the damage" (slice M5-H1's brief): player 1's Disciple (1)
/// channels Shroud, its cloud 3 grid units round it, and player 1's Sparrow (2) circles over it on Hold, always under
/// the cloud by its centre. Player 2's Militant (3) on Hold 300 su east, outside the cloud, shoots it whenever it comes
/// into range (its attack is Universal): each hit lands in full, 4, at once, and nothing goes into the cloud's buffer.
#[test]
fn a_glider_under_a_cloud_takes_an_anti_air_attack_s_hits_in_full() {
    let disciple = Vec2::new(400.0, 400.0);
    let mut t = flat(30, 12)
        .spawn(ObjectKind::Disciple, Some(P1), SpawnAt::Point(disciple))
        .spawn(SPARROW, Some(P1), SpawnAt::Point(disciple))
        .spawn(ObjectKind::Militant, Some(P2), point(700.0, 400.0))
        .start();
    order(&mut t, 1, P1, &[1], OrderKind::Shroud);
    order(&mut t, 1, P1, &[2], OrderKind::Hold);
    order(&mut t, 1, P2, &[3], OrderKind::Hold);
    let mut seen = 0;
    let mut landed = Vec::new();
    for _ in 0..16 * 20 {
        t.step();
        let clouds: Vec<&Cloud> = t.sim().clouds().list().iter().collect();
        assert_eq!(clouds.len(), 1, "the Disciple's cloud");
        assert_eq!(clouds[0].buffer, 0.0, "nothing absorbed");
        assert!(t.pos(2).distance(disciple) < 192.0, "under the cloud: {}", t.pos(2));
        let shots = hits(&t);
        landed.extend(
            shots[seen..]
                .iter()
                .filter(|&&(_, attacker, ..)| attacker == 3)
                .copied(),
        );
        seen = shots.len();
        if landed.len() >= 3 {
            break;
        }
    }
    assert!(landed.len() >= 3, "{landed:?}");
    for &(_, _, target, damage) in &landed {
        assert_eq!((target, damage), (2, 4.0), "{landed:?}");
    }
    let hp = t.sim().object(ObjectId(2)).and_then(|o| o.hp).expect("alive");
    assert_eq!(hp, 50.0 - 4.0 * landed.len() as f32);
}
