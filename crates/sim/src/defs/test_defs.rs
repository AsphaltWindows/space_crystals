//! Test-only object types, for the sim's own unit tests (`#[cfg(test)]`): a test gives an existing kind other numbers,
//! on its own thread, for as long as the guard [`TestDef`] lives, so that it can try what no type of the design has
//! yet (a MinRange above 0, an artillery's Range, a unit that can't move) through the whole sim. `ObjectKind::def`
//! reads them first. Nothing outside tests has them: the game's types are the design's alone.

use std::cell::RefCell;
use std::collections::BTreeMap;

use super::{ObjectDef, ObjectKind};

thread_local! {
    /// The kinds whose def a test replaced on this thread.
    static DEFS: RefCell<BTreeMap<ObjectKind, &'static ObjectDef>> = const { RefCell::new(BTreeMap::new()) };
}

/// The def a test gave `kind` on this thread, if any.
pub(super) fn get(kind: ObjectKind) -> Option<&'static ObjectDef> {
    DEFS.with(|defs| defs.borrow().get(&kind).copied())
}

/// While it lives, `ObjectKind::def` gives its def for its kind on this thread; dropped, the kind is what it was.
#[must_use = "the def is replaced only while the guard lives"]
pub(crate) struct TestDef {
    kind: ObjectKind,
    before: Option<&'static ObjectDef>,
}

impl TestDef {
    /// Replaces the def of `def.kind` by `def`.
    pub(crate) fn set(def: ObjectDef) -> Self {
        let kind = def.kind;
        let def: &'static ObjectDef = Box::leak(Box::new(def));
        let before = DEFS.with(|defs| defs.borrow_mut().insert(kind, def));
        Self { kind, before }
    }

    /// Replaces the def of the unit type `kind` by its own with `change` applied to its unit attributes.
    pub(crate) fn unit(kind: ObjectKind, change: impl FnOnce(&mut super::UnitDef)) -> Self {
        let mut def = kind.def().clone();
        change(def.unit.as_mut().expect("a unit type"));
        Self::set(def)
    }
}

impl Drop for TestDef {
    fn drop(&mut self) {
        DEFS.with(|defs| {
            let mut defs = defs.borrow_mut();
            match self.before {
                Some(before) => defs.insert(self.kind, before),
                None => defs.remove(&self.kind),
            };
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_test_def_lasts_as_long_as_its_guard() {
        let guard = ObjectKind::Guard.def();
        {
            let _far = TestDef::unit(ObjectKind::Guard, |unit| unit.attack.as_mut().unwrap().range = 9.0);
            assert_eq!(
                ObjectKind::Guard.def().unit.as_ref().unwrap().attack.unwrap().range,
                9.0
            );
            {
                let _near = TestDef::unit(ObjectKind::Guard, |unit| unit.attack.as_mut().unwrap().range = 1.0);
                assert_eq!(
                    ObjectKind::Guard.def().unit.as_ref().unwrap().attack.unwrap().range,
                    1.0
                );
            }
            assert_eq!(
                ObjectKind::Guard.def().unit.as_ref().unwrap().attack.unwrap().range,
                9.0
            );
            assert!(std::ptr::eq(
                ObjectKind::Peacekeeper.def(),
                &super::super::gdo::units::peacekeeper::PEACEKEEPER
            ));
        }
        assert!(std::ptr::eq(ObjectKind::Guard.def(), guard));
    }
}
