use std::time::Duration;

/// Duration to poll for terminal input events in the background loop.
pub const INPUT_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Target frame duration for animations (~60 FPS).
pub const ANIMATION_FRAME_INTERVAL: Duration = Duration::from_millis(16);

/// Idle tick duration for TTL expiration and spinner animation (~4 FPS).
pub const IDLE_TICK_INTERVAL: Duration = Duration::from_millis(250);

/// Debounce duration for file system refresh events.
pub const REFRESH_DEBOUNCE_INTERVAL: Duration = Duration::from_secs(2);

/// Default number of recent commits to display in inspector drawers.
pub const DEFAULT_INSPECTOR_COMMIT_LIMIT: usize = 4;
