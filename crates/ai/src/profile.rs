//! How a computer player plays (`core/game_rules.md > Computer players (playtest tool)`: "Their behavior (profiles
//! such as standard, rush and passive; build orders) is the Director's placeholder"). **Placeholder (Q0147 point 2)**:
//! the profiles and what each does.

use std::fmt;
use std::str::FromStr;

/// A computer player's profile, named on the command line (`--ai 2:passive`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Profile {
    /// Its faction's opening, its economy, an army trained without pause, and waves of attacks on what it knows of
    /// the enemy; its army at home defends its buildings.
    #[default]
    Standard,
    /// The same economy and army, but it never attacks: its army waits at its staging point and defends (Q0147 point 2:
    /// "builds its economy and army but never attacks, for racing economies").
    Passive,
}

impl Profile {
    /// Every profile, in declaration order: what the command line accepts, and lists in its error.
    pub const ALL: &'static [Profile] = &[Profile::Standard, Profile::Passive];

    /// Its name on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Profile::Standard => "standard",
            Profile::Passive => "passive",
        }
    }

    /// Whether its army attacks in waves ([`Profile::Passive`]'s never does).
    pub fn attacks(self) -> bool {
        match self {
            Profile::Standard => true,
            Profile::Passive => false,
        }
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
        assert_eq!(Profile::default(), Profile::Standard);
        let err = "rush".parse::<Profile>().unwrap_err();
        assert!(err.contains("\"rush\"") && err.contains("standard, passive"), "{err}");
        for &p in Profile::ALL {
            assert_eq!(p.name().parse(), Ok(p));
            assert_eq!(p.to_string(), p.name());
        }
        assert!(Profile::Standard.attacks() && !Profile::Passive.attacks());
    }
}
