//! Weapon Conversion — `factions/colonists/units/weapon_conversion.md`: "Sabre Legionnaire and Rifle Legionnaire are
//! two forms of the same underlying unit", each converting into the other (`crate::conversion`). Both forms' defs
//! carry the one [`LEGIONNAIRE_CONVERSION`].

use crate::defs::ObjectKind;
use crate::defs::units::ConversionDef;

/// The Legionnaire's conversion, on both of its forms.
pub(crate) const LEGIONNAIRE_CONVERSION: ConversionDef = ConversionDef {
    forms: [ObjectKind::SabreLegionnaire, ObjectKind::RifleLegionnaire],
    ticks: 4, // `> Convert Command`: "The unit converts in place over 4 frames"
};

/// The Legionnaire's conversion time, ticks, as its def gives it (`ConversionDef::ticks`): its name for the tests that
/// count the frames of a conversion.
pub const CONVERT_TICKS: u32 = LEGIONNAIRE_CONVERSION.ticks;
