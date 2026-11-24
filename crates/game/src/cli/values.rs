//! The values the flags take, parsed.

use bevy::math::UVec2;
use sc_sim::{Faction, Tick};

use super::MAX_WINDOW_SIDE;
use crate::editor::objects::faction_name;
use crate::qa::ScreenshotRequest;

pub(super) fn parse_tick(s: &str) -> Result<Tick, String> {
    s.parse().map_err(|_| format!("{s:?} is not a tick number"))
}

pub(super) fn parse_player(s: &str) -> Result<u8, String> {
    match s.parse::<u8>() {
        Ok(n) if n >= 1 => Ok(n),
        _ => Err(format!("{s:?} is not a player number (1, 2, …)")),
    }
}

pub(super) fn parse_size(s: &str) -> Result<UVec2, String> {
    let error = || format!("{s:?} is not a size like 1280x720");
    let (w, h) = s.split_once(['x', 'X']).ok_or_else(error)?;
    let side = |v: &str| v.parse::<u32>().ok().filter(|n| (1..=MAX_WINDOW_SIDE).contains(n));
    Ok(UVec2::new(side(w).ok_or_else(error)?, side(h).ok_or_else(error)?))
}

pub(super) fn parse_screenshot(s: &str) -> Result<ScreenshotRequest, String> {
    let (tick, path) = s
        .split_once(':')
        .filter(|(_, path)| !path.is_empty())
        .ok_or_else(|| format!("{s:?} is not <tick>:<path>"))?;
    Ok(ScreenshotRequest {
        tick: parse_tick(tick)?,
        path: path.into(),
    })
}

/// `--factions`: a comma-separated list of faction names, any case (`gdo,colonists`), one per slot in slot order.
pub(super) fn parse_factions(s: &str) -> Result<Vec<Faction>, String> {
    s.split(',')
        .map(|name| {
            let name = name.trim();
            Faction::ALL
                .iter()
                .copied()
                .find(|&f| faction_name(f).eq_ignore_ascii_case(name))
                .ok_or_else(|| {
                    let names: Vec<String> = Faction::ALL.iter().map(|&f| faction_name(f).to_lowercase()).collect();
                    format!("{name:?} is not a faction ({})", names.join(", "))
                })
        })
        .collect()
}
