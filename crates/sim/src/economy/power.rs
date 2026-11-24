//! Power — `factions/gdo/overview.md > GDOResources > Power`: "Each GDO building has a static Power value —
//! positive for generators, negative for consumers. The player's total Power is the sum across all owned
//! buildings." When it is negative, the consumers work slower, at the speed ratio.
//!
//! Derived from the objects, never stored (plan decision T7): a structure counts from the tick it is placed until
//! the tick it is destroyed (design change 0089, point 4), simply because it is among the objects in between. The
//! tick of its placement counts and the tick of its destruction doesn't: the economy phase runs after the tick's
//! commands and deaths (plan decision T5, which 0089 confirms).

use std::collections::BTreeMap;

use crate::defs::ObjectKind;
use crate::ids::PlayerId;
use crate::object::Object;

/// 100% in the fixed point of speed ratios and progress (plan decision T6): a structure working at full speed
/// advances its progress by `RATIO_ONE` a tick, one at a ratio `r` by `r`.
pub const RATIO_ONE: u32 = 1 << 16;

/// A player's power: the two totals the HUD shows, consumed / generated (design change 0090: "used / available"),
/// and the speed ratio that follows from them.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PowerSummary {
    /// "available": the Power of the player's structures with a positive Power, added up.
    pub generated: u32,
    /// "used": the Power drawn by the player's structures with a negative Power, added up.
    pub consumed: u32,
}

impl PowerSummary {
    /// Counts a structure of Power `power`.
    fn add(&mut self, power: i32) {
        if power > 0 {
            self.generated = self.generated.saturating_add(power.unsigned_abs());
        } else {
            self.consumed = self.consumed.saturating_add(power.unsigned_abs());
        }
    }

    /// `> Power > Speed ratio`: "generated power ÷ consumed power, capped at 100%", in [`RATIO_ONE`]s: `RATIO_ONE`
    /// without consumers, 0 with nothing generated. Rounded up (plan decision T6), so that a ratio given exactly
    /// takes the exact number of ticks: a 48-tick cycle at 30/51 ends in tick 82 (48 × 51/30 = 81.6), five of them
    /// in tick 408.
    pub fn ratio(&self) -> u32 {
        if self.consumed == 0 {
            return RATIO_ONE;
        }
        let ratio = (u64::from(self.generated) * u64::from(RATIO_ONE)).div_ceil(u64::from(self.consumed));
        ratio.min(u64::from(RATIO_ONE)) as u32
    }

    /// Whether the player uses more than it generates: negative power, which the HUD shows in red (design change
    /// 0090) and which slows the consumers.
    pub fn is_negative(&self) -> bool {
        self.consumed > self.generated
    }

    /// The speed at which a structure of `kind` works for this player, in [`RATIO_ONE`]s (design change 0089):
    /// a consumer, a structure of negative Power (the Extraction Facility's plate construction, the Barracks'
    /// production), at the [`PowerSummary::ratio`]; any other at full speed, "power producers are never slowed"
    /// (the Deployment Center builds at full speed at negative power). The plates' mining follows its own rule
    /// (`> ExtractionPlate > PowerPenalty`), which is the ratio as well.
    pub fn speed(&self, kind: ObjectKind) -> u32 {
        match kind.def().structure.as_ref() {
            Some(structure) if structure.power < 0 => self.ratio(),
            _ => RATIO_ONE,
        }
    }
}

/// `player`'s power: the structures among `objects` that the player owns.
pub(crate) fn summary<'a>(objects: impl IntoIterator<Item = &'a Object>, player: PlayerId) -> PowerSummary {
    let mut summary = PowerSummary::default();
    for o in objects {
        if o.owner == Some(player)
            && let Some(structure) = &o.def().structure
        {
            summary.add(structure.power);
        }
    }
    summary
}

/// Every owner's power, in one pass over `objects`. A player who owns no structure isn't listed: the default,
/// nothing generated and nothing consumed, is theirs.
pub(crate) fn summaries<'a>(objects: impl IntoIterator<Item = &'a Object>) -> BTreeMap<PlayerId, PowerSummary> {
    let mut summaries: BTreeMap<PlayerId, PowerSummary> = BTreeMap::new();
    for o in objects {
        if let (Some(owner), Some(structure)) = (o.owner, &o.def().structure) {
            summaries.entry(owner).or_default().add(structure.power);
        }
    }
    summaries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn power(generated: u32, consumed: u32) -> PowerSummary {
        PowerSummary { generated, consumed }
    }

    /// `> Power > Speed ratio`: generated ÷ consumed, capped at 100%, rounded up in 1/65536 (plan decision T6).
    #[test]
    fn the_ratio_is_generated_over_consumed_capped_and_rounded_up() {
        // The opening baseline's all-in Barracks: 30 generated, 51 consumed, ≈ 59%.
        assert_eq!(power(30, 51).ratio(), 38_551);
        assert_eq!(u64::from(power(30, 51).ratio()) * 51 / 30, 65_536, "≈ 30/51");
        assert_eq!(power(30, 21).ratio(), RATIO_ONE, "capped at 100%: the starting base");
        assert_eq!(power(30, 30).ratio(), RATIO_ONE);
        assert_eq!(power(1, 2).ratio(), RATIO_ONE / 2);
        assert_eq!(power(0, 0).ratio(), RATIO_ONE, "no consumers");
        assert_eq!(power(0, 6).ratio(), 0, "nothing generated");
        assert_eq!(power(1, u32::MAX).ratio(), 1, "rounded up");
        assert_eq!(power(u32::MAX, 1).ratio(), RATIO_ONE);
    }

    /// Design change 0089: producers are never slowed, consumers are; 0090: negative power is consumption over
    /// generation.
    #[test]
    fn only_consumers_are_slowed() {
        let short = power(30, 51);
        assert!(short.is_negative() && !power(30, 30).is_negative() && !power(30, 21).is_negative());
        for (kind, slowed) in [
            (ObjectKind::DeploymentCenter, false),
            (ObjectKind::Generator, false),
            (ObjectKind::ExtractionFacility, true),
            (ObjectKind::Barracks, true),
            (ObjectKind::ExtractionPlate, true),
            (ObjectKind::Peacekeeper, false),
        ] {
            let expected = if slowed { short.ratio() } else { RATIO_ONE };
            assert_eq!(short.speed(kind), expected, "{kind:?}");
            assert_eq!(power(30, 21).speed(kind), RATIO_ONE, "{kind:?} at positive power");
        }
    }

    #[test]
    fn structures_add_up_by_sign() {
        let mut summary = PowerSummary::default();
        for p in [30, -15, -3, -3, 0, 20] {
            summary.add(p);
        }
        assert_eq!(summary, power(50, 21));
        summary.add(i32::MIN);
        summary.add(i32::MIN);
        summary.add(i32::MIN);
        assert_eq!(summary.consumed, u32::MAX, "saturates");
    }
}
