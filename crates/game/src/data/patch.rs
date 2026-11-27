//! (M5-X2) A data file is a JSON merge patch (RFC 7386) over its built-in counterpart ([`merge`]), checked against
//! it before it is read ([`check`]): a full exported file is a valid patch that changes nothing, and a file that
//! names one number changes that number alone.
//!
//! - **Objects merge** member by member; anything else (a number, a text, a list) replaces the counterpart's whole, so
//!   a patch of a list (a building's recipes) gives the whole list.
//! - **`null`** sets a member to `null` (an optional value to none), where RFC 7386 removes the member: every member
//!   of the rules' and the client's JSON is always there, so a removed member could only mean none.
//! - **The checks**, each refused with the member's path in the file ([`Refused`]): a member the counterpart doesn't
//!   have ("unknown field"); a value of another kind of JSON than the counterpart's ("expected a number"); a fraction
//!   where the counterpart has a whole number ("expected a whole number"); and a number whose sign isn't the
//!   counterpart's, the rule `Ruleset::check` applies to the rules ("must be positive": a number the built-in data
//!   have positive stays positive, one at 0 doesn't go negative, a negative one stays negative), which holds for the
//!   client's tuning too. A list's members are checked against the counterpart's at the same place, as far as both go.

use std::fmt::Write;

use serde_json::Value;

/// A refusal: the path of the member in its file (`unit.attack.damage`, `recipes[0].cost`; empty for the file's
/// whole), and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused {
    pub path: String,
    pub why: String,
}

/// `patch` merged onto `target` (module docs).
pub fn merge(target: &mut Value, patch: &Value) {
    match (target, patch) {
        (Value::Object(target), Value::Object(patch)) => {
            for (key, value) in patch {
                match target.get_mut(key) {
                    Some(member) => merge(member, value),
                    None => {
                        target.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        (target, patch) => *target = patch.clone(),
    }
}

/// The first problem of `patch` over its counterpart `builtin` (module docs), if it has one.
pub fn check(builtin: &Value, patch: &Value) -> Option<Refused> {
    let mut path = String::new();
    check_at(builtin, patch, &mut path).map(|why| Refused {
        path,
        why: why.to_owned(),
    })
}

/// [`check`] at `path`, which it leaves at the refused member's.
fn check_at(builtin: &Value, patch: &Value, path: &mut String) -> Option<&'static str> {
    match (builtin, patch) {
        (_, Value::Null) | (Value::Null, _) => None,
        (Value::Object(builtin), Value::Object(patch)) => {
            for (key, value) in patch {
                let len = path.len();
                if !path.is_empty() {
                    path.push('.');
                }
                path.push_str(key);
                let Some(counterpart) = builtin.get(key) else {
                    return Some("unknown field");
                };
                if let Some(why) = check_at(counterpart, value, path) {
                    return Some(why);
                }
                path.truncate(len);
            }
            None
        }
        (Value::Array(builtin), Value::Array(patch)) => {
            for (i, (counterpart, value)) in builtin.iter().zip(patch).enumerate() {
                let len = path.len();
                let _ = write!(path, "[{i}]");
                if let Some(why) = check_at(counterpart, value, path) {
                    return Some(why);
                }
                path.truncate(len);
            }
            None
        }
        (Value::Number(builtin), Value::Number(number)) => {
            let whole = |n: &serde_json::Number| n.is_i64() || n.is_u64();
            if whole(builtin) && !whole(number) {
                return Some("expected a whole number");
            }
            let (was, is) = (builtin.as_f64()?, number.as_f64()?);
            sign(was, is)
        }
        (Value::Bool(_), Value::Bool(_)) | (Value::String(_), Value::String(_)) => None,
        (builtin, _) => Some(expected(builtin)),
    }
}

/// What a number of `was` in the built-in data must stay as `is` (module docs): `None` if it does.
pub fn sign(was: f64, is: f64) -> Option<&'static str> {
    if was > 0.0 && is <= 0.0 {
        Some("must be positive")
    } else if was == 0.0 && is < 0.0 {
        Some("must not be negative")
    } else if was < 0.0 && is >= 0.0 {
        Some("must be negative")
    } else {
        None
    }
}

/// The refusal of a value of another kind than `builtin`'s.
fn expected(builtin: &Value) -> &'static str {
    match builtin {
        Value::Null => "expected null",
        Value::Bool(_) => "expected true or false",
        Value::Number(_) => "expected a number",
        Value::String(_) => "expected a text",
        Value::Array(_) => "expected a list",
        Value::Object(_) => "expected an object",
    }
}

/// The paths of the leaves where `patched` differs from `builtin`, in order: a member, a list's element, a list that
/// grew or shrank (whole), as `check`'s paths name them, each with its value in `patched`.
pub fn changes(builtin: &Value, patched: &Value) -> Vec<(Vec<Step>, Value)> {
    let mut found = Vec::new();
    changes_at(builtin, patched, &mut Vec::new(), &mut found);
    found
}

/// A step of a path in a file's JSON: a member, or a list's element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    Member(String),
    Element(usize),
}

fn changes_at(builtin: &Value, patched: &Value, path: &mut Vec<Step>, found: &mut Vec<(Vec<Step>, Value)>) {
    match (builtin, patched) {
        (Value::Object(builtin), Value::Object(patched)) => {
            for (key, value) in patched {
                path.push(Step::Member(key.clone()));
                match builtin.get(key) {
                    Some(counterpart) => changes_at(counterpart, value, path, found),
                    None => found.push((path.clone(), value.clone())),
                }
                path.pop();
            }
        }
        (Value::Array(builtin), Value::Array(patched)) if builtin.len() == patched.len() => {
            for (i, (counterpart, value)) in builtin.iter().zip(patched).enumerate() {
                path.push(Step::Element(i));
                changes_at(counterpart, value, path, found);
                path.pop();
            }
        }
        (builtin, patched) if builtin != patched => found.push((path.clone(), patched.clone())),
        _ => {}
    }
}

/// `path` as `check` names it: `unit.attack.damage`, `recipes[0].cost`.
pub fn path_text(path: &[Step]) -> String {
    let mut text = String::new();
    for step in path {
        match step {
            Step::Member(key) => {
                if !text.is_empty() {
                    text.push('.');
                }
                text.push_str(key);
            }
            Step::Element(i) => {
                let _ = write!(text, "[{i}]");
            }
        }
    }
    text
}
