use macroquad::prelude::*;
use neothesia_core::design::{colors, effects, radius, sizes, spacing};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavItem {
    Library,
    Practice,
    Settings,
}

pub struct Header {
    pub active_nav: NavItem,
    pub search_query: String,
    pub is_search_focused: bool,
    pub height: f32,
}

impl Header {
    pub fn new() -> Self {
        Self {
            active_nav: NavItem::Library,
            search_query: String::new(),
            is_search_focused: false,
            height: 64.0,
        }
    }

    pub fn active_nav(mut self, nav: NavItem) -> Self {
        self.active_nav = nav;
        self
    }

    pub fn render(&mut self, mx: f32, my: f32, mouse_pressed: bool) -> Option<NavItem> {
        let sw = screen_width();

        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_LOW);
        draw_rectangle(0.0, 0.0, sw, self.height, Color::new(bg_r, bg_g, bg_b, 1.0));

        let (shadow_r, shadow_g, shadow_b) = colors::to_normalized(colors::PRIMARY);
        draw_rectangle(
            0.0,
            self.height - 1.0,
            sw,
            1.0,
            Color::new(shadow_r, shadow_g, shadow_b, 0.15),
        );

        let mut clicked_nav = None;

        let logo_x = spacing::XL;
        let logo_y = self.height / 2.0 + 8.0;
        let (text_r, text_g, text_b) = colors::to_normalized(colors::ON_SURFACE);
        draw_text(
            "Sonic Obsidian Piano",
            logo_x,
            logo_y,
            24.0,
            Color::new(text_r, text_g, text_b, 1.0),
        );

        let nav_x = logo_x + 280.0;
        let nav_y = self.height / 2.0;
        let nav_items = [
            (NavItem::Library, "Library"),
            (NavItem::Practice, "Practice"),
            (NavItem::Settings, "Settings"),
        ];

        let mut current_x = nav_x;
        for (nav, label) in nav_items.iter() {
            let is_active = self.active_nav == *nav;
            let text_width = measure_text(label, None, 16, 1.0).width;
            let is_hovered = mx >= current_x
                && mx <= current_x + text_width + 16.0
                && my >= nav_y - 20.0
                && my <= nav_y + 10.0;

            let (label_r, label_g, label_b) = if is_active {
                colors::to_normalized(colors::PRIMARY)
            } else if is_hovered {
                colors::to_normalized(colors::ON_SURFACE)
            } else {
                colors::to_normalized(colors::ON_SURFACE_VARIANT)
            };

            draw_text(
                label,
                current_x,
                nav_y,
                16.0,
                Color::new(label_r, label_g, label_b, 1.0),
            );

            if is_active {
                draw_rectangle(
                    current_x,
                    nav_y + 4.0,
                    text_width,
                    2.0,
                    Color::new(label_r, label_g, label_b, 1.0),
                );
            }

            if is_hovered && mouse_pressed {
                clicked_nav = Some(*nav);
            }

            current_x += text_width + 32.0;
        }

        let search_x = sw - 380.0;
        let search_y = self.height / 2.0 - 12.0;
        let search_w = 256.0;
        let search_h = 32.0;

        let (search_bg_r, search_bg_g, search_bg_b) =
            colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
        draw_rectangle(
            search_x,
            search_y,
            search_w,
            search_h,
            Color::new(search_bg_r, search_bg_g, search_bg_b, 1.0),
        );

        let (icon_r, icon_g, icon_b) = colors::to_normalized(colors::ON_SURFACE_VARIANT);
        draw_text(
            "🔍",
            search_x + 10.0,
            search_y + 22.0,
            14.0,
            Color::new(icon_r, icon_g, icon_b, 1.0),
        );

        if self.search_query.is_empty() && !self.is_search_focused {
            draw_text(
                "Search song library...",
                search_x + 36.0,
                search_y + 22.0,
                12.0,
                Color::new(icon_r, icon_g, icon_b, 0.6),
            );
        } else {
            draw_text(
                &self.search_query,
                search_x + 36.0,
                search_y + 22.0,
                12.0,
                Color::new(0.973, 0.961, 0.992, 1.0),
            );
        }

        let avatar_x = sw - 100.0;
        let avatar_y = self.height / 2.0 - 16.0;
        let avatar_size = 32.0;

        let is_avatar_hovered = mx >= avatar_x
            && mx <= avatar_x + avatar_size
            && my >= avatar_y
            && my <= avatar_y + avatar_size;

        let (avatar_r, avatar_g, avatar_b) = colors::to_normalized(colors::PRIMARY);
        let avatar_alpha = if is_avatar_hovered { 0.2 } else { 0.0 };
        draw_rectangle(
            avatar_x,
            avatar_y,
            avatar_size,
            avatar_size,
            Color::new(avatar_r, avatar_g, avatar_b, avatar_alpha),
        );

        draw_text(
            "👤",
            avatar_x + 6.0,
            avatar_y + 24.0,
            20.0,
            Color::new(avatar_r, avatar_g, avatar_b, 1.0),
        );

        clicked_nav
    }

    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        y <= self.height
    }
}
