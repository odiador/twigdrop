use std::time::{Duration, Instant};
use ratatui::style::Color;
use super::particles::ParticleSystem;

const FLASH_DURATION: Duration = Duration::from_millis(200);
const SETTLE_DURATION: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapPhase {
    Flash,
    Dissolve,
    Settle,
    Done,
}

pub struct SnapAnimation {
    pub phase: SnapPhase,
    pub start_time: Instant,
    pub captured: bool,
    pub rows: Vec<SnapRow>,
    pub particles: ParticleSystem,
}

pub struct SnapRow {
    pub branch_name: String,
    pub screen_y: Option<u16>,
    pub cells: Vec<SnapCell>,
    pub progress: f32, // 0.0 to 1.0
}

pub struct SnapCell {
    pub x: u16,
    #[allow(dead_code)]
    pub ch: char,
    pub color: Color,
    pub dissolved: bool,
}

impl SnapAnimation {
    pub fn new(branch_names: Vec<String>) -> Self {
        Self {
            phase: SnapPhase::Flash,
            start_time: Instant::now(),
            captured: false,
            rows: branch_names.into_iter().map(|name| SnapRow {
                branch_name: name,
                screen_y: None,
                cells: Vec::new(),
                progress: 0.0,
            }).collect(),
            particles: ParticleSystem::new(),
        }
    }

    pub fn tick(&mut self) {
        let elapsed = self.start_time.elapsed();

        match self.phase {
            SnapPhase::Flash => {
                if elapsed >= FLASH_DURATION {
                    self.phase = SnapPhase::Dissolve;
                }
            }
            SnapPhase::Dissolve => {
                let mut all_done = true;
                for row in self.rows.iter_mut() {
                    if row.progress < 1.0 {
                        row.progress += 0.05;
                        all_done = false;
                        
                        // Dissolve more cells
                        let to_dissolve = (row.cells.len() as f32 * row.progress) as usize;
                        for i in 0..to_dissolve.min(row.cells.len()) {
                            if !row.cells[i].dissolved {
                                row.cells[i].dissolved = true;
                                if let Some(y) = row.screen_y
                                    && fastrand::f32() < 0.4 {
                                        self.particles.spawn(row.cells[i].x, y, row.cells[i].color);
                                }
                            }
                        }
                    }
                }
                
                self.particles.tick();
                
                if all_done && self.particles.particles.is_empty() {
                    self.phase = SnapPhase::Settle;
                    self.start_time = Instant::now();
                }
            }
            SnapPhase::Settle => {
                if elapsed >= SETTLE_DURATION {
                    self.phase = SnapPhase::Done;
                }
            }
            SnapPhase::Done => {}
        }
    }
}
