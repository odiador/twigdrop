use ratatui::style::Color;

/// Unicode characters for particle density levels.
/// Index 0 is lowest density (nearly gone), index 4 is highest (just spawned).
pub const DENSITY_CHARS: [char; 5] = ['·', '░', '▒', '▓', '█'];

/// Maximum number of active particles at any time.
pub const MAX_PARTICLES: usize = 400; // Increased for more "dust"

/// A single particle drifting across the screen.
#[derive(Debug, Clone)]
pub struct Particle {
    /// Column position (fractional for smooth movement).
    pub x: f32,
    /// Row position (fractional for smooth movement).
    pub y: f32,
    /// Horizontal velocity (cells per tick; positive = rightward).
    vx: f32,
    /// Vertical velocity (cells per tick; negative = upward).
    vy: f32,
    /// Density level: 4=█, 3=▓, 2=▒, 1=░, 0=·
    pub density: u8,
    /// Color inherited from the source cell.
    pub color: Color,
    /// Frames remaining before this particle is removed.
    pub lifetime: u8,
}

impl Particle {
    pub fn new(x: u16, y: u16, color: Color) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            // Random upward/rightward drift
            vx: fastrand::f32() * 0.5 - 0.1,
            vy: -(fastrand::f32() * 0.4 + 0.1),
            density: 4,
            color,
            lifetime: 20 + fastrand::u8(0..20),
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.lifetime == 0 {
            return false;
        }

        self.x += self.vx;
        self.y += self.vy;
        
        // Gravity
        self.vy += 0.02;
        
        // Jitter
        self.vx += (fastrand::f32() - 0.5) * 0.1;

        // Density decay every 4 frames
        if (self.lifetime as u32).is_multiple_of(4) && self.density > 0 {
            self.density -= 1;
        }

        self.lifetime -= 1;
        self.lifetime > 0
    }
}

pub struct ParticleSystem {
    pub particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(MAX_PARTICLES),
        }
    }

    pub fn spawn(&mut self, x: u16, y: u16, color: Color) {
        if self.particles.len() < MAX_PARTICLES {
            self.particles.push(Particle::new(x, y, color));
        }
    }

    pub fn tick(&mut self) {
        self.particles.retain_mut(|p| p.tick());
    }
}
