use ratatui::style::Color;

#[allow(dead_code)]
pub const DENSITY_CHARS: [char; 5] = ['·', '░', '▒', '▓', '█'];

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    vx: f32,
    /// Vertical velocity (cells per tick; negative = upward).
    vy: f32,
    pub color: Color,
    pub density: u8,
    pub lifetime: u8,
}

#[allow(dead_code)]
impl Particle {
    pub fn new(x: u16, y: u16, color: Color) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            vx: (fast_rand_f32() - 0.5) * 4.0,
            vy: (fast_rand_f32() - 1.0) * 2.0,
            color,
            density: 4,
            lifetime: 100,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.x += self.vx;
        self.y += self.vy;
        self.vy += 0.1; // gravity
        self.lifetime = self.lifetime.saturating_sub(2);
        self.density = (self.lifetime / 20).min(4);
        self.lifetime > 0
    }
}

#[allow(dead_code)]
pub struct ParticleSystem {
    pub particles: Vec<Particle>,
}

#[allow(dead_code)]
impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
        }
    }

    pub fn spawn(&mut self, x: u16, y: u16, color: Color) {
        self.particles.push(Particle::new(x, y, color));
    }

    pub fn tick(&mut self) {
        self.particles.retain_mut(|p| p.tick());
    }
}

fn fast_rand_f32() -> f32 {
    fastrand::f32()
}
