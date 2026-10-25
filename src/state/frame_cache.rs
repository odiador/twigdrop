use crate::state::ui::UiState;

/// Per-frame render cache. Owned by `App` behind a `RefCell` so render
/// (`ui::render`, `&App`) can record hit-test data without `&mut`.
/// Scroll math stays pure in `UiState::branch_viewport`; this only stores
/// the result plus mouse hit positions rebuilt each frame.
#[derive(Default, Debug)]
pub struct FrameCache {
    pub list_start_index: usize,
    pub branch_screen_positions: Vec<(usize, u16)>,
    /// Spinner frame counter; advanced in `draw` while `App::is_busy()`.
    pub spinner_tick: u64,
}

impl FrameCache {
    /// Sync scroll cache; returns `(start, visible)`. Resets hit positions.
    pub fn sync_branches(
        &mut self,
        selected: usize,
        total: usize,
        height: usize,
    ) -> (usize, usize) {
        let (start, visible) = UiState::branch_viewport(selected, total, height);
        self.list_start_index = start;
        self.branch_screen_positions.clear();
        (start, visible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_cache_sync_resets_hits() {
        let mut cache = FrameCache::default();
        cache.branch_screen_positions.push((0, 3));
        let (start, visible) = cache.sync_branches(0, 87, 20);
        assert_eq!((start, visible), (0, 20));
        assert_eq!(cache.list_start_index, start);
        assert!(cache.branch_screen_positions.is_empty());
    }
}
