use bevy::prelude::*;
use std::time::Instant;
use super::types::{PerformanceMetrics, TimerStarts};

/// System that starts a timer for a given category.
/// Must be ordered `.before()` the systems it measures.
pub fn start_timer(category: &'static str) -> impl FnMut(ResMut<TimerStarts>) {
    move |mut timer_starts: ResMut<TimerStarts>| {
        timer_starts.starts.insert(category, Instant::now());
    }
}

/// System that stops the timer for a given category and records the duration.
/// Must be ordered `.after()` the systems it measures.
pub fn stop_timer(category: &'static str) -> impl FnMut(ResMut<TimerStarts>, ResMut<PerformanceMetrics>) {
    move |mut timer_starts: ResMut<TimerStarts>, mut metrics: ResMut<PerformanceMetrics>| {
        if let Some(start) = timer_starts.starts.remove(category) {
            let elapsed = start.elapsed();
            metrics.record(category, elapsed);
        }
    }
}

/// System that prunes old samples from the metrics resource.
/// Runs once per frame.
pub fn prune_metrics(mut metrics: ResMut<PerformanceMetrics>) {
    metrics.prune();
}

/// System that logs metrics to the console at the configured interval.
pub fn console_log_metrics_system(mut metrics: ResMut<PerformanceMetrics>) {
    let interval = match metrics.console_log_interval {
        Some(i) => i,
        None => return,
    };

    let elapsed = metrics.last_console_log.elapsed().as_secs_f32();
    if elapsed < interval {
        return;
    }

    metrics.last_console_log = std::time::Instant::now();

    if !metrics.overlay_visible {
        return;
    }

    info!("=== Performance Diagnostics ===");
    let cats = metrics.categories();
    for cat in cats {
        if let Some(stats) = metrics.stats(cat) {
            info!(
                "  {:<16} min={:>8} avg={:>8} max={:>8} (n={})",
                cat,
                super::utils::format_duration(stats.min),
                super::utils::format_duration(stats.avg),
                super::utils::format_duration(stats.max),
                stats.sample_count,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::*;

    #[test]
    fn timer_starts_insert_and_remove() {
        let mut ts = TimerStarts::default();
        ts.starts.insert("Test", std::time::Instant::now());
        assert!(ts.starts.contains_key("Test"));
        ts.starts.remove("Test");
        assert!(!ts.starts.contains_key("Test"));
    }

    #[test]
    fn performance_metrics_record_from_instrumentation() {
        let mut metrics = PerformanceMetrics::default();
        metrics.record("Instrumented", std::time::Duration::from_micros(42));
        assert_eq!(metrics.stats("Instrumented").unwrap().sample_count, 1);
    }
}
