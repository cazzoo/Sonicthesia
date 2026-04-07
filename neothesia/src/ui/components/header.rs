use macroquad::prelude::*;
use neothesia_core::design::{colors, spacing};

use crate::scene::ply_fonts;
use crate::virtual_resolution::{vh, vw};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavItem {
    Back,
    Library,
    FreePlay,
    Practice,
    Settings,
}

pub struct Header {
    pub active_nav: NavItem,
    pub height: f32,
    pub show_back_button: bool,
}

impl Header {
    pub fn new() -> Self {
        Self {
            active_nav: NavItem::Library,
            height: 64.0,
            show_back_button: false,
        }
    }

    pub fn set_active_nav(&mut self, nav: NavItem) {
        self.active_nav = nav;
    }

    pub fn render(&mut self, mx: f32, my: f32, mouse_pressed: bool) -> Option<NavItem> {
        let sw = vw();
        let _sh = vh();

        let (bg_r, bg_g, bg_b) = colors::to_normalized(colors::SURFACE_CONTAINER_LOW);
        draw_rectangle(0.0, 0.0, sw, self.height, Color::new(bg_r, bg_g, bg_b, 1.0));

        let (primary_r, primary_g, primary_b) = colors::to_normalized(colors::PRIMARY);
        draw_rectangle(
            0.0,
            self.height - 1.0,
            sw,
            1.0,
            Color::new(primary_r, primary_g, primary_b, 0.15),
        );

        let mut clicked_nav = None;

        let mut logo_x = spacing::XL;

        if self.show_back_button {
            let btn_x = 8.0;
            let btn_y = self.height / 2.0 - 14.0;
            let btn_size = 28.0;

            let is_back_hovered =
                mx >= btn_x && mx <= btn_x + btn_size && my >= btn_y && my <= btn_y + btn_size;

            let (btn_r, btn_g, btn_b) = colors::to_normalized(colors::SURFACE_CONTAINER_HIGHEST);
            if is_back_hovered {
                draw_rectangle(
                    btn_x,
                    btn_y,
                    btn_size,
                    btn_size,
                    Color::new(btn_r, btn_g, btn_b, 0.6),
                );
            }

            let (icon_r, icon_g, icon_b) = colors::to_normalized(colors::ON_SURFACE);
            ply_fonts::draw_body(
                "←",
                btn_x + 8.0,
                btn_y + 19.0,
                18.0,
                Color::new(icon_r, icon_g, icon_b, 1.0),
            );

            if is_back_hovered && mouse_pressed {
                clicked_nav = Some(NavItem::Back);
            }

            logo_x = btn_x + btn_size + 8.0;
        }

        let logo_y = self.height / 2.0 + 8.0;
        let (text_r, text_g, text_b) = colors::to_normalized(colors::ON_SURFACE);
        ply_fonts::draw_headline(
            "Sonic Obsidian Piano",
            logo_x,
            logo_y,
            24.0,
            Color::new(text_r, text_g, text_b, 1.0),
        );

        let nav_x = logo_x + 280.0;
        let nav_y = self.height / 2.0 + 6.0;
        let nav_items = [
            (NavItem::Library, "Library"),
            (NavItem::FreePlay, "🎹 Free Play"),
            (NavItem::Settings, "Settings"),
        ];

        let mut current_x = nav_x;
        for (nav, label) in nav_items.iter() {
            let is_active = self.active_nav == *nav;
            let text_width = measure_text(label, ply_fonts::body_font(), 16, 1.0).width;
            let is_hovered = mx >= current_x
                && mx <= current_x + text_width + 16.0
                && my >= nav_y - 24.0
                && my <= nav_y + 8.0;

            let (label_r, label_g, label_b) = if is_active {
                colors::to_normalized(colors::PRIMARY)
            } else if is_hovered {
                colors::to_normalized(colors::ON_SURFACE)
            } else {
                colors::to_normalized(colors::ON_SURFACE_VARIANT)
            };

            ply_fonts::draw_body(
                label,
                current_x,
                nav_y,
                16.0,
                Color::new(label_r, label_g, label_b, 1.0),
            );

            if is_active {
                draw_rectangle(
                    current_x,
                    nav_y + 6.0,
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
        let search_y = self.height / 2.0 - 14.0;
        let search_w = 256.0;
        let search_h = 28.0;

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
        ply_fonts::draw_body(
            "Search song library...",
            search_x + 12.0,
            search_y + 18.0,
            12.0,
            Color::new(icon_r, icon_g, icon_b, 0.6),
        );

        let avatar_x = sw - 100.0;
        let avatar_y = self.height / 2.0 - 16.0;
        let avatar_size = 32.0;

        let is_avatar_hovered = mx >= avatar_x
            && mx <= avatar_x + avatar_size
            && my >= avatar_y
            && my <= avatar_y + avatar_size;

        let (avatar_r, avatar_g, avatar_b) = colors::to_normalized(colors::PRIMARY);
        let avatar_alpha = if is_avatar_hovered { 0.2 } else { 0.0 };
        draw_circle(
            avatar_x + avatar_size / 2.0,
            avatar_y + avatar_size / 2.0,
            avatar_size / 2.0,
            Color::new(avatar_r, avatar_g, avatar_b, avatar_alpha),
        );

        draw_circle_lines(
            avatar_x + avatar_size / 2.0,
            avatar_y + avatar_size / 2.0,
            avatar_size / 2.0,
            1.0,
            Color::new(avatar_r, avatar_g, avatar_b, 0.5),
        );

        ply_fonts::draw_body(
            "👤",
            avatar_x + 9.0,
            avatar_y + 22.0,
            16.0,
            Color::new(avatar_r, avatar_g, avatar_b, 1.0),
        );

        clicked_nav
    }

    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        y <= self.height
    }
}
