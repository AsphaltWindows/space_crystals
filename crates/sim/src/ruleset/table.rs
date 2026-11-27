//! A value for each key of a fixed list (`Ruleset::defs`, `Ruleset::bases`, the Cults' techs and abilities): an
//! array in the list's order, indexed by key. In files it is a map by key, every key once, so that a data file names
//! what it tunes and a JSON merge patch (plan decision T66) changes one entry without restating the others.

use std::fmt;
use std::marker::PhantomData;
use std::ops::Index;

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A key of a [`Table`]: a fieldless enum, whose values in declaration order are its list. (M6-GR) The list may leave
/// out the enum's last values, which no table of it holds (`AreaAbility::Scan`, whose numbers aren't in the Cults'
/// table): a file that names one is refused.
pub trait Key: Copy + Ord + fmt::Debug + Serialize + for<'de> Deserialize<'de> + 'static {
    /// Every key, in order: the table's.
    const ALL: &'static [Self];

    /// Its place in [`Key::ALL`]; (M6-GR) for a value that isn't one of them, at least the list's length.
    fn index(self) -> usize;
}

/// A value for each key of `K` (module docs), `N` of them.
#[derive(Clone, Debug, PartialEq)]
pub struct Table<K, V, const N: usize> {
    values: [V; N],
    keys: PhantomData<K>,
}

impl<K: Key, V, const N: usize> Table<K, V, N> {
    /// The table of `values`, one for each key in [`Key::ALL`]'s order.
    pub const fn new(values: [V; N]) -> Self {
        assert!(K::ALL.len() == N, "a value for each key");
        Self {
            values,
            keys: PhantomData,
        }
    }

    /// The table of `value(key)` for each key.
    pub fn from_fn(mut value: impl FnMut(K) -> V) -> Self {
        Self::new(std::array::from_fn(|i| value(K::ALL[i])))
    }

    /// Each key with its value, in the keys' order.
    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        K::ALL.iter().copied().zip(&self.values)
    }

    /// The value of `key`, to change.
    pub fn get_mut(&mut self, key: K) -> &mut V {
        &mut self.values[key.index()]
    }
}

impl<K: Key, V, const N: usize> Index<K> for Table<K, V, N> {
    type Output = V;

    #[inline]
    fn index(&self, key: K) -> &V {
        &self.values[key.index()]
    }
}

impl<K: Key, V: Serialize, const N: usize> Serialize for Table<K, V, N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(N))?;
        for (key, value) in self.iter() {
            map.serialize_entry(&key, value)?;
        }
        map.end()
    }
}

impl<'de, K: Key, V: Deserialize<'de>, const N: usize> Deserialize<'de> for Table<K, V, N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Entries<K, V, const N: usize>(PhantomData<(K, V)>);

        impl<'de, K: Key, V: Deserialize<'de>, const N: usize> Visitor<'de> for Entries<K, V, N> {
            type Value = Table<K, V, N>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a map with each of the {N} keys once")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut entries: A) -> Result<Self::Value, A::Error> {
                let mut values: [Option<V>; N] = std::array::from_fn(|_| None);
                while let Some(key) = entries.next_key::<K>()? {
                    // (M6-GR) A value of the key's type that the table hasn't (`Key::index`).
                    let Some(value) = values.get_mut(key.index()) else {
                        return Err(de::Error::custom(format_args!("{key:?} isn't one of the table's keys")));
                    };
                    if value.replace(entries.next_value()?).is_some() {
                        return Err(de::Error::custom(format_args!("{key:?} appears more than once")));
                    }
                }
                if let Some(missing) = K::ALL.iter().find(|key| values[key.index()].is_none()) {
                    return Err(de::Error::custom(format_args!("{missing:?} is missing")));
                }
                Ok(Table::new(values.map(|value| value.expect("every key is there"))))
            }
        }

        deserializer.deserialize_map(Entries(PhantomData))
    }
}
