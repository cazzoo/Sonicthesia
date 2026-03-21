use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ScreenShake {
    pub intensity: f32,
    pub duration: Duration,
    pub elapsed: Duration,
    pub frequency: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl ScreenShake {
    pub fn new(intensity: f32, duration: Duration, frequency: f32) -> Self {
        Self {
            intensity,
            duration,
            elapsed: Duration::ZERO,
            frequency,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }

    pub fn small() -> Self {
        Self::new(0.3, Duration::from_millis(100), 20.0)
    }

    pub fn medium() -> Self {
        Self::new(0.5, Duration::from_millis(200), 25.0)
    }

    pub fn large() -> Self {
        Self::new(0.8, Duration::from_millis(300), 30.0)
    }

    pub fn update(&mut self, dt: Duration) {
        if self.elapsed >= self.duration {
            self.offset_x = 0.0;
            self.offset_y = 0.0;
            return;
        }

        self.elapsed += dt;

        let progress = self.elapsed.as_secs_f32() / self.duration.as_secs_f32();
        let decay = 1.0 - progress;

        self.offset_x =
            (self.elapsed.as_secs_f32() * self.frequency).sin() * self.intensity * decay;
        self.offset_y =
            (self.elapsed.as_secs_f32() * self.frequency * 1.3).cos() * self.intensity * decay;
    }

    pub fn is_active(&self) -> bool {
        self.elapsed < self.duration
    }

    pub fn offset(&self) -> (f32, f32) {
        (self.offset_x, self.offset_y)
    }

    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.offset_x = 0.0;
        self.offset_y = 0.0;
    }
}

impl Default for ScreenShake {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            duration: Duration::ZERO,
            elapsed: Duration::ZERO,
            frequency: 20.0,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScreenFlash {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub intensity: f32,
    pub duration: Duration,
    pub elapsed: Duration,
}

impl ScreenFlash {
    pub fn new(r: f32, g: f32, b: f32, intensity: f32, duration: Duration) -> Self {
        Self {
            r,
            g,
            b,
            intensity,
            duration,
            elapsed: Duration::ZERO,
        }
    }

    pub fn gold(intensity: f32) -> Self {
        Self::new(1.0, 0.84, 0.0, intensity, Duration::from_millis(200))
    }

    pub fn green(intensity: f32) -> Self {
        Self::new(0.0, 1.0, 0.0, intensity, Duration::from_millis(150))
    }

    pub fn red(intensity: f32) -> Self {
        Self::new(1.0, 0.0, 0.0, intensity, Duration::from_millis(100))
    }

    pub fn update(&mut self, dt: Duration) {
        if self.elapsed < self.duration {
            self.elapsed += dt;
        }
    }

    pub fn is_active(&self) -> bool {
        self.elapsed < self.duration
    }

    pub fn current_alpha(&self) -> f32 {
        if !self.is_active() {
            return 0.0;
        }

        let progress = self.elapsed.as_secs_f32() / self.duration.as_secs_f32();
        self.intensity * (1.0 - progress)
    }

    pub fn color(&self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.current_alpha())
    }

    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
    }
}

impl Default for ScreenFlash {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            intensity: 0.0,
            duration: Duration::ZERO,
            elapsed: Duration::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimingFeedback {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub elapsed: Duration,
    pub duration: Duration,
    pub velocity_y: f32,
}

impl TimingFeedback {
    pub fn new(x: f32, y: f32, text: &str, r: f32, g: f32, b: f32) -> Self {
        Self {
            x,
            y,
            text: text.to_string(),
            r,
            g,
            b,
            elapsed: Duration::ZERO,
            duration: Duration::from_millis(500),
            velocity_y: -80.0,
        }
    }

    pub fn perfect(x: f32, y: f32) -> Self {
        Self::new(x, y, "PERFECT", 1.0, 0.84, 0.0)
    }

    pub fn good(x: f32, y: f32) -> Self {
        Self::new(x, y, "GOOD", 0.0, 1.0, 0.0)
    }

    pub fn okay(x: f32, y: f32) -> Self {
        Self::new(x, y, "OKAY", 0.0, 0.53, 1.0)
    }

    pub fn miss(x: f32, y: f32) -> Self {
        Self::new(x, y, "MISS", 1.0, 0.0, 0.0)
    }

    pub fn update(&mut self, dt: Duration) {
        self.elapsed += dt;
        self.y += self.velocity_y * dt.as_secs_f32();
    }

    pub fn is_active(&self) -> bool {
        self.elapsed < self.duration
    }

    pub fn alpha(&self) -> f32 {
        if !self.is_active() {
            return 0.0;
        }
        1.0 - (self.elapsed.as_secs_f32() / self.duration.as_secs_f32())
    }

    pub fn color(&self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.alpha())
    }
}

#[derive(Debug, Default)]
pub struct EffectsManager {
    shake: ScreenShake,
    flash: ScreenFlash,
    timing_feedbacks: Vec<TimingFeedback>,
}

impl EffectsManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn shake(&self) -> &ScreenShake {
        &self.shake
    }

    pub fn flash(&self) -> &ScreenFlash {
        &self.flash
    }

    pub fn timing_feedbacks(&self) -> &[TimingFeedback] {
        &self.timing_feedbacks
    }

    pub fn trigger_shake(&mut self, shake: ScreenShake) {
        self.shake = shake;
    }

    pub fn trigger_flash(&mut self, flash: ScreenFlash) {
        self.flash = flash;
    }

    pub fn add_timing_feedback(&mut self, feedback: TimingFeedback) {
        self.timing_feedbacks.push(feedback);
    }

    pub fn update(&mut self, dt: Duration) {
        self.shake.update(dt);
        self.flash.update(dt);

        for feedback in &mut self.timing_feedbacks {
            feedback.update(dt);
        }

        self.timing_feedbacks.retain(|f| f.is_active());
    }

    pub fn clear(&mut self) {
        self.shake = ScreenShake::default();
        self.flash = ScreenFlash::default();
        self.timing_feedbacks.clear();
    }
}
