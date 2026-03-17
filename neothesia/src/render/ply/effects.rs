//! PLY-based visual effects for Neothesia
//!
//! This module provides PLY-integrated visual effects including
//! glow effects, background animations, and particle effects.

use ply_engine::prelude::*;
use ply_engine::color::Color;
use std::time::Duration;
use piano_layout::KeyboardLayout;

/// PLY-based glow effect renderer
pub struct PlyGlowRenderer {
    /// Glow instances
    glows: Vec<GlowInstance>,
    /// Maximum number of glow instances
    max_glows: usize,
}

/// Glow instance data
#[derive(Debug, Clone)]
pub struct GlowInstance {
    /// Key/note ID
    pub key_id: u8,
    /// Color (RGB)
    pub color: (u8, u8, u8),
    /// Position (X, Y)
    pub position: (f32, f32),
    /// Size (width, height)
    pub size: (f32, f32),
    /// Intensity (0.0 - 1.0)
    pub intensity: f32,
    /// Fade duration
    pub fade_duration: Duration,
}

impl PlyGlowRenderer {
    /// Create a new PLY glow renderer
    pub fn new(max_glows: usize) -> Self {
        Self {
            glows: Vec::with_capacity(max_glows),
            max_glows,
        }
    }

    /// Clear all glow instances
    pub fn clear(&mut self) {
        self.glows.clear();
    }

    /// Add a glow instance
    pub fn push(&mut self, key_id: u8, color: (u8, u8, u8), x: f32, y: f32, width: f32, delta: Duration) {
        if self.glows.len() >= self.max_glows {
            return;
        }

        self.glows.push(GlowInstance {
            key_id,
            color,
            position: (x, y),
            size: (width, width * 4.0), // Height is 4x width for glow effect
            intensity: 1.0,
            fade_duration: delta,
        });
    }

    /// Update glow intensities
    pub fn update(&mut self, dt: Duration) {
        for glow in &mut self.glows {
            // Fade out over time
            let fade_factor = dt.as_secs_f32() / glow.fade_duration.as_secs_f32().max(0.001);
            glow.intensity = (glow.intensity - fade_factor).max(0.0);
        }

        // Remove faded glows
        self.glows.retain(|g| g.intensity > 0.01);
    }

    /// Get glow instances
    pub fn instances(&self) -> &[GlowInstance] {
        &self.glows
    }

    /// Get mutable glow instances
    pub fn instances_mut(&mut self) -> &mut [GlowInstance] {
        &mut self.glows
    }
}

/// PLY-based background animation renderer
pub struct PlyBackgroundRenderer {
    /// Animation time
    time: f32,
    /// Animation speed
    speed: f32,
    /// Background color
    background_color: Color,
    /// Animation mode
    mode: BackgroundMode,
}

/// Background animation mode
#[derive(Debug, Clone, Copy)]
pub enum BackgroundMode {
    /// Static color
    Static,
    /// Animated gradient
    Gradient,
    /// Animated notes
    Notes,
    /// Particle effect
    Particles,
}

impl Default for BackgroundMode {
    fn default() -> Self {
        Self::Notes
    }
}

impl PlyBackgroundRenderer {
    /// Create a new PLY background renderer
    pub fn new() -> Self {
        Self {
            time: 10.0, // Start at 10.0 like the original
            speed: 1.0,
            background_color: Color::rgb(0.01, 0.01, 0.01),
            mode: BackgroundMode::default(),
        }
    }

    /// Update animation time
    pub fn update(&mut self, delta: Duration) {
        self.time += delta.as_secs_f32() * self.speed;
    }

    /// Get current animation time
    pub fn time(&self) -> f32 {
        self.time
    }

    /// Set animation speed
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    /// Get animation speed
    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// Set background color
    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    /// Get background color
    pub fn background_color(&self) -> Color {
        self.background_color
    }

    /// Set animation mode
    pub fn set_mode(&mut self, mode: BackgroundMode) {
        self.mode = mode;
    }

    /// Get animation mode
    pub fn mode(&self) -> BackgroundMode {
        self.mode
    }

    /// Calculate background color at current time
    pub fn calculate_color(&self) -> Color {
        match self.mode {
            BackgroundMode::Static => self.background_color,
            BackgroundMode::Gradient => {
                // Slow gradient animation
                let t = (self.time * 0.1).sin() * 0.5 + 0.5;
                Color::rgb(
                    self.background_color.r * (1.0 - t) + 0.02 * t,
                    self.background_color.g * (1.0 - t) + 0.01 * t,
                    self.background_color.b * (1.0 - t) + 0.03 * t,
                )
            }
            BackgroundMode::Notes | BackgroundMode::Particles => {
                // Animated notes effect - darker base with subtle animation
                let t = (self.time * 0.2).sin() * 0.5 + 0.5;
                Color::rgb(
                    0.01 + 0.005 * t,
                    0.01 + 0.002 * t,
                    0.02 + 0.01 * t,
                )
            }
        }
    }
}

impl Default for PlyBackgroundRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// PLY-based particle effect renderer
pub struct PlyParticleRenderer {
    /// Active particles
    particles: Vec<Particle>,
    /// Maximum number of particles
    max_particles: usize,
}

/// Particle instance
#[derive(Debug, Clone)]
pub struct Particle {
    /// Position (X, Y)
    pub position: (f32, f32),
    /// Velocity (X, Y)
    pub velocity: (f32, f32),
    /// Color
    pub color: Color,
    /// Size
    pub size: f32,
    /// Lifetime (seconds)
    pub lifetime: f32,
    /// Current age
    pub age: f32,
}

impl PlyParticleRenderer {
    /// Create a new PLY particle renderer
    pub fn new(max_particles: usize) -> Self {
        Self {
            particles: Vec::with_capacity(max_particles),
            max_particles,
        }
    }

    /// Clear all particles
    pub fn clear(&mut self) {
        self.particles.clear();
    }

    /// Spawn particles at position
    pub fn spawn(&mut self, x: f32, y: f32, color: Color, count: usize) {
        if self.particles.len() + count > self.max_particles {
            return;
        }

        // Simple pseudo-random generator
        let mut seed = self.particles.len() as u32;

        for i in 0..count {
            // Simple pseudo-random values
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let rand1 = (seed % 1000) as f32 / 1000.0;
            
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let rand2 = (seed % 1000) as f32 / 1000.0;
            
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let rand3 = (seed % 1000) as f32 / 1000.0;
            
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let rand4 = (seed % 1000) as f32 / 1000.0;

            // Random velocity in all directions
            let angle = rand1 * std::f32::consts::PI * 2.0;
            let speed = rand2 * 100.0 + 50.0;
            let velocity = (angle.cos() * speed, angle.sin() * speed);

            self.particles.push(Particle {
                position: (x, y),
                velocity,
                color,
                size: rand3 * 5.0 + 2.0,
                lifetime: rand4 * 1.0 + 0.5,
                age: 0.0,
            });
        }
    }

    /// Update particles
    pub fn update(&mut self, dt: Duration) {
        let dt_secs = dt.as_secs_f32();

        for particle in &mut self.particles {
            // Update position
            particle.position.0 += particle.velocity.0 * dt_secs;
            particle.position.1 += particle.velocity.1 * dt_secs;

            // Update age
            particle.age += dt_secs;

            // Fade out based on age
            let life_fraction = particle.age / particle.lifetime;
            if life_fraction >= 1.0 {
                particle.size = 0.0; // Mark for removal
            }
        }

        // Remove dead particles
        self.particles.retain(|p| p.age < p.lifetime);
    }

    /// Get particles
    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    /// Get mutable particles
    pub fn particles_mut(&mut self) -> &mut [Particle] {
        &mut self.particles
    }
}

/// PLY-based shader effects
pub struct PlyShaderEffects {
    /// Glow renderer
    glow: PlyGlowRenderer,
    /// Background renderer
    background: PlyBackgroundRenderer,
    /// Particle renderer
    particles: PlyParticleRenderer,
}

impl PlyShaderEffects {
    /// Create a new PLY shader effects manager
    pub fn new() -> Self {
        Self {
            glow: PlyGlowRenderer::new(100_000),
            background: PlyBackgroundRenderer::new(),
            particles: PlyParticleRenderer::new(10_000),
        }
    }

    /// Update all effects
    pub fn update(&mut self, dt: Duration) {
        self.glow.update(dt);
        self.background.update(dt);
        self.particles.update(dt);
    }

    /// Get glow renderer
    pub fn glow(&self) -> &PlyGlowRenderer {
        &self.glow
    }

    /// Get mutable glow renderer
    pub fn glow_mut(&mut self) -> &mut PlyGlowRenderer {
        &mut self.glow
    }

    /// Get background renderer
    pub fn background(&self) -> &PlyBackgroundRenderer {
        &self.background
    }

    /// Get mutable background renderer
    pub fn background_mut(&mut self) -> &mut PlyBackgroundRenderer {
        &mut self.background
    }

    /// Get particle renderer
    pub fn particles(&self) -> &PlyParticleRenderer {
        &self.particles
    }

    /// Get mutable particle renderer
    pub fn particles_mut(&mut self) -> &mut PlyParticleRenderer {
        &mut self.particles
    }

    /// Clear all effects
    pub fn clear_all(&mut self) {
        self.glow.clear();
        self.particles.clear();
    }
}

impl Default for PlyShaderEffects {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glow_renderer() {
        let mut glow = PlyGlowRenderer::new(100);
        assert_eq!(glow.instances().len(), 0);

        glow.push(60, (255, 0, 0), 100.0, 200.0, 10.0, Duration::from_secs(1));
        assert_eq!(glow.instances().len(), 1);

        glow.update(Duration::from_millis(100));
        assert_eq!(glow.instances().len(), 1); // Should still be there

        glow.clear();
        assert_eq!(glow.instances().len(), 0);
    }

    #[test]
    fn test_background_renderer() {
        let mut bg = PlyBackgroundRenderer::new();
        assert_eq!(bg.time(), 10.0);

        bg.update(Duration::from_secs(1));
        assert_eq!(bg.time(), 11.0);

        bg.set_speed(2.0);
        bg.update(Duration::from_secs(1));
        assert_eq!(bg.time(), 13.0);
    }

    #[test]
    fn test_particle_renderer() {
        let mut particles = PlyParticleRenderer::new(100);
        assert_eq!(particles.particles().len(), 0);

        particles.spawn(100.0, 200.0, Color::rgb(255.0, 0.0, 0.0), 10);
        assert_eq!(particles.particles().len(), 10);

        particles.update(Duration::from_millis(100));
        assert!(particles.particles().len() <= 10); // Some may have died

        particles.clear();
        assert_eq!(particles.particles().len(), 0);
    }

    #[test]
    fn test_shader_effects() {
        let mut effects = PlyShaderEffects::new();
        
        effects.glow_mut().push(60, (255, 0, 0), 100.0, 200.0, 10.0, Duration::from_secs(1));
        effects.particles_mut().spawn(100.0, 200.0, Color::rgb(0.0, 255.0, 0.0), 5);

        effects.update(Duration::from_millis(100));

        assert_eq!(effects.glow().instances().len(), 1);
        assert_eq!(effects.particles().particles().len(), 5);

        effects.clear_all();
        assert_eq!(effects.glow().instances().len(), 0);
        assert_eq!(effects.particles().particles().len(), 0);
    }
}
