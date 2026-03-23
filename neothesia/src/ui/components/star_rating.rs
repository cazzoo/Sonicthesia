use macroquad::prelude::*;
use neothesia_core::design::{colors, radius, sizes};

pub struct StarRating {
    pub x: f32,
    pub y: f32,
    pub rating: u8,
    pub max_stars: u8,
    pub star_size: f32,
    pub gap: f32,
}

impl StarRating {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            rating: 0,
            max_stars: 5,
            star_size: 16.0,
            gap: 2.0,
        }
    }

    pub fn rating(mut self, rating: u8) -> Self {
        self.rating = rating.min(self.max_stars);
        self
    }

    pub fn max_stars(mut self, max: u8) -> Self {
        self.max_stars = max;
        self
    }

    pub fn star_size(mut self, size: f32) -> Self {
        self.star_size = size;
        self
    }

    pub fn render(&self) {
        let (filled_r, filled_g, filled_b) = colors::to_normalized(colors::PRIMARY);
        let (empty_r, empty_g, empty_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);

        for i in 0..self.max_stars {
            let star_x = self.x + i as f32 * (self.star_size + self.gap);
            let is_filled = i < self.rating;

            let (r, g, b) = if is_filled {
                (filled_r, filled_g, filled_b)
            } else {
                (empty_r, empty_g, empty_b)
            };
            let alpha = if is_filled { 1.0 } else { 0.6 };

            draw_text(
                "★",
                star_x,
                self.y + self.star_size,
                self.star_size,
                Color::new(r, g, b, alpha),
            );
        }
    }

    pub fn width(&self) -> f32 {
        self.max_stars as f32 * self.star_size + (self.max_stars - 1) as f32 * self.gap
    }

    pub fn height(&self) -> f32 {
        self.star_size
    }
}

pub struct DifficultyStars {
    pub rating: StarRating,
    pub difficulty: u8,
}

impl DifficultyStars {
    pub fn new(x: f32, y: f32, difficulty: u8) -> Self {
        let stars = ((difficulty as f32 / 10.0) * 5.0).ceil() as u8;

        Self {
            rating: StarRating::new(x, y).rating(stars),
            difficulty,
        }
    }

    pub fn render(&self) {
        self.rating.render();
    }

    pub fn difficulty_color(&self) -> (f32, f32, f32) {
        match self.difficulty {
            1..=3 => colors::to_normalized((80, 180, 112)),
            4..=7 => colors::to_normalized((255, 193, 7)),
            8..=10 => colors::to_normalized(colors::ERROR),
            _ => colors::to_normalized(colors::ON_SURFACE_VARIANT),
        }
    }
}
