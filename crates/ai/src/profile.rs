//! How a computer player plays (`core/game_rules.md > Computer players (playtest tool)`: "Their behavior (profiles
//! such as standard, rush and passive; build orders) is the Director's placeholder"). **Placeholder (Q0147 point 2)**:
//! the profiles and what each does.

use std::fmt;
use std::str::FromStr;

/// A computer player's profile, named on the command line (`--ai 2:passive`, `--ai 2:rush`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Profile {
    /// Its faction's opening, its economy, an army trained without pause, and waves of attacks on what it knows of
    /// the enemy; its army at home defends its buildings.
    #[default]
    Standard,
    /// Its faction's opening cut short toward an early army, and an early, small first wave; then it plays and
    /// reinforces as standard does (Q0147 point 2: "a small early army and an early first attack"; slice A2d). Each
    /// faction's module plays its rush opening ([`Profile::rushes`]) and sizes its first wave (`FactionAi::waves`).
    Rush,
    /// The same economy and army, but it never attacks: its army waits at its staging point and defends (Q0147 point 2:
    /// "builds its economy and army but never attacks, for racing economies").
    Passive,
}

impl Profile {
    /// Every profile, in the design's order ("standard, rush and passive"): what the command line accepts, and lists in
    /// its error.
    pub const ALL: &'static [Profile] = &[Profile::Standard, Profile::Rush, Profile::Passive];

    /// Its name on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Profile::Standard => "standard",
            Profile::Rush => "rush",
            Profile::Passive => "passive",
        }
    }

    /// Whether its army attacks in waves ([`Profile::Passive`]'s never does).
    pub fn attacks(self) -> bool {
        match self {
            Profile::Standard | Profile::Rush => true,
            Profile::Passive => false,
        }
    }

    /// Whether it plays its faction's rush opening and sends an early, small first wave ([`Profile::Rush`]).
    pub fn rushes(self) -> bool {
        self == Profile::Rush
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Profile {
    type Err = String;

    /// A profile by its name, ignoring case; the error lists the names there are.
    fn from_str(s: &str) -> Result<Self, String> {
        Profile::ALL
            .iter()
            .copied()
            .find(|p| p.name().eq_ignore_ascii_case(s.trim()))
            .ok_or_else(|| {
                let names: Vec<&str> = Profile::ALL.iter().map(|p| p.name()).collect();
                format!(
                    "unknown computer player profile {s:?} (expected one of: {})",
                    names.join(", ")
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_parse_by_name_and_the_error_lists_them() {
        assert_eq!("standard".parse(), Ok(Profile::Standard));
        assert_eq!(" Passive ".parse(), Ok(Profile::Passive));
        assert_eq!("RUSH".parse(), Ok(Profile::Rush));
        assert_eq!(Profile::default(), Profile::Standard);
        let err = "blitz".parse::<Profile>().unwrap_err();
        assert!(
            err.contains("\"blitz\"") && err.contains("standard, rush, passive"),
            "{err}"
        );
        for &p in Profile::ALL {
            assert_eq!(p.name().parse(), Ok(p));
            assert_eq!(p.to_string(), p.name());
        }
        assert!(Profile::Standard.attacks() && Profile::Rush.attacks() && !Profile::Passive.attacks());
        assert!(Profile::Rush.rushes() && !Profile::Standard.rushes() && !Profile::Passive.rushes());
    }
}
