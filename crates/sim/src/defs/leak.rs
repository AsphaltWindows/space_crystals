//! Reading the defs' `&'static` data back (`#[serde(deserialize_with)]`): the defs are plain data that serialize as
//! they are, and a def read from a file lives as long as the game, as the built-in ones do (`ObjectKind::def` hands out
//! `&'static ObjectDef`), so its name and its lists are leaked once, when it is read. The field types stay
//! `&'static str` and `&'static [T]`, so the defs stay `Copy` where they were and their readers see no change.

use serde::{Deserialize, Deserializer};

/// A `&'static str` field: a name.
pub(super) fn str<'de, D: Deserializer<'de>>(deserializer: D) -> Result<&'static str, D::Error> {
    String::deserialize(deserializer).map(|name| &*name.leak())
}

/// A `&'static [T]` field: a list, such as a producer's recipes or a Tunnel's tiers.
pub(super) fn slice<'de, D: Deserializer<'de>, T: Deserialize<'de>>(deserializer: D) -> Result<&'static [T], D::Error> {
    Vec::<T>::deserialize(deserializer).map(|list| &*list.leak())
}

/// An `Option<&'static [T]>` field: a list a type may not have, such as a structure's abilities.
pub(super) fn option_slice<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<&'static [T]>, D::Error> {
    Option::<Vec<T>>::deserialize(deserializer).map(|list| list.map(|list| &*list.leak()))
}
