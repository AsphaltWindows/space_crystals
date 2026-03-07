use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Per-system performance metrics resource.
/// Stores timing samples per category over a rolling window.
#[derive(Resource)]
pub struct PerformanceMetrics {
    /// Per-category timing samples: (timestamp, duration) pairs
    pub samples: HashMap<&'static str, VecDeque<(Instant, Duration)>>,
    /// Rolling window duration in seconds
    pub window_seconds: f32,
    /// Whether the debug overlay is visible
    pub overlay_visible: bool,
    /// Console log interval in seconds (None = disabled)
    pub console_log_interval: Option<f32>,
    /// Last time console metrics were logged
    pub last_console_log: Instant,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            samples: HashMap::new(),
            window_seconds: 2.0,
            overlay_visible: false,
            console_log_interval: None,
            last_console_log: Instant::now(),
        }
    }
}

impl PerformanceMetrics {
    /// Record a timing sample for a category.
    pub fn record(&mut self, category: &'static str, duration: Duration) {
        let entry = self.samples.entry(category).or_default();
        entry.push_back((Instant::now(), duration));
    }

    /// Prune entries older than the rolling window.
    pub fn prune(&mut self) {
        let cutoff = Instant::now() - Duration::from_secs_f32(self.window_seconds);
        for samples in self.samples.values_mut() {
            while let Some(&(ts, _)) = samples.front() {
                if ts < cutoff {
                    samples.pop_front();
                } else {
                    break;
                }
            }
        }
    }

    /// Compute aggregated stats for a category.
    pub fn stats(&self, category: &str) -> Option<CategoryStats> {
        let samples = self.samples.get(category)?;
        if samples.is_empty() {
            return None;
        }
        let mut min = Duration::MAX;
        let mut max = Duration::ZERO;
        let mut total = Duration::ZERO;
        let count = samples.len() as u32;
        for &(_, dur) in samples {
            if dur < min {
                min = dur;
            }
            if dur > max {
                max = dur;
            }
            total += dur;
        }
        Some(CategoryStats {
            min,
            avg: total / count,
            max,
            sample_count: count,
        })
    }

    /// Get all category names, sorted alphabetically.
    pub fn categories(&self) -> Vec<&'static str> {
        let mut cats: Vec<&'static str> = self.samples.keys().copied().collect();
        cats.sort();
        cats
    }
}

/// Aggregated stats for a single system category.
#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub min: Duration,
    pub avg: Duration,
    pub max: Duration,
    pub sample_count: u32,
}

/// Marker for timer start instants stored per-category.
/// Used by start/stop timer systems.
#[derive(Resource)]
pub struct TimerStarts {
    pub starts: HashMap<&'static str, Instant>,
}

impl Default for TimerStarts {
    fn default() -> Self {
        Self {
            starts: HashMap::new(),
        }
    }
}

/// Marker component for the diagnostics overlay root node.
#[derive(Component)]
pub struct DiagnosticsOverlay;

/// Marker component for the diagnostics text content.
#[derive(Component)]
pub struct DiagnosticsText;

/// Clock type for categorizing systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockType {
    /// Render-frame systems (~60fps)
    Update,
    /// Simulation-tick systems (16fps)
    FixedUpdate,
}

/// Definition of a system category to instrument
pub struct CategoryDef {
    pub name: &'static str,
    pub clock: ClockType,
}

/// All system categories to instrument
pub const CATEGORIES: &[CategoryDef] = &[
    // FixedUpdate categories
    CategoryDef { name: "FogOfWar", clock: ClockType::FixedUpdate },
    CategoryDef { name: "Construction", clock: ClockType::FixedUpdate },
    CategoryDef { name: "SupplyDelivery", clock: ClockType::FixedUpdate },
    // Update categories
    CategoryDef { name: "Combat", clock: ClockType::Update },
    CategoryDef { name: "Turrets", clock: ClockType::Update },
    CategoryDef { name: "Projectiles", clock: ClockType::Update },
    CategoryDef { name: "Movement", clock: ClockType::Update },
    CategoryDef { name: "Commands", clock: ClockType::Update },
    CategoryDef { name: "Selection", clock: ClockType::Update },
    CategoryDef { name: "Map", clock: ClockType::Update },
    CategoryDef { name: "Faction", clock: ClockType::Update },
    CategoryDef { name: "UI/HUD", clock: ClockType::Update },
    CategoryDef { name: "Camera", clock: ClockType::Update },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.window_seconds, 2.0);
        assert!(!metrics.overlay_visible);
        assert!(metrics.console_log_interval.is_none());
        assert!(metrics.samples.is_empty());
    }

    #[test]
    fn record_and_stats() {
        let mut metrics = PerformanceMetrics::default();
        metrics.record("TestCat", Duration::from_micros(100));
        metrics.record("TestCat", Duration::from_micros(200));
        metrics.record("TestCat", Duration::from_micros(300));

        let stats = metrics.stats("TestCat").unwrap();
        assert_eq!(stats.min, Duration::from_micros(100));
        assert_eq!(stats.max, Duration::from_micros(300));
        assert_eq!(stats.avg, Duration::from_micros(200));
        assert_eq!(stats.sample_count, 3);
    }

    #[test]
    fn stats_nonexistent_category() {
        let metrics = PerformanceMetrics::default();
        assert!(metrics.stats("Nonexistent").is_none());
    }

    #[test]
    fn stats_single_sample() {
        let mut metrics = PerformanceMetrics::default();
        metrics.record("Single", Duration::from_micros(500));
        let stats = metrics.stats("Single").unwrap();
        assert_eq!(stats.min, Duration::from_micros(500));
        assert_eq!(stats.max, Duration::from_micros(500));
        assert_eq!(stats.avg, Duration::from_micros(500));
        assert_eq!(stats.sample_count, 1);
    }

    #[test]
    fn categories_sorted() {
        let mut metrics = PerformanceMetrics::default();
        metrics.record("Zebra", Duration::from_micros(1));
        metrics.record("Alpha", Duration::from_micros(1));
        metrics.record("Middle", Duration::from_micros(1));
        let cats = metrics.categories();
        assert_eq!(cats, vec!["Alpha", "Middle", "Zebra"]);
    }

    #[test]
    fn prune_removes_old_entries() {
        let mut metrics = PerformanceMetrics::default();
        metrics.window_seconds = 0.0; // Prune everything
        metrics.record("Cat", Duration::from_micros(100));
        // Sleep a tiny bit so entries become "old"
        std::thread::sleep(Duration::from_millis(1));
        metrics.prune();
        assert!(metrics.stats("Cat").is_none() || metrics.stats("Cat").unwrap().sample_count == 0);
    }

    #[test]
    fn clock_type_variants() {
        assert_ne!(ClockType::Update, ClockType::FixedUpdate);
    }

    #[test]
    fn category_defs_not_empty() {
        assert!(!CATEGORIES.is_empty());
    }

    #[test]
    fn all_categories_have_names() {
        for cat in CATEGORIES {
            assert!(!cat.name.is_empty());
        }
    }

    #[test]
    fn timer_starts_default() {
        let ts = TimerStarts::default();
        assert!(ts.starts.is_empty());
    }
}
